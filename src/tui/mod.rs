//! Menú interactivo a pantalla completa (ratatui + crossterm, funciona igual
//! en Linux, macOS y Windows). No duplica lógica: arma los mismos `Args` que
//! el CLI y llama a `cli::execute`.

mod app;
mod guard;
mod picker;
mod theme;
mod tools;
mod ui;
mod wizard;

pub use app::{App, FIELDS, FieldKey};

use std::path::Path;
use std::time::Duration;

use ratatui::crossterm::event::{self, Event, KeyEventKind};

use crate::i18n;
use crate::logging;
use crate::project::Project;
use crate::settings::Settings;

/// Abre el menú y devuelve el código de salida. `converter` es este programa,
/// para convertir SVG y WEBP.
pub fn run(converter: Option<std::path::PathBuf>) -> i32 {
    let project = Project::discover();
    // Sin plantillas el menú no sirve de nada: mejor decirlo antes de abrirlo.
    if let Err(error) = project.require_templates() {
        eprintln!("Error: {error}");
        return 1;
    }
    // El idioma guardado manda; si no hay, se pregunta con el del sistema marcado.
    let (settings, problem) = load_settings(&project.settings_file());
    let saved = i18n::configured(&settings);
    i18n::set(saved.unwrap_or_else(i18n::detect));
    theme::set(&settings.get(&theme::THEME_SETTINGS));
    let mut app = App::new(project);
    app.converter = converter;
    // Un archivo roto se avisa en el menú; sigue con los valores por omisión.
    if let Some(problem) = problem {
        app.log.push(app::LogLine::Error(problem));
    }
    app.detect_tools();
    // La primera vez: idioma, carpeta de trabajo y, sin perfiles, el asistente.
    let ask_workspace = app.project.saved_workspace().is_none();
    app.start(saved.is_none().then(i18n::current), ask_workspace);
    let mut terminal = ratatui::init();
    guard::install_hook();
    let result = event_loop(&mut terminal, &mut app);
    ratatui::restore();
    match result {
        Ok(()) => 0,
        Err(error) => {
            logging::error(format_args!("tui: closed by an error: {error}"));
            let log = logging::path().map(|p| p.display().to_string()).unwrap_or_else(|| "-".into());
            eprintln!(
                "{}",
                tr!(
                    es: "El menú se cerró por un error: {error}\nDetalles en el registro: {log}",
                    en: "The menu closed because of an error: {error}\nDetails in the log: {log}"
                )
            );
            1
        }
    }
}

/// Ajustes para arrancar. Si el archivo tiene errores, valores por omisión
/// y el motivo para mostrar en el menú.
fn load_settings(path: &Path) -> (Settings, Option<String>) {
    match Settings::load(path) {
        Ok(settings) => (settings, None),
        Err(error) => (Settings::default(), Some(error.0)),
    }
}

fn event_loop(terminal: &mut ratatui::DefaultTerminal, app: &mut App) -> std::io::Result<()> {
    // Un dibujo que falla dos veces seguidas no se arregla solo: se sale.
    let mut failed_draws = 0;
    while !app.should_quit {
        // Un pánico al recoger el resultado no tumba el menú.
        if let Err(message) = guard::run(|| app.poll_worker()) {
            app.abandon_generation();
            app.recover(message);
        }
        if let Err(message) = guard::run(|| app.poll_tools()) {
            app.recover(message);
        }
        let mut drawn = Ok(());
        terminal.draw(|frame| drawn = guard::run(|| ui::draw(frame, app)))?;
        match drawn {
            Ok(()) => failed_draws = 0,
            Err(message) if failed_draws == 0 => {
                failed_draws += 1;
                app.recover(message);
            }
            Err(message) => return Err(std::io::Error::other(message)),
        }
        // Espera corta para refrescar el panel mientras se genera el PDF.
        if event::poll(Duration::from_millis(100))? {
            // En Windows llegan también los eventos de soltar la tecla.
            if let Event::Key(key) = event::read()?
                && key.kind == KeyEventKind::Press
                && let Err(message) = guard::run(|| app.handle_key(key))
            {
                app.recover(message);
            }
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests;
