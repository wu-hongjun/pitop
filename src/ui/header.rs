use crate::app::App;
use crate::collectors::throttle::ThrottleData;
use crate::util::format::format_duration;
use ratatui::layout::Rect;
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::Paragraph;
use ratatui::Frame;

/// Render the board info header line.
pub fn draw(f: &mut Frame, app: &App, area: Rect) {
    let board_name = app.profile.name();
    let uptime = format_duration(app.uptime_seconds());

    let throttle_span = throttle_indicator(&app.throttle);

    let line = Line::from(vec![
        Span::styled(
            format!(" {} ", board_name),
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        ),
        Span::raw("│ "),
        Span::styled(format!("up {}", uptime), Style::default().fg(Color::White)),
        Span::raw(" │ "),
        throttle_span,
    ]);

    f.render_widget(Paragraph::new(line), area);
}

fn throttle_indicator(throttle: &ThrottleData) -> Span<'static> {
    if throttle.is_any_active() {
        let mut flags = Vec::new();
        if throttle.is_under_voltage {
            flags.push("Under-voltage");
        }
        if throttle.is_freq_capped {
            flags.push("Freq capped");
        }
        if throttle.is_throttled {
            flags.push("Throttled");
        }
        if throttle.is_soft_temp_limit {
            flags.push("Temp limit");
        }
        Span::styled(
            format!("⚠ {}", flags.join(", ")),
            Style::default().fg(Color::Red).add_modifier(Modifier::BOLD),
        )
    } else if throttle.has_any_occurred() {
        let mut flags = Vec::new();
        if throttle.was_under_voltage {
            flags.push("under-voltage");
        }
        if throttle.was_freq_capped {
            flags.push("freq capped");
        }
        if throttle.was_throttled {
            flags.push("throttled");
        }
        if throttle.was_soft_temp_limit {
            flags.push("temp limit");
        }
        Span::styled(
            format!("⚠ {} (since boot)", flags.join(", ")),
            Style::default().fg(Color::Yellow),
        )
    } else {
        Span::styled("✓ No throttling", Style::default().fg(Color::Green))
    }
}
