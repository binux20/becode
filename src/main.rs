//! BeCode - Autonomous AI coding agent with beautiful TUI
//!
//! BeCode is a standalone code agent that helps you write, edit, and debug code
//! using AI models from multiple providers.

mod agent;
mod attachments;
mod config;
mod permissions;
mod providers;
mod session;
mod tools;
mod tui;
mod utils;

use anyhow::Result;
use clap::{Parser, Subcommand};
use std::path::PathBuf;

/// BeCode - Autonomous AI coding agent
#[derive(Parser)]
#[command(name = "becode")]
#[command(author, version, about, long_about = None)]
#[command(propagate_version = true)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Option<Commands>,

    /// Provider to use (anthropic, openai, gemini, etc.)
    #[arg(short, long, global = true)]
    pub provider: Option<String>,

    /// Model to use
    #[arg(short, long, global = true)]
    pub model: Option<String>,

    /// Project directory
    #[arg(long, global = true)]
    pub project: Option<PathBuf>,

    /// Permission mode (read-only, workspace-write, danger)
    #[arg(long, global = true, default_value = "workspace-write")]
    pub permission: String,

    /// Maximum agent steps
    #[arg(long, global = true, default_value = "25")]
    pub max_steps: u32,

    /// Resume last session
    #[arg(long, global = true)]
    pub resume: bool,

    /// Disable TUI, use simple output
    #[arg(long, global = true)]
    pub no_tui: bool,

    /// Attach file (image or text) to first message
    #[arg(long, global = true)]
    pub attach: Option<PathBuf>,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Run a one-shot task without interactive TUI
    Run {
        /// Task description
        task: String,
    },

    /// Simple chat mode without tools
    Chat,

    /// Configuration management
    Config {
        #[command(subcommand)]
        action: Option<ConfigCommands>,
    },

    /// Authentication management
    Auth {
        #[command(subcommand)]
        action: AuthCommands,
    },

    /// Session management
    Session {
        #[command(subcommand)]
        action: SessionCommands,
    },

    /// System diagnostics
    Doctor,

    /// Easter egg
    Bee,

    /// Party mode
    Party,
}

#[derive(Subcommand)]
pub enum ConfigCommands {
    /// Show current configuration
    Show,
    /// Set a configuration value
    Set { key: String, value: String },
    /// Open config in editor
    Edit,
}

#[derive(Subcommand)]
pub enum AuthCommands {
    /// Set API key for a provider
    SetKey { provider: String },
    /// Show authentication status
    Status,
    /// Clear API key for a provider
    ClearKey { provider: String },
}

#[derive(Subcommand)]
pub enum SessionCommands {
    /// List saved sessions
    List,
    /// Load a session
    Load { id: String },
    /// Export session to markdown
    Export {
        id: String,
        #[arg(short, long)]
        output: Option<PathBuf>,
    },
}

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::from_default_env()
                .add_directive("becode=info".parse()?),
        )
        .init();

    let cli = Cli::parse();
    let config = config::Config::load()?;

    match cli.command {
        Some(Commands::Run { task }) => {
            run_oneshot(&cli, &config, &task).await?;
        }
        Some(Commands::Chat) => {
            run_chat(&cli, &config).await?;
        }
        Some(Commands::Config { action }) => {
            handle_config(action, &config)?;
        }
        Some(Commands::Auth { action }) => {
            handle_auth(action, &config).await?;
        }
        Some(Commands::Session { action }) => {
            handle_session(action, &config)?;
        }
        Some(Commands::Doctor) => {
            run_doctor(&config).await?;
        }
        Some(Commands::Bee) => {
            print_bee();
        }
        Some(Commands::Party) => {
            run_party();
        }
        None => {
            if cli.no_tui {
                println!("BeCode v{}", env!("CARGO_PKG_VERSION"));
                println!("Use --help for available commands");
            } else {
                tui::run_tui(&cli, &config).await?;
            }
        }
    }

    Ok(())
}

async fn run_oneshot(cli: &Cli, config: &config::Config, task: &str) -> Result<()> {
    println!("BeCode - Running task: {}", task);
    let _provider = cli.provider.as_deref().unwrap_or(&config.default_provider);
    let _model = cli.model.as_deref();
    let _project = cli.project.as_deref().unwrap_or(&std::env::current_dir()?);
    println!("Agent execution not yet implemented");
    Ok(())
}

async fn run_chat(_cli: &Cli, _config: &config::Config) -> Result<()> {
    println!("BeCode Chat Mode");
    println!("Chat mode not yet implemented");
    Ok(())
}

fn handle_config(action: Option<ConfigCommands>, config: &config::Config) -> Result<()> {
    match action {
        Some(ConfigCommands::Show) | None => {
            println!("BeCode Configuration");
            println!("Config path: {:?}", config::Config::config_path());
            println!("Default provider: {}", config.default_provider);
            println!("Default model: {:?}", config.default_model);
            println!("Theme: {}", config.ui.theme);
        }
        Some(ConfigCommands::Set { key, value }) => {
            println!("Setting {} = {}", key, value);
        }
        Some(ConfigCommands::Edit) => {
            println!("Opening config in editor...");
        }
    }
    Ok(())
}

async fn handle_auth(action: AuthCommands, _config: &config::Config) -> Result<()> {
    match action {
        AuthCommands::SetKey { provider } => {
            println!("Setting API key for: {}", provider);
        }
        AuthCommands::Status => {
            println!("Authentication Status");
        }
        AuthCommands::ClearKey { provider } => {
            println!("Clearing API key for: {}", provider);
        }
    }
    Ok(())
}

fn handle_session(action: SessionCommands, _config: &config::Config) -> Result<()> {
    match action {
        SessionCommands::List => {
            println!("Saved Sessions");
        }
        SessionCommands::Load { id } => {
            println!("Loading session: {}", id);
        }
        SessionCommands::Export { id, output } => {
            println!("Exporting session {} to {:?}", id, output);
        }
    }
    Ok(())
}

async fn run_doctor(_config: &config::Config) -> Result<()> {
    println!("BeCode Doctor");
    println!("BeCode version: {}", env!("CARGO_PKG_VERSION"));
    println!("Config loaded from: {:?}", config::Config::config_path());
    println!("Full diagnostics not yet implemented");
    Ok(())
}

fn print_bee() {
    println!(r#"
        \ _ /
      -= (_) =-
        /   \         Bzzzz!
          |           I'm BeCode, your coding bee!
         /|\          Ready to pollinate your codebase!
        / | \
    "#);
    println!("Tip: Type 'becode party' for a surprise!");
}

fn run_party() {
    use rand::Rng;
    let mut rng = rand::thread_rng();
    let confetti = ['*', '+', 'o', '.', '~'];
    println!();
    for _ in 0..5 {
        let line: String = (0..40)
            .map(|_| confetti[rng.gen_range(0..confetti.len())])
            .collect();
        println!("  {}", line);
    }
    println!();
    println!("  BeCode Party Mode!");
    println!("  Thanks for using BeCode!");
    println!();
    for _ in 0..5 {
        let line: String = (0..40)
            .map(|_| confetti[rng.gen_range(0..confetti.len())])
            .collect();
        println!("  {}", line);
    }
    println!();
}
