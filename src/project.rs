//! Carpetas del proyecto: dónde están las plantillas, los trabajos, la salida
//! y la caché, se ejecute el programa desde donde se ejecute.

use std::path::{Path, PathBuf};

use crate::error::{GenerationError, Result};
use crate::markdown::expand_home;

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
/// Nombres que busca la portada dentro de la carpeta de logos.
pub const LOGO_FILES: [&str; 2] = ["logo-universidad.png", "logo-facultad.png"];

/// Ruta absoluta sin resolver enlaces. Se evita `canonicalize()` porque en
/// Windows devuelve rutas `\\?\C:\...` que pdflatex y Pandoc no entienden.
pub fn absolute(path: &Path) -> PathBuf {
    std::path::absolute(path).unwrap_or_else(|_| path.to_path_buf())
}

#[derive(Debug, Clone)]
pub struct Project {
    pub root: PathBuf,
}

impl Project {
    pub fn at(root: impl Into<PathBuf>) -> Self {
        Self { root: absolute(&root.into()) }
    }

    /// Raíz del proyecto: `INVESTIGACION_HOME`, luego el primer ancestro del
    /// directorio actual que tenga las plantillas y, por último, la carpeta
    /// donde se compiló el binario (así `cargo install` funciona desde fuera).
    pub fn discover() -> Self {
        if let Some(home) = std::env::var_os("INVESTIGACION_HOME").filter(|v| !v.is_empty()) {
            return Self::at(PathBuf::from(home));
        }
        let cwd = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
        if let Some(found) = cwd.ancestors().find(|dir| dir.join(ROOT_MARKER).is_file()) {
            return Self::at(found);
        }
        let built = Path::new(env!("CARGO_MANIFEST_DIR"));
        if built.join(ROOT_MARKER).is_file() {
            return Self::at(built);
        }
        Self::at(cwd)
    }

    /// Si la raíz tiene las plantillas; si no, `discover` cayó en una carpeta
    /// cualquiera y no conviene escribir nada en ella.
    pub fn has_templates(&self) -> bool {
        self.root.join(ROOT_MARKER).is_file()
    }

    pub fn input_dir(&self) -> PathBuf {
        self.root.join("input")
    }
    pub fn output_dir(&self) -> PathBuf {
        self.root.join("output")
    }
    pub fn templates_dir(&self) -> PathBuf {
        self.root.join("templates")
    }
    pub fn designs_dir(&self) -> PathBuf {
        self.templates_dir().join("designs")
    }
    pub fn formats_dir(&self) -> PathBuf {
        self.templates_dir().join("formats")
    }
    /// El `format.sty` de un formato (`apa7`, `harvard`...).
    pub fn format_sty(&self, format: &str) -> PathBuf {
        self.formats_dir().join(format).join(FORMAT_FILE)
    }
    pub fn common_dir(&self) -> PathBuf {
        self.templates_dir().join("common")
    }
    pub fn courses_dir(&self) -> PathBuf {
        self.root.join("courses")
    }
    pub fn cache_dir(&self) -> PathBuf {
        self.root.join("cache")
    }
    /// Lo que el programa recuerda (idioma del menú, carpeta de logos). Los
    /// datos de la portada van en los perfiles.
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
    /// `logos/` junto a la plantilla o la compartida `templates/logos/`.
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
        let shared = self.templates_dir().join(LOGOS_DIRECTORY);
        Ok(beside.into_iter().chain([shared]).find(|dir| dir.is_dir()))
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
    }
}
