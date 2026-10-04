//! Binario `investigacion`: sin argumentos abre el menú; con argumentos es el CLI.

use investigacion::{cli, logging, project::Project, tui};

fn main() {
    // El registro y su gancho de pánico van antes que todo (y antes de que la
    // TUI llame a `ratatui::init()`), para los dos modos.
    logging::init(&Project::discover());
    logging::install_panic_hook();
    let code = if std::env::args_os().len() <= 1 {
        logging::info("mode: menu");
        tui::run()
    } else {
        logging::info("mode: command line");
        cli::main_with_args(std::env::args_os())
    };
    logging::info(format_args!("exit code {code}"));
    std::process::exit(code);
}
