//! Asistente de perfiles: pide paso a paso el nombre, el formato, el diseño y
//! todos los datos que ese diseño usa, y guarda `courses/<clave>.toml`. Cada
//! perfil lleva sus propios datos, así que el asistente los pide completos.

use crossterm::event::{KeyCode, KeyEvent};

use super::app::TextInput;
use super::picker::{Item, PickOutcome, PickValue, Picker, PickerPurpose};
use crate::courses::{Course, CourseProfile};
use crate::i18n::Text;
use crate::project::Project;
use crate::template::{list_designs, list_formats, load_design, load_format};

/// Qué dato llena cada paso.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Target {
    Key,
    Format,
    Design,
    /// Un dato estándar de la portada, por su marcador (`UNIVERSIDAD`…).
    Standard(&'static str),
    /// Un campo propio del diseño (`SALON`).
    Custom(String),
    Folder,
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

/// Los datos que exige el programa si el diseño los usa (y no los marca opcionales).
const REQUIRED_STANDARD: [&str; 4] = ["UNIVERSIDAD", "FACULTAD", "SEMESTRE", "MATERIA"];

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
    /// `true` si se edita un perfil que ya existía.
    pub editing: bool,
    pub error: Option<String>,
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
                target: Target::Format,
                title: Text::new("Formato", "Format").get().into(),
                help: Text::new(
                    "La norma: letra, interlineado, encabezados, referencias.",
                    "The norm: font, spacing, headings, references.",
                )
                .get()
                .into(),
                required: true,
            },
            Step {
                target: Target::Design,
                title: Text::new("Diseño", "Design").get().into(),
                help: Text::new(
                    "La portada y el aspecto; solo se muestran los que sirven para el formato.",
                    "The cover and look; only the ones that work with the format are shown.",
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
            editing: existing.is_some(),
            error: None,
        };
        // Al editar se conservan formato y diseño, así que ya se conocen sus datos.
        if !wizard.profile.design.trim().is_empty() {
            wizard.add_data_steps(project);
        }
        wizard.enter_step(project);
        wizard
    }

    pub fn step(&self) -> &Step {
        &self.steps[self.index]
    }

    /// Valor actual del dato del paso, para mostrarlo al entrar.
    fn current_value(&mut self) -> String {
        match self.step().target.clone() {
            Target::Key => self.key.clone(),
            Target::Format => self.profile.format.clone(),
            Target::Design => self.profile.design.clone(),
            Target::Standard(marker) => {
                standard_field(&mut self.profile, marker).cloned().unwrap_or_default()
            }
            Target::Custom(name) => self.profile.fields.get(&name).cloned().unwrap_or_default(),
            Target::Folder => self.profile.folder.clone(),
        }
    }

    fn enter_step(&mut self, project: &Project) {
        self.error = None;
        let current = self.current_value();
        self.picker = match self.step().target {
            Target::Format => {
                let items = list_formats(project)
                    .into_iter()
                    .map(|key| {
                        let detail = load_format(project, &key)
                            .map(|f| f.manifest.name.get().to_owned())
                            .unwrap_or_default();
                        Item { label: key.clone(), detail, value: PickValue::Choice(key) }
                    })
                    .collect();
                Some(Picker::choices(PickerPurpose::Format, self.step().title.clone(), items))
            }
            Target::Design => {
                let format = self.profile.format.clone();
                let items = list_designs(project)
                    .into_iter()
                    .filter_map(|key| {
                        let design = load_design(project, Some(&key)).ok()?;
                        let fits = design.is_self_contained() || format.is_empty() || design.accepts(&format);
                        let detail = design
                            .manifest
                            .description
                            .as_ref()
                            .map(|d| d.get().to_owned())
                            .unwrap_or_default();
                        fits.then(|| Item { label: key.clone(), detail, value: PickValue::Choice(key) })
                    })
                    .collect();
                Some(Picker::choices(PickerPurpose::Template, self.step().title.clone(), items))
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
        self.steps.truncate(3);
        let Ok(design) = load_design(project, Some(&self.profile.design)) else { return };
        for (marker, label, help) in STANDARD_STEPS {
            if design.uses(marker) {
                let required = REQUIRED_STANDARD.contains(&marker)
                    && !design.manifest.optional.iter().any(|o| o == marker);
                self.steps.push(Step {
                    target: Target::Standard(marker),
                    title: label.get().into(),
                    help: help.get().into(),
                    required,
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
                "Subcarpeta de input/ con los trabajos de esta materia (y de output/ para los PDF). Puede quedar vacía.",
                "Subfolder of input/ with this course's papers (and of output/ for the PDFs). May be empty.",
            )
            .get()
            .into(),
            required: false,
        });
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
                if taken && (!self.editing || key != self.key) {
                    self.error = Some(tr!(
                        es: "Ya existe el perfil «{key}»; elige otro nombre.",
                        en: "The profile \"{key}\" already exists; choose another name."
                    ));
                    return Outcome::Continue;
                }
                self.key = key;
            }
            Target::Format => {
                // Si el diseño elegido no combina con el formato nuevo, se vuelve a pedir.
                let fits = load_design(project, Some(&self.profile.design))
                    .map(|d| d.accepts(&value))
                    .unwrap_or(false);
                if !fits {
                    self.profile.design.clear();
                    self.steps.truncate(3);
                }
                self.profile.format = value;
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
        }
        self.index += 1;
        if self.index >= self.steps.len() {
            return Outcome::Done;
        }
        self.enter_step(project);
        Outcome::Continue
    }

    pub fn handle_key(&mut self, key: KeyEvent, project: &Project) -> Outcome {
        match key.code {
            KeyCode::Esc => return Outcome::Cancel,
            // Shift+Tab vuelve al paso anterior.
            KeyCode::BackTab => {
                if self.index > 0 {
                    self.index -= 1;
                    self.enter_step(project);
                }
                return Outcome::Continue;
            }
            _ => {}
        }
        if let Some(picker) = &mut self.picker {
            match key.code {
                KeyCode::Up => picker.move_by(-1),
                KeyCode::Down => picker.move_by(1),
                KeyCode::Backspace => {
                    picker.pop_filter();
                }
                KeyCode::Char(c) => picker.push_filter(c),
                KeyCode::Enter => {
                    if let PickOutcome::Chosen(PickValue::Choice(value)) = picker.activate("") {
                        return self.commit(value, project);
                    }
                }
                _ => {}
            }
            return Outcome::Continue;
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
