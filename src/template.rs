//! Formatos (las normas: APA 7, Harvard, MLA…) y diseños (la portada y el
//! aspecto). Un formato es `templates/formats/<f>/format.sty` + `format.toml`;
//! un diseño es `templates/designs/<d>/template.ltx` + `template.toml`
//! (opcional). Los propios de cada quien van en `my-templates/`, con la misma
//! estructura.

use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};
use std::sync::LazyLock;

use regex::Regex;
use serde::Deserialize;

use crate::encoding::decode_text;
use crate::error::{GenerationError, Result};
use crate::i18n::{self, Lang};
use crate::markdown::expand_home;
use crate::project::{
    DEFAULT_FORMAT, DEFAULT_TEMPLATE, FORMAT_FILE, Project, TEMPLATE_FILE, absolute, legacy_design,
};

/// Marcadores que el programa llena por su cuenta. Cualquier otro `%%NOMBRE%%`
/// de un diseño es un campo propio (como `%%SALON%%`).
pub const STANDARD_MARKERS: [&str; 13] = [
    "TITULO",
    "UNIVERSIDAD",
    "FACULTAD",
    "ALUMNO",
    "INTEGRANTES",
    "MATERIA",
    "DOCENTE",
    "GRUPO",
    "SEMESTRE",
    "FECHA_ENTREGA",
    "CONTENIDO_MARKDOWN",
    "FORMAT",
    "CLASS_OPTIONS",
];

/// Texto de una ficha: uno solo (`"APA 7"`) o en los dos idiomas
/// (`{ es = "…", en = "…" }`).
#[derive(Debug, Clone, Deserialize, PartialEq, Eq)]
#[serde(untagged)]
pub enum Localized {
    Plain(String),
    Both { es: String, en: String },
}

impl Localized {
    /// El texto en el idioma de la interfaz.
    pub fn get(&self) -> &str {
        match (self, i18n::current()) {
            (Localized::Plain(text), _) => text,
            (Localized::Both { es, .. }, Lang::Es) => es,
            (Localized::Both { en, .. }, Lang::En) => en,
        }
    }
}

/// `format.toml`: el nombre de la norma y lo que el programa necesita saber de ella.
#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct FormatManifest {
    pub name: Localized,
    /// Idioma del documento por omisión (`es` o `en`); el perfil puede cambiarlo.
    #[serde(default = "default_language")]
    pub language: String,
    /// Opciones de `\documentclass` (tamaño de letra, papel, columnas).
    #[serde(default)]
    pub class_options: String,
    /// Estructura recomendada; vacía = no se valida.
    #[serde(default)]
    pub headings: Vec<String>,
}

fn default_language() -> String {
    "es".to_owned()
}

/// Descripción de un campo propio del diseño.
#[derive(Debug, Clone, Default, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct FieldSpec {
    pub label: Option<Localized>,
    pub help: Option<Localized>,
    #[serde(default)]
    pub required: bool,
}

/// `template.toml`: opcional. Sin él, el diseño sirve con cualquier formato.
#[derive(Debug, Clone, Default, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DesignManifest {
    pub description: Option<Localized>,
    /// Formatos con los que combina; vacío = con todos. El primero es el de omisión.
    #[serde(default)]
    pub formats: Vec<String>,
    #[serde(default)]
    pub fields: BTreeMap<String, FieldSpec>,
    /// Marcadores estándar que el diseño usa pero no exige (`UNIVERSIDAD`…).
    #[serde(default)]
    pub optional: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct Format {
    pub key: String,
    pub sty: PathBuf,
    pub manifest: FormatManifest,
}

impl Format {
    pub fn language(&self) -> Lang {
        i18n::parse(&self.manifest.language).unwrap_or(Lang::Es)
    }
}

#[derive(Debug, Clone)]
pub struct Design {
    pub key: String,
    pub file: PathBuf,
    pub manifest: DesignManifest,
    /// Marcadores que usa el `template.ltx`, sin los `%%`.
    pub markers: BTreeSet<String>,
}

impl Design {
    pub fn uses(&self, marker: &str) -> bool {
        self.markers.contains(marker)
    }

    /// Un diseño sin `%%FORMAT%%` trae sus propias reglas: no lleva formato.
    pub fn is_self_contained(&self) -> bool {
        !self.uses("FORMAT")
    }

    /// Campos propios: los marcadores que no son del programa.
    pub fn custom_fields(&self) -> Vec<&str> {
        self.markers.iter().map(String::as_str).filter(|m| !STANDARD_MARKERS.contains(m)).collect()
    }

    pub fn field_spec(&self, name: &str) -> FieldSpec {
        self.manifest.fields.get(name).cloned().unwrap_or_default()
    }

    pub fn accepts(&self, format: &str) -> bool {
        self.manifest.formats.is_empty() || self.manifest.formats.iter().any(|f| f == format)
    }
}

/// Los `%%NOMBRE%%` de un texto, sin los `%%`. Las líneas de comentario de
/// LaTeX no cuentan, para poder mencionar un marcador al explicarlo.
pub fn extract_markers(text: &str) -> BTreeSet<String> {
    static MARKER: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"%%([A-Z_]+)%%").unwrap());
    text.lines()
        .filter(|line| !line.trim_start().starts_with('%') || line.trim_start().starts_with("%%"))
        .flat_map(|line| MARKER.captures_iter(line).map(|c| c[1].to_owned()).collect::<Vec<_>>())
        .collect()
}

fn read_toml<T: for<'de> Deserialize<'de>>(path: &Path) -> Result<T> {
    let text = decode_text(&std::fs::read(path)?, path)?;
    // El detalle (`e.message()`) lo redacta el crate toml y va en inglés.
    toml::from_str(&text).map_err(|e| {
        GenerationError::new(tr!(
            es: "La ficha {} no es válida: {}",
            en: "The file {} is not valid: {}",
            path.display(),
            e.message()
        ))
    })
}

/// Carpetas donde buscar, en orden: las incluidas y luego las de cada quien.
fn roots(project: &Project, kind: &str) -> [PathBuf; 2] {
    [project.templates_dir().join(kind), project.root.join("my-templates").join(kind)]
}

fn list_keys(project: &Project, kind: &str, file: &str) -> Vec<String> {
    let mut keys: Vec<String> = roots(project, kind)
        .iter()
        .flat_map(|root| std::fs::read_dir(root).into_iter().flatten().flatten())
        .filter(|entry| entry.path().join(file).is_file())
        .map(|entry| entry.file_name().to_string_lossy().into_owned())
        .collect();
    keys.sort();
    keys.dedup();
    keys
}

pub fn list_formats(project: &Project) -> Vec<String> {
    list_keys(project, "formats", FORMAT_FILE)
}

pub fn list_designs(project: &Project) -> Vec<String> {
    list_keys(project, "designs", TEMPLATE_FILE)
}

pub fn load_format(project: &Project, key: &str) -> Result<Format> {
    let key = key.trim();
    let Some(dir) =
        roots(project, "formats").into_iter().map(|r| r.join(key)).find(|d| d.join(FORMAT_FILE).is_file())
    else {
        let available = list_formats(project).join(", ");
        return Err(GenerationError::new(tr!(
            es: "No se encontró el formato {key}. Formatos disponibles: {available}.",
            en: "Format {key} not found. Available formats: {available}."
        )));
    };
    let manifest_path = dir.join("format.toml");
    let manifest = if manifest_path.is_file() {
        read_toml(&manifest_path)?
    } else {
        FormatManifest {
            name: Localized::Plain(key.to_owned()),
            language: default_language(),
            class_options: String::new(),
            headings: Vec::new(),
        }
    };
    Ok(Format { key: key.to_owned(), sty: dir.join(FORMAT_FILE), manifest })
}

/// Un diseño por nombre (los antiguos `apa`/`apa-simple` también valen), por
/// carpeta o por ruta al `.ltx`. Sin nada, el diseño por omisión.
pub fn load_design(project: &Project, choice: Option<&str>) -> Result<Design> {
    let choice = choice.map(str::trim).filter(|c| !c.is_empty()).unwrap_or(DEFAULT_TEMPLATE);
    let name = legacy_design(choice);
    let by_name =
        roots(project, "designs").into_iter().map(|r| r.join(name).join(TEMPLATE_FILE)).find(|f| f.is_file());
    let path = absolute(&expand_home(Path::new(choice)));
    let file = by_name
        .or_else(|| path.is_file().then(|| path.clone()))
        .or_else(|| path.join(TEMPLATE_FILE).is_file().then(|| path.join(TEMPLATE_FILE)));
    let Some(file) = file else {
        let available = list_designs(project);
        let available =
            if available.is_empty() { tr!(es: "ninguno", en: "none") } else { available.join(", ") };
        return Err(GenerationError::new(tr!(
            es: "No se encontró el diseño {choice}. Diseños disponibles: {available}.",
            en: "Design {choice} not found. Available designs: {available}."
        )));
    };
    let dir = file.parent().unwrap_or(Path::new(".")).to_path_buf();
    let manifest_path = dir.join("template.toml");
    let manifest =
        if manifest_path.is_file() { read_toml(&manifest_path)? } else { DesignManifest::default() };
    let text = decode_text(&std::fs::read(&file)?, &file)?;
    let key = dir.file_name().map(|n| n.to_string_lossy().into_owned()).unwrap_or_default();
    Ok(Design { key, file, manifest, markers: extract_markers(&text) })
}

/// Diseño + formato listos para generar. El formato es `None` en un diseño
/// autocontenido.
#[derive(Debug, Clone)]
pub struct Layout {
    pub design: Design,
    pub format: Option<Format>,
}

/// Resuelve la pareja diseño/formato. Sin formato indicado se usa el primero
/// que el diseño declara (o APA 7); uno que el diseño no acepta es un error.
pub fn resolve_layout(project: &Project, design: Option<&str>, format: Option<&str>) -> Result<Layout> {
    let design = load_design(project, design)?;
    if design.is_self_contained() {
        return Ok(Layout { design, format: None });
    }
    let chosen = format
        .map(str::trim)
        .filter(|f| !f.is_empty())
        .map(str::to_owned)
        .or_else(|| design.manifest.formats.first().cloned())
        .unwrap_or_else(|| DEFAULT_FORMAT.to_owned());
    if !design.accepts(&chosen) {
        let compatible = design.manifest.formats.join(", ");
        let key = &design.key;
        return Err(GenerationError::new(tr!(
            es: "El diseño {key} no combina con el formato {chosen}. Formatos compatibles: {compatible}.",
            en: "The design {key} does not work with the format {chosen}. Compatible formats: {compatible}."
        )));
    }
    let format = load_format(project, &chosen)?;
    Ok(Layout { design, format: Some(format) })
}

impl Layout {
    /// Idioma del documento: el indicado, si no el del formato, si no español.
    pub fn document_language(&self, chosen: Option<Lang>) -> Lang {
        chosen.or_else(|| self.format.as_ref().map(Format::language)).unwrap_or(Lang::Es)
    }

    /// Lo que va en `%%FORMAT%%`: babel con el idioma del documento y el formato.
    pub fn format_block(&self, lang: Lang) -> String {
        let babel = match lang {
            Lang::Es => "spanish, es-tabla",
            Lang::En => "english",
        };
        format!("\\usepackage[{babel}]{{babel}}\n\\usepackage{{investigacion-format}}")
    }

    pub fn class_options(&self) -> String {
        self.format.as_ref().map(|f| f.manifest.class_options.clone()).unwrap_or_default()
    }

    pub fn headings(&self) -> Vec<String> {
        self.format.as_ref().map(|f| f.manifest.headings.clone()).unwrap_or_default()
    }

    /// Clave para la caché de LaTeX: cambia si cambia el diseño o el formato.
    pub fn cache_key(&self) -> String {
        let format = self.format.as_ref().map(|f| f.key.as_str()).unwrap_or("-");
        format!("{}|{}", self.design.file.display(), format)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn write(path: &Path, text: &str) {
        std::fs::create_dir_all(path.parent().unwrap()).unwrap();
        std::fs::write(path, text).unwrap();
    }

    fn sample() -> (tempfile::TempDir, Project) {
        let directory = tempfile::tempdir().unwrap();
        let project = Project::at(directory.path());
        let t = project.templates_dir();
        write(&t.join("formats/apa7/format.sty"), "x");
        write(
            &t.join("formats/apa7/format.toml"),
            "name = \"APA 7\"\nclass_options = \"12pt\"\nheadings = [\"Introducción\"]\n",
        );
        write(&t.join("formats/mla/format.sty"), "x");
        write(
            &t.join("formats/mla/format.toml"),
            "name = { es = \"MLA\", en = \"MLA\" }\nlanguage = \"en\"\n",
        );
        write(
            &t.join("designs/cover/template.ltx"),
            "\\documentclass[%%CLASS_OPTIONS%%]{article}\n% comentario con %%IGNORADO%%\n%%FORMAT%%\n%%TITULO%% %%SALON%%\n%%CONTENIDO_MARKDOWN%%\n",
        );
        write(
            &t.join("designs/cover/template.toml"),
            "formats = [\"apa7\"]\n[fields.SALON]\nlabel = { es = \"Salón\", en = \"Room\" }\nrequired = true\n",
        );
        write(&project.root.join("my-templates/designs/mine/template.ltx"), "%%CONTENIDO_MARKDOWN%%\n");
        (directory, project)
    }

    #[test]
    fn markers_and_custom_fields_are_detected() {
        let (_dir, project) = sample();
        let design = load_design(&project, Some("cover")).unwrap();
        assert!(design.uses("TITULO") && design.uses("FORMAT") && !design.uses("IGNORADO"));
        assert_eq!(design.custom_fields(), ["SALON"]);
        assert!(design.field_spec("SALON").required);
        i18n::set(Lang::En);
        assert_eq!(design.field_spec("SALON").label.unwrap().get(), "Room");
    }

    #[test]
    fn layouts_check_compatibility_and_defaults() {
        let (_dir, project) = sample();
        let layout = resolve_layout(&project, Some("cover"), None).unwrap();
        assert_eq!(layout.format.as_ref().unwrap().key, "apa7");
        assert_eq!(layout.class_options(), "12pt");
        assert_eq!(layout.headings(), ["Introducción"]);
        assert!(resolve_layout(&project, Some("cover"), Some("mla")).is_err());
        assert!(resolve_layout(&project, Some("cover"), Some("nope")).is_err());
        // Diseño propio en my-templates/ y autocontenido (sin %%FORMAT%%).
        let mine = resolve_layout(&project, Some("mine"), Some("mla")).unwrap();
        assert!(mine.format.is_none() && mine.design.is_self_contained());
        assert_eq!(list_designs(&project), ["cover", "mine"]);
        assert_eq!(list_formats(&project), ["apa7", "mla"]);
    }

    #[test]
    fn document_language_and_babel() {
        let (_dir, project) = sample();
        let design = load_design(&project, Some("cover")).unwrap();
        let mla = Layout { design, format: Some(load_format(&project, "mla").unwrap()) };
        assert_eq!(mla.document_language(None), Lang::En);
        assert_eq!(mla.document_language(Some(Lang::Es)), Lang::Es);
        assert!(mla.format_block(Lang::En).contains("[english]{babel}"));
    }

    #[test]
    fn invalid_manifests_are_errors() {
        let (_dir, project) = sample();
        write(&project.templates_dir().join("designs/cover/template.toml"), "formatos = [\"apa7\"]\n");
        assert!(load_design(&project, Some("cover")).is_err());
        assert!(load_design(&project, Some("nada")).is_err());
    }
}
