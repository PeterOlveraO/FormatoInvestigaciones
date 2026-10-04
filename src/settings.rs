//! Datos fijos de la persona usuaria: variables de entorno y archivo `.env`.
//!
//! No se modifica el entorno del proceso: el entorno real gana y el `.env`
//! sirve de respaldo, igual que hacía `os.environ.setdefault` en Python.

use std::collections::HashMap;
use std::path::Path;
use std::sync::LazyLock;

use regex::Regex;

use crate::encoding::decode_text;
use crate::error::{GenerationError, Result};
use crate::logging;

#[derive(Debug, Default, Clone)]
pub struct Settings {
    file: HashMap<String, String>,
}

impl Settings {
    /// Carga un `.env` sencillo (`CLAVE=valor`, comentarios con `#`, `export`
    /// opcional y comillas). Si el archivo no existe no es error. Un `.env`
    /// que no se puede leer queda en el registro aunque quien llama lo ignore.
    pub fn load(path: &Path) -> Result<Self> {
        let result = Self::read(path);
        match &result {
            Ok(_) if !path.exists() => {
                logging::debug(format_args!("settings: {} does not exist", path.display()))
            }
            Ok(settings) => {
                logging::debug(format_args!("settings: {} ({} keys)", path.display(), settings.file.len()))
            }
            Err(error) => logging::warn(format_args!("settings file {} not read: {error}", path.display())),
        }
        result
    }

    fn read(path: &Path) -> Result<Self> {
        if !path.exists() {
            return Ok(Self::default());
        }
        if !path.is_file() {
            return Err(GenerationError::new(tr!(
                es: "El archivo de configuración no es un archivo: {}",
                en: "The settings file is not a file: {}",
                path.display()
            )));
        }
        static KEY: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"^[A-Za-z_][A-Za-z0-9_]*$").unwrap());

        let text = decode_text(&std::fs::read(path)?, path)?;
        let mut file = HashMap::new();
        for (number, raw_line) in text.lines().enumerate() {
            let mut line = raw_line.trim();
            if line.is_empty() || line.starts_with('#') {
                continue;
            }
            if let Some(rest) = line.strip_prefix("export ") {
                line = rest.trim_start();
            }
            let Some((key, value)) = line.split_once('=') else {
                return Err(GenerationError::new(tr!(
                    es: "Sintaxis no válida en {}, línea {}: se esperaba CLAVE=VALOR",
                    en: "Invalid syntax in {}, line {}: expected KEY=VALUE",
                    path.display(),
                    number + 1
                )));
            };
            let (key, mut value) = (key.trim(), value.trim());
            if !KEY.is_match(key) {
                return Err(GenerationError::new(tr!(
                    es: "Nombre de variable no válido en {}, línea {}: {key}",
                    en: "Invalid variable name in {}, line {}: {key}",
                    path.display(),
                    number + 1
                )));
            }
            let quoted = value.len() >= 2
                && (value.starts_with('"') && value.ends_with('"')
                    || value.starts_with('\'') && value.ends_with('\''));
            if quoted {
                value = &value[1..value.len() - 1];
            } else if let Some((before, _)) = value.split_once(" #") {
                value = before.trim_end();
            }
            file.insert(key.to_owned(), value.to_owned());
        }
        Ok(Self { file })
    }

    /// Guarda `key=value` en el `.env`: reemplaza la línea de esa clave (con o
    /// sin `export`) o la añade al final, sin tocar las demás ni los
    /// comentarios. Si el archivo no existe, lo crea.
    pub fn save_value(path: &Path, key: &str, value: &str) -> Result<()> {
        let text = if path.exists() { decode_text(&std::fs::read(path)?, path)? } else { String::new() };
        let new_line = format!("{key}=\"{value}\"");
        let mut found = false;
        let mut lines: Vec<String> = text
            .lines()
            .map(|line| {
                let bare = line.trim_start().strip_prefix("export ").unwrap_or(line.trim_start());
                let is_key = bare.split_once('=').is_some_and(|(k, _)| k.trim() == key);
                if is_key && !found {
                    found = true;
                    new_line.clone()
                } else {
                    line.to_owned()
                }
            })
            .collect();
        if !found {
            lines.push(new_line);
        }
        std::fs::write(path, lines.join("\n") + "\n")?;
        logging::info(format_args!("settings: {key} saved in {}", path.display()));
        Ok(())
    }

    /// Primer valor no vacío entre los nombres dados: primero el entorno y
    /// luego el `.env`. Se aceptan los nombres en español y en inglés.
    pub fn get(&self, names: &[&str]) -> String {
        for name in names {
            if let Ok(value) = std::env::var(name)
                && !value.trim().is_empty()
            {
                return value.trim().to_owned();
            }
            if let Some(value) = self.file.get(*name).filter(|v| !v.trim().is_empty()) {
                return value.trim().to_owned();
            }
        }
        String::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn dotenv_values_are_parsed() {
        let directory = tempfile::tempdir().unwrap();
        let path = directory.path().join(".env");
        std::fs::write(
            &path,
            "# comentario\nexport ZZ_UNI=\"Mi # Uni\"\nZZ_FAC=Facultad # nota\nZZ_EMPTY=\n",
        )
        .unwrap();
        let settings = Settings::load(&path).unwrap();
        assert_eq!(settings.get(&["ZZ_UNI"]), "Mi # Uni");
        assert_eq!(settings.get(&["ZZ_FAC"]), "Facultad");
        assert_eq!(settings.get(&["ZZ_EMPTY", "ZZ_FAC"]), "Facultad");
        assert_eq!(settings.get(&["ZZ_MISSING"]), "");
    }

    #[test]
    fn a_value_is_replaced_or_appended_keeping_the_rest() {
        let directory = tempfile::tempdir().unwrap();
        let path = directory.path().join(".env");
        Settings::save_value(&path, "IDIOMA", "en").unwrap();
        assert_eq!(std::fs::read_to_string(&path).unwrap(), "IDIOMA=\"en\"\n");

        std::fs::write(&path, "# datos\nUNIVERSIDAD=\"U\"\nexport IDIOMA=es\n#IDIOMA=viejo\n").unwrap();
        Settings::save_value(&path, "IDIOMA", "en").unwrap();
        let text = std::fs::read_to_string(&path).unwrap();
        assert_eq!(text, "# datos\nUNIVERSIDAD=\"U\"\nIDIOMA=\"en\"\n#IDIOMA=viejo\n");
        assert_eq!(Settings::load(&path).unwrap().get(&["ZZ_NOPE", "IDIOMA"]), "en");
    }

    #[test]
    fn invalid_lines_are_errors() {
        let directory = tempfile::tempdir().unwrap();
        let path = directory.path().join(".env");
        std::fs::write(&path, "SIN_IGUAL\n").unwrap();
        assert!(Settings::load(&path).is_err());
        std::fs::write(&path, "1MAL=x\n").unwrap();
        assert!(Settings::load(&path).is_err());
    }
}
