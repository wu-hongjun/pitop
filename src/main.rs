use anyhow::Result;

fn main() -> Result<()> {
    println!("pitop v{} — starting...", env!("CARGO_PKG_VERSION"));
    println!("Board detection and TUI not yet implemented.");
    println!("See CLAUDE.md and bmad-artifacts/ to begin development.");
    Ok(())
}
