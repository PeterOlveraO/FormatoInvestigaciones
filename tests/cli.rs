//! El CLI de punta a punta, sin llegar a compilar.

mod common;

use investigacion::cli::{Args, Reporter, execute};

struct Silent;
impl Reporter for Silent {
    fn warning(&mut self, _: String) {}
    fn info(&mut self, _: String) {}
}

#[test]
fn a_missing_student_is_not_an_error_but_a_missing_markdown_is() {
    let directory = tempfile::tempdir().unwrap();
    let env = common::write(directory.path(), ".env", "UNIVERSIDAD=U\nFACULTAD=F\nSEMESTRE=7\n");
    let args: Args = clap::Parser::try_parse_from([
        "investigacion",
        directory.path().join("no-existe.md").to_str().unwrap(),
        "--title",
        "T",
        "--course",
        "M",
        "--env-file",
        env.to_str().unwrap(),
    ])
    .unwrap();
    let error = execute(&args, &common::isolated_project(directory.path()), &mut Silent).unwrap_err();
    assert!(!error.0.contains("Faltan datos") && !error.0.contains("Missing data"));
    assert!(error.0.contains("no-existe.md"));
}

fn run(dir: &std::path::Path, env: &str, extra: &[&str]) -> String {
    let env = common::write(dir, ".env", env);
    let markdown = dir.join("no-existe.md");
    let mut argv =
        vec!["investigacion", markdown.to_str().unwrap(), "--title", "T", "--course", "M", "--env-file"];
    argv.push(env.to_str().unwrap());
    argv.extend_from_slice(extra);
    let args: Args = clap::Parser::try_parse_from(argv).unwrap();
    execute(&args, &common::isolated_project(dir), &mut Silent).unwrap_err().0
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
