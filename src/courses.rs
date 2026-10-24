//! Perfiles de materia: los datos que se repiten en todos los trabajos de una
//! materia (nombre, docente, grupo, plantilla y carpeta) guardados en
//! `courses/<clave>.toml`, para elegirlos en vez de escribirlos cada vez.

use std::path::{Path, PathBuf};

use serde::Deserialize;

use crate::encoding::decode_text;
use crate::error::{GenerationError, Result};
use crate::logging;
use crate::project::{Project, replace_file};

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
    /// Datos generales de la portada: cada perfil guarda los suyos.
    #[serde(default)]
    pub university: String,
    #[serde(default)]
    pub faculty: String,
    #[serde(default)]
    pub student: String,
    /// Ya no se usa (el grupo cumple su función); se lee con un aviso para
    /// no romper perfiles viejos y no se guarda.
    #[serde(default)]
    pub semester: String,
    /// Ya no se usa (solo hay APA 7); se lee para no romper perfiles viejos y
    /// no se guarda.
    #[serde(default)]
    pub format: String,
    /// Diseño (`geometric-cover`…); `template` es su nombre antiguo.
    #[serde(default, alias = "template")]
    pub design: String,
    /// Idioma del documento (`es` o `en`); vacío = el del formato (español).
    #[serde(default)]
    pub language: String,
    /// Campos propios del diseño (`SALON = "B-204"`).
    #[serde(default)]
    pub fields: std::collections::BTreeMap<String, String>,
    /// Subcarpeta de `input/` con los trabajos de la materia; el PDF se
    /// guarda en la misma subcarpeta de `output/`.
    #[serde(default)]
    pub folder: String,
    /// Carpeta del PDF; gana sobre `output/<folder>`. Relativa a la carpeta de trabajo.
    #[serde(default)]
    pub output: String,
    /// Carpetas extra para una copia del PDF.
    #[serde(default)]
    pub copies: Vec<String>,
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
        let mut profile: CourseProfile = toml::from_str(&text).map_err(|e| {
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
        // Los marcadores van en mayúsculas: `salon` es `SALON` para el CLI y para el menú.
        // Al revés, para que con `salon` y `SALON` repetidos gane `SALON` (el que escribe el asistente).
        profile.fields = profile.fields.into_iter().rev().map(|(k, v)| (k.to_uppercase(), v)).collect();
        let key = path.file_stem().unwrap_or_default().to_string_lossy().into_owned();
        Ok(Self { key, profile })
    }

    /// Carpeta de entrada de la materia, si existe.
    pub fn input_dir(&self, project: &Project) -> Option<PathBuf> {
        let folder = self.profile.folder.trim();
        (!folder.is_empty()).then(|| project.input_dir().join(folder)).filter(|dir| dir.is_dir())
    }

    /// Carpeta de salida: `output`, si no `output/<folder>` y si no `output/`.
    pub fn output_dir(&self, project: &Project) -> PathBuf {
        match (self.profile.output.trim(), self.profile.folder.trim()) {
            ("", "") => project.output_dir(),
            ("", folder) => project.output_dir().join(folder),
            (output, _) => project.resolve(output),
        }
    }

    /// Carpetas de las copias extra, relativas a la carpeta de trabajo.
    pub fn copy_dirs(&self, project: &Project) -> Vec<PathBuf> {
        self.profile
            .copies
            .iter()
            .map(|c| c.trim())
            .filter(|c| !c.is_empty())
            .map(|c| project.resolve(c))
            .collect()
    }
}

/// Todos los perfiles de `courses/`, ordenados por clave. Un perfil roto no
/// impide ver los demás: se devuelve aparte para poder avisar.
/// El perfil de ejemplo que trae el repositorio.
pub const EXAMPLE_PROFILE: &str = "example";

pub fn list_courses(project: &Project) -> (Vec<Course>, Vec<GenerationError>) {
    let mut paths: Vec<PathBuf> = std::fs::read_dir(project.courses_dir())
        .into_iter()
        .flatten()
        .flatten()
        .map(|entry| entry.path())
        .filter(|path| path.extension().is_some_and(|ext| ext == "toml") && path.is_file())
        // `example.toml` es de relleno (docs, `-p example`): no cuenta como perfil
        // propio, así el asistente se abre la primera vez. `find_course` lo sigue leyendo.
        .filter(|path| path.file_stem().is_none_or(|stem| stem != EXAMPLE_PROFILE))
        .collect();
    paths.sort();
    let (mut courses, mut errors) = (Vec::new(), Vec::new());
    for path in paths {
        match Course::load(&path) {
            Ok(course) => courses.push(course),
            Err(error) => {
                logging::warn(format_args!("course profile skipped: {error}"));
                errors.push(error);
            }
        }
    }
    (courses, errors)
}

/// Busca un perfil por clave (sin distinguir mayúsculas) o por ruta.
pub fn find_course(project: &Project, choice: &str) -> Result<Course> {
    // `example.toml` vive en el programa aunque la carpeta de trabajo sea otra.
    let file = format!("{}.toml", choice.trim());
    if let Some(by_key) = [project.courses_dir(), project.root.join("courses")]
        .iter()
        .map(|d| d.join(&file))
        .find(|f| f.is_file())
    {
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

/// Texto TOML entre comillas, con lo necesario escapado.
fn toml_string(value: &str) -> String {
    let mut out = String::from("\"");
    for c in value.chars() {
        match c {
            '\\' => out.push_str("\\\\"),
            '"' => out.push_str("\\\""),
            '\n' => out.push_str("\\n"),
            '\t' => out.push_str("\\t"),
            c if c.is_control() => out.push_str(&format!("\\u{:04X}", c as u32)),
            c => out.push(c),
        }
    }
    out.push('"');
    out
}

/// Guarda el perfil en `courses/<clave>.toml` (solo los datos no vacíos) y
/// devuelve la ruta. Si ya existía, se reemplaza.
pub fn save_profile(project: &Project, key: &str, profile: &CourseProfile) -> Result<PathBuf> {
    let key = crate::document::slugify(key);
    let mut text = String::new();
    let entries = [
        ("name", &profile.name),
        ("university", &profile.university),
        ("faculty", &profile.faculty),
        ("student", &profile.student),
        ("members", &profile.members),
        ("teacher", &profile.teacher),
        ("group", &profile.group),
        ("design", &profile.design),
        ("language", &profile.language),
        ("folder", &profile.folder),
        ("output", &profile.output),
    ];
    for (name, value) in entries {
        if !value.trim().is_empty() {
            text.push_str(&format!("{name} = {}\n", toml_string(value.trim())));
        }
    }
    let copies: Vec<String> =
        profile.copies.iter().map(|c| c.trim()).filter(|c| !c.is_empty()).map(toml_string).collect();
    if !copies.is_empty() {
        text.push_str(&format!("copies = [{}]\n", copies.join(", ")));
    }
    let fields: Vec<_> = profile.fields.iter().filter(|(_, v)| !v.trim().is_empty()).collect();
    if !fields.is_empty() {
        text.push_str("\n[fields]\n");
        for (name, value) in fields {
            text.push_str(&format!("{name} = {}\n", toml_string(value.trim())));
        }
    }
    std::fs::create_dir_all(project.courses_dir())?;
    let path = project.courses_dir().join(format!("{key}.toml"));
    replace_file(&path, |temporary| std::fs::write(temporary, &text))?;
    logging::info(format_args!("course profile saved: {}", path.display()));
    Ok(path)
}

#[cfg(test)]
mod tests {

    #[test]
    fn the_example_profile_is_not_listed_but_can_be_used() {
        let directory = tempfile::tempdir().unwrap();
        let project = Project::at(directory.path());
        std::fs::create_dir_all(project.courses_dir()).unwrap();
        std::fs::write(project.courses_dir().join("example.toml"), "name = \"Materia\"\n").unwrap();
        assert!(list_courses(&project).0.is_empty());
        assert_eq!(find_course(&project, "example").unwrap().profile.name, "Materia");
    }
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
    fn a_saved_profile_reads_back_the_same() {
        let (_dir, project) = project_with(&[]);
        let mut profile = CourseProfile {
            name: "Inteligencia \"artificial\"".into(),
            university: "Universidad \\ Norte".into(),
            design: "report".into(),
            folder: "IA".into(),
            output: "salida/IA".into(),
            copies: vec!["/media/usb".into(), "Drive \"IA\"".into()],
            ..Default::default()
        };
        profile.fields.insert("SALON".into(), "B-204".into());
        let path = save_profile(&project, "IA Uno", &profile).unwrap();
        assert!(path.ends_with("ia-uno.toml"));
        let course = find_course(&project, "ia-uno").unwrap();
        assert_eq!(course.profile, profile);
        // `output` gana sobre `folder`; las rutas relativas parten de la raíz.
        assert_eq!(course.output_dir(&project), project.root.join("salida/IA"));
        assert_eq!(course.copy_dirs(&project)[1], project.root.join("Drive \"IA\""));
    }

    #[test]
    fn an_old_semester_is_read_but_not_saved() {
        let (_dir, project) =
            project_with(&[("viejo.toml", "name = \"M\"\nsemester = \"7\"\ngroup = \"7-A\"\n")]);
        let course = find_course(&project, "viejo").unwrap();
        assert_eq!(course.profile.semester, "7");
        let path = save_profile(&project, "viejo", &course.profile).unwrap();
        let saved = std::fs::read_to_string(path).unwrap();
        assert!(!saved.contains("semester") && saved.contains("group = \"7-A\""), "{saved}");
    }

    #[test]
    fn field_keys_are_read_in_capitals() {
        let text = "name = \"M\"\n[fields]\nsalon = \"viejo\"\nSALON = \"nuevo\"\naula = \"B\"\n";
        let (_dir, project) = project_with(&[("m.toml", text)]);
        let fields = find_course(&project, "m").unwrap().profile.fields;
        assert_eq!(
            fields.into_iter().collect::<Vec<_>>(),
            [("AULA".to_owned(), "B".to_owned()), ("SALON".to_owned(), "nuevo".to_owned())]
        );
    }

    #[test]
    fn the_old_subject_key_is_still_accepted() {
        let (_dir, project) = project_with(&[("viejo.toml", "subject = \"Algebra\"\n")]);
        assert_eq!(find_course(&project, "viejo").unwrap().profile.name, "Algebra");
    }
}
