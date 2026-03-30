use crate::app::App;
use crate::collectors::throttle::ThrottleData;
use crate::ui::theme::Theme;
use crate::util::format::format_duration;
use crate::util::update_check::UpdateStatus;
use ratatui::layout::Rect;
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::Paragraph;
use ratatui::Frame;

/// Render the board info header line.
pub fn draw(f: &mut Frame, app: &App, area: Rect) {
    let theme = &app.theme;
    let board_name = app.profile.name();
    let uptime = format_duration(app.uptime_seconds());

    let throttle_span = throttle_indicator(theme, &app.throttle);

    let mut spans = vec![
        Span::styled(
            format!(" {} ", board_name),
            Style::default()
                .fg(theme.border_highlight)
                .add_modifier(Modifier::BOLD),
        ),
        Span::raw("│ "),
        Span::styled(format!("up {}", uptime), Style::default().fg(theme.text)),
        Span::raw(" │ "),
        throttle_span,
    ];

    // Show update dot in header — full instructions are in the footer
    if let Some(ref handle) = app.update_status {
        if let Ok(guard) = handle.try_lock() {
            if let UpdateStatus::Available(ref ver) = *guard {
                spans.push(Span::raw(" │ "));
                spans.push(Span::styled(
                    format!("v{} available \u{2193}", ver),
                    Style::default()
                        .fg(theme.gauge_warn)
                        .add_modifier(Modifier::BOLD),
                ));
            }
        }
    }

    f.render_widget(Paragraph::new(Line::from(spans)), area);
}

fn throttle_indicator(theme: &Theme, throttle: &ThrottleData) -> Span<'static> {
    if !throttle.available {
        return Span::styled("— Throttle: N/A", Style::default().fg(theme.text_dim));
    }

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
            Style::default()
                .fg(theme.throttle_crit)
                .add_modifier(Modifier::BOLD),
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
            Style::default().fg(theme.throttle_warn),
        )
    } else {
        Span::styled("✓ No throttling", Style::default().fg(theme.throttle_ok))
    }
}
