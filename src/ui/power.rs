use crate::app::App;
use crate::board::VoltageSource;
use crate::util::format::{format_temp, format_watts};
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Cell, Paragraph, Row, Sparkline, Table};
use ratatui::Frame;

pub fn draw(f: &mut Frame, app: &App, area: Rect) {
    match app.profile.voltage_source() {
        VoltageSource::Pmic => draw_pmic_layout(f, app, area),
        VoltageSource::MeasureVolts => draw_measure_volts_layout(f, app, area),
        VoltageSource::None => draw_minimal_layout(f, app, area),
    }
}

/// Pi 5 layout: PMIC rails table + total wattage sparkline + PCIe + PoE + GPU
fn draw_pmic_layout(f: &mut Frame, app: &App, area: Rect) {
    let gpu_height = if app.gpu.available { 5 } else { 0 };
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),          // Summary (total watts + input voltage)
            Constraint::Min(8),             // PMIC rails table
            Constraint::Length(4),          // Wattage sparkline
            Constraint::Length(gpu_height), // GPU section
            Constraint::Length(5),          // PCIe + PoE
        ])
        .split(area);

    draw_power_summary(f, app, chunks[0]);
    draw_pmic_table(f, app, chunks[1]);
    draw_power_sparkline(f, app, chunks[2]);
    if app.gpu.available {
        draw_gpu_section(f, app, chunks[3]);
    }

    let bottom = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
        .split(chunks[4]);
    draw_pcie_section(f, app, bottom[0]);
    draw_poe_section(f, app, bottom[1]);
}

/// Pi 4B / Zero 2W layout: voltage table + GPU + PoE (if applicable)
fn draw_measure_volts_layout(f: &mut Frame, app: &App, area: Rect) {
    let gpu_height = if app.gpu.available { 5 } else { 0 };
    let has_poe = app.profile.has_poe();
    let poe_height = if has_poe { 5 } else { 0 };

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Min(6),             // Voltage table
            Constraint::Length(gpu_height), // GPU section
            Constraint::Length(poe_height), // PoE
        ])
        .split(area);

    draw_voltage_table(f, app, chunks[0]);
    if app.gpu.available {
        draw_gpu_section(f, app, chunks[1]);
    }
    if has_poe {
        draw_poe_section(f, app, chunks[2]);
    }
}

/// Unknown board: minimal layout
fn draw_minimal_layout(f: &mut Frame, app: &App, area: Rect) {
    let theme = &app.theme;
    let block = Block::default()
        .title(" Power ")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(theme.text_dim));
    let msg = Paragraph::new("  No voltage monitoring available on this board")
        .style(Style::default().fg(theme.text_dim))
        .block(block);
    f.render_widget(msg, area);
}

fn draw_power_summary(f: &mut Frame, app: &App, area: Rect) {
    let theme = &app.theme;
    let block = Block::default()
        .title(" Power Summary ")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(theme.power_border));

    let inner = block.inner(area);
    f.render_widget(block, area);

    if let Some(ref pmic) = app.power.pmic {
        let line = Line::from(vec![
            Span::styled("  Total: ", Style::default().fg(theme.text_dim)),
            Span::styled(
                format_watts(pmic.estimated_real_watts),
                Style::default().fg(theme.text).add_modifier(Modifier::BOLD),
            ),
            Span::raw("  "),
            Span::styled("Input: ", Style::default().fg(theme.text_dim)),
            Span::styled(
                format!("{:.2}V / {:.3}A", pmic.ext5v_voltage, pmic.ext5v_current),
                Style::default().fg(theme.gauge_low),
            ),
        ]);
        f.render_widget(Paragraph::new(line), inner);
    } else {
        let msg =
            Paragraph::new("  Waiting for PMIC data...").style(Style::default().fg(theme.text_dim));
        f.render_widget(msg, inner);
    }
}

fn draw_pmic_table(f: &mut Frame, app: &App, area: Rect) {
    let theme = &app.theme;
    let block = Block::default()
        .title(" PMIC Rails ")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(theme.power_border));

    let pmic = match &app.power.pmic {
        Some(p) => p,
        None => {
            let msg = Paragraph::new("  No PMIC data yet")
                .style(Style::default().fg(theme.text_dim))
                .block(block);
            f.render_widget(msg, area);
            return;
        }
    };

    let header = Row::new(["Rail", "Voltage", "Current", "Power"].iter().map(|h| {
        Cell::from(*h).style(
            Style::default()
                .fg(theme.title)
                .add_modifier(Modifier::BOLD),
        )
    }))
    .height(1);

    let rows: Vec<Row> = pmic
        .rails
        .iter()
        .map(|rail| {
            Row::new(vec![
                Cell::from(rail.name.clone()).style(Style::default().fg(theme.border_highlight)),
                Cell::from(format!("{:.4} V", rail.voltage)),
                Cell::from(format!("{:.4} A", rail.current)),
                Cell::from(format_watts(rail.power)).style(Style::default().fg(theme.text)),
            ])
        })
        .collect();

    let table = Table::new(
        rows,
        [
            Constraint::Min(14),    // Rail
            Constraint::Length(12), // Voltage
            Constraint::Length(12), // Current
            Constraint::Length(10), // Power
        ],
    )
    .header(header)
    .block(block);

    f.render_widget(table, area);
}

fn draw_power_sparkline(f: &mut Frame, app: &App, area: Rect) {
    let theme = &app.theme;
    let data: Vec<u64> = app
        .power_history
        .as_slice()
        .iter()
        .map(|v| (*v * 100.0).max(0.0) as u64) // Scale for sparkline resolution
        .collect();

    let max = data.iter().copied().max().unwrap_or(1).max(1);

    let current = app
        .power
        .pmic
        .as_ref()
        .map(|p| format_watts(p.estimated_real_watts))
        .unwrap_or_default();

    let spark = Sparkline::default()
        .block(
            Block::default()
                .title(format!(" Power {} ", current))
                .borders(Borders::ALL)
                .border_style(Style::default().fg(theme.sparkline_power)),
        )
        .data(&data)
        .max(max)
        .style(Style::default().fg(theme.sparkline_power));
    f.render_widget(spark, area);
}

fn draw_voltage_table(f: &mut Frame, app: &App, area: Rect) {
    let theme = &app.theme;
    let block = Block::default()
        .title(" Voltages ")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(theme.power_border));

    if app.power.voltages.is_empty() {
        let msg = Paragraph::new("  Waiting for voltage data...")
            .style(Style::default().fg(theme.text_dim))
            .block(block);
        f.render_widget(msg, area);
        return;
    }

    let header = Row::new(["Rail", "Voltage"].iter().map(|h| {
        Cell::from(*h).style(
            Style::default()
                .fg(theme.title)
                .add_modifier(Modifier::BOLD),
        )
    }))
    .height(1);

    let rows: Vec<Row> = app
        .power
        .voltages
        .iter()
        .map(|v| {
            Row::new(vec![
                Cell::from(v.name.clone()).style(Style::default().fg(theme.border_highlight)),
                Cell::from(format!("{:.4} V", v.voltage)),
            ])
        })
        .collect();

    let table = Table::new(rows, [Constraint::Min(14), Constraint::Length(12)])
        .header(header)
        .block(block);

    f.render_widget(table, area);
}

fn draw_pcie_section(f: &mut Frame, app: &App, area: Rect) {
    let theme = &app.theme;
    let block = Block::default()
        .title(" PCIe ")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(theme.cpu_border));

    let inner = block.inner(area);
    f.render_widget(block, area);

    if app.pcie.devices.is_empty() {
        let msg = Paragraph::new("  No PCIe devices").style(Style::default().fg(theme.text_dim));
        f.render_widget(msg, inner);
        return;
    }

    let lines: Vec<Line> = app
        .pcie
        .devices
        .iter()
        .map(|dev| {
            let downgrade_indicator = if dev.downgraded { " (downgraded)" } else { "" };
            Line::from(vec![
                Span::styled(
                    format!("  {} ", dev.address),
                    Style::default().fg(theme.text_dim),
                ),
                Span::styled(
                    format!("{} x{}", dev.gen_label, dev.current_width),
                    Style::default()
                        .fg(if dev.downgraded {
                            theme.gauge_warn
                        } else {
                            theme.gauge_low
                        })
                        .add_modifier(Modifier::BOLD),
                ),
                Span::styled(
                    format!(" ({}){}", dev.current_speed, downgrade_indicator),
                    Style::default().fg(theme.text_dim),
                ),
            ])
        })
        .collect();

    f.render_widget(Paragraph::new(lines), inner);
}

fn draw_poe_section(f: &mut Frame, app: &App, area: Rect) {
    let theme = &app.theme;
    let block = Block::default()
        .title(" PoE ")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(theme.mem_border));

    let inner = block.inner(area);
    f.render_widget(block, area);

    if !app.poe.available {
        let msg =
            Paragraph::new("  No PoE HAT detected").style(Style::default().fg(theme.text_dim));
        f.render_widget(msg, inner);
        return;
    }

    let status_color = if app.poe.online {
        theme.gauge_low
    } else {
        theme.text_dim
    };
    let status_text = if app.poe.online { "Active" } else { "Offline" };

    let lines = vec![
        Line::from(vec![
            Span::styled("  Status: ", Style::default().fg(theme.text_dim)),
            Span::styled(
                status_text,
                Style::default()
                    .fg(status_color)
                    .add_modifier(Modifier::BOLD),
            ),
        ]),
        Line::from(vec![
            Span::styled("  Current: ", Style::default().fg(theme.text_dim)),
            Span::styled(
                format!("{:.2}A", app.poe.current_amps),
                Style::default().fg(theme.text),
            ),
            Span::styled(
                format!(" / {:.2}A max", app.poe.current_max_amps),
                Style::default().fg(theme.text_dim),
            ),
        ]),
    ];

    f.render_widget(Paragraph::new(lines), inner);
}

fn draw_gpu_section(f: &mut Frame, app: &App, area: Rect) {
    let theme = &app.theme;
    let block = Block::default()
        .title(" GPU ")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(theme.sparkline_power));

    let inner = block.inner(area);
    f.render_widget(block, area);

    let gpu = &app.gpu;

    let mut lines = vec![Line::from(vec![
        Span::styled("  Freq: ", Style::default().fg(theme.text_dim)),
        Span::styled(
            format!("{} MHz", gpu.frequency_mhz),
            Style::default().fg(theme.text).add_modifier(Modifier::BOLD),
        ),
        Span::styled("  Mem: ", Style::default().fg(theme.text_dim)),
        Span::styled(
            format!("{} MB", gpu.memory_mb),
            Style::default().fg(theme.text),
        ),
        Span::styled("  Temp: ", Style::default().fg(theme.text_dim)),
        Span::styled(
            format_temp(gpu.temperature_celsius),
            Style::default().fg(theme.text),
        ),
    ])];

    // Codec status
    if !gpu.codecs.is_empty() {
        let mut codec_spans: Vec<Span> = vec![Span::styled(
            "  Codecs: ",
            Style::default().fg(theme.text_dim),
        )];
        for (i, (name, enabled)) in gpu.codecs.iter().enumerate() {
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
