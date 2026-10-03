//! Binario `investigacion`: con argumentos es el CLI.

fn main() {
    std::process::exit(investigacion::cli::main_with_args(std::env::args_os()));
}
