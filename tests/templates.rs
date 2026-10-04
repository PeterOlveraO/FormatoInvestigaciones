//! El contrato entre la salida de Pandoc y las plantillas: lo que Pandoc y los
//! filtros dan por definido tiene que estar en el preámbulo común.

mod common;

use common::{data, full_template};
use investigacion::latex::render_template;

#[test]
fn every_marker_of_the_apa_template_is_replaced() {
    let mut values = data("T");
    values.student.clear();
    values.group = "7-A".into();
    let rendered = render_template(&full_template(), r"\section{Texto}", &values).unwrap();
    assert!(!rendered.contains("%%UNIVERSIDAD%%"));
    assert!(!rendered.contains("%%CONTENIDO_MARKDOWN%%"));
    assert!(rendered.contains(r"\newcommand{\ListaIntegrantes}{}"));
    assert!(rendered.contains(r"\newcommand{\GrupoMateria}{7-A}"));
}

#[test]
fn unicode_symbols_ascii_art_and_emoji_are_covered() {
    let template = full_template();
    for needle in [
        r"\newunicodechar{≠}{\ensuremath{\neq}}",
        r"\def\UTFviii@undefined@err",
        "pmboxdraw",
        "twemojis",
        "►",
        r"\texttwemoji",
    ] {
        assert!(template.contains(needle), "{needle}");
    }
}

#[test]
fn the_fragile_commands_of_soul_are_replaced() {
    let template = full_template();
    for needle in [
        r"\DeclareRobustCommand{\st}[1]{\sout{#1}}",
        r"\DeclareRobustCommand{\ul}[1]{\uline{#1}}",
        r"\IfFileExists{ulem.sty}{\RequirePackage[normalem]{ulem}}",
        r"\AtBeginEnvironment{longtable}{\let\hl\ResaltadoEnTabla}",
    ] {
        assert!(template.contains(needle), "{needle}");
    }
}

#[test]
fn what_the_filters_produce_is_defined() {
    let template = full_template();
    for needle in [
        r"\newenvironment{CajaMarcada}[1]",
        r"\newenvironment{ReferenceList}",
        r"\RequirePackage{pgfplots}",
        r"/pgfplots/bar cycle list/.style",
        r"\floatsetup[figure]{capposition=top}",
        r"\setlength\LTleft{\fill}",
        r"\@ifundefined{floatsetup}{\def\fps@figure{htbp}}",
        r"\newenvironment{Shaded}{\singlespacing\small}{}",
        r"\AtBeginEnvironment{verbatim}{\singlespacing\small}",
    ] {
        assert!(template.contains(needle), "{needle}");
    }
}

#[test]
fn the_cover_order_is_fixed() {
    let template = full_template();
    let positions: Vec<usize> = [
        r"\textbf{ALUMNO:}",
        r"\textbf{MATERIA:}",
        r"\textbf{DOCENTE:}",
        r"\textbf{SEMESTRE:}",
        r"\textbf{GRUPO:}",
    ]
    .iter()
    .map(|label| template.find(label).unwrap())
    .collect();
    let mut sorted = positions.clone();
    sorted.sort();
    assert_eq!(positions, sorted);
    assert!(template.find(r"\textbf{INTEGRANTES:}").unwrap() < template.find(r"\textbf{MATERIA:}").unwrap());
    assert!(!template.contains("TRABAJO ACADÉMICO"));
    assert!(template.contains(r"\ifdefempty{\NombreMaestro}{}{\textbf{DOCENTE:}"));
}

#[test]
fn the_template_looks_for_the_generic_logo_names() {
    let template = full_template();
    for name in investigacion::project::LOGO_FILES {
        assert!(template.contains(&format!(r"\IfFileExists{{{name}}}")));
    }
}
