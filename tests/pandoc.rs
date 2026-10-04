//! La sintaxis extendida de Markdown tiene que llegar a LaTeX, no perderse.
//! Se saltan solas si Pandoc no está instalado.

mod common;

use common::{has_tool, project, write};
use investigacion::i18n::Lang;
use investigacion::pandoc::pandoc_to_latex;

fn convert_with_warnings(markdown: &str) -> Option<(String, Vec<String>)> {
    convert_in(markdown, Lang::Es)
}

fn convert_in(markdown: &str, doc_lang: Lang) -> Option<(String, Vec<String>)> {
    convert_with(markdown, doc_lang, false)
}

fn convert_with(markdown: &str, doc_lang: Lang, two_column: bool) -> Option<(String, Vec<String>)> {
    if !has_tool("pandoc") {
        eprintln!("skipped: pandoc is not installed");
        return None;
    }
    let directory = tempfile::tempdir().unwrap();
    let source = write(directory.path(), "trabajo.md", markdown);
    let mut warnings = Vec::new();
    let latex = pandoc_to_latex(&project(), &source, markdown, false, doc_lang, two_column, &mut |w| {
        warnings.push(w)
    })
    .unwrap();
    Some((latex, warnings))
}

fn convert(markdown: &str) -> Option<String> {
    convert_with_warnings(markdown).map(|(latex, _)| latex)
}

#[test]
fn highlight_strikethrough_and_scripts() {
    let Some(latex) = convert("==resaltado== ~~tachado~~ H~2~O X^2^\n") else { return };
    for needle in [r"\hl{resaltado}", r"\st{tachado}", r"\textsubscript{2}", r"\textsuperscript{2}"] {
        assert!(latex.contains(needle), "{needle}");
    }
}

#[test]
fn emoji_and_bare_urls() {
    let Some(latex) = convert("Despegue :rocket: Ver https://example.com\n") else { return };
    assert!(latex.contains('🚀'));
    assert!(latex.contains(r"\url{https://example.com}"));
}

#[test]
fn inline_html_is_translated_instead_of_discarded() {
    let Some(latex) =
        convert("Uno<br>dos <mark>marca</mark> <u>subrayado</u> <sub>b</sub> <del>viejo</del>\n")
    else {
        return;
    };
    for needle in ["\\\\", r"\hl{marca}", r"\ul{subrayado}", r"\textsubscript{b}", r"\st{viejo}"] {
        assert!(latex.contains(needle), "{needle}");
    }
    let Some(code) = convert("```html\n<mark>literal</mark>\n```\n") else { return };
    assert!(!code.contains(r"\hl{"));
}

#[test]
fn boxes_and_references() {
    let Some(latex) = convert("::: nota\nOjo con esto.\n:::\n") else { return };
    assert!(latex.contains(r"\begin{CajaMarcada}{Nota}") && latex.contains(r"\end{CajaMarcada}"));
    let Some(titled) = convert("::: {.aviso title=\"Antes de entregar\"}\nRevisa.\n:::\n") else { return };
    assert!(titled.contains(r"\begin{CajaMarcada}{Antes de entregar}"));
    let Some(references) = convert("# Referencias\n\n- Autor, A. (2020). *Titulo*. Editorial.\n") else {
        return;
    };
    assert!(references.contains(r"\begin{ReferenceList}") && references.contains(r"\end{ReferenceList}"));
    assert!(!references.contains(r"\begin{itemize}"));
}

#[test]
fn images_that_cannot_be_loaded_do_not_break_the_document() {
    let Some((latex, warnings)) = convert_with_warnings("![Un logo](https://localhost:1/no-existe.png)\n")
    else {
        return;
    };
    assert!(!latex.contains("localhost:1"));
    assert!(latex.contains(r"\emph{Un logo}"));
    assert!(warnings.iter().any(|w| w.contains("no-existe.png")));
    assert!(!latex.contains(r"\caption"));

    let Some((latex, warnings)) = convert_with_warnings("![Un diagrama](no-esta.png)\n") else { return };
    assert!(latex.contains(r"\emph{Un diagrama}"));
    assert!(warnings.iter().any(|w| w.contains("no-esta.png")));
}

#[test]
fn dot_blocks_become_diagrams_or_stay_as_code() {
    let markdown = "```{.dot caption=\"Un arbol\"}\ndigraph { a -> b; }\n```\n";
    let Some((latex, warnings)) = convert_with_warnings(markdown) else { return };
    if has_tool("dot") {
        assert!(latex.contains(r"\includegraphics") && latex.contains("diagrams/"));
        assert!(!latex.contains("digraph"));
    } else {
        assert!(latex.contains("digraph"));
        assert!(warnings.iter().any(|w| w.contains("Graphviz")));
    }
}

#[test]
fn pgfplot_and_tikz_blocks() {
    let markdown = "```{.pgfplot caption=\"Horas\"}\n\\begin{axis}[ybar]\n  \\addplot coordinates {(A,1)};\n\\end{axis}\n```\n";
    let Some(latex) = convert(markdown) else { return };
    assert!(latex.contains(r"\begin{tikzpicture}") && latex.contains(r"\begin{axis}[ybar]"));
    assert!(latex.contains(r"\caption{Horas}") && !latex.contains(r"\begin{Shaded}"));
    let Some(tikz) =
        convert("```tikz\n\\begin{tikzpicture}\n\\draw (0,0) -- (1,1);\n\\end{tikzpicture}\n```\n")
    else {
        return;
    };
    assert_eq!(tikz.matches(r"\begin{tikzpicture}").count(), 1);
}

#[test]
fn task_and_definition_lists() {
    let Some(latex) = convert("- [x] hecha\n- [ ] pendiente\n\nTermino\n: Definicion\n") else { return };
    for needle in [r"\boxtimes", r"\square", r"\begin{description}"] {
        assert!(latex.contains(needle), "{needle}");
    }
}

#[test]
fn boxes_and_references_follow_the_document_language() {
    let Some((latex, _)) =
        convert_in("::: note\nCareful.\n:::\n\n# References\n\n- Smith, J. (2020). *Title*.\n", Lang::En)
    else {
        return;
    };
    assert!(latex.contains(r"\begin{CajaMarcada}{Note}"), "{latex}");
    assert!(latex.contains(r"\begin{ReferenceList}"));
    let Some((spanish, _)) = convert_in("::: note\nOjo.\n:::\n", Lang::Es) else { return };
    assert!(spanish.contains(r"\begin{CajaMarcada}{Nota}"));
}

#[test]
fn tables_become_tabularx_in_two_column_formats() {
    let table = "| Modelo | Riesgo |\n|:---|---:|\n| **Cascada** | Bajo |\n\n: Modelos de desarrollo\n";
    let Some((one, _)) = convert_with(table, Lang::Es, false) else { return };
    assert!(one.contains(r"\begin{longtable}"));
    let Some((two, _)) = convert_with(table, Lang::Es, true) else { return };
    assert!(!two.contains("longtable"), "{two}");
    assert!(two.contains(r"\begin{tabularx}{\columnwidth}"));
    assert!(two.contains(r"\caption{Modelos de desarrollo}"));
    assert!(two.contains(r"\textbf{Cascada} & Bajo \\"));
    assert!(two.contains(r">{\raggedleft\arraybackslash}X"));
}
