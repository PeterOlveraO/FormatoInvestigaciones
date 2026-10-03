//! Generador de trabajos académicos: Markdown → PDF con formato APA.

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
pub mod tui;

pub use error::{GenerationError, Result};
