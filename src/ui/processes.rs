use crate::app::App;
use crate::util::format::format_bytes;
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Paragraph, Row, Table};
use ratatui::Frame;

/// Sort column names for the header.
const SORT_COLUMNS: [&str; 5] = ["PID", "Name", "CPU%", "Memory", "User"];

pub fn draw(f: &mut Frame, app: &App, area: Rect) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Min(0),    // Process table
            Constraint::Length(1), // Status bar
        ])
        .split(area);

    draw_process_table(f, app, chunks[0]);
    draw_status_bar(f, app, chunks[1]);
}

fn draw_process_table(f: &mut Frame, app: &App, area: Rect) {
    let sort_col = app.process_sort_column;

    // Build header with sort indicator
    let header_cells: Vec<Span> = SORT_COLUMNS
        .iter()
        .enumerate()
        .map(|(i, name)| {
            if i == sort_col {
                Span::styled(
                    format!("{} ▼", name),
                    Style::default()
                        .fg(Color::Yellow)
                        .add_modifier(Modifier::BOLD),
                )
            } else {
                Span::styled(name.to_string(), Style::default().fg(Color::White))
            }
        })
        .collect();

    let header = Row::new(header_cells)
        .style(Style::default().add_modifier(Modifier::UNDERLINED))
        .height(1);

    // Use the shared sort method so UI and event handler agree on ordering
    let sorted_procs = app.sorted_processes();

    // Build rows
    let rows: Vec<Row> = sorted_procs
        .iter()
        .enumerate()
        .map(|(i, proc)| {
            let style = if i == app.process_selected {
                Style::default()
                    .bg(Color::DarkGray)
                    .fg(Color::White)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(Color::White)
            };

            Row::new(vec![
                proc.pid.to_string(),
                proc.name.clone(),
                format!("{:.1}", proc.cpu_percent),
                format_bytes(proc.rss_bytes),
                proc.user.clone(),
            ])
            .style(style)
        })
        .collect();

    let widths = [
        Constraint::Length(8),  // PID
        Constraint::Min(15),    // Name
        Constraint::Length(8),  // CPU%
        Constraint::Length(12), // Memory
        Constraint::Length(10), // User
    ];

    let table = Table::new(rows, widths)
        .header(header)
        .block(
            Block::default()
                .title(format!(" Processes ({}) ", sorted_procs.len()))
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::Cyan)),
        )
        .column_spacing(1);

    f.render_widget(table, area);
}

fn draw_status_bar(f: &mut Frame, app: &App, area: Rect) {
    // Kill confirmation prompt takes priority
    if let Some((pid, ref proc_name)) = app.kill_confirm {
        let line = Line::from(vec![
            Span::styled(
                format!(" Kill PID {} ({})? ", pid, proc_name),
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(
                "y/n",
                Style::default()
                    .fg(Color::White)
                    .add_modifier(Modifier::BOLD),
            ),
        ]);
        let bar = Paragraph::new(line).style(Style::default().bg(Color::Red));
        f.render_widget(bar, area);
        return;
    }

    // Show kill result message if present
    if let Some(ref msg) = app.kill_result {
        let color = if msg.starts_with("Sent") {
            Color::Green
        } else {
            Color::Red
        };
        let bar =
            Paragraph::new(format!(" {} (press any key)", msg)).style(Style::default().fg(color));
        f.render_widget(bar, area);
        return;
    }

    // Default status bar with keybinding hints
    let hints = " j/k:Navigate  s:Sort  K:Kill  q:Quit";
    let footer = Paragraph::new(hints).style(Style::default().fg(Color::DarkGray));
    f.render_widget(footer, area);
}
