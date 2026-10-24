//! El límite de tiempo del conversor de imágenes, de punta a punta. Va en su
//! propio binario de pruebas porque baja el límite con una variable de entorno.

mod common;

use common::{has_tool, isolated_project, write};
use investigacion::i18n::Lang;
use investigacion::pandoc::pandoc_to_latex;

// Un SVG que tarda minutos en dibujarse (muchos desenfoques sobre ~40 Mpx)
// pasa el límite: aviso que nombra la imagen, texto alternativo, y Pandoc sigue.
#[test]
fn a_slow_image_is_a_warning_and_pandoc_goes_on() {
    if !has_tool("pandoc") {
        return;
    }
    // SAFETY: es la única prueba de este binario; nadie más lee el entorno a la vez.
    unsafe { std::env::set_var("INVESTIGACION_CONVERTER_TIMEOUT_MS", "1000") };
    let directory = tempfile::tempdir().unwrap();
    let filters: String = (0..20)
        .map(|i| format!("<filter id=\"f{i}\"><feGaussianBlur stdDeviation=\"80\"/><feMorphology radius=\"40\"/></filter>"))
        .collect();
    let rects: String = (0..20)
        .map(|i| format!("<rect x=\"{0}\" y=\"{0}\" width=\"3000\" height=\"3000\" fill=\"red\" filter=\"url(#f{i})\"/>", i * 10))
        .collect();
    let svg = format!(
        "<svg xmlns=\"http://www.w3.org/2000/svg\" width=\"3160\" height=\"3160\"><defs>{filters}</defs>{rects}</svg>"
    );
    write(directory.path(), "pesado.svg", &svg);
    let markdown = "# Introducción\n\n![Pesado](pesado.svg)\n";
    let source = write(directory.path(), "trabajo.md", markdown);
    let converter = std::path::Path::new(env!("CARGO_BIN_EXE_investigacion"));
    investigacion::i18n::set(Lang::Es);
    let started = std::time::Instant::now();
    let mut warnings = Vec::new();
    let latex = pandoc_to_latex(
        &isolated_project(directory.path()),
        &source,
        markdown,
        false,
        Lang::Es,
        Some(converter),
        &mut |w| warnings.push(w),
    )
    .unwrap();
    assert!(started.elapsed() < std::time::Duration::from_secs(20), "{:?}", started.elapsed());
    assert!(
        warnings
            .iter()
            .any(|w| w.contains("pesado.svg") && w.contains("tardó demasiado") && w.contains("PNG")),
        "{warnings:?}"
    );
    assert!(!latex.contains("includegraphics") && latex.contains("Pesado"), "{latex}");
}
