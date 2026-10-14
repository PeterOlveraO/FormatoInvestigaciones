//! Carpetas del proyecto: dónde están las plantillas y la caché (la raíz del
//! programa) y dónde los trabajos, los PDF, los perfiles, los diseños propios y
//! los logos (la carpeta de trabajo, que por omisión es la misma raíz).

use std::path::{Path, PathBuf};

use crate::error::{GenerationError, Result};
use crate::markdown::expand_home;
use crate::settings::Settings;

/// Archivo que identifica la raíz del proyecto.
const ROOT_MARKER: &str = "templates/common/investigacion-base.sty";
/// Diseño y formato por omisión: la portada geométrica con APA 7.
pub const DEFAULT_TEMPLATE: &str = "geometric-cover";
pub const DEFAULT_FORMAT: &str = "apa7";
pub const TEMPLATE_FILE: &str = "template.ltx";
pub const FORMAT_FILE: &str = "format.sty";

/// Nombres de plantilla de la versión anterior, que siguen valiendo.
pub fn legacy_design(name: &str) -> &str {
    match name {
        "apa" => "geometric-cover",
        "apa-simple" => "classic-cover",
        other => other,
    }
}
pub const LOGOS_DIRECTORY: &str = "logos";
/// Claves de `settings.toml` con la carpeta de trabajo, la española primero.
pub const WORKSPACE_SETTINGS: [&str; 2] = ["CARPETA_TRABAJO", "INVESTIGACION_WORKSPACE"];
/// Lo que se crea en una carpeta de trabajo nueva.
pub const WORKSPACE_DIRS: [&str; 6] =
    ["input", "output", "courses", "my-templates/designs", "my-templates/formats", LOGOS_DIRECTORY];
/// Nombres que busca la portada dentro de la carpeta de logos.
pub const LOGO_FILES: [&str; 2] = ["logo-universidad.png", "logo-facultad.png"];

/// Ruta absoluta sin resolver enlaces. Se evita `canonicalize()` porque en
/// Windows devuelve rutas `\\?\C:\...` que pdflatex y Pandoc no entienden.
pub fn absolute(path: &Path) -> PathBuf {
    std::path::absolute(path).unwrap_or_else(|_| path.to_path_buf())
}

#[derive(Debug, Clone)]
pub struct Project {
    /// La carpeta del programa: plantillas, ejemplos, caché y `settings.toml`.
    pub root: PathBuf,
    /// La carpeta de trabajo de la persona; por omisión, `root`.
    pub work: PathBuf,
}

/// Copia `from` dentro de `to` sin pisar nada que ya exista.
fn copy_missing(from: &Path, to: &Path) -> std::io::Result<()> {
    for entry in std::fs::read_dir(from)?.flatten() {
        let target = to.join(entry.file_name());
        if entry.path().is_dir() {
            std::fs::create_dir_all(&target)?;
            copy_missing(&entry.path(), &target)?;
        } else if !target.exists() {
            std::fs::copy(entry.path(), &target)?;
        }
    }
    Ok(())
}

impl Project {
    pub fn at(root: impl Into<PathBuf>) -> Self {
        let root = absolute(&root.into());
        Self { work: root.clone(), root }
    }

    /// La carpeta de trabajo guardada (`CARPETA_TRABAJO`), si todavía existe.
    pub fn saved_workspace(&self) -> Option<PathBuf> {
        let settings = Settings::load(&self.settings_file()).unwrap_or_default();
        Some(settings.get(&WORKSPACE_SETTINGS))
            .filter(|dir| !dir.is_empty())
            .map(|dir| absolute(&expand_home(Path::new(&dir))))
            .filter(|dir| dir.is_dir())
    }

    /// Prepara `dir` como carpeta de trabajo: crea sus carpetas, copia lo que
    /// ya había en el programa (trabajos, perfiles, diseños propios, logos) sin
    /// pisar nada, la guarda en `settings.toml` y la usa desde ahora.
    pub fn create_workspace(&mut self, dir: &Path) -> Result<()> {
        let dir = absolute(dir);
        for sub in WORKSPACE_DIRS {
            std::fs::create_dir_all(dir.join(sub))?;
        }
        if dir != self.root {
            let previous = [
                (self.root.join("input"), "input"),
                (self.root.join("courses"), "courses"),
                (self.root.join("my-templates"), "my-templates"),
                (self.templates_dir().join(LOGOS_DIRECTORY), LOGOS_DIRECTORY),
            ];
            for (from, to) in previous.iter().filter(|(from, _)| from.is_dir()) {
                copy_missing(from, &dir.join(to))?;
            }
        }
        Settings::save_value(&self.settings_file(), WORKSPACE_SETTINGS[0], &dir.display().to_string())?;
        crate::logging::info(format_args!("workspace: {}", dir.display()));
        self.work = dir;
        Ok(())
    }

    /// Una ruta escrita por la persona (perfil, formulario): `~` es la carpeta
    /// personal y lo relativo parte de la carpeta de trabajo.
    pub fn resolve(&self, path: &str) -> PathBuf {
        self.work.join(expand_home(Path::new(path)))
    }

    /// Raíz del proyecto: `INVESTIGACION_HOME`, luego el primer ancestro del
    /// directorio actual que tenga las plantillas y, por último, la carpeta
    /// donde se compiló el binario (así `cargo install` funciona desde fuera).
    pub fn discover() -> Self {
        let cwd = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
        let built = Path::new(env!("CARGO_MANIFEST_DIR"));
        let root = match std::env::var_os("INVESTIGACION_HOME").filter(|v| !v.is_empty()) {
            Some(home) => PathBuf::from(home),
            None => cwd
                .ancestors()
                .chain([built])
                .find(|dir| dir.join(ROOT_MARKER).is_file())
                .map_or(cwd.clone(), Path::to_path_buf),
        };
        let mut project = Self::at(root);
        if let Some(work) = project.saved_workspace() {
            project.work = work;
        }
        project
    }

    /// Si la raíz tiene las plantillas; si no, `discover` cayó en una carpeta
    /// cualquiera y no conviene escribir nada en ella.
    pub fn has_templates(&self) -> bool {
        self.root.join(ROOT_MARKER).is_file()
    }

    /// Error claro cuando no se encontró la carpeta del proyecto (por ejemplo,
    /// si se movió o borró el clon después de `cargo install`).
    pub fn require_templates(&self) -> Result<()> {
        if self.has_templates() {
            return Ok(());
        }
        Err(GenerationError::new(tr!(
            es: "No se encontró la carpeta del proyecto (la que tiene templates/). Ejecuta el programa dentro de ella, define INVESTIGACION_HOME=<carpeta> o vuelve a instalar con «cargo install --path .» desde ella.",
            en: "The project folder (the one with templates/) was not found. Run the program inside it, set INVESTIGACION_HOME=<folder>, or reinstall with \"cargo install --path .\" from it."
        )))
    }

    pub fn input_dir(&self) -> PathBuf {
        self.work.join("input")
    }
    pub fn output_dir(&self) -> PathBuf {
        self.work.join("output")
    }
    pub fn templates_dir(&self) -> PathBuf {
        self.root.join("templates")
    }
    pub fn common_dir(&self) -> PathBuf {
        self.templates_dir().join("common")
    }
    pub fn courses_dir(&self) -> PathBuf {
        self.work.join("courses")
    }
    /// Los formatos y diseños propios (`formats/`, `designs/`).
    pub fn user_templates_dir(&self) -> PathBuf {
        self.work.join("my-templates")
    }
    pub fn cache_dir(&self) -> PathBuf {
        self.root.join("cache")
    }
    /// Lo que el programa recuerda (idioma, tema, carpeta de trabajo, logos).
    /// Los datos de la portada van en los perfiles.
    pub fn settings_file(&self) -> PathBuf {
        self.root.join("settings.toml")
    }

    /// Crea y devuelve las carpetas de caché: imágenes descargadas y diagramas.
    /// Se pueden borrar enteras; se vuelven a llenar solas.
    pub fn media_directories(&self) -> Result<(PathBuf, PathBuf)> {
        let remote = self.cache_dir().join("remote");
        let diagrams = self.cache_dir().join("diagrams");
        for directory in [&remote, &diagrams] {
            std::fs::create_dir_all(directory)?;
        }
        Ok((remote, diagrams))
    }

    /// De dónde salen los logos: la carpeta indicada (que debe existir), la
    /// `logos/` junto a la plantilla, la de la carpeta de trabajo o la
    /// compartida `templates/logos/`.
    /// Que no haya ninguna no es error: la portada se compila sin logos.
    pub fn resolve_logos_directory(
        &self,
        template: &Path,
        explicit: Option<&Path>,
    ) -> Result<Option<PathBuf>> {
        if let Some(explicit) = explicit {
            let directory = absolute(&expand_home(explicit));
            if !directory.is_dir() {
                return Err(GenerationError::new(tr!(
                    es: "No existe la carpeta de logos: {}",
                    en: "The logos folder does not exist: {}",
                    directory.display()
                )));
            }
            return Ok(Some(directory));
        }
        let beside = template.parent().map(|dir| dir.join(LOGOS_DIRECTORY));
        // Las compartidas cuentan solo si traen un logo: una `logos/` vacía de
        // la carpeta de trabajo no debe tapar a `templates/logos/`.
        let shared = [self.work.join(LOGOS_DIRECTORY), self.templates_dir().join(LOGOS_DIRECTORY)];
        let has_logo = |dir: &PathBuf| LOGO_FILES.iter().any(|logo| dir.join(logo).is_file());
        Ok(beside.filter(|dir| dir.is_dir()).or_else(|| shared.into_iter().find(has_logo)))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn an_explicit_logos_directory_wins_and_must_exist() {
        let directory = tempfile::tempdir().unwrap();
        let project = Project::at(directory.path());
        let template = directory.path().join("base.ltx");
        let own = directory.path().join("mis-logos");
        std::fs::create_dir(&own).unwrap();
        assert_eq!(project.resolve_logos_directory(&template, Some(&own)).unwrap(), Some(own));
        assert!(project.resolve_logos_directory(&template, Some(&directory.path().join("no"))).is_err());
        assert_eq!(project.resolve_logos_directory(&template, None).unwrap(), None);
        // Una `logos/` vacía en la carpeta de trabajo no tapa a `templates/logos/`.
        let shared = project.templates_dir().join(LOGOS_DIRECTORY);
        std::fs::create_dir_all(&shared).unwrap();
        std::fs::write(shared.join(LOGO_FILES[0]), "png").unwrap();
        std::fs::create_dir(directory.path().join(LOGOS_DIRECTORY)).unwrap();
        let design = directory.path().join("templates/designs/x/template.ltx");
        assert_eq!(project.resolve_logos_directory(&design, None).unwrap(), Some(shared));
    }

    #[test]
    fn a_new_work_folder_gets_the_user_data_without_overwriting() {
        let program = tempfile::tempdir().unwrap();
        let work = tempfile::tempdir().unwrap();
        let mut project = Project::at(program.path());
        std::fs::create_dir_all(program.path().join("courses")).unwrap();
        std::fs::write(program.path().join("courses/ia.toml"), "viejo").unwrap();
        std::fs::write(program.path().join("courses/fisica.toml"), "f").unwrap();
        std::fs::create_dir_all(work.path().join("courses")).unwrap();
        std::fs::write(work.path().join("courses/ia.toml"), "nuevo").unwrap();
        project.create_workspace(work.path()).unwrap();
        assert_eq!(std::fs::read_to_string(work.path().join("courses/ia.toml")).unwrap(), "nuevo");
        assert!(work.path().join("courses/fisica.toml").is_file() && work.path().join("logos").is_dir());
        assert_eq!(project.courses_dir(), absolute(work.path()).join("courses"));
        assert_eq!(project.saved_workspace(), Some(absolute(work.path())));
    }
}
