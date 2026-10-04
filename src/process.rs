//! Ejecución de Pandoc y pdflatex con límite de tiempo.
//!
//! `std::process` no trae timeout, y soul (por ejemplo) puede dejar a pdflatex
//! dando vueltas para siempre: más vale un error legible que una terminal
//! congelada.

use std::io::{Read, Write};
use std::process::{Command, ExitStatus, Stdio};
use std::thread;
use std::time::{Duration, Instant};

use crate::logging::{self, Level};

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

/// La orden completa para el registro: programa, argumentos, carpeta y las
/// variables que se le añaden.
fn describe_command(command: &Command, input: Option<&[u8]>) -> String {
    let mut text = command.get_program().to_string_lossy().into_owned();
    for arg in command.get_args() {
        text.push(' ');
        text.push_str(&arg.to_string_lossy());
    }
    if let Some(dir) = command.get_current_dir() {
        text.push_str(&format!("\ncwd: {}", dir.display()));
    }
    for (key, value) in command.get_envs() {
        let value = value.map(|v| v.to_string_lossy().into_owned()).unwrap_or_default();
        text.push_str(&format!("\nenv: {}={value}", key.to_string_lossy()));
    }
    if let Some(input) = input {
        text.push_str(&format!("\nstdin: {} bytes", input.len()));
    }
    text
}

/// Lanza el comando, le pasa `input` por stdin y recoge stdout/stderr en hilos
/// aparte (si no, un log grande llenaría la tubería y bloquearía al proceso).
/// Registra la orden (debug), el resultado y lo que tardó.
pub fn run_with_timeout(
    command: Command,
    input: Option<Vec<u8>>,
    timeout: Duration,
) -> Result<Output, RunError> {
    let program = command.get_program().to_string_lossy().into_owned();
    if logging::enabled(Level::Debug) {
        logging::debug(format_args!("run: {}", describe_command(&command, input.as_deref())));
    }
    let started = Instant::now();
    let result = run(command, input, timeout);
    let seconds = started.elapsed().as_secs_f64();
    match &result {
        Ok(output) if output.status.success() => {
            logging::info(format_args!("{program}: {} in {seconds:.2} s", output.status));
        }
        Ok(output) => logging::warn(format_args!(
            "{program}: {} in {seconds:.2} s ({} bytes of stdout, {} of stderr)",
            output.status,
            output.stdout.len(),
            output.stderr.len()
        )),
        Err(RunError::NotFound) => logging::error(format_args!("{program}: not found on the PATH")),
        Err(RunError::Timeout) => {
            logging::error(format_args!("{program}: stopped after the {} s time limit", timeout.as_secs()))
        }
        Err(RunError::Io(error)) => logging::error(format_args!("{program}: I/O error: {error}")),
    }
    result
}

fn run(mut command: Command, input: Option<Vec<u8>>, timeout: Duration) -> Result<Output, RunError> {
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
/// para medir dónde se va el tiempo sin instalar un perfilador. El registro
/// (en debug) lo anota siempre.
pub fn report_timing(label: &str, started: Instant) {
    let seconds = started.elapsed().as_secs_f64();
    logging::debug(format_args!("[timing] {label}: {seconds:.2} s"));
    if std::env::var_os("INVESTIGACION_TIMING").is_some_and(|v| v == "1") {
        eprintln!("[timing] {label}: {seconds:.2} s");
    }
}
