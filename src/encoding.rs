//! Decodificación de texto: UTF-8 con respaldo Windows-1252.
//!
//! Los trabajos escritos en Windows suelen venir en Windows-1252, y el log de
//! pdflatex mezcla UTF-8 con la codificación interna de la fuente (T1).

use std::path::Path;

use crate::error::{GenerationError, Result};

// Bytes 0x80..=0x9F de Windows-1252; `None` son los cinco sin definir.
const CP1252_HIGH: [Option<char>; 32] = [
    Some('\u{20AC}'),
    None,
    Some('\u{201A}'),
    Some('\u{0192}'),
    Some('\u{201E}'),
    Some('\u{2026}'),
    Some('\u{2020}'),
    Some('\u{2021}'),
    Some('\u{02C6}'),
    Some('\u{2030}'),
    Some('\u{0160}'),
    Some('\u{2039}'),
    Some('\u{0152}'),
    None,
    Some('\u{017D}'),
    None,
    None,
    Some('\u{2018}'),
    Some('\u{2019}'),
    Some('\u{201C}'),
    Some('\u{201D}'),
    Some('\u{2022}'),
    Some('\u{2013}'),
    Some('\u{2014}'),
    Some('\u{02DC}'),
    Some('\u{2122}'),
    Some('\u{0161}'),
    Some('\u{203A}'),
    Some('\u{0153}'),
    None,
    Some('\u{017E}'),
    Some('\u{0178}'),
];

/// Decodifica Windows-1252. En modo estricto un byte sin definir es `None`;
/// si no, se sustituye por U+FFFD.
pub fn decode_cp1252(raw: &[u8], strict: bool) -> Option<String> {
    let mut text = String::with_capacity(raw.len());
    for &byte in raw {
        let character = match byte {
            0x80..=0x9F => CP1252_HIGH[usize::from(byte - 0x80)],
            _ => Some(char::from(byte)),
        };
        match character {
            Some(c) => text.push(c),
            None if strict => return None,
            None => text.push('\u{FFFD}'),
        }
    }
    Some(text)
}

/// Lee archivos de texto en UTF-8 (con o sin BOM) o Windows-1252.
pub fn decode_text(raw: &[u8], path: &Path) -> Result<String> {
    let raw = raw.strip_prefix(b"\xEF\xBB\xBF").unwrap_or(raw);
    if let Ok(text) = std::str::from_utf8(raw) {
        return Ok(text.to_owned());
    }
    decode_cp1252(raw, true).ok_or_else(|| {
        GenerationError::new(tr!(
            es: "No se pudo leer {}: guárdalo en UTF-8 o Windows-1252.",
            en: "Could not read {}: save it as UTF-8 or Windows-1252.",
            path.display()
        ))
    })
}

/// Salida de una herramienta externa, que puede seguir la configuración regional.
pub fn decode_process_output(raw: &[u8]) -> String {
    match std::str::from_utf8(raw) {
        Ok(text) => text.to_owned(),
        Err(_) => decode_cp1252(raw, false).unwrap_or_default(),
    }
}

/// El log de pdflatex se decodifica línea por línea: los mensajes van en UTF-8
/// pero las líneas de división silábica salen en T1, y tratar todo el archivo
/// como cp1252 convertiría «└» en «â””».
pub fn decode_latex_log(raw: &[u8]) -> String {
    raw.split(|&byte| byte == b'\n').map(decode_process_output).collect::<Vec<_>>().join("\n")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn windows_encoding_is_accepted() {
        let raw = [b'I', b'n', b'f', b'o', b'r', b'm', b'a', b'c', 0xED, b'a'];
        assert_eq!(decode_text(&raw, Path::new("x.md")).unwrap(), "Informacía");
    }

    #[test]
    fn each_log_line_is_decoded_on_its_own() {
        let mut raw =
            "LaTeX Warning: Caracter Unicode sin definir: └ (U+2514) on input line 7.\n".as_bytes().to_vec();
        raw.extend_from_slice(b"Underfull \\hbox in paragraph at lines 5--5: ingenier\xeda\n");
        raw.extend_from_slice("Otro aviso con acento: compilación\n".as_bytes());
        let log = decode_latex_log(&raw);
        assert!(log.contains("└ (U+2514)"));
        assert!(log.contains("compilación"));
        assert!(log.contains("Underfull"));
    }
}
