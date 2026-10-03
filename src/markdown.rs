//! Lectura, limpieza, búsqueda y validación del Markdown de entrada.

use std::path::{Path, PathBuf};
use std::sync::LazyLock;

use regex::Regex;
use unicode_normalization::UnicodeNormalization;
use unicode_normalization::char::canonical_combining_class;

use crate::encoding::decode_text;
use crate::error::{GenerationError, Result};
use crate::project::absolute;

/// Encabezados que se recomiendan, ya normalizados y en este orden.
pub const REQUIRED_HEADINGS: [&str; 4] = ["introduccion", "desarrollo", "conclusion", "referencias"];
// Cómo se escriben en el trabajo (que va en español, sea cual sea la interfaz).
const HEADING_NAMES: [&str; 4] = ["Introducción", "Desarrollo", "Conclusión", "Referencias"];

// Espacios que parecen normales pero no lo son: duro, de figura, estrecho y de
// ancho cero. Word, Notion y las IA los sueltan a menudo.
const DISGUISED_SPACES: [char; 4] = ['\u{00A0}', '\u{2007}', '\u{202F}', '\u{200B}'];

static FENCE: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"^\s{0,3}(`{3,}|~{3,})").unwrap());
// Marcador de Markdown (encabezado, viñeta, lista numerada o cita) separado del
// texto por espacios invisibles en vez de por uno normal.
static MARKER_SEPARATOR: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new("^(\\s{0,3})(#{1,6}|[-*+]|\\d+[.)]|>)[\u{00A0}\u{2007}\u{202F}\u{200B}\t]+").unwrap()
});
static ATX_HEADING: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"^\s{0,3}#{1,6}\s+(.+?)\s*$").unwrap());

/// Líneas como `str.splitlines()` de Python para los finales habituales.
fn split_lines(text: &str) -> Vec<&str> {
    let mut lines: Vec<&str> = text.split('\n').map(|l| l.strip_suffix('\r').unwrap_or(l)).collect();
    if text.ends_with('\n') {
        lines.pop();
    }
    lines
}

/// Indica si alguna línea termina en un espacio disfrazado (también la que
/// parece vacía pero solo lleva uno). Sólo entonces se recorta el espacio
/// normal del final, que en un documento limpio es un salto de línea deliberado.
pub fn has_disguised_spaces(markdown: &str) -> bool {
    split_lines(markdown).iter().any(|line| {
        line.trim_end_matches([' ', '\t']).chars().last().is_some_and(|c| DISGUISED_SPACES.contains(&c))
    })
}

/// Quita el espacio invisible que impide a Pandoc ver párrafos, encabezados y
/// tablas. No toca el espacio duro dentro del texto ni los bloques ``` / ~~~.
pub fn normalize_markdown(markdown: &str) -> String {
    let dirty = has_disguised_spaces(markdown);
    let trailing = |c: char| DISGUISED_SPACES.contains(&c) || c == '\t' || (dirty && c == ' ');

    let mut lines = Vec::new();
    let mut fence: Option<char> = None;
    for line in split_lines(markdown) {
        if let Some(found) = FENCE.captures(line) {
            let marker = found[1].chars().next().unwrap();
            match fence {
                None => fence = Some(marker),
                Some(open) if open == marker => fence = None,
                Some(_) => {}
            }
            lines.push(line.to_owned());
            continue;
        }
        if fence.is_some() {
            lines.push(line.to_owned());
            continue;
        }
        let trimmed = line.trim_end_matches(trailing);
        lines.push(MARKER_SEPARATOR.replace(trimmed, "${1}${2} ").into_owned());
    }
    let normalized = lines.join("\n");
    if markdown.ends_with('\n') { normalized + "\n" } else { normalized }
}

/// Lee Markdown UTF-8 (o Windows-1252) y normaliza el espacio invisible.
pub fn read_markdown(path: &Path) -> Result<String> {
    let raw = std::fs::read(path)?;
    Ok(normalize_markdown(&decode_text(&raw, path)?))
}

/// Quita acentos y marcas combinantes (NFKD); lo usan los encabezados y el slug.
pub fn strip_accents(value: &str) -> String {
    value.nfkd().filter(|&c| canonical_combining_class(c) == 0).collect()
}

/// Normaliza un encabezado para validarlo sin depender de mayúsculas, acentos,
/// numeración ni de los `#` de cierre.
pub fn normalize_heading(heading: &str) -> String {
    static CLOSING: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"\s+#+\s*$").unwrap());
    static NUMBERING: LazyLock<Regex> =
        LazyLock::new(|| Regex::new(r"^\s*(?:\d+(?:\.\d+)*[.)]?\s*)").unwrap());
    static SPACES: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"\s+").unwrap());

    let plain = strip_accents(heading);
    let plain = CLOSING.replace(&plain, "");
    let plain = NUMBERING.replace(&plain, "");
    SPACES.replace_all(&plain, " ").trim().to_lowercase()
}

/// Encabezados ATX del documento, sin contar los que estén dentro de código.
pub fn markdown_headings(markdown: &str) -> Vec<String> {
    let mut headings = Vec::new();
    let mut fence: Option<char> = None;
    for line in split_lines(markdown) {
        if let Some(found) = FENCE.captures(line) {
            let marker = found[1].chars().next().unwrap();
            match fence {
                None => fence = Some(marker),
                Some(open) if open == marker => fence = None,
                Some(_) => {}
            }
            continue;
        }
        if fence.is_some() {
            continue;
        }
        if let Some(found) = ATX_HEADING.captures(line) {
            headings.push(normalize_heading(&found[1]));
        }
    }
    headings
}

/// Advertencias de estructura; nunca impiden generar el PDF.
pub fn validate_markdown(markdown: &str) -> Vec<String> {
    let headings = markdown_headings(markdown);
    let mut warnings = Vec::new();
    let mut found: Vec<(usize, &str)> = Vec::new();
    for (required, name) in REQUIRED_HEADINGS.into_iter().zip(HEADING_NAMES) {
        match headings.iter().position(|h| h == required) {
            Some(index) => found.push((index, name)),
            None => warnings.push(tr!(
                es: "Falta el encabezado recomendado «{name}».",
                en: "The recommended heading \"{name}\" is missing."
            )),
        }
    }
    let mut sorted = found.clone();
    sorted.sort();
    if found.len() > 1 && found != sorted {
        let order: Vec<&str> = found.iter().map(|(_, name)| *name).collect();
        let order = order.join(", ");
        warnings.push(tr!(
            es: "Los encabezados recomendados no van en el orden esperado: {order}.",
            en: "The recommended headings are not in the expected order: {order}."
        ));
    }
    warnings
}

/// Busca el Markdown tal cual, dentro de `input/` y, si solo se dio un nombre,
/// por nombre en todas las subcarpetas de `input/`. Una ruta que ya existe
/// siempre gana; dos archivos con el mismo nombre son un error.
pub fn find_markdown(path: &Path, input_dir: &Path) -> Result<PathBuf> {
    let candidate = expand_home(path);
    if candidate.exists() {
        return Ok(absolute(&candidate));
    }
    let inside = input_dir.join(&candidate);
    if inside.exists() {
        return Ok(absolute(&inside));
    }
    let bare_name = candidate.components().count() == 1;
    if !input_dir.is_dir() || !bare_name {
        return Ok(absolute(&candidate));
    }

    let mut matches = Vec::new();
    collect_named(input_dir, candidate.as_os_str(), &mut matches);
    matches.sort();
    match matches.len() {
        1 => Ok(absolute(&matches[0])),
        0 => Ok(absolute(&candidate)),
        _ => {
            let options: Vec<String> = matches
                .iter()
                .map(|m| m.strip_prefix(input_dir).unwrap_or(m).display().to_string())
                .collect();
            Err(GenerationError::new(tr!(
                es: "Hay varios archivos llamados {} en {}: {}. Indica cuál con su subcarpeta.",
                en: "There are several files named {} in {}: {}. Say which one with its subfolder.",
                candidate.display(),
                input_dir.display(),
                options.join(", ")
            )))
        }
    }
}

// Recorrido recursivo sencillo; las carpetas ilegibles se ignoran.
fn collect_named(directory: &Path, name: &std::ffi::OsStr, found: &mut Vec<PathBuf>) {
    let Ok(entries) = std::fs::read_dir(directory) else { return };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            collect_named(&path, name, found);
        } else if path.file_name() == Some(name) && path.is_file() {
            found.push(path);
        }
    }
}

/// Valida la ruta de entrada y devuelve su forma absoluta.
pub fn resolve_markdown_path(path: &Path, input_dir: &Path) -> Result<PathBuf> {
    let resolved = find_markdown(path, input_dir)?;
    if !resolved.exists() {
        return Err(GenerationError::new(tr!(
            es: "No existe el archivo Markdown: {}",
            en: "The Markdown file does not exist: {}",
            resolved.display()
        )));
    }
    if !resolved.is_file() {
        return Err(GenerationError::new(tr!(
            es: "La ruta indicada no es un archivo: {}",
            en: "The given path is not a file: {}",
            resolved.display()
        )));
    }
    let is_md = resolved.extension().is_some_and(|ext| ext.to_string_lossy().eq_ignore_ascii_case("md"));
    if !is_md {
        return Err(GenerationError::new(tr!(
            es: "El archivo de entrada debe tener la extensión .md.",
            en: "The input file must have the .md extension."
        )));
    }
    Ok(resolved)
}

/// `~/algo` → carpeta personal, como `Path.expanduser()`.
pub fn expand_home(path: &Path) -> PathBuf {
    let Ok(rest) = path.strip_prefix("~") else { return path.to_path_buf() };
    match std::env::var_os("HOME").or_else(|| std::env::var_os("USERPROFILE")) {
        Some(home) => PathBuf::from(home).join(rest),
        None => path.to_path_buf(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const NBSP: &str = "\u{00A0}";

    #[test]
    fn headings_are_normalized() {
        assert_eq!(normalize_heading("  2. Introducción  ###"), "introduccion");
    }

    #[test]
    fn missing_sections_are_warned() {
        assert_eq!(validate_markdown("# Introducción\n\nTexto").len(), 3);
    }

    #[test]
    fn headings_inside_code_blocks_are_ignored() {
        let markdown = "# Introducción\n\n```python\n# Desarrollo\n```\n\n# Conclusión\n";
        assert_eq!(markdown_headings(markdown), ["introduccion", "conclusion"]);
    }

    #[test]
    fn wrong_order_is_reported() {
        let warnings = validate_markdown("# Desarrollo\n\n# Introducción\n\n# Conclusión\n\n# Referencias\n");
        assert!(warnings.iter().any(|w| w.contains("orden")));
    }

    #[test]
    fn the_recommended_structure_is_accepted() {
        let markdown = "# Introducción\n\n# Desarrollo\n\n# Conclusión\n\n# Referencias\n";
        assert!(validate_markdown(markdown).is_empty());
    }

    #[test]
    fn a_line_with_only_a_hard_space_becomes_empty() {
        let markdown = format!("Parrafo uno.{NBSP}\n{NBSP} \nParrafo dos.\n");
        assert_eq!(normalize_markdown(&markdown), "Parrafo uno.\n\nParrafo dos.\n");
    }

    #[test]
    fn trailing_spaces_are_trimmed_in_a_dirty_document() {
        let markdown = format!("Texto.  \n{NBSP}\nOtro parrafo.\n");
        assert_eq!(normalize_markdown(&markdown), "Texto.\n\nOtro parrafo.\n");
    }

    #[test]
    fn a_clean_document_keeps_its_line_break() {
        assert_eq!(normalize_markdown("Texto.  \n"), "Texto.  \n");
    }

    #[test]
    fn a_hard_space_at_the_end_is_always_trimmed() {
        assert_eq!(normalize_markdown(&format!("Texto.{NBSP}\n")), "Texto.\n");
    }

    #[test]
    fn only_a_disguised_space_marks_the_document_as_dirty() {
        assert!(!has_disguised_spaces("Texto.  \nOtro.\t\n"));
        assert!(has_disguised_spaces(&format!("Texto.{NBSP}\n")));
        assert!(has_disguised_spaces(&format!("Parrafo.\n{NBSP}\nOtro.\n")));
    }

    #[test]
    fn markers_followed_by_a_hard_space_are_repaired() {
        assert_eq!(normalize_markdown(&format!("###{NBSP}1.7.3. Cotas\n")), "### 1.7.3. Cotas\n");
        let lists = format!("-{NBSP}item\n>{NBSP}cita\n1.{NBSP}uno\n");
        assert_eq!(normalize_markdown(&lists), "- item\n> cita\n1. uno\n");
    }

    #[test]
    fn a_hard_space_inside_the_text_is_preserved() {
        let markdown = format!("Fig.{NBSP}1 es la referencia\n");
        assert_eq!(normalize_markdown(&markdown), markdown);
    }

    #[test]
    fn code_blocks_are_left_untouched() {
        let markdown = "```python\ncode  \n```\n";
        assert_eq!(normalize_markdown(markdown), markdown);
    }

    #[test]
    fn headings_are_detected_after_normalizing() {
        let markdown = format!("#{NBSP}Introducción{NBSP}\n{NBSP}\nTexto.  \n{NBSP}\n#{NBSP}Conclusión\n");
        assert_eq!(markdown_headings(&normalize_markdown(&markdown)), ["introduccion", "conclusion"]);
    }

    #[test]
    fn read_markdown_normalizes_and_accepts_cp1252() {
        let directory = tempfile::tempdir().unwrap();
        let path = directory.path().join("trabajo.md");
        std::fs::write(&path, format!("# Título  \n{NBSP}\nTexto.\n")).unwrap();
        assert_eq!(read_markdown(&path).unwrap(), "# Título\n\nTexto.\n");
        std::fs::write(&path, b"# Introducci\xf3n\n\nInformaci\xf3n acad\xe9mica").unwrap();
        assert!(read_markdown(&path).unwrap().contains("Información"));
    }

    fn create_project(root: &Path, files: &[&str]) {
        for file in files {
            let path = root.join("input").join(file);
            std::fs::create_dir_all(path.parent().unwrap()).unwrap();
            std::fs::write(path, "# Introducción\n").unwrap();
        }
    }

    #[test]
    fn a_file_is_found_inside_input_and_its_subfolders() {
        let directory = tempfile::tempdir().unwrap();
        let root = directory.path();
        create_project(root, &["Actividad1.md", "IS/Tarea.md"]);
        let input = root.join("input");
        assert_eq!(
            resolve_markdown_path(Path::new("Actividad1.md"), &input).unwrap(),
            input.join("Actividad1.md")
        );
        let expected = input.join("IS").join("Tarea.md");
        assert_eq!(resolve_markdown_path(Path::new("IS/Tarea.md"), &input).unwrap(), expected);
        assert_eq!(resolve_markdown_path(Path::new("Tarea.md"), &input).unwrap(), expected);
    }

    #[test]
    fn two_files_with_the_same_name_are_an_error() {
        let directory = tempfile::tempdir().unwrap();
        create_project(directory.path(), &["IS/Tarea.md", "IA/Tarea.md"]);
        let error =
            resolve_markdown_path(Path::new("Tarea.md"), &directory.path().join("input")).unwrap_err();
        assert!(error.0.contains(&format!("IA{}Tarea.md", std::path::MAIN_SEPARATOR)));
        assert!(error.0.contains(&format!("IS{}Tarea.md", std::path::MAIN_SEPARATOR)));
    }

    #[test]
    fn an_existing_path_wins_over_the_input_folder() {
        let directory = tempfile::tempdir().unwrap();
        let root = directory.path();
        create_project(root, &["trabajo.md"]);
        let loose = root.join("trabajo.md");
        std::fs::write(&loose, "# Introducción\n").unwrap();
        assert_eq!(resolve_markdown_path(&loose, &root.join("input")).unwrap(), loose);
    }
}
