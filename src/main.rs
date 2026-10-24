//! Binario `investigacion`: sin argumentos abre el menú; con argumentos es el CLI.

use investigacion::{cli, image, logging, project::Project, tui};

fn main() {
    // El filtro de imágenes llama al binario para convertir: sin registro ni ajustes.
    let argv: Vec<_> = std::env::args_os().collect();
    if argv.get(1).is_some_and(|arg| arg == "--convert-image") {
        std::process::exit(image::run_cli(&argv[2..]));
    }
    // El registro y su gancho de pánico van antes que todo (y antes de que la
    // TUI llame a `ratatui::init()`), para los dos modos.
    logging::init(&Project::discover());
    logging::install_panic_hook();
    // Los filtros convierten SVG y WEBP llamando a este mismo binario.
    let converter = std::env::current_exe().ok();
    let code = if argv.len() <= 1 {
        logging::info("mode: menu");
        tui::run(converter)
    } else {
        logging::info("mode: command line");
        cli::main_with_args(argv, converter)
    };
    logging::info(format_args!("exit code {code}"));
    std::process::exit(code);
}
