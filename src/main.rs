mod app;
mod board;
mod collectors;
mod event;
mod ui;
mod util;

use anyhow::Result;
use app::App;
use crossterm::event::{DisableMouseCapture, EnableMouseCapture};
use crossterm::execute;
use crossterm::terminal::{
    disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen,
};
use ratatui::backend::CrosstermBackend;
use ratatui::Terminal;
use std::io;
use std::path::Path;
use std::time::Duration;

#[tokio::main]
async fn main() -> Result<()> {
    // Install panic hook that restores terminal before printing error
    let original_hook = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |panic_info| {
        let _ = restore_terminal();
        original_hook(panic_info);
    }));

    let root = Path::new("/");
    let board_type = board::detect(root);

    let mut app = App::new(board_type, root, false);

    // Run TUI
    let result = run_tui(&mut app).await;

    // Ensure terminal is restored even if run_tui fails
    restore_terminal()?;

    result
}

async fn run_tui(app: &mut App) -> Result<()> {
    // Setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let tick_rate = Duration::from_millis(1000);

    // Initial data collection
    app.tick().await;

    loop {
        // Render
        terminal.draw(|f| ui::draw(f, app))?;

        // Handle input events (non-blocking with short timeout)
        let timeout = Duration::from_millis(50);
        event::handle_events(app, timeout)?;

        if app.should_quit {
            break;
        }

        // Tick at configured interval
        // We use a simple sleep approach — good enough for a 1s refresh
        tokio::time::sleep(tick_rate.saturating_sub(timeout)).await;
        app.tick().await;
    }

    Ok(())
}

fn restore_terminal() -> Result<()> {
    disable_raw_mode()?;
    execute!(io::stdout(), LeaveAlternateScreen, DisableMouseCapture)?;
    Ok(())
}
