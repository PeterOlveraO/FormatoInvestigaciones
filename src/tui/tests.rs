//! Pruebas del menú sin terminal real: teclas simuladas y `TestBackend`.

use std::path::Path;

use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use ratatui::Terminal;
use ratatui::backend::TestBackend;

use super::app::{
    App, FIELDS, FieldKey, HOME_ITEMS, HomeAction, LogLine, Mode, OPTIONS, OptionAction, index_of,
};
use super::picker::PickerPurpose;
use super::tools::{self, Tool};
use super::ui;
use crate::i18n::{self, Lang};
use crate::project::Project;
use crate::settings::Settings;

fn press(app: &mut App, code: KeyCode) {
    app.handle_key(KeyEvent::new(code, KeyModifiers::NONE));
}

fn type_text(app: &mut App, text: &str) {
    for c in text.chars() {
        press(app, KeyCode::Char(c));
    }
}

fn select(app: &mut App, key: FieldKey) {
    app.selected = index_of(key);
}

/// Proyecto de juguete: dos plantillas, un perfil y trabajos en input/IA.
fn sample_project() -> (tempfile::TempDir, Project) {
    let directory = tempfile::tempdir().unwrap();
    let root = directory.path();
    let write = |path: &str, text: &str| {
        let path = root.join(path);
        std::fs::create_dir_all(path.parent().unwrap()).unwrap();
        std::fs::write(path, text).unwrap();
    };
    // Dos formatos y dos diseños: uno solo para APA, con un campo propio
    // obligatorio, y otro que combina con los dos y pide pocos datos.
    for format in ["apa7", "harvard"] {
        write(&format!("templates/formats/{format}/format.sty"), "x");
        write(&format!("templates/formats/{format}/format.toml"), &format!("name = \"{format}\"\n"));
    }
    write(
        "templates/designs/geometric-cover/template.ltx",
        "%%FORMAT%%\n%%TITULO%% %%UNIVERSIDAD%% %%MATERIA%% %%DOCENTE%% %%SALON%%\n%%CONTENIDO_MARKDOWN%%\n",
    );
    write(
        "templates/designs/geometric-cover/template.toml",
        "formats = [\"apa7\"]\n[fields.SALON]\nlabel = { es = \"Salón\", en = \"Room\" }\nrequired = true\n",
    );
    write("templates/designs/classic-cover/template.ltx", "%%FORMAT%%\n%%TITULO%%\n%%CONTENIDO_MARKDOWN%%\n");
    write("templates/designs/classic-cover/template.toml", "formats = [\"apa7\", \"harvard\"]\n");
    std::fs::create_dir_all(root.join("input/IA")).unwrap();
    std::fs::write(root.join("input/IA/Tarea 1.md"), "# Introducción\n").unwrap();
    std::fs::write(root.join("input/suelto.md"), "# Introducción\n").unwrap();
    std::fs::create_dir_all(root.join("courses")).unwrap();
    std::fs::write(
        root.join("courses/ia.toml"),
        "name = \"Inteligencia artificial\"\nteacher = \"Docente IA\"\ngroup = \"M\"\ntemplate = \"apa-simple\"\nfolder = \"IA\"\n",
    )
    .unwrap();
    let project = Project::at(root);
    (directory, project)
}

fn render(app: &App) -> String {
    render_sized(app, 120, 32)
}

fn render_sized(app: &App, width: u16, height: u16) -> String {
    let mut terminal = Terminal::new(TestBackend::new(width, height)).unwrap();
    terminal.draw(|frame| ui::draw(frame, app)).unwrap();
    let buffer = terminal.backend().buffer().clone();
    buffer
        .content()
        .chunks(buffer.area.width as usize)
        .map(|row| row.iter().map(|c| c.symbol()).collect::<String>() + "\n")
        .collect()
}

/// Las dos últimas filas de la pantalla: la barra de teclas.
fn footer(screen: &str) -> String {
    let rows: Vec<&str> = screen.lines().collect();
    rows[rows.len() - 2..].join("\n")
}

fn is_folder_picker(app: &App) -> bool {
    matches!(&app.mode, Mode::Picking(picker) if picker.purpose == PickerPurpose::Folder)
}

#[test]
fn the_removed_options_are_not_in_the_menu() {
    let labels: Vec<&str> = FIELDS.iter().flat_map(|f| [f.label.es, f.label.en]).collect();
    for removed in [".env", "LaTeX", "Logos", "logos"] {
        assert!(labels.iter().all(|l| !l.contains(removed)), "{removed}");
    }
}

#[test]
fn the_menu_is_drawn_in_the_current_language() {
    let (_dir, project) = sample_project();
    let app = App::new(project);
    i18n::set(Lang::Es);
    let screen = render(&app);
    assert!(
        screen.contains("Archivo Markdown")
            && screen.contains("Nombre del PDF")
            && screen.contains("[falta]")
    );
    i18n::set(Lang::En);
    let screen = render(&app);
    assert!(
        screen.contains("Markdown file") && screen.contains("PDF file name") && screen.contains("[missing]")
    );
}

#[test]
fn the_first_run_asks_for_the_language_saves_it_and_opens_home() {
    let (_dir, project) = sample_project();
    let env = project.env_file();
    let mut app = App::new(project);
    i18n::set(Lang::Es);
    app.start(Some(Lang::Es));
    assert!(matches!(app.mode, Mode::ChooseLanguage(Lang::Es)));
    assert!(render(&app).contains("Idioma / Language"));
    press(&mut app, KeyCode::Down);
    assert!(matches!(app.mode, Mode::ChooseLanguage(Lang::En)));
    press(&mut app, KeyCode::Enter);
    assert!(matches!(app.mode, Mode::Home(0)), "con perfiles no hace falta el asistente");
    assert_eq!(i18n::current(), Lang::En);
    assert_eq!(i18n::configured(&Settings::load(&env).unwrap()), Some(Lang::En));
    assert!(render(&app).contains("Generate a PDF"));
}

#[test]
fn with_profiles_and_a_saved_language_the_menu_opens_at_home() {
    i18n::set(Lang::Es);
    let (_dir, project) = sample_project();
    let mut app = App::new(project);
    app.start(None);
    assert!(matches!(app.mode, Mode::Home(0)));
    let screen = render(&app);
    for expected in ["Generar un PDF", "Nuevo perfil", "Opciones", "Salir", "APA 7, Harvard o MLA"] {
        assert!(screen.contains(expected), "{expected} en:\n{screen}");
    }
    assert!(screen.contains("Perfiles (1): ia"));
    let footer = footer(&screen);
    assert!(
        footer.contains("mover") && footer.contains("elegir") && footer.contains(" s  salir"),
        "{footer}"
    );
    assert!(!footer.contains("generar"));
}

#[test]
fn without_profiles_the_wizard_opens_and_then_home() {
    i18n::set(Lang::En);
    let (_dir, project) = sample_project();
    std::fs::remove_file(project.courses_dir().join("ia.toml")).unwrap();
    let mut app = App::new(project);
    app.start(None);
    assert!(matches!(app.mode, Mode::Wizard(_)));
    press(&mut app, KeyCode::Esc);
    assert!(matches!(app.mode, Mode::Home(0)));
    assert!(render(&app).contains("none yet"), "el inicio sugiere crear un perfil");
}

#[test]
fn escape_uses_the_language_without_saving_it() {
    let (_dir, project) = sample_project();
    let env = project.env_file();
    let mut app = App::new(project);
    app.ask_language(Lang::En);
    press(&mut app, KeyCode::Esc);
    assert_eq!(i18n::current(), Lang::En);
    assert!(!env.exists());
}

#[test]
fn the_l_key_switches_the_language_and_saves_it() {
    let (_dir, project) = sample_project();
    let env = project.env_file();
    std::fs::write(&env, "UNIVERSIDAD=\"U\"\nIDIOMA=\"es\"\n").unwrap();
    let mut app = App::new(project);
    i18n::set(Lang::Es);
    press(&mut app, KeyCode::Char('l'));
    assert_eq!(i18n::current(), Lang::En);
    let text = std::fs::read_to_string(&env).unwrap();
    assert_eq!(text, "UNIVERSIDAD=\"U\"\nIDIOMA=\"en\"\n");
    press(&mut app, KeyCode::Char('l'));
    assert_eq!(i18n::current(), Lang::Es);
}

#[test]
fn choosing_a_profile_fills_the_form_and_the_markdown_picker_starts_in_its_folder() {
    let (_dir, project) = sample_project();
    let root = project.root.clone();
    let mut app = App::new(project);

    select(&mut app, FieldKey::Profile);
    press(&mut app, KeyCode::Enter);
    type_text(&mut app, "ia");
    press(&mut app, KeyCode::Enter);
    assert_eq!(app.value(FieldKey::Course), "Inteligencia artificial");
    assert_eq!(app.value(FieldKey::Teacher), "Docente IA");
    assert_eq!(app.value(FieldKey::Template), "apa-simple");
    assert_eq!(Path::new(app.value(FieldKey::Output)), Path::new("output/IA"));

    select(&mut app, FieldKey::Markdown);
    press(&mut app, KeyCode::Enter);
    let Mode::Picking(picker) = &app.mode else { panic!("the picker did not open") };
    assert_eq!(picker.dir.as_deref(), Some(root.join("input/IA").as_path()));
    assert!(render(&app).contains("Tarea 1.md"));
    type_text(&mut app, "tarea");
    press(&mut app, KeyCode::Enter);
    assert!(matches!(app.mode, Mode::Form));
    assert_eq!(Path::new(app.value(FieldKey::Markdown)), root.join("input/IA/Tarea 1.md"));
    // El nombre del PDF se propone a partir del Markdown, no del título.
    assert_eq!(app.value(FieldKey::FileName), "tarea-1");
}

#[test]
fn the_picker_browses_folders_without_typing_paths() {
    let (_dir, project) = sample_project();
    let root = project.root.clone();
    let mut app = App::new(project);
    select(&mut app, FieldKey::Markdown);
    press(&mut app, KeyCode::Enter);
    // En input/: ../, IA/, suelto.md. Bajar a IA/ y entrar.
    press(&mut app, KeyCode::Down);
    press(&mut app, KeyCode::Enter);
    press(&mut app, KeyCode::Down);
    press(&mut app, KeyCode::Enter);
    assert_eq!(Path::new(app.value(FieldKey::Markdown)), root.join("input/IA/Tarea 1.md"));
}

#[test]
fn a_typed_file_name_is_not_overwritten_by_the_next_markdown() {
    let (_dir, project) = sample_project();
    let mut app = App::new(project);
    select(&mut app, FieldKey::FileName);
    press(&mut app, KeyCode::Enter);
    type_text(&mut app, "entrega-final");
    press(&mut app, KeyCode::Enter);
    select(&mut app, FieldKey::Markdown);
    press(&mut app, KeyCode::Enter);
    type_text(&mut app, "suelto");
    press(&mut app, KeyCode::Enter);
    assert_eq!(app.value(FieldKey::FileName), "entrega-final");
}

#[test]
fn text_editing_supports_cursor_and_cancel() {
    let (_dir, project) = sample_project();
    let mut app = App::new(project);
    select(&mut app, FieldKey::Title);
    press(&mut app, KeyCode::Enter);
    type_text(&mut app, "Álgebra");
    press(&mut app, KeyCode::Home);
    type_text(&mut app, "1 ");
    press(&mut app, KeyCode::End);
    press(&mut app, KeyCode::Backspace);
    press(&mut app, KeyCode::Enter);
    assert_eq!(app.value(FieldKey::Title), "1 Álgebr");
    press(&mut app, KeyCode::Enter);
    type_text(&mut app, "xyz");
    press(&mut app, KeyCode::Esc);
    assert_eq!(app.value(FieldKey::Title), "1 Álgebr");
}

#[test]
fn generation_requires_the_mandatory_fields() {
    i18n::set(Lang::En);
    let (_dir, project) = sample_project();
    let mut app = App::new(project);
    press(&mut app, KeyCode::Char('g'));
    assert!(matches!(app.mode, Mode::Form));
    assert!(matches!(app.log.last(), Some(LogLine::Error(e)) if e.contains("Markdown file")));
}

#[test]
fn the_form_becomes_the_same_arguments_as_the_cli() {
    let (_dir, project) = sample_project();
    let root = project.root.clone();
    let mut app = App::new(project);
    app.values[index_of(FieldKey::Markdown)] = "a.md".into();
    app.values[index_of(FieldKey::Title)] = "T".into();
    app.values[index_of(FieldKey::Course)] = "M".into();
    app.values[index_of(FieldKey::Copies)] = "x; ; y".into();
    app.values[index_of(FieldKey::Output)] = "output/IA".into();
    let args = app.build_args();
    assert_eq!(args.title, "T");
    assert_eq!(args.course.as_deref(), Some("M"));
    // Lo vacío no se pasa, para que valgan el perfil y el .env.
    assert!(args.teacher.is_none() && args.group.is_none() && args.file_name.is_none());
    assert_eq!(args.copy.len(), 2);
    assert_eq!(args.output, Some(root.join("output/IA")));
    assert!(!args.allow_latex && args.logos.is_none() && args.env_file.is_none());
}

#[test]
fn quitting() {
    let (_dir, project) = sample_project();
    let mut app = App::new(project);
    press(&mut app, KeyCode::Char('s'));
    assert!(app.should_quit);
    let (_dir, project) = sample_project();
    let mut app = App::new(project);
    press(&mut app, KeyCode::Char('q'));
    assert!(app.should_quit);
}

#[test]
fn the_wizard_creates_a_profile_with_the_data_of_the_design() {
    i18n::set(Lang::Es);
    let (_dir, project) = sample_project();
    std::fs::remove_file(project.courses_dir().join("ia.toml")).unwrap();
    let mut app = App::new(project.clone());
    app.start(None);
    assert!(matches!(app.mode, Mode::Wizard(_)), "sin perfiles se abre el asistente");

    type_text(&mut app, "Bases de Datos");
    press(&mut app, KeyCode::Enter);
    type_text(&mut app, "apa7");
    press(&mut app, KeyCode::Enter);
    // Con apa7 se ofrecen los dos diseños; se elige el que tiene campo propio.
    type_text(&mut app, "geometric");
    press(&mut app, KeyCode::Enter);
    // Pasos: universidad*, materia*, docente, salón* y carpeta.
    press(&mut app, KeyCode::Enter);
    let Mode::Wizard(wizard) = &app.mode else { panic!("el asistente se cerró") };
    assert!(wizard.error.is_some(), "la universidad es obligatoria");
    for answer in ["Universidad Norte", "Bases de datos", "", "B-204", "BD"] {
        type_text(&mut app, answer);
        press(&mut app, KeyCode::Enter);
    }
    assert!(matches!(app.mode, Mode::Home(0)), "el asistente vuelve a la vista de la que salió");
    let saved = std::fs::read_to_string(project.courses_dir().join("bases-de-datos.toml")).unwrap();
    for expected in [
        "name = \"Bases de datos\"",
        "university = \"Universidad Norte\"",
        "format = \"apa7\"",
        "design = \"geometric-cover\"",
        "folder = \"BD\"",
        "SALON = \"B-204\"",
    ] {
        assert!(saved.contains(expected), "{expected} en:\n{saved}");
    }
    assert!(!saved.contains("teacher"), "lo vacío no se guarda");
    // Se aplica al formulario, con el campo propio lleno.
    assert_eq!(app.value(FieldKey::Profile), "bases-de-datos");
    assert_eq!(app.extras[0].value, "B-204");
}

#[test]
fn the_first_run_asks_the_language_and_then_opens_the_wizard() {
    let (_dir, project) = sample_project();
    std::fs::remove_file(project.courses_dir().join("ia.toml")).unwrap();
    let mut app = App::new(project);
    app.start(Some(Lang::Es));
    assert!(matches!(app.mode, Mode::ChooseLanguage(_)));
    // Las flechas solo cambian la marca; antes abrían el asistente sin elegir.
    press(&mut app, KeyCode::Down);
    press(&mut app, KeyCode::Up);
    assert!(matches!(app.mode, Mode::ChooseLanguage(Lang::Es)));
    press(&mut app, KeyCode::Enter);
    assert!(matches!(app.mode, Mode::Wizard(_)));
    press(&mut app, KeyCode::Esc);
    assert!(matches!(app.mode, Mode::Home(0)));
}

#[test]
fn designs_are_filtered_by_format_and_custom_fields_appear() {
    i18n::set(Lang::Es);
    let (_dir, project) = sample_project();
    let mut app = App::new(project);
    // Con harvard solo se ofrece el diseño que lo acepta.
    select(&mut app, FieldKey::Format);
    press(&mut app, KeyCode::Enter);
    type_text(&mut app, "harvard");
    press(&mut app, KeyCode::Enter);
    select(&mut app, FieldKey::Template);
    press(&mut app, KeyCode::Enter);
    let Mode::Picking(picker) = &app.mode else { panic!() };
    let labels: Vec<&str> = picker.visible().iter().map(|i| i.label.as_str()).collect();
    assert_eq!(labels, ["classic-cover"]);
    press(&mut app, KeyCode::Esc);
    // El diseño con %%SALON%% suma un campo obligatorio al formulario.
    app.values[index_of(FieldKey::Format)].clear();
    select(&mut app, FieldKey::Template);
    press(&mut app, KeyCode::Enter);
    type_text(&mut app, "geometric");
    press(&mut app, KeyCode::Enter);
    assert_eq!(app.extras.len(), 1);
    assert_eq!(app.field_label(FIELDS.len()), "Salón");
    assert!(app.missing_fields().iter().any(|m| m == "Salón"));
    assert!(render(&app).contains("Salón"));
    app.selected = FIELDS.len();
    press(&mut app, KeyCode::Enter);
    type_text(&mut app, "B-204");
    press(&mut app, KeyCode::Enter);
    assert_eq!(app.build_args().fields, [("SALON".to_owned(), "B-204".to_owned())]);
}

#[test]
fn p_edits_the_chosen_profile_with_its_data() {
    i18n::set(Lang::Es);
    let (_dir, project) = sample_project();
    let mut app = App::new(project);
    select(&mut app, FieldKey::Profile);
    press(&mut app, KeyCode::Enter);
    type_text(&mut app, "ia");
    press(&mut app, KeyCode::Enter);
    press(&mut app, KeyCode::Char('p'));
    let Mode::Wizard(wizard) = &app.mode else { panic!("p no abrió el asistente") };
    assert!(wizard.editing);
    assert_eq!(wizard.input.text, "ia");
    assert!(render(&app).contains("Editar perfil"));
}

#[test]
fn the_home_menu_wraps_around_and_opens_each_item() {
    i18n::set(Lang::Es);
    let (_dir, project) = sample_project();
    let mut app = App::new(project);
    app.start(None);
    press(&mut app, KeyCode::Up);
    assert!(matches!(app.mode, Mode::Home(3)));
    press(&mut app, KeyCode::Down);
    assert!(matches!(app.mode, Mode::Home(0)));
    // Con un perfil elegido, «Nuevo perfil» abre el asistente vacío igual.
    app.values[index_of(FieldKey::Profile)] = "ia".into();
    for (index, item) in HOME_ITEMS.iter().enumerate() {
        app.mode = Mode::Home(index);
        press(&mut app, KeyCode::Enter);
        match item.action {
            HomeAction::Generate => {
                assert!(matches!(app.mode, Mode::Form));
                press(&mut app, KeyCode::Esc);
                assert!(matches!(app.mode, Mode::Home(0)), "Esc en el formulario vuelve al inicio");
            }
            HomeAction::NewProfile => {
                let Mode::Wizard(wizard) = &app.mode else { panic!("no se abrió el asistente") };
                assert!(!wizard.editing && wizard.input.text.is_empty());
                assert!(render(&app).contains("Nuevo perfil"));
                press(&mut app, KeyCode::Esc);
                assert!(matches!(app.mode, Mode::Home(i) if i == index));
            }
            HomeAction::Options => {
                assert!(matches!(app.mode, Mode::Options(0)));
                assert!(render(&app).contains("Volver al inicio"));
                press(&mut app, KeyCode::Esc);
                assert!(matches!(app.mode, Mode::Home(i) if i == index), "vuelve a donde se abrió");
            }
            HomeAction::Quit => assert!(app.should_quit),
        }
    }
    let (_dir, project) = sample_project();
    let mut app = App::new(project);
    app.start(None);
    press(&mut app, KeyCode::Char('q'));
    assert!(app.should_quit);
}

#[test]
fn each_option_runs_with_enter_and_with_its_letters() {
    for (index, option) in OPTIONS.iter().enumerate() {
        for letter in [None, Some(option.keys[0]), Some(option.keys[1])] {
            i18n::set(Lang::Es);
            let (_dir, project) = sample_project();
            let mut app = App::new(project);
            press(&mut app, KeyCode::Char('o'));
            assert!(matches!(app.mode, Mode::Options(0)));
            match letter {
                None => {
                    for _ in 0..index {
                        press(&mut app, KeyCode::Down);
                    }
                    press(&mut app, KeyCode::Enter);
                }
                Some(c) => press(&mut app, KeyCode::Char(c)),
            }
            let how = format!("{:?} con {letter:?}", option.action);
            match option.action {
                OptionAction::EditProfile | OptionAction::NewProfile => {
                    let Mode::Wizard(wizard) = &app.mode else { panic!("{how}") };
                    assert!(!wizard.editing, "sin perfil elegido se crea uno: {how}");
                    press(&mut app, KeyCode::Esc);
                    assert!(matches!(app.mode, Mode::Form), "{how}");
                }
                OptionAction::Folders => {
                    assert!(is_folder_picker(&app), "{how}");
                    press(&mut app, KeyCode::Esc);
                    assert!(matches!(app.mode, Mode::Form), "{how}");
                }
                OptionAction::Language => {
                    assert_eq!(i18n::current(), Lang::En, "{how}");
                    assert!(matches!(app.mode, Mode::Options(i) if i == index), "se queda en las opciones");
                    assert!(render(&app).contains("Español (menu language)"));
                }
                // En las pruebas no hay registro: se avisa y no se abre nada.
                OptionAction::Log => {
                    assert!(matches!(app.mode, Mode::Options(i) if i == index), "{how}");
                    assert!(
                        matches!(app.log.last(), Some(LogLine::Warning(w)) if w.contains("registro")),
                        "{how}"
                    );
                }
                OptionAction::Home => assert!(matches!(app.mode, Mode::Home(0)), "{how}"),
            }
        }
    }
}

#[test]
fn the_options_view_wraps_around_and_escape_closes_it() {
    i18n::set(Lang::Es);
    let (_dir, project) = sample_project();
    let mut app = App::new(project);
    press(&mut app, KeyCode::Char('o'));
    press(&mut app, KeyCode::Up);
    assert!(matches!(app.mode, Mode::Options(i) if i == OPTIONS.len() - 1));
    press(&mut app, KeyCode::Down);
    assert!(matches!(app.mode, Mode::Options(0)));
    // Una letra que no es de ninguna fila no hace nada.
    press(&mut app, KeyCode::Char('z'));
    assert!(matches!(app.mode, Mode::Options(0)));
    press(&mut app, KeyCode::Esc);
    assert!(matches!(app.mode, Mode::Form));
}

#[test]
fn editing_the_profile_from_the_options_uses_the_chosen_one() {
    i18n::set(Lang::Es);
    let (_dir, project) = sample_project();
    let mut app = App::new(project);
    select(&mut app, FieldKey::Profile);
    press(&mut app, KeyCode::Enter);
    type_text(&mut app, "ia");
    press(&mut app, KeyCode::Enter);
    press(&mut app, KeyCode::Char('o'));
    let screen = render(&app);
    assert!(screen.contains("Editar el perfil elegido  ia"), "{screen}");
    press(&mut app, KeyCode::Enter);
    let Mode::Wizard(wizard) = &app.mode else { panic!("no se abrió el asistente") };
    assert!(wizard.editing);
    // «Nuevo perfil» empieza vacío aunque haya uno elegido.
    press(&mut app, KeyCode::Esc);
    press(&mut app, KeyCode::Char('o'));
    press(&mut app, KeyCode::Char('n'));
    let Mode::Wizard(wizard) = &app.mode else { panic!("no se abrió el asistente") };
    assert!(!wizard.editing && wizard.input.text.is_empty());
}

#[test]
fn windows_opened_from_home_go_back_home() {
    i18n::set(Lang::En);
    let (_dir, project) = sample_project();
    let mut app = App::new(project);
    app.start(None);
    let options = HOME_ITEMS.iter().position(|i| i.action == HomeAction::Options).unwrap();
    app.mode = Mode::Home(options);
    press(&mut app, KeyCode::Enter);
    assert!(render(&app).contains("Back to the home view"), "las opciones se ven sobre el inicio");
    press(&mut app, KeyCode::Char('f'));
    assert!(is_folder_picker(&app));
    press(&mut app, KeyCode::Esc);
    assert!(matches!(app.mode, Mode::Home(i) if i == options));
    press(&mut app, KeyCode::Enter);
    press(&mut app, KeyCode::Char('n'));
    press(&mut app, KeyCode::Esc);
    assert!(matches!(app.mode, Mode::Home(i) if i == options));
}

#[test]
fn the_form_footer_only_shows_the_main_keys() {
    let (_dir, project) = sample_project();
    let app = App::new(project);
    i18n::set(Lang::Es);
    let shown = footer(&render(&app));
    for key in
        ["↑↓", "mover", "editar/elegir", "Supr", "vaciar", "generar", "ver PDF", " o  opciones", " s  salir"]
    {
        assert!(shown.contains(key), "{key} en:\n{shown}");
    }
    for removed in ["perfil", "carpetas", "English", " p ", " c ", " l "] {
        assert!(!shown.contains(removed), "{removed} en:\n{shown}");
    }
    i18n::set(Lang::En);
    let shown = footer(&render(&app));
    for key in ["move", "edit/choose", "Del", "clear", "generate", "view PDF", " o  options", " q  quit"] {
        assert!(shown.contains(key), "{key} in:\n{shown}");
    }
    for removed in ["profile", "folders", "Español", " p ", " f ", " l "] {
        assert!(!shown.contains(removed), "{removed} in:\n{shown}");
    }
}

#[test]
fn the_options_show_each_language_letters() {
    let (_dir, project) = sample_project();
    let mut app = App::new(project);
    press(&mut app, KeyCode::Char('o'));
    i18n::set(Lang::Es);
    let screen = render(&app);
    for row in [
        " p  Editar el perfil elegido",
        " c  Abrir una carpeta",
        " l  English (idioma del menú)",
        " i  Volver",
    ] {
        assert!(screen.contains(row), "{row} en:\n{screen}");
    }
    assert!(footer(&screen).contains("Esc  volver"));
    // La ventana se ensancha para que la fila más larga quepa en 80 columnas.
    let narrow = render_sized(&app, 80, 24);
    assert!(narrow.contains("Editar el perfil elegido  ninguno elegido: crea uno"), "{narrow}");
    i18n::set(Lang::En);
    let screen = render(&app);
    for row in [" n  New profile", " f  Open a project folder", " l  Español (menu language)", " h  Back"] {
        assert!(screen.contains(row), "{row} in:\n{screen}");
    }
}

#[test]
fn the_old_letters_still_work_hidden_in_the_form() {
    i18n::set(Lang::Es);
    let (_dir, project) = sample_project();
    let mut app = App::new(project);
    for letter in ['c', 'f'] {
        press(&mut app, KeyCode::Char(letter));
        assert!(is_folder_picker(&app), "{letter}");
        press(&mut app, KeyCode::Esc);
        assert!(matches!(app.mode, Mode::Form));
    }
    for letter in ['p', 'n'] {
        press(&mut app, KeyCode::Char(letter));
        assert!(matches!(app.mode, Mode::Wizard(_)), "{letter}");
        press(&mut app, KeyCode::Esc);
        assert!(matches!(app.mode, Mode::Form), "el asistente vuelve al formulario");
    }
    press(&mut app, KeyCode::F(5));
    assert!(matches!(app.log.last(), Some(LogLine::Error(_))), "F5 genera (y avisa lo que falta)");
}

#[test]
fn editing_a_field_of_the_design_does_not_crash() {
    i18n::set(Lang::Es);
    let (_dir, project) = sample_project();
    let mut app = App::new(project);
    select(&mut app, FieldKey::Template);
    press(&mut app, KeyCode::Enter);
    type_text(&mut app, "geometric");
    press(&mut app, KeyCode::Enter);
    app.selected = FIELDS.len();
    press(&mut app, KeyCode::Enter);
    assert!(matches!(app.mode, Mode::Editing(_)));
    // Antes la ventana indexaba FIELDS con el número del campo propio y fallaba.
    let screen = render(&app);
    assert_eq!(screen.matches("╭ Salón ").count(), 2, "ayuda y ventana de edición:\n{screen}");
    render_sized(&app, 80, 40);
    press(&mut app, KeyCode::Esc);
    // En una terminal angosta el formulario tiene alto para los campos propios.
    app.selected = 0;
    assert!(render_sized(&app, 80, 40).contains("Salón*"));
}

#[test]
fn home_shows_the_last_pdf_and_the_missing_tools() {
    i18n::set(Lang::En);
    let (_dir, project) = sample_project();
    let root = project.root.clone();
    let mut app = App::new(project);
    app.start(None);
    app.last_pdf = Some(root.join("output").join("IA").join("tarea-1.pdf"));
    app.tools = vec![
        Tool { name: "Pandoc", required: true, found: false },
        Tool { name: "pdflatex", required: true, found: true },
        Tool { name: "Graphviz (dot)", required: false, found: false },
    ];
    let screen = render(&app);
    let pdf = Path::new("output").join("IA").join("tarea-1.pdf");
    for expected in [
        "Profiles (1): ia",
        &pdf.display().to_string(),
        "✗ Pandoc: not installed (required)",
        "✓ pdflatex",
        "Graphviz (dot): not installed",
        "INSTALL.md",
    ] {
        assert!(screen.contains(expected), "{expected} in:\n{screen}");
    }
    app.tools.iter_mut().for_each(|t| t.found = true);
    let screen = render(&app);
    assert!(!screen.contains("INSTALL.md") && screen.contains("✓ Graphviz (dot) (optional)"));
}

#[test]
fn a_missing_program_is_reported_as_not_installed() {
    assert!(!tools::is_installed("investigacion-no-existe-esta-herramienta", "--version"));
}

#[test]
fn small_terminals_do_not_panic() {
    i18n::set(Lang::Es);
    let (_dir, project) = sample_project();
    let mut app = App::new(project);
    app.tools = vec![Tool { name: "Pandoc", required: true, found: false }];
    app.log.push(LogLine::Warning("aviso de prueba con un texto bastante largo".into()));
    // Un diseño con campo propio, para el formulario y su edición.
    select(&mut app, FieldKey::Template);
    press(&mut app, KeyCode::Enter);
    type_text(&mut app, "geometric");
    press(&mut app, KeyCode::Enter);
    let check = |app: &App| {
        for (width, height) in [(120, 32), (80, 24), (40, 12), (20, 6), (5, 2)] {
            render_sized(app, width, height);
        }
    };
    app.start(None);
    check(&app);
    press(&mut app, KeyCode::Down);
    press(&mut app, KeyCode::Down);
    press(&mut app, KeyCode::Enter);
    assert!(matches!(app.mode, Mode::Options(0)));
    check(&app);
    press(&mut app, KeyCode::Char('n'));
    check(&app);
    press(&mut app, KeyCode::Esc);
    app.mode = Mode::Home(0);
    press(&mut app, KeyCode::Enter);
    check(&app);
    app.selected = FIELDS.len();
    check(&app);
    press(&mut app, KeyCode::Enter);
    check(&app);
    press(&mut app, KeyCode::Esc);
    press(&mut app, KeyCode::Char('o'));
    check(&app);
    press(&mut app, KeyCode::Char('c'));
    check(&app);
    app.ask_language(Lang::Es);
    check(&app);
}

#[test]
fn v_on_home_reports_that_there_is_no_pdf_yet() {
    i18n::set(Lang::En);
    let (_dir, project) = sample_project();
    let mut app = App::new(project);
    app.start(None);
    press(&mut app, KeyCode::Char('v'));
    assert!(matches!(app.mode, Mode::Home(0)));
    assert!(matches!(app.log.last(), Some(LogLine::Warning(w)) if w.contains("no PDF yet")));
    assert!(render(&app).contains("no PDF yet"), "los mensajes se ven en el inicio");
}

#[test]
fn the_help_does_not_offer_an_empty_value_for_a_required_course() {
    i18n::set(Lang::En);
    let (_dir, project) = sample_project();
    let mut app = App::new(project);
    // El diseño de omisión usa %%MATERIA%%: la materia es obligatoria.
    select(&mut app, FieldKey::Course);
    assert!(app.is_required(app.selected));
    assert!(!render(&app).contains("If empty:"));
    select(&mut app, FieldKey::Teacher);
    assert!(render(&app).contains("If empty:"));
}

#[test]
fn a_recovered_panic_closes_the_window_and_keeps_the_menu() {
    i18n::set(Lang::Es);
    let (_dir, project) = sample_project();
    let mut app = App::new(project);
    press(&mut app, KeyCode::Char('o'));
    assert!(matches!(app.mode, Mode::Options(_)));
    app.recover("índice fuera de rango".into());
    assert!(matches!(app.mode, Mode::Form));
    assert!(matches!(app.log.last(), Some(LogLine::Error(e)) if e.contains("índice fuera de rango")));
    assert!(!app.should_quit);
    render(&app);
}
