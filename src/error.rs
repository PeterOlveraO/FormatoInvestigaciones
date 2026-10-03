//! Error esperado del generador: se muestra tal cual a quien usa el programa.

use thiserror::Error;

/// Todo fallo previsible (archivo inexistente, Pandoc ausente, LaTeX roto...)
/// llega como este error; el CLI lo imprime y termina con código 1.
#[derive(Debug, Error, PartialEq, Eq)]
#[error("{0}")]
pub struct GenerationError(pub String);

impl GenerationError {
    pub fn new(message: impl Into<String>) -> Self {
        Self(message.into())
    }
}

impl From<std::io::Error> for GenerationError {
    fn from(error: std::io::Error) -> Self {
        Self(error.to_string())
    }
}

pub type Result<T> = std::result::Result<T, GenerationError>;
