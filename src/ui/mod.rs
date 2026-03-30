mod disk;
mod header;
mod help;
mod network;
mod overview;
mod power;
mod processes;
mod system;
pub mod theme;
mod widgets;

use crate::app::{App, TAB_NAMES};
use crate::util::update_check::UpdateStatus;
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Paragraph, Tabs};
use ratatui::Frame;

/// Render the complete UI frame.
pub fn draw(f: &mut Frame, app: &App) {
    let full_area = f.area();

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1), // Tab bar
            Constraint::Min(0),    // Main content
            Constraint::Length(1), // Footer
        ])
        .split(full_area);

    draw_tab_bar(f, app, chunks[0]);
    draw_active_tab(f, app, chunks[1]);
    draw_footer(f, app, chunks[2]);

    // Help overlay (renders on top of everything)
    if app.show_help {
        help::draw(f, app);
    }
}

fn draw_tab_bar(f: &mut Frame, app: &App, area: Rect) {
    let theme = &app.theme;
    let titles: Vec<Line> = TAB_NAMES
        .iter()
        .enumerate()
        .map(|(i, t)| {
            Line::from(Span::styled(
                format!(" {}:{} ", i + 1, t),
                if i == app.active_tab {
                    Style::default()
                        .fg(theme.title)
                        .add_modifier(Modifier::BOLD)
                } else {
                    Style::default().fg(theme.text_dim)
                },
            ))
        })
        .collect();

    let tabs = Tabs::new(titles)
        .select(app.active_tab)
        .style(Style::default().fg(theme.text))
        .divider(Span::styled("│", Style::default().fg(theme.text_dim)));

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
    let theme = &app.theme;
    let pause_indicator = if app.paused { " [PAUSED]" } else { "" };

    let stress_indicator = if let Some(ref stress) = app.stress {
        if stress.is_running() {
            let workers = stress.num_workers();
            let max = stress.max_workers();
            let elapsed = stress.elapsed().unwrap_or_default();
            let total_secs = elapsed.as_secs();
            let mins = total_secs / 60;
            let secs = total_secs % 60;
            format!(
                " [STRESS {}/{} workers {:02}:{:02}]",
                workers, max, mins, secs
            )
        } else {
            " [STRESS STOPPED]".to_string()
        }
    } else {
        String::new()
    };

    let stress_hint = if app.stress.is_some() {
        "  Ctrl+S:Stress"
    } else {
        ""
    };

    let worker_hint = if app.stress.as_ref().is_some_and(|s| s.is_running()) {
        "  Ctrl+\u{2191}/\u{2193}:Workers"
    } else {
        ""
    };

    let hints_prefix = format!(
        " q:Quit  1-6:Tabs  Tab:Next  Space:Pause  t:Theme  ?:Help{}{}{}",
        stress_hint, worker_hint, pause_indicator
    );

    // Build styled spans: base hints in dim, stress indicator in color
    let mut spans = vec![Span::styled(
        hints_prefix,
        Style::default().fg(theme.text_dim),
    )];

    if let Some(ref stress) = app.stress {
        if stress.is_running() {
            let workers = stress.num_workers();
            let initial = stress.initial_workers();
            let color = if workers < initial {
                theme.gauge_low
            } else if workers == initial {
                theme.gauge_warn
            } else {
                theme.gauge_crit
            };
            spans.push(Span::styled(
                stress_indicator,
                Style::default().fg(color).add_modifier(Modifier::BOLD),
            ));
        } else {
            spans.push(Span::styled(
                stress_indicator,
                Style::default().fg(theme.text_dim),
            ));
        }
    }

    // If an update is available, show the install command instead of normal hints
    if let Some(ref handle) = app.update_status {
        if let Ok(guard) = handle.try_lock() {
            if let UpdateStatus::Available(ref ver) = *guard {
                let update_spans = vec![
                    Span::styled(
                        format!(" Update v{} available: ", ver),
                        Style::default()
                            .fg(theme.gauge_warn)
                            .add_modifier(Modifier::BOLD),
                    ),
                    Span::styled(
                        "curl -sL https://pitop.hongjunwu.com/install.sh | sh",
                        Style::default().fg(theme.text),
                    ),
                ];
                f.render_widget(Paragraph::new(Line::from(update_spans)), area);
                return;
            }
        }
    }

    let footer = Paragraph::new(Line::from(spans));
    f.render_widget(footer, area);
}
