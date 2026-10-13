//! El registro global de punta a punta: `init`, cabecera, errores de E/S y el
//! gancho de pánico. Va en su propio binario porque toca estado del proceso.

use investigacion::GenerationError;
use investigacion::logging;
use investigacion::project::Project;

#[test]
fn the_global_log_records_the_session_io_errors_and_panics() {
    // Con el registro apagado a propósito no hay nada que comprobar.
    if logging::threshold(std::env::var(logging::LOG_ENV).ok().as_deref()).is_none() {
        return;
    }
    investigacion::i18n::set(investigacion::i18n::Lang::En);
    let directory = tempfile::tempdir().unwrap();
    // Una carpeta que no es un proyecto no recibe un cache/logs/.
    let stray = tempfile::tempdir().unwrap();
    logging::init(&Project::at(stray.path()));
    assert!(logging::path().is_none() && !stray.path().join("cache").exists());
    logging::shutdown();

    let project = Project::at(directory.path());
    std::fs::create_dir_all(project.common_dir()).unwrap();
    std::fs::write(project.common_dir().join("investigacion-base.sty"), "").unwrap();
    logging::init(&project);
    logging::init(&project);
    let path = logging::path().expect("the log should be open");
    assert_eq!(path, project.cache_dir().join("logs").join(logging::LOG_FILE));

    logging::info("hola\nsegunda línea");
    // Un error de E/S queda con el lugar del `?` y el mensaje remite al registro.
    let io = || -> Result<Vec<u8>, GenerationError> { Ok(std::fs::read(directory.path().join("nada"))?) };
    let error = io().unwrap_err();
    assert!(error.0.contains("Details in the log") && error.0.contains(&*path.to_string_lossy()), "{error}");

    logging::install_panic_hook();
    let worker = std::thread::Builder::new().name("worker".into()).spawn(|| panic!("boom 42")).unwrap();
    assert!(worker.join().is_err());

    let text = std::fs::read_to_string(&path).unwrap();
    assert_eq!(text.matches("==== investigacion").count(), 1, "init is idempotent:\n{text}");
    assert!(text.contains(&format!("project: {}", project.root.display())), "{text}");
    assert!(text.contains("[INFO ] [") && text.contains("] hola\n") && text.contains("| segunda línea"));
    // `file!()` usa el separador del sistema (`tests\logging.rs` en Windows).
    assert!(text.contains(&format!("I/O error at {}:", file!())), "{text}");
    assert!(text.contains(&format!("[ERROR] [worker] panic in thread 'worker' at {}:", file!())), "{text}");
    assert!(text.contains("boom 42") && text.contains("backtrace:"), "{text}");

    logging::shutdown();
    assert!(logging::path().is_none());
    logging::info("después de cerrar");
    assert!(!std::fs::read_to_string(&path).unwrap().contains("después de cerrar"));
}
