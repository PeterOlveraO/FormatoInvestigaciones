//! Generador de trabajos académicos: Markdown → PDF con formato APA.

// Primero, para que la macro `tr!` exista en todos los módulos de abajo.
#[macro_use]
pub mod i18n;

pub mod cli;
pub mod compile;
pub mod courses;
pub mod document;
pub mod encoding;
pub mod error;
pub mod generate;
pub mod latex;
pub mod markdown;
pub mod pandoc;
pub mod process;
pub mod project;
pub mod settings;
pub mod template;
pub mod tui;

pub use error::{GenerationError, Result};
