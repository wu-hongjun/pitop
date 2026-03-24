use crate::app::App;
use crate::ui::header;
use crate::ui::theme::Theme;
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

    let temp_height = if !app.gpu.codecs.is_empty() { 4 } else { 3 };
    let main = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(app.cpu.cores.len() as u16 + 2), // CPU block
            Constraint::Length(5),                              // Memory + swap
            Constraint::Length(temp_height),                    // Temperature + codecs
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
    let theme = &app.theme;
    let block = Block::default()
        .title(format!(
            " CPU {} — {} ",
            format_freq_mhz(app.cpu.frequency_khz),
            app.cpu.governor
        ))
        .borders(Borders::ALL)
        .border_style(Style::default().fg(theme.cpu_border));

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
        let color = percent_color(theme, usage, 60.0, 85.0);
        let label = format!("cpu{}: {:5.1}%", core.core_id, usage);

        let gauge = Gauge::default()
            .gauge_style(Style::default().fg(color))
            .label(label)
            .ratio(usage / 100.0);

        f.render_widget(gauge, rows[i]);
    }
}

fn draw_memory_section(f: &mut Frame, app: &App, area: Rect) {
    let theme = &app.theme;
    let block = Block::default()
        .title(" Memory ")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(theme.mem_border));

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
        let color = percent_color(theme, ratio * 100.0, 60.0, 85.0);
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
        let color = percent_color(theme, ratio * 100.0, 50.0, 80.0);
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
            Paragraph::new("Swap: N/A").style(Style::default().fg(theme.text_dim)),
            rows[1],
        );
    }

    // Load average
    let load_line = Line::from(vec![
        Span::styled("Load: ", Style::default().fg(theme.text_dim)),
        Span::styled(
            format!(
                "{:.2}  {:.2}  {:.2}",
                app.cpu.load_avg_1, app.cpu.load_avg_5, app.cpu.load_avg_15
            ),
            Style::default().fg(theme.text),
        ),
    ]);
    f.render_widget(Paragraph::new(load_line), rows[2]);
}

fn draw_temp_section(f: &mut Frame, app: &App, area: Rect) {
    let theme = &app.theme;
    let temp = app.thermal.soc_temp_celsius;
    let color = temp_color(theme, temp);

    let block = Block::default()
        .title(" Temperature ")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(theme.temp_border));

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
            Style::default().fg(theme.text),
        ));
    }

    // Fan speed (Pi 5 only)
    if app.fan.available {
        spans.push(Span::raw("│ "));
        spans.push(Span::styled(
            format!("Fan: {} RPM ({:.0}%)", app.fan.rpm, app.fan.pwm_percent),
            Style::default().fg(theme.border_highlight),
        ));
    }

    // GPU info
    if app.gpu.available {
        spans.push(Span::raw("│ "));
        spans.push(Span::styled(
            format!(
                "GPU: {} MHz / {}M / {:.1}°C",
                app.gpu.frequency_mhz, app.gpu.memory_mb, app.gpu.temperature_celsius
            ),
            Style::default().fg(theme.sparkline_power),
        ));
    }

    let mut lines = vec![Line::from(spans)];

    // Codec status line
    if !app.gpu.codecs.is_empty() {
        let mut codec_spans: Vec<Span> = Vec::new();
        for (i, (name, enabled)) in app.gpu.codecs.iter().enumerate() {
            if i > 0 {
                codec_spans.push(Span::raw("  "));
            }
            let (symbol, color) = if *enabled {
                ("\u{2713}", theme.gauge_low)
            } else {
                ("\u{2717}", theme.text_dim)
            };
            codec_spans.push(Span::styled(
                format!("{}: {}", name, symbol),
                Style::default().fg(color),
            ));
        }
        lines.push(Line::from(codec_spans));
    }

    f.render_widget(Paragraph::new(lines), inner);
}

fn draw_network_section(f: &mut Frame, app: &App, area: Rect) {
    let theme = &app.theme;
    let block = Block::default()
        .title(" Network ")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(theme.net_border));

    let inner = block.inner(area);
    f.render_widget(block, area);

    let line = Line::from(vec![
        Span::styled("↓ ", Style::default().fg(theme.net_border)),
        Span::styled(
            format_bytes_per_sec(app.network.total_rx_bytes_per_sec),
            Style::default().fg(theme.text),
        ),
        Span::raw("  "),
        Span::styled("↑ ", Style::default().fg(theme.gauge_warn)),
        Span::styled(
            format_bytes_per_sec(app.network.total_tx_bytes_per_sec),
            Style::default().fg(theme.text),
        ),
    ]);

    f.render_widget(Paragraph::new(line), inner);
}

fn draw_sparklines(f: &mut Frame, app: &App, area: Rect) {
    let theme = &app.theme;

    // CPU sparkline data
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
                .border_style(Style::default().fg(theme.sparkline_cpu)),
        )
        .data(&cpu_data)
        .max(100)
        .style(Style::default().fg(theme.sparkline_cpu));

    // Memory sparkline data
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
                .border_style(Style::default().fg(theme.sparkline_mem)),
        )
        .data(&mem_data)
        .max(100)
        .style(Style::default().fg(theme.sparkline_mem));

    // Temp sparkline data
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
                .border_style(Style::default().fg(theme.temp_border)),
        )
        .data(&temp_data)
        .max(100)
        .style(Style::default().fg(theme.sparkline_temp));

    if app.gpu.available {
        // 2x2 grid: top row (CPU | MEM), bottom row (TEMP | GPU freq)
        let rows = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
            .split(area);

        let top_cols = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
            .split(rows[0]);

        let bottom_cols = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
            .split(rows[1]);

        f.render_widget(cpu_spark, top_cols[0]);
        f.render_widget(mem_spark, top_cols[1]);
        f.render_widget(temp_spark, bottom_cols[0]);

        // GPU frequency sparkline
        let gpu_data: Vec<u64> = app
            .gpu_freq_history
            .as_slice()
            .iter()
            .map(|v| *v as u64)
            .collect();
        let gpu_spark = Sparkline::default()
            .block(
                Block::default()
                    .title(format!(" GPU {} MHz ", app.gpu.frequency_mhz))
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(theme.sparkline_power)),
            )
            .data(&gpu_data)
            .max(1500)
            .style(Style::default().fg(theme.sparkline_power));
        f.render_widget(gpu_spark, bottom_cols[1]);
    } else {
        // 1x3 layout: CPU | MEM | TEMP
        let chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Percentage(33),
                Constraint::Percentage(34),
                Constraint::Percentage(33),
            ])
            .split(area);

        f.render_widget(cpu_spark, chunks[0]);
        f.render_widget(mem_spark, chunks[1]);
        f.render_widget(temp_spark, chunks[2]);
    }
}

/// Color by percentage thresholds.
fn percent_color(theme: &Theme, percent: f64, warn: f64, crit: f64) -> Color {
    if percent >= crit {
        theme.gauge_crit
    } else if percent >= warn {
        theme.gauge_warn
    } else {
        theme.gauge_low
    }
}

/// Color by temperature thresholds.
fn temp_color(theme: &Theme, celsius: f64) -> Color {
    if celsius >= 70.0 {
        theme.gauge_crit
    } else if celsius >= 60.0 {
        theme.gauge_warn
    } else {
        theme.gauge_low
    }
}
