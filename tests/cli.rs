//! El CLI de punta a punta, sin llegar a compilar.

mod common;

use investigacion::cli::{Args, Reporter, execute};
use investigacion::project::Project;

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
        "--subject",
        "M",
        "--env-file",
        env.to_str().unwrap(),
    ])
    .unwrap();
    let error = execute(&args, &Project::at(directory.path()), &mut Silent).unwrap_err();
    assert!(!error.0.contains("Missing environment"));
    assert!(error.0.contains("no-existe.md"));
}
