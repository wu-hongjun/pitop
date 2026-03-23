use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyModifiers};
use std::time::Duration;

use crate::app::App;

/// Poll for keyboard events with timeout.
/// Returns true if an event was handled.
pub fn handle_events(app: &mut App, timeout: Duration) -> anyhow::Result<bool> {
    if event::poll(timeout)? {
        if let Event::Key(key) = event::read()? {
            handle_key(app, key);
            return Ok(true);
        }
    }
    Ok(false)
}

fn handle_key(app: &mut App, key: KeyEvent) {
    // Ctrl+C always quits
    if key.modifiers.contains(KeyModifiers::CONTROL) && key.code == KeyCode::Char('c') {
        app.should_quit = true;
        return;
    }

    // If kill confirmation is active, only handle y/n/Esc
    if let Some((pid, _)) = &app.kill_confirm {
        let pid = *pid;
        match key.code {
            KeyCode::Char('y') => {
                let result = unsafe { libc::kill(pid as i32, libc::SIGTERM) };
                if result == 0 {
                    app.kill_result = Some(format!("Sent SIGTERM to PID {}", pid));
                } else {
                    let errno = std::io::Error::last_os_error();
                    app.kill_result = Some(format!("Failed to kill PID {}: {}", pid, errno));
                }
                app.kill_confirm = None;
            }
            KeyCode::Char('n') | KeyCode::Esc => {
                app.kill_confirm = None;
                app.kill_result = None;
            }
            _ => {}
        }
        return;
    }

    // Clear kill_result on any keypress
    if app.kill_result.is_some() {
        app.kill_result = None;
    }

    match key.code {
        // Quit
        KeyCode::Char('q') => app.should_quit = true,

        // Tab switching by number
        KeyCode::Char('1') => app.set_tab(0),
        KeyCode::Char('2') => app.set_tab(1),
        KeyCode::Char('3') => app.set_tab(2),
        KeyCode::Char('4') => app.set_tab(3),
        KeyCode::Char('5') => app.set_tab(4),
        KeyCode::Char('6') => app.set_tab(5),

        // Tab cycling
        KeyCode::Tab => app.next_tab(),
        KeyCode::BackTab => app.prev_tab(),

        // Pause/resume
        KeyCode::Char(' ') => app.toggle_pause(),

        // Process table navigation (only on Processes tab)
        KeyCode::Char('j') | KeyCode::Down => {
            if app.active_tab == 1 && !app.processes.is_empty() {
                app.process_selected =
                    (app.process_selected + 1).min(app.processes.len().saturating_sub(1));
            }
        }
        KeyCode::Char('k') | KeyCode::Up => {
            if app.active_tab == 1 {
                app.process_selected = app.process_selected.saturating_sub(1);
            }
        }

        // Sort column cycling (Processes tab)
        KeyCode::Char('s') => {
            if app.active_tab == 1 {
                app.process_sort_column = (app.process_sort_column + 1) % 5;
            }
        }

        // Kill process (Processes tab, uppercase K)
        KeyCode::Char('K') => {
            if app.active_tab == 1 && !app.processes.is_empty() {
                let sorted = app.sorted_processes();
                if let Some(proc) = sorted.get(app.process_selected) {
                    app.kill_confirm = Some((proc.pid, proc.name.clone()));
                }
            }
        }

        _ => {}
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::board::BoardType;
    use tempfile::TempDir;

    fn test_app() -> App {
        let tmp = TempDir::new().unwrap();
        App::new(BoardType::Unknown, tmp.path(), false)
    }

    #[test]
    fn quit_on_q() {
        let mut app = test_app();
        handle_key(
            &mut app,
            KeyEvent::new(KeyCode::Char('q'), KeyModifiers::NONE),
        );
        assert!(app.should_quit);
    }

    #[test]
    fn quit_on_ctrl_c() {
        let mut app = test_app();
        handle_key(
            &mut app,
            KeyEvent::new(KeyCode::Char('c'), KeyModifiers::CONTROL),
        );
        assert!(app.should_quit);
    }

    #[test]
    fn tab_switching() {
        let mut app = test_app();
        assert_eq!(app.active_tab, 0);

        handle_key(
            &mut app,
            KeyEvent::new(KeyCode::Char('3'), KeyModifiers::NONE),
        );
        assert_eq!(app.active_tab, 2);

        handle_key(&mut app, KeyEvent::new(KeyCode::Tab, KeyModifiers::NONE));
        assert_eq!(app.active_tab, 3);

        handle_key(
            &mut app,
            KeyEvent::new(KeyCode::BackTab, KeyModifiers::SHIFT),
        );
        assert_eq!(app.active_tab, 2);
    }

    #[test]
    fn pause_toggle() {
        let mut app = test_app();
        assert!(!app.paused);

        handle_key(
            &mut app,
            KeyEvent::new(KeyCode::Char(' '), KeyModifiers::NONE),
        );
        assert!(app.paused);

        handle_key(
            &mut app,
            KeyEvent::new(KeyCode::Char(' '), KeyModifiers::NONE),
        );
        assert!(!app.paused);
    }

    #[test]
    fn tab_wraps_around() {
        let mut app = test_app();
        app.set_tab(5); // last tab
        app.next_tab();
        assert_eq!(app.active_tab, 0);

        app.prev_tab();
        assert_eq!(app.active_tab, 5);
    }
}
