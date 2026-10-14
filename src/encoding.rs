//! Decodificación de texto: UTF-8 con respaldo Windows-1252.
//!
//! Los trabajos escritos en Windows suelen venir en Windows-1252, y el log de
//! pdflatex mezcla UTF-8 con la codificación interna de la fuente (T1).

use std::path::Path;

use crate::error::{GenerationError, Result};

// Bytes 0x80..=0x9F de Windows-1252; U+FFFD marca los cinco sin definir.
const CP1252_HIGH: [char; 32] = [
    '€', '\u{FFFD}', '‚', 'ƒ', '„', '…', '†', '‡', 'ˆ', '‰', 'Š', '‹', 'Œ', '\u{FFFD}', 'Ž', '\u{FFFD}',
    '\u{FFFD}', '‘', '’', '“', '”', '•', '–', '—', '˜', '™', 'š', '›', 'œ', '\u{FFFD}', 'ž', 'Ÿ',
];

/// Decodifica Windows-1252. En modo estricto un byte sin definir es `None`;
/// si no, se sustituye por U+FFFD.
pub fn decode_cp1252(raw: &[u8], strict: bool) -> Option<String> {
    raw.iter()
        .map(|&byte| match byte {
            0x80..=0x9F => CP1252_HIGH[usize::from(byte - 0x80)],
            _ => char::from(byte),
        })
        .map(|c| (c != '\u{FFFD}' || !strict).then_some(c))
        .collect()
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
        // Comillas y guion de Word; 0x81 no existe en Windows-1252.
        assert_eq!(decode_cp1252(&[0x93, b'a', 0x94, 0x96], true).unwrap(), "“a”–");
        assert_eq!(decode_cp1252(&[0x81], true), None);
        assert_eq!(decode_cp1252(&[0x81], false).unwrap(), "\u{FFFD}");
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
