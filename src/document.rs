//! Datos de la portada y utilidades de nombres y fechas.

use chrono::{Datelike, NaiveDate};
use regex::Regex;
use std::sync::LazyLock;

use crate::markdown::strip_accents;

const MONTHS: [&str; 12] = [
    "Enero",
    "Febrero",
    "Marzo",
    "Abril",
    "Mayo",
    "Junio",
    "Julio",
    "Agosto",
    "Septiembre",
    "Octubre",
    "Noviembre",
    "Diciembre",
];

/// Valores que se insertan en la portada. `student`, `members`, `teacher` y
/// `group` son opcionales: la plantilla omite su línea cuando van vacíos.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct DocumentData {
    pub university: String,
    pub faculty: String,
    pub student: String,
    pub semester: String,
    pub title: String,
    pub course: String,
    pub teacher: String,
    pub date: String,
    pub members: Vec<String>,
    pub group: String,
}

/// Separa los nombres del equipo escritos en un solo argumento (comas o punto
/// y coma) y descarta los vacíos: "Ana,,Luis," son dos nombres.
pub fn parse_members(value: &str) -> Vec<String> {
    value.split([',', ';']).map(str::trim).filter(|name| !name.is_empty()).map(str::to_owned).collect()
}

/// Fecha de entrega en formato largo y en español (va en la portada).
pub fn format_delivery_date(date: NaiveDate) -> String {
    format!("{} {}, {}", MONTHS[date.month0() as usize], date.day(), date.year())
}

/// Fecha local de hoy, ya formateada.
pub fn today() -> String {
    format_delivery_date(chrono::Local::now().date_naive())
}

/// Nombre de archivo estable y seguro: sin acentos, minúsculas y guiones.
pub fn slugify(value: &str) -> String {
    static NOT_ALNUM: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"[^a-zA-Z0-9]+").unwrap());
    let plain = strip_accents(value);
    let slug = NOT_ALNUM.replace_all(&plain, "-").trim_matches('-').to_lowercase();
    if slug.is_empty() { "investigacion".to_owned() } else { slug }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn delivery_date_is_in_spanish() {
        let date = NaiveDate::from_ymd_opt(2026, 8, 23).unwrap();
        assert_eq!(format_delivery_date(date), "Agosto 23, 2026");
    }

    #[test]
    fn slugify_removes_accents() {
        assert_eq!(slugify("Álgebra: teoría y práctica"), "algebra-teoria-y-practica");
        assert_eq!(slugify("¿?"), "investigacion");
    }

    #[test]
    fn members_accept_mixed_separators_and_drop_empty_names() {
        assert_eq!(parse_members("Ana Ruiz, Luis Paz; Sofia Vela"), ["Ana Ruiz", "Luis Paz", "Sofia Vela"]);
        assert_eq!(parse_members("  Ana ,, Luis ,"), ["Ana", "Luis"]);
        assert!(parse_members("").is_empty());
        assert!(parse_members("   ,  ;  ").is_empty());
    }
}
