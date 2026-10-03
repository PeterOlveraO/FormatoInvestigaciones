//! Utilidades compartidas por las pruebas de integración.
#![allow(dead_code)]

use std::path::{Path, PathBuf};
use std::process::Command;

use investigacion::document::DocumentData;
use investigacion::project::Project;

pub fn root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

pub fn project() -> Project {
    Project::at(root())
}

/// La plantilla APA junto con el preámbulo común que carga.
/// Proyecto aislado en `dir` con una copia de las plantillas: así las pruebas
/// no dejan rastro en la caché del proyecto real.
pub fn isolated_project(dir: &Path) -> Project {
    for part in ["common", "apa"] {
        let source = root().join("templates").join(part);
        let target = dir.join("templates").join(part);
        std::fs::create_dir_all(&target).unwrap();
        for entry in std::fs::read_dir(source).unwrap().flatten() {
            std::fs::copy(entry.path(), target.join(entry.file_name())).unwrap();
        }
    }
    Project::at(dir)
}

pub fn full_template() -> String {
    [
        "templates/common/investigacion.sty",
        "templates/common/investigacion-final.sty",
        "templates/apa/template.ltx",
    ]
    .iter()
    .map(|part| std::fs::read_to_string(root().join(part)).unwrap())
    .collect()
}

pub fn has_tool(name: &str) -> bool {
    Command::new(name).arg("--version").output().is_ok()
}

pub fn data(title: &str) -> DocumentData {
    DocumentData {
        university: "U".into(),
        faculty: "F".into(),
        student: "A".into(),
        semester: "2026-2".into(),
        title: title.into(),
        subject: "M".into(),
        teacher: "D".into(),
        date: "Agosto 23, 2026".into(),
        ..Default::default()
    }
}

pub fn write(dir: &Path, name: &str, text: &str) -> PathBuf {
    let path = dir.join(name);
    std::fs::write(&path, text).unwrap();
    path
}
