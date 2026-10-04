//! Línea de comandos: traduce argumentos y perfil a un `DocumentData`, llama
//! al generador e imprime. Toda la lógica vive en los demás módulos.

use std::ffi::OsString;
use std::path::PathBuf;

use clap::error::{ContextKind, ContextValue, ErrorKind};
use clap::{Arg, ArgAction, CommandFactory, FromArgMatches, Parser};

use crate::courses::{Course, CourseProfile, find_course};
use crate::document::{DocumentData, parse_members, today};
use crate::error::Result;
use crate::generate::{GenerateOptions, copy_pdf_to, generate_pdf, missing_data, missing_data_error};
use crate::i18n::{self, Lang, Text};
use crate::logging;
use crate::markdown::{read_markdown, resolve_markdown_path, validate_markdown};
use crate::project::Project;
use crate::settings::Settings;
use crate::template::resolve_layout;

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

    // El nombre viejo `--env-file` sigue valiendo.
    #[arg(long, alias = "env-file")]
    pub settings: Option<PathBuf>,

    #[arg(long, alias = "permitir-latex")]
    pub allow_latex: bool,

    #[arg(long, aliases = ["template", "plantilla", "diseno"])]
    pub design: Option<String>,

    #[arg(long, alias = "formato")]
    pub format: Option<String>,

    #[arg(long = "set", value_parser = parse_field)]
    pub fields: Vec<(String, String)>,

    #[arg(long, alias = "idioma-documento", value_parser = parse_lang_arg)]
    pub doc_lang: Option<Lang>,

    #[arg(long)]
    pub logos: Option<PathBuf>,

    #[arg(long, alias = "idioma", value_parser = parse_lang_arg)]
    pub lang: Option<Lang>,
}

/// `NOMBRE=valor` para un campo propio del diseño; el nombre va en mayúsculas.
fn parse_field(value: &str) -> std::result::Result<(String, String), String> {
    match value.split_once('=') {
        Some((name, field)) if !name.trim().is_empty() => Ok((name.trim().to_uppercase(), field.to_owned())),
        _ => Err(tr!(es: "usa NOMBRE=valor", en: "use NAME=value")),
    }
}

fn parse_lang_arg(value: &str) -> std::result::Result<Lang, String> {
    i18n::parse(value).ok_or_else(|| tr!(es: "usa es o en", en: "use es or en"))
}

/// Ayuda de cada opción: (id del argumento, descripción, nombre del valor).
const ARG_HELP: [(&str, Text, Option<Text>); 18] = [
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
            "Título del trabajo: portada y datos del PDF (no cambia el nombre del archivo)",
            "Title of the paper: cover and PDF metadata (does not change the file name)",
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
            "Perfil de materia de courses/ (p. ej. ia): todos los datos de portada, formato, diseño y carpetas",
            "Course profile from courses/ (e.g. ia): all the cover data, format, design and folders",
        ),
        Some(Text::new("PERFIL", "PROFILE")),
    ),
    (
        "teacher",
        Text::new(
            "Docente. Si se omite, el del perfil; si no hay, la línea no sale",
            "Teacher. Falls back to the profile's; if none, the line is omitted",
        ),
        Some(Text::new("DOCENTE", "TEACHER")),
    ),
    (
        "members",
        Text::new(
            "Integrantes separados por comas: \"Ana Ruiz, Luis Paz\". Si se omite, los del perfil",
            "Team members separated by commas: \"Ana Ruiz, Luis Paz\". Falls back to the profile's",
        ),
        Some(Text::new("INTEGRANTES", "MEMBERS")),
    ),
    (
        "group",
        Text::new(
            "Grupo, p. ej. \"7-A\". Si se omite, el del perfil; si no hay, la línea no sale",
            "Group, e.g. \"7-A\". Falls back to the profile's; if none, the line is omitted",
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
        "settings",
        Text::new(
            "Archivo de ajustes (idioma, logos). Por omisión, settings.toml del proyecto",
            "Settings file (language, logos). Default: the project's settings.toml",
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
        "design",
        Text::new(
            "Diseño (portada y aspecto): un nombre de templates/designs/ o la ruta a un .ltx",
            "Design (cover and look): a name from templates/designs/ or the path to a .ltx file",
        ),
        Some(Text::new("DISEÑO", "DESIGN")),
    ),
    (
        "format",
        Text::new(
            "Formato (la norma): apa7, harvard, mla… Por omisión, el primero que acepta el diseño",
            "Format (the norm): apa7, harvard, mla… Default: the first one the design accepts",
        ),
        Some(Text::new("FORMATO", "FORMAT")),
    ),
    (
        "fields",
        Text::new(
            "Campo propio del diseño, como --set SALON=\"B-204\". Se puede repetir",
            "A design's own field, like --set SALON=\"B-204\". Can be repeated",
        ),
        Some(Text::new("NOMBRE=VALOR", "NAME=VALUE")),
    ),
    (
        "doc_lang",
        Text::new(
            "Idioma del documento (es o en). Por omisión, el del perfil o el del formato",
            "Document language (es or en). Default: the profile's, or the format's",
        ),
        Some(Text::new("IDIOMA", "LANG")),
    ),
    (
        "logos",
        Text::new(
            "Carpeta con logo-universidad.png y logo-facultad.png. Si se omite, LOGOS de settings.toml",
            "Folder with logo-universidad.png and logo-facultad.png. Falls back to LOGOS in settings.toml",
        ),
        Some(Text::new("CARPETA", "FOLDER")),
    ),
    (
        "lang",
        Text::new(
            "Idioma de la interfaz: es o en. Por omisión, el guardado en settings.toml o el del sistema",
            "Interface language: es or en. Default: the one saved in settings.toml, or the system's",
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
                "Genera un trabajo académico en PDF (APA 7, Harvard o MLA 9) a partir de un archivo Markdown.",
                "Generates an academic paper in PDF (APA 7, Harvard or MLA 9) from a Markdown file.",
            )
            .get(),
        )
        .after_help(
            Text::new(
                "Sin argumentos abre el menú interactivo.\nPara revisar un diseño propio: investigacion --check-template <diseño>",
                "Run it without arguments to open the interactive menu.\nTo check your own design: investigacion --check-template <design>",
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

/// Busca `--lang` y `--settings` antes de analizar el resto, para que la
/// ayuda y los errores ya salgan en su idioma.
pub fn prescan_language(argv: &[OsString], project: &Project) -> Lang {
    let mut lang = None;
    let mut settings_file = None;
    let mut iter = argv.iter().skip(1).map(|arg| arg.to_string_lossy().into_owned());
    while let Some(arg) = iter.next() {
        let (flag, inline) = match arg.split_once('=') {
            Some((flag, value)) => (flag.to_owned(), Some(value.to_owned())),
            None => (arg, None),
        };
        match flag.as_str() {
            "--lang" | "--idioma" => lang = inline.or_else(|| iter.next()).as_deref().and_then(i18n::parse),
            "--settings" | "--env-file" => settings_file = inline.or_else(|| iter.next()).map(PathBuf::from),
            _ => {}
        }
    }
    let settings =
        Settings::load(&settings_file.unwrap_or_else(|| project.settings_file())).unwrap_or_default();
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

/// Envuelve al `Reporter` de quien llama y copia cada mensaje al registro,
/// tal como se muestra; así el CLI y la TUI lo tienen sin hacer nada.
struct LoggingReporter<'a>(&'a mut dyn Reporter);

impl Reporter for LoggingReporter<'_> {
    fn warning(&mut self, message: String) {
        logging::warn(format_args!("user warning: {message}"));
        self.0.warning(message);
    }
    fn info(&mut self, message: String) {
        logging::info(format_args!("user message: {message}"));
        self.0.info(message);
    }
}

/// Valor opcional para el registro: `-` si no se dio.
fn shown(value: Option<impl std::fmt::Display>) -> String {
    value.map_or_else(|| "-".to_owned(), |v| v.to_string())
}

/// La opción gana y luego el perfil de la materia.
fn pick(option: Option<&str>, profile: Option<&str>) -> String {
    option.or(profile).map(str::trim).unwrap_or_default().to_owned()
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

/// Ejecuta una generación completa con argumentos ya analizados. Registra el
/// inicio, cada mensaje, el resultado y el tiempo total.
pub fn execute(args: &Args, project: &Project, reporter: &mut dyn Reporter) -> Result<PathBuf> {
    let started = std::time::Instant::now();
    logging::info(format_args!(
        "generation started: markdown={} profile={} design={} format={} doc_lang={} output={}",
        args.markdown.display(),
        shown(args.profile.as_deref()),
        shown(args.design.as_deref()),
        shown(args.format.as_deref()),
        shown(args.doc_lang.map(Lang::code)),
        shown(args.output.as_ref().map(|o| o.display())),
    ));
    let result = run_generation(args, project, &mut LoggingReporter(reporter));
    let seconds = started.elapsed().as_secs_f64();
    match &result {
        Ok(pdf) => logging::info(format_args!("generation finished in {seconds:.2} s: {}", pdf.display())),
        Err(error) => logging::error(format_args!("generation failed after {seconds:.2} s: {error}")),
    }
    result
}

fn run_generation(args: &Args, project: &Project, reporter: &mut dyn Reporter) -> Result<PathBuf> {
    let settings = Settings::load(&args.settings.clone().unwrap_or_else(|| project.settings_file()))?;
    let course = args.profile.as_deref().map(|key| find_course(project, key)).transpose()?;
    if let Some(course) = &course {
        logging::info(format_args!("course profile: {} ({})", course.key, course.profile.name));
    }
    let profile = course.as_ref().map(|c| &c.profile);
    let from_profile =
        |get: fn(&CourseProfile) -> &str| profile.map(get).map(str::trim).filter(|v| !v.is_empty());

    // Diseño y formato: la opción gana, luego el perfil.
    let layout = resolve_layout(
        project,
        args.design.as_deref().or(from_profile(|p| &p.design)),
        args.format.as_deref().or(from_profile(|p| &p.format)),
    )?;
    let doc_lang =
        layout.document_language(args.doc_lang.or(from_profile(|p| &p.language).and_then(i18n::parse)));
    logging::info(format_args!(
        "layout: design {} ({}), format {}, document language {}",
        layout.design.key,
        layout.design.file.display(),
        shown(layout.format.as_ref().map(|f| &f.key)),
        doc_lang.code()
    ));

    // Campos propios: los del perfil y, encima, los de --set.
    let mut fields = profile.map(|p| p.fields.clone()).unwrap_or_default();
    fields = fields.into_iter().map(|(k, v)| (k.to_uppercase(), v)).collect();
    fields.extend(args.fields.iter().cloned());

    // Sin perfil, los datos generales también se dan con --set UNIVERSIDAD=….
    let set = |name: &str| args.fields.iter().rev().find(|(k, _)| k == name).map(|(_, v)| v.as_str());
    let data = DocumentData {
        university: pick(set("UNIVERSIDAD"), from_profile(|p| &p.university)),
        faculty: pick(set("FACULTAD"), from_profile(|p| &p.faculty)),
        student: pick(set("ALUMNO"), from_profile(|p| &p.student)),
        semester: pick(set("SEMESTRE"), from_profile(|p| &p.semester)),
        title: args.title.trim().to_owned(),
        course: pick(args.course.as_deref(), from_profile(|p| &p.name)),
        teacher: pick(args.teacher.as_deref().or(set("DOCENTE")), from_profile(|p| &p.teacher)),
        date: today(doc_lang),
        members: parse_members(&pick(
            args.members.as_deref().or(set("INTEGRANTES")),
            from_profile(|p| &p.members),
        )),
        group: pick(args.group.as_deref().or(set("GRUPO")), from_profile(|p| &p.group)),
        fields,
    };
    // Solo se exigen los datos que el diseño realmente usa.
    let missing = missing_data(&layout, &data);
    if !missing.is_empty() {
        logging::warn(format_args!("missing data for design {}: {}", layout.design.key, missing.join(", ")));
        return Err(missing_data_error(&layout, &missing));
    }

    let markdown_path = locate_markdown(args, project, course.as_ref())?;
    let markdown = read_markdown(&markdown_path)?;
    logging::info(format_args!("markdown: {} ({} bytes)", markdown_path.display(), markdown.len()));
    // Un Markdown vacío ya falla en `generate_pdf`; avisar de sus encabezados sobra.
    if !markdown.trim().is_empty() {
        for warning in validate_markdown(&markdown, &layout.headings()) {
            reporter.warning(warning);
        }
    }

    // Los logos suelen vivir fuera del proyecto: por eso admiten los ajustes.
    let logos = args.logos.clone().or_else(|| {
        let from_env = settings.get(&["LOGOS"]);
        (!from_env.is_empty()).then(|| PathBuf::from(from_env))
    });
    let options = GenerateOptions {
        design: Some(layout.design.file.display().to_string()),
        format: layout.format.as_ref().map(|f| f.key.clone()),
        doc_lang: Some(doc_lang),
        allow_raw_latex: args.allow_latex,
        logos,
        file_name: args.file_name.clone(),
        markdown: Some(markdown),
    };
    let output_dir = args.output.clone().unwrap_or_else(|| match &course {
        Some(course) => course.output_dir(project),
        None => project.output_dir(),
    });
    logging::debug(format_args!(
        "output folder: {}, logos: {}, raw LaTeX: {}, copies: {}",
        output_dir.display(),
        shown(options.logos.as_ref().map(|l| l.display())),
        args.allow_latex,
        args.copy.len()
    ));
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
    // `--check-template <diseño>` revisa un diseño en vez de generar un trabajo.
    // Vale `--check-template x` y `--check-template=x`; una opción no es un nombre.
    let check = argv.iter().enumerate().find_map(|(i, arg)| {
        let arg = arg.to_string_lossy();
        let (flag, inline) =
            arg.split_once('=').map_or((arg.as_ref(), None), |(f, v)| (f, Some(v.to_owned())));
        matches!(flag, "--check-template" | "--revisar-plantilla").then(|| {
            inline
                .or_else(|| argv.get(i + 1).map(|a| a.to_string_lossy().into_owned()))
                .filter(|d| !d.starts_with('-'))
        })
    });
    if let Some(design) = check {
        let Some(design) = design else {
            logging::warn("--check-template without a design name");
            eprintln!(
                "{}",
                tr!(
                    es: "error: falta el nombre del diseño: --check-template <diseño>",
                    en: "error: missing the design name: --check-template <design>"
                )
            );
            return 2;
        };
        return crate::check::run_cli(&project, &design);
    }
    let args = match parse_args(argv) {
        Ok(args) => args,
        Err(error) => {
            match error.kind() {
                ErrorKind::DisplayHelp | ErrorKind::DisplayVersion => {
                    logging::debug(format_args!("shown: {:?}", error.kind()));
                    let _ = error.print();
                }
                _ => {
                    let message = describe_clap_error(&error);
                    logging::warn(format_args!("invalid arguments:\n{message}"));
                    eprintln!("{message}");
                }
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
        std::fs::write(project.settings_file(), "IDIOMA=en\n").unwrap();
        assert_eq!(prescan_language(&argv(&["investigacion", "--help"]), &project), Lang::En);
        let other = directory.path().join("otro.toml");
        std::fs::write(&other, "IDIOMA=es\n").unwrap();
        let with_file = argv(&["investigacion", "--settings", other.to_str().unwrap()]);
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
        assert_eq!(args.design.as_deref(), Some("apa"));
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
    fn design_format_fields_and_document_language() {
        let args = parse(&[
            "t.md",
            "--title",
            "T",
            "--design",
            "report",
            "--format",
            "mla",
            "--set",
            "salon=B-204",
            "--set",
            "AULA=",
            "--doc-lang",
            "en",
        ]);
        assert_eq!((args.design.as_deref(), args.format.as_deref()), (Some("report"), Some("mla")));
        assert_eq!(
            args.fields,
            [("SALON".to_owned(), "B-204".to_owned()), ("AULA".to_owned(), String::new())]
        );
        assert_eq!(args.doc_lang, Some(Lang::En));
        // El nombre antiguo de la opción sigue valiendo.
        assert_eq!(parse(&["t.md", "--title", "T", "--template", "apa"]).design.as_deref(), Some("apa"));
    }
}
