//! Comprueba si están Pandoc, pdflatex y Graphviz para avisarlo en el inicio.

use std::process::Command;
use std::sync::mpsc::{self, Receiver};
use std::time::Duration;

use crate::process::{RunError, run_with_timeout};

/// Una herramienta externa y si se encontró.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Tool {
    pub name: &'static str,
    /// Sin ella no se genera el PDF; Graphviz solo hace falta para diagramas.
    pub required: bool,
    pub found: bool,
}

// (nombre visible, programa, argumento que solo imprime la versión, obligatoria)
const TOOLS: [(&str, &str, &str, bool); 3] = [
    ("Pandoc", "pandoc", "--version", true),
    ("pdflatex", "pdflatex", "--version", true),
    ("Graphviz (dot)", "dot", "-V", false),
];

const TIMEOUT: Duration = Duration::from_secs(5);

/// Las comprueba en un hilo aparte: Pandoc tarda hasta un par de segundos en
/// arrancar en frío y el menú no debe esperarlo.
pub fn detect_in_background() -> Receiver<Vec<Tool>> {
    let (sender, receiver) = mpsc::channel();
    // Si el hilo no arranca, el canal se cierra y el inicio no muestra la lista.
    let _ = std::thread::Builder::new().name("tools".into()).spawn(move || {
        let tools = TOOLS
            .iter()
            .map(|&(name, program, version, required)| Tool {
                name,
                required,
                found: is_installed(program, version),
            })
            .collect();
        let _ = sender.send(tools);
    });
    receiver
}

/// Si responde a tiempo o se pasa del límite, existe (MiKTeX puede tardar la
/// primera vez); solo falta si el sistema no encuentra el programa.
pub fn is_installed(program: &str, version_arg: &str) -> bool {
    let mut command = Command::new(program);
    command.arg(version_arg);
    matches!(run_with_timeout(command, None, TIMEOUT), Ok(_) | Err(RunError::Timeout))
}
