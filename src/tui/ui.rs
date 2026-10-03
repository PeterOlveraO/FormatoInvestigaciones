//! Dibujo del menú interactivo con ratatui. Solo lee el estado de `App`.

use ratatui::Frame;
use ratatui::layout::{Constraint, Layout, Position, Rect};
use ratatui::style::{Color, Modifier, Style, Stylize};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, BorderType, Borders, Clear, List, ListItem, ListState, Paragraph, Wrap};

use super::app::{App, FIELDS, FieldKey, FieldKind, LogLine, Mode};
use super::picker::Picker;
use crate::i18n::{Lang, Text};

const ACCENT: Color = Color::Cyan;

pub fn draw(frame: &mut Frame, app: &App) {
    let area = frame.area();
    let [header, body, footer] =
        Layout::vertical([Constraint::Length(3), Constraint::Min(8), Constraint::Length(2)]).areas(area);

    draw_header(frame, header);

    // En una terminal angosta los paneles van uno debajo de otro, con el
    // formulario y la ayuda a su altura justa y el resto para el resultado.
    let (form, help, log) = if body.width >= 100 {
        let [form, side] =
            Layout::horizontal([Constraint::Percentage(58), Constraint::Percentage(42)]).areas(body);
        let [help, log] = Layout::vertical([Constraint::Length(9), Constraint::Min(4)]).areas(side);
        (form, help, log)
    } else {
        let form_height = FIELDS.len() as u16 + 2;
        let [form, help, log] =
            Layout::vertical([Constraint::Length(form_height), Constraint::Length(6), Constraint::Min(3)])
                .areas(body);
        (form, help, log)
    };
    draw_form(frame, app, form);
    draw_help(frame, app, help);
    draw_log(frame, app, log);
    draw_footer(frame, app, footer);

    match &app.mode {
        Mode::Picking(picker) => draw_picker(frame, app, picker),
        Mode::Editing(input) => {
            let field = FIELDS[app.selected];
            let popup = centered(area, 70, 5);
            frame.render_widget(Clear, popup);
            let block = titled_block(format!(" {} ", field.label)).border_style(Style::new().fg(ACCENT));
            let inner = block.inner(popup);
            frame.render_widget(block, popup);
            let [text_area, hint] =
                Layout::vertical([Constraint::Length(1), Constraint::Length(1)]).areas(inner);
            // Si el texto no cabe se muestra el final, que es donde se escribe.
            let width = text_area.width.saturating_sub(1) as usize;
            let skip = input.cursor.saturating_sub(width);
            let visible: String = input.text.chars().skip(skip).collect();
            frame.render_widget(Paragraph::new(visible), text_area);
            frame.render_widget(
                Paragraph::new(
                    Text::new(
                        "Enter guarda · Esc cancela · Ctrl+U borra",
                        "Enter save · Esc cancel · Ctrl+U clear",
                    )
                    .get(),
                )
                .style(Style::new().fg(Color::DarkGray)),
                hint,
            );
            frame.set_cursor_position(Position::new(text_area.x + (input.cursor - skip) as u16, text_area.y));
        }
        Mode::ChooseLanguage(selected) => draw_language(frame, *selected),
        _ => {}
    }
}

/// Pantalla de la primera vez. Va en los dos idiomas a la vez porque todavía
/// no se sabe cuál lee la persona.
fn draw_language(frame: &mut Frame, selected: Lang) {
    let popup = centered(frame.area(), 50, 9);
    frame.render_widget(Clear, popup);
    let block = titled_block(" Idioma / Language ").border_style(Style::new().fg(ACCENT));
    let inner = block.inner(popup);
    frame.render_widget(block, popup);
    let option = |lang: Lang, label: &'static str| {
        let style = if lang == selected {
            Style::new().bg(Color::DarkGray).add_modifier(Modifier::BOLD)
        } else {
            Style::new()
        };
        let mark = if lang == selected { "› " } else { "  " };
        Line::styled(format!("{mark}{label}"), style)
    };
    let lines = vec![
        Line::raw("¿En qué idioma quieres el menú?"),
        Line::raw("Which language do you want for the menu?"),
        Line::raw(""),
        option(Lang::Es, "Español"),
        option(Lang::En, "English"),
        Line::raw(""),
        Line::styled("↑↓ · Enter", Style::new().fg(Color::DarkGray)),
    ];
    frame.render_widget(Paragraph::new(lines), inner);
}

fn titled_block<'a>(title: impl Into<Line<'a>>) -> Block<'a> {
    Block::new().borders(Borders::ALL).border_type(BorderType::Rounded).title(title)
}

fn draw_header(frame: &mut Frame, area: Rect) {
    let title = Line::from(vec![
        Span::styled(" INVESTIGACION ", Style::new().fg(Color::Black).bg(ACCENT).bold()),
        Span::raw("  Markdown → APA PDF"),
    ]);
    frame.render_widget(Paragraph::new(title).block(titled_block("")), area);
}

fn draw_form(frame: &mut Frame, app: &App, area: Rect) {
    let label_width = 20;
    let items: Vec<ListItem> = FIELDS
        .iter()
        .enumerate()
        .map(|(index, field)| {
            let value = app.values[index].trim();
            let marker = if field.required { "*" } else { " " };
            let picker = if matches!(field.kind, FieldKind::Pick(_)) { "▸ " } else { "  " };
            let shown = if value.is_empty() {
                if field.required {
                    Span::styled(Text::new("[falta]", "[missing]").get(), Style::new().fg(Color::Red))
                } else {
                    Span::styled(format!("({})", field.empty), Style::new().fg(Color::DarkGray))
                }
            } else if field.key == FieldKey::Markdown {
                Span::raw(app.display_path(value))
            } else {
                Span::raw(value.to_owned())
            };
            ListItem::new(Line::from(vec![
                Span::styled(format!("{:>2} ", index + 1), Style::new().fg(Color::DarkGray)),
                Span::raw(picker),
                Span::styled(
                    format!("{:<label_width$}", format!("{}{marker}", field.label)),
                    Style::new().bold(),
                ),
                shown,
            ]))
        })
        .collect();
    let list = List::new(items)
        .block(titled_block(Text::new(" Trabajo ", " Paper ").get()))
        .highlight_style(Style::new().bg(Color::DarkGray).add_modifier(Modifier::BOLD));
    let mut state = ListState::default().with_selected(Some(app.selected));
    frame.render_stateful_widget(list, area, &mut state);
}

fn draw_help(frame: &mut Frame, app: &App, area: Rect) {
    let field = FIELDS[app.selected];
    let mut lines = vec![
        Line::from(field.help.get()),
        Line::from(vec![
            Span::styled(Text::new("Ejemplo: ", "Example: ").get(), Style::new().fg(Color::DarkGray)),
            Span::raw(field.example.get()),
        ]),
    ];
    if !field.required {
        lines.push(Line::from(vec![
            Span::styled(
                Text::new("Si se deja vacío: ", "If empty: ").get(),
                Style::new().fg(Color::DarkGray),
            ),
            Span::raw(field.empty.get()),
        ]));
    }
    let action = match field.kind {
        FieldKind::Text => Text::new("Enter para escribirlo", "Enter to write it"),
        FieldKind::Pick(_) => Text::new("Enter para elegir de la lista", "Enter to choose from the list"),
    }
    .get();
    lines.push(Line::from(Span::styled(action, Style::new().fg(ACCENT))));
    frame.render_widget(
        Paragraph::new(lines).wrap(Wrap { trim: true }).block(titled_block(format!(" {} ", field.label))),
        area,
    );
}

fn draw_log(frame: &mut Frame, app: &App, area: Rect) {
    let missing = app.missing_fields();
    let mut lines: Vec<Line> = Vec::new();
    if app.log.is_empty() {
        lines.push(if missing.is_empty() {
            Line::styled(
                Text::new("Listo. Pulsa g para generar el PDF.", "Ready. Press g to generate the PDF.").get(),
                Style::new().fg(Color::Green),
            )
        } else {
            let missing = missing.join(", ");
            Line::styled(
                tr!(es: "Falta: {missing}.", en: "Missing: {missing}."),
                Style::new().fg(Color::Yellow),
            )
        });
    }
    for entry in &app.log {
        lines.push(match entry {
            LogLine::Info(text) => Line::raw(text.clone()),
            LogLine::Warning(text) => Line::styled(format!("! {text}"), Style::new().fg(Color::Yellow)),
            LogLine::Error(text) => Line::styled(format!("✗ {text}"), Style::new().fg(Color::Red)),
        });
    }
    let title = if matches!(app.mode, Mode::Generating(_)) {
        Text::new(" Resultado (trabajando…) ", " Result (working…) ")
    } else {
        Text::new(" Resultado ", " Result ")
    }
    .get();
    // Se ve el final del registro, que es lo más reciente. `line_count` ya
    // suma los bordes del bloque, por eso se compara con la altura completa.
    let paragraph = Paragraph::new(lines).wrap(Wrap { trim: false }).block(titled_block(title));
    let total = paragraph.line_count(area.width.saturating_sub(2));
    let scroll = total.saturating_sub(area.height as usize) as u16;
    frame.render_widget(paragraph.scroll((scroll, 0)), area);
}

fn draw_footer(frame: &mut Frame, app: &App, area: Rect) {
    // Cada idioma muestra sus propias iniciales; la tecla del idioma dice el
    // nombre del otro, para que lo encuentre quien no lee el actual.
    let keys: Vec<(&str, Text)> = match app.mode {
        Mode::ChooseLanguage(_) => {
            vec![("↑↓", Text::new("elegir", "choose")), ("Enter", Text::new("aceptar", "accept"))]
        }
        Mode::Form => {
            let (folders, quit) = match crate::i18n::current() {
                Lang::Es => ("c", "s"),
                Lang::En => ("f", "q"),
            };
            vec![
                ("↑↓", Text::new("mover", "move")),
                ("Enter", Text::new("editar/elegir", "edit/choose")),
                (Text::new("Supr", "Del").get(), Text::new("vaciar", "clear")),
                ("g", Text::new("generar", "generate")),
                ("v", Text::new("ver PDF", "view PDF")),
                (folders, Text::new("carpetas", "folders")),
                ("l", Text::new("English", "Español")),
                (quit, Text::new("salir", "quit")),
            ]
        }
        Mode::Editing(_) => {
            vec![("Enter", Text::new("guardar", "save")), ("Esc", Text::new("cancelar", "cancel"))]
        }
        Mode::Picking(_) => vec![
            ("↑↓", Text::new("mover", "move")),
            ("Enter/→", Text::new("abrir/elegir", "open/choose")),
            ("←", Text::new("subir", "up")),
            (Text::new("letras", "type").get(), Text::new("filtrar", "filter")),
            ("Esc", Text::new("cancelar", "cancel")),
        ],
        Mode::Generating(_) => vec![("…", Text::new("Generando el PDF", "Generating the PDF"))],
    };
    let spans: Vec<Span> = keys
        .iter()
        .flat_map(|(key, action)| {
            [
                Span::styled(format!(" {key} "), Style::new().fg(Color::Black).bg(ACCENT)),
                Span::raw(format!(" {action}  ")),
            ]
        })
        .collect();
    frame.render_widget(Paragraph::new(Line::from(spans)).wrap(Wrap { trim: true }), area);
}

fn draw_picker(frame: &mut Frame, app: &App, picker: &Picker) {
    let area = centered(frame.area(), 80, frame.area().height.saturating_sub(4).max(8));
    frame.render_widget(Clear, area);
    let block = titled_block(format!(" {} ", picker.title)).border_style(Style::new().fg(ACCENT));
    let inner = block.inner(area);
    frame.render_widget(block, area);
    let [location, filter, list_area] =
        Layout::vertical([Constraint::Length(1), Constraint::Length(1), Constraint::Min(1)]).areas(inner);

    if let Some(dir) = &picker.dir {
        let shown = app.display_path(&dir.display().to_string());
        let shown =
            if shown.is_empty() { Text::new("(proyecto)", "(project)").get().to_owned() } else { shown };
        frame.render_widget(
            Paragraph::new(Line::from(vec![
                Span::styled(Text::new("En: ", "In: ").get(), Style::new().fg(Color::DarkGray)),
                Span::raw(shown),
            ])),
            location,
        );
    }
    let filter_line = if picker.filter.is_empty() {
        Line::styled(
            Text::new("Escribe para filtrar…", "Type to filter…").get(),
            Style::new().fg(Color::DarkGray),
        )
    } else {
        Line::from(vec![
            Span::styled(Text::new("Filtro: ", "Filter: ").get(), Style::new().fg(Color::DarkGray)),
            Span::raw(picker.filter.clone()),
        ])
    };
    frame.render_widget(Paragraph::new(filter_line), filter);

    let visible = picker.visible();
    let items: Vec<ListItem> = if visible.is_empty() {
        vec![ListItem::new(Line::styled(
            Text::new("(nada aquí)", "(nothing here)").get(),
            Style::new().fg(Color::DarkGray),
        ))]
    } else {
        visible
            .iter()
            .map(|item| {
                let style = if item.label.ends_with('/') { Style::new().fg(ACCENT) } else { Style::new() };
                let mut spans = vec![Span::styled(item.label.clone(), style)];
                if !item.detail.is_empty() {
                    spans.push(Span::styled(format!("  {}", item.detail), Style::new().fg(Color::DarkGray)));
                }
                ListItem::new(Line::from(spans))
            })
            .collect()
    };
    let list = List::new(items)
        .highlight_style(Style::new().bg(Color::DarkGray).add_modifier(Modifier::BOLD))
        .highlight_symbol("› ");
    let mut state = ListState::default().with_selected(Some(picker.selected));
    frame.render_stateful_widget(list, list_area, &mut state);
}

/// Rectángulo centrado de `percent_x` de ancho y `height` filas.
fn centered(area: Rect, percent_x: u16, height: u16) -> Rect {
    let width = (area.width * percent_x / 100).max(30).min(area.width);
    let height = height.min(area.height);
    Rect::new(area.x + (area.width - width) / 2, area.y + (area.height - height) / 2, width, height)
}
