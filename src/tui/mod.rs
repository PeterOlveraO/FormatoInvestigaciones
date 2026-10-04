//! Menú interactivo a pantalla completa (ratatui + crossterm, funciona igual
//! en Linux, macOS y Windows). No duplica lógica: arma los mismos `Args` que
//! el CLI y llama a `cli::execute`.

mod app;
mod picker;
mod tools;
mod ui;
mod wizard;

pub use app::{App, FIELDS, FieldKey};

use std::time::Duration;

use crossterm::event::{self, Event, KeyEventKind};

use crate::i18n;
use crate::project::Project;
use crate::settings::Settings;

/// Abre el menú y devuelve el código de salida.
pub fn run() -> i32 {
    let project = Project::discover();
    // El idioma guardado manda; si no hay, se pregunta con el del sistema marcado.
    let settings = Settings::load(&project.env_file()).unwrap_or_default();
    let saved = i18n::configured(&settings);
    i18n::set(saved.unwrap_or_else(i18n::detect));
    let mut app = App::new(project);
    app.detect_tools();
    // Idioma la primera vez, el asistente si no hay perfiles y luego el inicio.
    app.start(saved.is_none().then(i18n::current));
    let mut terminal = ratatui::init();
    let result = event_loop(&mut terminal, &mut app);
    ratatui::restore();
    match result {
        Ok(()) => 0,
        Err(error) => {
            eprintln!("Error: {error}");
            1
        }
    }
}

fn event_loop(terminal: &mut ratatui::DefaultTerminal, app: &mut App) -> std::io::Result<()> {
    while !app.should_quit {
        app.poll_worker();
        app.poll_tools();
        terminal.draw(|frame| ui::draw(frame, app))?;
        // Espera corta para refrescar el panel mientras se genera el PDF.
        if event::poll(Duration::from_millis(100))? {
            // En Windows llegan también los eventos de soltar la tecla.
            if let Event::Key(key) = event::read()?
                && key.kind == KeyEventKind::Press
            {
                app.handle_key(key);
            }
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests;
