//! Pruebas del menú sin terminal real: teclas simuladas y `TestBackend`.

use std::path::Path;

use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use ratatui::Terminal;
use ratatui::backend::TestBackend;

use super::app::{App, FIELDS, FieldKey, LogLine, Mode, index_of};
use super::ui;
use crate::project::Project;

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
    for template in ["apa", "apa-simple"] {
        std::fs::create_dir_all(root.join("templates").join(template)).unwrap();
        std::fs::write(root.join("templates").join(template).join("template.ltx"), "x").unwrap();
    }
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
    let mut terminal = Terminal::new(TestBackend::new(120, 32)).unwrap();
    terminal.draw(|frame| ui::draw(frame, app)).unwrap();
    let buffer = terminal.backend().buffer().clone();
    buffer
        .content()
        .chunks(buffer.area.width as usize)
        .map(|row| row.iter().map(|c| c.symbol()).collect::<String>() + "\n")
        .collect()
}

#[test]
fn the_removed_options_are_not_in_the_menu() {
    let labels: Vec<&str> = FIELDS.iter().map(|f| f.label).collect();
    for removed in [".env", "Allow LaTeX", "Logos"] {
        assert!(labels.iter().all(|l| !l.contains(removed)), "{removed}");
    }
    let (_dir, project) = sample_project();
    let screen = render(&App::new(project));
    assert!(
        screen.contains("Markdown file") && screen.contains("PDF file name") && screen.contains("[missing]")
    );
    assert!(!screen.contains("Allow LaTeX") && !screen.contains("Logos"));
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
