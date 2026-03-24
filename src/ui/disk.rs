use crate::app::App;
use crate::ui::theme::Theme;
use crate::util::format::{format_bytes, format_bytes_per_sec};
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::widgets::{Block, Borders, Cell, Row, Table};
use ratatui::Frame;

pub fn draw(f: &mut Frame, app: &App, area: Rect) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage(55), // Partition table
            Constraint::Percentage(45), // I/O stats
        ])
        .split(area);

    draw_partition_table(f, app, chunks[0]);
    draw_io_stats(f, app, chunks[1]);
}

fn draw_partition_table(f: &mut Frame, app: &App, area: Rect) {
    let theme = &app.theme;
    let header_cells = ["Device", "Mount", "Type", "Total", "Used", "Free", "Use%"]
        .iter()
        .map(|h| {
            Cell::from(*h).style(
                Style::default()
                    .fg(theme.title)
                    .add_modifier(Modifier::BOLD),
            )
        });
    let header = Row::new(header_cells).height(1);

    let rows: Vec<Row> = app
        .disk
        .partitions
        .iter()
        .map(|p| {
            let usage_color = usage_color(theme, p.usage_percent);
            Row::new(vec![
                Cell::from(p.device.clone()),
                Cell::from(p.mountpoint.clone()),
                Cell::from(p.fs_type.clone()),
                Cell::from(format_bytes(p.total_bytes)),
                Cell::from(format_bytes(p.used_bytes)),
                Cell::from(format_bytes(p.free_bytes)),
                Cell::from(format!("{:.1}%", p.usage_percent))
                    .style(Style::default().fg(usage_color)),
            ])
        })
        .collect();

    let table = Table::new(
        rows,
        [
            Constraint::Min(16),    // Device
            Constraint::Min(16),    // Mount
            Constraint::Length(6),  // Type
            Constraint::Length(10), // Total
            Constraint::Length(10), // Used
            Constraint::Length(10), // Free
            Constraint::Length(7),  // Use%
        ],
    )
    .header(header)
    .block(
        Block::default()
            .title(" Partitions ")
            .borders(Borders::ALL)
            .border_style(Style::default().fg(theme.border_highlight)),
    )
    .style(Style::default().fg(theme.text));

    f.render_widget(table, area);
}

fn draw_io_stats(f: &mut Frame, app: &App, area: Rect) {
    let theme = &app.theme;
    let header_cells = ["Device", "Read/s", "Write/s", "Total Read", "Total Written"]
        .iter()
        .map(|h| {
            Cell::from(*h).style(
                Style::default()
                    .fg(theme.title)
                    .add_modifier(Modifier::BOLD),
            )
        });
    let header = Row::new(header_cells).height(1);

    let rows: Vec<Row> = app
        .disk
        .io_stats
        .iter()
        .map(|io| {
            let read_color = throughput_color(theme, io.read_bytes_per_sec);
            let write_color = throughput_color(theme, io.write_bytes_per_sec);
            Row::new(vec![
                Cell::from(io.device.clone()),
                Cell::from(format_bytes_per_sec(io.read_bytes_per_sec))
                    .style(Style::default().fg(read_color)),
                Cell::from(format_bytes_per_sec(io.write_bytes_per_sec))
                    .style(Style::default().fg(write_color)),
                Cell::from(format_bytes(io.total_read_bytes)),
                Cell::from(format_bytes(io.total_write_bytes)),
            ])
        })
        .collect();

    let table = Table::new(
        rows,
        [
            Constraint::Min(16),    // Device
            Constraint::Length(12), // Read/s
            Constraint::Length(12), // Write/s
            Constraint::Length(12), // Total Read
            Constraint::Length(14), // Total Written
        ],
    )
    .header(header)
    .block(
        Block::default()
            .title(" Disk I/O ")
            .borders(Borders::ALL)
            .border_style(Style::default().fg(theme.border_highlight)),
    )
    .style(Style::default().fg(theme.text));

    f.render_widget(table, area);
}

/// Color for disk usage percentage.
fn usage_color(theme: &Theme, percent: f64) -> Color {
    if percent >= 90.0 {
        theme.gauge_crit
    } else if percent >= 70.0 {
        theme.gauge_warn
    } else {
        theme.gauge_low
    }
}

/// Color for I/O throughput — highlight for active, dim for idle.
fn throughput_color(theme: &Theme, bytes_per_sec: f64) -> Color {
    if bytes_per_sec >= 1_048_576.0 {
        // >= 1 MiB/s
        theme.border_highlight
    } else if bytes_per_sec > 0.5 {
        theme.text
    } else {
        theme.text_dim
    }
}
