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
    // Copia recursiva de templates/ salvo los logos, que son de cada quien.
    fn copy_tree(source: &Path, target: &Path) {
        std::fs::create_dir_all(target).unwrap();
        for entry in std::fs::read_dir(source).unwrap().flatten() {
            let path = entry.path();
            if path.is_dir() {
                if entry.file_name() != "logos" {
                    copy_tree(&path, &target.join(entry.file_name()));
                }
            } else {
                std::fs::copy(&path, target.join(entry.file_name())).unwrap();
            }
        }
    }
    copy_tree(&root().join("templates"), &dir.join("templates"));
    Project::at(dir)
}

pub fn full_template() -> String {
    [
        "templates/common/investigacion-base.sty",
        "templates/formats/apa7/format.sty",
        "templates/common/investigacion-final.sty",
        "templates/designs/geometric-cover/template.ltx",
    ]
    .iter()
    .map(|part| std::fs::read_to_string(root().join(part)).unwrap())
    .collect::<String>()
    // Los marcadores del contrato del diseño los resuelve el generador.
    .replace("%%FORMAT%%", "")
    .replace("%%CLASS_OPTIONS%%", "12pt")
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
        course: "M".into(),
        teacher: "D".into(),
        date: "23 de agosto de 2026".into(),
        ..Default::default()
    }
}

pub fn write(dir: &Path, name: &str, text: &str) -> PathBuf {
    let path = dir.join(name);
    std::fs::write(&path, text).unwrap();
    path
}
