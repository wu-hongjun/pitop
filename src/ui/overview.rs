use crate::app::App;
use crate::ui::header;
use crate::ui::theme::Theme;
use crate::util::format::{format_bytes, format_bytes_per_sec, format_freq_mhz, format_temp};
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Gauge, Paragraph, Row, Table};
use ratatui::Frame;

pub fn draw(f: &mut Frame, app: &App, area: Rect) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1), // Header
            Constraint::Min(0),    // Main content
        ])
        .split(area);

    header::draw(f, app, chunks[0]);

    let main = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage(30), // Top 2 gauges: CPU + GPU/Temp
            Constraint::Percentage(25), // Middle 2 gauges: Temp/Fan + Memory
            Constraint::Length(5),      // 4 info panels
            Constraint::Min(5),         // Process list
        ])
        .split(chunks[1]);

    // Top row: CPU | GPU (or Load if no GPU)
    let top_cols = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
        .split(main[0]);
    draw_cpu_gauge(f, app, top_cols[0]);
    draw_gpu_gauge(f, app, top_cols[1]);

    // Middle row: Temperature | Memory
    let mid_cols = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
        .split(main[1]);
    draw_temp_gauge(f, app, mid_cols[0]);
    draw_memory_gauge(f, app, mid_cols[1]);

    draw_info_row(f, app, main[2]);
    draw_process_list(f, app, main[3]);
}

// ---------------------------------------------------------------------------
// Big gauge blocks
// ---------------------------------------------------------------------------

fn draw_cpu_gauge(f: &mut Frame, app: &App, area: Rect) {
    let theme = &app.theme;
    let usage = app.cpu.aggregate_usage_percent.min(100.0);
    let core_count = app.cpu.cores.len();
    let freq = format_freq_mhz(app.cpu.frequency_khz);
    let temp = format_temp(app.thermal.soc_temp_celsius);

    let title = format!(" {} Cores {:.2}% ({}) ({}) ", core_count, usage, freq, temp);
    let color = percent_color(theme, usage, 60.0, 85.0);

    let block = Block::default()
        .title(Line::from(Span::styled(
            title,
            Style::default()
                .fg(theme.gauge_low)
                .add_modifier(Modifier::BOLD),
        )))
        .borders(Borders::ALL)
        .border_style(Style::default().fg(theme.gauge_low));

    let inner = block.inner(area);
    f.render_widget(block, area);

    let gauge = Gauge::default()
        .gauge_style(Style::default().fg(color))
        .label(format!("{}%", usage as u64))
        .ratio((usage / 100.0).min(1.0));
    f.render_widget(gauge, inner);
}

fn draw_gpu_gauge(f: &mut Frame, app: &App, area: Rect) {
    let theme = &app.theme;

    if app.gpu.available {
        let freq = app.gpu.frequency_mhz;
        let ratio = (freq as f64 / 1500.0).clamp(0.0, 1.0);
        let mem_text = if app.gpu.shared_memory {
            "Shared".to_string()
        } else {
            format!("{}M", app.gpu.memory_mb)
        };
        let temp = format_temp(app.gpu.temperature_celsius);

        let title = format!(" GPU: {} @ {} MHz ({}) ", mem_text, freq, temp);

        let block = Block::default()
            .title(Line::from(Span::styled(
                title,
                Style::default()
                    .fg(theme.gauge_low)
                    .add_modifier(Modifier::BOLD),
            )))
            .borders(Borders::ALL)
            .border_style(Style::default().fg(theme.gauge_low));

        let inner = block.inner(area);
        f.render_widget(block, area);

        let gauge = Gauge::default()
            .gauge_style(Style::default().fg(theme.sparkline_power))
            .label(format!("{} MHz", freq))
            .ratio(ratio);
        f.render_widget(gauge, inner);
    } else {
        // No GPU — show load averages
        let load = app.cpu.load_avg_1;
        let cores = app.cpu.cores.len().max(1) as f64;
        let ratio = (load / cores).clamp(0.0, 1.0);
        let color = percent_color(theme, ratio * 100.0, 60.0, 85.0);

        let title = format!(
            " Load: {:.2} {:.2} {:.2} ",
            app.cpu.load_avg_1, app.cpu.load_avg_5, app.cpu.load_avg_15
        );

        let block = Block::default()
            .title(Line::from(Span::styled(
                title,
                Style::default()
                    .fg(theme.gauge_low)
                    .add_modifier(Modifier::BOLD),
            )))
            .borders(Borders::ALL)
            .border_style(Style::default().fg(theme.gauge_low));

        let inner = block.inner(area);
        f.render_widget(block, area);

        let gauge = Gauge::default()
            .gauge_style(Style::default().fg(color))
            .label(format!("{:.2}", load))
            .ratio(ratio);
        f.render_widget(gauge, inner);
    }
}

fn draw_temp_gauge(f: &mut Frame, app: &App, area: Rect) {
    let theme = &app.theme;
    let temp = app.thermal.soc_temp_celsius;
    let color = temp_color(theme, temp);
    let ratio = (temp / 85.0).clamp(0.0, 1.0);

    let fan_text = if app.fan.available {
        format!(" Fan: {} RPM ({:.0}%)", app.fan.rpm, app.fan.pwm_percent)
    } else {
        String::new()
    };
    let nvme_text = if let Some(nvme) = app.thermal.nvme_temp_celsius {
        format!(" NVMe: {}", format_temp(nvme))
    } else {
        String::new()
    };

    let title = format!(" SoC: {}{}{} ", format_temp(temp), fan_text, nvme_text);

    let block = Block::default()
        .title(Line::from(Span::styled(
            title,
            Style::default()
                .fg(theme.gauge_low)
                .add_modifier(Modifier::BOLD),
        )))
        .borders(Borders::ALL)
        .border_style(Style::default().fg(theme.gauge_low));

    let inner = block.inner(area);
    f.render_widget(block, area);

    let gauge = Gauge::default()
        .gauge_style(Style::default().fg(color))
        .label(format_temp(temp))
        .ratio(ratio);
    f.render_widget(gauge, inner);
}

fn draw_memory_gauge(f: &mut Frame, app: &App, area: Rect) {
    let theme = &app.theme;
    let mem = &app.memory;

    let ratio = if mem.total_bytes > 0 {
        (mem.used_bytes as f64 / mem.total_bytes as f64).min(1.0)
    } else {
        0.0
    };
    let pct = ratio * 100.0;
    let color = percent_color(theme, pct, 60.0, 85.0);

    let swap_text = if mem.swap_total_bytes > 0 {
        format!(
            " (Swap: {}/{})",
            format_bytes(mem.swap_used_bytes),
            format_bytes(mem.swap_total_bytes)
        )
    } else {
        String::new()
    };

    let title = format!(
        " Memory: {} / {}{} ",
        format_bytes(mem.used_bytes),
        format_bytes(mem.total_bytes),
        swap_text
    );

    let block = Block::default()
        .title(Line::from(Span::styled(
            title,
            Style::default()
                .fg(theme.gauge_low)
                .add_modifier(Modifier::BOLD),
        )))
        .borders(Borders::ALL)
        .border_style(Style::default().fg(theme.gauge_low));

    let inner = block.inner(area);
    f.render_widget(block, area);

    let gauge = Gauge::default()
        .gauge_style(Style::default().fg(color))
        .label(format!("{}%", pct as u64))
        .ratio(ratio);
    f.render_widget(gauge, inner);
}

// ---------------------------------------------------------------------------
// Info row: 4 compact panels
// ---------------------------------------------------------------------------

fn draw_info_row(f: &mut Frame, app: &App, area: Rect) {
    let cols = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(25),
            Constraint::Percentage(25),
            Constraint::Percentage(25),
            Constraint::Percentage(25),
        ])
        .split(area);

    draw_info_power(f, app, cols[0]);
    draw_info_board(f, app, cols[1]);
    draw_info_network(f, app, cols[2]);
    draw_info_disk(f, app, cols[3]);
}

fn draw_info_power(f: &mut Frame, app: &App, area: Rect) {
    let theme = &app.theme;
    let block = Block::default()
        .title(Span::styled(
            " Power Usage ",
            Style::default().fg(theme.gauge_low),
        ))
        .borders(Borders::ALL)
        .border_style(Style::default().fg(theme.gauge_low));
    let inner = block.inner(area);
    f.render_widget(block, area);

    let mut lines: Vec<Line> = Vec::new();

    if let Some(ref pmic) = app.power.pmic {
        lines.push(Line::from(Span::styled(
            format!("Total: {:.2} W", pmic.estimated_real_watts),
            Style::default().fg(theme.text).add_modifier(Modifier::BOLD),
        )));
        if pmic.ext5v_voltage > 0.0 {
            lines.push(Line::from(Span::styled(
                format!("Input: {:.2}V", pmic.ext5v_voltage),
                Style::default().fg(theme.text),
            )));
        }
        lines.push(Line::from(Span::styled(
            "Thermals: Normal",
            Style::default().fg(theme.gauge_low),
        )));
    } else if !app.power.voltages.is_empty() {
        for v in app.power.voltages.iter().take(inner.height as usize) {
            lines.push(Line::from(Span::styled(
                format!("{}: {:.4}V", v.name, v.voltage),
                Style::default().fg(theme.text),
            )));
        }
    } else {
        lines.push(Line::from(Span::styled(
            "N/A",
            Style::default().fg(theme.text_dim),
        )));
    }

    lines.truncate(inner.height as usize);
    f.render_widget(Paragraph::new(lines), inner);
}

fn draw_info_board(f: &mut Frame, app: &App, area: Rect) {
    let theme = &app.theme;
    let block = Block::default()
        .title(Span::styled(
            " Raspberry Pi ",
            Style::default().fg(theme.gauge_low),
        ))
        .borders(Borders::ALL)
        .border_style(Style::default().fg(theme.gauge_low));
    let inner = block.inner(area);
    f.render_widget(block, area);

    let mut lines: Vec<Line> = Vec::new();

    lines.push(Line::from(Span::styled(
        app.profile.name().to_string(),
        Style::default().fg(theme.text),
    )));
    lines.push(Line::from(Span::styled(
        format!("{} Cores", app.cpu.cores.len()),
        Style::default().fg(theme.text),
    )));

    // Video decoder info
    if let Some(ref decoder) = app.gpu.video_decoder {
        lines.push(Line::from(Span::styled(
            decoder.clone(),
            Style::default().fg(theme.text_dim),
        )));
    } else if !app.gpu.codecs.is_empty() {
        let codec_str: Vec<String> = app
            .gpu
            .codecs
            .iter()
            .filter(|(_, enabled)| *enabled)
            .map(|(name, _)| name.clone())
            .collect();
        if !codec_str.is_empty() {
            lines.push(Line::from(Span::styled(
                codec_str.join(", "),
                Style::default().fg(theme.text_dim),
            )));
        }
    }

    lines.truncate(inner.height as usize);
    f.render_widget(Paragraph::new(lines), inner);
}

fn draw_info_network(f: &mut Frame, app: &App, area: Rect) {
    let theme = &app.theme;

    let primary_name = app
        .network
        .interfaces
        .iter()
        .find(|i| i.operstate == "up" && i.name != "lo")
        .or_else(|| app.network.interfaces.iter().find(|i| i.name != "lo"))
        .map(|i| i.name.as_str())
        .unwrap_or("net");

    let block = Block::default()
        .title(Span::styled(
            " Network ",
            Style::default().fg(theme.gauge_low),
        ))
        .borders(Borders::ALL)
        .border_style(Style::default().fg(theme.gauge_low));
    let inner = block.inner(area);
    f.render_widget(block, area);

    let mut lines: Vec<Line> = Vec::new();

    lines.push(Line::from(vec![
        Span::styled(
            format!("{}: ", primary_name),
            Style::default().fg(theme.text_dim),
        ),
        Span::styled(
            format!(
                "\u{2191} {} \u{2193} {}",
                format_bytes_per_sec(app.network.total_tx_bytes_per_sec),
                format_bytes_per_sec(app.network.total_rx_bytes_per_sec),
            ),
            Style::default().fg(theme.text),
        ),
    ]));

    // Show additional interfaces
    for iface in app
        .network
        .interfaces
        .iter()
        .filter(|i| i.name != "lo" && i.name != primary_name)
        .take(2)
    {
        lines.push(Line::from(Span::styled(
            format!(
                "{}: \u{2191}{} \u{2193}{}",
                iface.name,
                format_bytes_per_sec(iface.tx_bytes_per_sec),
                format_bytes_per_sec(iface.rx_bytes_per_sec),
            ),
            Style::default().fg(theme.text_dim),
        )));
    }

    lines.truncate(inner.height as usize);
    f.render_widget(Paragraph::new(lines), inner);
}

fn draw_info_disk(f: &mut Frame, app: &App, area: Rect) {
    let theme = &app.theme;
    let block = Block::default()
        .title(Span::styled(" Disk ", Style::default().fg(theme.gauge_low)))
        .borders(Borders::ALL)
        .border_style(Style::default().fg(theme.gauge_low));
    let inner = block.inner(area);
    f.render_widget(block, area);

    let max_lines = inner.height as usize;
    let mut lines: Vec<Line> = Vec::new();

    // Show root partition (or first) with name on one line, usage on next
    let root_part = app
        .disk
        .partitions
        .iter()
        .find(|p| p.mountpoint == "/")
        .or_else(|| app.disk.partitions.first());

    if let Some(part) = root_part {
        let dev_name = part.device.strip_prefix("/dev/").unwrap_or(&part.device);
        let color = percent_color(theme, part.usage_percent, 70.0, 90.0);

        // Line 1: device name + mountpoint
        lines.push(Line::from(vec![
            Span::styled(
                format!("{} ", dev_name),
                Style::default().fg(theme.text).add_modifier(Modifier::BOLD),
            ),
            Span::styled(
                format!("({})", part.mountpoint),
                Style::default().fg(theme.text_dim),
            ),
        ]));

        // Line 2: used / total (percent)
        lines.push(Line::from(Span::styled(
            format!(
                "{}/{} ({:.0}% used)",
                format_bytes(part.used_bytes),
                format_bytes(part.total_bytes),
                part.usage_percent
            ),
            Style::default().fg(color),
        )));
    }

    // Line 3: I/O throughput
    if lines.len() < max_lines {
        if let Some(io) = app.disk.io_stats.first() {
            lines.push(Line::from(vec![
                Span::styled("I/O: ", Style::default().fg(theme.text_dim)),
                Span::styled(
                    format!(
                        "R {} W {}",
                        format_bytes_per_sec(io.read_bytes_per_sec),
                        format_bytes_per_sec(io.write_bytes_per_sec),
                    ),
                    Style::default().fg(theme.text),
                ),
            ]));
        }
    }

    if lines.is_empty() {
        lines.push(Line::from(Span::styled(
            "N/A",
            Style::default().fg(theme.text_dim),
        )));
    }

    lines.truncate(inner.height as usize);
    f.render_widget(Paragraph::new(lines), inner);
}

// ---------------------------------------------------------------------------
// Process list (like mactop)
// ---------------------------------------------------------------------------

fn draw_process_list(f: &mut Frame, app: &App, area: Rect) {
    let theme = &app.theme;

    let block = Block::default()
        .title(Span::styled(
            " Process List (\u{2191}/\u{2193} scroll, s sort, K kill) ",
            Style::default().fg(theme.gauge_low),
        ))
        .borders(Borders::ALL)
        .border_style(Style::default().fg(theme.gauge_low));

    let inner = block.inner(area);
    f.render_widget(block, area);

    if app.processes.is_empty() {
        f.render_widget(
            Paragraph::new(Span::styled(
                "Loading...",
                Style::default().fg(theme.text_dim),
            )),
            inner,
        );
        return;
    }

    let header_style = Style::default()
        .fg(theme.gauge_low)
        .add_modifier(Modifier::BOLD);
    let header = Row::new(vec!["PID", "USER", "CPU%", "MEM", "CMD"]).style(header_style);

    let sorted = app.sorted_processes();
    let visible = inner.height.saturating_sub(1) as usize; // -1 for header
    let rows: Vec<Row> = sorted
        .iter()
        .take(visible)
        .map(|p| {
            let cpu_color = percent_color(theme, p.cpu_percent, 30.0, 70.0);
            Row::new(vec![
                Line::from(Span::styled(
                    format!("{}", p.pid),
                    Style::default().fg(theme.text_dim),
                )),
                Line::from(Span::styled(
                    truncate(&p.user, 8),
                    Style::default().fg(theme.text_dim),
                )),
                Line::from(Span::styled(
                    format!("{:.1}%", p.cpu_percent),
                    Style::default().fg(cpu_color),
                )),
                Line::from(Span::styled(
                    format_bytes(p.rss_bytes),
                    Style::default().fg(theme.text),
                )),
                Line::from(Span::styled(
                    p.name.clone(),
                    Style::default().fg(theme.text),
                )),
            ])
        })
        .collect();

    let widths = [
        Constraint::Length(7),
        Constraint::Length(9),
        Constraint::Length(7),
        Constraint::Length(9),
        Constraint::Min(10),
    ];

    let table = Table::new(rows, widths).header(header).column_spacing(1);

    f.render_widget(table, inner);
}

/// Truncate a string to max_len, appending "..." if it exceeds.
fn truncate(s: &str, max_len: usize) -> String {
    if s.len() <= max_len {
        s.to_string()
    } else {
        format!("{}...", &s[..max_len.saturating_sub(3)])
    }
}

// ---------------------------------------------------------------------------
// Color helpers
// ---------------------------------------------------------------------------

fn percent_color(theme: &Theme, percent: f64, warn: f64, crit: f64) -> Color {
    if percent >= crit {
        theme.gauge_crit
    } else if percent >= warn {
        theme.gauge_warn
    } else {
        theme.gauge_low
    }
}

fn temp_color(theme: &Theme, celsius: f64) -> Color {
    if celsius >= 70.0 {
        theme.gauge_crit
    } else if celsius >= 60.0 {
        theme.gauge_warn
    } else {
        theme.gauge_low
    }
}
