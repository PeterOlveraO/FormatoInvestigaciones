//! Selector de listas: el mismo componente recorre carpetas para elegir un
//! Markdown y muestra los perfiles, las plantillas y las carpetas del proyecto.
//! Escribir filtra la lista; así no hace falta teclear rutas.

use std::path::{Path, PathBuf};

/// Lo que devuelve una entrada al elegirla.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PickValue {
    /// Subir a la carpeta padre.
    Up(PathBuf),
    /// Entrar en una carpeta.
    Dir(PathBuf),
    /// Archivo elegido.
    File(PathBuf),
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
    Folder,
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
        Self { purpose, title: title.into(), dir: None, items, filter: String::new(), selected: 0 }
    }

    /// Navegador de archivos que muestra carpetas y archivos con `extension`.
    pub fn files(purpose: PickerPurpose, title: impl Into<String>, dir: &Path, extension: &str) -> Self {
        let mut picker = Self::choices(purpose, title, Vec::new());
        picker.open_dir(dir, extension);
        picker
    }

    /// Carga una carpeta: primero `..`, luego carpetas y luego archivos, cada
    /// grupo en orden alfabético sin distinguir mayúsculas. Ocultos fuera.
    pub fn open_dir(&mut self, dir: &Path, extension: &str) {
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
            } else if path.extension().is_some_and(|e| e.to_string_lossy().eq_ignore_ascii_case(extension)) {
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
                detail: "up".into(),
                value: PickValue::Up(parent.into()),
            });
        }
        self.items.extend(dirs);
        self.items.extend(files);
        self.dir = Some(dir.to_path_buf());
        self.filter.clear();
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
        ranked.into_iter().map(|(_, item)| item).collect()
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
        self.select_first_match();
    }

    pub fn pop_filter(&mut self) -> bool {
        let popped = self.filter.pop().is_some();
        self.select_first_match();
        popped
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
    pub fn activate(&mut self, extension: &str) -> PickOutcome {
        let Some(item) = self.visible().get(self.selected).map(|item| item.value.clone()) else {
            return PickOutcome::Stay;
        };
        match item {
            PickValue::Up(dir) | PickValue::Dir(dir) if self.purpose != PickerPurpose::Folder => {
                self.open_dir(&dir, extension);
                PickOutcome::Stay
            }
            other => PickOutcome::Chosen(other),
        }
    }

    /// Retroceso con el filtro vacío o flecha izquierda: carpeta padre.
    pub fn go_up(&mut self, extension: &str) {
        if let Some(parent) = self.dir.as_ref().and_then(|d| d.parent()).map(Path::to_path_buf) {
            let previous = self.dir.clone();
            self.open_dir(&parent, extension);
            // Deja seleccionada la carpeta de la que se salió.
            if let Some(previous) = previous {
                let position =
                    self.visible().iter().position(|i| i.value == PickValue::Dir(previous.clone()));
                self.selected = position.unwrap_or(0);
            }
        }
    }
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
        let picker = Picker::files(PickerPurpose::Markdown, "x", root, "md");
        let labels: Vec<&str> = picker.items.iter().map(|i| i.label.as_str()).collect();
        assert_eq!(labels, ["../", "IA/", "A.MD", "b.md"]);
    }

    #[test]
    fn enter_opens_folders_and_returns_files() {
        let directory = tempfile::tempdir().unwrap();
        let root = directory.path();
        std::fs::create_dir(root.join("IA")).unwrap();
        std::fs::write(root.join("IA").join("Tarea.md"), "").unwrap();
        let mut picker = Picker::files(PickerPurpose::Markdown, "x", root, "md");
        picker.push_filter('i');
        assert_eq!(picker.activate("md"), PickOutcome::Stay);
        assert_eq!(picker.dir.as_deref(), Some(root.join("IA").as_path()));
        picker.push_filter('t');
        assert_eq!(
            picker.activate("md"),
            PickOutcome::Chosen(PickValue::File(root.join("IA").join("Tarea.md")))
        );
        picker.go_up("md");
        assert_eq!(picker.dir.as_deref(), Some(root));
        assert_eq!(picker.visible()[picker.selected].label, "IA/");
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
        assert_eq!(picker.activate("md"), PickOutcome::Chosen(PickValue::Choice("ia".into())));
        picker.move_by(10);
        assert_eq!(picker.selected, 0);
    }
}
