//! Ejecución de Pandoc y pdflatex con límite de tiempo.
//!
//! `std::process` no trae timeout, y soul (por ejemplo) puede dejar a pdflatex
//! dando vueltas para siempre: más vale un error legible que una terminal
//! congelada.

use std::io::{Read, Write};
use std::process::{Command, ExitStatus, Stdio};
use std::thread;
use std::time::{Duration, Instant};

/// Ninguna pasada sobre un trabajo escolar tarda tanto; si lo hace, se colgó.
pub const TOOL_TIMEOUT: Duration = Duration::from_secs(180);

pub struct Output {
    pub status: ExitStatus,
    pub stdout: Vec<u8>,
    pub stderr: Vec<u8>,
}

#[derive(Debug)]
pub enum RunError {
    NotFound,
    Timeout,
    Io(std::io::Error),
}

/// Lanza el comando, le pasa `input` por stdin y recoge stdout/stderr en hilos
/// aparte (si no, un log grande llenaría la tubería y bloquearía al proceso).
pub fn run_with_timeout(
    mut command: Command,
    input: Option<Vec<u8>>,
    timeout: Duration,
) -> Result<Output, RunError> {
    command
        .stdin(if input.is_some() { Stdio::piped() } else { Stdio::null() })
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());
    let mut child = command.spawn().map_err(|error| match error.kind() {
        std::io::ErrorKind::NotFound => RunError::NotFound,
        _ => RunError::Io(error),
    })?;

    let writer = input.zip(child.stdin.take()).map(|(data, mut stdin)| {
        thread::spawn(move || {
            // Si el proceso cierra stdin antes de tiempo no es un error nuestro.
            let _ = stdin.write_all(&data);
        })
    });
    let stdout = child.stdout.take().map(read_in_background);
    let stderr = child.stderr.take().map(read_in_background);

    let start = Instant::now();
    let status = loop {
        if let Some(status) = child.try_wait().map_err(RunError::Io)? {
            break status;
        }
        if start.elapsed() > timeout {
            let _ = child.kill();
            let _ = child.wait();
            return Err(RunError::Timeout);
        }
        thread::sleep(Duration::from_millis(5));
    };

    if let Some(writer) = writer {
        let _ = writer.join();
    }
    let collect =
        |handle: Option<thread::JoinHandle<Vec<u8>>>| handle.and_then(|h| h.join().ok()).unwrap_or_default();
    Ok(Output { status, stdout: collect(stdout), stderr: collect(stderr) })
}

fn read_in_background(mut pipe: impl Read + Send + 'static) -> thread::JoinHandle<Vec<u8>> {
    thread::spawn(move || {
        let mut buffer = Vec::new();
        let _ = pipe.read_to_end(&mut buffer);
        buffer
    })
}

/// Con `INVESTIGACION_TIMING=1` imprime cuánto tardó cada herramienta; sirve
/// para medir dónde se va el tiempo sin instalar un perfilador.
pub fn report_timing(label: &str, started: Instant) {
    if std::env::var_os("INVESTIGACION_TIMING").is_some_and(|v| v == "1") {
        eprintln!("[timing] {label}: {:.2} s", started.elapsed().as_secs_f64());
    }
}
