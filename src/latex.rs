//! Todo lo que habla con LaTeX como texto: escape, marcadores de la plantilla
//! y lectura del log de pdflatex.

use std::collections::BTreeSet;
use std::sync::LazyLock;

use regex::Regex;

use crate::document::DocumentData;
use crate::error::{GenerationError, Result};

pub const CONTENT_MARKER: &str = "%%CONTENIDO_MARKDOWN%%";

/// Escapa texto del usuario antes de insertarlo en LaTeX.
pub fn latex_escape(value: &str) -> String {
    let mut escaped = String::with_capacity(value.len());
    for c in value.chars() {
        match c {
            '\\' => escaped.push_str(r"\textbackslash{}"),
            '&' | '%' | '$' | '#' | '_' | '{' | '}' => {
                escaped.push('\\');
                escaped.push(c);
            }
            '~' => escaped.push_str(r"\textasciitilde{}"),
            '^' => escaped.push_str(r"\textasciicircum{}"),
            _ => escaped.push(c),
        }
    }
    escaped
}

/// Sustituye los marcadores `%%NOMBRE%%` de la plantilla. El contenido de
/// Pandoc no se escapa y se inserta al final, para que un `%%...%%` mencionado
/// en el trabajo no cuente como marcador pendiente.
pub fn render_template(template: &str, content: &str, data: &DocumentData) -> Result<String> {
    // Cada nombre se escapa por separado y luego se une con `\\`: escapar la
    // cadena ya unida convertiría ese salto en \textbackslash{}.
    let members = data.members.iter().map(|name| latex_escape(name)).collect::<Vec<_>>().join(r"\\");

    let replacements = [
        ("%%UNIVERSIDAD%%", latex_escape(&data.university)),
        ("%%FACULTAD%%", latex_escape(&data.faculty)),
        ("%%TITULO%%", latex_escape(&data.title)),
        ("%%ALUMNO%%", latex_escape(&data.student)),
        ("%%INTEGRANTES%%", members),
        ("%%MATERIA%%", latex_escape(&data.course)),
        ("%%GRUPO%%", latex_escape(&data.group)),
        ("%%DOCENTE%%", latex_escape(&data.teacher)),
        ("%%SEMESTRE%%", latex_escape(&data.semester)),
        ("%%FECHA_ENTREGA%%", latex_escape(&data.date)),
    ];
    let mut rendered = template.to_owned();
    for (marker, value) in &replacements {
        rendered = rendered.replace(marker, value);
    }

    static MARKER: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"%%[A-Z_]+%%").unwrap());
    let pending: BTreeSet<&str> =
        MARKER.find_iter(&rendered).map(|m| m.as_str()).filter(|m| *m != CONTENT_MARKER).collect();
    if !pending.is_empty() {
        let list: Vec<&str> = pending.into_iter().collect();
        return Err(GenerationError::new(format!(
            "The template has unreplaced markers: {}",
            list.join(", ")
        )));
    }
    if !rendered.contains(CONTENT_MARKER) {
        return Err(GenerationError::new(format!(
            "The template does not contain the {CONTENT_MARKER} marker."
        )));
    }
    Ok(rendered.replace(CONTENT_MARKER, content))
}

static FILE_LINE_ERROR: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"^.+?\.\w+:\d+: ").unwrap());

/// Extrae del log los errores reales (líneas `archivo:línea:` y `!`) y
/// descarta las rutas de paquetes; como máximo tres bloques.
pub fn summarize_latex_errors(log: &str) -> String {
    let lines: Vec<&str> = log.lines().collect();
    let mut blocks = Vec::new();
    for (index, line) in lines.iter().enumerate() {
        if !(FILE_LINE_ERROR.is_match(line) || line.starts_with("! ")) {
            continue;
        }
        let mut block = vec![line.trim().to_owned()];
        for extra in lines.iter().skip(index + 1).take(5) {
            let stripped = extra.trim();
            if stripped.is_empty() || FILE_LINE_ERROR.is_match(extra) || extra.starts_with("! ") {
                break;
            }
            block.push(stripped.to_owned());
            if stripped.starts_with("l.") {
                break;
            }
        }
        blocks.push(block.join("\n"));
        if blocks.len() == 3 {
            break;
        }
    }
    blocks.join("\n\n")
}

/// Los símbolos Unicode que la plantilla no supo componer (salen como [?]).
/// El texto «Caracter Unicode sin definir» lo escribe investigacion.sty.
pub fn unsupported_character_warnings(log: &str) -> Vec<String> {
    static UNDEFINED: LazyLock<Regex> =
        LazyLock::new(|| Regex::new(r"Caracter Unicode sin definir:\s*(\S+)\s*\(U\+([0-9A-F]+)\)").unwrap());
    let mut seen = BTreeSet::new();
    let mut warnings = Vec::new();
    for found in UNDEFINED.captures_iter(log) {
        let (character, codepoint) = (found[1].to_owned(), found[2].to_owned());
        if seen.insert((character.clone(), codepoint.clone())) {
            warnings.push(format!(
                "The symbol {character} (U+{codepoint}) could not be typeset and shows as [?] in the PDF. \
                 Replace it in the Markdown or declare it in templates/common/investigacion.sty."
            ));
        }
    }
    warnings
}

#[cfg(test)]
mod tests {
    use super::*;

    fn data() -> DocumentData {
        DocumentData {
            university: "U".into(),
            faculty: "F".into(),
            student: "A".into(),
            semester: "S".into(),
            title: "T".into(),
            course: "M".into(),
            teacher: "D".into(),
            date: "Agosto 23, 2026".into(),
            ..Default::default()
        }
    }

    #[test]
    fn escape_protects_special_characters() {
        assert_eq!(latex_escape("A&B_50%"), r"A\&B\_50\%");
    }

    #[test]
    fn all_markers_are_replaced() {
        let template = "%%UNIVERSIDAD%% %%CONTENIDO_MARKDOWN%% %%FECHA_ENTREGA%%";
        assert_eq!(
            render_template(template, r"\section{Texto}", &data()).unwrap(),
            r"U \section{Texto} Agosto 23, 2026"
        );
    }

    #[test]
    fn unknown_markers_and_missing_content_are_errors() {
        assert!(render_template("%%DOCENT%% %%CONTENIDO_MARKDOWN%%", "x", &data()).is_err());
        assert!(render_template("%%TITULO%% sin contenido", "x", &data()).is_err());
    }

    #[test]
    fn content_may_mention_a_marker() {
        assert_eq!(
            render_template(CONTENT_MARKER, "habla de %%TITULO%%", &data()).unwrap(),
            "habla de %%TITULO%%"
        );
    }

    #[test]
    fn members_are_joined_with_a_latex_newline_and_escaped_one_by_one() {
        let mut team = data();
        team.members = vec!["Ana".into(), "Luis".into()];
        assert_eq!(
            render_template("%%INTEGRANTES%% %%CONTENIDO_MARKDOWN%%", "x", &team).unwrap(),
            r"Ana\\Luis x"
        );
        team.members = vec!["A&B".into(), "C_D".into(), "50%".into()];
        let rendered = render_template("%%INTEGRANTES%%%%CONTENIDO_MARKDOWN%%", "", &team).unwrap();
        assert_eq!(rendered, r"A\&B\\C\_D\\50\%");
        assert!(!rendered.contains(r"\textbackslash"));
    }

    #[test]
    fn optional_fields_may_be_empty() {
        let mut empty = data();
        empty.student.clear();
        empty.teacher.clear();
        let rendered = render_template(
            "[%%ALUMNO%%][%%INTEGRANTES%%][%%DOCENTE%%][%%GRUPO%%]%%CONTENIDO_MARKDOWN%%",
            "x",
            &empty,
        );
        assert_eq!(rendered.unwrap(), "[][][][]x");
    }

    #[test]
    fn group_is_escaped() {
        let mut grouped = data();
        grouped.group = "A&B_1".into();
        assert_eq!(render_template("%%GRUPO%%%%CONTENIDO_MARKDOWN%%", "", &grouped).unwrap(), r"A\&B\_1");
    }

    #[test]
    fn only_the_real_latex_error_is_kept() {
        let log = "(/usr/share/texmf-dist/tex/latex/base/article.cls)\n\
                   (/usr/share/texmf-dist/tex/latex/hyperref/hyperref.sty)\n\
                   /tmp/x/trabajo.tex:228: LaTeX Error: No counter 'none' defined.\n\
                   \n\
                   See the LaTeX manual for explanation.\n\
                   l.228 ...width - 6\\tabcolsep) * \\real{0.2500}}@{}}\n";
        let summary = summarize_latex_errors(log);
        assert!(summary.contains("No counter 'none' defined"));
        assert!(!summary.contains("hyperref.sty"));
    }

    #[test]
    fn unicode_warnings_are_extracted_once() {
        let line = "LaTeX Warning: Caracter Unicode sin definir: ∮ (U+222E) on input line 27.\n";
        let warnings = unsupported_character_warnings(&line.repeat(3));
        assert_eq!(warnings.len(), 1);
        assert!(warnings[0].contains('∮') && warnings[0].contains("U+222E"));
        assert!(unsupported_character_warnings("Output written on trabajo.pdf").is_empty());
    }

    #[test]
    fn warnings_survive_a_log_with_font_encoded_lines() {
        let mut raw = b"Underfull \\hbox: c\xf3digo\n".to_vec();
        raw.extend_from_slice(
            "LaTeX Warning: Caracter Unicode sin definir: ⭐ (U+2B50) on input line 3.\n".as_bytes(),
        );
        let warnings = unsupported_character_warnings(&crate::encoding::decode_latex_log(&raw));
        assert_eq!(warnings.len(), 1);
        assert!(warnings[0].contains('⭐'));
    }
}
