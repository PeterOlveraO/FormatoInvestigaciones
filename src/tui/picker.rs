//! Selector de listas: el mismo componente recorre carpetas para elegir un
//! Markdown y muestra los perfiles, las plantillas y las carpetas del proyecto.
//! Escribir filtra la lista; así no hace falta teclear rutas.

use std::path::{Component, Path, PathBuf};

use ratatui::crossterm::event::KeyCode;

use crate::logging;
use crate::project::Project;
use crate::template::{list_designs, load_design};

/// Lo que devuelve una entrada al elegirla.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PickValue {
    /// Subir a la carpeta padre.
    Up(PathBuf),
    /// Entrar en una carpeta.
    Dir(PathBuf),
    /// Archivo elegido.
    File(PathBuf),
    /// La carpeta que se está viendo, en un selector de carpetas.
    Here(PathBuf),
    /// Una carpeta que todavía no existe, con el nombre escrito en el filtro.
    New(PathBuf),
    /// Opción de una lista fija (clave de perfil, nombre de plantilla...).
    Choice(String),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Item {
    pub label: String,
    /// Texto secundario (por ejemplo, la materia de un perfil).
    pub detail: String,
    pub value: PickValue,
}

/// Para qué se abrió el selector; lo usa la app al recibir la elección.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PickerPurpose {
    Markdown,
    Profile,
    Template,
    /// Abrir una carpeta del proyecto con el explorador del sistema.
    Folder,
    /// Elegir la carpeta de salida o una de las copias extra.
    Output,
    Copies,
    Theme,
    /// La carpeta de trabajo (primera vez u Opciones).
    Workspace,
}

#[derive(Debug, Clone)]
pub struct Picker {
    pub purpose: PickerPurpose,
    pub title: String,
    /// Carpeta actual cuando se navega por archivos.
    pub dir: Option<PathBuf>,
    pub items: Vec<Item>,
    pub filter: String,
    pub selected: usize,
    /// «+ nombre/» para crear la carpeta escrita; va al final de la lista.
    new_folder: Option<Item>,
}

/// Resultado de una tecla dentro del selector.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PickOutcome {
    Stay,
    Cancel,
    Chosen(PickValue),
}

impl Picker {
    /// Lista fija de opciones.
    pub fn choices(purpose: PickerPurpose, title: impl Into<String>, items: Vec<Item>) -> Self {
        Self {
            purpose,
            title: title.into(),
            dir: None,
            items,
            filter: String::new(),
            selected: 0,
            new_folder: None,
        }
    }

    /// Navegador de carpetas. El del Markdown muestra también los `.md`; los
    /// demás son selectores de carpetas, con `./` para elegir la que se ve.
    pub fn files(purpose: PickerPurpose, title: impl Into<String>, dir: &Path) -> Self {
        let mut picker = Self::choices(purpose, title, Vec::new());
        picker.open_dir(dir);
        picker
    }

    fn extension(&self) -> &'static str {
        if self.purpose == PickerPurpose::Markdown { "md" } else { "" }
    }

    /// Carga una carpeta: primero `..`, luego carpetas y luego archivos, cada
    /// grupo en orden alfabético sin distinguir mayúsculas. Ocultos fuera.
    pub fn open_dir(&mut self, dir: &Path) {
        let extension = self.extension();
        let mut dirs = Vec::new();
        let mut files = Vec::new();
        for entry in std::fs::read_dir(dir).into_iter().flatten().flatten() {
            let name = entry.file_name().to_string_lossy().into_owned();
            if name.starts_with('.') {
                continue;
            }
            let path = entry.path();
            if path.is_dir() {
                dirs.push(Item {
                    label: format!("{name}/"),
                    detail: String::new(),
                    value: PickValue::Dir(path),
                });
            } else if !extension.is_empty()
                && path.extension().is_some_and(|e| e.to_string_lossy().eq_ignore_ascii_case(extension))
            {
                files.push(Item { label: name, detail: String::new(), value: PickValue::File(path) });
            }
        }
        let by_name = |a: &Item, b: &Item| a.label.to_lowercase().cmp(&b.label.to_lowercase());
        dirs.sort_by(by_name);
        files.sort_by(by_name);

        self.items.clear();
        if let Some(parent) = dir.parent() {
            self.items.push(Item {
                label: "../".into(),
                detail: crate::i18n::Text::new("subir", "up").get().into(),
                value: PickValue::Up(parent.into()),
            });
        }
        if extension.is_empty() {
            self.items.push(Item {
                label: "./".into(),
                detail: crate::i18n::Text::new("usar esta carpeta", "use this folder").get().into(),
                value: PickValue::Here(dir.to_path_buf()),
            });
        }
        // shortcut: en la raíz se prueban las letras A: a Z: para llegar a otra unidad o
        // a una USB (Windows; en Linux y macOS no existen). Sin probar en Windows.
        if dir.parent().is_none() {
            for letter in 'A'..='Z' {
                let drive = PathBuf::from(format!("{letter}:{}", std::path::MAIN_SEPARATOR));
                if drive != dir && drive.is_dir() {
                    self.items.push(Item {
                        label: drive.display().to_string(),
                        detail: crate::i18n::Text::new("unidad", "drive").get().into(),
                        value: PickValue::Dir(drive),
                    });
                }
            }
        }
        self.items.extend(dirs);
        self.items.extend(files);
        self.dir = Some(dir.to_path_buf());
        self.filter.clear();
        self.new_folder = None;
        self.selected = 0;
    }

    /// Entradas que pasan el filtro (sin distinguir mayúsculas), las mejores
    /// primero: nombre exacto, nombre que empieza igual, nombre que lo
    /// contiene y, al final, coincidencias solo en la descripción. Así `ia`
    /// elige el perfil `ia` y no uno cuya materia diga «materia». `..` se
    /// muestra siempre arriba para poder salir de la carpeta.
    pub fn visible(&self) -> Vec<&Item> {
        let filter = self.filter.to_lowercase();
        let rank = |item: &Item| -> Option<u8> {
            if matches!(item.value, PickValue::Up(_)) {
                return Some(0);
            }
            if filter.is_empty() {
                return Some(1);
            }
            let label = item.label.trim_end_matches('/').to_lowercase();
            let stem = label.rsplit_once('.').map_or(label.as_str(), |(stem, _)| stem);
            if label == filter || stem == filter {
                Some(1)
            } else if label.starts_with(&filter) {
                Some(2)
            } else if label.contains(&filter) {
                Some(3)
            } else if item.detail.to_lowercase().contains(&filter) {
                Some(4)
            } else {
                None
            }
        };
        let mut ranked: Vec<(u8, &Item)> =
            self.items.iter().filter_map(|item| rank(item).map(|r| (r, item))).collect();
        ranked.sort_by_key(|(r, _)| *r);
        ranked.into_iter().map(|(_, item)| item).chain(&self.new_folder).collect()
    }

    pub fn move_by(&mut self, delta: isize) {
        let count = self.visible().len();
        if count == 0 {
            self.selected = 0;
            return;
        }
        let last = count as isize - 1;
        self.selected = (self.selected as isize + delta).clamp(0, last) as usize;
    }

    pub fn push_filter(&mut self, c: char) {
        self.filter.push(c);
        self.refresh_new_folder();
        self.select_first_match();
    }

    pub fn pop_filter(&mut self) -> bool {
        let popped = self.filter.pop().is_some();
        self.refresh_new_folder();
        self.select_first_match();
        popped
    }

    /// En un selector de carpetas, un nombre escrito que no existe se ofrece
    /// para crearlo: solo un nombre simple, sin rutas ni carpetas ocultas.
    fn refresh_new_folder(&mut self) {
        let name = self.filter.trim();
        let mut parts = Path::new(name).components();
        let simple = matches!((parts.next(), parts.next()), (Some(Component::Normal(_)), None))
            && !name.starts_with('.');
        let taken = self.items.iter().any(|item| {
            matches!(item.value, PickValue::Dir(_))
                && item.label.trim_end_matches('/').eq_ignore_ascii_case(name)
        });
        self.new_folder = match &self.dir {
            Some(dir) if self.extension().is_empty() && simple && !taken && !dir.join(name).exists() => {
                Some(Item {
                    label: format!("+ {name}/"),
                    detail: crate::i18n::Text::new("crear y entrar", "create and open").get().into(),
                    value: PickValue::New(dir.join(name)),
                })
            }
            _ => None,
        };
    }

    /// Crea la carpeta nueva y entra en ella con `./` marcado; si no se puede,
    /// el motivo queda en su fila.
    fn create_folder(&mut self, path: &Path) {
        let created = if may_write(path) {
            std::fs::create_dir_all(path)
        } else {
            Err(std::io::ErrorKind::PermissionDenied.into())
        };
        match created {
            Ok(()) => {
                logging::info(format_args!("tui: created folder {}", path.display()));
                self.open_dir(path);
                self.selected =
                    self.items.iter().position(|i| matches!(i.value, PickValue::Here(_))).unwrap_or(0);
            }
            Err(error) => {
                logging::warn(format_args!("tui: could not create {}: {error}", path.display()));
                if let Some(item) = &mut self.new_folder {
                    item.detail = tr!(es: "no se pudo crear: {error}", en: "could not create it: {error}");
                }
            }
        }
    }

    // Tras filtrar se salta el `..` para que Enter elija la primera coincidencia.
    fn select_first_match(&mut self) {
        let visible = self.visible();
        self.selected =
            if !self.filter.is_empty() && visible.len() > 1 && matches!(visible[0].value, PickValue::Up(_)) {
                1
            } else {
                0
            };
    }

    /// Enter: entra en carpetas y devuelve lo demás a quien abrió el selector.
    pub fn activate(&mut self) -> PickOutcome {
        let Some(item) = self.visible().get(self.selected).map(|item| item.value.clone()) else {
            return PickOutcome::Stay;
        };
        match item {
            PickValue::New(path) => {
                self.create_folder(&path);
                PickOutcome::Stay
            }
            PickValue::Up(dir) | PickValue::Dir(dir) if self.purpose != PickerPurpose::Folder => {
                self.open_dir(&dir);
                PickOutcome::Stay
            }
            other => PickOutcome::Chosen(other),
        }
    }

    /// Retroceso con el filtro vacío o flecha izquierda: carpeta padre.
    pub fn go_up(&mut self) {
        if let Some(parent) = self.dir.as_ref().and_then(|d| d.parent()).map(Path::to_path_buf) {
            let previous = self.dir.clone();
            self.open_dir(&parent);
            // Deja seleccionada la carpeta de la que se salió.
            if let Some(previous) = previous {
                let position =
                    self.visible().iter().position(|i| i.value == PickValue::Dir(previous.clone()));
                self.selected = position.unwrap_or(0);
            }
        }
    }

    /// Las teclas de cualquier selector: mover, filtrar, entrar, subir y Esc.
    pub fn handle_key(&mut self, code: KeyCode) -> PickOutcome {
        let browsing = self.dir.is_some();
        match code {
            KeyCode::Esc => return PickOutcome::Cancel,
            KeyCode::Up => self.move_by(-1),
            KeyCode::Down => self.move_by(1),
            KeyCode::PageUp => self.move_by(-10),
            KeyCode::PageDown => self.move_by(10),
            KeyCode::Enter => return self.activate(),
            KeyCode::Right if browsing => return self.activate(),
            KeyCode::Left if browsing => self.go_up(),
            // Con el filtro vacío, retroceso sube de carpeta.
            KeyCode::Backspace => {
                if !self.pop_filter() && browsing {
                    self.go_up();
                }
            }
            KeyCode::Char(c) => self.push_filter(c),
            _ => {}
        }
        PickOutcome::Stay
    }
}

/// Las pruebas solo escriben dentro de sus carpetas temporales (`.tmp…`): la de
/// teclas al azar recorre todo el disco con los selectores.
pub fn may_write(path: &Path) -> bool {
    !cfg!(test)
        || path
            .strip_prefix(std::env::temp_dir())
            .ok()
            .and_then(|inside| inside.components().next())
            .is_some_and(|first| first.as_os_str().to_string_lossy().starts_with(".tmp"))
}

/// Los diseños, con su descripción.
pub fn design_items(project: &Project) -> Vec<Item> {
    list_designs(project)
        .into_iter()
        .filter_map(|key| {
            let design = load_design(project, Some(&key)).ok()?;
            let detail = design.manifest.description.as_ref().map(|d| d.get().to_owned()).unwrap_or_default();
            Some(Item { label: key.clone(), detail, value: PickValue::Choice(key) })
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn folders_come_first_and_only_markdown_is_listed() {
        let directory = tempfile::tempdir().unwrap();
        let root = directory.path();
        std::fs::create_dir(root.join("IA")).unwrap();
        std::fs::write(root.join("b.md"), "").unwrap();
        std::fs::write(root.join("A.MD"), "").unwrap();
        std::fs::write(root.join("notas.txt"), "").unwrap();
        std::fs::write(root.join(".oculto.md"), "").unwrap();
        let picker = Picker::files(PickerPurpose::Markdown, "x", root);
        let labels: Vec<&str> = picker.items.iter().map(|i| i.label.as_str()).collect();
        assert_eq!(labels, ["../", "IA/", "A.MD", "b.md"]);
    }

    #[test]
    fn without_an_extension_only_folders_are_listed_and_dot_chooses_the_current_one() {
        let directory = tempfile::tempdir().unwrap();
        let root = directory.path();
        std::fs::create_dir(root.join("IA")).unwrap();
        std::fs::write(root.join("a.md"), "").unwrap();
        std::fs::write(root.join("sin-extension."), "").unwrap();
        let mut picker = Picker::files(PickerPurpose::Output, "x", root);
        let labels: Vec<&str> = picker.items.iter().map(|i| i.label.as_str()).collect();
        assert_eq!(labels, ["../", "./", "IA/"]);
        picker.move_by(2);
        assert_eq!(picker.activate(), PickOutcome::Stay);
        picker.move_by(1);
        assert_eq!(picker.activate(), PickOutcome::Chosen(PickValue::Here(root.join("IA"))));
    }

    #[test]
    fn enter_opens_folders_and_returns_files() {
        let directory = tempfile::tempdir().unwrap();
        let root = directory.path();
        std::fs::create_dir(root.join("IA")).unwrap();
        std::fs::write(root.join("IA").join("Tarea.md"), "").unwrap();
        let mut picker = Picker::files(PickerPurpose::Markdown, "x", root);
        picker.push_filter('i');
        assert_eq!(picker.activate(), PickOutcome::Stay);
        assert_eq!(picker.dir.as_deref(), Some(root.join("IA").as_path()));
        picker.push_filter('t');
        assert_eq!(picker.activate(), PickOutcome::Chosen(PickValue::File(root.join("IA").join("Tarea.md"))));
        picker.go_up();
        assert_eq!(picker.dir.as_deref(), Some(root));
        assert_eq!(picker.visible()[picker.selected].label, "IA/");
    }

    fn type_filter(picker: &mut Picker, text: &str) {
        picker.filter.clear();
        for c in text.chars() {
            picker.push_filter(c);
        }
    }

    fn offers_to_create(picker: &Picker) -> bool {
        picker.visible().iter().any(|i| matches!(i.value, PickValue::New(_)))
    }

    #[test]
    fn a_typed_folder_name_can_be_created_and_chosen() {
        let directory = tempfile::tempdir().unwrap();
        let root = directory.path();
        std::fs::create_dir(root.join("IA")).unwrap();
        let mut picker = Picker::files(PickerPurpose::Copies, "x", root);
        // Lo que ya existe (sin importar mayúsculas), rutas y ocultas no se ofrecen.
        for text in ["ia", "a/b", ".oculta", ".."] {
            type_filter(&mut picker, text);
            assert!(!offers_to_create(&picker), "{text}");
        }
        type_filter(&mut picker, "Redes");
        assert_eq!(picker.visible()[picker.selected].label, "+ Redes/");
        assert_eq!(picker.activate(), PickOutcome::Stay);
        assert!(root.join("Redes").is_dir());
        assert_eq!(picker.dir.as_deref(), Some(root.join("Redes").as_path()));
        // Dentro queda marcado ./: Enter elige la carpeta recién creada.
        assert_eq!(picker.activate(), PickOutcome::Chosen(PickValue::Here(root.join("Redes"))));
        // El selector del Markdown no crea carpetas.
        let mut markdown = Picker::files(PickerPurpose::Markdown, "x", root);
        type_filter(&mut markdown, "nueva");
        assert!(!offers_to_create(&markdown));
    }

    #[test]
    fn a_filesystem_root_only_offers_drives_that_exist() {
        let root = Path::new(std::path::MAIN_SEPARATOR_STR);
        let picker = Picker::files(PickerPurpose::Copies, "x", root);
        let drives: Vec<&Item> = picker.items.iter().filter(|i| i.label.contains(':')).collect();
        assert!(drives.iter().all(|i| matches!(&i.value, PickValue::Dir(d) if d.is_dir())));
        // En Linux y macOS no hay unidades con letra.
        if std::path::MAIN_SEPARATOR == '/' {
            assert!(drives.is_empty());
        }
    }

    #[test]
    fn the_filter_matches_label_or_detail() {
        let items = vec![
            Item {
                label: "ia".into(),
                detail: "Inteligencia artificial".into(),
                value: PickValue::Choice("ia".into()),
            },
            Item {
                label: "pm".into(), detail: "Programación".into(), value: PickValue::Choice("pm".into())
            },
            Item {
                label: "example".into(),
                detail: "Nombre de la materia".into(),
                value: PickValue::Choice("example".into()),
            },
        ];
        let mut picker = Picker::choices(PickerPurpose::Profile, "x", items);
        // «ia» está en la descripción de example, pero el nombre exacto gana.
        picker.push_filter('i');
        picker.push_filter('a');
        assert_eq!(picker.visible()[0].label, "ia");
        assert_eq!(picker.visible().len(), 2);
        picker.filter.clear();
        for c in "intel".chars() {
            picker.push_filter(c);
        }
        assert_eq!(picker.visible().len(), 1);
        assert_eq!(picker.activate(), PickOutcome::Chosen(PickValue::Choice("ia".into())));
        picker.move_by(10);
        assert_eq!(picker.selected, 0);
    }
}
