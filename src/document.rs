//! Datos de la portada y utilidades de nombres y fechas.

use std::collections::BTreeMap;

use chrono::{Datelike, NaiveDate};
use regex::Regex;
use std::sync::LazyLock;

use crate::i18n::Lang;
use crate::markdown::strip_accents;

const MONTHS_ES: [&str; 12] = [
    "enero",
    "febrero",
    "marzo",
    "abril",
    "mayo",
    "junio",
    "julio",
    "agosto",
    "septiembre",
    "octubre",
    "noviembre",
    "diciembre",
];

/// Valores que se insertan en la portada. `student`, `members`, `teacher` y
/// `group` son opcionales: la plantilla omite su línea cuando van vacíos.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct DocumentData {
    pub university: String,
    pub faculty: String,
    pub student: String,
    pub title: String,
    pub course: String,
    pub teacher: String,
    pub date: String,
    pub members: Vec<String>,
    pub group: String,
    /// Campos propios del diseño (`%%SALON%%` → `SALON`), ya en mayúsculas.
    pub fields: BTreeMap<String, String>,
}

/// Separa los nombres del equipo escritos en un solo argumento (comas o punto
/// y coma) y descarta los vacíos: "Ana,,Luis," son dos nombres.
pub fn parse_members(value: &str) -> Vec<String> {
    value.split([',', ';']).map(str::trim).filter(|name| !name.is_empty()).map(str::to_owned).collect()
}

const MONTHS_EN: [&str; 12] = [
    "January",
    "February",
    "March",
    "April",
    "May",
    "June",
    "July",
    "August",
    "September",
    "October",
    "November",
    "December",
];

/// Fecha de entrega en formato largo, en el idioma del documento.
pub fn format_delivery_date(date: NaiveDate, lang: Lang) -> String {
    let month = date.month0() as usize;
    match lang {
        // Como lo escriben la RAE y APA en español: «23 de agosto de 2026».
        Lang::Es => format!("{} de {} de {}", date.day(), MONTHS_ES[month], date.year()),
        Lang::En => format!("{} {}, {}", MONTHS_EN[month], date.day(), date.year()),
    }
}

/// Fecha local de hoy, ya formateada.
pub fn today(lang: Lang) -> String {
    format_delivery_date(chrono::Local::now().date_naive(), lang)
}

/// Nombre de archivo estable y seguro: sin acentos, minúsculas y guiones.
pub fn slugify(value: &str) -> String {
    static NOT_ALNUM: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"[^a-zA-Z0-9]+").unwrap());
    let plain = strip_accents(value);
    let slug = NOT_ALNUM.replace_all(&plain, "-").trim_matches('-').to_lowercase();
    if slug.is_empty() { "investigacion".to_owned() } else { slug }
}

/// Nombres que Windows reserva: no sirven como archivo, tenga o no extensión.
const WINDOWS_RESERVED: [&str; 22] = [
    "con", "prn", "aux", "nul", "com1", "com2", "com3", "com4", "com5", "com6", "com7", "com8", "com9",
    "lpt1", "lpt2", "lpt3", "lpt4", "lpt5", "lpt6", "lpt7", "lpt8", "lpt9",
];

/// Nombre de archivo seguro en Windows y Linux: un slug, sin nombres reservados
/// y con tope de largo (el temporal de la escritura añade caracteres). Se corta
/// en un guion para no dejar una palabra a medias.
pub fn safe_file_stem(value: &str) -> String {
    const MAX_LEN: usize = 100;
    // El slug es ASCII, así que cortar por bytes es seguro.
    let mut stem = slugify(value);
    if stem.len() > MAX_LEN {
        let head = &stem[..MAX_LEN];
        let cut = head.rfind('-').filter(|&i| i > 0).unwrap_or(MAX_LEN);
        stem = head[..cut].trim_end_matches('-').to_owned();
    }
    if WINDOWS_RESERVED.contains(&stem.as_str()) { format!("investigacion-{stem}") } else { stem }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn delivery_date_is_in_spanish() {
        let date = NaiveDate::from_ymd_opt(2026, 8, 23).unwrap();
        assert_eq!(format_delivery_date(date, Lang::Es), "23 de agosto de 2026");
        assert_eq!(format_delivery_date(date, Lang::En), "August 23, 2026");
    }

    #[test]
    fn slugify_removes_accents() {
        assert_eq!(slugify("Álgebra: teoría y práctica"), "algebra-teoria-y-practica");
        assert_eq!(slugify("¿?"), "investigacion");
    }

    #[test]
    fn safe_file_stem_avoids_windows_reserved_names_and_long_names() {
        assert_eq!(safe_file_stem("con"), "investigacion-con");
        assert_eq!(safe_file_stem("AUX"), "investigacion-aux");
        assert_eq!(safe_file_stem("com1"), "investigacion-com1");
        assert_eq!(safe_file_stem("conferencia"), "conferencia");
        let long = safe_file_stem(&"palabra ".repeat(60));
        assert!(long.len() <= 100 && long.ends_with("palabra") && !long.ends_with('-'));
    }

    #[test]
    fn members_accept_mixed_separators_and_drop_empty_names() {
        assert_eq!(parse_members("Ana Ruiz, Luis Paz; Sofia Vela"), ["Ana Ruiz", "Luis Paz", "Sofia Vela"]);
        assert_eq!(parse_members("  Ana ,, Luis ,"), ["Ana", "Luis"]);
        assert!(parse_members("").is_empty());
        assert!(parse_members("   ,  ;  ").is_empty());
    }
}
