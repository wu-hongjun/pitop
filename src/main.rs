// Policy: No unwrap() or expect() in production code.
// CI enforces via: cargo clippy -- -D clippy::unwrap_used -D clippy::expect_used

mod app;
mod board;
mod collectors;
mod event;
mod ui;
mod util;

use anyhow::Result;
use app::App;
use clap::Parser;
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

#[derive(Parser)]
#[command(
    name = "pitop",
    version,
    about = "Terminal-based system monitor for Raspberry Pi"
)]
struct Cli {
    /// Refresh interval in milliseconds (minimum 100)
    #[arg(short = 'i', long, default_value = "1000", value_parser = clap::value_parser!(u64).range(100..))]
    interval: u64,

    /// Starting tab number (1-6)
    #[arg(short, long, default_value = "1", value_parser = clap::value_parser!(u8).range(1..=6))]
    tab: u8,

    /// Force board type (auto/pi5/pi4b/zero2w)
    #[arg(long, default_value = "auto")]
    board: String,

    /// Enable verbose error output
    #[arg(short, long)]
    verbose: bool,
}

#[tokio::main]
async fn main() -> Result<()> {
    // Install panic hook that restores terminal before printing error
    let original_hook = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |panic_info| {
        let _ = restore_terminal();
        original_hook(panic_info);
    }));

    let cli = Cli::parse();

    let root = Path::new("/");

    // Determine board type: use override if specified, otherwise auto-detect
    let board_type = if cli.board == "auto" {
        board::detect(root)
    } else {
        board::parse_board_override(&cli.board)?
    };

    let mut app = App::new(board_type, root, cli.verbose);

    // Set starting tab (convert 1-based CLI arg to 0-based index)
    app.set_tab((cli.tab - 1) as usize);

    let tick_rate = Duration::from_millis(cli.interval);

    // Run TUI
    let result = run_tui(&mut app, tick_rate).await;

    // Ensure terminal is restored even if run_tui fails
    restore_terminal()?;

    result
}

async fn run_tui(app: &mut App, tick_rate: Duration) -> Result<()> {
    // Setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // Initial data collection
    app.tick().await;
    let mut last_tick = std::time::Instant::now();

    loop {
        // Render
        terminal.draw(|f| ui::draw(f, app))?;

        // Wait for input events or tick timeout — blocks to avoid busy-spinning
        let poll_timeout = tick_rate.saturating_sub(last_tick.elapsed());
        let had_event = event::handle_events(app, poll_timeout)?;

        // Drain any remaining queued events without blocking
        if had_event {
            while event::handle_events(app, Duration::ZERO)? {}
        }

        if app.should_quit {
            break;
        }

        // Tick collectors when interval has elapsed
        if last_tick.elapsed() >= tick_rate {
            app.tick().await;
            last_tick = std::time::Instant::now();
        }
    }

    Ok(())
}

fn restore_terminal() -> Result<()> {
    disable_raw_mode()?;
    execute!(io::stdout(), LeaveAlternateScreen, DisableMouseCapture)?;
    Ok(())
}
