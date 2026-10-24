//! El formato (APA 7, la única norma) y los diseños (la portada y el aspecto).
//! El formato es `templates/formats/apa7/format.sty` + `format.toml`; un
//! diseño es `templates/designs/<d>/template.ltx` + `template.toml` (opcional).
//! Los diseños propios van en `my-templates/designs/`, con la misma estructura.

use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};
use std::sync::LazyLock;

use regex::Regex;
use serde::Deserialize;

use crate::encoding::decode_text;
use crate::error::{GenerationError, Result};
use crate::i18n::{self, Lang};
use crate::markdown::expand_home;
use crate::project::{DEFAULT_FORMAT, DEFAULT_TEMPLATE, FORMAT_FILE, Project, TEMPLATE_FILE, absolute};

/// Marcadores que el programa llena por su cuenta. Cualquier otro `%%NOMBRE%%`
/// de un diseño es un campo propio (como `%%SALON%%`).
pub const STANDARD_MARKERS: [&str; 12] = [
    "TITULO",
    "UNIVERSIDAD",
    "FACULTAD",
    "ALUMNO",
    "INTEGRANTES",
    "MATERIA",
    "DOCENTE",
    "GRUPO",
    "FECHA_ENTREGA",
    "CONTENIDO_MARKDOWN",
    "FORMAT",
    "CLASS_OPTIONS",
];

/// Marcadores estándar que se exigen si el diseño los usa y no los marca opcionales.
pub const REQUIRED_MARKERS: [&str; 4] = ["TITULO", "UNIVERSIDAD", "FACULTAD", "MATERIA"];

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

/// `format.toml`: lo que el programa necesita saber de la norma.
#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct FormatManifest {
    /// Idioma del documento por omisión (`es` o `en`); el perfil puede cambiarlo.
    #[serde(default = "default_language")]
    pub language: String,
    /// Opciones de `\documentclass` (tamaño de letra, papel, columnas).
    #[serde(default)]
    pub class_options: String,
    /// Estructura recomendada; vacía = no se valida.
    #[serde(default)]
    pub headings: Vec<String>,
    /// Lo mismo en inglés; vacía = con un documento en inglés no se revisa.
    #[serde(default)]
    pub headings_en: Vec<String>,
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

/// `template.toml`: opcional.
#[derive(Debug, Clone, Default, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DesignManifest {
    pub description: Option<Localized>,
    /// Ya no se usa (solo hay APA 7); se acepta para no romper diseños viejos.
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
    pub sty: PathBuf,
    pub manifest: FormatManifest,
}

impl Format {
    /// `load_format` ya rechazó los valores no válidos.
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

    /// Si el diseño exige un dato estándar: lo usa, es de los exigibles y no
    /// lo declara opcional. Lo comparten el CLI, el formulario y el asistente.
    pub fn requires(&self, marker: &str) -> bool {
        REQUIRED_MARKERS.contains(&marker)
            && self.uses(marker)
            && !self.manifest.optional.iter().any(|o| o == marker)
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
}

/// Los `%%NOMBRE%%` de un texto, sin los `%%`. Las líneas de comentario de
/// LaTeX no cuentan, para poder mencionar un marcador al explicarlo.
pub fn extract_markers(text: &str) -> BTreeSet<String> {
    static MARKER: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"%%([A-Z_]+)%%").unwrap());
    text.lines()
        .filter(|line| !crate::latex::is_comment_line(line))
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

/// Carpetas de diseños, en orden: los incluidos y luego los de cada quien.
fn roots(project: &Project) -> [PathBuf; 2] {
    [project.templates_dir().join("designs"), project.user_templates_dir().join("designs")]
}

pub fn list_designs(project: &Project) -> Vec<String> {
    let mut keys: Vec<String> = roots(project)
        .iter()
        .flat_map(|root| std::fs::read_dir(root).into_iter().flatten().flatten())
        .filter(|entry| entry.path().join(TEMPLATE_FILE).is_file())
        .map(|entry| entry.file_name().to_string_lossy().into_owned())
        .collect();
    keys.sort();
    keys.dedup();
    keys
}

/// El formato incluido (APA 7).
pub fn load_format(project: &Project) -> Result<Format> {
    let dir = project.templates_dir().join("formats").join(DEFAULT_FORMAT);
    let manifest: FormatManifest = read_toml(&dir.join("format.toml"))?;
    if i18n::parse(&manifest.language).is_none() {
        let value = manifest.language.as_str();
        return Err(GenerationError::new(tr!(
            es: "El formato tiene un idioma no válido en format.toml: «{value}». Usa es o en.",
            en: "The format has an invalid language in format.toml: \"{value}\". Use es or en."
        )));
    }
    Ok(Format { sty: dir.join(FORMAT_FILE), manifest })
}

/// Diseños que ya no existen: llevan a la portada geométrica, con aviso.
pub const RETIRED_DESIGNS: [&str; 5] = ["apa", "apa-simple", "classic-cover", "report", "starter"];

/// El aviso para quien pidió un diseño retirado y recibió el de omisión. Un
/// diseño propio con ese nombre gana y no lleva aviso.
pub fn retired_design_warning(choice: &str, design: &Design) -> Option<String> {
    let choice = choice.trim();
    (RETIRED_DESIGNS.contains(&choice) && design.key != choice).then(|| {
        tr!(
            es: "El diseño «{choice}» ya no existe; se usa {DEFAULT_TEMPLATE}.",
            en: "The design \"{choice}\" no longer exists; {DEFAULT_TEMPLATE} is used."
        )
    })
}

/// Un diseño por nombre, por carpeta o por ruta al `.ltx`; un nombre retirado
/// lleva al de omisión. Sin nada, el diseño por omisión.
pub fn load_design(project: &Project, choice: Option<&str>) -> Result<Design> {
    let choice = choice.map(str::trim).filter(|c| !c.is_empty()).unwrap_or(DEFAULT_TEMPLATE);
    let by_name = |name: &str| {
        roots(project).into_iter().map(|r| r.join(name).join(TEMPLATE_FILE)).find(|f| f.is_file())
    };
    let path = absolute(&expand_home(Path::new(choice)));
    let file = by_name(choice)
        .or_else(|| path.is_file().then(|| path.clone()))
        .or_else(|| path.join(TEMPLATE_FILE).is_file().then(|| path.join(TEMPLATE_FILE)))
        .or_else(|| RETIRED_DESIGNS.contains(&choice).then(|| by_name(DEFAULT_TEMPLATE)).flatten());
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

/// Carga el diseño y, si no es autocontenido, el formato APA 7.
pub fn resolve_layout(project: &Project, design: Option<&str>) -> Result<Layout> {
    let design = load_design(project, design)?;
    let format = if design.is_self_contained() { None } else { Some(load_format(project)?) };
    Ok(Layout { design, format })
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

    /// Encabezados recomendados en el idioma del documento.
    pub fn headings_for(&self, lang: Lang) -> Vec<String> {
        self.format
            .as_ref()
            .map(|f| match lang {
                Lang::Es => f.manifest.headings.clone(),
                Lang::En => f.manifest.headings_en.clone(),
            })
            .unwrap_or_default()
    }

    /// Clave para la caché de LaTeX: cambia si cambia el diseño o el formato.
    pub fn cache_key(&self) -> String {
        let format = if self.format.is_some() { DEFAULT_FORMAT } else { "-" };
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
            "class_options = \"12pt\"\nheadings = [\"Introducción\"]\nheadings_en = [\"Introduction\"]\n",
        );
        write(
            &t.join("designs/cover/template.ltx"),
            "\\documentclass[%%CLASS_OPTIONS%%]{article}\n% comentario con %%IGNORADO%%\n%%FORMAT%%\n%%TITULO%% %%SALON%%\n%%CONTENIDO_MARKDOWN%%\n",
        );
        write(
            &t.join("designs/cover/template.toml"),
            "[fields.SALON]\nlabel = { es = \"Salón\", en = \"Room\" }\nrequired = true\n",
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
    fn layouts_load_apa7_unless_the_design_is_self_contained() {
        let (_dir, project) = sample();
        let layout = resolve_layout(&project, Some("cover")).unwrap();
        assert!(layout.format.is_some());
        assert_eq!(layout.class_options(), "12pt");
        assert_eq!(layout.headings_for(Lang::Es), ["Introducción"]);
        assert_eq!(layout.headings_for(Lang::En), ["Introduction"]);
        assert_eq!(layout.document_language(None), Lang::Es);
        assert_eq!(layout.document_language(Some(Lang::En)), Lang::En);
        assert!(layout.format_block(Lang::En).contains("[english]{babel}"));
        // Diseño propio en my-templates/ y autocontenido (sin %%FORMAT%%).
        let mine = resolve_layout(&project, Some("mine")).unwrap();
        assert!(mine.format.is_none() && mine.design.is_self_contained());
        assert_eq!(list_designs(&project), ["cover", "mine"]);
    }

    #[test]
    fn format_languages_accept_regions_and_reject_unknown_ones() {
        assert_eq!(i18n::parse("fr"), None);
        let (_dir, project) = sample();
        let manifest = project.templates_dir().join("formats/apa7/format.toml");
        write(&manifest, "language = \"fr\"\n");
        let error = load_format(&project).unwrap_err().to_string();
        assert!(error.contains("«fr»"), "{error}");
        write(&manifest, "language = \"en-US\"\n");
        assert_eq!(load_format(&project).unwrap().language(), Lang::En);
    }

    #[test]
    fn retired_designs_lead_to_the_default_with_a_warning() {
        i18n::set(Lang::Es);
        let (_dir, project) = sample();
        write(
            &project.templates_dir().join("designs/geometric-cover/template.ltx"),
            "%%CONTENIDO_MARKDOWN%%\n",
        );
        for old in RETIRED_DESIGNS {
            let design = load_design(&project, Some(old)).unwrap();
            assert_eq!(design.key, DEFAULT_TEMPLATE);
            assert!(retired_design_warning(old, &design).is_some_and(|w| w.contains(&format!("«{old}»"))));
        }
        // Uno propio con un nombre retirado gana, sin aviso.
        write(&project.root.join("my-templates/designs/report/template.ltx"), "%%CONTENIDO_MARKDOWN%%\n");
        let mine = load_design(&project, Some("report")).unwrap();
        assert_eq!(mine.key, "report");
        assert!(retired_design_warning("report", &mine).is_none());
        let current = load_design(&project, Some("cover")).unwrap();
        assert!(retired_design_warning("cover", &current).is_none());
    }

    #[test]
    fn invalid_manifests_are_errors() {
        let (_dir, project) = sample();
        write(&project.templates_dir().join("designs/cover/template.toml"), "formatos = [\"apa7\"]\n");
        assert!(load_design(&project, Some("cover")).is_err());
        assert!(load_design(&project, Some("nada")).is_err());
    }
}
