//! Binario `investigacion`: sin argumentos abre el menú; con argumentos es el CLI.

fn main() {
    let code = if std::env::args_os().len() <= 1 {
        investigacion::tui::run()
    } else {
        investigacion::cli::main_with_args(std::env::args_os())
    };
    std::process::exit(code);
}
