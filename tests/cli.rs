//! El CLI de punta a punta, sin llegar a compilar.

mod common;

use investigacion::cli::{Args, Reporter, execute};

struct Silent;
impl Reporter for Silent {
    fn warning(&mut self, _: String) {}
    fn info(&mut self, _: String) {}
}

/// `data` son líneas `NOMBRE=valor` que se pasan con `--set`, como sin perfil.
fn run(dir: &std::path::Path, data: &str, extra: &[&str]) -> String {
    let markdown = dir.join("no-existe.md");
    let mut argv = vec!["investigacion", markdown.to_str().unwrap(), "--title", "T"];
    for line in data.lines() {
        argv.extend(["--set", line]);
    }
    argv.extend_from_slice(extra);
    let args: Args = clap::Parser::try_parse_from(argv).unwrap();
    execute(&args, &common::isolated_project(dir), &mut Silent).unwrap_err().0
}

#[test]
fn a_missing_student_is_not_an_error_but_a_missing_markdown_is() {
    let directory = tempfile::tempdir().unwrap();
    // Sin perfil, todo llega por --set; la materia también.
    let error = run(directory.path(), "MATERIA=M\nUNIVERSIDAD=U\nFACULTAD=F\nSEMESTRE=7", &[]);
    assert!(!error.contains("Faltan datos") && !error.contains("Missing data"), "{error}");
    assert!(error.contains("no-existe.md"));
}

#[test]
fn only_the_data_the_design_uses_is_required() {
    investigacion::i18n::set(investigacion::i18n::Lang::Es);
    let directory = tempfile::tempdir().unwrap();
    let dir = directory.path();
    // APA (portada geométrica) usa UNIVERSIDAD: sin ella, falta un dato.
    let apa = run(dir, "FACULTAD=F\nSEMESTRE=7\n", &[]);
    assert!(apa.contains("Faltan datos") && apa.contains("UNIVERSIDAD"), "{apa}");
    // Un diseño propio que no la usa, pero con un campo obligatorio.
    let mine = dir.join("my-templates/designs/simple");
    std::fs::create_dir_all(&mine).unwrap();
    std::fs::write(mine.join("template.ltx"), "%%FORMAT%%\n%%TITULO%% %%SALON%%\n%%CONTENIDO_MARKDOWN%%\n")
        .unwrap();
    std::fs::write(mine.join("template.toml"), "[fields.SALON]\nrequired = true\n").unwrap();
    let without_field = run(dir, "", &["--design", "simple"]);
    assert!(without_field.contains("SALON") && !without_field.contains("UNIVERSIDAD"), "{without_field}");
    // El informe muestra universidad y materia si existen, pero no las exige.
    let report = run(dir, "", &["--design", "report"]);
    assert!(report.contains("no-existe.md"), "{report}");
    // Con el campo, los datos están completos y el error ya es el Markdown que no existe.
    let complete = run(dir, "", &["--design", "simple", "--set", "SALON=B-204"]);
    assert!(complete.contains("no-existe.md"), "{complete}");
}

#[test]
fn a_missing_project_folder_is_explained() {
    investigacion::i18n::set(investigacion::i18n::Lang::En);
    let directory = tempfile::tempdir().unwrap();
    let markdown = common::write(directory.path(), "t.md", "# A\n");
    let args: Args =
        clap::Parser::try_parse_from(["investigacion", markdown.to_str().unwrap(), "--title", "T"]).unwrap();
    let project = investigacion::project::Project::at(directory.path());
    let error = execute(&args, &project, &mut Silent).unwrap_err().0;
    assert!(error.contains("project folder") && error.contains("INVESTIGACION_HOME"), "{error}");
}

struct Collect(Vec<String>);
impl Reporter for Collect {
    fn warning(&mut self, message: String) {
        self.0.push(message);
    }
    fn info(&mut self, _: String) {}
}

/// Avisos de una ejecución con el perfil `viejo` (su texto) y opciones extra.
fn warnings_with(profile: &str, extra: &[&str]) -> Vec<String> {
    let directory = tempfile::tempdir().unwrap();
    let dir = directory.path();
    std::fs::create_dir_all(dir.join("courses")).unwrap();
    std::fs::write(dir.join("courses/viejo.toml"), format!("name = \"M\"\n{profile}")).unwrap();
    let markdown = dir.join("no-existe.md");
    let mut argv = vec!["investigacion", markdown.to_str().unwrap(), "--title", "T", "-p", "viejo"];
    argv.extend_from_slice(extra);
    let args: Args = clap::Parser::try_parse_from(argv).unwrap();
    let mut reporter = Collect(Vec::new());
    let error = execute(&args, &common::isolated_project(dir), &mut reporter).unwrap_err().0;
    assert!(error.contains("no-existe.md") || error.contains("UNIVERSIDAD"), "{error}");
    reporter.0
}

#[test]
fn old_formats_are_ignored_with_a_warning() {
    investigacion::i18n::set(investigacion::i18n::Lang::Es);
    // apa7 sigue valiendo en silencio, en el perfil y en --format.
    assert!(warnings_with("format = \"apa7\"\n", &["--format", "apa7"]).is_empty());
    let profile = warnings_with("format = \"mla\"\n", &[]);
    assert!(
        profile.len() == 1 && profile[0].contains("«mla»") && profile[0].contains("APA 7"),
        "{profile:?}"
    );
    // La opción gana sobre el perfil, también al avisar.
    let option = warnings_with("format = \"mla\"\n", &["--format", "harvard"]);
    assert!(option.len() == 1 && option[0].contains("«harvard»"), "{option:?}");
}
