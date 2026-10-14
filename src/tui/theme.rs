//! Temas de colores del menú. El clásico usa los colores de la terminal; los
//! demás son paletas conocidas con fondo propio (necesitan color verdadero).

use std::cell::Cell;

use ratatui::style::Color;

use crate::i18n::Text;

/// Claves de `settings.toml` donde se guarda el tema, la española primero.
pub const THEME_SETTINGS: [&str; 2] = ["TEMA", "THEME"];

pub struct Theme {
    /// Lo que se guarda en `settings.toml`.
    pub key: &'static str,
    pub name: Text,
    pub bg: Color,
    pub fg: Color,
    /// Bordes de las ventanas, letras de las teclas y lo que se puede abrir.
    pub accent: Color,
    /// Texto sobre el color de acento.
    pub on_accent: Color,
    /// Fondo de la fila marcada.
    pub selection: Color,
    /// Texto secundario.
    pub dim: Color,
    pub ok: Color,
    pub warn: Color,
    pub error: Color,
}

const fn rgb(hex: u32) -> Color {
    Color::Rgb((hex >> 16) as u8, (hex >> 8) as u8, hex as u8)
}

pub const THEMES: [Theme; 6] = [
    Theme {
        key: "classic",
        name: Text::new("Clásico (colores de la terminal)", "Classic (terminal colors)"),
        bg: Color::Reset,
        fg: Color::Reset,
        accent: Color::Cyan,
        on_accent: Color::Black,
        selection: Color::DarkGray,
        dim: Color::DarkGray,
        ok: Color::Green,
        warn: Color::Yellow,
        error: Color::Red,
    },
    Theme {
        key: "dracula",
        name: Text::new("Dracula", "Dracula"),
        bg: rgb(0x282a36),
        fg: rgb(0xf8f8f2),
        accent: rgb(0xbd93f9),
        on_accent: rgb(0x282a36),
        selection: rgb(0x44475a),
        dim: rgb(0x6272a4),
        ok: rgb(0x50fa7b),
        warn: rgb(0xf1fa8c),
        error: rgb(0xff5555),
    },
    Theme {
        key: "nord",
        name: Text::new("Nord", "Nord"),
        bg: rgb(0x2e3440),
        fg: rgb(0xd8dee9),
        accent: rgb(0x88c0d0),
        on_accent: rgb(0x2e3440),
        selection: rgb(0x434c5e),
        dim: rgb(0x7b88a1),
        ok: rgb(0xa3be8c),
        warn: rgb(0xebcb8b),
        error: rgb(0xbf616a),
    },
    Theme {
        key: "gruvbox",
        name: Text::new("Gruvbox", "Gruvbox"),
        bg: rgb(0x282828),
        fg: rgb(0xebdbb2),
        accent: rgb(0xfe8019),
        on_accent: rgb(0x282828),
        selection: rgb(0x504945),
        dim: rgb(0x928374),
        ok: rgb(0xb8bb26),
        warn: rgb(0xfabd2f),
        error: rgb(0xfb4934),
    },
    Theme {
        key: "catppuccin",
        name: Text::new("Catppuccin Mocha", "Catppuccin Mocha"),
        bg: rgb(0x1e1e2e),
        fg: rgb(0xcdd6f4),
        accent: rgb(0xcba6f7),
        on_accent: rgb(0x1e1e2e),
        selection: rgb(0x45475a),
        dim: rgb(0x7f849c),
        ok: rgb(0xa6e3a1),
        warn: rgb(0xf9e2af),
        error: rgb(0xf38ba8),
    },
    Theme {
        key: "tokyo-night",
        name: Text::new("Tokyo Night", "Tokyo Night"),
        bg: rgb(0x1a1b26),
        fg: rgb(0xc0caf5),
        accent: rgb(0x7aa2f7),
        on_accent: rgb(0x1a1b26),
        selection: rgb(0x283457),
        dim: rgb(0x565f89),
        ok: rgb(0x9ece6a),
        warn: rgb(0xe0af68),
        error: rgb(0xf7768e),
    },
];

// Por hilo, como el idioma: el menú dibuja y atiende teclas en el mismo hilo,
// y cada prueba empieza con el clásico.
thread_local! {
    static CURRENT: Cell<usize> = const { Cell::new(0) };
}

pub fn current() -> &'static Theme {
    &THEMES[CURRENT.with(Cell::get)]
}

/// Cambia el tema; una clave desconocida (o vacía) deja el que estaba.
pub fn set(key: &str) -> bool {
    let found = THEMES.iter().position(|t| t.key.eq_ignore_ascii_case(key.trim()));
    if let Some(index) = found {
        CURRENT.with(|c| c.set(index));
    }
    found.is_some()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn themes_are_chosen_by_key() {
        assert_eq!(current().key, "classic");
        assert!(set("Nord"));
        assert_eq!(current().key, "nord");
        assert!(!set("no-existe"));
        assert_eq!(current().key, "nord");
        assert_eq!(rgb(0x282a36), Color::Rgb(0x28, 0x2a, 0x36));
    }
}
