// Policy: No unwrap() or expect() in production code.
// CI enforces via: cargo clippy -- -D clippy::unwrap_used -D clippy::expect_used

mod app;
mod board;
mod collectors;
mod config;
mod event;
mod stress;
mod ui;
mod util;

use anyhow::Result;
use app::App;
use clap::Parser;
use config::Config;
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
use stress::StressTest;
use ui::theme::Theme;

#[derive(Parser)]
#[command(
    name = "pitop",
    version,
    about = "Terminal-based system monitor for Raspberry Pi"
)]
struct Cli {
    /// Refresh interval in milliseconds (minimum 100)
    #[arg(short = 'i', long, value_parser = clap::value_parser!(u64).range(100..))]
    interval: Option<u64>,

    /// Starting tab number (1-6)
    #[arg(short, long, value_parser = clap::value_parser!(u8).range(1..=6))]
    tab: Option<u8>,

    /// Path to config file
    #[arg(short, long)]
    config: Option<String>,

    /// Force board type (auto/pi5/pi4b/zero2w)
    #[arg(long, default_value = "auto")]
    board: String,

    /// Color theme (default/monochrome/solarized)
    #[arg(long)]
    theme: Option<String>,

    /// Enable verbose error output
    #[arg(short, long)]
    verbose: bool,

    /// Start CPU stress test on launch
    #[arg(long)]
    stress: bool,

    /// Number of stress test worker threads (defaults to CPU core count)
    #[arg(long, value_parser = clap::value_parser!(u16).range(1..))]
    stress_workers: Option<u16>,

    /// Print a fully-commented sample config.toml to stdout and exit
    #[arg(long)]
    generate_config: bool,

    /// Load config, validate it, print results, and exit
    #[arg(long)]
    config_check: bool,
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

    // --generate-config: print sample config and exit
    if cli.generate_config {
        print!("{}", config::generate_sample());
        return Ok(());
    }

    // --config-check: load, validate, print result, and exit
    if cli.config_check {
        let config = Config::load_raw(cli.config.as_deref().map(Path::new))?;
        match config.validate() {
            Ok(()) => {
                println!("Configuration is valid.");
                return Ok(());
            }
            Err(e) => {
                eprintln!("Configuration error: {e}");
                std::process::exit(1);
            }
        }
    }

    // Load configuration: CLI --config path > default XDG path > built-in defaults
    let config = Config::load(cli.config.as_deref().map(Path::new))?;

    // CLI args override config file values
    let interval = cli.interval.unwrap_or(config.general.interval_ms);
    let tab = cli.tab.unwrap_or(config.general.default_tab);
    let theme_name: String = cli
        .theme
        .clone()
        .unwrap_or_else(|| config.general.theme.clone());

    let root = Path::new("/");

    // Determine board type: use override if specified, otherwise auto-detect
    let board_type = if cli.board == "auto" {
        board::detect(root)
    } else {
        board::parse_board_override(&cli.board)?
    };

    let mut app = App::new(board_type, root, cli.verbose, config);

    // Set the initial theme, and sync theme_index so cycling starts from here
    if theme_name == "custom" {
        if let Some(ref ct) = app.config.custom_theme {
            app.theme = Theme::from_config(ct);
        }
    } else {
        app.theme = Theme::from_name(&theme_name).unwrap_or_default();
    }
    // Find the matching index in theme_names for the starting theme
    if let Some(idx) = app.theme_names.iter().position(|n| n == &theme_name) {
        app.theme_index = idx;
    }

    // Set starting tab (convert 1-based to 0-based index)
    app.set_tab(tab.saturating_sub(1) as usize);

    let tick_rate = Duration::from_millis(interval);

    // Start stress test if requested
    if cli.stress {
        let worker_count = cli
            .stress_workers
            .map(|n| n as usize)
            .unwrap_or_else(|| stress::num_cpus_from_proc(root).max(1));
        let mut st = StressTest::new(worker_count);
        st.start();
        app.stress = Some(st);
    }

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
