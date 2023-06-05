use chrono::Utc;
use ratatui::{
    backend::Backend,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Cell, Padding, Paragraph, Row, Table, Wrap},
    Frame,
};

use crate::app::{Action, Hourglass, View, TIME_FORMAT};
use crate::util::{convert_utc_to_local, format_time};

struct Field {
    name: String,
    value: String,
}

pub fn build_ui<B: Backend>(f: &mut Frame<B>, app: &mut Hourglass) {
    let rects = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(0), Constraint::Length(3)].as_ref())
        .split(f.size())
        .to_vec();

    match app.tab_index {
        0 => render_tasks(app, rects.clone(), f),
        1 => render_issues(app, rects.clone(), f),
        _ => {}
    }

    render_command(app, rects, f);
}

fn render_tasks<B: Backend>(app: &mut Hourglass, rects: Vec<Rect>, f: &mut Frame<B>) {
    let task_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
        .split(rects[0]);

    let rows = app.tasks.iter().map(|task| {
        let height = 1;

        let cells = vec![
            format!("{}", task.id),
            format!("{}", task.description),
            format_time(task.created_at, Utc::now()),
        ]
        .into_iter()
        .map(|c| Cell::from(c));

        let mut style = Style::default();

        if task.completed {
            style = style
                .add_modifier(Modifier::CROSSED_OUT)
                .add_modifier(Modifier::DIM);
        }

        Row::new(cells).height(height).style(style)
    });

    let table = render_table(rows, vec!["ID", "Description", "Age"]);

    f.render_stateful_widget(table, task_layout[0], &mut app.table_state);

    // display details for issue selected
    if let Some(i) = app.table_state.selected() {
        let selected_task = app.tasks.get(i);

        if let Some(task) = selected_task {
            render_details(
                f,
                task_layout.to_vec(),
                vec![String::from("Name"), String::from("Value")],
                vec![
                    Field {
                        name: String::from("ID"),
                        value: task.id.to_string(),
                    },
                    Field {
                        name: String::from("Description"),
                        value: task.description.clone(),
                    },
                    Field {
                        name: String::from("Age"),
                        value: format_time(task.created_at, Utc::now()),
                    },
                    Field {
                        name: String::from("Created at"),
                        value: format!("{}", convert_utc_to_local(task.created_at, TIME_FORMAT)),
                    },
                    Field {
                        name: String::from("Modified at"),
                        value: format!("{}", convert_utc_to_local(task.modified_at, TIME_FORMAT)),
                    },
                ],
            );
        }
    }
}

fn render_issues<B: Backend>(app: &mut Hourglass, rects: Vec<Rect>, f: &mut Frame<B>) {
    let issue_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
        .split(rects[0]);

    let rows = app.issues.iter().map(|issue| {
        let height = 1;

        let cells = vec![
            Span::styled(
                format!("#{}", issue.number),
                Style::default().fg(Color::Green),
            ),
            Span::from(format!("{}", issue.title)),
            Span::from(format_time(issue.created_at, Utc::now())),
        ]
        .into_iter()
        .map(|c| Cell::from(c));

        let style = Style::default();

        Row::new(cells).height(height).style(style)
    });

    let table = render_table(rows, vec!["#", "Title", "Age"]);

    f.render_stateful_widget(table, issue_layout[0], &mut app.table_state);

    // display details for issue selected
    if let Some(i) = app.table_state.selected() {
        let selected_issue = app.issues.get(i);

        if let Some(issue) = selected_issue {
            render_details(
                f,
                issue_layout.to_vec(),
                vec![String::from("Name"), String::from("Value")],
                vec![
                    Field {
                        name: String::from("Number"),
                        value: issue.number.to_string(),
                    },
                    Field {
                        name: String::from("Author"),
                        value: issue.user.login.clone(),
                    },
                    Field {
                        name: String::from("Title"),
                        value: issue.title.clone(),
                    },
                    Field {
                        name: String::from("Body"),
                        value: issue.body.clone(),
                    },
                    Field {
                        name: String::from("Created at"),
                        value: format!("{}", convert_utc_to_local(issue.created_at, TIME_FORMAT)),
                    },
                    Field {
                        name: String::from("Modified at"),
                        value: format!("{}", convert_utc_to_local(issue.updated_at, TIME_FORMAT)),
                    },
                    Field {
                        name: String::from("Link"),
                        value: issue.html_url.clone(),
                    },
                ],
            );
        }
    }
}

fn render_command<B: Backend>(app: &mut Hourglass, rects: Vec<Rect>, f: &mut Frame<B>) {
    let mut title = String::from("Command");

    match &app.view {
        View::Task(action) => match action {
            Action::Add => title.push_str(" - Add task"),
            Action::Update => title.push_str(" - Update task"),
            _ => {}
        },
        View::Issues(action) => {}
    }
    let command = Block::default().borders(Borders::ALL).title(title);

    f.render_widget(Paragraph::new(app.input.clone()).block(command), rects[1]);
}

fn render_details<'a, B: ratatui::backend::Backend>(
    f: &mut Frame<B>,
    rects: Vec<Rect>,
    columns: Vec<String>,
    fields: Vec<Field>,
) {
    let gap = 2;
    let column_width = 12;
    let border_char = "-";

    let mut lines: Vec<Line> = vec![];

    let mut border_text: String = String::new();
    let mut header_text: String = String::new();

    // ======================= Column name ====================
    for col in columns.iter() {
        let header_text_gap = column_width + gap - col.len();

        header_text.push_str(
            format!(
                "{name}{yeet:<width$}",
                width = header_text_gap,
                name = col,
                yeet = ""
            )
            .as_str(),
        );

        border_text.push_str(
            format!(
                "{a}{b}",
                a = border_char.repeat(column_width),
                b = " ".repeat(gap)
            )
            .as_str(),
        );
    }

    lines.push(Line::from(Span::styled(
        header_text,
        Style::default().add_modifier(Modifier::DIM),
    )));
    lines.push(Line::from(Span::styled(
        border_text,
        Style::default().add_modifier(Modifier::DIM),
    )));

    // ====================== END COLUMN NAME ======================

    // ====================== COLUMN FIELDS ========================

    for field in fields.iter() {
        let field_text = format!(
            "{field}:{space}{value}",
            field = field.name,
            space = " ".repeat(column_width + gap - field.name.len() - 1),
            value = field.value
        );

        lines.push(Line::from(Span::styled(field_text, Style::default())));
    }

    // ====================== END COLUMN FIELDS ========================

    let details_block = Block::default().padding(Padding::horizontal(2));
    let details_text = Paragraph::new(lines.clone()).wrap(Wrap { trim: true });

    let description_block = details_text.block(details_block);
    f.render_widget(description_block, rects[1]);
}

fn render_table<'a, T>(rows: T, header_content: Vec<&'a str>) -> Table<'a>
where
    T: IntoIterator<Item = Row<'a>>,
{
    let header_cells = header_content.iter().map(|x| {
        Cell::from(*x).style(
            Style::default()
                // .add_modifier(Modifier::UNDERLINED)
                .add_modifier(Modifier::DIM),
        )
    });

    let header = Row::new(header_cells)
        .style(Style::default())
        .height(1)
        .bottom_margin(1);

    Table::new(rows)
        .header(header)
        .block(
            Block::default()
                .borders(Borders::BOTTOM)
                .title("Tasks")
                .padding(Padding::uniform(1)),
        )
        .highlight_symbol(">")
        .highlight_style(Style::default().add_modifier(Modifier::BOLD))
        .widths(&[
            Constraint::Percentage(15),
            Constraint::Percentage(75),
            Constraint::Percentage(10),
        ])
}
