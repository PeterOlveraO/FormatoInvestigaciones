//! Perfiles de materia: los datos que se repiten en todos los trabajos de una
//! materia (nombre, docente, grupo, plantilla y carpeta) guardados en
//! `subjects/<clave>.toml`, para elegirlos en vez de escribirlos cada vez.

use std::path::{Path, PathBuf};

use serde::Deserialize;

use crate::encoding::decode_text;
use crate::error::{GenerationError, Result};
use crate::project::Project;

/// Contenido de un perfil. Todos los campos son opcionales salvo `subject`.
#[derive(Debug, Clone, Default, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct SubjectProfile {
    /// Nombre de la materia tal como sale en la portada.
    pub subject: String,
    #[serde(default)]
    pub teacher: String,
    #[serde(default)]
    pub group: String,
    /// Integrantes del equipo, separados por comas.
    #[serde(default)]
    pub members: String,
    /// Nombre de una plantilla de `templates/`.
    #[serde(default)]
    pub template: String,
    /// Subcarpeta de `input/` con los trabajos de la materia; el PDF se
    /// guarda en la misma subcarpeta de `output/`.
    #[serde(default)]
    pub folder: String,
}

/// Un perfil junto con su clave (el nombre del archivo sin `.toml`).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Subject {
    pub key: String,
    pub profile: SubjectProfile,
}

impl Subject {
    /// Lee y valida un perfil; los errores dicen qué archivo falló.
    pub fn load(path: &Path) -> Result<Self> {
        let text = decode_text(&std::fs::read(path)?, path)?;
        let profile: SubjectProfile = toml::from_str(&text).map_err(|e| {
            GenerationError::new(format!("Invalid subject profile {}: {}", path.display(), e.message()))
        })?;
        if profile.subject.trim().is_empty() {
            return Err(GenerationError::new(format!(
                "The subject profile {} needs a non-empty `subject`.",
                path.display()
            )));
        }
        let key = path.file_stem().unwrap_or_default().to_string_lossy().into_owned();
        Ok(Self { key, profile })
    }

    /// Carpeta de entrada de la materia, si existe.
    pub fn input_dir(&self, project: &Project) -> Option<PathBuf> {
        let folder = self.profile.folder.trim();
        (!folder.is_empty()).then(|| project.input_dir().join(folder)).filter(|dir| dir.is_dir())
    }

    /// Carpeta de salida: `output/<folder>` o `output/` si no hay carpeta.
    pub fn output_dir(&self, project: &Project) -> PathBuf {
        match self.profile.folder.trim() {
            "" => project.output_dir(),
            folder => project.output_dir().join(folder),
        }
    }
}

/// Todos los perfiles de `subjects/`, ordenados por clave. Un perfil roto no
/// impide ver los demás: se devuelve aparte para poder avisar.
pub fn list_subjects(project: &Project) -> (Vec<Subject>, Vec<GenerationError>) {
    let mut paths: Vec<PathBuf> = std::fs::read_dir(project.subjects_dir())
        .into_iter()
        .flatten()
        .flatten()
        .map(|entry| entry.path())
        .filter(|path| path.extension().is_some_and(|ext| ext == "toml") && path.is_file())
        .collect();
    paths.sort();
    let (mut subjects, mut errors) = (Vec::new(), Vec::new());
    for path in paths {
        match Subject::load(&path) {
            Ok(subject) => subjects.push(subject),
            Err(error) => errors.push(error),
        }
    }
    (subjects, errors)
}

/// Busca un perfil por clave (sin distinguir mayúsculas) o por ruta.
pub fn find_subject(project: &Project, choice: &str) -> Result<Subject> {
    let by_key = project.subjects_dir().join(format!("{}.toml", choice.trim()));
    if by_key.is_file() {
        return Subject::load(&by_key);
    }
    let (subjects, _) = list_subjects(project);
    if let Some(found) = subjects.iter().find(|s| s.key.eq_ignore_ascii_case(choice.trim())) {
        return Ok(found.clone());
    }
    let path = Path::new(choice);
    if path.is_file() {
        return Subject::load(path);
    }
    let keys: Vec<&str> = subjects.iter().map(|s| s.key.as_str()).collect();
    Err(GenerationError::new(format!(
        "Subject profile not found: {choice}. Available profiles: {}.",
        if keys.is_empty() { "none (create one in subjects/)".to_owned() } else { keys.join(", ") }
    )))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn project_with(files: &[(&str, &str)]) -> (tempfile::TempDir, Project) {
        let directory = tempfile::tempdir().unwrap();
        let project = Project::at(directory.path());
        std::fs::create_dir_all(project.subjects_dir()).unwrap();
        for (name, text) in files {
            std::fs::write(project.subjects_dir().join(name), text).unwrap();
        }
        (directory, project)
    }

    #[test]
    fn profiles_are_listed_and_found_by_key() {
        let (_dir, project) = project_with(&[
            (
                "ia.toml",
                "subject = \"Inteligencia artificial\"\nteacher = \"T\"\ngroup = \"M\"\nfolder = \"IA\"\n",
            ),
            ("roto.toml", "subject = \n"),
            ("notas.txt", "x"),
        ]);
        let (subjects, errors) = list_subjects(&project);
        assert_eq!(subjects.len(), 1);
        assert_eq!(errors.len(), 1);
        let ia = find_subject(&project, "IA").unwrap();
        assert_eq!(ia.profile.teacher, "T");
        assert_eq!(ia.output_dir(&project), project.output_dir().join("IA"));
        assert!(find_subject(&project, "nada").unwrap_err().0.contains("ia"));
    }

    #[test]
    fn unknown_fields_and_empty_subjects_are_errors() {
        let (_dir, project) =
            project_with(&[("a.toml", "subject = \"X\"\nprofe = \"Y\"\n"), ("b.toml", "teacher = \"Y\"\n")]);
        assert!(find_subject(&project, "a").is_err());
        assert!(find_subject(&project, "b").is_err());
    }
}
