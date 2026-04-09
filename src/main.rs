//! 🐝 BeCode - Autonomous AI coding agent with beautiful TUI
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

/// 🐝 BeCode - Autonomous AI coding agent
#[derive(Parser)]
#[command(name = "becode")]
#[command(author, version, about, long_about = None)]
#[command(propagate_version = true)]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,

    /// Provider to use (anthropic, openai, gemini, etc.)
    #[arg(short, long, global = true)]
    provider: Option<String>,

    /// Model to use
    #[arg(short, long, global = true)]
    model: Option<String>,

    /// Project directory
    #[arg(long, global = true)]
    project: Option<PathBuf>,

    /// Permission mode (read-only, workspace-write, danger)
    #[arg(long, global = true, default_value = "workspace-write")]
    permission: String,

    /// Maximum agent steps
    #[arg(long, global = true, default_value = "25")]
    max_steps: u32,

    /// Resume last session
    #[arg(long, global = true)]
    resume: bool,

    /// Disable TUI, use simple output
    #[arg(long, global = true)]
    no_tui: bool,

    /// Attach file (image or text) to first message
    #[arg(long, global = true)]
    attach: Option<PathBuf>,
}

#[derive(Subcommand)]
enum Commands {
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

    /// 🐝 Easter egg
    Bee,

    /// 🎉 Party mode
    Party,
}

#[derive(Subcommand)]
enum ConfigCommands {
    /// Show current configuration
    Show,
    /// Set a configuration value
    Set {
        key: String,
        value: String,
    },
    /// Open config in editor
    Edit,
}

#[derive(Subcommand)]
enum AuthCommands {
    /// Set API key for a provider
    SetKey {
        /// Provider name
        provider: String,
    },
    /// Show authentication status
    Status,
    /// Clear API key for a provider
    ClearKey {
        /// Provider name
        provider: String,
    },
}

#[derive(Subcommand)]
enum SessionCommands {
    /// List saved sessions
    List,
    /// Load a session
    Load {
        /// Session ID
        id: String,
    },
    /// Export session to markdown
    Export {
        /// Session ID
        id: String,
        /// Output file
        #[arg(short, long)]
        output: Option<PathBuf>,
    },
}

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize tracing
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::from_default_env()
                .add_directive("becode=info".parse()?),
        )
        .init();

    let cli = Cli::parse();

    // Load configuration
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
            // Default: launch TUI
            if cli.no_tui {
                println!("🐝 BeCode v{}", env!("CARGO_PKG_VERSION"));
                println!("Use --help for available commands");
            } else {
                tui::run_tui(&cli, &config).await?;
            }
        }
    }

    Ok(())
}

async fn run_oneshot(
    cli: &Cli,
    config: &config::Config,
    task: &str,
) -> Result<()> {
    println!("🐝 BeCode - Running task: {}", task);

    // TODO: Initialize agent and run task
    let _provider = cli.provider.as_deref().unwrap_or(&config.default_provider);
    let _model = cli.model.as_deref();
    let _project = cli.project.as_deref().unwrap_or(&std::env::current_dir()?);

    println!("⚡ Agent execution not yet implemented");

    Ok(())
}

async fn run_chat(_cli: &Cli, _config: &config::Config) -> Result<()> {
    println!("🐝 BeCode Chat Mode");
    println!("💬 Chat mode not yet implemented");
    Ok(())
}

fn handle_config(action: Option<ConfigCommands>, config: &config::Config) -> Result<()> {
    match action {
        Some(ConfigCommands::Show) | None => {
            println!("🐝 BeCode Configuration");
            println!("━━━━━━━━━━━━━━━━━━━━━━");
            println!("Config path: {:?}", config::Config::config_path());
            println!("Default provider: {}", config.default_provider);
            println!("Default model: {:?}", config.default_model);
            println!("Theme: {}", config.ui.theme);
        }
        Some(ConfigCommands::Set { key, value }) => {
            println!("Setting {} = {}", key, value);
            // TODO: Implement config set
        }
        Some(ConfigCommands::Edit) => {
            println!("Opening config in editor...");
            // TODO: Open config file in $EDITOR
        }
    }
    Ok(())
}

async fn handle_auth(action: AuthCommands, _config: &config::Config) -> Result<()> {
    match action {
        AuthCommands::SetKey { provider } => {
            println!("🔑 Setting API key for: {}", provider);
            // TODO: Implement secure key input
        }
        AuthCommands::Status => {
            println!("🔐 Authentication Status");
            println!("━━━━━━━━━━━━━━━━━━━━━━━");
            // TODO: Show auth status for all providers
        }
        AuthCommands::ClearKey { provider } => {
            println!("🗑️ Clearing API key for: {}", provider);
            // TODO: Implement key clearing
        }
    }
    Ok(())
}

fn handle_session(action: SessionCommands, _config: &config::Config) -> Result<()> {
    match action {
        SessionCommands::List => {
            println!("📋 Saved Sessions");
            println!("━━━━━━━━━━━━━━━━");
            // TODO: List sessions
        }
        SessionCommands::Load { id } => {
            println!("📂 Loading session: {}", id);
            // TODO: Load session
        }
        SessionCommands::Export { id, output } => {
            println!("📤 Exporting session {} to {:?}", id, output);
            // TODO: Export session
        }
    }
    Ok(())
}

async fn run_doctor(_config: &config::Config) -> Result<()> {
    println!("🏥 BeCode Doctor");
    println!("━━━━━━━━━━━━━━━");
    println!();

    // Check Rust version
    println!("✅ BeCode version: {}", env!("CARGO_PKG_VERSION"));

    // Check config
    println!("✅ Config loaded from: {:?}", config::Config::config_path());

    // TODO: Check API keys, network, etc.
    println!("⚠️  Full diagnostics not yet implemented");

    Ok(())
}

fn print_bee() {
    let bee = r#"
        \ _ /
      -= (_) =-
        /   \         🐝 Bzzzz!
          |           I'm BeCode, your coding bee!
         /|\          Ready to pollinate your codebase!
        / | \
    "#;
    println!("{}", bee);
    println!("🍯 Tip: Type 'becode party' for a surprise!");
}

fn run_party() {
    use rand::Rng;
    let mut rng = rand::thread_rng();

    let confetti = ['🎉', '🎊', '✨', '🌟', '💫', '🎈', '🎁', '🐝'];

    println!();
    for _ in 0..5 {
        let line: String = (0..40)
            .map(|_| confetti[rng.gen_range(0..confetti.len())])
            .collect();
        println!("  {}", line);
    }
    println!();
    println!("  🐝 BeCode Party Mode! 🐝");
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
