mod disk;
mod header;
mod network;
mod overview;
mod power;
mod processes;
mod system;
mod widgets;

use crate::app::{App, TAB_NAMES};
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Paragraph, Tabs};
use ratatui::Frame;

/// Render the complete UI frame.
pub fn draw(f: &mut Frame, app: &App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1), // Tab bar
            Constraint::Min(0),    // Main content
            Constraint::Length(1), // Footer
        ])
        .split(f.area());

    draw_tab_bar(f, app, chunks[0]);
    draw_active_tab(f, app, chunks[1]);
    draw_footer(f, app, chunks[2]);
}

fn draw_tab_bar(f: &mut Frame, app: &App, area: Rect) {
    let titles: Vec<Line> = TAB_NAMES
        .iter()
        .enumerate()
        .map(|(i, t)| {
            Line::from(Span::styled(
                format!(" {}:{} ", i + 1, t),
                if i == app.active_tab {
                    Style::default()
                        .fg(Color::Yellow)
                        .add_modifier(Modifier::BOLD)
                } else {
                    Style::default().fg(Color::DarkGray)
                },
            ))
        })
        .collect();

    let tabs = Tabs::new(titles)
        .select(app.active_tab)
        .style(Style::default().fg(Color::White))
        .divider(Span::raw("│"));

    f.render_widget(tabs, area);
}

fn draw_active_tab(f: &mut Frame, app: &App, area: Rect) {
    match app.active_tab {
        0 => overview::draw(f, app, area),
        1 => processes::draw(f, app, area),
        2 => power::draw(f, app, area),
        3 => network::draw(f, app, area),
        4 => disk::draw(f, app, area),
        5 => system::draw(f, app, area),
        _ => {}
    }
}

fn draw_footer(f: &mut Frame, app: &App, area: Rect) {
    let pause_indicator = if app.paused { " [PAUSED]" } else { "" };
    let hints = format!(
        " q:Quit  1-6:Tabs  Tab:Next  Space:Pause  ?:Help{}",
        pause_indicator
    );
    let footer = Paragraph::new(hints).style(Style::default().fg(Color::DarkGray));
    f.render_widget(footer, area);
}
