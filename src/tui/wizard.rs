//! Asistente de perfiles: pide paso a paso el nombre, el diseño y
//! todos los datos que ese diseño usa, y guarda `courses/<clave>.toml`. Cada
//! perfil lleva sus propios datos, así que el asistente los pide completos.

use ratatui::crossterm::event::{KeyCode, KeyEvent};

use super::app::{TextInput, existing_dir, short_path};
use super::picker::{PickOutcome, PickValue, Picker, PickerPurpose, design_items};
use crate::courses::{Course, CourseProfile};
use crate::i18n::Text;
use crate::project::Project;
use crate::template::load_design;

/// Qué dato llena cada paso.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Target {
    Key,
    Design,
    /// Un dato estándar de la portada, por su marcador (`UNIVERSIDAD`…).
    Standard(&'static str),
    /// Un campo propio del diseño (`SALON`).
    Custom(String),
    Folder,
    /// Carpetas de las copias, elegidas con el selector de carpetas.
    Copies,
}

#[derive(Debug, Clone)]
pub struct Step {
    pub target: Target,
    pub title: String,
    pub help: String,
    pub required: bool,
}

/// Resultado de una tecla.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Outcome {
    Continue,
    Cancel,
    Done,
}

// Datos estándar, en el orden en que se piden: (marcador, etiqueta, ayuda).
const STANDARD_STEPS: [(&str, Text, Text); 8] = [
    (
        "UNIVERSIDAD",
        Text::new("Universidad", "University"),
        Text::new("Nombre de la institución.", "Name of the institution."),
    ),
    ("FACULTAD", Text::new("Facultad", "Faculty"), Text::new("Facultad o escuela.", "Faculty or school.")),
    (
        "ALUMNO",
        Text::new("Alumno", "Student"),
        Text::new("Tu nombre, para trabajos individuales.", "Your name, for individual papers."),
    ),
    (
        "INTEGRANTES",
        Text::new("Integrantes", "Team members"),
        Text::new(
            "Nombres del equipo separados por comas; si hay, sustituyen al alumno.",
            "Team names separated by commas; when given, they replace the student.",
        ),
    ),
    ("MATERIA", Text::new("Materia", "Course"), Text::new("Nombre de la materia.", "Name of the course.")),
    ("DOCENTE", Text::new("Docente", "Teacher"), Text::new("Nombre del docente.", "Name of the teacher.")),
    ("GRUPO", Text::new("Grupo", "Group"), Text::new("Grupo de la materia.", "Group of the course.")),
    (
        "SEMESTRE",
        Text::new("Semestre", "Semester"),
        Text::new("Semestre o periodo, p. ej. 2026-2.", "Term, e.g. 2026-2."),
    ),
];

/// El campo del perfil que corresponde a cada marcador estándar.
pub fn standard_field<'a>(profile: &'a mut CourseProfile, marker: &str) -> Option<&'a mut String> {
    Some(match marker {
        "UNIVERSIDAD" => &mut profile.university,
        "FACULTAD" => &mut profile.faculty,
        "ALUMNO" => &mut profile.student,
        "INTEGRANTES" => &mut profile.members,
        "MATERIA" => &mut profile.name,
        "DOCENTE" => &mut profile.teacher,
        "GRUPO" => &mut profile.group,
        "SEMESTRE" => &mut profile.semester,
        _ => return None,
    })
}

pub struct Wizard {
    pub steps: Vec<Step>,
    pub index: usize,
    pub key: String,
    pub profile: CourseProfile,
    pub input: TextInput,
    pub picker: Option<Picker>,
    /// La clave del perfil que se edita (`None` al crear uno).
    pub original: Option<String>,
    pub error: Option<String>,
    /// Ya se pulsó Esc una vez: otro Esc sale sin guardar.
    confirm_exit: bool,
}

impl Wizard {
    pub fn new(project: &Project, existing: Option<&Course>) -> Self {
        let steps = vec![
            Step {
                target: Target::Key,
                title: Text::new("Nombre del perfil", "Profile name").get().into(),
                help: Text::new(
                    "Un nombre corto, como «ia»; se guarda en courses/<nombre>.toml.",
                    "A short name, like \"ia\"; it is saved as courses/<name>.toml.",
                )
                .get()
                .into(),
                required: true,
            },
            Step {
                target: Target::Design,
                title: Text::new("Diseño", "Design").get().into(),
                help: Text::new(
                    "La portada y el aspecto del documento.",
                    "The cover and look of the document.",
                )
                .get()
                .into(),
                required: true,
            },
        ];
        let mut wizard = Self {
            steps,
            index: 0,
            key: existing.map(|c| c.key.clone()).unwrap_or_default(),
            profile: existing.map(|c| c.profile.clone()).unwrap_or_default(),
            input: TextInput::default(),
            picker: None,
            original: existing.map(|c| c.key.clone()),
            error: None,
            confirm_exit: false,
        };
        // Al editar se conserva el diseño, así que ya se conocen sus datos.
        if !wizard.profile.design.trim().is_empty() {
            wizard.add_data_steps(project);
        }
        wizard.enter_step(project);
        wizard
    }

    pub fn editing(&self) -> bool {
        self.original.is_some()
    }

    pub fn step(&self) -> &Step {
        &self.steps[self.index]
    }

    /// Valor actual del dato del paso, para mostrarlo al entrar.
    fn current_value(&mut self) -> String {
        match self.step().target.clone() {
            Target::Key => self.key.clone(),
            Target::Design => self.profile.design.clone(),
            Target::Standard(marker) => {
                standard_field(&mut self.profile, marker).cloned().unwrap_or_default()
            }
            Target::Custom(name) => self.profile.fields.get(&name).cloned().unwrap_or_default(),
            Target::Folder => self.profile.folder.clone(),
            Target::Copies => self.profile.copies.join("; "),
        }
    }

    fn enter_step(&mut self, project: &Project) {
        self.error = None;
        let current = self.current_value();
        self.picker = match self.step().target {
            Target::Design => Some(Picker::choices(
                PickerPurpose::Template,
                self.step().title.clone(),
                design_items(project),
            )),
            // La carpeta de trabajos se elige dentro de input/, donde ya está o desde su raíz
            // (con cualquier selector de carpetas: la elección la atiende el asistente).
            Target::Folder => {
                let input = project.input_dir();
                let start = Some(input.join(&current)).filter(|d| !current.is_empty() && d.is_dir());
                Some(Picker::files(PickerPurpose::Output, self.step().title.clone(), &start.unwrap_or(input)))
            }
            Target::Copies => {
                // Empieza en la última copia guardada o en la raíz.
                let last = current.rsplit(';').next().unwrap_or_default().trim();
                let start = existing_dir(&project.resolve(last)).unwrap_or_else(|| project.work.clone());
                Some(Picker::files(PickerPurpose::Copies, self.step().title.clone(), &start))
            }
            _ => None,
        };
        // En una lista se deja marcado el valor que ya tenía el perfil.
        if let Some(picker) = &mut self.picker {
            if let Some(position) =
                picker.visible().iter().position(|i| i.value == PickValue::Choice(current.clone()))
            {
                picker.selected = position;
            }
        } else {
            self.input = TextInput::new(&current);
        }
    }

    /// Pasos de los datos que usa el diseño elegido, más la carpeta.
    fn add_data_steps(&mut self, project: &Project) {
        self.steps.truncate(2);
        let Ok(design) = load_design(project, Some(&self.profile.design)) else { return };
        for (marker, label, help) in STANDARD_STEPS {
            if design.uses(marker) {
                self.steps.push(Step {
                    target: Target::Standard(marker),
                    title: label.get().into(),
                    help: help.get().into(),
                    required: design.requires(marker),
                });
            }
        }
        for name in design.custom_fields() {
            let spec = design.field_spec(name);
            self.steps.push(Step {
                target: Target::Custom(name.to_owned()),
                title: spec.label.as_ref().map(|l| l.get().to_owned()).unwrap_or_else(|| name.to_owned()),
                help: spec.help.as_ref().map(|h| h.get().to_owned()).unwrap_or_default(),
                required: spec.required,
            });
        }
        self.steps.push(Step {
            target: Target::Folder,
            title: Text::new("Carpeta de trabajos", "Papers folder").get().into(),
            help: Text::new(
                "Subcarpeta de input/ con los trabajos de esta materia (y de output/ para los PDF). Entra en ella y elige ./, o escribe un nombre nuevo para crearla; Tab sigue sin cambiarla.",
                "Subfolder of input/ with this course's papers (and of output/ for the PDFs). Go into it and choose ./, or type a new name to create it; Tab moves on without changing it.",
            )
            .get()
            .into(),
            required: false,
        });
        self.steps.push(Step {
            target: Target::Copies,
            title: Text::new("Copias extra", "Extra copies").get().into(),
            help: Text::new(
                "Carpetas para una copia del PDF. Cada ./ elegido se suma; Supr vacía la lista y Tab termina.",
                "Folders for a copy of the PDF. Each ./ you choose is added; Del clears the list and Tab finishes.",
            )
            .get()
            .into(),
            required: false,
        });
    }

    /// Paso que se llena con el selector de carpetas.
    pub fn is_folder_step(&self) -> bool {
        matches!(self.step().target, Target::Folder | Target::Copies)
    }

    /// Guarda la respuesta del paso actual y avanza.
    fn commit(&mut self, value: String, project: &Project) -> Outcome {
        let value = value.trim().to_owned();
        if self.step().required && value.is_empty() {
            self.error = Some(Text::new("Este dato es obligatorio.", "This field is required.").get().into());
            return Outcome::Continue;
        }
        match self.step().target.clone() {
            Target::Key => {
                let key = crate::document::slugify(&value);
                // Crear (o renombrar) sobre un perfil que ya existe lo borraría.
                let taken = project.courses_dir().join(format!("{key}.toml")).exists();
                if taken && self.original.as_deref() != Some(key.as_str()) {
                    self.error = Some(tr!(
                        es: "Ya existe el perfil «{key}»; elige otro nombre.",
                        en: "The profile \"{key}\" already exists; choose another name."
                    ));
                    return Outcome::Continue;
                }
                self.key = key;
            }
            Target::Design => {
                self.profile.design = value;
                self.add_data_steps(project);
            }
            Target::Standard(marker) => {
                if let Some(field) = standard_field(&mut self.profile, marker) {
                    *field = value;
                }
            }
            Target::Custom(name) => {
                if value.is_empty() {
                    self.profile.fields.remove(&name);
                } else {
                    self.profile.fields.insert(name, value);
                }
            }
            Target::Folder => self.profile.folder = value,
            Target::Copies => {
                self.profile.copies =
                    value.split(';').map(str::trim).filter(|c| !c.is_empty()).map(str::to_owned).collect()
            }
        }
        self.index += 1;
        if self.index >= self.steps.len() {
            return Outcome::Done;
        }
        self.enter_step(project);
        Outcome::Continue
    }

    pub fn handle_key(&mut self, key: KeyEvent, project: &Project) -> Outcome {
        // El aviso del primer Esc dura una tecla.
        if std::mem::take(&mut self.confirm_exit) {
            if key.code == KeyCode::Esc {
                return Outcome::Cancel;
            }
            self.error = None;
        }
        match key.code {
            // Desde el segundo paso hay respuestas que se perderían: se pide otro Esc.
            KeyCode::Esc if self.index > 0 => {
                self.confirm_exit = true;
                self.error = Some(
                    Text::new(
                        "Pulsa Esc otra vez para salir sin guardar.",
                        "Press Esc again to leave without saving.",
                    )
                    .get()
                    .into(),
                );
                return Outcome::Continue;
            }
            KeyCode::Esc => return Outcome::Cancel,
            // Shift+Tab vuelve al paso anterior.
            KeyCode::BackTab => {
                if self.index > 0 {
                    self.index -= 1;
                    self.enter_step(project);
                }
                return Outcome::Continue;
            }
            // En los pasos de carpetas, Tab sigue con lo elegido y Supr lo vacía.
            KeyCode::Tab if self.is_folder_step() => {
                let value = self.current_value();
                return self.commit(value, project);
            }
            KeyCode::Delete if self.is_folder_step() => {
                match self.step().target {
                    Target::Folder => self.profile.folder.clear(),
                    _ => self.profile.copies.clear(),
                }
                return Outcome::Continue;
            }
            _ => {}
        }
        if let Some(picker) = &mut self.picker {
            return match picker.handle_key(key.code) {
                PickOutcome::Chosen(PickValue::Choice(value)) => self.commit(value, project),
                // La carpeta de trabajos se guarda relativa a input/ («./» en input/ = ninguna).
                PickOutcome::Chosen(PickValue::Here(dir)) if self.step().target == Target::Folder => {
                    match dir.strip_prefix(project.input_dir()) {
                        Ok(relative) => self.commit(relative.display().to_string(), project),
                        Err(_) => {
                            self.error = Some(
                                Text::new(
                                    "Elige una carpeta dentro de input/.",
                                    "Choose a folder inside input/.",
                                )
                                .get()
                                .into(),
                            );
                            Outcome::Continue
                        }
                    }
                }
                // Las copias se suman y el paso sigue abierto.
                PickOutcome::Chosen(PickValue::Here(dir)) => {
                    let path = short_path(&project.work, &dir);
                    if !self.profile.copies.contains(&path) {
                        self.profile.copies.push(path);
                    }
                    Outcome::Continue
                }
                _ => Outcome::Continue,
            };
        }
        match key.code {
            KeyCode::Enter => return self.commit(self.input.text.clone(), project),
            KeyCode::Char(c) => self.input.insert(c),
            KeyCode::Backspace => self.input.backspace(),
            KeyCode::Delete => self.input.delete(),
            KeyCode::Left => self.input.cursor = self.input.cursor.saturating_sub(1),
            KeyCode::Right => {
                self.input.cursor = (self.input.cursor + 1).min(self.input.text.chars().count())
            }
            KeyCode::Home => self.input.cursor = 0,
            KeyCode::End => self.input.cursor = self.input.text.chars().count(),
            _ => {}
        }
        Outcome::Continue
    }

    /// Guardar falló: se vuelve al último paso con el error, sin perder lo escrito.
    pub fn show_save_error(&mut self, project: &Project, error: String) {
        self.index = self.steps.len() - 1;
        self.enter_step(project);
        self.error = Some(error);
    }

    /// Perfil listo para guardar: si el diseño no usa la materia, el nombre
    /// del perfil la sustituye (un perfil siempre necesita `name`).
    pub fn finished_profile(&self) -> CourseProfile {
        let mut profile = self.profile.clone();
        if profile.name.trim().is_empty() {
            profile.name = self.key.clone();
        }
        profile
    }
}
