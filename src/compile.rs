//! Compilación del .tex con pdflatex.

use std::ffi::{OsStr, OsString};
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::Duration;

use crate::encoding::{decode_latex_log, decode_process_output};
use crate::error::{GenerationError, Result};
use crate::latex::{summarize_latex_errors, unsupported_character_warnings};
use crate::logging;
use crate::process::{RunError, TOOL_TIMEOUT, run_with_timeout};
use crate::project::LOGO_FILES;

/// Un índice que cambia de longitud desplaza las páginas; se repite pdflatex
/// hasta que el `.toc` se estabiliza, con este máximo.
pub const MAX_LATEX_RUNS: usize = 4;

/// Copia al temporal lo que la plantilla carga por nombre: el preámbulo común
/// (`*.sty` de `common/`), los `.sty` que traiga la propia plantilla y los
/// demás archivos de la carpeta de logos. pdflatex corre dentro de ese temporal y los encuentra por nombre.
pub fn copy_template_assets(
    template: &Path,
    common_dir: &Path,
    format_sty: Option<&Path>,
    destination: &Path,
    logos: Option<&Path>,
) -> Result<()> {
    // El formato elegido se copia con un nombre fijo: el diseño lo carga como
    // `investigacion-format` sin saber cuál es.
    if let Some(format_sty) = format_sty {
        std::fs::copy(format_sty, destination.join("investigacion-format.sty"))?;
    }
    let template_dir = template.parent().unwrap_or(Path::new("."));
    for directory in [common_dir, template_dir] {
        for entry in std::fs::read_dir(directory).into_iter().flatten().flatten() {
            let path = entry.path();
            if path.extension().is_some_and(|ext| ext == "sty") && path.is_file() {
                std::fs::copy(&path, destination.join(entry.file_name()))?;
            }
        }
    }
    // Sin carpeta de logos no pasa nada: la portada los envuelve en \IfFileExists.
    // Los dos logos de la portada no: los deja `generate::copy_logos`, ya listos.
    if let Some(logos) = logos {
        for entry in std::fs::read_dir(logos)?.flatten() {
            let is_cover_logo =
                entry.path().file_stem().is_some_and(|stem| LOGO_FILES.iter().any(|l| stem == *l));
            if entry.path().is_file() && !is_cover_logo {
                std::fs::copy(entry.path(), destination.join(entry.file_name()))?;
            }
        }
    }
    Ok(())
}

/// TEXINPUTS que busca primero en `directories`, en orden. El separador final
/// (entrada vacía) significa «y además las rutas por omisión»; sin él pdflatex
/// no encontraría ni sus propios paquetes.
pub fn latex_search_path(directories: &[PathBuf]) -> Result<OsString> {
    let mut paths = Vec::new();
    for directory in directories {
        // Una carpeta que `join_paths` no acepta (con `:` en Linux y macOS; en
        // Windows `;` se entrecomilla) no cabe en TEXINPUTS: se omite y sigue. Al usuario ya lo avisó Pandoc,
        // que copia a la caché las imágenes de esa carpeta.
        if std::env::join_paths([directory]).is_err() {
            logging::warn(format_args!(
                "TEXINPUTS: skipped {} (its name has the path separator)",
                directory.display()
            ));
            continue;
        }
        paths.push(directory.clone());
    }
    if let Some(previous) = std::env::var_os("TEXINPUTS") {
        paths.extend(std::env::split_paths(&previous).filter(|p| !p.as_os_str().is_empty()));
    }
    paths.push(PathBuf::new());
    std::env::join_paths(paths)
        .map_err(|e| GenerationError::new(tr!(es: "TEXINPUTS no válido: {e}", en: "Invalid TEXINPUTS: {e}")))
}

/// Guarda el .tex y el .log fuera del temporal (que se borra) para revisarlos.
pub fn keep_failure_artifacts(tex_path: &Path, destination: &Path) -> Option<PathBuf> {
    let saved = copy_failure_artifacts(tex_path, destination);
    match &saved {
        Some(log) => logging::info(format_args!("failed build kept: {} and last-error.tex", log.display())),
        None => {
            logging::warn(format_args!("could not keep last-error.tex/.log in {}", destination.display()))
        }
    }
    saved
}

fn copy_failure_artifacts(tex_path: &Path, destination: &Path) -> Option<PathBuf> {
    std::fs::create_dir_all(destination).ok()?;
    let mut saved = None;
    for extension in ["tex", "log"] {
        let mut source = tex_path.with_extension(extension);
        // Si pdflatex no pudo abrir el .tex, su registro se llama `texput.log`.
        if !source.is_file() && extension == "log" {
            source = tex_path.with_file_name("texput.log");
        }
        if source.is_file() {
            let copy = destination.join(format!("last-error.{extension}"));
            std::fs::copy(&source, &copy).ok()?;
            if extension == "log" {
                saved = Some(copy);
            }
        }
    }
    saved
}

/// Si el pdflatex del PATH es el de MiKTeX. MiKTeX recién instalado pregunta en
/// una ventana antes de bajar cada paquete que falta, y pdflatex se queda
/// esperando; con `--enable-installer` los instala solo. TeX Live no conoce esa
/// opción, por eso se pregunta una vez por proceso.
fn is_miktex() -> bool {
    static MIKTEX: std::sync::OnceLock<bool> = std::sync::OnceLock::new();
    *MIKTEX.get_or_init(|| {
        let mut command = Command::new("pdflatex");
        command.arg("--version");
        let miktex = run_with_timeout(command, None, std::time::Duration::from_secs(30))
            .is_ok_and(|output| String::from_utf8_lossy(&output.stdout).contains("MiKTeX"));
        logging::info(format_args!("pdflatex distribution: {}", if miktex { "MiKTeX" } else { "other" }));
        miktex
    })
}

/// Archivos que pdflatex escribe y relee en la siguiente pasada.
const STATE_EXTENSIONS: [&str; 2] = ["aux", "toc"];

/// Quita `last-error.tex` y `last-error.log` de la carpeta de salida. Que no
/// existan no es un problema; si no se pueden borrar, solo se avisa en el registro.
fn remove_last_error(output_dir: &Path) {
    for extension in ["tex", "log"] {
        let path = output_dir.join(format!("last-error.{extension}"));
        match std::fs::remove_file(&path) {
            Ok(()) => logging::info(format_args!("removed the old {}", path.display())),
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
            Err(error) => logging::warn(format_args!("could not remove {}: {error}", path.display())),
        }
    }
}

/// Compila las veces necesarias para estabilizar índice y referencias.
/// `search_dirs` son las carpetas donde pdflatex busca imágenes y diagramas.
///
/// `state_dir` guarda el `.aux` y el `.toc` de la generación anterior del
/// mismo trabajo: si el documento no cambió de estructura, la primera pasada
/// ya lee el índice correcto y basta con una sola. Si ese estado viejo hiciera
/// fallar la compilación, se descarta y se compila desde cero.
pub fn compile_pdf(
    tex_path: &Path,
    output_pdf: &Path,
    search_dirs: &[PathBuf],
    state_dir: Option<&Path>,
    on_warning: &mut dyn FnMut(String),
) -> Result<()> {
    compile(tex_path, output_pdf, search_dirs, state_dir, TOOL_TIMEOUT, on_warning)
}

/// `compile_pdf` con el límite de cada pasada aparte, para probarlo sin esperar 180 s.
fn compile(
    tex_path: &Path,
    output_pdf: &Path,
    search_dirs: &[PathBuf],
    state_dir: Option<&Path>,
    timeout: Duration,
    on_warning: &mut dyn FnMut(String),
) -> Result<()> {
    // Se calcula una sola vez para las dos ejecuciones posibles.
    let mut directories = vec![PathBuf::from(".")];
    directories.extend(search_dirs.iter().map(|d| crate::project::absolute(d)));
    let texinputs = latex_search_path(&directories)?;
    let restored = state_dir.and_then(|dir| restore_state(dir, tex_path));
    match (state_dir, &restored) {
        (Some(dir), Some(_)) => logging::info(format_args!("latex state: cache hit ({})", dir.display())),
        (Some(dir), None) => logging::info(format_args!("latex state: cache miss ({})", dir.display())),
        (None, _) => logging::debug("latex state: not used"),
    }
    // El intento con estado previo no deja `last-error.*`: si falla se repite
    // desde cero, y solo un fallo de ese segundo intento es un error real. Un
    // tiempo agotado no se repite: serían otros tantos segundos para el mismo final.
    let first = run_passes(tex_path, output_pdf, &texinputs, restored.clone(), restored.is_none(), timeout);
    let log = match first {
        Err((error, false)) if restored.is_some() => {
            logging::warn(format_args!(
                "build with the cached latex state failed; retrying from scratch:\n{error}"
            ));
            for extension in STATE_EXTENSIONS {
                let _ = std::fs::remove_file(tex_path.with_extension(extension));
            }
            run_passes(tex_path, output_pdf, &texinputs, None, true, timeout).map_err(|(error, _)| error)?
        }
        other => other.map_err(|(error, _)| error)?,
    };

    let generated = tex_path.with_extension("pdf");
    if !generated.is_file() {
        return Err(GenerationError::new(tr!(
            es: "pdflatex terminó sin producir el PDF esperado.",
            en: "pdflatex finished without producing the expected PDF."
        )));
    }
    for warning in unsupported_character_warnings(&log) {
        on_warning(warning);
    }
    publish_built_pdf(tex_path, &generated, output_pdf, state_dir)
}

/// Pasa el PDF recién compilado a su destino. El estado de LaTeX se guarda
/// antes de copiar, para que sirva aunque el destino esté bloqueado. Si el PDF
/// no se puede escribir (abierto en un visor, típico de Windows), se deja una
/// copia en la carpeta de estado de este trabajo y el error dice dónde está.
fn publish_built_pdf(
    tex_path: &Path,
    generated: &Path,
    output_pdf: &Path,
    state_dir: Option<&Path>,
) -> Result<()> {
    if let Some(dir) = state_dir {
        // La caché es una ayuda: si no se puede guardar, no es un error.
        match save_state(dir, tex_path) {
            Ok(()) => logging::debug(format_args!("latex state saved in {}", dir.display())),
            Err(error) => logging::warn(format_args!("latex state not saved in {}: {error}", dir.display())),
        }
    }
    let copied =
        crate::project::replace_file(output_pdf, |temporary| std::fs::copy(generated, temporary).map(drop));
    let Err(error) = copied else {
        // Solo un éxito borra los restos de un fallo anterior en la carpeta de salida.
        if let Some(output_dir) = output_pdf.parent() {
            remove_last_error(output_dir);
        }
        return Ok(());
    };
    let Some(kept) = state_dir.and_then(|dir| keep_built_pdf(generated, dir, output_pdf).ok()) else {
        return Err(crate::generate::cannot_write(output_pdf, &error));
    };
    logging::error(format_args!(
        "PDF kept in {} because {} could not be written: {error}",
        kept.display(),
        output_pdf.display()
    ));
    Err(GenerationError::new(tr!(
        es: "No se pudo escribir en {}: {error}. El PDF recién compilado quedó en {}. Revisa los permisos o cierra el PDF si está abierto en otro programa y vuelve a generarlo.",
        en: "Could not write to {}: {error}. The new PDF was kept in {}. Check the permissions or close the PDF if another program has it open, and generate it again.",
        output_pdf.display(),
        kept.display()
    )))
}

/// Copia el PDF compilado a la carpeta de estado, con el nombre de la salida.
fn keep_built_pdf(generated: &Path, state_dir: &Path, output_pdf: &Path) -> std::io::Result<PathBuf> {
    std::fs::create_dir_all(state_dir)?;
    let kept = state_dir.join(output_pdf.file_name().unwrap_or_default());
    std::fs::copy(generated, &kept)?;
    Ok(kept)
}

/// Copia el estado guardado junto al .tex y devuelve el índice que contenía.
fn restore_state(state_dir: &Path, tex_path: &Path) -> Option<String> {
    let toc = std::fs::read(state_file(state_dir, tex_path, "toc")).ok()?;
    for extension in STATE_EXTENSIONS {
        std::fs::copy(state_file(state_dir, tex_path, extension), tex_path.with_extension(extension)).ok()?;
    }
    Some(String::from_utf8_lossy(&toc).into_owned())
}

/// El archivo de estado guardado, con el nombre del `.tex` de la generación.
fn state_file(state_dir: &Path, tex_path: &Path, extension: &str) -> PathBuf {
    state_dir.join(tex_path.with_extension(extension).file_name().unwrap_or_default())
}

fn save_state(state_dir: &Path, tex_path: &Path) -> std::io::Result<()> {
    std::fs::create_dir_all(state_dir)?;
    for extension in STATE_EXTENSIONS {
        let source = tex_path.with_extension(extension);
        let target = state_file(state_dir, tex_path, extension);
        // Un diseño sin índice no escribe `.toc`: se guarda vacío, que es lo
        // mismo que lee `run_passes`, y la caché sirve igual.
        if source.is_file() {
            std::fs::copy(source, target)?;
        } else {
            std::fs::write(target, "")?;
        }
    }
    Ok(())
}

/// Las pasadas de pdflatex. Sin estado previo, la primera va con `-draftmode`
/// (escribe `.aux` y `.toc` pero no el PDF, que es lo caro) porque nunca puede
/// ser la definitiva. Devuelve el log de la última pasada, o el error y si
/// fue por tiempo agotado.
fn run_passes(
    tex_path: &Path,
    output_pdf: &Path,
    texinputs: &OsStr,
    restored_toc: Option<String>,
    keep_artifacts: bool,
    timeout: Duration,
) -> std::result::Result<String, (GenerationError, bool)> {
    let temp_dir = tex_path.parent().unwrap_or(Path::new("."));
    // pdflatex corre dentro del temporal y recibe solo el nombre del .tex: una
    // ruta completa en la línea de órdenes se rompe con espacios o con el `~`
    // de las rutas cortas de Windows (`C:\Users\USUARI~1\...`), que TeX lee
    // como carácter activo. Las imágenes y diagramas se buscan por TEXINPUTS.
    let tex_name = tex_path.file_name().unwrap_or(tex_path.as_os_str());
    let log_path = tex_path.with_extension("log");
    let toc_path = tex_path.with_extension("toc");
    let output_dir = output_pdf.parent().unwrap_or(Path::new("."));
    let failure_note = |keep: bool| {
        keep.then(|| keep_failure_artifacts(tex_path, output_dir))
            .flatten()
            .map(|log| tr!(es: "\n\nRegistro completo: {}", en: "\n\nFull log: {}", log.display()))
            .unwrap_or_default()
    };

    let has_state = restored_toc.is_some();
    let mut previous_toc = restored_toc;
    let mut log = String::new();
    for run in 1..=MAX_LATEX_RUNS {
        let mut command = Command::new("pdflatex");
        command.args(["-interaction=nonstopmode", "-halt-on-error", "-file-line-error"]);
        if is_miktex() {
            command.arg("--enable-installer");
        }
        let draft = run == 1 && !has_state;
        if draft {
            command.arg("-draftmode");
        }
        logging::info(format_args!("pdflatex pass {run}{}", if draft { " (draft mode)" } else { "" }));
        // Sin las claves PTEX.*: pdfTeX guardaría en el PDF la ruta absoluta de cada
        // PDF incluido (caché, carpeta personal). Va en la orden y no en la base para
        // cubrir cualquier diseño; `\ifdefined` por si el pdfTeX es anterior a 1.40.15.
        let mut input =
            OsString::from("\\ifdefined\\pdfsuppressptexinfo\\pdfsuppressptexinfo=-1\\fi\\input{");
        input.push(tex_name);
        input.push("}");
        command.arg(format!("-jobname={}", tex_path.file_stem().unwrap_or_default().to_string_lossy()));
        command.arg(input).current_dir(temp_dir).env("TEXINPUTS", texinputs);

        let started = std::time::Instant::now();
        let result = run_with_timeout(command, None, timeout).map_err(|e| match e {
            RunError::NotFound => (GenerationError::new(tr!(
                es: "No se encontró pdflatex. Instala TeX Live (o MiKTeX) y comprueba que esté en el PATH.",
                en: "pdflatex was not found. Install TeX Live (or MiKTeX) and make sure it is on the PATH."
            )), false),
            // Como no se repite, el tiempo agotado siempre deja `last-error.*`.
            RunError::Timeout => (GenerationError::new(tr!(
                es: "pdflatex siguió trabajando más de {} segundos y se detuvo; busca en el documento \
                     algo que LaTeX no pueda componer. Con MiKTeX, la primera generación puede tardar \
                     mientras descarga paquetes: vuelve a intentarlo.{}",
                en: "pdflatex kept working for more than {} seconds and was stopped; look in the document \
                     for something LaTeX cannot typeset. With MiKTeX, the first build can take long while it \
                     downloads packages: try again.{}",
                timeout.as_secs(),
                failure_note(true)
            )), true),
            RunError::Io(error) => (error.into(), false),
        })?;

        crate::process::report_timing(&format!("pdflatex pass {run}"), started);
        log = std::fs::read(&log_path).map(|raw| decode_latex_log(&raw)).unwrap_or_default();
        if log.is_empty() {
            log = decode_process_output(&result.stdout);
        }
        if !result.status.success() {
            let mut detail = summarize_latex_errors(&log);
            if detail.is_empty() {
                let text = match log.trim() {
                    "" => decode_process_output(&result.stderr).trim().to_owned(),
                    trimmed => trimmed.to_owned(),
                };
                let start = text.char_indices().rev().nth(1999).map_or(0, |(i, _)| i);
                detail = text[start..].to_owned();
            }
            return Err((
                GenerationError::new(tr!(
                    es: "Falló la compilación de LaTeX:\n{detail}{}",
                    en: "The LaTeX build failed:\n{detail}{}",
                    failure_note(keep_artifacts)
                )),
                false,
            ));
        }

        let current_toc = std::fs::read(&toc_path)
            .map(|raw| String::from_utf8_lossy(&raw).into_owned())
            .unwrap_or_default();
        let needs_rerun = log.contains("Rerun to get")
            || log.contains("Rerun LaTeX")
            || previous_toc.as_deref() != Some(current_toc.as_str());
        previous_toc = Some(current_toc);
        if (run > 1 || has_state) && !needs_rerun {
            logging::info(format_args!("latex: stable after pass {run}"));
            break;
        }
        if run == MAX_LATEX_RUNS {
            logging::warn(format_args!("latex: the table of contents still changed after {run} passes"));
        } else if needs_rerun {
            logging::debug("latex: rerun needed (the .toc changed or LaTeX asked for it)");
        }
    }
    Ok(log)
}

#[cfg(test)]
mod tests {
    use super::*;

    // Un directorio donde debe ir el PDF hace fallar la copia, como un PDF abierto.
    #[test]
    fn a_locked_pdf_keeps_the_build_in_the_state_folder() {
        let directory = tempfile::tempdir().unwrap();
        let base = directory.path();
        let generated = base.join("trabajo.pdf");
        std::fs::write(&generated, b"%PDF-1.5 nuevo").unwrap();
        let tex = base.join("trabajo.tex");
        std::fs::write(&tex, "x").unwrap();
        let output = base.join("salida").join("tarea.pdf");
        std::fs::create_dir_all(&output).unwrap();
        let state = base.join("cache");

        let error = publish_built_pdf(&tex, &generated, &output, Some(&state)).unwrap_err();

        let kept = state.join("tarea.pdf");
        assert_eq!(std::fs::read(&kept).unwrap(), b"%PDF-1.5 nuevo");
        assert!(error.0.contains(&kept.display().to_string()));
        assert!(state.join("trabajo.toc").is_file(), "el estado de LaTeX se guarda antes de copiar");
    }

    #[test]
    fn a_locked_pdf_without_state_folder_is_a_plain_write_error() {
        let directory = tempfile::tempdir().unwrap();
        let base = directory.path();
        let generated = base.join("trabajo.pdf");
        std::fs::write(&generated, b"%PDF-1.5").unwrap();
        let output = base.join("tarea.pdf");
        std::fs::create_dir_all(&output).unwrap();

        let error = publish_built_pdf(&base.join("trabajo.tex"), &generated, &output, None).unwrap_err();
        assert!(error.0.contains(&output.display().to_string()));
        assert!(!error.0.contains("quedó en"));
    }

    #[test]
    fn a_successful_build_removes_the_old_last_error_files() {
        let directory = tempfile::tempdir().unwrap();
        let output = directory.path();
        std::fs::write(output.join("last-error.tex"), "viejo").unwrap();
        std::fs::write(output.join("last-error.log"), "viejo").unwrap();
        std::fs::write(output.join("tarea.pdf"), "pdf").unwrap();

        remove_last_error(output);

        assert!(!output.join("last-error.tex").exists());
        assert!(!output.join("last-error.log").exists());
        assert!(output.join("tarea.pdf").is_file());
        // Sin restos que borrar no es un error.
        remove_last_error(output);
    }

    // Con estado guardado, un pdflatex que se cuelga se corta una vez y no se
    // repite desde cero (serían el doble de minutos para el mismo error).
    #[test]
    fn a_timeout_is_not_retried_from_scratch() {
        if Command::new("pdflatex").arg("--version").output().is_err() {
            return;
        }
        let directory = tempfile::tempdir().unwrap();
        let base = directory.path();
        // MiKTeX recién instalado arma el formato en la primera llamada y tarda
        // más que el límite de la prueba sin llegar a escribir el .log: se le
        // da una compilación previa, sin límite corto, para medir solo el corte.
        let warm_up = base.join("previa.tex");
        std::fs::write(&warm_up, "\\documentclass{article}\\begin{document}x\\end{document}\n").unwrap();
        let _ = compile(&warm_up, &base.join("previa-final.pdf"), &[], None, TOOL_TIMEOUT, &mut |_| {});

        let tex = base.join("trabajo.tex");
        // `\x` se llama a sí mismo para siempre.
        std::fs::write(&tex, "\\documentclass{article}\\begin{document}\\def\\x{\\x}\\x\\end{document}\n")
            .unwrap();
        let state = base.join("estado");
        std::fs::create_dir_all(&state).unwrap();
        for extension in STATE_EXTENSIONS {
            std::fs::write(state.join(format!("trabajo.{extension}")), "").unwrap();
        }
        let output = base.join("salida").join("trabajo.pdf");
        let started = std::time::Instant::now();
        let limit = Duration::from_secs(2);
        let error = compile(&tex, &output, &[], Some(&state), limit, &mut |_| {}).unwrap_err();
        assert!(started.elapsed() < limit * 2 - Duration::from_millis(300), "{:?}", started.elapsed());
        assert!(error.0.contains("last-error.log"), "{error}");
    }

    // `:` no cabe en Linux y macOS, `"` en Windows: se elige el que rechace este sistema.
    #[test]
    fn a_folder_with_the_path_separator_is_skipped() {
        let rejected =
            [':', '"'].into_iter().find(|c| std::env::join_paths([format!("a{c}b")]).is_err()).unwrap();
        let bad = PathBuf::from(format!("carpeta{rejected}separador"));
        let path = latex_search_path(&[PathBuf::from("."), bad.clone(), PathBuf::from("buena")]).unwrap();

        let entries: Vec<PathBuf> = std::env::split_paths(&path).collect();
        assert!(!entries.contains(&bad));
        assert!(entries.contains(&PathBuf::from("buena")));
    }
}
