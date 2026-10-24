//! `investigacion --check-template <diseño>`: revisa que un diseño cumpla el
//! contrato y lo compila con APA 7, para encontrar los
//! problemas antes de entregar un trabajo.

use std::collections::BTreeSet;
use std::sync::LazyLock;

use regex::Regex;

use crate::document::DocumentData;
use crate::encoding::decode_text;
use crate::error::Result;
use crate::generate::{GenerateOptions, generate_pdf};
use crate::latex::is_comment_line;
use crate::logging;
use crate::project::DEFAULT_FORMAT;
use crate::project::Project;
use crate::template::{Design, STANDARD_MARKERS, load_design, resolve_layout, retired_design_warning};

/// Resultado de la revisión: errores (impiden usarlo), avisos y la
/// compilación de prueba.
#[derive(Debug, Default)]
pub struct CheckReport {
    pub errors: Vec<String>,
    pub warnings: Vec<String>,
    pub builds: Vec<(String, std::result::Result<(), String>)>,
}

impl CheckReport {
    pub fn passed(&self) -> bool {
        self.errors.is_empty() && self.builds.iter().all(|(_, r)| r.is_ok())
    }
}

/// Distancia de edición, para sugerir el marcador estándar más parecido.
fn distance(a: &str, b: &str) -> usize {
    let b: Vec<char> = b.chars().collect();
    let mut row: Vec<usize> = (0..=b.len()).collect();
    for (i, ca) in a.chars().enumerate() {
        let mut previous = row[0];
        row[0] = i + 1;
        for (j, cb) in b.iter().enumerate() {
            let current = row[j + 1];
            row[j + 1] = (previous + usize::from(ca != *cb)).min(row[j] + 1).min(current + 1);
            previous = current;
        }
    }
    row[b.len()]
}

/// El marcador estándar al que se parece un nombre (a 1 o 2 letras).
pub fn similar_marker(name: &str) -> Option<&'static str> {
    STANDARD_MARKERS.iter().copied().filter(|m| distance(name, m) <= 2).min_by_key(|m| distance(name, m))
}

/// Revisión estática: contrato y marcadores.
pub fn inspect(design: &Design) -> CheckReport {
    let mut report = CheckReport::default();
    let text = std::fs::read(&design.file)
        .ok()
        .and_then(|raw| decode_text(&raw, &design.file).ok())
        .unwrap_or_default();

    for (needle, what) in [
        ("\\usepackage{investigacion-base}", "\\usepackage{investigacion-base}"),
        ("\\usepackage{investigacion-final}", "\\usepackage{investigacion-final}"),
    ] {
        if !text.contains(needle) {
            report.errors.push(tr!(
                es: "Falta {what}: sin él no funcionan las tablas, los diagramas ni los símbolos.",
                en: "{what} is missing: without it tables, diagrams and symbols do not work."
            ));
        }
    }
    if !design.uses("CONTENIDO_MARKDOWN") {
        report.errors.push(tr!(
            es: "Falta %%CONTENIDO_MARKDOWN%%: es donde entra el trabajo.",
            en: "%%CONTENIDO_MARKDOWN%% is missing: it is where the paper goes."
        ));
    }
    if design.is_self_contained() {
        report.warnings.push(tr!(
            es: "No usa %%FORMAT%%: es un diseño autocontenido y no aplica ningún formato.",
            en: "It does not use %%FORMAT%%: it is self-contained and applies no format."
        ));
    } else if !design.uses("CLASS_OPTIONS") {
        report.warnings.push(tr!(
            es: "No usa %%CLASS_OPTIONS%% en \\documentclass: el formato no podrá fijar el tamaño de letra ni el papel.",
            en: "It does not use %%CLASS_OPTIONS%% in \\documentclass: the format cannot set the font size or paper."
        ));
    }

    if !design.manifest.formats.is_empty() {
        report.warnings.push(tr!(
            es: "template.toml todavía tiene formats: ya no se usa, todos los diseños van con APA 7.",
            en: "template.toml still has formats: it is no longer used, every design goes with APA 7."
        ));
    }

    for name in design.custom_fields() {
        if name == "SEMESTRE" {
            report.warnings.push(tr!(
                es: "%%SEMESTRE%% ya no es un dato estándar: queda vacío salvo con --set SEMESTRE=… o [fields] del perfil. Usa %%GRUPO%%.",
                en: "%%SEMESTRE%% is no longer a standard field: it stays empty unless given with --set SEMESTRE=… or the profile's [fields]. Use %%GRUPO%%."
            ));
        } else if let Some(standard) = similar_marker(name) {
            report.warnings.push(tr!(
                es: "%%{name}%% no es un dato estándar: ¿quisiste decir %%{standard}%%? Si no, es un campo propio.",
                en: "%%{name}%% is not a standard field: did you mean %%{standard}%%? Otherwise it is a custom field."
            ));
        } else if !design.manifest.fields.contains_key(name) {
            report.warnings.push(tr!(
                es: "%%{name}%% es un campo propio sin describir en template.toml: el menú lo mostrará como «{name}».",
                en: "%%{name}%% is a custom field not described in template.toml: the menu will show it as \"{name}\"."
            ));
        }
    }
    for name in design.manifest.fields.keys() {
        if !design.uses(name) {
            report.warnings.push(tr!(
                es: "template.toml describe el campo {name}, pero el diseño no usa %%{name}%%.",
                en: "template.toml describes the field {name}, but the design does not use %%{name}%%."
            ));
        }
    }
    for name in &design.manifest.optional {
        if !design.uses(name) {
            report.warnings.push(tr!(
                es: "optional menciona {name}, pero el diseño no usa %%{name}%%.",
                en: "optional mentions {name}, but the design does not use %%{name}%%."
            ));
        }
    }
    // Un marcador mal escrito (%%AULA2%%, %%salon%%) no se rellena, y su % deja
    // en LaTeX el resto de la línea como comentario.
    static MARKER_LIKE: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"%%([A-Za-z0-9_]+)%%").unwrap());
    let mut malformed = BTreeSet::new();
    for line in text.lines().filter(|line| !is_comment_line(line)) {
        for found in MARKER_LIKE.captures_iter(line) {
            let name = &found[1];
            if !name.chars().all(|c| c.is_ascii_uppercase() || c == '_') {
                malformed.insert(name.to_owned());
            }
        }
    }
    for name in malformed {
        report.warnings.push(tr!(
            es: "%%{name}%% no es un marcador válido (solo mayúsculas y _): no se rellena y en LaTeX el % deja comentado el resto de la línea.",
            en: "%%{name}%% is not a valid marker (only capitals and _): it is not filled and in LaTeX the % comments out the rest of the line."
        ));
    }
    report
}

/// Datos de ejemplo para la compilación de prueba: todos llenos.
fn sample_data(design: &Design) -> DocumentData {
    let mut data = DocumentData {
        university: "Universidad".into(),
        faculty: "Facultad".into(),
        student: "Alumno".into(),
        title: "Prueba del diseño".into(),
        course: "Materia".into(),
        teacher: "Docente".into(),
        date: crate::document::today(crate::i18n::Lang::Es),
        members: vec!["Ana Ruiz".into(), "Luis Paz".into()],
        group: "7-A".into(),
        ..Default::default()
    };
    for name in design.custom_fields() {
        data.fields.insert(name.to_owned(), "Ejemplo".into());
    }
    data
}

/// Revisa el diseño y, si el contrato está completo, lo compila con el
/// catálogo de ejemplo.
pub fn check_design(
    project: &Project,
    choice: &str,
    on_progress: &mut dyn FnMut(&str),
) -> Result<CheckReport> {
    let design = load_design(project, Some(choice))?;
    logging::info(format_args!("check design: {} ({})", design.key, design.file.display()));
    let mut report = inspect(&design);
    report.warnings.extend(retired_design_warning(choice, &design));
    for error in &report.errors {
        logging::warn(format_args!("check error: {error}"));
    }
    for warning in &report.warnings {
        logging::warn(format_args!("check warning: {warning}"));
    }
    if !report.errors.is_empty() {
        return Ok(report);
    }
    let catalog = project.root.join("examples").join("catalog.md");
    if !catalog.is_file() {
        report.warnings.push(tr!(
            es: "No se encontró examples/catalog.md: no se hizo la compilación de prueba.",
            en: "examples/catalog.md was not found: the test build was skipped."
        ));
        logging::warn(format_args!("check: test build skipped, {} not found", catalog.display()));
        return Ok(report);
    }
    let output = tempfile::Builder::new().prefix("investigacion-check-").tempdir()?;
    let label = if design.is_self_contained() { "-" } else { DEFAULT_FORMAT };
    on_progress(label);
    let result = resolve_layout(project, Some(choice)).and_then(|layout| {
        let options =
            GenerateOptions { layout: Some(layout), file_name: Some(label.to_owned()), ..Default::default() };
        generate_pdf(project, &catalog, output.path(), &sample_data(&design), &options, &mut |_| {})
    });
    // El registro guarda el error entero; el informe, solo el principio.
    match &result {
        Ok(_) => logging::info(format_args!("check build with {label}: ok")),
        Err(error) => logging::warn(format_args!("check build with {label} failed: {error}")),
    }
    let result = result.map(|_| ()).map_err(|e| e.0.lines().take(4).collect::<Vec<_>>().join("\n"));
    report.builds.push((label.to_owned(), result));
    Ok(report)
}

/// Lo que imprime el CLI; devuelve el código de salida.
pub fn run_cli(project: &Project, choice: &str) -> i32 {
    println!("{}", tr!(es: "Revisando el diseño {choice}…", en: "Checking the design {choice}…"));
    let report = match check_design(project, choice, &mut |format| {
        println!(
            "{}",
            tr!(es: "  compilando el catálogo con {format}…", en: "  building the catalog with {format}…")
        );
    }) {
        Ok(report) => report,
        Err(error) => {
            logging::error(format_args!("check design {choice} failed: {error}"));
            eprintln!("Error: {error}");
            return 1;
        }
    };
    for error in &report.errors {
        println!("✗ {error}");
    }
    for warning in &report.warnings {
        println!("! {warning}");
    }
    for (format, result) in &report.builds {
        match result {
            Ok(()) => println!("✓ {}", tr!(es: "Compila con {format}.", en: "Builds with {format}.")),
            Err(error) => println!(
                "✗ {}\n{error}",
                tr!(es: "No compila con {format}:", en: "Does not build with {format}:")
            ),
        }
    }
    logging::info(format_args!(
        "check design {choice}: {}",
        if report.passed() { "passed" } else { "failed" }
    ));
    if report.passed() {
        println!("{}", tr!(es: "Listo: el diseño se puede usar.", en: "Done: the design is ready to use."));
        0
    } else {
        1
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn design(project: &Project, ltx: &str, toml: &str) -> Design {
        let dir = project.root.join("my-templates/designs/test");
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::write(dir.join("template.ltx"), ltx).unwrap();
        std::fs::write(dir.join("template.toml"), toml).unwrap();
        load_design(project, Some("test")).unwrap()
    }

    #[test]
    fn malformed_markers_are_reported_but_commented_ones_are_not() {
        crate::i18n::set(crate::i18n::Lang::Es);
        let directory = tempfile::tempdir().unwrap();
        let project = Project::at(directory.path());
        let ltx = "%%FORMAT%%\n%%AULA2%% %%salon%%\n% %%AULA3%%\n%%CONTENIDO_MARKDOWN%%\n";
        let report = inspect(&design(&project, ltx, ""));
        let all = format!("{:?}", report.warnings);
        assert!(all.contains("%%AULA2%%") && all.contains("%%salon%%"), "{all}");
        assert!(!all.contains("%%AULA3%%"), "{all}");
    }

    #[test]
    fn typos_get_a_suggestion() {
        assert_eq!(similar_marker("DOCENTES"), Some("DOCENTE"));
        assert_eq!(similar_marker("TITLO"), Some("TITULO"));
        assert_eq!(similar_marker("SALON"), None);
    }

    #[test]
    fn the_contract_and_the_markers_are_checked() {
        crate::i18n::set(crate::i18n::Lang::Es);
        let directory = tempfile::tempdir().unwrap();
        let project = Project::at(directory.path());
        let broken =
            design(&project, "%%FORMAT%%\n%%DOCENTES%% %%SALON%%\n", "formats = [\"nope\"]\n[fields.AULA]\n");
        let report = inspect(&broken);
        let all = format!("{:?} {:?}", report.errors, report.warnings);
        assert!(all.contains("investigacion-base") && all.contains("investigacion-final"), "{all}");
        assert!(all.contains("%%CONTENIDO_MARKDOWN%%") && all.contains("todavía tiene formats"));
        assert!(all.contains("¿quisiste decir %%DOCENTE%%?"));
        assert!(all.contains("SALON") && all.contains("AULA"));
        assert!(!report.passed());

        let good = design(
            &project,
            "\\documentclass[%%CLASS_OPTIONS%%]{article}\n\\usepackage{investigacion-base}\n%%FORMAT%%\n\\usepackage{investigacion-final}\n%%CONTENIDO_MARKDOWN%%\n",
            "",
        );
        let report = inspect(&good);
        assert!(report.errors.is_empty() && report.warnings.is_empty(), "{report:?}");

        // Un diseño viejo con %%SEMESTRE%% recibe un aviso, no un error.
        let old = design(
            &project,
            "\\documentclass[%%CLASS_OPTIONS%%]{article}\n\\usepackage{investigacion-base}\n%%FORMAT%%\n\\usepackage{investigacion-final}\n%%SEMESTRE%%\n%%CONTENIDO_MARKDOWN%%\n",
            "",
        );
        let report = inspect(&old);
        assert!(report.errors.is_empty(), "{report:?}");
        assert!(report.warnings.len() == 1 && report.warnings[0].contains("%%GRUPO%%"), "{report:?}");
    }
}
