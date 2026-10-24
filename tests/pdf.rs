//! Generación real de PDF. Se saltan solas si faltan Pandoc o pdflatex.

mod common;

use common::{data, has_tool, isolated_project, write};
use investigacion::generate::{GenerateOptions, generate_pdf};
use investigacion::i18n::Lang;
use investigacion::template::list_designs;

const RICH_MARKDOWN: &str = r#"# Introducción

Texto con `codigo`, ~~tachado~~ y una fórmula $E = mc^2$.

$$\int_0^1 x^2\,dx = \frac{1}{3}$$

# Desarrollo

| Tipo | Total | Unitario |
|---|---|---|
| Fijo | Constante | Decrece |
| Variable | Proporcional | Constante |

```python
def f(x):
    return x
```

Una nota al pie[^1] y una cita.

[^1]: Contenido de la nota.

> Cita en bloque.

```tikz
\draw[->] (0,0) -- (2,0);
\draw[<->] (0,1) -- (2,1);
```

# Conclusión

Cierre con símbolos 100% & seguros.

# Referencias

Autor, A. (2026). *Título*. Editorial.
"#;

// Cada elemento fallaba antes: el tachado dentro de una tabla colgaba a
// pdflatex, el diagrama de caja salía como [?] y el HTML no llegaba al PDF.
const EXTENDED_MARKDOWN: &str = r#"# Introducción

Texto ==resaltado==, ~~tachado~~, H~2~O, X^2^, <mark>marcado</mark>,
<u>subrayado</u>, <kbd>Ctrl</kbd> y un salto<br>de linea.

Emoji pegado 🚀 y por código :joy:. URL suelta https://example.com

# Desarrollo

| Elemento | Estado |
|---|---|
| ~~tachado~~ | ==resaltado== |

```
  Inicio
      └──► Paso
              └──► Fin
        ┌─────┬─────┐
        │     │     │
        └─────┴─────┘
        ╭─────╮
        ╰─────╯
```

- [x] Tarea hecha
- [ ] Tarea pendiente

Término
: Definición del término.

# Conclusión

Cierre.

# Referencias

Autor, A. (2026). *Título*. Editorial.
"#;

fn build(markdown: &str, title: &str) -> Option<(std::path::PathBuf, Vec<String>, tempfile::TempDir)> {
    if !(has_tool("pandoc") && has_tool("pdflatex")) {
        eprintln!("skipped: pandoc and pdflatex are required");
        return None;
    }
    let directory = tempfile::tempdir().unwrap();
    let source = write(directory.path(), "trabajo.md", markdown);
    let mut warnings = Vec::new();
    let pdf = generate_pdf(
        &isolated_project(directory.path()),
        &source,
        &directory.path().join("salida"),
        &data(title),
        &GenerateOptions::default(),
        &mut |w| warnings.push(w),
    )
    .unwrap();
    Some((pdf, warnings, directory))
}

#[test]
fn a_document_with_tables_and_code_compiles() {
    let Some((pdf, _, _dir)) = build(RICH_MARKDOWN, "Prueba") else { return };
    assert!(pdf.is_file() && std::fs::metadata(&pdf).unwrap().len() > 0);
}

#[test]
fn the_extended_syntax_compiles_without_lost_symbols() {
    let Some((pdf, warnings, _dir)) = build(EXTENDED_MARKDOWN, "Extendida") else { return };
    assert!(pdf.is_file());
    assert!(warnings.is_empty(), "{warnings:?}");
}

#[test]
fn backslashes_in_the_text_do_not_break_the_build() {
    let Some((pdf, _, _dir)) = build("# Introducción\n\nRuta C:\\Users\\alumno\n", "Rutas") else { return };
    assert!(pdf.is_file());
}

// babel-spanish redefinía \% y fallaba tras un espacio de math (`\,`).
#[test]
fn percent_after_a_math_spacing_compiles_in_spanish() {
    let markdown = "# Introducción\n\nCrece $28.9\\,\\%$, $x\\;\\%$ y $28\\%$ en texto 28.9\\%.\n";
    let Some((pdf, _, _dir)) = build(markdown, "Porcentaje") else { return };
    assert!(pdf.is_file());
}

#[test]
fn images_with_dot_relative_or_percent_encoded_paths_are_found() {
    if !(has_tool("pandoc") && has_tool("pdflatex")) {
        return;
    }
    // pdflatex corre en el temporal: `./` y `../` no se resuelven por TEXINPUTS.
    let directory = tempfile::tempdir().unwrap();
    let image = common::root().join("examples/sample-image.png");
    let paper = directory.path().join("Mis trabajos/Física");
    std::fs::create_dir_all(paper.join("img")).unwrap();
    std::fs::create_dir_all(directory.path().join("Mis trabajos/compartidas")).unwrap();
    std::fs::copy(&image, paper.join("img/a.png")).unwrap();
    std::fs::copy(&image, directory.path().join("Mis trabajos/compartidas/b.png")).unwrap();
    // Y un espacio escrito como %20, como lo guardan muchos editores.
    std::fs::copy(&image, paper.join("img/mi foto.png")).unwrap();
    let markdown = "# Introducción\n\n![Uno](./img/a.png)\n\n![Dos](../compartidas/b.png)\n\n![Tres](img/mi%20foto.png)\n";
    let source = write(&paper, "trabajo.md", markdown);
    let mut warnings = Vec::new();
    let pdf = generate_pdf(
        &isolated_project(directory.path()),
        &source,
        &directory.path().join("salida"),
        &data("Rutas"),
        &GenerateOptions::default(),
        &mut |w| warnings.push(w),
    )
    .unwrap();
    assert!(pdf.is_file());
    assert!(warnings.is_empty(), "{warnings:?}");
}

#[test]
fn every_design_compiles_with_and_without_optional_fields() {
    if !(has_tool("pandoc") && has_tool("pdflatex")) {
        return;
    }
    let directory = tempfile::tempdir().unwrap();
    let project = isolated_project(directory.path());
    let source = write(directory.path(), "trabajo.md", RICH_MARKDOWN);
    let designs = list_designs(&project);
    assert!(!designs.is_empty(), "{designs:?}");

    let mut full = data("Completo");
    full.members = vec!["Ana Ruiz".into(), "Luis Paz".into()];
    full.group = "7-A".into();
    let mut bare = data("Minimo");
    bare.student.clear();
    bare.teacher.clear();

    for design in designs {
        for (index, values) in [&full, &bare].into_iter().enumerate() {
            let options = GenerateOptions {
                design: Some(design.clone()),
                file_name: Some(format!("{design}-{index}")),
                ..Default::default()
            };
            let pdf = generate_pdf(
                &project,
                &source,
                &directory.path().join("salida"),
                values,
                &options,
                &mut |_| {},
            )
            .unwrap_or_else(|e| panic!("{design}: {e}"));
            assert!(pdf.is_file());
            // El estado de LaTeX se guarda aunque el diseño no tenga índice.
            let prefix = format!("{design}-{index}-");
            let saved = std::fs::read_dir(project.cache_dir().join("latex")).unwrap().flatten().any(|e| {
                e.file_name().to_string_lossy().starts_with(&prefix) && e.path().join("trabajo.toc").is_file()
            });
            assert!(saved, "{design}: the latex state was not saved");
        }
    }
}

#[test]
fn a_document_in_english_compiles() {
    if !(has_tool("pandoc") && has_tool("pdflatex")) {
        return;
    }
    let directory = tempfile::tempdir().unwrap();
    let project = isolated_project(directory.path());
    let source = write(
        directory.path(),
        "paper.md",
        "# Introduction\n\n::: note\nText.\n:::\n\n# References\n\nSmith (2020).\n",
    );
    let options = GenerateOptions { doc_lang: Some(Lang::En), ..Default::default() };
    let pdf =
        generate_pdf(&project, &source, &directory.path().join("out"), &data("Paper"), &options, &mut |_| {})
            .unwrap();
    assert!(pdf.is_file());
}

// Las etiquetas de la portada siguen el idioma del documento.
#[test]
fn cover_labels_follow_the_document_language() {
    if !(has_tool("pandoc") && has_tool("pdflatex") && has_tool("pdftotext")) {
        return;
    }
    let directory = tempfile::tempdir().unwrap();
    let project = isolated_project(directory.path());
    let source =
        write(directory.path(), "paper.md", "# Introduction\n\nText.\n\n# References\n\nSmith (2020).\n");
    let options = GenerateOptions { doc_lang: Some(Lang::En), ..Default::default() };
    let pdf =
        generate_pdf(&project, &source, &directory.path().join("out"), &data("Paper"), &options, &mut |_| {})
            .unwrap();
    let text = std::process::Command::new("pdftotext").arg(&pdf).arg("-").output().unwrap();
    let text = String::from_utf8_lossy(&text.stdout);
    assert!(text.contains("COURSE:") && !text.contains("MATERIA:"), "{text}");
}

#[test]
fn a_minimal_design_with_a_custom_field_compiles() {
    if !(has_tool("pandoc") && has_tool("pdflatex")) {
        return;
    }
    let directory = tempfile::tempdir().unwrap();
    let project = isolated_project(directory.path());
    // Solo el contrato: clase, base, formato, final y el contenido.
    let design = directory.path().join("my-templates/designs/minimal");
    std::fs::create_dir_all(&design).unwrap();
    std::fs::write(
        design.join("template.ltx"),
        "\\documentclass[%%CLASS_OPTIONS%%]{article}\n\\usepackage{investigacion-base}\n%%FORMAT%%\n\\usepackage{investigacion-final}\n\\begin{document}\nSalon: %%SALON%%\n\n%%CONTENIDO_MARKDOWN%%\n\\end{document}\n",
    )
    .unwrap();
    let source = write(directory.path(), "trabajo.md", RICH_MARKDOWN);
    let mut values = data("Minimo");
    values.fields.insert("SALON".into(), "B-204 & Lab".into());
    let options = GenerateOptions { design: Some("minimal".into()), ..Default::default() };
    let pdf = generate_pdf(&project, &source, &directory.path().join("out"), &values, &options, &mut |_| {})
        .unwrap();
    assert!(pdf.is_file());
}

#[test]
fn the_built_in_design_passes_the_check() {
    if !(has_tool("pandoc") && has_tool("pdflatex")) {
        return;
    }
    let directory = tempfile::tempdir().unwrap();
    let project = isolated_project(directory.path());
    // El catálogo de ejemplo es lo que compila la revisión.
    let examples = directory.path().join("examples");
    std::fs::create_dir_all(&examples).unwrap();
    for file in ["catalog.md", "sample-image.png"] {
        std::fs::copy(common::root().join("examples").join(file), examples.join(file)).unwrap();
    }
    let report = investigacion::check::check_design(&project, "geometric-cover", &mut |_| {}).unwrap();
    assert!(report.errors.is_empty() && report.warnings.is_empty(), "{report:?}");
    assert_eq!(report.builds.len(), 1);
    assert!(report.passed(), "{report:?}");
}

/// Genera un trabajo con una imagen copiada de `examples/` (o con `content`)
/// usando el binario como conversor; devuelve los avisos y la carpeta.
fn build_with_image(name: &str, content: &[u8]) -> Option<(Vec<String>, tempfile::TempDir)> {
    if !(has_tool("pandoc") && has_tool("pdflatex")) {
        return None;
    }
    let directory = tempfile::tempdir().unwrap();
    std::fs::write(directory.path().join(name), content).unwrap();
    let source = write(directory.path(), "trabajo.md", &format!("# Introducción\n\n![Figura]({name})\n"));
    let options =
        GenerateOptions { converter: Some(env!("CARGO_BIN_EXE_investigacion").into()), ..Default::default() };
    let mut warnings = Vec::new();
    let pdf = generate_pdf(
        &isolated_project(directory.path()),
        &source,
        &directory.path().join("salida"),
        &data("Imagen"),
        &options,
        &mut |w| warnings.push(w),
    )
    .unwrap();
    assert!(pdf.is_file());
    Some((warnings, directory))
}

fn converted_pngs(directory: &tempfile::TempDir) -> usize {
    std::fs::read_dir(directory.path().join("cache/remote"))
        .unwrap()
        .flatten()
        .filter(|e| e.path().extension().is_some_and(|x| x == "png"))
        .count()
}

#[test]
fn an_svg_image_is_converted_to_png() {
    let svg = std::fs::read(common::root().join("examples/sample-vector.svg")).unwrap();
    let Some((warnings, directory)) = build_with_image("figura.svg", &svg) else { return };
    assert!(warnings.is_empty(), "{warnings:?}");
    assert_eq!(converted_pngs(&directory), 1);
}

// Un WEBP con extensión .png (pasa al guardar desde el navegador) rompía pdflatex.
#[test]
fn a_webp_image_is_converted_even_with_a_misleading_extension() {
    let webp = std::fs::read(common::root().join("examples/sample-photo.webp")).unwrap();
    let Some((warnings, directory)) = build_with_image("falsa.png", &webp) else { return };
    assert!(warnings.is_empty(), "{warnings:?}");
    assert_eq!(converted_pngs(&directory), 1);
}

#[test]
fn a_broken_svg_is_a_warning_and_the_pdf_is_still_made() {
    let Some((warnings, _directory)) = build_with_image("rota.svg", b"<svg <<< no es un svg") else { return };
    assert_eq!(warnings.len(), 1, "{warnings:?}");
    assert!(warnings[0].contains("rota.svg"), "{warnings:?}");
}

/// Genera un trabajo sin imágenes con un solo logo, `name`, en su carpeta;
/// devuelve los avisos, el PDF y la carpeta.
fn build_with_logo(name: &str, content: &[u8]) -> Option<(Vec<String>, Vec<u8>, tempfile::TempDir)> {
    if !(has_tool("pandoc") && has_tool("pdflatex")) {
        return None;
    }
    let directory = tempfile::tempdir().unwrap();
    let logos = directory.path().join("mis-logos");
    std::fs::create_dir(&logos).unwrap();
    std::fs::write(logos.join(name), content).unwrap();
    let source = write(directory.path(), "trabajo.md", "# Introducción\n\nTexto.\n");
    let options = GenerateOptions { logos: Some(logos), ..Default::default() };
    let mut warnings = Vec::new();
    let pdf = generate_pdf(
        &isolated_project(directory.path()),
        &source,
        &directory.path().join("salida"),
        &data("Logo"),
        &options,
        &mut |w| warnings.push(w),
    )
    .unwrap();
    Some((warnings, std::fs::read(pdf).unwrap(), directory))
}

fn has_image(pdf: &[u8]) -> bool {
    pdf.windows(13).any(|w| w == b"/Subtype/Imag" || w == b"/Subtype /Ima")
}

#[test]
fn an_svg_logo_is_converted_and_shown() {
    let svg = std::fs::read(common::root().join("examples/sample-vector.svg")).unwrap();
    let Some((warnings, pdf, directory)) = build_with_logo("logo-universidad.svg", &svg) else { return };
    assert!(warnings.is_empty(), "{warnings:?}");
    assert!(has_image(&pdf));
    assert_eq!(converted_pngs(&directory), 1);
}

// Un WEBP guardado como .png: se reconoce por el contenido.
#[test]
fn a_webp_logo_is_converted_even_with_a_misleading_extension() {
    let webp = std::fs::read(common::root().join("examples/sample-photo.webp")).unwrap();
    let Some((warnings, pdf, directory)) = build_with_logo("logo-universidad.png", &webp) else { return };
    assert!(warnings.is_empty(), "{warnings:?}");
    assert!(has_image(&pdf));
    assert_eq!(converted_pngs(&directory), 1);
}

#[test]
fn a_broken_logo_is_a_warning_and_the_cover_goes_without_it() {
    let Some((warnings, pdf, _directory)) = build_with_logo("logo-facultad.svg", b"<svg <<<") else { return };
    assert_eq!(warnings.len(), 1, "{warnings:?}");
    assert!(warnings[0].contains("logo-facultad.svg"), "{warnings:?}");
    assert!(!has_image(&pdf));
}
