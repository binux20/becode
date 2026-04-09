//! TUI (Terminal User Interface) for BeCode
//!
//! Beautiful terminal interface inspired by OpenCode.

mod app;
mod events;
mod themes;
mod mascot;
mod widgets;

pub use app::App;

use crate::config::Config;
use anyhow::Result;

/// Run the TUI application
pub async fn run_tui(cli: &crate::Cli, config: &Config) -> Result<()> {
    // For now, just print a message
    // TODO: Implement full TUI
    println!("🐝 BeCode TUI");
    println!("━━━━━━━━━━━━━");
    println!();
    println!("TUI is under construction!");
    println!();
    println!("Current settings:");
    println!("  Provider: {}", cli.provider.as_deref().unwrap_or(&config.default_provider));
    println!("  Model: {}", cli.model.as_deref().or(config.default_model.as_deref()).unwrap_or("(default)"));
    println!("  Permission: {}", cli.permission);
    println!();
    println!("For now, use 'becode run <task>' for one-shot execution.");
    println!("Full TUI coming soon! 🚧");

    Ok(())
}
