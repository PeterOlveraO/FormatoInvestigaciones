//! Línea de comandos: traduce argumentos y `.env` a un `DocumentData`, llama
//! al generador e imprime. Toda la lógica vive en los demás módulos.

use std::ffi::OsString;
use std::path::PathBuf;

use clap::Parser;

use crate::document::{DocumentData, parse_members, today};
use crate::error::{GenerationError, Result};
use crate::generate::{GenerateOptions, copy_pdf_to, generate_pdf};
use crate::markdown::{read_markdown, resolve_markdown_path, validate_markdown};
use crate::project::Project;
use crate::settings::Settings;

/// Variables obligatorias del `.env` (nombre en español y en inglés).
pub const REQUIRED_ENV: [[&str; 2]; 3] =
    [["UNIVERSIDAD", "UNIVERSITY"], ["FACULTAD", "FACULTY"], ["SEMESTRE", "SEMESTER"]];

/// Las opciones antiguas en español (`--titulo`, `--materia`...) siguen
/// valiendo como alias ocultos para no romper comandos ya escritos.
#[derive(Debug, Parser)]
#[command(
    name = "investigacion",
    version,
    about = "Generates an APA academic paper in PDF from a Markdown file.",
    after_help = "Run it without arguments to open the interactive menu."
)]
pub struct Args {
    /// Markdown (.md) file with the full content. A bare name is also searched inside input/.
    pub markdown: PathBuf,

    /// Title of the paper (shown on the cover).
    #[arg(long, alias = "titulo")]
    pub title: String,

    /// Name of the PDF file (without .pdf is fine). Default: the name of the Markdown file.
    #[arg(long, alias = "nombre")]
    pub file_name: Option<String>,

    /// Subject name.
    #[arg(long, alias = "materia")]
    pub subject: String,

    /// Teacher name. Falls back to DOCENTE in the .env; if empty the line is omitted.
    #[arg(long, alias = "docente")]
    pub teacher: Option<String>,

    /// Team members in one argument, separated by commas: "Ana Ruiz, Luis Paz". Falls back to INTEGRANTES.
    #[arg(long, alias = "integrantes")]
    pub members: Option<String>,

    /// Group, e.g. "7-A". Falls back to GRUPO in the .env; if empty the line is omitted.
    #[arg(long, alias = "grupo")]
    pub group: Option<String>,

    /// Directory for the PDF (default: output/ of the project).
    #[arg(long, alias = "salida")]
    pub output: Option<PathBuf>,

    /// Extra directory for a copy of the PDF. Can be repeated.
    #[arg(long, alias = "copia", value_name = "DIRECTORY")]
    pub copy: Vec<PathBuf>,

    /// dotenv file with the permanent data (default: .env of the project).
    #[arg(long)]
    pub env_file: Option<PathBuf>,

    /// Interpret LaTeX commands written in the Markdown (\newpage, etc.).
    #[arg(long, alias = "permitir-latex")]
    pub allow_latex: bool,

    /// Template name inside templates/ (e.g. apa) or path to a .ltx file.
    #[arg(long, alias = "plantilla")]
    pub template: Option<String>,

    /// Directory with logo-universidad.png and logo-facultad.png. Falls back to LOGOS in the .env.
    #[arg(long)]
    pub logos: Option<PathBuf>,
}

/// Mensajes que produce una ejecución; el CLI los imprime y la TUI los muestra.
pub trait Reporter {
    fn warning(&mut self, message: String);
    fn info(&mut self, message: String);
}

/// Imprime en la terminal: avisos por stderr, resultados por stdout.
pub struct ConsoleReporter;

impl Reporter for ConsoleReporter {
    fn warning(&mut self, message: String) {
        eprintln!("Warning: {message}");
    }
    fn info(&mut self, message: String) {
        println!("{message}");
    }
}

/// Opción gana; si no, el valor del entorno o del `.env`.
fn option_or_setting(option: Option<&str>, settings: &Settings, names: &[&str]) -> String {
    match option {
        Some(value) => value.trim().to_owned(),
        None => settings.get(names),
    }
}

/// Ejecuta una generación completa con argumentos ya analizados.
pub fn execute(args: &Args, project: &Project, reporter: &mut dyn Reporter) -> Result<PathBuf> {
    let env_file = args.env_file.clone().unwrap_or_else(|| project.env_file());
    let settings = Settings::load(&env_file)?;
    let missing: Vec<&str> = REQUIRED_ENV
        .iter()
        .filter(|names| settings.get(names.as_slice()).is_empty())
        .map(|names| names[0])
        .collect();
    if !missing.is_empty() {
        return Err(GenerationError::new(format!(
            "Missing environment variables: {}. Set them in your .env file.",
            missing.join(", ")
        )));
    }

    let markdown_path = resolve_markdown_path(&args.markdown, &project.input_dir())?;
    let markdown = read_markdown(&markdown_path)?;
    for warning in validate_markdown(&markdown) {
        reporter.warning(warning);
    }

    let data = DocumentData {
        university: settings.get(&["UNIVERSIDAD", "UNIVERSITY"]),
        faculty: settings.get(&["FACULTAD", "FACULTY"]),
        student: settings.get(&["ALUMNO", "STUDENT"]),
        semester: settings.get(&["SEMESTRE", "SEMESTER"]),
        title: args.title.trim().to_owned(),
        subject: args.subject.trim().to_owned(),
        teacher: option_or_setting(args.teacher.as_deref(), &settings, &["DOCENTE", "TEACHER"]),
        date: today(),
        members: parse_members(&option_or_setting(
            args.members.as_deref(),
            &settings,
            &["INTEGRANTES", "MEMBERS"],
        )),
        group: option_or_setting(args.group.as_deref(), &settings, &["GRUPO", "GROUP"]),
    };
    if data.title.is_empty() || data.subject.is_empty() {
        return Err(GenerationError::new("The title and the subject cannot be empty."));
    }

    // Los logos suelen vivir fuera del proyecto: por eso admiten el .env.
    let logos = args.logos.clone().or_else(|| {
        let from_env = settings.get(&["LOGOS"]);
        (!from_env.is_empty()).then(|| PathBuf::from(from_env))
    });
    let options = GenerateOptions {
        template: args.template.clone(),
        allow_raw_latex: args.allow_latex,
        logos,
        file_name: args.file_name.clone(),
        markdown: Some(markdown),
    };
    let output_dir = args.output.clone().unwrap_or_else(|| project.output_dir());
    let pdf =
        generate_pdf(project, &markdown_path, &output_dir, &data, &options, &mut |w| reporter.warning(w))?;
    reporter.info(format!("PDF generated: {}", pdf.display()));
    for copy in copy_pdf_to(&pdf, &args.copy)? {
        reporter.info(format!("Copy saved: {}", copy.display()));
    }
    Ok(pdf)
}

/// Punto de entrada del CLI; devuelve el código de salida.
pub fn main_with_args<I, T>(argv: I) -> i32
where
    I: IntoIterator<Item = T>,
    T: Into<OsString> + Clone,
{
    let args = match Args::try_parse_from(argv) {
        Ok(args) => args,
        Err(error) => {
            let _ = error.print();
            return error.exit_code();
        }
    };
    match execute(&args, &Project::discover(), &mut ConsoleReporter) {
        Ok(_) => 0,
        Err(error) => {
            eprintln!("Error: {error}");
            1
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn parse(argv: &[&str]) -> Args {
        Args::try_parse_from(std::iter::once("investigacion").chain(argv.iter().copied())).unwrap()
    }

    #[test]
    fn option_order_does_not_matter() {
        let orders: [&[&str]; 5] = [
            &["t.md", "--title", "T", "--subject", "M", "--teacher", "D"],
            &["--title", "T", "--subject", "M", "--teacher", "D", "t.md"],
            &["--teacher", "D", "t.md", "--title", "T", "--subject", "M"],
            &["--subject", "M", "--title", "T", "t.md", "--teacher", "D"],
            &["--copy", "otro", "--title", "T", "t.md", "--teacher", "D", "--subject", "M"],
        ];
        for argv in orders {
            let args = parse(argv);
            assert_eq!(args.markdown, PathBuf::from("t.md"));
            assert_eq!(
                (args.title.as_str(), args.subject.as_str(), args.teacher.as_deref()),
                ("T", "M", Some("D"))
            );
        }
    }

    #[test]
    fn the_spanish_options_still_work() {
        let args = parse(&[
            "t.md",
            "--titulo",
            "T",
            "--materia",
            "M",
            "--docente",
            "D",
            "--grupo",
            "7-A",
            "--integrantes",
            "Ana, Luis",
            "--copia",
            "a",
            "--permitir-latex",
            "--plantilla",
            "apa",
        ]);
        assert_eq!(args.title, "T");
        assert_eq!(args.group.as_deref(), Some("7-A"));
        assert_eq!(args.members.as_deref(), Some("Ana, Luis"));
        assert!(args.allow_latex);
        assert_eq!(args.template.as_deref(), Some("apa"));
    }

    #[test]
    fn optional_values_default_to_none() {
        let args = parse(&["t.md", "--title", "T", "--subject", "M"]);
        assert!(args.teacher.is_none() && args.members.is_none() && args.group.is_none());
        assert!(args.copy.is_empty());
        let args = parse(&["t.md", "--title", "T", "--subject", "M", "--copy", "a", "--copy", "b"]);
        assert_eq!(args.copy, [PathBuf::from("a"), PathBuf::from("b")]);
    }

    #[test]
    fn the_file_name_is_its_own_option() {
        let args = parse(&["t.md", "--title", "Un titulo largo", "--subject", "M", "--file-name", "entrega"]);
        assert_eq!(args.file_name.as_deref(), Some("entrega"));
        assert_eq!(
            parse(&["t.md", "--title", "T", "--subject", "M", "--nombre", "x"]).file_name.as_deref(),
            Some("x")
        );
    }

    #[test]
    fn alumno_is_not_required() {
        assert!(REQUIRED_ENV.iter().all(|names| names[0] != "ALUMNO"));
    }
}
