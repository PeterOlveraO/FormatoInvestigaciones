//! Generación real de PDF. Se saltan solas si faltan Pandoc o pdflatex.

mod common;

use common::{data, has_tool, isolated_project, write};
use investigacion::generate::{GenerateOptions, generate_pdf};

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
