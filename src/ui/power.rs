use crate::app::App;
use crate::board::VoltageSource;
use crate::util::format::format_watts;
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::{Color, Modifier, Style};
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

/// Pi 5 layout: PMIC rails table + total wattage sparkline + PCIe + PoE
fn draw_pmic_layout(f: &mut Frame, app: &App, area: Rect) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3), // Summary (total watts + input voltage)
            Constraint::Min(8),    // PMIC rails table
            Constraint::Length(4), // Wattage sparkline
            Constraint::Length(5), // PCIe + PoE
        ])
        .split(area);

    draw_power_summary(f, app, chunks[0]);
    draw_pmic_table(f, app, chunks[1]);
    draw_power_sparkline(f, app, chunks[2]);

    let bottom = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
        .split(chunks[3]);
    draw_pcie_section(f, app, bottom[0]);
    draw_poe_section(f, app, bottom[1]);
}

/// Pi 4B / Zero 2W layout: voltage table + PoE (if applicable)
fn draw_measure_volts_layout(f: &mut Frame, app: &App, area: Rect) {
    if app.profile.has_poe() {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Min(6),    // Voltage table
                Constraint::Length(5), // PoE
            ])
            .split(area);

        draw_voltage_table(f, app, chunks[0]);
        draw_poe_section(f, app, chunks[1]);
    } else {
        draw_voltage_table(f, app, area);
    }
}

/// Unknown board: minimal layout
fn draw_minimal_layout(f: &mut Frame, _app: &App, area: Rect) {
    let block = Block::default()
        .title(" Power ")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::DarkGray));
    let msg = Paragraph::new("  No voltage monitoring available on this board")
        .style(Style::default().fg(Color::DarkGray))
        .block(block);
    f.render_widget(msg, area);
}

fn draw_power_summary(f: &mut Frame, app: &App, area: Rect) {
    let block = Block::default()
        .title(" Power Summary ")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Yellow));

    let inner = block.inner(area);
    f.render_widget(block, area);

    if let Some(ref pmic) = app.power.pmic {
        let line = Line::from(vec![
            Span::styled("  Total: ", Style::default().fg(Color::DarkGray)),
            Span::styled(
                format_watts(pmic.estimated_real_watts),
                Style::default()
                    .fg(Color::White)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::raw("  "),
            Span::styled("Input: ", Style::default().fg(Color::DarkGray)),
            Span::styled(
                format!("{:.2}V / {:.3}A", pmic.ext5v_voltage, pmic.ext5v_current),
                Style::default().fg(Color::Green),
            ),
        ]);
        f.render_widget(Paragraph::new(line), inner);
    } else {
        let msg = Paragraph::new("  Waiting for PMIC data...")
            .style(Style::default().fg(Color::DarkGray));
        f.render_widget(msg, inner);
    }
}

fn draw_pmic_table(f: &mut Frame, app: &App, area: Rect) {
    let block = Block::default()
        .title(" PMIC Rails ")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Yellow));

    let pmic = match &app.power.pmic {
        Some(p) => p,
        None => {
            let msg = Paragraph::new("  No PMIC data yet")
                .style(Style::default().fg(Color::DarkGray))
                .block(block);
            f.render_widget(msg, area);
            return;
        }
    };

    let header = Row::new(["Rail", "Voltage", "Current", "Power"].iter().map(|h| {
        Cell::from(*h).style(
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        )
    }))
    .height(1);

    let rows: Vec<Row> = pmic
        .rails
        .iter()
        .map(|rail| {
            Row::new(vec![
                Cell::from(rail.name.clone()).style(Style::default().fg(Color::Cyan)),
                Cell::from(format!("{:.4} V", rail.voltage)),
                Cell::from(format!("{:.4} A", rail.current)),
                Cell::from(format_watts(rail.power)).style(Style::default().fg(Color::White)),
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
                .border_style(Style::default().fg(Color::Yellow)),
        )
        .data(&data)
        .max(max)
        .style(Style::default().fg(Color::Yellow));
    f.render_widget(spark, area);
}

fn draw_voltage_table(f: &mut Frame, app: &App, area: Rect) {
    let block = Block::default()
        .title(" Voltages ")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Yellow));

    if app.power.voltages.is_empty() {
        let msg = Paragraph::new("  Waiting for voltage data...")
            .style(Style::default().fg(Color::DarkGray))
            .block(block);
        f.render_widget(msg, area);
        return;
    }

    let header = Row::new(["Rail", "Voltage"].iter().map(|h| {
        Cell::from(*h).style(
            Style::default()
                .fg(Color::Yellow)
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
                Cell::from(v.name.clone()).style(Style::default().fg(Color::Cyan)),
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
    let block = Block::default()
        .title(" PCIe ")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Blue));

    let inner = block.inner(area);
    f.render_widget(block, area);

    if app.pcie.devices.is_empty() {
        let msg = Paragraph::new("  No PCIe devices").style(Style::default().fg(Color::DarkGray));
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
                    Style::default().fg(Color::DarkGray),
                ),
                Span::styled(
                    format!("{} x{}", dev.gen_label, dev.current_width),
                    Style::default()
                        .fg(if dev.downgraded {
                            Color::Yellow
                        } else {
                            Color::Green
                        })
                        .add_modifier(Modifier::BOLD),
                ),
                Span::styled(
                    format!(" ({}){}", dev.current_speed, downgrade_indicator),
                    Style::default().fg(Color::DarkGray),
                ),
            ])
        })
        .collect();

    f.render_widget(Paragraph::new(lines), inner);
}

fn draw_poe_section(f: &mut Frame, app: &App, area: Rect) {
    let block = Block::default()
        .title(" PoE ")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Magenta));

    let inner = block.inner(area);
    f.render_widget(block, area);

    if !app.poe.available {
        let msg =
            Paragraph::new("  No PoE HAT detected").style(Style::default().fg(Color::DarkGray));
        f.render_widget(msg, inner);
        return;
    }

    let status_color = if app.poe.online {
        Color::Green
    } else {
        Color::DarkGray
    };
    let status_text = if app.poe.online { "Active" } else { "Offline" };

    let lines = vec![
        Line::from(vec![
            Span::styled("  Status: ", Style::default().fg(Color::DarkGray)),
            Span::styled(
                status_text,
                Style::default()
                    .fg(status_color)
                    .add_modifier(Modifier::BOLD),
            ),
        ]),
        Line::from(vec![
            Span::styled("  Current: ", Style::default().fg(Color::DarkGray)),
            Span::styled(
                format!("{:.2}A", app.poe.current_amps),
                Style::default().fg(Color::White),
            ),
            Span::styled(
                format!(" / {:.2}A max", app.poe.current_max_amps),
                Style::default().fg(Color::DarkGray),
            ),
        ]),
    ];

    f.render_widget(Paragraph::new(lines), inner);
}
