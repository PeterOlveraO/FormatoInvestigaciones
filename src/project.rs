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
pub const LOGOS_DIRECTORY: &str = "logos";
/// Claves de `settings.toml` con la carpeta de trabajo, la española primero.
pub const WORKSPACE_SETTINGS: [&str; 2] = ["CARPETA_TRABAJO", "INVESTIGACION_WORKSPACE"];
/// Lo que se crea en una carpeta de trabajo nueva.
pub const WORKSPACE_DIRS: [&str; 5] = ["input", "output", "courses", "my-templates/designs", LOGOS_DIRECTORY];
/// Nombres (sin extensión) que busca la portada dentro de la carpeta de logos.
pub const LOGO_FILES: [&str; 2] = ["logo-universidad", "logo-facultad"];
/// Extensiones de logo en orden de preferencia: los vectores primero.
pub const LOGO_EXTENSIONS: [&str; 6] = ["svg", "pdf", "png", "jpg", "jpeg", "webp"];

/// El primer `<name>.<extensión>` de la carpeta, en el orden de `LOGO_EXTENSIONS`.
pub fn find_logo(directory: &Path, name: &str) -> Option<PathBuf> {
    LOGO_EXTENSIONS.iter().map(|ext| directory.join(format!("{name}.{ext}"))).find(|path| path.is_file())
}

/// Ruta absoluta sin resolver enlaces. Se evita `canonicalize()` porque en
/// Windows devuelve rutas `\\?\C:\...` que pdflatex y Pandoc no entienden.
pub fn absolute(path: &Path) -> PathBuf {
    std::path::absolute(path).unwrap_or_else(|_| path.to_path_buf())
}

/// Reemplaza un archivo de forma segura: escribe en un temporal oculto de la
/// misma carpeta y lo renombra encima, para que un corte no deje el archivo a
/// medias. Si algo falla, borra el temporal y devuelve el error.
// shortcut: si `path` es un enlace simbólico, queda un archivo normal en su lugar;
// si alguien enlaza sus perfiles o ajustes, escribir en el destino del enlace.
pub fn replace_file(path: &Path, write: impl FnOnce(&Path) -> std::io::Result<()>) -> std::io::Result<()> {
    let temporary = temporary_for(path);
    match write(&temporary).and_then(|()| std::fs::rename(&temporary, path)) {
        Ok(()) => Ok(()),
        Err(error) => {
            let _ = std::fs::remove_file(&temporary);
            Err(error)
        }
    }
}

/// El temporal que usa `replace_file` para `path` en este proceso. Con el id
/// del proceso: dos generaciones a la vez no se pisan el temporal.
pub fn temporary_for(path: &Path) -> PathBuf {
    path.with_file_name(format!(
        ".{}.{}.tmp",
        path.file_name().unwrap_or_default().to_string_lossy(),
        std::process::id()
    ))
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
        // `file_type` no sigue enlaces: un enlace a carpeta se salta (no es
        // archivo ni carpeta) y un archivo enlazado se copia igual.
        if entry.file_type().is_ok_and(|kind| kind.is_dir()) {
            std::fs::create_dir_all(&target)?;
            copy_missing(&entry.path(), &target)?;
        } else if !target.exists() && entry.path().is_file() {
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
    /// había en la carpeta de trabajo actual (trabajos, perfiles, diseños
    /// propios, logos) sin pisar nada, la guarda en `settings.toml` y la usa
    /// desde ahora.
    pub fn create_workspace(&mut self, dir: &Path) -> Result<()> {
        let dir = absolute(dir);
        for sub in WORKSPACE_DIRS {
            std::fs::create_dir_all(dir.join(sub))?;
        }
        if dir != self.work {
            // Los datos salen de la carpeta de trabajo actual, no del programa,
            // y los logos compartidos de `templates/logos/` siguen sumándose.
            let previous = [
                (self.work.join("input"), "input"),
                (self.work.join("courses"), "courses"),
                (self.work.join("my-templates"), "my-templates"),
                (self.work.join(LOGOS_DIRECTORY), LOGOS_DIRECTORY),
                (self.templates_dir().join(LOGOS_DIRECTORY), LOGOS_DIRECTORY),
            ];
            // No copiar un origen que contiene a la carpeta nueva: sería copiarse
            // a sí misma sin fin.
            for (from, to) in previous.iter().filter(|(from, _)| from.is_dir()) {
                if dir.starts_with(from) {
                    continue;
                }
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
    /// Los diseños propios (`designs/`).
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

    /// De dónde salen los logos: la carpeta indicada (si no existe, ninguna: el CLI ya avisó), la
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
                crate::logging::warn(format_args!(
                    "logos folder {} not found; no logos",
                    directory.display()
                ));
                return Ok(None);
            }
            return Ok(Some(directory));
        }
        let beside = template.parent().map(|dir| dir.join(LOGOS_DIRECTORY));
        // Las compartidas cuentan solo si traen un logo: una `logos/` vacía de
        // la carpeta de trabajo no debe tapar a `templates/logos/`.
        let shared = [self.work.join(LOGOS_DIRECTORY), self.templates_dir().join(LOGOS_DIRECTORY)];
        let has_logo = |dir: &PathBuf| LOGO_FILES.iter().any(|logo| find_logo(dir, logo).is_some());
        Ok(beside.filter(|dir| dir.is_dir()).or_else(|| shared.into_iter().find(has_logo)))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn an_explicit_logos_directory_wins_and_a_missing_one_means_no_logos() {
        let directory = tempfile::tempdir().unwrap();
        let project = Project::at(directory.path());
        let template = directory.path().join("base.ltx");
        let own = directory.path().join("mis-logos");
        std::fs::create_dir(&own).unwrap();
        assert_eq!(project.resolve_logos_directory(&template, Some(&own)).unwrap(), Some(own));
        // Que no exista no es error ni cae en otra carpeta: la portada sale sin logos.
        std::fs::create_dir_all(project.templates_dir().join(LOGOS_DIRECTORY)).unwrap();
        std::fs::write(project.templates_dir().join(LOGOS_DIRECTORY).join("logo-facultad.png"), "x").unwrap();
        assert_eq!(
            project.resolve_logos_directory(&template, Some(&directory.path().join("no"))).unwrap(),
            None
        );
        std::fs::remove_dir_all(project.templates_dir().join(LOGOS_DIRECTORY)).unwrap();
        assert_eq!(project.resolve_logos_directory(&template, None).unwrap(), None);
        // Una `logos/` vacía en la carpeta de trabajo no tapa a `templates/logos/`.
        let shared = project.templates_dir().join(LOGOS_DIRECTORY);
        std::fs::create_dir_all(&shared).unwrap();
        std::fs::write(shared.join("logo-universidad.webp"), "webp").unwrap();
        std::fs::create_dir(directory.path().join(LOGOS_DIRECTORY)).unwrap();
        let design = directory.path().join("templates/designs/x/template.ltx");
        assert_eq!(project.resolve_logos_directory(&design, None).unwrap(), Some(shared));
    }

    #[test]
    fn logos_are_looked_up_vector_first() {
        let directory = tempfile::tempdir().unwrap();
        assert_eq!(find_logo(directory.path(), "logo-facultad"), None);
        for extension in LOGO_EXTENSIONS.iter().rev() {
            let path = directory.path().join(format!("logo-facultad.{extension}"));
            std::fs::write(&path, "").unwrap();
            assert_eq!(find_logo(directory.path(), "logo-facultad"), Some(path));
        }
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

    #[test]
    fn a_workspace_inside_input_does_not_copy_itself() {
        let program = tempfile::tempdir().unwrap();
        let mut project = Project::at(program.path());
        let paper = program.path().join("input").join("IA").join("Tarea.md");
        std::fs::create_dir_all(paper.parent().unwrap()).unwrap();
        std::fs::write(&paper, "# Introducción\n").unwrap();
        let workspace = program.path().join("input");
        project.create_workspace(&workspace).unwrap();
        assert!(!workspace.join("input").join("input").exists());
    }

    #[test]
    fn a_new_folder_gets_the_profiles_of_the_current_one() {
        let program = tempfile::tempdir().unwrap();
        let first = tempfile::tempdir().unwrap();
        let second = tempfile::tempdir().unwrap();
        let mut project = Project::at(program.path());
        project.create_workspace(first.path()).unwrap();
        let profile = first.path().join("courses").join("ia.toml");
        std::fs::write(&profile, "perfil").unwrap();
        project.create_workspace(second.path()).unwrap();
        assert_eq!(std::fs::read_to_string(second.path().join("courses/ia.toml")).unwrap(), "perfil");
        assert_eq!(std::fs::read_to_string(&profile).unwrap(), "perfil");
        assert_eq!(project.work, absolute(second.path()));
    }

    #[test]
    fn replace_file_swaps_the_content_without_leaving_temporaries() {
        let directory = tempfile::tempdir().unwrap();
        let path = directory.path().join("datos.toml");
        std::fs::write(&path, "viejo").unwrap();
        replace_file(&path, |temporary| std::fs::write(temporary, "nuevo")).unwrap();
        assert_eq!(std::fs::read_to_string(&path).unwrap(), "nuevo");
        assert_eq!(std::fs::read_dir(directory.path()).unwrap().count(), 1);
    }

    #[test]
    fn a_failing_write_keeps_the_original_file() {
        let directory = tempfile::tempdir().unwrap();
        let path = directory.path().join("datos.toml");
        std::fs::write(&path, "viejo").unwrap();
        let error = replace_file(&path, |_| Err(std::io::Error::other("sin espacio"))).unwrap_err();
        assert_eq!(error.kind(), std::io::ErrorKind::Other);
        assert_eq!(std::fs::read_to_string(&path).unwrap(), "viejo");
        assert_eq!(std::fs::read_dir(directory.path()).unwrap().count(), 1);
    }

    // Crear enlaces simbólicos en Windows pide permisos especiales.
    #[cfg(unix)]
    #[test]
    fn a_folder_link_is_skipped_and_a_file_link_is_copied() {
        let program = tempfile::tempdir().unwrap();
        let work = tempfile::tempdir().unwrap();
        let mut project = Project::at(program.path());
        let input = program.path().join("input");
        std::fs::create_dir_all(input.join("IA")).unwrap();
        std::fs::write(input.join("IA/Tarea.md"), "x").unwrap();
        // Un enlace a su propia carpeta padre: seguirlo sería un ciclo sin fin.
        std::os::unix::fs::symlink(&input, input.join("IA/atajo")).unwrap();
        std::os::unix::fs::symlink(input.join("IA/Tarea.md"), input.join("enlace.md")).unwrap();
        project.create_workspace(work.path()).unwrap();
        assert!(work.path().join("input/IA/Tarea.md").is_file());
        assert!(!work.path().join("input/IA/atajo").exists());
        assert_eq!(std::fs::read_to_string(work.path().join("input/enlace.md")).unwrap(), "x");
    }
}
