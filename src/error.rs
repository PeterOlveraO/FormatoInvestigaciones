//! Error esperado del generador: se muestra tal cual a quien usa el programa.

/// Todo fallo previsible (archivo inexistente, Pandoc ausente, LaTeX roto...)
/// llega como este error; el CLI lo imprime y termina con código 1.
#[derive(Debug, PartialEq, Eq)]
pub struct GenerationError(pub String);

impl std::fmt::Display for GenerationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.0)
    }
}

impl std::error::Error for GenerationError {}

impl GenerationError {
    pub fn new(message: impl Into<String>) -> Self {
        Self(message.into())
    }
}

/// Un error de E/S llega sin contexto («Permiso denegado»): se registra con
/// el lugar del `?` que lo convirtió y el mensaje remite al registro.
impl From<std::io::Error> for GenerationError {
    #[track_caller]
    fn from(error: std::io::Error) -> Self {
        crate::logging::warn(format_args!("I/O error at {}: {error}", std::panic::Location::caller()));
        let mut message = error.to_string();
        if let Some(log) = crate::logging::path() {
            message.push_str(
                &tr!(es: "\n\nDetalles en el registro: {}", en: "\n\nDetails in the log: {}", log.display()),
            );
        }
        Self(message)
    }
}

pub type Result<T> = std::result::Result<T, GenerationError>;
