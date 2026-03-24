use crate::app::App;
use crate::ui::theme::Theme;
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Clear, Paragraph};
use ratatui::Frame;

pub fn draw(f: &mut Frame, app: &App) {
    let area = f.area();
    let theme = &app.theme;

    // Calculate centered popup area (60% width, 70% height, minimum 15 rows)
    let popup_area = centered_rect(60, 70, 15, area);

    // Clear the area behind the popup
    f.render_widget(Clear, popup_area);

    let mut help_lines: Vec<Line<'static>> = vec![
        // Title
        Line::from(Span::styled(
            "Keyboard Shortcuts",
            Style::default()
                .fg(theme.title)
                .add_modifier(Modifier::BOLD),
        )),
        Line::from(""),
        // Section: Navigation
        section_header(theme, "Navigation"),
        help_line(theme, "1-6", "Switch tab"),
        help_line(theme, "Tab / Shift+Tab", "Next / prev tab"),
        help_line(theme, "q / Ctrl+C", "Quit"),
        Line::from(""),
        // Section: Display
        section_header(theme, "Display"),
        help_line(theme, "Space", "Pause / resume"),
        help_line(theme, "t", "Cycle theme"),
        help_line(theme, "?", "Toggle help"),
        Line::from(""),
        // Section: Processes Tab
        section_header(theme, "Processes Tab"),
        help_line(theme, "j / k / arrows", "Navigate"),
        help_line(theme, "s", "Cycle sort"),
        help_line(theme, "K", "Kill process"),
        Line::from(""),
    ];

    // Section: Stress Testing (only when stress mode is available)
    if app.stress.is_some() {
        help_lines.push(section_header(theme, "Stress Testing"));
        help_lines.push(help_line(theme, "Ctrl+S", "Toggle stress"));
        help_lines.push(help_line(theme, "Ctrl+Up/Down", "Add/remove workers"));
        help_lines.push(Line::from(""));
    }

    // Config location
    help_lines.push(Line::from(Span::styled(
        "  Config: ~/.config/pitop/config.toml".to_string(),
        Style::default().fg(theme.text),
    )));
    help_lines.push(Line::from(""));

    // Version footer
    help_lines.push(Line::from(Span::styled(
        format!("pitop v{}", env!("CARGO_PKG_VERSION")),
        Style::default()
            .fg(theme.text_dim)
            .add_modifier(Modifier::ITALIC),
    )));

    let total_lines = help_lines.len();

    // Apply scroll offset: skip `help_scroll` lines from the top
    // The visible area inside the block is popup height minus 2 (top + bottom border)
    let inner_height = popup_area.height.saturating_sub(2) as usize;
    let max_scroll = total_lines.saturating_sub(inner_height);
    let scroll = app.help_scroll.min(max_scroll);

    let block = Block::default()
        .title(" Help ")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(theme.border))
        .title_style(Style::default().fg(theme.title))
        .style(Style::default().bg(theme.highlight_bg));

    let paragraph = Paragraph::new(help_lines)
        .block(block)
        .scroll((scroll as u16, 0));

    f.render_widget(paragraph, popup_area);
}

fn section_header(theme: &Theme, title: &str) -> Line<'static> {
    Line::from(Span::styled(
        format!("  {}", title),
        Style::default()
            .fg(theme.title)
            .add_modifier(Modifier::BOLD),
    ))
}

fn help_line(theme: &Theme, key: &str, desc: &str) -> Line<'static> {
    Line::from(vec![
        Span::styled(
            format!("  {:>20}  ", key),
            Style::default()
                .fg(theme.border_highlight)
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled(desc.to_string(), Style::default().fg(theme.text)),
    ])
}

/// Create a centered rectangle with given percentage width and height,
/// enforcing a minimum height in rows.
fn centered_rect(percent_x: u16, percent_y: u16, min_height: u16, area: Rect) -> Rect {
    // Calculate desired height as percentage, enforce minimum
    let desired_height = (area.height as u32 * percent_y as u32 / 100) as u16;
    let height = desired_height.max(min_height).min(area.height);

    let vertical_pad = area.height.saturating_sub(height) / 2;

    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(vertical_pad),
            Constraint::Length(height),
            Constraint::Min(0),
        ])
        .split(area);

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(popup_layout[1])[1]
}
