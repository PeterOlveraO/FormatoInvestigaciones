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

/// Lee archivos de texto en UTF-8 (con o sin BOM), UTF-16 (con BOM) o
/// Windows-1252. Un byte suelto en cp1252 no convierte el resto del archivo
/// UTF-8 en mojibake: cada trozo se decodifica según sea válido o no.
pub fn decode_text(raw: &[u8], path: &Path) -> Result<String> {
    let unreadable = || {
        GenerationError::new(tr!(
            es: "No se pudo leer {}: guárdalo en UTF-8 o Windows-1252.",
            en: "Could not read {}: save it as UTF-8 or Windows-1252.",
            path.display()
        ))
    };
    if let Some(text) = decode_utf16_bom(raw) {
        return text.ok_or_else(unreadable);
    }
    let raw = raw.strip_prefix(b"\xEF\xBB\xBF").unwrap_or(raw);
    decode_utf8_or_cp1252(raw).ok_or_else(unreadable)
}

/// Lo válido en UTF-8 se lee como UTF-8 y cada byte suelto, en Windows-1252
/// estricto. `None` si aparece un byte que cp1252 no define.
fn decode_utf8_or_cp1252(raw: &[u8]) -> Option<String> {
    let mut text = String::with_capacity(raw.len());
    for chunk in raw.utf8_chunks() {
        text.push_str(chunk.valid());
        text.push_str(&decode_cp1252(chunk.invalid(), true)?);
    }
    Some(text)
}

/// UTF-16 con BOM (little o big endian). `None` si no empieza por BOM;
/// `Some(None)` si lo tiene pero el contenido no es UTF-16 válido.
fn decode_utf16_bom(raw: &[u8]) -> Option<Option<String>> {
    let (body, big_endian) = match raw {
        [0xFF, 0xFE, body @ ..] => (body, false),
        [0xFE, 0xFF, body @ ..] => (body, true),
        _ => return None,
    };
    let (pairs, rest) = body.as_chunks::<2>();
    if !rest.is_empty() {
        return Some(None);
    }
    let units = pairs.iter().map(|&pair| match big_endian {
        true => u16::from_be_bytes(pair),
        false => u16::from_le_bytes(pair),
    });
    let units: Vec<u16> = units.collect();
    Some(String::from_utf16(&units).ok())
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
    fn one_windows_line_does_not_break_the_utf8_lines() {
        let mut raw = "Información: año\n".as_bytes().to_vec();
        raw.extend_from_slice(b"Compilaci\xf3n\n");
        raw.extend_from_slice("Fin: ñandú\n".as_bytes());
        // Una misma línea con UTF-8 y un byte cp1252 suelto (el caso de mixed.md).
        raw.extend_from_slice(b"Informaci\xc3\xb3n y caf\xe9\n");
        assert_eq!(
            decode_text(&raw, Path::new("x.md")).unwrap(),
            "Información: año\nCompilación\nFin: ñandú\nInformación y café\n"
        );
    }

    #[test]
    fn utf16_with_bom_is_decoded_in_both_byte_orders() {
        let units: Vec<u16> = "Hola ñ\n".encode_utf16().collect();
        let little: Vec<u8> =
            [0xFF, 0xFE].into_iter().chain(units.iter().flat_map(|u| u.to_le_bytes())).collect();
        let big: Vec<u8> =
            [0xFE, 0xFF].into_iter().chain(units.iter().flat_map(|u| u.to_be_bytes())).collect();
        assert_eq!(decode_text(&little, Path::new("x.md")).unwrap(), "Hola ñ\n");
        assert_eq!(decode_text(&big, Path::new("x.md")).unwrap(), "Hola ñ\n");
    }

    #[test]
    fn broken_utf16_is_an_error_not_garbage() {
        // Longitud impar y un sustituto suelto: ninguno es texto válido.
        assert!(decode_text(&[0xFF, 0xFE, 0x41], Path::new("x.md")).is_err());
        assert!(decode_text(&[0xFF, 0xFE, 0x00, 0xD8], Path::new("x.md")).is_err());
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
