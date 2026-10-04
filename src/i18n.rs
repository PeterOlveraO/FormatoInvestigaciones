//! Idioma de la interfaz (menú, mensajes y ayuda): español o inglés.
//!
//! El PDF no pasa por aquí: su idioma es el del trabajo y lo fija la plantilla.

use std::cell::Cell;

use crate::settings::Settings;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Lang {
    Es,
    En,
}

/// Claves de `settings.toml` (o del entorno) que fijan el idioma.
pub const LANG_SETTINGS: [&str; 2] = ["IDIOMA", "INTERFACE_LANGUAGE"];

// Por hilo y no global: así cada prueba (que corre en su propio hilo) fija su
// idioma sin pisar a las demás. La TUI lo copia al hilo que genera el PDF.
thread_local! {
    static CURRENT: Cell<Lang> = const { Cell::new(Lang::Es) };
}

pub fn current() -> Lang {
    CURRENT.with(Cell::get)
}

pub fn set(lang: Lang) {
    CURRENT.with(|cell| cell.set(lang));
}

impl Lang {
    /// Código que se guarda en `settings.toml` y se pasa a los filtros Lua.
    pub fn code(self) -> &'static str {
        match self {
            Lang::Es => "es",
            Lang::En => "en",
        }
    }

    pub fn other(self) -> Lang {
        match self {
            Lang::Es => Lang::En,
            Lang::En => Lang::Es,
        }
    }
}

/// Acepta el código o el nombre del idioma, en cualquiera de los dos.
pub fn parse(value: &str) -> Option<Lang> {
    match value.trim().to_lowercase().as_str() {
        "es" | "español" | "espanol" | "spanish" => Some(Lang::Es),
        "en" | "inglés" | "ingles" | "english" => Some(Lang::En),
        _ => None,
    }
}

/// Idioma del sistema operativo: español si empieza por `es`, si no inglés.
/// `sys-locale` lo lee también en Windows, donde no existe la variable `LANG`.
pub fn detect() -> Lang {
    match sys_locale::get_locale() {
        Some(locale) if locale.to_lowercase().starts_with("es") => Lang::Es,
        _ => Lang::En,
    }
}

/// El idioma guardado en el entorno o en `settings.toml`, si hay uno válido.
pub fn configured(settings: &Settings) -> Option<Lang> {
    parse(&settings.get(&LANG_SETTINGS))
}

/// Orden: la opción `--lang`, luego `IDIOMA` y, si no, el idioma del sistema.
pub fn resolve(option: Option<Lang>, settings: &Settings) -> Lang {
    option.or_else(|| configured(settings)).unwrap_or_else(detect)
}

/// Texto constante en los dos idiomas (etiquetas, ayudas).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Text {
    pub es: &'static str,
    pub en: &'static str,
}

impl Text {
    pub const fn new(es: &'static str, en: &'static str) -> Self {
        Self { es, en }
    }

    pub fn get(self) -> &'static str {
        match current() {
            Lang::Es => self.es,
            Lang::En => self.en,
        }
    }
}

impl std::fmt::Display for Text {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.get())
    }
}

/// Formatea un mensaje en el idioma actual. Pide las dos versiones juntas, así
/// que no se puede añadir un mensaje sin su traducción.
#[macro_export]
macro_rules! tr {
    (es: $es:literal, en: $en:literal $(, $arg:expr)* $(,)?) => {
        match $crate::i18n::current() {
            $crate::i18n::Lang::Es => format!($es $(, $arg)*),
            $crate::i18n::Lang::En => format!($en $(, $arg)*),
        }
    };
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn messages_follow_the_current_language() {
        let name = "x.md";
        set(Lang::Es);
        assert_eq!(tr!(es: "No existe {name}", en: "{name} does not exist"), "No existe x.md");
        assert_eq!(Text::new("Materia", "Course").get(), "Materia");
        set(Lang::En);
        assert_eq!(tr!(es: "No existe {}", en: "{} does not exist", name), "x.md does not exist");
        assert_eq!(Text::new("Materia", "Course").get(), "Course");
    }

    #[test]
    fn languages_are_parsed_by_code_or_name() {
        assert_eq!(parse("ES"), Some(Lang::Es));
        assert_eq!(parse("English"), Some(Lang::En));
        assert_eq!(parse("fr"), None);
        assert_eq!(Lang::Es.other(), Lang::En);
    }

    #[test]
    fn the_option_wins_over_the_setting() {
        let directory = tempfile::tempdir().unwrap();
        let env = directory.path().join("settings.toml");
        std::fs::write(&env, "ZZ=1\n").unwrap();
        let settings = Settings::load(&env).unwrap();
        assert_eq!(resolve(Some(Lang::En), &settings), Lang::En);
    }
}
