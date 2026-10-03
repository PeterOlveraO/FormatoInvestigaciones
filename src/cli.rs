//! Línea de comandos: traduce argumentos y `.env` a un `DocumentData`, llama
//! al generador e imprime. Toda la lógica vive en los demás módulos.

use std::ffi::OsString;
use std::path::PathBuf;

use clap::error::{ContextKind, ContextValue, ErrorKind};
use clap::{Arg, ArgAction, CommandFactory, FromArgMatches, Parser};

use crate::courses::{Course, find_course};
use crate::document::{DocumentData, parse_members, today};
use crate::error::{GenerationError, Result};
use crate::generate::{GenerateOptions, copy_pdf_to, generate_pdf};
use crate::i18n::{self, Lang, Text};
use crate::markdown::{read_markdown, resolve_markdown_path, validate_markdown};
use crate::project::Project;
use crate::settings::Settings;

/// Variables obligatorias del `.env` (nombre en español y en inglés).
pub const REQUIRED_ENV: [[&str; 2]; 3] =
    [["UNIVERSIDAD", "UNIVERSITY"], ["FACULTAD", "FACULTY"], ["SEMESTRE", "SEMESTER"]];

/// Opciones del comando. Sus nombres van en inglés; la ayuda de cada una está
/// en `ARG_HELP`, en los dos idiomas. Los nombres antiguos (`--titulo`,
/// `--subject`...) siguen valiendo como alias ocultos.
#[derive(Debug, Parser)]
#[command(name = "investigacion", version)]
pub struct Args {
    pub markdown: PathBuf,

    #[arg(long, alias = "titulo")]
    pub title: String,

    #[arg(long, alias = "nombre")]
    pub file_name: Option<String>,

    #[arg(long, aliases = ["materia", "subject"])]
    pub course: Option<String>,

    #[arg(long, short = 'p', aliases = ["perfil", "subject-profile"])]
    pub profile: Option<String>,

    #[arg(long, alias = "docente")]
    pub teacher: Option<String>,

    #[arg(long, alias = "integrantes")]
    pub members: Option<String>,

    #[arg(long, alias = "grupo")]
    pub group: Option<String>,

    #[arg(long, alias = "salida")]
    pub output: Option<PathBuf>,

    #[arg(long, alias = "copia")]
    pub copy: Vec<PathBuf>,

    #[arg(long)]
    pub env_file: Option<PathBuf>,

    #[arg(long, alias = "permitir-latex")]
    pub allow_latex: bool,

    #[arg(long, alias = "plantilla")]
    pub template: Option<String>,

    #[arg(long)]
    pub logos: Option<PathBuf>,

    #[arg(long, alias = "idioma", value_parser = parse_lang_arg)]
    pub lang: Option<Lang>,
}

fn parse_lang_arg(value: &str) -> std::result::Result<Lang, String> {
    i18n::parse(value).ok_or_else(|| tr!(es: "usa es o en", en: "use es or en"))
}

/// Ayuda de cada opción: (id del argumento, descripción, nombre del valor).
const ARG_HELP: [(&str, Text, Option<Text>); 15] = [
    (
        "markdown",
        Text::new(
            "Archivo .md con el trabajo. Basta el nombre: también se busca dentro de input/",
            "Markdown (.md) file with the paper. A bare name is also searched inside input/",
        ),
        Some(Text::new("MARKDOWN", "MARKDOWN")),
    ),
    (
        "title",
        Text::new(
            "Título del trabajo (solo sale en la portada)",
            "Title of the paper (only shown on the cover)",
        ),
        Some(Text::new("TÍTULO", "TITLE")),
    ),
    (
        "file_name",
        Text::new(
            "Nombre del PDF, con o sin .pdf. Por omisión, el del Markdown",
            "Name of the PDF, with or without .pdf. Default: the Markdown's name",
        ),
        Some(Text::new("NOMBRE", "NAME")),
    ),
    (
        "course",
        Text::new(
            "Nombre de la materia. Opcional si lo da un perfil",
            "Course name. Optional when a profile gives it",
        ),
        Some(Text::new("MATERIA", "COURSE")),
    ),
    (
        "profile",
        Text::new(
            "Perfil de materia de courses/ (p. ej. ia): pone materia, docente, grupo, plantilla y carpetas",
            "Course profile from courses/ (e.g. ia): sets course, teacher, group, template and folders",
        ),
        Some(Text::new("PERFIL", "PROFILE")),
    ),
    (
        "teacher",
        Text::new(
            "Docente. Si se omite, DOCENTE del .env; si está vacío, la línea no sale",
            "Teacher. Falls back to DOCENTE in the .env; if empty, the line is omitted",
        ),
        Some(Text::new("DOCENTE", "TEACHER")),
    ),
    (
        "members",
        Text::new(
            "Integrantes separados por comas: \"Ana Ruiz, Luis Paz\". Si se omite, INTEGRANTES del .env",
            "Team members separated by commas: \"Ana Ruiz, Luis Paz\". Falls back to INTEGRANTES in the .env",
        ),
        Some(Text::new("INTEGRANTES", "MEMBERS")),
    ),
    (
        "group",
        Text::new(
            "Grupo, p. ej. \"7-A\". Si se omite, GRUPO del .env; si está vacío, la línea no sale",
            "Group, e.g. \"7-A\". Falls back to GRUPO in the .env; if empty, the line is omitted",
        ),
        Some(Text::new("GRUPO", "GROUP")),
    ),
    (
        "output",
        Text::new(
            "Carpeta del PDF (por omisión output/, u output/<carpeta> del perfil)",
            "Folder for the PDF (default: output/, or output/<folder> of the profile)",
        ),
        Some(Text::new("CARPETA", "FOLDER")),
    ),
    (
        "copy",
        Text::new(
            "Carpeta extra para una copia del PDF. Se puede repetir",
            "Extra folder for a copy of the PDF. Can be repeated",
        ),
        Some(Text::new("CARPETA", "FOLDER")),
    ),
    (
        "env_file",
        Text::new(
            "Archivo .env con los datos fijos (por omisión, el .env del proyecto)",
            "The .env file with the permanent data (default: the project's .env)",
        ),
        Some(Text::new("ARCHIVO", "FILE")),
    ),
    (
        "allow_latex",
        Text::new(
            "Interpreta los comandos LaTeX escritos en el Markdown (\\newpage, etc.)",
            "Interpret LaTeX commands written in the Markdown (\\newpage, etc.)",
        ),
        None,
    ),
    (
        "template",
        Text::new(
            "Plantilla: un nombre de templates/ (apa, apa-simple) o la ruta a un .ltx",
            "Template: a name from templates/ (apa, apa-simple) or the path to a .ltx file",
        ),
        Some(Text::new("PLANTILLA", "TEMPLATE")),
    ),
    (
        "logos",
        Text::new(
            "Carpeta con logo-universidad.png y logo-facultad.png. Si se omite, LOGOS del .env",
            "Folder with logo-universidad.png and logo-facultad.png. Falls back to LOGOS in the .env",
        ),
        Some(Text::new("CARPETA", "FOLDER")),
    ),
    (
        "lang",
        Text::new(
            "Idioma de la interfaz: es o en. Por omisión, IDIOMA del .env o el del sistema",
            "Interface language: es or en. Default: IDIOMA in the .env, or the system's",
        ),
        Some(Text::new("IDIOMA", "LANG")),
    ),
];

// Plantillas de la ayuda. No pasan por `format!`: las llaves son de clap.
const HELP_TEMPLATE: Text = Text::new(
    "{about}\n\nUso: {usage}\n\nArgumentos:\n{positionals}\n\nOpciones:\n{options}{after-help}",
    "{about}\n\nUsage: {usage}\n\nArguments:\n{positionals}\n\nOptions:\n{options}{after-help}",
);

/// El comando con la ayuda en el idioma actual.
pub fn localized_command() -> clap::Command {
    let mut command = Args::command()
        .about(
            Text::new(
                "Genera un trabajo académico en PDF, con formato APA, a partir de un archivo Markdown.",
                "Generates an APA academic paper in PDF from a Markdown file.",
            )
            .get(),
        )
        .after_help(
            Text::new(
                "Sin argumentos abre el menú interactivo.",
                "Run it without arguments to open the interactive menu.",
            )
            .get(),
        )
        .override_usage(
            Text::new(
                "investigacion <MARKDOWN> --title <TÍTULO> [opciones]",
                "investigacion <MARKDOWN> --title <TITLE> [options]",
            )
            .get(),
        )
        .help_template(HELP_TEMPLATE.get())
        // La ayuda y la versión propias, para describirlas en el idioma actual.
        .disable_help_flag(true)
        .disable_version_flag(true)
        .arg(
            Arg::new("help")
                .short('h')
                .long("help")
                .action(ArgAction::Help)
                .help(Text::new("Muestra esta ayuda", "Show this help").get()),
        )
        .arg(
            Arg::new("version")
                .short('V')
                .long("version")
                .action(ArgAction::Version)
                .help(Text::new("Muestra la versión", "Show the version").get()),
        );
    for (id, help, value_name) in ARG_HELP {
        command = command.mut_arg(id, |arg| {
            let arg = arg.help(help.get());
            match value_name {
                Some(name) => arg.value_name(name.get()),
                None => arg,
            }
        });
    }
    command
}

/// Analiza los argumentos con la ayuda y los errores en el idioma actual.
pub fn parse_args<I, T>(argv: I) -> std::result::Result<Args, clap::Error>
where
    I: IntoIterator<Item = T>,
    T: Into<OsString> + Clone,
{
    Args::from_arg_matches(&localized_command().try_get_matches_from(argv)?)
}

/// Mensaje de un error de argumentos en el idioma actual. Los casos comunes
/// se redactan aquí; el resto lo describe clap (en inglés).
pub fn describe_clap_error(error: &clap::Error) -> String {
    let context = |kind| match error.get(kind) {
        Some(ContextValue::String(value)) => value.clone(),
        Some(ContextValue::Strings(values)) => values.join(", "),
        _ => String::new(),
    };
    let arg = context(ContextKind::InvalidArg);
    let value = context(ContextKind::InvalidValue);
    let message = match error.kind() {
        ErrorKind::MissingRequiredArgument => {
            tr!(es: "faltan argumentos obligatorios: {arg}", en: "missing required arguments: {arg}")
        }
        // Un argumento suelto casi siempre es un dato con espacios sin comillas.
        ErrorKind::UnknownArgument if !arg.starts_with('-') => tr!(
            es: "sobra el argumento «{arg}». Si el título u otro dato lleva espacios, escríbelo entre comillas.",
            en: "unexpected argument \"{arg}\". If the title or another value has spaces, put it in quotes."
        ),
        ErrorKind::UnknownArgument => tr!(es: "opción no reconocida: {arg}", en: "unknown option: {arg}"),
        ErrorKind::InvalidValue | ErrorKind::ValueValidation if value.is_empty() => {
            tr!(es: "falta el valor de {arg}", en: "a value is required for {arg}")
        }
        ErrorKind::InvalidValue | ErrorKind::ValueValidation => {
            tr!(es: "valor no válido «{value}» para {arg}", en: "invalid value \"{value}\" for {arg}")
        }
        _ => return error.render().to_string(),
    };
    tr!(
        es: "error: {message}\n\nUsa --help para ver las opciones.",
        en: "error: {message}\n\nUse --help to see the options."
    )
}

/// Busca `--lang` y `--env-file` antes de analizar el resto, para que la
/// ayuda y los errores ya salgan en su idioma.
pub fn prescan_language(argv: &[OsString], project: &Project) -> Lang {
    let mut lang = None;
    let mut env_file = None;
    let mut iter = argv.iter().skip(1).map(|arg| arg.to_string_lossy().into_owned());
    while let Some(arg) = iter.next() {
        let (flag, inline) = match arg.split_once('=') {
            Some((flag, value)) => (flag.to_owned(), Some(value.to_owned())),
            None => (arg, None),
        };
        match flag.as_str() {
            "--lang" | "--idioma" => lang = inline.or_else(|| iter.next()).as_deref().and_then(i18n::parse),
            "--env-file" => env_file = inline.or_else(|| iter.next()).map(PathBuf::from),
            _ => {}
        }
    }
    let settings = Settings::load(&env_file.unwrap_or_else(|| project.env_file())).unwrap_or_default();
    i18n::resolve(lang, &settings)
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
        eprintln!("{}", tr!(es: "Aviso: {message}", en: "Warning: {message}"));
    }
    fn info(&mut self, message: String) {
        println!("{message}");
    }
}

/// La opción gana; luego el perfil de la materia y, al final, el `.env`.
fn pick(option: Option<&str>, profile: Option<&str>, settings: &Settings, names: &[&str]) -> String {
    match (option, profile.map(str::trim).filter(|v| !v.is_empty())) {
        (Some(value), _) => value.trim().to_owned(),
        (None, Some(value)) => value.to_owned(),
        (None, None) => settings.get(names),
    }
}

/// Primero se busca el Markdown en la carpeta de la materia (así dos materias
/// pueden tener una `Tarea1.md` cada una) y luego en todo `input/`.
fn locate_markdown(args: &Args, project: &Project, course: Option<&Course>) -> Result<PathBuf> {
    if let Some(dir) = course.and_then(|c| c.input_dir(project))
        && !args.markdown.exists()
        && let Ok(found) = resolve_markdown_path(&args.markdown, &dir)
    {
        return Ok(found);
    }
    resolve_markdown_path(&args.markdown, &project.input_dir())
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
        let missing = missing.join(", ");
        return Err(GenerationError::new(tr!(
            es: "Faltan datos en el .env: {missing}. Agrégalos a tu archivo .env.",
            en: "Missing data in the .env: {missing}. Add them to your .env file."
        )));
    }

    let course = args.profile.as_deref().map(|key| find_course(project, key)).transpose()?;
    let profile = course.as_ref().map(|c| &c.profile);
    let markdown_path = locate_markdown(args, project, course.as_ref())?;
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
        course: pick(args.course.as_deref(), profile.map(|p| p.name.as_str()), &settings, &[]),
        teacher: pick(
            args.teacher.as_deref(),
            profile.map(|p| p.teacher.as_str()),
            &settings,
            &["DOCENTE", "TEACHER"],
        ),
        date: today(),
        members: parse_members(&pick(
            args.members.as_deref(),
            profile.map(|p| p.members.as_str()),
            &settings,
            &["INTEGRANTES", "MEMBERS"],
        )),
        group: pick(args.group.as_deref(), profile.map(|p| p.group.as_str()), &settings, &["GRUPO", "GROUP"]),
    };
    if data.title.is_empty() || data.course.is_empty() {
        return Err(GenerationError::new(tr!(
            es: "El título y la materia no pueden quedar vacíos (usa --course o un perfil con -p).",
            en: "The title and the course cannot be empty (use --course or a profile with -p)."
        )));
    }

    // Los logos suelen vivir fuera del proyecto: por eso admiten el .env.
    let logos = args.logos.clone().or_else(|| {
        let from_env = settings.get(&["LOGOS"]);
        (!from_env.is_empty()).then(|| PathBuf::from(from_env))
    });
    let options = GenerateOptions {
        template: args
            .template
            .clone()
            .or_else(|| profile.map(|p| p.template.trim().to_owned()).filter(|t| !t.is_empty())),
        allow_raw_latex: args.allow_latex,
        logos,
        file_name: args.file_name.clone(),
        markdown: Some(markdown),
    };
    let output_dir = args.output.clone().unwrap_or_else(|| match &course {
        Some(course) => course.output_dir(project),
        None => project.output_dir(),
    });
    let pdf =
        generate_pdf(project, &markdown_path, &output_dir, &data, &options, &mut |w| reporter.warning(w))?;
    reporter.info(tr!(es: "PDF generado: {}", en: "PDF generated: {}", pdf.display()));
    for copy in copy_pdf_to(&pdf, &args.copy)? {
        reporter.info(tr!(es: "Copia guardada: {}", en: "Copy saved: {}", copy.display()));
    }
    Ok(pdf)
}

/// Punto de entrada del CLI; devuelve el código de salida.
pub fn main_with_args<I, T>(argv: I) -> i32
where
    I: IntoIterator<Item = T>,
    T: Into<OsString> + Clone,
{
    let argv: Vec<OsString> = argv.into_iter().map(Into::into).collect();
    let project = Project::discover();
    i18n::set(prescan_language(&argv, &project));
    let args = match parse_args(argv) {
        Ok(args) => args,
        Err(error) => {
            match error.kind() {
                ErrorKind::DisplayHelp | ErrorKind::DisplayVersion => {
                    let _ = error.print();
                }
                _ => eprintln!("{}", describe_clap_error(&error)),
            }
            return error.exit_code();
        }
    };
    match execute(&args, &project, &mut ConsoleReporter) {
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
        parse_args(std::iter::once("investigacion").chain(argv.iter().copied())).unwrap()
    }

    fn error_for(argv: &[&str]) -> String {
        let error = parse_args(std::iter::once("investigacion").chain(argv.iter().copied())).unwrap_err();
        describe_clap_error(&error)
    }

    fn help() -> String {
        localized_command().render_help().to_string()
    }

    #[test]
    fn the_help_follows_the_language() {
        i18n::set(Lang::Es);
        let ayuda = help();
        assert!(ayuda.contains("Uso: investigacion <MARKDOWN> --title <TÍTULO>"), "{ayuda}");
        assert!(ayuda.contains("Opciones:") && ayuda.contains("Muestra esta ayuda"));
        assert!(ayuda.contains("--course <MATERIA>") && ayuda.contains("Perfil de materia"));
        i18n::set(Lang::En);
        let help = help();
        assert!(help.contains("Usage: investigacion <MARKDOWN> --title <TITLE>"), "{help}");
        assert!(help.contains("--course <COURSE>") && help.contains("Show this help"));
        assert!(!help.contains("--subject"), "los alias quedan ocultos");
    }

    #[test]
    fn argument_errors_are_explained_in_spanish() {
        i18n::set(Lang::Es);
        assert!(error_for(&["t.md"]).contains("faltan argumentos obligatorios: --title <TÍTULO>"));
        let unquoted = error_for(&["t.md", "--title", "Conceptos", "de", "costos"]);
        assert!(
            unquoted.contains("sobra el argumento «de»") && unquoted.contains("entre comillas"),
            "{unquoted}"
        );
        assert!(error_for(&["t.md", "--title", "T", "--nope"]).contains("opción no reconocida: --nope"));
        let lang = error_for(&["t.md", "--title", "T", "--lang", "fr"]);
        assert!(lang.contains("valor no válido «fr»"), "{lang}");
        assert!(error_for(&["t.md", "--title"]).contains("falta el valor de --title"));
    }

    #[test]
    fn argument_errors_in_english() {
        i18n::set(Lang::En);
        let unquoted = error_for(&["t.md", "--title", "Conceptos", "de", "costos"]);
        assert!(
            unquoted.contains("unexpected argument \"de\"") && unquoted.contains("in quotes"),
            "{unquoted}"
        );
        assert!(error_for(&["t.md"]).contains("missing required arguments"));
    }

    #[test]
    fn the_language_is_prescanned_before_parsing() {
        let directory = tempfile::tempdir().unwrap();
        let project = Project::at(directory.path());
        let argv = |items: &[&str]| -> Vec<OsString> { items.iter().map(OsString::from).collect() };
        assert_eq!(prescan_language(&argv(&["investigacion", "--help", "--lang", "en"]), &project), Lang::En);
        assert_eq!(prescan_language(&argv(&["investigacion", "--idioma=es"]), &project), Lang::Es);
        std::fs::write(project.env_file(), "IDIOMA=en\n").unwrap();
        assert_eq!(prescan_language(&argv(&["investigacion", "--help"]), &project), Lang::En);
        let other = directory.path().join("otro.env");
        std::fs::write(&other, "IDIOMA=es\n").unwrap();
        let with_file = argv(&["investigacion", "--env-file", other.to_str().unwrap()]);
        assert_eq!(prescan_language(&with_file, &project), Lang::Es);
    }

    #[test]
    fn option_order_does_not_matter() {
        let orders: [&[&str]; 5] = [
            &["t.md", "--title", "T", "--course", "M", "--teacher", "D"],
            &["--title", "T", "--course", "M", "--teacher", "D", "t.md"],
            &["--teacher", "D", "t.md", "--title", "T", "--course", "M"],
            &["--course", "M", "--title", "T", "t.md", "--teacher", "D"],
            &["--copy", "otro", "--title", "T", "t.md", "--teacher", "D", "--course", "M"],
        ];
        for argv in orders {
            let args = parse(argv);
            assert_eq!(args.markdown, PathBuf::from("t.md"));
            assert_eq!(
                (args.title.as_str(), args.course.as_deref(), args.teacher.as_deref()),
                ("T", Some("M"), Some("D"))
            );
        }
    }

    #[test]
    fn the_old_option_names_still_work() {
        let args = parse(&["t.md", "--title", "T", "--subject", "M", "--subject-profile", "ia"]);
        assert_eq!((args.course.as_deref(), args.profile.as_deref()), (Some("M"), Some("ia")));
        let args = parse(&["t.md", "--title", "T", "--perfil", "pm"]);
        assert_eq!(args.profile.as_deref(), Some("pm"));
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
        let args = parse(&["t.md", "--title", "T", "--course", "M"]);
        assert!(args.teacher.is_none() && args.members.is_none() && args.group.is_none());
        assert!(args.copy.is_empty());
        let args = parse(&["t.md", "--title", "T", "--course", "M", "--copy", "a", "--copy", "b"]);
        assert_eq!(args.copy, [PathBuf::from("a"), PathBuf::from("b")]);
    }

    #[test]
    fn the_file_name_is_its_own_option() {
        let args = parse(&["t.md", "--title", "Un titulo largo", "--course", "M", "--file-name", "entrega"]);
        assert_eq!(args.file_name.as_deref(), Some("entrega"));
        assert_eq!(
            parse(&["t.md", "--title", "T", "--course", "M", "--nombre", "x"]).file_name.as_deref(),
            Some("x")
        );
    }

    #[test]
    fn alumno_is_not_required() {
        assert!(REQUIRED_ENV.iter().all(|names| names[0] != "ALUMNO"));
    }
}
