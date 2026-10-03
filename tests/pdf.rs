//! Generación real de PDF. Se saltan solas si faltan Pandoc o pdflatex.

mod common;

use common::{data, has_tool, isolated_project, write};
use investigacion::generate::{GenerateOptions, generate_pdf};
use investigacion::i18n::Lang;
use investigacion::template::{list_designs, list_formats, load_design};

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

#[test]
fn every_design_compiles_with_each_compatible_format_and_without_optional_fields() {
    if !(has_tool("pandoc") && has_tool("pdflatex")) {
        return;
    }
    let directory = tempfile::tempdir().unwrap();
    let project = isolated_project(directory.path());
    let source = write(directory.path(), "trabajo.md", RICH_MARKDOWN);
    let designs = list_designs(&project);
    assert!(designs.len() >= 2, "{designs:?}");

    let mut full = data("Completo");
    full.members = vec!["Ana Ruiz".into(), "Luis Paz".into()];
    full.group = "7-A".into();
    let mut bare = data("Minimo");
    bare.student.clear();
    bare.teacher.clear();

    for design in designs {
        let accepted = load_design(&project, Some(&design)).unwrap();
        for format in list_formats(&project).into_iter().filter(|f| accepted.accepts(f)) {
            for (index, values) in [&full, &bare].into_iter().enumerate() {
                let options = GenerateOptions {
                    design: Some(design.clone()),
                    format: Some(format.clone()),
                    file_name: Some(format!("{design}-{format}-{index}")),
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
                .unwrap_or_else(|e| panic!("{design} + {format}: {e}"));
                assert!(pdf.is_file());
            }
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
