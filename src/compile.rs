//! Compilación del .tex con pdflatex.

use std::ffi::OsString;
use std::path::{Path, PathBuf};
use std::process::Command;

use crate::encoding::{decode_latex_log, decode_process_output};
use crate::error::{GenerationError, Result};
use crate::latex::{summarize_latex_errors, unsupported_character_warnings};
use crate::logging;
use crate::process::{RunError, TOOL_TIMEOUT, run_with_timeout};

/// Un índice que cambia de longitud desplaza las páginas; se repite pdflatex
/// hasta que el `.toc` se estabiliza, con este máximo.
pub const MAX_LATEX_RUNS: usize = 4;

/// Copia al temporal lo que la plantilla carga por nombre: el preámbulo común
/// (`*.sty` de `common/`), los `.sty` que traiga la propia plantilla y los
/// logos. pdflatex corre con el cwd del Markdown, así que una ruta relativa no
/// serviría; el temporal va al principio de TEXINPUTS.
pub fn copy_template_assets(
    template: &Path,
    common_dir: &Path,
    format_sty: Option<&Path>,
    destination: &Path,
    logos: Option<&Path>,
) -> Result<()> {
    // El formato elegido se copia con un nombre fijo: el diseño lo carga como
    // `investigacion-format` sin saber cuál es.
    if let Some(format_sty) = format_sty {
        std::fs::copy(format_sty, destination.join("investigacion-format.sty"))?;
    }
    let template_dir = template.parent().unwrap_or(Path::new("."));
    for directory in [common_dir, template_dir] {
        for entry in std::fs::read_dir(directory).into_iter().flatten().flatten() {
            let path = entry.path();
            if path.extension().is_some_and(|ext| ext == "sty") && path.is_file() {
                std::fs::copy(&path, destination.join(entry.file_name()))?;
            }
        }
    }
    // Sin carpeta de logos no pasa nada: la portada los envuelve en \IfFileExists.
    if let Some(logos) = logos {
        for entry in std::fs::read_dir(logos)?.flatten() {
            if entry.path().is_file() {
                std::fs::copy(entry.path(), destination.join(entry.file_name()))?;
            }
        }
    }
    Ok(())
}

/// TEXINPUTS que busca primero en `directory`. El separador final (entrada
/// vacía) significa «y además las rutas por omisión»; sin él pdflatex no
/// encontraría ni sus propios paquetes.
pub fn latex_search_path(directory: &Path) -> Result<OsString> {
    let mut paths = vec![directory.to_path_buf()];
    if let Some(previous) = std::env::var_os("TEXINPUTS") {
        paths.extend(std::env::split_paths(&previous).filter(|p| !p.as_os_str().is_empty()));
    }
    paths.push(PathBuf::new());
    std::env::join_paths(paths)
        .map_err(|e| GenerationError::new(tr!(es: "TEXINPUTS no válido: {e}", en: "Invalid TEXINPUTS: {e}")))
}

/// Guarda el .tex y el .log fuera del temporal (que se borra) para revisarlos.
pub fn keep_failure_artifacts(tex_path: &Path, destination: &Path) -> Option<PathBuf> {
    let saved = copy_failure_artifacts(tex_path, destination);
    match &saved {
        Some(log) => logging::info(format_args!("failed build kept: {} and last-error.tex", log.display())),
        None => {
            logging::warn(format_args!("could not keep last-error.tex/.log in {}", destination.display()))
        }
    }
    saved
}

fn copy_failure_artifacts(tex_path: &Path, destination: &Path) -> Option<PathBuf> {
    std::fs::create_dir_all(destination).ok()?;
    let mut saved = None;
    for extension in ["tex", "log"] {
        let source = tex_path.with_extension(extension);
        if source.is_file() {
            let copy = destination.join(format!("last-error.{extension}"));
            std::fs::copy(&source, &copy).ok()?;
            if extension == "log" {
                saved = Some(copy);
            }
        }
    }
    saved
}

/// Archivos que pdflatex escribe y relee en la siguiente pasada.
const STATE_EXTENSIONS: [&str; 2] = ["aux", "toc"];

/// Compila las veces necesarias para estabilizar índice y referencias.
///
/// `state_dir` guarda el `.aux` y el `.toc` de la generación anterior del
/// mismo trabajo: si el documento no cambió de estructura, la primera pasada
/// ya lee el índice correcto y basta con una sola. Si ese estado viejo hiciera
/// fallar la compilación, se descarta y se compila desde cero.
pub fn compile_pdf(
    tex_path: &Path,
    output_pdf: &Path,
    working_directory: &Path,
    state_dir: Option<&Path>,
    on_warning: &mut dyn FnMut(String),
) -> Result<()> {
    let restored = state_dir.and_then(|dir| restore_state(dir, tex_path));
    match (state_dir, &restored) {
        (Some(dir), Some(_)) => logging::info(format_args!("latex state: cache hit ({})", dir.display())),
        (Some(dir), None) => logging::info(format_args!("latex state: cache miss ({})", dir.display())),
        (None, _) => logging::debug("latex state: not used"),
    }
    // El intento con estado previo no deja `last-error.*`: si falla se repite
    // desde cero, y solo un fallo de ese segundo intento es un error real.
    let log = match run_passes(tex_path, output_pdf, working_directory, restored.clone(), restored.is_none())
    {
        Err(error) if restored.is_some() => {
            logging::warn(format_args!(
                "build with the cached latex state failed; retrying from scratch:\n{error}"
            ));
            for extension in STATE_EXTENSIONS {
                let _ = std::fs::remove_file(tex_path.with_extension(extension));
            }
            run_passes(tex_path, output_pdf, working_directory, None, true)?
        }
        other => other?,
    };

    let generated = tex_path.with_extension("pdf");
    if !generated.is_file() {
        return Err(GenerationError::new(tr!(
            es: "pdflatex terminó sin producir el PDF esperado.",
            en: "pdflatex finished without producing the expected PDF."
        )));
    }
    for warning in unsupported_character_warnings(&log) {
        on_warning(warning);
    }
    std::fs::copy(&generated, output_pdf).map_err(|e| crate::generate::cannot_write(output_pdf, &e))?;
    if let Some(dir) = state_dir {
        // La caché es una ayuda: si no se puede guardar, no es un error.
        match save_state(dir, tex_path) {
            Ok(()) => logging::debug(format_args!("latex state saved in {}", dir.display())),
            Err(error) => logging::warn(format_args!("latex state not saved in {}: {error}", dir.display())),
        }
    }
    Ok(())
}

/// Copia el estado guardado junto al .tex y devuelve el índice que contenía.
fn restore_state(state_dir: &Path, tex_path: &Path) -> Option<String> {
    let toc = std::fs::read(state_dir.join("trabajo.toc")).ok()?;
    for extension in STATE_EXTENSIONS {
        std::fs::copy(state_dir.join(format!("trabajo.{extension}")), tex_path.with_extension(extension))
            .ok()?;
    }
    Some(String::from_utf8_lossy(&toc).into_owned())
}

fn save_state(state_dir: &Path, tex_path: &Path) -> std::io::Result<()> {
    std::fs::create_dir_all(state_dir)?;
    for extension in STATE_EXTENSIONS {
        let source = tex_path.with_extension(extension);
        let target = state_dir.join(format!("trabajo.{extension}"));
        // Un diseño sin índice no escribe `.toc`: se guarda vacío, que es lo
        // mismo que lee `run_passes`, y la caché sirve igual.
        if source.is_file() {
            std::fs::copy(source, target)?;
        } else {
            std::fs::write(target, "")?;
        }
    }
    Ok(())
}

/// Las pasadas de pdflatex. Sin estado previo, la primera va con `-draftmode`
/// (escribe `.aux` y `.toc` pero no el PDF, que es lo caro) porque nunca puede
/// ser la definitiva. Devuelve el log de la última pasada.
fn run_passes(
    tex_path: &Path,
    output_pdf: &Path,
    working_directory: &Path,
    restored_toc: Option<String>,
    keep_artifacts: bool,
) -> Result<String> {
    let temp_dir = tex_path.parent().unwrap_or(Path::new("."));
    let texinputs = latex_search_path(temp_dir)?;
    let log_path = tex_path.with_extension("log");
    let toc_path = tex_path.with_extension("toc");
    let output_dir = output_pdf.parent().unwrap_or(Path::new("."));
    let failure_note = || {
        keep_artifacts
            .then(|| keep_failure_artifacts(tex_path, output_dir))
            .flatten()
            .map(|log| tr!(es: "\n\nRegistro completo: {}", en: "\n\nFull log: {}", log.display()))
            .unwrap_or_default()
    };

    let has_state = restored_toc.is_some();
    let mut previous_toc = restored_toc;
    let mut log = String::new();
    for run in 1..=MAX_LATEX_RUNS {
        let mut command = Command::new("pdflatex");
        command.args(["-interaction=nonstopmode", "-halt-on-error", "-file-line-error"]);
        let draft = run == 1 && !has_state;
        if draft {
            command.arg("-draftmode");
        }
        logging::info(format_args!("pdflatex pass {run}{}", if draft { " (draft mode)" } else { "" }));
        let mut output_arg = OsString::from("-output-directory=");
        output_arg.push(temp_dir);
        command.arg(output_arg).arg(tex_path).current_dir(working_directory).env("TEXINPUTS", &texinputs);

        let started = std::time::Instant::now();
        let result = run_with_timeout(command, None, TOOL_TIMEOUT).map_err(|e| match e {
            RunError::NotFound => GenerationError::new(tr!(
                es: "No se encontró pdflatex. Instala TeX Live (o MiKTeX) y comprueba que esté en el PATH.",
                en: "pdflatex was not found. Install TeX Live (or MiKTeX) and make sure it is on the PATH."
            )),
            RunError::Timeout => GenerationError::new(tr!(
                es: "pdflatex siguió trabajando más de {} segundos y se detuvo; busca en el documento \
                     algo que LaTeX no pueda componer.{}",
                en: "pdflatex kept working for more than {} seconds and was stopped; look in the document \
                     for something LaTeX cannot typeset.{}",
                TOOL_TIMEOUT.as_secs(),
                failure_note()
            )),
            RunError::Io(error) => error.into(),
        })?;

        crate::process::report_timing(&format!("pdflatex pass {run}"), started);
        log = std::fs::read(&log_path).map(|raw| decode_latex_log(&raw)).unwrap_or_default();
        if log.is_empty() {
            log = decode_process_output(&result.stdout);
        }
        if !result.status.success() {
            let mut detail = summarize_latex_errors(&log);
            if detail.is_empty() {
                let text = match log.trim() {
                    "" => decode_process_output(&result.stderr).trim().to_owned(),
                    trimmed => trimmed.to_owned(),
                };
                let start = text.char_indices().rev().nth(1999).map_or(0, |(i, _)| i);
                detail = text[start..].to_owned();
            }
            return Err(GenerationError::new(tr!(
                es: "Falló la compilación de LaTeX:\n{detail}{}",
                en: "The LaTeX build failed:\n{detail}{}",
                failure_note()
            )));
        }

        let current_toc = std::fs::read(&toc_path)
            .map(|raw| String::from_utf8_lossy(&raw).into_owned())
            .unwrap_or_default();
        let needs_rerun = log.contains("Rerun to get")
            || log.contains("Rerun LaTeX")
            || previous_toc.as_deref() != Some(current_toc.as_str());
        previous_toc = Some(current_toc);
        if (run > 1 || has_state) && !needs_rerun {
            logging::info(format_args!("latex: stable after pass {run}"));
            break;
        }
        if run == MAX_LATEX_RUNS {
            logging::warn(format_args!("latex: the table of contents still changed after {run} passes"));
        } else if needs_rerun {
            logging::debug("latex: rerun needed (the .toc changed or LaTeX asked for it)");
        }
    }
    Ok(log)
}
