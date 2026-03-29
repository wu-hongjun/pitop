use crate::app::App;
use crate::ui::header;
use crate::ui::theme::Theme;
use crate::util::format::{format_bytes, format_bytes_per_sec, format_freq_mhz, format_temp};
use ratatui::layout::{Alignment, Constraint, Direction, Layout, Rect};
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
            Constraint::Percentage(50), // Top 2x2 gauge grid
            Constraint::Length(5),      // Info row (compact panels)
            Constraint::Min(4),         // Sparklines
        ])
        .split(chunks[1]);

    draw_gauge_grid(f, app, main[0]);
    draw_info_row(f, app, main[1]);
    draw_sparklines(f, app, main[2]);
}

// ---------------------------------------------------------------------------
// Top 2x2 gauge grid: CPU | Memory / Temperature | GPU
// ---------------------------------------------------------------------------

fn draw_gauge_grid(f: &mut Frame, app: &App, area: Rect) {
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

    draw_cpu_gauge(f, app, top_cols[0]);
    draw_memory_gauge(f, app, top_cols[1]);
    draw_temp_gauge(f, app, bottom_cols[0]);
    draw_gpu_gauge(f, app, bottom_cols[1]);
}

/// CPU gauge block -- aggregate usage bar with core count, frequency, governor.
fn draw_cpu_gauge(f: &mut Frame, app: &App, area: Rect) {
    let theme = &app.theme;
    let usage = app.cpu.aggregate_usage_percent.min(100.0);
    let core_count = app.cpu.cores.len();
    let freq = format_freq_mhz(app.cpu.frequency_khz);

    let title = format!(" CPU {} Cores {:.1}% ({}) ", core_count, usage, freq);

    let color = percent_color(theme, usage, 60.0, 85.0);

    let block = Block::default()
        .title(
            Line::from(Span::styled(
                title,
                Style::default()
                    .fg(theme.gauge_low)
                    .add_modifier(Modifier::BOLD),
            ))
            .alignment(Alignment::Left),
        )
        .borders(Borders::ALL)
        .border_style(Style::default().fg(theme.gauge_low));

    let inner = block.inner(area);
    f.render_widget(block, area);

    // Layout: gauge bar takes most space, detail line below
    let rows = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(1), Constraint::Length(1)])
        .split(inner);

    let gauge = Gauge::default()
        .gauge_style(Style::default().fg(color))
        .label(format!("{:.1}%", usage))
        .ratio((usage / 100.0).min(1.0));
    f.render_widget(gauge, rows[0]);

    // Governor line
    let governor_text = if app.cpu.governor.is_empty() {
        "N/A".to_string()
    } else {
        app.cpu.governor.clone()
    };
    let detail = Line::from(vec![
        Span::styled("Governor: ", Style::default().fg(theme.text_dim)),
        Span::styled(governor_text, Style::default().fg(theme.text)),
    ]);
    f.render_widget(Paragraph::new(detail), rows[1]);
}

/// Memory gauge block -- RAM bar with percentage, used/total, swap.
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

    let block = Block::default()
        .title(
            Line::from(Span::styled(
                " Memory ",
                Style::default()
                    .fg(theme.gauge_low)
                    .add_modifier(Modifier::BOLD),
            ))
            .alignment(Alignment::Left),
        )
        .borders(Borders::ALL)
        .border_style(Style::default().fg(theme.gauge_low));

    let inner = block.inner(area);
    f.render_widget(block, area);

    let rows = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Min(1),    // Gauge bar
            Constraint::Length(1), // used / total
            Constraint::Length(1), // swap info
        ])
        .split(inner);

    let gauge = Gauge::default()
        .gauge_style(Style::default().fg(color))
        .label(format!("{:.1}%", pct))
        .ratio(ratio);
    f.render_widget(gauge, rows[0]);

    // Used / Total line
    let mem_line = Line::from(vec![Span::styled(
        format!(
            "{} / {}",
            format_bytes(mem.used_bytes),
            format_bytes(mem.total_bytes)
        ),
        Style::default().fg(theme.text),
    )]);
    f.render_widget(Paragraph::new(mem_line), rows[1]);

    // Swap line
    let swap_text = if mem.swap_total_bytes > 0 {
        format!(
            "Swap: {} / {}",
            format_bytes(mem.swap_used_bytes),
            format_bytes(mem.swap_total_bytes)
        )
    } else {
        "Swap: N/A".to_string()
    };
    let swap_line = Line::from(Span::styled(swap_text, Style::default().fg(theme.text_dim)));
    f.render_widget(Paragraph::new(swap_line), rows[2]);
}

/// Temperature gauge block -- SoC temp bar (0-85 C), fan info.
fn draw_temp_gauge(f: &mut Frame, app: &App, area: Rect) {
    let theme = &app.theme;
    let temp = app.thermal.soc_temp_celsius;
    let color = temp_color(theme, temp);
    let ratio = (temp / 85.0).clamp(0.0, 1.0);

    let block = Block::default()
        .title(
            Line::from(Span::styled(
                " Temperature ",
                Style::default()
                    .fg(theme.gauge_low)
                    .add_modifier(Modifier::BOLD),
            ))
            .alignment(Alignment::Left),
        )
        .borders(Borders::ALL)
        .border_style(Style::default().fg(theme.gauge_low));

    let inner = block.inner(area);
    f.render_widget(block, area);

    let rows = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(1), Constraint::Length(1)])
        .split(inner);

    let gauge = Gauge::default()
        .gauge_style(Style::default().fg(color))
        .label(format_temp(temp))
        .ratio(ratio);
    f.render_widget(gauge, rows[0]);

    // Fan info or extra thermal zones
    let detail = if app.fan.available {
        Line::from(vec![Span::styled(
            format!("Fan: {} RPM ({:.0}%)", app.fan.rpm, app.fan.pwm_percent),
            Style::default().fg(theme.border_highlight),
        )])
    } else {
        // Show additional thermal zone if available
        let extra: Vec<Span> = app
            .thermal
            .zones
            .iter()
            .filter(|z| !z.zone_name.contains("cpu"))
            .take(2)
            .enumerate()
            .flat_map(|(i, z)| {
                let mut spans = Vec::new();
                if i > 0 {
                    spans.push(Span::raw("  "));
                }
                spans.push(Span::styled(
                    format!("{}: {}", z.zone_name, format_temp(z.temp_celsius)),
                    Style::default().fg(theme.text_dim),
                ));
                spans
            })
            .collect();
        if extra.is_empty() {
            Line::from(Span::styled(
                "No fan detected",
                Style::default().fg(theme.text_dim),
            ))
        } else {
            Line::from(extra)
        }
    };
    f.render_widget(Paragraph::new(detail), rows[1]);
}

/// GPU gauge block -- frequency bar, memory, temp, codecs.
/// Falls back to load averages when GPU data is unavailable.
fn draw_gpu_gauge(f: &mut Frame, app: &App, area: Rect) {
    let theme = &app.theme;

    if app.gpu.available {
        let freq = app.gpu.frequency_mhz;
        let ratio = (freq as f64 / 1500.0).clamp(0.0, 1.0);

        let block = Block::default()
            .title(
                Line::from(Span::styled(
                    " GPU ",
                    Style::default()
                        .fg(theme.gauge_low)
                        .add_modifier(Modifier::BOLD),
                ))
                .alignment(Alignment::Left),
            )
            .borders(Borders::ALL)
            .border_style(Style::default().fg(theme.gauge_low));

        let inner = block.inner(area);
        f.render_widget(block, area);

        // Determine how many detail lines we need
        let has_codecs = !app.gpu.codecs.is_empty();
        let has_video_decoder = app.gpu.video_decoder.is_some();
        let has_second_line = has_codecs || has_video_decoder;
        let detail_lines: u16 = if has_second_line { 2 } else { 1 };

        let rows = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Min(1), Constraint::Length(detail_lines)])
            .split(inner);

        let gauge = Gauge::default()
            .gauge_style(Style::default().fg(theme.sparkline_power))
            .label(format!("{} MHz", freq))
            .ratio(ratio);
        f.render_widget(gauge, rows[0]);

        // Detail area
        let detail_rows = Layout::default()
            .direction(Direction::Vertical)
            .constraints(if has_second_line {
                vec![Constraint::Length(1), Constraint::Length(1)]
            } else {
                vec![Constraint::Length(1)]
            })
            .split(rows[1]);

        // Line 1: memory + temp
        let mem_text = if app.gpu.shared_memory {
            format!("{} MHz / Shared", freq)
        } else {
            format!("{} MHz / {} MB", freq, app.gpu.memory_mb)
        };
        let info_line = Line::from(vec![
            Span::styled(mem_text, Style::default().fg(theme.text)),
            Span::styled("  ", Style::default()),
            Span::styled(
                format!("Temp: {}", format_temp(app.gpu.temperature_celsius)),
                Style::default().fg(temp_color(theme, app.gpu.temperature_celsius)),
            ),
        ]);
        f.render_widget(Paragraph::new(info_line), detail_rows[0]);

        // Line 2: video decoder info or codecs
        if has_second_line && detail_rows.len() > 1 {
            if let Some(ref decoder) = app.gpu.video_decoder {
                // Pi 5: show hardware decoder description instead of codec X marks
                let decoder_line = Line::from(vec![
                    Span::styled("Video: ", Style::default().fg(theme.text_dim)),
                    Span::styled(decoder.clone(), Style::default().fg(theme.gauge_low)),
                ]);
                f.render_widget(Paragraph::new(decoder_line), detail_rows[1]);
            } else if has_codecs {
                let mut codec_spans: Vec<Span> = Vec::new();
                for (i, (name, enabled)) in app.gpu.codecs.iter().enumerate() {
                    if i > 0 {
                        codec_spans.push(Span::raw("  "));
                    }
                    let (symbol, c) = if *enabled {
                        ("\u{2713}", theme.gauge_low)
                    } else {
                        ("\u{2717}", theme.text_dim)
                    };
                    codec_spans.push(Span::styled(
                        format!("{}: {}", name, symbol),
                        Style::default().fg(c),
                    ));
                }
                f.render_widget(Paragraph::new(Line::from(codec_spans)), detail_rows[1]);
            }
        }
    } else {
        // GPU unavailable -- show load averages instead
        let block = Block::default()
            .title(
                Line::from(Span::styled(
                    " Load Averages ",
                    Style::default()
                        .fg(theme.gauge_low)
                        .add_modifier(Modifier::BOLD),
                ))
                .alignment(Alignment::Left),
            )
            .borders(Borders::ALL)
            .border_style(Style::default().fg(theme.gauge_low));

        let inner = block.inner(area);
        f.render_widget(block, area);

        let load_ratio = (app.cpu.load_avg_1 / (app.cpu.cores.len().max(1) as f64)).clamp(0.0, 1.0);
        let load_color = percent_color(theme, load_ratio * 100.0, 60.0, 85.0);

        let rows = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Min(1), Constraint::Length(1)])
            .split(inner);

        let gauge = Gauge::default()
            .gauge_style(Style::default().fg(load_color))
            .label(format!("{:.2}", app.cpu.load_avg_1))
            .ratio(load_ratio);
        f.render_widget(gauge, rows[0]);

        let detail = Line::from(vec![
            Span::styled("1m: ", Style::default().fg(theme.text_dim)),
            Span::styled(
                format!("{:.2}", app.cpu.load_avg_1),
                Style::default().fg(theme.text),
            ),
            Span::styled("  5m: ", Style::default().fg(theme.text_dim)),
            Span::styled(
                format!("{:.2}", app.cpu.load_avg_5),
                Style::default().fg(theme.text),
            ),
            Span::styled("  15m: ", Style::default().fg(theme.text_dim)),
            Span::styled(
                format!("{:.2}", app.cpu.load_avg_15),
                Style::default().fg(theme.text),
            ),
        ]);
        f.render_widget(Paragraph::new(detail), rows[1]);
    }
}

// ---------------------------------------------------------------------------
// Info row: 4 compact bordered panels side by side
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

    draw_info_thermal(f, app, cols[0]);
    draw_info_power(f, app, cols[1]);
    draw_info_network(f, app, cols[2]);
    draw_info_disk(f, app, cols[3]);
}

/// Thermal info panel: SoC temp, other zones, load averages.
fn draw_info_thermal(f: &mut Frame, app: &App, area: Rect) {
    let theme = &app.theme;
    let block = Block::default()
        .title(Span::styled(
            " Thermal ",
            Style::default().fg(theme.gauge_low),
        ))
        .borders(Borders::ALL)
        .border_style(Style::default().fg(theme.gauge_low));

    let inner = block.inner(area);
    f.render_widget(block, area);

    let mut lines: Vec<Line> = Vec::new();

    // SoC temp
    let soc_color = temp_color(theme, app.thermal.soc_temp_celsius);
    lines.push(Line::from(vec![
        Span::styled("SoC: ", Style::default().fg(theme.text_dim)),
        Span::styled(
            format_temp(app.thermal.soc_temp_celsius),
            Style::default().fg(soc_color).add_modifier(Modifier::BOLD),
        ),
    ]));

    // NVMe temperature if available
    if let Some(nvme_temp) = app.thermal.nvme_temp_celsius {
        lines.push(Line::from(vec![
            Span::styled("NVMe: ", Style::default().fg(theme.text_dim)),
            Span::styled(
                format_temp(nvme_temp),
                Style::default().fg(temp_color(theme, nvme_temp)),
            ),
        ]));
    } else {
        // Extra thermal zones (skip cpu-like ones, take first one that fits)
        for zone in app
            .thermal
            .zones
            .iter()
            .filter(|z| !z.zone_name.contains("cpu"))
            .take(1)
        {
            lines.push(Line::from(vec![Span::styled(
                format!("{}: {}", zone.zone_name, format_temp(zone.temp_celsius)),
                Style::default().fg(theme.text),
            )]));
        }
    }

    // Load average
    lines.push(Line::from(vec![
        Span::styled("Load: ", Style::default().fg(theme.text_dim)),
        Span::styled(
            format!("{:.2}", app.cpu.load_avg_1),
            Style::default().fg(theme.text),
        ),
    ]));

    let max_lines = inner.height as usize;
    lines.truncate(max_lines);
    f.render_widget(Paragraph::new(lines), inner);
}

/// Power info panel: total wattage or voltage readings.
fn draw_info_power(f: &mut Frame, app: &App, area: Rect) {
    let theme = &app.theme;
    let block = Block::default()
        .title(Span::styled(
            " Power ",
            Style::default().fg(theme.gauge_low),
        ))
        .borders(Borders::ALL)
        .border_style(Style::default().fg(theme.gauge_low));

    let inner = block.inner(area);
    f.render_widget(block, area);

    let mut lines: Vec<Line> = Vec::new();

    if let Some(ref pmic) = app.power.pmic {
        lines.push(Line::from(Span::styled(
            format!("{:.1} W", pmic.estimated_real_watts),
            Style::default().fg(theme.text).add_modifier(Modifier::BOLD),
        )));
        if pmic.ext5v_voltage > 0.0 {
            lines.push(Line::from(vec![
                Span::styled(
                    format!("{:.2}V", pmic.ext5v_voltage),
                    Style::default().fg(theme.text),
                ),
                Span::styled(" OK", Style::default().fg(theme.gauge_low)),
            ]));
        }
    } else if !app.power.voltages.is_empty() {
        for v in app.power.voltages.iter().take(3) {
            lines.push(Line::from(vec![Span::styled(
                format!("{}: {:.2}V", v.name, v.voltage),
                Style::default().fg(theme.text),
            )]));
        }
    } else {
        lines.push(Line::from(Span::styled(
            "N/A",
            Style::default().fg(theme.text_dim),
        )));
    }

    let max_lines = inner.height as usize;
    lines.truncate(max_lines);
    f.render_widget(Paragraph::new(lines), inner);
}

/// Network info panel: total throughput, primary interface.
fn draw_info_network(f: &mut Frame, app: &App, area: Rect) {
    let theme = &app.theme;
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
        Span::styled("\u{2193} ", Style::default().fg(theme.net_border)),
        Span::styled(
            format_bytes_per_sec(app.network.total_rx_bytes_per_sec),
            Style::default().fg(theme.text),
        ),
    ]));

    lines.push(Line::from(vec![
        Span::styled("\u{2191} ", Style::default().fg(theme.gauge_warn)),
        Span::styled(
            format_bytes_per_sec(app.network.total_tx_bytes_per_sec),
            Style::default().fg(theme.text),
        ),
    ]));

    // Primary interface name (first "up" interface, or first non-lo)
    let primary = app
        .network
        .interfaces
        .iter()
        .find(|i| i.operstate == "up" && i.name != "lo")
        .or_else(|| app.network.interfaces.iter().find(|i| i.name != "lo"));
    if let Some(iface) = primary {
        lines.push(Line::from(Span::styled(
            iface.name.clone(),
            Style::default().fg(theme.text_dim),
        )));
    }

    let max_lines = inner.height as usize;
    lines.truncate(max_lines);
    f.render_widget(Paragraph::new(lines), inner);
}

/// Disk info panel: root partition usage, I/O rates.
fn draw_info_disk(f: &mut Frame, app: &App, area: Rect) {
    let theme = &app.theme;
    let block = Block::default()
        .title(Span::styled(" Disk ", Style::default().fg(theme.gauge_low)))
        .borders(Borders::ALL)
        .border_style(Style::default().fg(theme.gauge_low));

    let inner = block.inner(area);
    f.render_widget(block, area);

    let mut lines: Vec<Line> = Vec::new();

    // Root partition (or first partition)
    let root_part = app
        .disk
        .partitions
        .iter()
        .find(|p| p.mountpoint == "/")
        .or_else(|| app.disk.partitions.first());

    if let Some(part) = root_part {
        lines.push(Line::from(Span::styled(
            format!(
                "{}: {}/{}",
                part.mountpoint,
                format_bytes(part.used_bytes),
                format_bytes(part.total_bytes)
            ),
            Style::default().fg(theme.text),
        )));
        lines.push(Line::from(Span::styled(
            format!("{:.0}% used", part.usage_percent),
            Style::default().fg(percent_color(theme, part.usage_percent, 70.0, 90.0)),
        )));
    } else {
        lines.push(Line::from(Span::styled(
            "N/A",
            Style::default().fg(theme.text_dim),
        )));
    }

    // I/O rates from first block device
    if let Some(io) = app.disk.io_stats.first() {
        lines.push(Line::from(vec![
            Span::styled("R:", Style::default().fg(theme.text_dim)),
            Span::styled(
                format_bytes_per_sec(io.read_bytes_per_sec),
                Style::default().fg(theme.text),
            ),
            Span::styled(" W:", Style::default().fg(theme.text_dim)),
            Span::styled(
                format_bytes_per_sec(io.write_bytes_per_sec),
                Style::default().fg(theme.text),
            ),
        ]));
    }

    let max_lines = inner.height as usize;
    lines.truncate(max_lines);
    f.render_widget(Paragraph::new(lines), inner);
}

// ---------------------------------------------------------------------------
// Sparklines row
// ---------------------------------------------------------------------------

fn draw_sparklines(f: &mut Frame, app: &App, area: Rect) {
    let theme = &app.theme;

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
                .border_style(Style::default().fg(theme.sparkline_cpu)),
        )
        .data(&cpu_data)
        .max(100)
        .style(Style::default().fg(theme.sparkline_cpu));

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
                .border_style(Style::default().fg(theme.sparkline_mem)),
        )
        .data(&mem_data)
        .max(100)
        .style(Style::default().fg(theme.sparkline_mem));

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
                .border_style(Style::default().fg(theme.sparkline_temp)),
        )
        .data(&temp_data)
        .max(100)
        .style(Style::default().fg(theme.sparkline_temp));

    if app.gpu.available {
        // 4 sparklines in a row: CPU | MEM | TEMP | GPU
        let cols = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Percentage(25),
                Constraint::Percentage(25),
                Constraint::Percentage(25),
                Constraint::Percentage(25),
            ])
            .split(area);

        f.render_widget(cpu_spark, cols[0]);
        f.render_widget(mem_spark, cols[1]);
        f.render_widget(temp_spark, cols[2]);

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
        f.render_widget(gpu_spark, cols[3]);
    } else {
        // 3 sparklines in a row: CPU | MEM | TEMP
        let cols = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Percentage(33),
                Constraint::Percentage(34),
                Constraint::Percentage(33),
            ])
            .split(area);

        f.render_widget(cpu_spark, cols[0]);
        f.render_widget(mem_spark, cols[1]);
        f.render_widget(temp_spark, cols[2]);
    }
}

// ---------------------------------------------------------------------------
// Color helpers
// ---------------------------------------------------------------------------

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
