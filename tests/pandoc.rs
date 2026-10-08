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
    if !has_tool("pandoc") {
        eprintln!("skipped: pandoc is not installed");
        return None;
    }
    let directory = tempfile::tempdir().unwrap();
    let source = write(directory.path(), "trabajo.md", markdown);
    let mut warnings = Vec::new();
    let latex =
        pandoc_to_latex(&project(), &source, markdown, false, doc_lang, &mut |w| warnings.push(w)).unwrap();
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
fn local_images_in_formats_pdflatex_cannot_typeset_are_replaced() {
    if !has_tool("pandoc") {
        return;
    }
    let directory = tempfile::tempdir().unwrap();
    write(directory.path(), "fig.svg", "<svg/>");
    let markdown = "![Una figura](fig.svg)\n";
    let source = write(directory.path(), "trabajo.md", markdown);
    let mut warnings = Vec::new();
    let latex =
        pandoc_to_latex(&project(), &source, markdown, false, Lang::Es, &mut |w| warnings.push(w)).unwrap();
    assert!(!latex.contains("includesvg"), "{latex}");
    assert!(latex.contains(r"\emph{Una figura}"));
    assert!(warnings.iter().any(|w| w.contains("fig.svg") && w.contains("svg")), "{warnings:?}");
}

#[test]
fn a_web_page_behind_an_image_address_is_not_cached_as_an_image() {
    // Un servidor que contesta HTML a una dirección que termina en `.png`.
    let listener = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
    let port = listener.local_addr().unwrap().port();
    std::thread::spawn(move || {
        for stream in listener.incoming().flatten() {
            use std::io::{Read, Write};
            let mut stream = stream;
            let _ = stream.read(&mut [0; 4096]);
            let body = "<html><body>no soy una imagen</body></html>";
            let _ = write!(
                stream,
                "HTTP/1.1 200 OK\r\nContent-Type: text/html\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}",
                body.len()
            );
        }
    });
    let markdown = format!("![Ejemplo](http://127.0.0.1:{port}/wiki/File:Ejemplo.png)\n");
    let Some((latex, warnings)) = convert_with_warnings(&markdown) else { return };
    assert!(latex.contains(r"\emph{Ejemplo}"), "{latex}");
    assert!(warnings.iter().any(|w| w.contains("File:Ejemplo.png")), "{warnings:?}");
}

#[test]
fn lists_deeper_than_latex_allows_are_moved_up() {
    let markdown = "- 1\n  - 2\n    - 3\n      - 4\n        - 5\n          - 6\n";
    let Some((latex, warnings)) = convert_with_warnings(markdown) else { return };
    let (mut depth, mut deepest) = (0, 0);
    for line in latex.lines() {
        if line.contains(r"\begin{itemize}") {
            depth += 1;
            deepest = deepest.max(depth);
        } else if line.contains(r"\end{itemize}") {
            depth -= 1;
        }
    }
    assert_eq!(deepest, 4, "{latex}");
    assert!(latex.contains("6"));
    assert!(warnings.iter().any(|w| w.contains("niveles")), "{warnings:?}");
}

#[test]
fn dot_blocks_become_diagrams_or_stay_as_code() {
    let markdown = "```{.dot caption=\"Un arbol\"}\ndigraph { a -> b; }\n```\n";
    let Some((latex, warnings)) = convert_with_warnings(markdown) else { return };
    if has_tool("dot") {
        // Solo el nombre del PDF de la caché: pdflatex lo encuentra por TEXINPUTS.
        assert!(latex.contains(r"\includegraphics") && !latex.contains("diagrams/"));
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
    // MLA en español: «Obras citadas» también abre la lista con sangría.
    let Some((cited, _)) = convert_in("# Obras citadas\n\nAutor, A. *Título*.\n", Lang::Es) else { return };
    assert!(cited.contains(r"\begin{ReferenceList}"), "{cited}");
}
