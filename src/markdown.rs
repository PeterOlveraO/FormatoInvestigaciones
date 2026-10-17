//! Lectura, limpieza, búsqueda y validación del Markdown de entrada.

use std::path::{Path, PathBuf};
use std::sync::LazyLock;

use regex::Regex;
use unicode_normalization::UnicodeNormalization;
use unicode_normalization::char::canonical_combining_class;

use crate::encoding::decode_text;
use crate::error::{GenerationError, Result};
use crate::project::absolute;

// Espacios que parecen normales pero no lo son: duro, de figura, estrecho y de
// ancho cero. Word, Notion y las IA los sueltan a menudo.
const DISGUISED_SPACES: [char; 4] = ['\u{00A0}', '\u{2007}', '\u{202F}', '\u{200B}'];

// Acepta espacios invisibles en la sangría para que `fenced_lines` coincida con
// lo que `normalize_markdown` produce después de quitarlos.
static FENCE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(&format!("^[ \\t{DISGUISED_CLASS}]{{0,3}}(`{{3,}}|~{{3,}})(.*)$")).unwrap());
// Marcador de Markdown (encabezado, viñeta, lista numerada o cita) separado del
// texto por espacios invisibles en vez de por uno normal.
static MARKER_SEPARATOR: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(&format!("^([ \\t]{{0,3}})(#{{1,6}}|[-*+]|\\d+[.)]|>)[{DISGUISED_CLASS}\\t]+")).unwrap()
});
// Un marcador al principio de la línea (también un cerco), para decidir si el
// espacio invisible de delante es de Markdown y debe quitarse.
// Una viñeta, número, almohadilla o cita solo cuenta si va seguida de espacio o
// del final de la línea; así «-5 grados» o «2020.Texto» no cambian.
static MARKER_START: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(&format!(
        "^[ \\t]{{0,3}}(?:(?:#{{1,6}}|[-*+]|\\d+[.)]|>)(?:[ \\t{DISGUISED_CLASS}]|$)|`{{3,}}|~{{3,}})"
    ))
    .unwrap()
});
static ATX_HEADING: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"^[ \t]{0,3}#{1,6}[ \t]+(.+?)[ \t]*$").unwrap());

/// Los mismos espacios invisibles, dentro de una clase de expresión regular.
const DISGUISED_CLASS: &str = "\u{00A0}\u{2007}\u{202F}\u{200B}";

/// Quita los espacios invisibles de delante de una línea solo si después viene
/// un marcador de Markdown (encabezado, lista, cita o cerco). En el resto de
/// líneas el espacio duro es texto y se deja como está.
fn strip_leading_disguised(line: &str) -> &str {
    if !line.starts_with(|c| DISGUISED_SPACES.contains(&c)) {
        return line;
    }
    let text = line.trim_start_matches(|c| DISGUISED_SPACES.contains(&c));
    if MARKER_START.is_match(text) { text } else { line }
}

/// Líneas como `str.splitlines()` de Python para los finales habituales.
fn split_lines(text: &str) -> Vec<&str> {
    let mut lines: Vec<&str> = text.split('\n').map(|l| l.strip_suffix('\r').unwrap_or(l)).collect();
    if text.ends_with('\n') {
        lines.pop();
    }
    lines
}

/// Cada línea con `true` si es parte de un bloque de código (``` o ~~~, con
/// sus cercos), y con `true` en el tercer campo si es un cerco. Como en
/// CommonMark, cierra un cerco del mismo carácter, al menos igual de largo y
/// sin texto detrás: un ```python dentro de un bloque de ```` sigue siendo código.
fn fenced_lines(text: &str) -> Vec<(&str, bool, bool)> {
    let mut open: Option<(char, usize)> = None;
    split_lines(text)
        .into_iter()
        .map(|line| {
            let fence = FENCE
                .captures(line)
                .map(|f| (f[1].chars().next().unwrap(), f[1].len(), f[2].trim().is_empty()));
            match (open, fence) {
                (None, Some((marker, len, _))) => {
                    open = Some((marker, len));
                    return (line, true, true);
                }
                (Some((marker, len)), Some((m, l, true))) if m == marker && l >= len => {
                    open = None;
                    return (line, true, true);
                }
                (None, None) => return (line, false, false),
                _ => {}
            }
            (line, true, false)
        })
        .collect()
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

    let lines: Vec<String> = fenced_lines(markdown)
        .into_iter()
        .map(|(line, code, fence)| match (code, fence) {
            // Un cerco con espacio invisible delante no lo reconoce Pandoc.
            (true, true) => strip_leading_disguised(line).to_owned(),
            (true, false) => line.to_owned(),
            (false, _) => {
                let line = strip_leading_disguised(line);
                MARKER_SEPARATOR.replace(line.trim_end_matches(trailing), "${1}${2} ").into_owned()
            }
        })
        .collect();
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
    fenced_lines(markdown)
        .into_iter()
        .filter(|(_, code, _)| !code)
        .filter_map(|(line, _, _)| ATX_HEADING.captures(line).map(|found| normalize_heading(&found[1])))
        .collect()
}

/// Advertencias de estructura; nunca impiden generar el PDF. Los
/// encabezados recomendados los pone el formato; sin ninguno, no hay avisos.
pub fn validate_markdown(markdown: &str, recommended: &[String]) -> Vec<String> {
    let headings = markdown_headings(markdown);
    let mut warnings = Vec::new();
    let mut found: Vec<(usize, &str)> = Vec::new();
    for name in recommended {
        let normalized = normalize_heading(name);
        match headings.iter().position(|h| *h == normalized) {
            Some(index) => found.push((index, name.as_str())),
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
        // `file_type` no sigue enlaces: un enlace a carpeta se salta para no
        // recorrerlo en círculo y un archivo enlazado se encuentra igual.
        if entry.file_type().is_ok_and(|kind| kind.is_dir()) {
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

    fn apa() -> Vec<String> {
        ["Introducción", "Desarrollo", "Conclusión", "Referencias"].map(String::from).to_vec()
    }

    #[test]
    fn without_recommended_headings_there_are_no_warnings() {
        assert!(validate_markdown("# Lo que sea\n", &[]).is_empty());
        assert_eq!(validate_markdown("# Abstract\n", &["Abstract".into(), "Methods".into()]).len(), 1);
    }

    #[test]
    fn missing_sections_are_warned() {
        assert_eq!(validate_markdown("# Introducción\n\nTexto", &apa()).len(), 3);
    }

    #[test]
    fn headings_inside_code_blocks_are_ignored() {
        let markdown = "# Introducción\n\n```python\n# Desarrollo\n```\n\n# Conclusión\n";
        assert_eq!(markdown_headings(markdown), ["introduccion", "conclusion"]);
        // Un ```python dentro de un bloque de ```` no lo cierra.
        let nested = "````markdown\n```python\n# Desarrollo\n```\n````\n# Conclusión\n";
        assert_eq!(markdown_headings(nested), ["conclusion"]);
    }

    #[test]
    fn wrong_order_is_reported() {
        let warnings =
            validate_markdown("# Desarrollo\n\n# Introducción\n\n# Conclusión\n\n# Referencias\n", &apa());
        assert!(warnings.iter().any(|w| w.contains("orden")));
    }

    #[test]
    fn the_recommended_structure_is_accepted() {
        let markdown = "# Introducción\n\n# Desarrollo\n\n# Conclusión\n\n# Referencias\n";
        assert!(validate_markdown(markdown, &apa()).is_empty());
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
    fn a_leading_hard_space_before_a_marker_is_removed() {
        // Antes el encabezado llegaba a Pandoc con el espacio delante y no lo veía.
        assert_eq!(normalize_markdown(&format!("{NBSP}# Introducción\n")), "# Introducción\n");
        assert_eq!(normalize_markdown(&format!("{NBSP}- uno\n")), "- uno\n");
        assert_eq!(normalize_markdown(&format!("{NBSP}```\n")), "```\n");
        assert_eq!(
            markdown_headings(&normalize_markdown(&format!("{NBSP}# Introducción\n"))),
            ["introduccion"]
        );
    }

    #[test]
    fn a_leading_hard_space_in_plain_text_is_kept() {
        assert_eq!(normalize_markdown(&format!("{NBSP}Texto\n")), format!("{NBSP}Texto\n"));
        // Números y signos que empiezan la línea, pero no son marcadores.
        for text in ["-5 grados", "2020.Texto", "+3"] {
            let markdown = format!("{NBSP}{text}\n");
            assert_eq!(normalize_markdown(&markdown), markdown);
        }
    }

    #[test]
    fn a_leading_hard_space_before_a_marker_followed_by_a_space_is_removed() {
        assert_eq!(normalize_markdown(&format!("{NBSP}- uno\n")), "- uno\n");
        assert_eq!(normalize_markdown(&format!("{NBSP}1. uno\n")), "1. uno\n");
    }

    #[test]
    fn a_leading_hard_space_inside_a_code_block_is_kept() {
        let markdown = format!("```\n{NBSP}# no es título\n```\n");
        assert_eq!(normalize_markdown(&markdown), markdown);
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
