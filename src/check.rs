//! `investigacion --check-template <diseño>`: revisa que un diseño cumpla el
//! contrato y lo compila con cada formato compatible, para encontrar los
//! problemas antes de entregar un trabajo.

use crate::document::DocumentData;
use crate::encoding::decode_text;
use crate::error::Result;
use crate::generate::{GenerateOptions, generate_pdf};
use crate::project::Project;
use crate::template::{Design, STANDARD_MARKERS, list_formats, load_design};

/// Resultado de la revisión: errores (impiden usarlo), avisos y la
/// compilación de prueba con cada formato.
#[derive(Debug, Default)]
pub struct CheckReport {
    pub design: String,
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

/// Revisión estática: contrato, formatos y marcadores.
pub fn inspect(project: &Project, design: &Design) -> CheckReport {
    let mut report = CheckReport { design: design.key.clone(), ..Default::default() };
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

    let available = list_formats(project);
    for format in &design.manifest.formats {
        if !available.contains(format) {
            report.errors.push(tr!(
                es: "El formato {format} de template.toml no existe. Disponibles: {}.",
                en: "The format {format} in template.toml does not exist. Available: {}.",
                available.join(", ")
            ));
        }
    }

    for name in design.custom_fields() {
        if let Some(standard) = similar_marker(name) {
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
    report
}

/// Datos de ejemplo para la compilación de prueba: todos llenos.
fn sample_data(design: &Design) -> DocumentData {
    let mut data = DocumentData {
        university: "Universidad".into(),
        faculty: "Facultad".into(),
        student: "Alumno".into(),
        semester: "2026-2".into(),
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
/// catálogo de ejemplo en cada formato compatible.
pub fn check_design(
    project: &Project,
    choice: &str,
    on_progress: &mut dyn FnMut(&str),
) -> Result<CheckReport> {
    let design = load_design(project, Some(choice))?;
    let mut report = inspect(project, &design);
    if !report.errors.is_empty() {
        return Ok(report);
    }
    let catalog = project.root.join("examples").join("catalog.md");
    if !catalog.is_file() {
        report.warnings.push(tr!(
            es: "No se encontró examples/catalog.md: no se hizo la compilación de prueba.",
            en: "examples/catalog.md was not found: the test build was skipped."
        ));
        return Ok(report);
    }
    let formats: Vec<Option<String>> = if design.is_self_contained() {
        vec![None]
    } else {
        list_formats(project).into_iter().filter(|f| design.accepts(f)).map(Some).collect()
    };
    let output = tempfile::Builder::new().prefix("investigacion-check-").tempdir()?;
    let data = sample_data(&design);
    for format in formats {
        let label = format.clone().unwrap_or_else(|| "-".into());
        on_progress(&label);
        let options = GenerateOptions {
            design: Some(design.file.display().to_string()),
            format: format.clone(),
            file_name: Some(label.clone()),
            ..Default::default()
        };
        let result = generate_pdf(project, &catalog, output.path(), &data, &options, &mut |_| {})
            .map(|_| ())
            .map_err(|e| e.0.lines().take(4).collect::<Vec<_>>().join("\n"));
        report.builds.push((label, result));
    }
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
        let report = inspect(&project, &broken);
        let all = format!("{:?} {:?}", report.errors, report.warnings);
        assert!(all.contains("investigacion-base") && all.contains("investigacion-final"), "{all}");
        assert!(all.contains("%%CONTENIDO_MARKDOWN%%") && all.contains("nope"));
        assert!(all.contains("¿quisiste decir %%DOCENTE%%?"));
        assert!(all.contains("SALON") && all.contains("AULA"));
        assert!(!report.passed());

        let good = design(
            &project,
            "\\documentclass[%%CLASS_OPTIONS%%]{article}\n\\usepackage{investigacion-base}\n%%FORMAT%%\n\\usepackage{investigacion-final}\n%%CONTENIDO_MARKDOWN%%\n",
            "",
        );
        let report = inspect(&project, &good);
        assert!(report.errors.is_empty() && report.warnings.is_empty(), "{report:?}");
    }
}
