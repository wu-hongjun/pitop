use crate::app::App;
use crate::util::format::{format_bytes, format_duration, format_freq_mhz};
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Paragraph};
use ratatui::Frame;

/// Render the System info tab.
pub fn draw(f: &mut Frame, app: &App, area: Rect) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(5), // Board info
            Constraint::Length(7), // System info (kernel, os, hostname, arch, uptime)
            Constraint::Length(6), // CPU info
            Constraint::Length(4), // Memory info
            Constraint::Min(4),    // Capabilities
        ])
        .split(area);

    draw_board_section(f, app, chunks[0]);
    draw_system_section(f, app, chunks[1]);
    draw_cpu_section(f, app, chunks[2]);
    draw_memory_section(f, app, chunks[3]);
    draw_capabilities_section(f, app, chunks[4]);
}

fn draw_board_section(f: &mut Frame, app: &App, area: Rect) {
    let theme = &app.theme;
    let block = Block::default()
        .title(" Board ")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(theme.border_highlight));

    let inner = block.inner(area);
    f.render_widget(block, area);

    let model = if app.system_info.model_name.is_empty() || app.system_info.model_name == "Unknown"
    {
        app.profile.name().to_string()
    } else {
        app.system_info.model_name.clone()
    };

    let board_type = app.profile.name();

    let lines = vec![
        info_line(app, "Model", &model),
        info_line(app, "Board", board_type),
        info_line(app, "SoC", app.profile.soc_name()),
    ];

    f.render_widget(Paragraph::new(lines), inner);
}

fn draw_system_section(f: &mut Frame, app: &App, area: Rect) {
    let theme = &app.theme;
    let block = Block::default()
        .title(" System ")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(theme.net_border));

    let inner = block.inner(area);
    f.render_widget(block, area);

    let os_display = if app.system_info.os_name.is_empty() {
        "N/A".to_string()
    } else if app.system_info.os_version.is_empty() {
        app.system_info.os_name.clone()
    } else {
        format!("{} {}", app.system_info.os_name, app.system_info.os_version)
    };

    let kernel = if app.system_info.kernel_version.is_empty() {
        "N/A"
    } else {
        &app.system_info.kernel_version
    };

    let hostname = if app.system_info.hostname.is_empty() {
        "N/A"
    } else {
        &app.system_info.hostname
    };

    let arch = if app.system_info.architecture.is_empty() {
        "N/A"
    } else {
        &app.system_info.architecture
    };

    let uptime = format_duration(app.uptime_seconds());

    let lines = vec![
        info_line(app, "Kernel", kernel),
        info_line(app, "OS", &os_display),
        info_line(app, "Hostname", hostname),
        info_line(app, "Arch", arch),
        info_line(app, "Uptime", &uptime),
    ];

    f.render_widget(Paragraph::new(lines), inner);
}

fn draw_cpu_section(f: &mut Frame, app: &App, area: Rect) {
    let theme = &app.theme;
    let block = Block::default()
        .title(" CPU ")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(theme.cpu_border));

    let inner = block.inner(area);
    f.render_widget(block, area);

    let core_count = app.cpu.cores.len().to_string();

    let freq_range = if app.cpu.min_frequency_khz > 0 && app.cpu.max_frequency_khz > 0 {
        format!(
            "{} - {}",
            format_freq_mhz(app.cpu.min_frequency_khz),
            format_freq_mhz(app.cpu.max_frequency_khz)
        )
    } else {
        "N/A".to_string()
    };

    let governor = if app.cpu.governor.is_empty() {
        "N/A".to_string()
    } else {
        app.cpu.governor.clone()
    };

    let cpu_model = if app.system_info.cpu_model.is_empty() {
        "N/A"
    } else {
        &app.system_info.cpu_model
    };

    let lines = vec![
        info_line(app, "Model", cpu_model),
        info_line(app, "Cores", &core_count),
        info_line(app, "Freq Range", &freq_range),
        info_line(app, "Governor", &governor),
    ];

    f.render_widget(Paragraph::new(lines), inner);
}

fn draw_memory_section(f: &mut Frame, app: &App, area: Rect) {
    let theme = &app.theme;
    let block = Block::default()
        .title(" Memory ")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(theme.mem_border));

    let inner = block.inner(area);
    f.render_widget(block, area);

    let total_ram = if app.memory.total_bytes > 0 {
        format_bytes(app.memory.total_bytes)
    } else {
        "N/A".to_string()
    };

    let total_swap = if app.memory.swap_total_bytes > 0 {
        format_bytes(app.memory.swap_total_bytes)
    } else {
        "None".to_string()
    };

    let lines = vec![
        info_line(app, "Total RAM", &total_ram),
        info_line(app, "Total Swap", &total_swap),
    ];

    f.render_widget(Paragraph::new(lines), inner);
}

fn draw_capabilities_section(f: &mut Frame, app: &App, area: Rect) {
    let theme = &app.theme;
    let block = Block::default()
        .title(" Capabilities ")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(theme.border_highlight));

    let inner = block.inner(area);
    f.render_widget(block, area);

    let capabilities = [
        ("PMIC", app.profile.has_pmic()),
        ("Fan Control", app.profile.has_fan()),
        ("PCIe", app.profile.has_pcie()),
        ("PoE", app.profile.has_poe()),
    ];

    let mut lines: Vec<Line> = Vec::new();

    for (name, available) in &capabilities {
        let (indicator, color) = if *available {
            ("Available", theme.gauge_low)
        } else {
            ("N/A", theme.text_dim)
        };

        lines.push(Line::from(vec![
            Span::styled(
                format!("  {:<14}", name),
                Style::default().fg(theme.text).add_modifier(Modifier::BOLD),
            ),
            Span::styled(indicator, Style::default().fg(color)),
        ]));
    }

    f.render_widget(Paragraph::new(lines), inner);
}

/// Helper to create a labeled info line with consistent formatting.
fn info_line<'a>(app: &'a App, label: &'a str, value: &'a str) -> Line<'a> {
    let theme = &app.theme;
    Line::from(vec![
        Span::styled(
            format!("  {:<14}", label),
            Style::default()
                .fg(theme.text_dim)
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled(value, Style::default().fg(theme.text)),
    ])
}
