//! Registro de diagnóstico en `cache/logs/investigacion.log`.
//!
//! Sirve para saber qué pasó cuando algo falla, sobre todo en el menú, donde
//! no hay terminal en la que leer avisos. Nunca rompe la generación: si el
//! archivo no se puede abrir o escribir, el registro se apaga solo y en
//! silencio. Hasta que alguien llama a `init` todo es un no-op (así las
//! pruebas no dejan archivos en el proyecto).
//!
//! Las líneas son técnicas y van en inglés; los mensajes que ve la persona
//! usuaria se copian tal como se mostraron.

use std::any::Any;
use std::backtrace::Backtrace;
use std::ffi::OsString;
use std::fmt::Display;
use std::fs::OpenOptions;
use std::io::{self, Write};
use std::panic::Location;
use std::path::{Path, PathBuf};
use std::sync::{Mutex, MutexGuard, TryLockError};
use std::time::Duration;

use crate::project::{Project, absolute};

/// Nivel mínimo: `debug` lo baja, `off` o `0` apaga el registro y cualquier
/// otro valor (o ninguno) deja `info`.
pub const LOG_ENV: &str = "INVESTIGACION_LOG";
pub const LOG_DIRECTORY: &str = "logs";
pub const LOG_FILE: &str = "investigacion.log";
/// Al abrirse, un registro más grande que esto pasa a `investigacion.old.log`.
pub const MAX_LOG_BYTES: u64 = 1024 * 1024;

/// Sangría de las líneas que siguen a la primera: el ancho de la fecha.
const CONTINUATION: &str = "                        | ";

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Level {
    Debug,
    Info,
    Warn,
    Error,
}

impl Level {
    /// Etiqueta de ancho fijo, para que las columnas queden alineadas.
    pub fn label(self) -> &'static str {
        match self {
            Level::Debug => "DEBUG",
            Level::Info => "INFO ",
            Level::Warn => "WARN ",
            Level::Error => "ERROR",
        }
    }
}

/// Umbral según el valor de `INVESTIGACION_LOG`; `None` = registro apagado.
pub fn threshold(value: Option<&str>) -> Option<Level> {
    match value.map(|v| v.trim().to_ascii_lowercase()).as_deref() {
        Some("off" | "0") => None,
        Some("debug") => Some(Level::Debug),
        _ => Some(Level::Info),
    }
}

/// Una entrada: `fecha [NIVEL] [hilo] mensaje`; las demás líneas del mensaje
/// van sangradas para que cada entrada se lea como un bloque.
pub fn format_entry(timestamp: &str, level: Level, thread: &str, message: &str) -> String {
    let mut lines = message.trim_end().lines();
    let first = lines.next().unwrap_or_default().trim_end();
    let mut entry = format!("{timestamp} [{}] [{thread}] {first}\n", level.label());
    for line in lines {
        let line = line.trim_end();
        entry.push_str(if line.is_empty() { CONTINUATION.trim_end() } else { CONTINUATION });
        entry.push_str(line);
        entry.push('\n');
    }
    entry
}

/// Hora local con milisegundos.
fn timestamp() -> String {
    chrono::Local::now().format("%Y-%m-%d %H:%M:%S%.3f").to_string()
}

/// Nombre del hilo o, si no tiene, su número (`thread-7`).
fn thread_label() -> String {
    let thread = std::thread::current();
    match thread.name() {
        Some(name) => name.to_owned(),
        None => format!("{:?}", thread.id()).replace("ThreadId(", "thread-").replace(')', ""),
    }
}

/// Si el registro pasa de `limit` bytes, lo renombra a `<nombre>.old.log`
/// (pisando el anterior). Devuelve si rotó.
pub fn rotate(path: &Path, limit: u64) -> bool {
    if !std::fs::metadata(path).is_ok_and(|m| m.len() > limit) {
        return false;
    }
    let old = path.with_extension("old.log");
    let _ = std::fs::remove_file(&old);
    std::fs::rename(path, &old).is_ok()
}

/// El núcleo: escribe entradas con nivel en cualquier destino. No sabe nada
/// del estado global, así que las pruebas lo usan sin pisarse.
pub struct Logger {
    sink: Box<dyn Write + Send>,
    threshold: Level,
}

impl Logger {
    pub fn new(sink: impl Write + Send + 'static, threshold: Level) -> Self {
        Self { sink: Box::new(sink), threshold }
    }

    /// Abre el archivo para añadir (creando su carpeta) tras rotarlo si es
    /// grande. Una sesión nueva empieza tras una línea en blanco.
    pub fn open(path: &Path, threshold: Level) -> io::Result<Self> {
        if let Some(directory) = path.parent() {
            std::fs::create_dir_all(directory)?;
        }
        rotate(path, MAX_LOG_BYTES);
        let mut file = OpenOptions::new().create(true).append(true).open(path)?;
        if file.metadata()?.len() > 0 {
            file.write_all(b"\n")?;
        }
        Ok(Self::new(file, threshold))
    }

    pub fn enabled(&self, level: Level) -> bool {
        level >= self.threshold
    }

    /// Escribe la entrada de una vez y sin búfer: si el programa muere justo
    /// después, la línea ya está en el disco.
    pub fn log(&mut self, level: Level, message: &str) -> io::Result<()> {
        if !self.enabled(level) {
            return Ok(());
        }
        let entry = format_entry(&timestamp(), level, &thread_label(), message);
        self.sink.write_all(entry.as_bytes())?;
        self.sink.flush()
    }
}

struct Active {
    logger: Logger,
    path: PathBuf,
}

struct State {
    started: bool,
    active: Option<Active>,
}

impl State {
    /// Escribe; si falla, apaga el registro en vez de devolver el error.
    fn write(&mut self, level: Level, message: &str) {
        if let Some(active) = &mut self.active
            && active.logger.log(level, message).is_err()
        {
            self.active = None;
        }
    }

    fn enabled(&self, level: Level) -> bool {
        self.active.as_ref().is_some_and(|a| a.logger.enabled(level))
    }
}

static STATE: Mutex<State> = Mutex::new(State { started: false, active: None });

/// Un pánico con el candado tomado no debe dejar el registro inservible.
fn state() -> MutexGuard<'static, State> {
    STATE.lock().unwrap_or_else(|poisoned| poisoned.into_inner())
}

/// Abre el registro del proyecto y escribe la cabecera de la sesión. Solo
/// la primera llamada hace algo; si no se puede abrir (o la carpeta no es un
/// proyecto, para no dejar un `cache/` en cualquier lado), queda apagado.
pub fn init(project: &Project) {
    {
        let mut state = state();
        if state.started {
            return;
        }
        state.started = true;
    }
    let Some(level) = threshold(std::env::var(LOG_ENV).ok().as_deref()) else { return };
    if !project.has_templates() {
        return;
    }
    let path = absolute(&project.cache_dir().join(LOG_DIRECTORY).join(LOG_FILE));
    // La cabecera se arma sin el candado: leer el .env también registra.
    let header = session_header(project, level);
    let Ok(mut logger) = Logger::open(&path, level) else { return };
    if logger.log(Level::Info, &header).is_ok() {
        state().active = Some(Active { logger, path });
    }
}

/// Versión, sistema, idioma, raíz del proyecto y argumentos de la sesión.
fn session_header(project: &Project, level: Level) -> String {
    let argv: Vec<OsString> = std::env::args_os().collect();
    let lang = crate::cli::prescan_language(&argv, project);
    let args: Vec<String> = argv.iter().map(|a| a.to_string_lossy().into_owned()).collect();
    let cwd = std::env::current_dir().map(|d| d.display().to_string()).unwrap_or_default();
    format!(
        "==== investigacion {} started ({}/{}) ====\ninterface language: {}\nproject: {}\n\
         working directory: {cwd}\narguments: {args:?}\nlog level: {level:?}",
        env!("CARGO_PKG_VERSION"),
        std::env::consts::OS,
        std::env::consts::ARCH,
        lang.code(),
        project.root.display(),
    )
}

/// Cierra el registro; un `init` posterior lo vuelve a abrir.
pub fn shutdown() {
    let mut state = state();
    state.active = None;
    state.started = false;
}

/// El archivo de registro activo (para ofrecer «abrir el registro»).
pub fn path() -> Option<PathBuf> {
    state().active.as_ref().map(|a| a.path.clone())
}

/// Si un nivel se escribiría; sirve para no armar mensajes caros en vano.
pub fn enabled(level: Level) -> bool {
    state().enabled(level)
}

/// Escribe una entrada. El mensaje se formatea fuera del candado: así un
/// `Display` que a su vez registre algo no se bloquea.
pub fn log(level: Level, message: impl Display) {
    if !enabled(level) {
        return;
    }
    let text = message.to_string();
    state().write(level, &text);
}

pub fn debug(message: impl Display) {
    log(Level::Debug, message);
}

pub fn info(message: impl Display) {
    log(Level::Info, message);
}

pub fn warn(message: impl Display) {
    log(Level::Warn, message);
}

pub fn error(message: impl Display) {
    log(Level::Error, message);
}

/// Texto de un pánico: mensaje, lugar, hilo y backtrace.
pub fn describe_panic(
    payload: &(dyn Any + Send),
    location: Option<&Location<'_>>,
    thread: &str,
    backtrace: &dyn Display,
) -> String {
    let message = payload
        .downcast_ref::<&str>()
        .copied()
        .or_else(|| payload.downcast_ref::<String>().map(String::as_str))
        .unwrap_or("(panic without a text message)");
    let place = location.map_or_else(|| "an unknown location".to_owned(), |l| l.to_string());
    format!("panic in thread '{thread}' at {place}: {message}\nbacktrace:\n{backtrace}")
}

/// Toma el candado sin esperar para siempre: si el pánico ocurrió mientras
/// este mismo hilo escribía, esperar sería un bloqueo eterno.
fn state_for_panic() -> Option<MutexGuard<'static, State>> {
    for _ in 0..100 {
        match STATE.try_lock() {
            Ok(guard) => return Some(guard),
            Err(TryLockError::Poisoned(poisoned)) => return Some(poisoned.into_inner()),
            Err(TryLockError::WouldBlock) => std::thread::sleep(Duration::from_millis(5)),
        }
    }
    None
}

/// Registra cada pánico (Error, con backtrace) y luego llama al gancho que ya
/// estaba, que imprime el mensaje como siempre.
///
/// Orden: se instala ANTES de `ratatui::init()`. ratatui envuelve el gancho
/// vigente con el suyo, así que ante un pánico primero restaura la terminal,
/// después corre este (registra) y al final el de Rust imprime el mensaje en
/// una terminal ya normal.
pub fn install_panic_hook() {
    let previous = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |info| {
        if state_for_panic().is_some_and(|state| state.enabled(Level::Error)) {
            let backtrace = Backtrace::force_capture();
            let entry = describe_panic(info.payload(), info.location(), &thread_label(), &backtrace);
            if let Some(mut state) = state_for_panic() {
                state.write(Level::Error, &entry);
            }
        }
        previous(info);
    }));
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use super::*;

    fn read(path: &Path) -> String {
        std::fs::read_to_string(path).unwrap_or_default()
    }

    /// Destino que siempre falla, como un disco lleno.
    struct Broken;
    impl Write for Broken {
        fn write(&mut self, _: &[u8]) -> io::Result<usize> {
            Err(io::Error::other("disco lleno"))
        }
        fn flush(&mut self) -> io::Result<()> {
            Ok(())
        }
    }

    /// Destino en memoria que la prueba puede leer después.
    #[derive(Clone, Default)]
    struct Shared(Arc<Mutex<Vec<u8>>>);
    impl Write for Shared {
        fn write(&mut self, data: &[u8]) -> io::Result<usize> {
            self.0.lock().unwrap().extend_from_slice(data);
            Ok(data.len())
        }
        fn flush(&mut self) -> io::Result<()> {
            Ok(())
        }
    }
    impl Shared {
        fn text(&self) -> String {
            String::from_utf8(self.0.lock().unwrap().clone()).unwrap()
        }
    }

    #[test]
    fn an_entry_has_time_level_thread_and_message() {
        assert_eq!(
            format_entry("2026-10-03 14:22:01.123", Level::Info, "main", "PDF listo"),
            "2026-10-03 14:22:01.123 [INFO ] [main] PDF listo\n"
        );
        let sink = Shared::default();
        let mut logger = Logger::new(sink.clone(), Level::Info);
        logger.log(Level::Warn, "falta una imagen").unwrap();
        let line = sink.text();
        let pattern = regex::Regex::new(
            r"^\d{4}-\d{2}-\d{2} \d{2}:\d{2}:\d{2}\.\d{3} \[WARN \] \[[^\]]+\] falta una imagen\n$",
        )
        .unwrap();
        assert!(pattern.is_match(&line), "{line:?}");
    }

    #[test]
    fn multi_line_messages_are_indented() {
        let entry = format_entry("T", Level::Error, "worker", "Falló LaTeX:\r\nlínea 1  \n\nlínea 3\n\n");
        assert_eq!(
            entry,
            format!(
                "T [ERROR] [worker] Falló LaTeX:\n{CONTINUATION}línea 1\n{}\n{CONTINUATION}línea 3\n",
                CONTINUATION.trim_end()
            )
        );
        assert_eq!(CONTINUATION.len(), "2026-10-03 14:22:01.123 ".len() + 2);
    }

    #[test]
    fn levels_below_the_threshold_are_skipped() {
        let sink = Shared::default();
        let mut logger = Logger::new(sink.clone(), Level::Info);
        for (level, text) in
            [(Level::Debug, "d"), (Level::Info, "i"), (Level::Warn, "w"), (Level::Error, "e")]
        {
            logger.log(level, text).unwrap();
        }
        let text = sink.text();
        assert_eq!(text.lines().count(), 3, "{text}");
        assert!(!text.contains("[DEBUG]"));
        let sink = Shared::default();
        let mut logger = Logger::new(sink.clone(), Level::Debug);
        logger.log(Level::Debug, "d").unwrap();
        assert!(sink.text().contains("[DEBUG]"));
    }

    #[test]
    fn the_variable_sets_the_threshold() {
        assert_eq!(threshold(None), Some(Level::Info));
        assert_eq!(threshold(Some("DEBUG")), Some(Level::Debug));
        assert_eq!(threshold(Some(" off ")), None);
        assert_eq!(threshold(Some("0")), None);
        assert_eq!(threshold(Some("ruido")), Some(Level::Info));
    }

    #[test]
    fn a_large_log_is_rotated_when_opened() {
        let directory = tempfile::tempdir().unwrap();
        let path = directory.path().join("logs").join(LOG_FILE);
        let old = directory.path().join("logs").join("investigacion.old.log");
        // Pequeño: se sigue escribiendo en el mismo, tras una línea en blanco.
        Logger::open(&path, Level::Info).unwrap().log(Level::Info, "primera").unwrap();
        Logger::open(&path, Level::Info).unwrap().log(Level::Info, "segunda").unwrap();
        let text = read(&path);
        assert!(text.contains("primera") && text.contains("\n\n") && text.contains("segunda"), "{text}");
        assert!(!old.exists());
        // Grande: pasa a .old.log (pisando el anterior) y se empieza de cero.
        std::fs::write(&old, "viejo").unwrap();
        std::fs::write(&path, vec![b'x'; MAX_LOG_BYTES as usize + 1]).unwrap();
        Logger::open(&path, Level::Info).unwrap().log(Level::Info, "nueva").unwrap();
        assert_eq!(std::fs::metadata(&old).unwrap().len(), MAX_LOG_BYTES + 1);
        assert!(read(&path).starts_with("20") && read(&path).contains("nueva"));
        assert!(!rotate(&path, MAX_LOG_BYTES));
        assert!(rotate(&path, 10));
    }

    #[test]
    fn an_unwritable_place_is_an_error_for_the_core_and_silence_for_the_global() {
        let directory = tempfile::tempdir().unwrap();
        let file = directory.path().join("ocupado");
        std::fs::write(&file, "x").unwrap();
        assert!(Logger::open(&file.join("logs").join(LOG_FILE), Level::Info).is_err());
        assert!(Logger::new(Broken, Level::Info).log(Level::Info, "x").is_err());
        // Un destino que falla apaga el registro en vez de propagar el error.
        let mut state = State {
            started: true,
            active: Some(Active { logger: Logger::new(Broken, Level::Info), path: file }),
        };
        state.write(Level::Error, "x");
        assert!(state.active.is_none());
        state.write(Level::Error, "y");
    }

    #[test]
    fn without_init_everything_is_a_no_op() {
        // Ninguna prueba de la biblioteca llama a `init`.
        debug("d");
        info(format_args!("i {}", 1));
        warn(String::from("w"));
        error("e");
        assert!(path().is_none());
        assert!(!enabled(Level::Error));
    }

    #[test]
    fn a_panic_is_described_with_place_thread_and_backtrace() {
        let location = Location::caller();
        let owned: Box<dyn Any + Send> = Box::new(String::from("índice fuera de rango"));
        let text = describe_panic(owned.as_ref(), Some(location), "generacion", &"frame 0: main");
        assert!(text.starts_with("panic in thread 'generacion' at src/logging.rs:"), "{text}");
        assert!(text.contains(": índice fuera de rango\nbacktrace:\nframe 0: main"), "{text}");
        let fixed: Box<dyn Any + Send> = Box::new("boom");
        assert!(describe_panic(fixed.as_ref(), None, "main", &"").contains("an unknown location: boom"));
        let other: Box<dyn Any + Send> = Box::new(7_u8);
        assert!(describe_panic(other.as_ref(), None, "main", &"").contains("without a text message"));
        // Y como entrada del registro, sangrada bajo una sola cabecera.
        let sink = Shared::default();
        Logger::new(sink.clone(), Level::Info).log(Level::Error, &text).unwrap();
        let logged = sink.text();
        let headers = logged.lines().filter(|l| !l.starts_with(CONTINUATION.trim_end())).count();
        assert_eq!(headers, 1, "{logged}");
    }
}
