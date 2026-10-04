//! Conversión Markdown → fragmento de LaTeX con Pandoc y los filtros Lua.

use std::ffi::OsString;
use std::path::{Path, PathBuf};
use std::process::Command;

use crate::encoding::decode_process_output;
use crate::error::{GenerationError, Result};
use crate::process::{RunError, TOOL_TIMEOUT, run_with_timeout};
use crate::project::Project;

/// Extensiones que el dialecto `markdown` de Pandoc no activa por omisión:
/// `==resaltado==`, `:emoji:` y URL sueltas convertidas en enlace.
pub const MARKDOWN_EXTENSIONS: [&str; 3] = ["mark", "emoji", "autolink_bare_uris"];

/// Filtros Lua, en el orden en que se aplican. Van embebidos en el binario
/// para no depender de dónde se instale; se escriben a un temporal al usarse.
pub const LUA_FILTERS: [(&str, &str); 6] = [
    ("inline_html", include_str!("../resources/filters/inline_html.lua")),
    ("images", include_str!("../resources/filters/images.lua")),
    ("diagrams", include_str!("../resources/filters/diagrams.lua")),
    ("charts", include_str!("../resources/filters/charts.lua")),
    ("blocks", include_str!("../resources/filters/blocks.lua")),
    // Al final: reescribe las tablas ya filtradas cuando el formato es a dos columnas.
    ("tables", include_str!("../resources/filters/tables.lua")),
];

/// Prefijo con el que los filtros marcan sus avisos en stderr.
pub const FILTER_WARNING_PREFIX: &str = "[investigacion]";
/// Pandoc no manda User-Agent y hay sitios que entonces responden 400.
pub const USER_AGENT: &str = "investigacion/0.2 (academic paper generator; pandoc)";

/// Separa los avisos de los filtros del resto de la salida de Pandoc.
pub fn filter_warnings(stderr: &str) -> Vec<String> {
    stderr
        .lines()
        .filter_map(|line| line.strip_prefix(FILTER_WARNING_PREFIX))
        .map(|warning| warning.trim().to_owned())
        .collect()
}

/// Ruta con barras normales: acaba dentro de `\includegraphics`, donde una
/// contrabarra de Windows empezaría un comando. TeX acepta `/` en todos lados.
pub fn posix(path: &Path) -> String {
    path.to_string_lossy().replace('\\', "/")
}

fn join_search_path(paths: &[PathBuf]) -> Result<OsString> {
    std::env::join_paths(paths).map_err(|e| {
        GenerationError::new(tr!(es: "Ruta de recursos no válida: {e}", en: "Invalid resource path: {e}"))
    })
}

/// Convierte el Markdown a un fragmento LaTeX (sin preámbulo).
///
/// Sin `allow_raw_latex` se desactiva `raw_tex`: una contrabarra suelta del
/// texto (`C:\Users`) llegaría a LaTeX como comando y rompería la compilación.
pub fn pandoc_to_latex(
    project: &Project,
    markdown_path: &Path,
    markdown: &str,
    allow_raw_latex: bool,
    doc_lang: crate::i18n::Lang,
    two_column: bool,
    on_warning: &mut dyn FnMut(String),
) -> Result<String> {
    let (remote, diagrams) = project.media_directories()?;
    let base_dir = markdown_path.parent().unwrap_or(Path::new(".")).to_path_buf();
    // Las imágenes se buscan junto al Markdown y en la caché del proyecto.
    let resources = vec![base_dir.clone(), project.cache_dir(), remote.clone(), diagrams.clone()];

    let filters_dir = tempfile::Builder::new().prefix("investigacion-filters-").tempdir()?;
    let mut format = String::from(if allow_raw_latex { "markdown" } else { "markdown-raw_tex" });
    for extension in MARKDOWN_EXTENSIONS {
        format.push('+');
        format.push_str(extension);
    }

    let mut command = Command::new("pandoc");
    command
        .arg("-")
        .arg(format!("--from={format}"))
        .arg("--to=latex")
        .arg("--wrap=none")
        .arg({
            let mut arg = OsString::from("--resource-path=");
            arg.push(join_search_path(&resources)?);
            arg
        })
        .arg(format!("--request-header=User-Agent:{USER_AGENT}"));
    for (name, source) in LUA_FILTERS {
        let path = filters_dir.path().join(format!("{name}.lua"));
        std::fs::write(&path, source)?;
        let mut arg = OsString::from("--lua-filter=");
        arg.push(&path);
        command.arg(arg);
    }
    command
        .current_dir(&base_dir)
        .env("INVESTIGACION_REMOTE_IMAGES", posix(&remote))
        .env("INVESTIGACION_DIAGRAMS", posix(&diagrams))
        // Una carpeta por línea: así una ruta puede llevar espacios.
        .env("INVESTIGACION_RESOURCES", resources.iter().map(|p| posix(p)).collect::<Vec<_>>().join("\n"))
        // Los filtros redactan sus avisos en el idioma de la interfaz.
        .env("INVESTIGACION_LANG", crate::i18n::current().code())
        // Y el texto que va dentro del PDF (títulos de las cajas), en el del documento.
        .env("INVESTIGACION_DOC_LANG", doc_lang.code())
        // longtable no funciona a dos columnas: el filtro `tables` lo resuelve.
        .env("INVESTIGACION_TWOCOLUMN", if two_column { "1" } else { "0" });

    let started = std::time::Instant::now();
    let output =
        run_with_timeout(command, Some(markdown.as_bytes().to_vec()), TOOL_TIMEOUT).map_err(|e| match e {
            RunError::NotFound => GenerationError::new(tr!(
                es: "No se encontró Pandoc. Instálalo y comprueba que esté en el PATH.",
                en: "Pandoc was not found. Install it and make sure it is on the PATH."
            )),
            RunError::Timeout => GenerationError::new(tr!(
                es: "Pandoc no respondió en {} segundos y se detuvo.",
                en: "Pandoc did not answer within {} seconds and was stopped.",
                TOOL_TIMEOUT.as_secs()
            )),
            RunError::Io(error) => error.into(),
        })?;

    crate::process::report_timing("pandoc", started);
    let stderr = decode_process_output(&output.stderr);
    if !output.status.success() {
        let detail = match stderr.trim() {
            "" => decode_process_output(&output.stdout).trim().to_owned(),
            text => text.to_owned(),
        };
        let suffix = if detail.is_empty() { ".".to_owned() } else { format!(": {detail}") };
        return Err(GenerationError::new(tr!(
            es: "Pandoc no pudo convertir el Markdown{suffix}",
            en: "Pandoc could not convert the Markdown{suffix}"
        )));
    }
    for warning in filter_warnings(&stderr) {
        on_warning(warning);
    }
    Ok(decode_process_output(&output.stdout).trim().to_owned())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn only_the_marked_lines_are_warnings() {
        let stderr =
            "[WARNING] Deprecated syntax\n[investigacion] No se pudo descargar la imagen x.png\notra linea\n";
        assert_eq!(filter_warnings(stderr), ["No se pudo descargar la imagen x.png"]);
        assert!(filter_warnings("").is_empty());
    }
}
