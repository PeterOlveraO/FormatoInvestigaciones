//! Estado del menú interactivo y su respuesta a cada tecla. No dibuja nada:
//! eso lo hace `ui.rs`, así que esta parte se puede probar sin terminal.

use std::path::{Path, PathBuf};
use std::sync::mpsc::{self, Receiver, TryRecvError};

use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

use super::picker::{Item, PickOutcome, PickValue, Picker, PickerPurpose};
use crate::cli::{Args, Reporter, execute};
use crate::project::Project;
use crate::subjects::{Subject, list_subjects};

/// Campos del formulario, en el orden en que se muestran.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FieldKey {
    Profile,
    Markdown,
    Title,
    FileName,
    Subject,
    Teacher,
    Members,
    Group,
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
    pub label: &'static str,
    pub kind: FieldKind,
    pub required: bool,
    pub help: &'static str,
    pub example: &'static str,
    /// Qué pasa si se deja vacío (solo campos opcionales).
    pub empty: &'static str,
}

/// El formulario. Ya no incluye el archivo .env, «permitir LaTeX», la
/// plantilla por ruta ni la carpeta de logos (las opciones 9 a 12 del menú
/// anterior): siguen disponibles en el CLI para quien las necesite.
pub const FIELDS: [Field; 11] = [
    Field {
        key: FieldKey::Profile,
        label: "Subject profile",
        kind: FieldKind::Pick(PickerPurpose::Profile),
        required: false,
        help: "Saved data of a subject (subjects/*.toml). Choosing one fills subject, teacher, group, template and output folder.",
        example: "ia",
        empty: "fill the fields by hand",
    },
    Field {
        key: FieldKey::Markdown,
        label: "Markdown file",
        kind: FieldKind::Pick(PickerPurpose::Markdown),
        required: true,
        help: "The .md file with the content. Browse the folders with the arrows; type to filter.",
        example: "input/IA/Tarea1.md",
        empty: "",
    },
    Field {
        key: FieldKey::Title,
        label: "Title",
        kind: FieldKind::Text,
        required: true,
        help: "Title shown on the cover. It does not change the file name.",
        example: "Introduction to databases",
        empty: "",
    },
    Field {
        key: FieldKey::FileName,
        label: "PDF file name",
        kind: FieldKind::Text,
        required: false,
        help: "Name of the PDF, independent of the title. Accents and spaces are simplified.",
        example: "tarea1-ia",
        empty: "the name of the Markdown file",
    },
    Field {
        key: FieldKey::Subject,
        label: "Subject",
        kind: FieldKind::Text,
        required: true,
        help: "Name of the subject, as shown on the cover.",
        example: "Inteligencia artificial",
        empty: "",
    },
    Field {
        key: FieldKey::Teacher,
        label: "Teacher",
        kind: FieldKind::Text,
        required: false,
        help: "Name of the teacher.",
        example: "Name of the teacher",
        empty: "DOCENTE from .env; if missing, omitted",
    },
    Field {
        key: FieldKey::Members,
        label: "Team members",
        kind: FieldKind::Text,
        required: false,
        help: "Team names separated by commas. With members, the cover does not show the student.",
        example: "Ana Ruiz, Luis Paz",
        empty: "INTEGRANTES from .env; if missing, the student",
    },
    Field {
        key: FieldKey::Group,
        label: "Group",
        kind: FieldKind::Text,
        required: false,
        help: "Group of the subject.",
        example: "7-A",
        empty: "GRUPO from .env; if missing, omitted",
    },
    Field {
        key: FieldKey::Template,
        label: "Template",
        kind: FieldKind::Pick(PickerPurpose::Template),
        required: false,
        help: "Cover design. Templates live in templates/<name>/template.ltx.",
        example: "apa-simple",
        empty: "apa",
    },
    Field {
        key: FieldKey::Output,
        label: "Output folder",
        kind: FieldKind::Text,
        required: false,
        help: "Folder where the PDF is saved.",
        example: "output/IA",
        empty: "output/ (or output/<folder> of the profile)",
    },
    Field {
        key: FieldKey::Copies,
        label: "Extra copies",
        kind: FieldKind::Text,
        required: false,
        help: "Extra folders for a copy of the PDF, separated by ;",
        example: "~/Drive/IA; /media/usb",
        empty: "no copies",
    },
];

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
    fn insert(&mut self, c: char) {
        let at = self.byte_at(self.cursor);
        self.text.insert(at, c);
        self.cursor += 1;
    }
    fn backspace(&mut self) {
        if self.cursor > 0 {
            self.cursor -= 1;
            let at = self.byte_at(self.cursor);
            self.text.remove(at);
        }
    }
    fn delete(&mut self) {
        if self.cursor < self.text.chars().count() {
            let at = self.byte_at(self.cursor);
            self.text.remove(at);
        }
    }
}

pub enum Mode {
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
    pub subjects: Vec<Subject>,
    pub should_quit: bool,
    /// El nombre del PDF se propone a partir del Markdown mientras la persona
    /// no lo haya escrito a mano.
    file_name_is_auto: bool,
}

pub fn index_of(key: FieldKey) -> usize {
    FIELDS.iter().position(|f| f.key == key).unwrap()
}

impl App {
    pub fn new(project: Project) -> Self {
        let (subjects, errors) = list_subjects(&project);
        let log = errors.into_iter().map(|e| LogLine::Warning(e.0)).collect();
        Self {
            project,
            values: Default::default(),
            selected: index_of(FieldKey::Markdown),
            mode: Mode::Form,
            log,
            last_pdf: None,
            subjects,
            should_quit: false,
            file_name_is_auto: true,
        }
    }

    pub fn value(&self, key: FieldKey) -> &str {
        self.values[index_of(key)].trim()
    }

    fn set(&mut self, key: FieldKey, value: impl Into<String>) {
        self.values[index_of(key)] = value.into();
    }

    pub fn missing_fields(&self) -> Vec<&'static str> {
        FIELDS.iter().filter(|f| f.required && self.value(f.key).is_empty()).map(|f| f.label).collect()
    }

    /// Ruta corta para mostrar: relativa a la raíz del proyecto si cae dentro.
    pub fn display_path(&self, path: &str) -> String {
        Path::new(path)
            .strip_prefix(&self.project.root)
            .map(|p| p.display().to_string())
            .unwrap_or_else(|_| path.to_owned())
    }

    fn current_subject(&self) -> Option<&Subject> {
        let key = self.value(FieldKey::Profile);
        self.subjects.iter().find(|s| s.key == key)
    }

    /// Arma los mismos argumentos que recibiría el CLI. Los campos vacíos no
    /// se pasan, para que sigan valiendo el perfil y el `.env`.
    pub fn build_args(&self) -> Args {
        let optional = |key| Some(self.value(key).to_owned()).filter(|v| !v.is_empty());
        Args {
            markdown: PathBuf::from(self.value(FieldKey::Markdown)),
            title: self.value(FieldKey::Title).to_owned(),
            file_name: optional(FieldKey::FileName),
            subject: optional(FieldKey::Subject),
            subject_profile: optional(FieldKey::Profile),
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
            template: optional(FieldKey::Template),
            logos: None,
        }
    }

    /// Procesa una tecla según el modo actual.
    pub fn handle_key(&mut self, key: KeyEvent) {
        if key.modifiers.contains(KeyModifiers::CONTROL) && key.code == KeyCode::Char('c') {
            self.should_quit = true;
            return;
        }
        match std::mem::replace(&mut self.mode, Mode::Form) {
            Mode::Form => self.form_key(key),
            Mode::Editing(input) => self.editing_key(key, input),
            Mode::Picking(picker) => self.picking_key(key, picker),
            // Mientras se genera no se acepta nada; el hilo no se puede cortar.
            generating @ Mode::Generating(_) => self.mode = generating,
        }
    }

    fn form_key(&mut self, key: KeyEvent) {
        match key.code {
            KeyCode::Up | KeyCode::Char('k') | KeyCode::BackTab => {
                self.selected = self.selected.checked_sub(1).unwrap_or(FIELDS.len() - 1)
            }
            KeyCode::Down | KeyCode::Char('j') | KeyCode::Tab => {
                self.selected = (self.selected + 1) % FIELDS.len()
            }
            KeyCode::Enter | KeyCode::Right => self.activate_field(),
            KeyCode::Char(c @ '1'..='9') => {
                self.selected = (c as usize - '1' as usize).min(FIELDS.len() - 1);
                self.activate_field();
            }
            KeyCode::Delete | KeyCode::Backspace => {
                let field = FIELDS[self.selected];
                self.values[self.selected].clear();
                if field.key == FieldKey::FileName {
                    self.file_name_is_auto = true;
                }
            }
            KeyCode::Char('g') | KeyCode::F(5) => self.start_generation(),
            KeyCode::Char('f') => self.open_folders(),
            KeyCode::Char('o') => self.open_last_pdf(),
            KeyCode::Char('q') | KeyCode::Esc => self.should_quit = true,
            _ => {}
        }
    }

    fn activate_field(&mut self) {
        let field = FIELDS[self.selected];
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
                    .current_subject()
                    .and_then(|s| s.input_dir(&self.project))
                    .or_else(|| Some(self.project.input_dir()).filter(|d| d.is_dir()))
                    .unwrap_or_else(|| self.project.root.clone());
                Picker::files(purpose, "Choose the Markdown file", &start, "md")
            }
            PickerPurpose::Profile => {
                let mut items = vec![Item {
                    label: "(none)".into(),
                    detail: "fill the fields by hand".into(),
                    value: PickValue::Choice(String::new()),
                }];
                items.extend(self.subjects.iter().map(|s| Item {
                    label: s.key.clone(),
                    detail: s.profile.subject.clone(),
                    value: PickValue::Choice(s.key.clone()),
                }));
                Picker::choices(purpose, "Choose a subject profile", items)
            }
            PickerPurpose::Template => {
                let items = self
                    .project
                    .list_templates()
                    .into_iter()
                    .map(|name| Item {
                        label: name.clone(),
                        detail: String::new(),
                        value: PickValue::Choice(name),
                    })
                    .collect();
                Picker::choices(purpose, "Choose a template", items)
            }
            PickerPurpose::Folder => {
                let project = &self.project;
                let folders = [
                    ("input", project.input_dir(), "your papers in Markdown"),
                    ("output", project.output_dir(), "generated PDFs"),
                    ("subjects", project.subjects_dir(), "subject profiles (*.toml)"),
                    ("templates", project.templates_dir(), "LaTeX templates and logos"),
                    ("cache", project.cache_dir(), "downloaded images, diagrams, LaTeX state"),
                    ("project", project.root.clone(), "project root"),
                ];
                let items = folders
                    .into_iter()
                    .map(|(name, path, detail)| Item {
                        label: name.into(),
                        detail: detail.into(),
                        value: PickValue::Dir(path),
                    })
                    .collect();
                Picker::choices(purpose, "Open a project folder", items)
            }
        }
    }

    fn editing_key(&mut self, key: KeyEvent, mut input: TextInput) {
        match key.code {
            KeyCode::Enter => {
                let field = FIELDS[self.selected];
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
            (PickerPurpose::Template, PickValue::Choice(name)) => self.set(FieldKey::Template, name),
            (PickerPurpose::Folder, PickValue::Dir(path)) => self.open_path(&path),
            _ => {}
        }
    }

    /// Rellena el formulario con el perfil; lo que la persona cambie después
    /// gana, igual que una opción del CLI gana sobre el perfil.
    fn apply_profile(&mut self, key: &str) {
        self.set(FieldKey::Profile, key);
        let Some(subject) = self.subjects.iter().find(|s| s.key == key).cloned() else { return };
        let profile = &subject.profile;
        self.set(FieldKey::Subject, profile.subject.trim());
        for (field, value) in [
            (FieldKey::Teacher, &profile.teacher),
            (FieldKey::Members, &profile.members),
            (FieldKey::Group, &profile.group),
            (FieldKey::Template, &profile.template),
        ] {
            if !value.trim().is_empty() {
                self.set(field, value.trim());
            }
        }
        if !profile.folder.trim().is_empty() {
            let output = subject.output_dir(&self.project);
            self.set(FieldKey::Output, self.display_path(&output.display().to_string()));
        }
    }

    fn start_generation(&mut self) {
        let missing = self.missing_fields();
        if !missing.is_empty() {
            self.log.push(LogLine::Error(format!("Cannot generate yet. Missing: {}.", missing.join(", "))));
            return;
        }
        self.log.clear();
        self.log.push(LogLine::Info("Generating the PDF, this can take a few seconds...".into()));
        let (sender, receiver) = mpsc::channel();
        let args = self.build_args();
        let project = self.project.clone();
        std::thread::spawn(move || {
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
                Err(TryRecvError::Disconnected) => break Err("The generation stopped unexpectedly.".into()),
            }
        };
        match finished {
            Ok(pdf) => {
                self.log.push(LogLine::Info("Done. Press o to open the PDF or f to open a folder.".into()));
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
            None => self.log.push(LogLine::Warning("There is no PDF yet: generate one with g.".into())),
        }
    }

    /// Abre un archivo o carpeta con el programa del sistema (explorador,
    /// visor de PDF). La carpeta se crea si aún no existe.
    fn open_path(&mut self, path: &Path) {
        if !path.exists() && path.extension().is_none() {
            let _ = std::fs::create_dir_all(path);
        }
        match opener::open(path) {
            Ok(()) => self
                .log
                .push(LogLine::Info(format!("Opened {}", self.display_path(&path.display().to_string())))),
            Err(error) => {
                self.log.push(LogLine::Error(format!("Could not open {}: {error}", path.display())))
            }
        }
    }
}
