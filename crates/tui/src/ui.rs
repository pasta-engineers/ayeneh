//! Terminal UI rendering.

use ayeneh_core::app::Selection;
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Cell, Paragraph, Row, Table, TableState},
    Frame,
};

use crate::{ActiveSection, App};

/// Draws the entire application UI into the given frame.
pub fn draw(frame: &mut Frame, app: &App) {
    let area = frame.area();

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3), // title
            Constraint::Length(4), // package manager menu
            Constraint::Min(6),    // results table
            Constraint::Length(3), // fastest mirror
            Constraint::Length(3), // status / help
            Constraint::Length(3), // error box
        ])
        .split(area);

    draw_title(frame, chunks[0]);
    draw_menu(frame, chunks[1], app);
    draw_results(frame, chunks[2], app);
    draw_fastest(frame, chunks[3], app);

    if app.mode == Mode::Input {
        draw_input(frame, chunks[4], app);
    } else {
        draw_status(frame, chunks[4], app);
    }

    draw_error(frame, chunks[5], app);
}

fn draw_title(frame: &mut Frame, area: Rect) {
    let title = Paragraph::new("Mirror Benchmark")
        .style(Style::default().add_modifier(Modifier::BOLD))
        .block(Block::default().borders(Borders::ALL));
    frame.render_widget(title, area);
}

fn draw_menu(frame: &mut Frame, area: Rect, app: &App) {
    let selected_style = Style::default()
        .fg(Color::Yellow)
        .add_modifier(Modifier::BOLD);

    let pypi_prefix = if app.core.selection == Selection::PyPi {
        "> "
    } else {
        "  "
    };
    let npm_prefix = if app.core.selection == Selection::Npm {
        "> "
    } else {
        "  "
    };

    let pypi_style = if app.core.selection == Selection::PyPi {
        selected_style
    } else {
        Style::default()
    };
    let npm_style = if app.core.selection == Selection::Npm {
        selected_style
    } else {
        Style::default()
    };

    let lines = vec![
        Line::from(Span::styled(format!("{pypi_prefix}PyPI"), pypi_style)),
        Line::from(Span::styled(format!("{npm_prefix}npm"), npm_style)),
    ];

    let border_style = if app.active_section == ActiveSection::PackageManagers {
        Style::default().fg(Color::Yellow)
    } else {
        Style::default()
    };
    let menu = Paragraph::new(lines).block(
        Block::default()
            .title("Package Manager")
            .borders(Borders::ALL)
            .border_style(border_style),
    );
    frame.render_widget(menu, area);
}

fn draw_results(frame: &mut Frame, area: Rect, app: &App) {
    let header = Row::new(vec![
        Cell::from("Mirror"),
        Cell::from("Avg(ms)"),
        Cell::from("Success"),
    ])
    .style(Style::default().add_modifier(Modifier::BOLD));

    let pm = app.core.selection.to_package_manager();
    let mirrors = ayeneh_core::mirror::load_mirrors(pm)
        .map(|config| config.mirrors)
        .unwrap_or_default();
    let pending = app
        .core
        .config
        .as_ref()
        .and_then(|config| config.mirrors.get(app.core.benchmark_index));
    let rows: Vec<Row> = mirrors
        .iter()
        .map(|mirror| {
            if pending == Some(mirror) {
                return Row::new(vec![
                    Cell::from(mirror.clone()),
                    Cell::from("testing"),
                    Cell::from(""),
                ])
                .style(
                    Style::default()
                        .fg(Color::Yellow)
                        .add_modifier(Modifier::BOLD),
                );
            }

            match app
                .core
                .results
                .iter()
                .find(|result| result.name == *mirror)
            {
                Some(result) => {
                    let latency = if result.timed_out {
                        "timeout".to_string()
                    } else {
                        result.average_latency_ms.to_string()
                    };
                    let style = if result.timed_out {
                        Style::default().fg(Color::Red)
                    } else {
                        Style::default().fg(Color::Green)
                    };
                    Row::new(vec![
                        Cell::from(mirror.clone()),
                        Cell::from(latency),
                        Cell::from(format!("{:.0}%", result.success_rate)),
                    ])
                    .style(style)
                }
                None => Row::new(vec![
                    Cell::from(mirror.clone()),
                    Cell::from("N/A"),
                    Cell::from("N/A"),
                ])
                .style(Style::default().fg(Color::DarkGray)),
            }
        })
        .collect();

    let widths = [
        Constraint::Percentage(60),
        Constraint::Percentage(20),
        Constraint::Percentage(20),
    ];

    let has_rows = !rows.is_empty();
    let border_style = if app.active_section == ActiveSection::Results {
        Style::default().fg(Color::Yellow)
    } else {
        Style::default()
    };
    let table = Table::new(rows, widths)
        .header(header)
        .highlight_symbol("> ")
        .highlight_style(Style::default().add_modifier(Modifier::REVERSED))
        .block(
            Block::default()
                .title("Results  [Enter] Test  [d] Delete")
                .borders(Borders::ALL)
                .border_style(border_style),
        );
    let selected = if has_rows && app.active_section == ActiveSection::Results {
        Some(app.selected_mirror)
    } else {
        None
    };
    let mut state = TableState::default().with_selected(selected);

    frame.render_stateful_widget(table, area, &mut state);
}

fn draw_fastest(frame: &mut Frame, area: Rect, app: &App) {
    let text = app.core.fastest().unwrap_or("N/A");
    let widget = Paragraph::new(text)
        .style(Style::default().fg(Color::Cyan))
        .block(
            Block::default()
                .title("Fastest Mirror")
                .borders(Borders::ALL),
        );
    frame.render_widget(widget, area);
}

fn draw_status(frame: &mut Frame, area: Rect, app: &App) {
    let help = format!(
        "{}   [q] Quit   [Enter] Run/Test   [up/down] Navigate   [Shift+Tab] Change Section   [a] Add Mirror",
        app.status
    );
    let widget = Paragraph::new(help).block(Block::default().borders(Borders::ALL));
    frame.render_widget(widget, area);
}

fn draw_input(frame: &mut Frame, area: Rect, app: &App) {
    let widget = Paragraph::new(app.input.as_str())
        .style(Style::default().fg(Color::Cyan))
        .block(
            Block::default()
                .title("Add Mirror URL  [Enter] Save  [Esc] Cancel")
                .borders(Borders::ALL),
        );
    frame.render_widget(widget, area);

    frame.set_cursor_position((area.x + 1 + app.cursor as u16, area.y + 1));
}

fn draw_error(frame: &mut Frame, area: Rect, app: &App) {
    let err_msg = app.error.as_deref().unwrap_or("");
    let widget = Paragraph::new(err_msg)
        .style(Style::default().fg(Color::Red))
        .block(Block::default().title("Error Log").borders(Borders::ALL));
    frame.render_widget(widget, area);
}

// Local Mode enum (kept inside the TUI crate since it only affects UI state).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Mode {
    Normal,
    Input,
}
