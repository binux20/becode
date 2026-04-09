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

/// TUI settings passed from CLI
pub struct TuiSettings {
    pub provider: Option<String>,
    pub model: Option<String>,
    pub permission: String,
}

/// Run the TUI application
pub async fn run_tui(settings: &TuiSettings, config: &Config) -> Result<()> {
    println!("BeCode TUI");
    println!("============");
    println!();
    println!("TUI is under construction!");
    println!();
    println!("Current settings:");
    println!("  Provider: {}", settings.provider.as_deref().unwrap_or(&config.default_provider));
    println!("  Model: {}", settings.model.as_deref().or(config.default_model.as_deref()).unwrap_or("(default)"));
    println!("  Permission: {}", settings.permission);
    println!();
    println!("For now, use 'becode run <task>' for one-shot execution.");
    println!("Full TUI coming soon!");

    Ok(())
}
