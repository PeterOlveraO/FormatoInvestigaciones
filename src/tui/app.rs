//! Estado del menú interactivo y su respuesta a cada tecla. No dibuja nada:
//! eso lo hace `ui.rs`, así que esta parte se puede probar sin terminal.

use std::path::{Path, PathBuf};
use std::sync::mpsc::{self, Receiver, TryRecvError};

use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

use super::picker::{Item, PickOutcome, PickValue, Picker, PickerPurpose};
use crate::cli::{Args, Reporter, execute};
use std::collections::BTreeMap;

use super::wizard::{Outcome as WizardOutcome, Wizard};
use crate::courses::{Course, list_courses, save_profile};
use crate::i18n::{self, LANG_SETTINGS, Lang, Text};
use crate::project::Project;
use crate::settings::Settings;
use crate::template::Design;

/// Campos del formulario, en el orden en que se muestran.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FieldKey {
    Profile,
    Markdown,
    Title,
    FileName,
    Course,
    Teacher,
    Members,
    Group,
    Format,
    Template,
    Output,
    Copies,
}

/// Cómo se edita un campo: escribiendo o eligiendo de una lista.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FieldKind {
    Text,
    Pick(PickerPurpose),
}

#[derive(Debug, Clone, Copy)]
pub struct Field {
    pub key: FieldKey,
    pub label: Text,
    pub kind: FieldKind,
    pub required: bool,
    pub help: Text,
    pub example: Text,
    /// Qué pasa si se deja vacío (solo campos opcionales).
    pub empty: Text,
}

const NOTHING: Text = Text::new("", "");

/// El formulario. Ya no incluye el archivo .env, «permitir LaTeX», la
/// plantilla por ruta ni la carpeta de logos (las opciones 9 a 12 del menú
/// anterior): siguen disponibles en el CLI para quien las necesite.
pub const FIELDS: [Field; 12] = [
    Field {
        key: FieldKey::Profile,
        label: Text::new("Perfil de materia", "Course profile"),
        kind: FieldKind::Pick(PickerPurpose::Profile),
        required: false,
        help: Text::new(
            "Datos guardados de una materia (courses/*.toml). Al elegirlo se llenan materia, docente, grupo, plantilla y carpeta de salida.",
            "Saved data of a course (courses/*.toml). Choosing one fills course, teacher, group, template and output folder.",
        ),
        example: Text::new("ia", "ia"),
        empty: Text::new("llenas los campos a mano", "fill the fields by hand"),
    },
    Field {
        key: FieldKey::Markdown,
        label: Text::new("Archivo Markdown", "Markdown file"),
        kind: FieldKind::Pick(PickerPurpose::Markdown),
        required: true,
        help: Text::new(
            "El archivo .md con el contenido. Recorre las carpetas con las flechas; escribe para filtrar.",
            "The .md file with the content. Browse the folders with the arrows; type to filter.",
        ),
        example: Text::new("input/IA/Tarea1.md", "input/IA/Tarea1.md"),
        empty: NOTHING,
    },
    Field {
        key: FieldKey::Title,
        label: Text::new("Título", "Title"),
        kind: FieldKind::Text,
        required: true,
        help: Text::new(
            "Título que sale en la portada. No cambia el nombre del archivo.",
            "Title shown on the cover. It does not change the file name.",
        ),
        example: Text::new("Introducción a las bases de datos", "Introduction to databases"),
        empty: NOTHING,
    },
    Field {
        key: FieldKey::FileName,
        label: Text::new("Nombre del PDF", "PDF file name"),
        kind: FieldKind::Text,
        required: false,
        help: Text::new(
            "Nombre del PDF, independiente del título. Los acentos y espacios se simplifican.",
            "Name of the PDF, independent of the title. Accents and spaces are simplified.",
        ),
        example: Text::new("tarea1-ia", "tarea1-ia"),
        empty: Text::new("el nombre del Markdown", "the name of the Markdown file"),
    },
    Field {
        key: FieldKey::Course,
        label: Text::new("Materia", "Course"),
        kind: FieldKind::Text,
        // Obligatoria solo si el diseño la usa: ver `App::is_required`.
        required: false,
        help: Text::new(
            "Nombre de la materia, tal como sale en la portada.",
            "Name of the course, as shown on the cover.",
        ),
        example: Text::new("Inteligencia artificial", "Inteligencia artificial"),
        empty: NOTHING,
    },
    Field {
        key: FieldKey::Teacher,
        label: Text::new("Docente", "Teacher"),
        kind: FieldKind::Text,
        required: false,
        help: Text::new("Nombre del docente.", "Name of the teacher."),
        example: Text::new("Nombre del docente", "Name of the teacher"),
        empty: Text::new("DOCENTE del .env; si no está, no sale", "DOCENTE from .env; if missing, omitted"),
    },
    Field {
        key: FieldKey::Members,
        label: Text::new("Integrantes", "Team members"),
        kind: FieldKind::Text,
        required: false,
        help: Text::new(
            "Nombres del equipo separados por comas. Con integrantes, la portada no muestra al alumno.",
            "Team names separated by commas. With members, the cover does not show the student.",
        ),
        example: Text::new("Ana Ruiz, Luis Paz", "Ana Ruiz, Luis Paz"),
        empty: Text::new(
            "INTEGRANTES del .env; si no está, el alumno",
            "INTEGRANTES from .env; if missing, the student",
        ),
    },
    Field {
        key: FieldKey::Group,
        label: Text::new("Grupo", "Group"),
        kind: FieldKind::Text,
        required: false,
        help: Text::new("Grupo de la materia.", "Group of the course."),
        example: Text::new("7-A", "7-A"),
        empty: Text::new("GRUPO del .env; si no está, no sale", "GRUPO from .env; if missing, omitted"),
    },
    Field {
        key: FieldKey::Format,
        label: Text::new("Formato", "Format"),
        kind: FieldKind::Pick(PickerPurpose::Format),
        required: false,
        help: Text::new(
            "La norma: APA 7, Harvard, MLA… Decide letra, interlineado, encabezados y referencias.",
            "The norm: APA 7, Harvard, MLA… It sets font, spacing, headings and references.",
        ),
        example: Text::new("harvard", "harvard"),
        empty: Text::new("el primero que acepta el diseño", "the first one the design accepts"),
    },
    Field {
        key: FieldKey::Template,
        label: Text::new("Diseño", "Design"),
        kind: FieldKind::Pick(PickerPurpose::Template),
        required: false,
        help: Text::new(
            "Portada y aspecto del documento. Los diseños están en templates/designs/ y en my-templates/designs/.",
            "Cover and look of the document. Designs live in templates/designs/ and my-templates/designs/.",
        ),
        example: Text::new("classic-cover", "classic-cover"),
        empty: Text::new("geometric-cover (APA 7)", "geometric-cover (APA 7)"),
    },
    Field {
        key: FieldKey::Output,
        label: Text::new("Carpeta de salida", "Output folder"),
        kind: FieldKind::Text,
        required: false,
        help: Text::new("Carpeta donde se guarda el PDF.", "Folder where the PDF is saved."),
        example: Text::new("output/IA", "output/IA"),
        empty: Text::new(
            "output/ (u output/<carpeta> del perfil)",
            "output/ (or output/<folder> of the profile)",
        ),
    },
    Field {
        key: FieldKey::Copies,
        label: Text::new("Copias extra", "Extra copies"),
        kind: FieldKind::Text,
        required: false,
        help: Text::new(
            "Carpetas extra para una copia del PDF, separadas por ;",
            "Extra folders for a copy of the PDF, separated by ;",
        ),
        example: Text::new("~/Drive/IA; /media/usb", "~/Drive/IA; /media/usb"),
        empty: Text::new("sin copias", "no copies"),
    },
];

/// Campo propio del diseño elegido (`%%SALON%%`), que se suma al formulario.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExtraField {
    pub name: String,
    pub label: String,
    pub help: String,
    pub required: bool,
    pub value: String,
}

/// Línea del panel de resultados.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LogLine {
    Info(String),
    Warning(String),
    Error(String),
}

/// Campo de texto en edición, con cursor (en caracteres, no en bytes).
#[derive(Debug, Clone, Default)]
pub struct TextInput {
    pub text: String,
    pub cursor: usize,
}

impl TextInput {
    pub fn new(text: &str) -> Self {
        Self { text: text.to_owned(), cursor: text.chars().count() }
    }
    fn byte_at(&self, cursor: usize) -> usize {
        self.text.char_indices().nth(cursor).map_or(self.text.len(), |(i, _)| i)
    }
    pub(crate) fn insert(&mut self, c: char) {
        let at = self.byte_at(self.cursor);
        self.text.insert(at, c);
        self.cursor += 1;
    }
    pub(crate) fn backspace(&mut self) {
        if self.cursor > 0 {
            self.cursor -= 1;
            let at = self.byte_at(self.cursor);
            self.text.remove(at);
        }
    }
    pub(crate) fn delete(&mut self) {
        if self.cursor < self.text.chars().count() {
            let at = self.byte_at(self.cursor);
            self.text.remove(at);
        }
    }
}

pub enum Mode {
    /// Pantalla de la primera vez: elegir el idioma (con el del sistema marcado).
    ChooseLanguage(Lang),
    /// El asistente de perfiles (crear o editar).
    Wizard(Box<Wizard>),
    Form,
    Editing(TextInput),
    Picking(Picker),
    Generating(Receiver<WorkerMessage>),
}

/// Mensajes del hilo que genera el PDF.
pub enum WorkerMessage {
    Line(LogLine),
    Finished(Result<PathBuf, String>),
}

struct ChannelReporter(mpsc::Sender<WorkerMessage>);

impl Reporter for ChannelReporter {
    fn warning(&mut self, message: String) {
        let _ = self.0.send(WorkerMessage::Line(LogLine::Warning(message)));
    }
    fn info(&mut self, message: String) {
        let _ = self.0.send(WorkerMessage::Line(LogLine::Info(message)));
    }
}

pub struct App {
    pub project: Project,
    pub values: [String; FIELDS.len()],
    pub selected: usize,
    pub mode: Mode,
    pub log: Vec<LogLine>,
    pub last_pdf: Option<PathBuf>,
    pub courses: Vec<Course>,
    pub should_quit: bool,
    /// El nombre del PDF se propone a partir del Markdown mientras la persona
    /// no lo haya escrito a mano.
    file_name_is_auto: bool,
    /// Campos propios del diseño elegido, después de los fijos.
    pub extras: Vec<ExtraField>,
    /// El diseño elegido (o el de omisión), para saber qué datos pide.
    pub design: Option<Design>,
    /// Abrir el asistente en cuanto se elija el idioma (primera vez sin perfiles).
    pending_wizard: bool,
}

pub fn index_of(key: FieldKey) -> usize {
    FIELDS.iter().position(|f| f.key == key).unwrap()
}

impl App {
    pub fn new(project: Project) -> Self {
        let (courses, errors) = list_courses(&project);
        let log = errors.into_iter().map(|e| LogLine::Warning(e.0)).collect();
        let mut app = Self {
            project,
            values: Default::default(),
            selected: index_of(FieldKey::Markdown),
            mode: Mode::Form,
            log,
            last_pdf: None,
            courses,
            should_quit: false,
            file_name_is_auto: true,
            extras: Vec::new(),
            design: None,
            pending_wizard: false,
        };
        app.refresh_design();
        app
    }

    /// Primer arranque: el idioma si no estaba guardado y, si no hay perfiles,
    /// el asistente para crear el primero.
    pub fn start(&mut self, ask_language: Option<Lang>) {
        let no_profiles = self.courses.is_empty();
        match ask_language {
            Some(lang) => {
                self.ask_language(lang);
                self.pending_wizard = no_profiles;
            }
            None if no_profiles => self.open_wizard(None),
            None => {}
        }
    }

    /// Abre el asistente: vacío, o con los datos de un perfil para editarlo.
    pub fn open_wizard(&mut self, existing: Option<&str>) {
        let course = existing.and_then(|key| self.courses.iter().find(|c| c.key == key)).cloned();
        self.mode = Mode::Wizard(Box::new(Wizard::new(&self.project, course.as_ref())));
    }

    fn wizard_key(&mut self, key: KeyEvent, mut wizard: Box<Wizard>) {
        match wizard.handle_key(key, &self.project) {
            WizardOutcome::Continue => self.mode = Mode::Wizard(wizard),
            WizardOutcome::Cancel => {}
            WizardOutcome::Done => match save_profile(&self.project, &wizard.key, &wizard.finished_profile())
            {
                Ok(path) => {
                    let (courses, _) = list_courses(&self.project);
                    self.courses = courses;
                    let key = crate::document::slugify(&wizard.key);
                    self.apply_profile(&key);
                    let shown = self.display_path(&path.display().to_string());
                    self.log.push(LogLine::Info(
                        tr!(es: "Perfil guardado: {shown}", en: "Profile saved: {shown}"),
                    ));
                }
                Err(error) => self.log.push(LogLine::Error(error.0)),
            },
        }
    }

    /// Relee el diseño elegido y ajusta los campos propios, conservando lo
    /// que ya se había escrito en los que siguen existiendo.
    fn refresh_design(&mut self) {
        let chosen = Some(self.value(FieldKey::Template).to_owned()).filter(|v| !v.is_empty());
        self.design = crate::template::load_design(&self.project, chosen.as_deref()).ok();
        let previous: BTreeMap<String, String> = self.extras.drain(..).map(|e| (e.name, e.value)).collect();
        if let Some(design) = &self.design {
            self.extras = design
                .custom_fields()
                .into_iter()
                .map(|name| {
                    let spec = design.field_spec(name);
                    ExtraField {
                        name: name.to_owned(),
                        label: spec
                            .label
                            .as_ref()
                            .map(|l| l.get().to_owned())
                            .unwrap_or_else(|| name.to_owned()),
                        help: spec.help.as_ref().map(|h| h.get().to_owned()).unwrap_or_default(),
                        required: spec.required,
                        value: previous.get(name).cloned().unwrap_or_default(),
                    }
                })
                .collect();
        }
        self.selected = self.selected.min(self.total_fields() - 1);
    }

    /// Campos del formulario: los fijos y luego los propios del diseño.
    pub fn total_fields(&self) -> usize {
        FIELDS.len() + self.extras.len()
    }

    pub fn field_label(&self, index: usize) -> String {
        match FIELDS.get(index) {
            Some(field) => field.label.get().to_owned(),
            None => self.extras[index - FIELDS.len()].label.clone(),
        }
    }

    pub fn field_value(&self, index: usize) -> &str {
        match FIELDS.get(index) {
            Some(_) => self.values[index].trim(),
            None => self.extras[index - FIELDS.len()].value.trim(),
        }
    }

    /// Obligatorio: el Markdown y el título siempre; la materia si el diseño
    /// la usa; los campos propios si su ficha lo dice.
    pub fn is_required(&self, index: usize) -> bool {
        match FIELDS.get(index) {
            Some(field) if field.key == FieldKey::Course => self
                .design
                .as_ref()
                .is_some_and(|d| d.uses("MATERIA") && !d.manifest.optional.iter().any(|o| o == "MATERIA")),
            Some(field) => field.required,
            None => self.extras[index - FIELDS.len()].required,
        }
    }

    pub fn value(&self, key: FieldKey) -> &str {
        self.values[index_of(key)].trim()
    }

    fn set(&mut self, key: FieldKey, value: impl Into<String>) {
        self.values[index_of(key)] = value.into();
    }

    pub fn missing_fields(&self) -> Vec<String> {
        (0..self.total_fields())
            .filter(|&i| self.is_required(i) && self.field_value(i).is_empty())
            .map(|i| self.field_label(i))
            .collect()
    }

    /// Abre la pantalla de idioma con `preselected` marcado.
    pub fn ask_language(&mut self, preselected: Lang) {
        self.mode = Mode::ChooseLanguage(preselected);
    }

    /// Cambia el idioma de la interfaz y lo guarda en el `.env` como IDIOMA.
    fn set_language(&mut self, lang: Lang) {
        i18n::set(lang);
        if let Err(error) = Settings::save_value(&self.project.env_file(), LANG_SETTINGS[0], lang.code()) {
            self.log.push(LogLine::Warning(error.0));
        }
    }

    /// Ruta corta para mostrar: relativa a la raíz del proyecto si cae dentro.
    pub fn display_path(&self, path: &str) -> String {
        Path::new(path)
            .strip_prefix(&self.project.root)
            .map(|p| p.display().to_string())
            .unwrap_or_else(|_| path.to_owned())
    }

    fn current_course(&self) -> Option<&Course> {
        let key = self.value(FieldKey::Profile);
        self.courses.iter().find(|s| s.key == key)
    }

    /// Arma los mismos argumentos que recibiría el CLI. Los campos vacíos no
    /// se pasan, para que sigan valiendo el perfil y el `.env`.
    pub fn build_args(&self) -> Args {
        let optional = |key| Some(self.value(key).to_owned()).filter(|v| !v.is_empty());
        Args {
            markdown: PathBuf::from(self.value(FieldKey::Markdown)),
            title: self.value(FieldKey::Title).to_owned(),
            file_name: optional(FieldKey::FileName),
            course: optional(FieldKey::Course),
            profile: optional(FieldKey::Profile),
            teacher: optional(FieldKey::Teacher),
            members: optional(FieldKey::Members),
            group: optional(FieldKey::Group),
            output: optional(FieldKey::Output).map(|o| self.project.root.join(o)),
            copy: self
                .value(FieldKey::Copies)
                .split(';')
                .map(str::trim)
                .filter(|p| !p.is_empty())
                .map(PathBuf::from)
                .collect(),
            env_file: None,
            allow_latex: false,
            design: optional(FieldKey::Template),
            format: optional(FieldKey::Format),
            fields: self
                .extras
                .iter()
                .filter(|e| !e.value.trim().is_empty())
                .map(|e| (e.name.clone(), e.value.trim().to_owned()))
                .collect(),
            doc_lang: None,
            logos: None,
            lang: Some(i18n::current()),
        }
    }

    /// Procesa una tecla según el modo actual.
    pub fn handle_key(&mut self, key: KeyEvent) {
        if key.modifiers.contains(KeyModifiers::CONTROL) && key.code == KeyCode::Char('c') {
            self.should_quit = true;
            return;
        }
        match std::mem::replace(&mut self.mode, Mode::Form) {
            Mode::ChooseLanguage(selected) => self.language_key(key, selected),
            Mode::Wizard(wizard) => self.wizard_key(key, wizard),
            Mode::Form => self.form_key(key),
            Mode::Editing(input) => self.editing_key(key, input),
            Mode::Picking(picker) => self.picking_key(key, picker),
            // Mientras se genera no se acepta nada; el hilo no se puede cortar.
            generating @ Mode::Generating(_) => self.mode = generating,
        }
    }

    fn language_key(&mut self, key: KeyEvent, selected: Lang) {
        match key.code {
            KeyCode::Up | KeyCode::Down | KeyCode::Left | KeyCode::Right | KeyCode::Tab => {
                self.mode = Mode::ChooseLanguage(selected.other())
            }
            KeyCode::Enter => self.set_language(selected),
            // Esc usa el idioma marcado sin guardarlo: se volverá a preguntar.
            KeyCode::Esc => i18n::set(selected),
            _ => {
                self.mode = Mode::ChooseLanguage(selected);
                return;
            }
        }
        if std::mem::take(&mut self.pending_wizard) {
            self.open_wizard(None);
        }
    }

    fn form_key(&mut self, key: KeyEvent) {
        match key.code {
            KeyCode::Up | KeyCode::Char('k') | KeyCode::BackTab => {
                self.selected = self.selected.checked_sub(1).unwrap_or(self.total_fields() - 1)
            }
            KeyCode::Down | KeyCode::Char('j') | KeyCode::Tab => {
                self.selected = (self.selected + 1) % self.total_fields()
            }
            KeyCode::Enter | KeyCode::Right => self.activate_field(),
            KeyCode::Char(c @ '1'..='9') => {
                self.selected = (c as usize - '1' as usize).min(FIELDS.len() - 1);
                self.activate_field();
            }
            KeyCode::Delete | KeyCode::Backspace => match FIELDS.get(self.selected) {
                Some(field) => {
                    self.values[self.selected].clear();
                    if field.key == FieldKey::FileName {
                        self.file_name_is_auto = true;
                    }
                    if field.key == FieldKey::Template {
                        self.refresh_design();
                    }
                }
                None => self.extras[self.selected - FIELDS.len()].value.clear(),
            },
            // p: perfil/profile en los dos idiomas. Edita el elegido o crea uno.
            KeyCode::Char('p') => {
                let current = Some(self.value(FieldKey::Profile).to_owned()).filter(|v| !v.is_empty());
                self.open_wizard(current.as_deref());
            }
            KeyCode::Char('g') | KeyCode::F(5) => self.start_generation(),
            // Las letras de cada acción en los dos idiomas: c/f carpetas, s/q salir.
            KeyCode::Char('c' | 'f') => self.open_folders(),
            KeyCode::Char('v') => self.open_last_pdf(),
            KeyCode::Char('l') => {
                let lang = i18n::current().other();
                self.set_language(lang);
                self.log.push(LogLine::Info(tr!(es: "Idioma: español.", en: "Language: English.")));
            }
            KeyCode::Char('s' | 'q') | KeyCode::Esc => self.should_quit = true,
            _ => {}
        }
    }

    fn activate_field(&mut self) {
        let Some(field) = FIELDS.get(self.selected) else {
            let value = self.extras[self.selected - FIELDS.len()].value.clone();
            self.mode = Mode::Editing(TextInput::new(&value));
            return;
        };
        self.mode = match field.kind {
            FieldKind::Text => Mode::Editing(TextInput::new(&self.values[self.selected])),
            FieldKind::Pick(purpose) => Mode::Picking(self.picker_for(purpose)),
        };
    }

    fn picker_for(&self, purpose: PickerPurpose) -> Picker {
        match purpose {
            PickerPurpose::Markdown => {
                // Empieza en la carpeta de la materia elegida, si existe.
                let start = self
                    .current_course()
                    .and_then(|s| s.input_dir(&self.project))
                    .or_else(|| Some(self.project.input_dir()).filter(|d| d.is_dir()))
                    .unwrap_or_else(|| self.project.root.clone());
                Picker::files(
                    purpose,
                    Text::new("Elige el archivo Markdown", "Choose the Markdown file").get(),
                    &start,
                    "md",
                )
            }
            PickerPurpose::Profile => {
                let mut items = vec![Item {
                    label: Text::new("(ninguno)", "(none)").get().into(),
                    detail: Text::new("llenar los campos a mano", "fill the fields by hand").get().into(),
                    value: PickValue::Choice(String::new()),
                }];
                items.extend(self.courses.iter().map(|s| Item {
                    label: s.key.clone(),
                    detail: s.profile.name.clone(),
                    value: PickValue::Choice(s.key.clone()),
                }));
                Picker::choices(
                    purpose,
                    Text::new("Elige un perfil de materia", "Choose a course profile").get(),
                    items,
                )
            }
            PickerPurpose::Format => {
                let items = crate::template::list_formats(&self.project)
                    .into_iter()
                    .map(|key| {
                        let detail = crate::template::load_format(&self.project, &key)
                            .map(|f| f.manifest.name.get().to_owned())
                            .unwrap_or_default();
                        Item { label: key.clone(), detail, value: PickValue::Choice(key) }
                    })
                    .collect();
                Picker::choices(purpose, Text::new("Elige un formato", "Choose a format").get(), items)
            }
            PickerPurpose::Template => {
                // Solo los diseños que combinan con el formato elegido.
                let format = self.value(FieldKey::Format).to_owned();
                let items = crate::template::list_designs(&self.project)
                    .into_iter()
                    .filter_map(|name| {
                        let design = crate::template::load_design(&self.project, Some(&name)).ok()?;
                        let fits = format.is_empty() || design.is_self_contained() || design.accepts(&format);
                        let detail = design
                            .manifest
                            .description
                            .as_ref()
                            .map(|d| d.get().to_owned())
                            .unwrap_or_default();
                        fits.then(|| Item { label: name.clone(), detail, value: PickValue::Choice(name) })
                    })
                    .collect();
                Picker::choices(purpose, Text::new("Elige un diseño", "Choose a design").get(), items)
            }
            PickerPurpose::Folder => {
                let project = &self.project;
                let folders = [
                    (
                        "input",
                        project.input_dir(),
                        Text::new("tus trabajos en Markdown", "your papers in Markdown"),
                    ),
                    ("output", project.output_dir(), Text::new("los PDF generados", "generated PDFs")),
                    (
                        "courses",
                        project.courses_dir(),
                        Text::new("perfiles de materia (*.toml)", "course profiles (*.toml)"),
                    ),
                    (
                        "templates",
                        project.templates_dir(),
                        Text::new("formatos, diseños y logos", "formats, designs and logos"),
                    ),
                    (
                        "cache",
                        project.cache_dir(),
                        Text::new(
                            "imágenes descargadas, diagramas y estado de LaTeX",
                            "downloaded images, diagrams, LaTeX state",
                        ),
                    ),
                    (".", project.root.clone(), Text::new("raíz del proyecto", "project root")),
                ];
                let items = folders
                    .into_iter()
                    .map(|(name, path, detail)| Item {
                        label: name.into(),
                        detail: detail.get().into(),
                        value: PickValue::Dir(path),
                    })
                    .collect();
                Picker::choices(
                    purpose,
                    Text::new("Abrir una carpeta del proyecto", "Open a project folder").get(),
                    items,
                )
            }
        }
    }

    fn editing_key(&mut self, key: KeyEvent, mut input: TextInput) {
        match key.code {
            KeyCode::Enter => {
                let Some(field) = FIELDS.get(self.selected) else {
                    self.extras[self.selected - FIELDS.len()].value = input.text.trim().to_owned();
                    return;
                };
                if field.key == FieldKey::FileName {
                    self.file_name_is_auto = input.text.trim().is_empty();
                }
                self.values[self.selected] = input.text.trim().to_owned();
                return;
            }
            KeyCode::Esc => return,
            KeyCode::Char('u') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                input = TextInput::default()
            }
            KeyCode::Char(c) => input.insert(c),
            KeyCode::Backspace => input.backspace(),
            KeyCode::Delete => input.delete(),
            KeyCode::Left => input.cursor = input.cursor.saturating_sub(1),
            KeyCode::Right => input.cursor = (input.cursor + 1).min(input.text.chars().count()),
            KeyCode::Home => input.cursor = 0,
            KeyCode::End => input.cursor = input.text.chars().count(),
            _ => {}
        }
        self.mode = Mode::Editing(input);
    }

    fn picking_key(&mut self, key: KeyEvent, mut picker: Picker) {
        let extension = "md";
        let outcome = match key.code {
            KeyCode::Esc => PickOutcome::Cancel,
            KeyCode::Up => {
                picker.move_by(-1);
                PickOutcome::Stay
            }
            KeyCode::Down => {
                picker.move_by(1);
                PickOutcome::Stay
            }
            KeyCode::PageUp => {
                picker.move_by(-10);
                PickOutcome::Stay
            }
            KeyCode::PageDown => {
                picker.move_by(10);
                PickOutcome::Stay
            }
            KeyCode::Enter => picker.activate(extension),
            KeyCode::Right if picker.dir.is_some() => picker.activate(extension),
            KeyCode::Left if picker.dir.is_some() => {
                picker.go_up(extension);
                PickOutcome::Stay
            }
            KeyCode::Backspace => {
                if !picker.pop_filter() && picker.dir.is_some() {
                    picker.go_up(extension);
                }
                PickOutcome::Stay
            }
            KeyCode::Char(c) => {
                picker.push_filter(c);
                PickOutcome::Stay
            }
            _ => PickOutcome::Stay,
        };
        match outcome {
            PickOutcome::Stay => self.mode = Mode::Picking(picker),
            PickOutcome::Cancel => {}
            PickOutcome::Chosen(value) => self.apply_choice(picker.purpose, value),
        }
    }

    fn apply_choice(&mut self, purpose: PickerPurpose, value: PickValue) {
        match (purpose, value) {
            (PickerPurpose::Markdown, PickValue::File(path)) => {
                if self.file_name_is_auto {
                    let stem = path.file_stem().unwrap_or_default().to_string_lossy();
                    self.set(FieldKey::FileName, crate::document::slugify(&stem));
                }
                self.set(FieldKey::Markdown, path.display().to_string());
            }
            (PickerPurpose::Profile, PickValue::Choice(key)) => self.apply_profile(&key),
            (PickerPurpose::Format, PickValue::Choice(name)) => {
                // Un diseño que no combina con el formato nuevo se quita.
                if self.design.as_ref().is_some_and(|d| !d.is_self_contained() && !d.accepts(&name))
                    && !self.value(FieldKey::Template).is_empty()
                {
                    self.set(FieldKey::Template, "");
                    self.refresh_design();
                }
                self.set(FieldKey::Format, name);
            }
            (PickerPurpose::Template, PickValue::Choice(name)) => {
                self.set(FieldKey::Template, name);
                self.refresh_design();
            }
            (PickerPurpose::Folder, PickValue::Dir(path)) => self.open_path(&path),
            _ => {}
        }
    }

    /// Rellena el formulario con el perfil; lo que la persona cambie después
    /// gana, igual que una opción del CLI gana sobre el perfil.
    fn apply_profile(&mut self, key: &str) {
        self.set(FieldKey::Profile, key);
        let Some(course) = self.courses.iter().find(|s| s.key == key).cloned() else { return };
        let profile = &course.profile;
        self.set(FieldKey::Course, profile.name.trim());
        for (field, value) in [
            (FieldKey::Teacher, &profile.teacher),
            (FieldKey::Members, &profile.members),
            (FieldKey::Group, &profile.group),
            (FieldKey::Format, &profile.format),
            (FieldKey::Template, &profile.design),
        ] {
            if !value.trim().is_empty() {
                self.set(field, value.trim());
            }
        }
        // Los campos propios del diseño del perfil, con sus valores.
        self.refresh_design();
        for extra in &mut self.extras {
            if let Some(value) = profile.fields.get(&extra.name) {
                extra.value = value.clone();
            }
        }
        if !profile.folder.trim().is_empty() {
            let output = course.output_dir(&self.project);
            self.set(FieldKey::Output, self.display_path(&output.display().to_string()));
        }
    }

    fn start_generation(&mut self) {
        let missing = self.missing_fields();
        if !missing.is_empty() {
            let missing = missing.join(", ");
            self.log.push(LogLine::Error(tr!(
                es: "Todavía no se puede generar. Falta: {missing}.",
                en: "Cannot generate yet. Missing: {missing}."
            )));
            return;
        }
        self.log.clear();
        self.log.push(LogLine::Info(tr!(
            es: "Generando el PDF; puede tardar unos segundos…",
            en: "Generating the PDF; this can take a few seconds…"
        )));
        let (sender, receiver) = mpsc::channel();
        let args = self.build_args();
        let project = self.project.clone();
        // El idioma es por hilo: el que genera hereda el de la interfaz.
        let lang = i18n::current();
        std::thread::spawn(move || {
            i18n::set(lang);
            let mut reporter = ChannelReporter(sender.clone());
            let result = execute(&args, &project, &mut reporter).map_err(|e| e.0);
            let _ = sender.send(WorkerMessage::Finished(result));
        });
        self.mode = Mode::Generating(receiver);
    }

    /// Recoge los mensajes del hilo de generación; se llama en cada ciclo.
    pub fn poll_worker(&mut self) {
        let Mode::Generating(receiver) = &self.mode else { return };
        let finished = loop {
            match receiver.try_recv() {
                Ok(WorkerMessage::Line(line)) => self.log.push(line),
                Ok(WorkerMessage::Finished(result)) => break result,
                Err(TryRecvError::Empty) => return,
                Err(TryRecvError::Disconnected) => {
                    break Err(
                        tr!(es: "La generación se detuvo inesperadamente.", en: "The generation stopped unexpectedly."),
                    );
                }
            }
        };
        match finished {
            Ok(pdf) => {
                self.log.push(LogLine::Info(tr!(
                    es: "Listo. Pulsa v para ver el PDF o c para abrir una carpeta.",
                    en: "Done. Press v to view the PDF or f to open a folder."
                )));
                self.last_pdf = Some(pdf);
            }
            Err(error) => self.log.push(LogLine::Error(error)),
        }
        self.mode = Mode::Form;
    }

    fn open_folders(&mut self) {
        self.mode = Mode::Picking(self.picker_for(PickerPurpose::Folder));
    }

    fn open_last_pdf(&mut self) {
        match self.last_pdf.clone() {
            Some(pdf) => self.open_path(&pdf),
            None => self.log.push(LogLine::Warning(tr!(
                es: "Todavía no hay PDF: genera uno con g.",
                en: "There is no PDF yet: generate one with g."
            ))),
        }
    }

    /// Abre un archivo o carpeta con el programa del sistema (explorador,
    /// visor de PDF). La carpeta se crea si aún no existe.
    fn open_path(&mut self, path: &Path) {
        if !path.exists() && path.extension().is_none() {
            let _ = std::fs::create_dir_all(path);
        }
        match opener::open(path) {
            Ok(()) => {
                let shown = self.display_path(&path.display().to_string());
                self.log.push(LogLine::Info(tr!(es: "Abierto: {shown}", en: "Opened: {shown}")))
            }
            Err(error) => self.log.push(LogLine::Error(tr!(
                es: "No se pudo abrir {}: {error}",
                en: "Could not open {}: {error}",
                path.display()
            ))),
        }
    }
}
