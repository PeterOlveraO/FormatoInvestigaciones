//! Perfiles de materia: los datos que se repiten en todos los trabajos de una
//! materia (nombre, docente, grupo, plantilla y carpeta) guardados en
//! `courses/<clave>.toml`, para elegirlos en vez de escribirlos cada vez.

use std::path::{Path, PathBuf};

use serde::Deserialize;

use crate::encoding::decode_text;
use crate::error::{GenerationError, Result};
use crate::project::Project;

/// Contenido de un perfil. Todos los campos son opcionales salvo `name`.
#[derive(Debug, Clone, Default, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct CourseProfile {
    /// Nombre de la materia tal como sale en la portada.
    #[serde(alias = "subject")]
    pub name: String,
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
pub struct Course {
    pub key: String,
    pub profile: CourseProfile,
}

impl Course {
    /// Lee y valida un perfil; los errores dicen qué archivo falló.
    pub fn load(path: &Path) -> Result<Self> {
        let text = decode_text(&std::fs::read(path)?, path)?;
        let profile: CourseProfile = toml::from_str(&text).map_err(|e| {
            // El detalle (`e.message()`) lo redacta el crate toml y va en inglés.
            GenerationError::new(tr!(
                es: "El perfil de materia {} no es válido: {}",
                en: "The course profile {} is not valid: {}",
                path.display(),
                e.message()
            ))
        })?;
        if profile.name.trim().is_empty() {
            return Err(GenerationError::new(tr!(
                es: "Al perfil de materia {} le falta el nombre (`name`).",
                en: "The course profile {} needs a name (`name`).",
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

/// Todos los perfiles de `courses/`, ordenados por clave. Un perfil roto no
/// impide ver los demás: se devuelve aparte para poder avisar.
pub fn list_courses(project: &Project) -> (Vec<Course>, Vec<GenerationError>) {
    let mut paths: Vec<PathBuf> = std::fs::read_dir(project.courses_dir())
        .into_iter()
        .flatten()
        .flatten()
        .map(|entry| entry.path())
        .filter(|path| path.extension().is_some_and(|ext| ext == "toml") && path.is_file())
        .collect();
    paths.sort();
    let (mut courses, mut errors) = (Vec::new(), Vec::new());
    for path in paths {
        match Course::load(&path) {
            Ok(course) => courses.push(course),
            Err(error) => errors.push(error),
        }
    }
    (courses, errors)
}

/// Busca un perfil por clave (sin distinguir mayúsculas) o por ruta.
pub fn find_course(project: &Project, choice: &str) -> Result<Course> {
    let by_key = project.courses_dir().join(format!("{}.toml", choice.trim()));
    if by_key.is_file() {
        return Course::load(&by_key);
    }
    let (courses, _) = list_courses(project);
    if let Some(found) = courses.iter().find(|s| s.key.eq_ignore_ascii_case(choice.trim())) {
        return Ok(found.clone());
    }
    let path = Path::new(choice);
    if path.is_file() {
        return Course::load(path);
    }
    let keys: Vec<&str> = courses.iter().map(|s| s.key.as_str()).collect();
    let available = if keys.is_empty() {
        tr!(es: "ninguno (crea uno en courses/)", en: "none (create one in courses/)")
    } else {
        keys.join(", ")
    };
    Err(GenerationError::new(tr!(
        es: "No se encontró el perfil de materia: {choice}. Perfiles disponibles: {available}.",
        en: "Course profile not found: {choice}. Available profiles: {available}."
    )))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn project_with(files: &[(&str, &str)]) -> (tempfile::TempDir, Project) {
        let directory = tempfile::tempdir().unwrap();
        let project = Project::at(directory.path());
        std::fs::create_dir_all(project.courses_dir()).unwrap();
        for (name, text) in files {
            std::fs::write(project.courses_dir().join(name), text).unwrap();
        }
        (directory, project)
    }

    #[test]
    fn profiles_are_listed_and_found_by_key() {
        let (_dir, project) = project_with(&[
            (
                "ia.toml",
                "name = \"Inteligencia artificial\"\nteacher = \"T\"\ngroup = \"M\"\nfolder = \"IA\"\n",
            ),
            ("roto.toml", "name = \n"),
            ("notas.txt", "x"),
        ]);
        let (courses, errors) = list_courses(&project);
        assert_eq!(courses.len(), 1);
        assert_eq!(errors.len(), 1);
        let ia = find_course(&project, "IA").unwrap();
        assert_eq!(ia.profile.teacher, "T");
        assert_eq!(ia.output_dir(&project), project.output_dir().join("IA"));
        assert!(find_course(&project, "nada").unwrap_err().0.contains("ia"));
    }

    #[test]
    fn unknown_fields_and_empty_names_are_errors() {
        let (_dir, project) =
            project_with(&[("a.toml", "name = \"X\"\nprofe = \"Y\"\n"), ("b.toml", "teacher = \"Y\"\n")]);
        assert!(find_course(&project, "a").is_err());
        assert!(find_course(&project, "b").is_err());
    }

    #[test]
    fn the_old_subject_key_is_still_accepted() {
        let (_dir, project) = project_with(&[("viejo.toml", "subject = \"Algebra\"\n")]);
        assert_eq!(find_course(&project, "viejo").unwrap().profile.name, "Algebra");
    }
}
