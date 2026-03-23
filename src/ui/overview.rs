use crate::app::App;
use crate::ui::header;
use crate::util::format::{format_bytes, format_bytes_per_sec, format_freq_mhz, format_temp};
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Gauge, Paragraph, Sparkline};
use ratatui::Frame;

pub fn draw(f: &mut Frame, app: &App, area: Rect) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1), // Header (board + throttle)
            Constraint::Min(0),    // Main content
        ])
        .split(area);

    header::draw(f, app, chunks[0]);

    let main = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(app.cpu.cores.len() as u16 + 2), // CPU block
            Constraint::Length(5),                              // Memory + swap
            Constraint::Length(3),                              // Temperature
            Constraint::Length(3),                              // Network
            Constraint::Min(0),                                 // Sparklines
        ])
        .split(chunks[1]);

    draw_cpu_section(f, app, main[0]);
    draw_memory_section(f, app, main[1]);
    draw_temp_section(f, app, main[2]);
    draw_network_section(f, app, main[3]);
    draw_sparklines(f, app, main[4]);
}

fn draw_cpu_section(f: &mut Frame, app: &App, area: Rect) {
    let block = Block::default()
        .title(format!(
            " CPU {} — {} ",
            format_freq_mhz(app.cpu.frequency_khz),
            app.cpu.governor
        ))
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Blue));

    let inner = block.inner(area);
    f.render_widget(block, area);

    if app.cpu.cores.is_empty() {
        return;
    }

    let core_constraints: Vec<Constraint> = app
        .cpu
        .cores
        .iter()
        .map(|_| Constraint::Length(1))
        .collect();

    let rows = Layout::default()
        .direction(Direction::Vertical)
        .constraints(core_constraints)
        .split(inner);

    for (i, core) in app.cpu.cores.iter().enumerate() {
        if i >= rows.len() {
            break;
        }

        let usage = core.usage_percent.min(100.0);
        let color = percent_color(usage, 60.0, 85.0);
        let label = format!("cpu{}: {:5.1}%", core.core_id, usage);

        let gauge = Gauge::default()
            .gauge_style(Style::default().fg(color))
            .label(label)
            .ratio(usage / 100.0);

        f.render_widget(gauge, rows[i]);
    }
}

fn draw_memory_section(f: &mut Frame, app: &App, area: Rect) {
    let block = Block::default()
        .title(" Memory ")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Magenta));

    let inner = block.inner(area);
    f.render_widget(block, area);

    let rows = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1), // RAM gauge
            Constraint::Length(1), // Swap gauge
            Constraint::Length(1), // Load average
        ])
        .split(inner);

    // RAM gauge
    let mem = &app.memory;
    if mem.total_bytes > 0 {
        let ratio = (mem.used_bytes as f64 / mem.total_bytes as f64).min(1.0);
        let color = percent_color(ratio * 100.0, 60.0, 85.0);
        let label = format!(
            "RAM: {} / {} ({:.1}%)",
            format_bytes(mem.used_bytes),
            format_bytes(mem.total_bytes),
            mem.usage_percent
        );
        let gauge = Gauge::default()
            .gauge_style(Style::default().fg(color))
            .label(label)
            .ratio(ratio);
        f.render_widget(gauge, rows[0]);
    }

    // Swap gauge
    if mem.swap_total_bytes > 0 {
        let ratio = (mem.swap_used_bytes as f64 / mem.swap_total_bytes as f64).min(1.0);
        let color = percent_color(ratio * 100.0, 50.0, 80.0);
        let label = format!(
            "Swap: {} / {}",
            format_bytes(mem.swap_used_bytes),
            format_bytes(mem.swap_total_bytes)
        );
        let gauge = Gauge::default()
            .gauge_style(Style::default().fg(color))
            .label(label)
            .ratio(ratio);
        f.render_widget(gauge, rows[1]);
    } else {
        f.render_widget(
            Paragraph::new("Swap: N/A").style(Style::default().fg(Color::DarkGray)),
            rows[1],
        );
    }

    // Load average
    let load_line = Line::from(vec![
        Span::styled("Load: ", Style::default().fg(Color::DarkGray)),
        Span::styled(
            format!(
                "{:.2}  {:.2}  {:.2}",
                app.cpu.load_avg_1, app.cpu.load_avg_5, app.cpu.load_avg_15
            ),
            Style::default().fg(Color::White),
        ),
    ]);
    f.render_widget(Paragraph::new(load_line), rows[2]);
}

fn draw_temp_section(f: &mut Frame, app: &App, area: Rect) {
    let temp = app.thermal.soc_temp_celsius;
    let color = temp_color(temp);

    let block = Block::default()
        .title(" Temperature ")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(color));

    let inner = block.inner(area);
    f.render_widget(block, area);

    let mut spans = vec![Span::styled(
        format!("SoC: {} ", format_temp(temp)),
        Style::default().fg(color).add_modifier(Modifier::BOLD),
    )];

    // Additional thermal zones
    for zone in &app.thermal.zones {
        if zone.zone_name.contains("cpu") {
            continue; // Already shown as SoC
        }
        spans.push(Span::raw("│ "));
        spans.push(Span::styled(
            format!("{}: {} ", zone.zone_name, format_temp(zone.temp_celsius)),
            Style::default().fg(Color::White),
        ));
    }

    f.render_widget(Paragraph::new(Line::from(spans)), inner);
}

fn draw_network_section(f: &mut Frame, app: &App, area: Rect) {
    let block = Block::default()
        .title(" Network ")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Green));

    let inner = block.inner(area);
    f.render_widget(block, area);

    let line = Line::from(vec![
        Span::styled("↓ ", Style::default().fg(Color::Green)),
        Span::styled(
            format_bytes_per_sec(app.network.total_rx_bytes_per_sec),
            Style::default().fg(Color::White),
        ),
        Span::raw("  "),
        Span::styled("↑ ", Style::default().fg(Color::Red)),
        Span::styled(
            format_bytes_per_sec(app.network.total_tx_bytes_per_sec),
            Style::default().fg(Color::White),
        ),
    ]);

    f.render_widget(Paragraph::new(line), inner);
}

fn draw_sparklines(f: &mut Frame, app: &App, area: Rect) {
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(33),
            Constraint::Percentage(34),
            Constraint::Percentage(33),
        ])
        .split(area);

    // CPU sparkline
    let cpu_data: Vec<u64> = app
        .cpu_history
        .as_slice()
        .iter()
        .map(|v| *v as u64)
        .collect();
    let cpu_spark = Sparkline::default()
        .block(
            Block::default()
                .title(format!(" CPU {:.1}% ", app.cpu.aggregate_usage_percent))
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::Blue)),
        )
        .data(&cpu_data)
        .max(100)
        .style(Style::default().fg(Color::Blue));
    f.render_widget(cpu_spark, chunks[0]);

    // Memory sparkline
    let mem_data: Vec<u64> = app
        .mem_history
        .as_slice()
        .iter()
        .map(|v| *v as u64)
        .collect();
    let mem_spark = Sparkline::default()
        .block(
            Block::default()
                .title(format!(" MEM {:.1}% ", app.memory.usage_percent))
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::Magenta)),
        )
        .data(&mem_data)
        .max(100)
        .style(Style::default().fg(Color::Magenta));
    f.render_widget(mem_spark, chunks[1]);

    // Temp sparkline
    let temp_data: Vec<u64> = app
        .temp_history
        .as_slice()
        .iter()
        .map(|v| *v as u64)
        .collect();
    let temp_spark = Sparkline::default()
        .block(
            Block::default()
                .title(format!(
                    " TEMP {} ",
                    format_temp(app.thermal.soc_temp_celsius)
                ))
                .borders(Borders::ALL)
                .border_style(Style::default().fg(temp_color(app.thermal.soc_temp_celsius))),
        )
        .data(&temp_data)
        .max(100)
        .style(Style::default().fg(temp_color(app.thermal.soc_temp_celsius)));
    f.render_widget(temp_spark, chunks[2]);
}

/// Color by percentage thresholds.
fn percent_color(percent: f64, warn: f64, crit: f64) -> Color {
    if percent >= crit {
        Color::Red
    } else if percent >= warn {
        Color::Yellow
    } else {
        Color::Green
    }
}

/// Color by temperature thresholds.
fn temp_color(celsius: f64) -> Color {
    if celsius >= 70.0 {
        Color::Red
    } else if celsius >= 60.0 {
        Color::Yellow
    } else {
        Color::Green
    }
}
