//! Main TUI application state

use crate::config::Config;
use std::path::PathBuf;

/// TUI Application state
pub struct App {
    /// Current project directory
    pub project: PathBuf,
    /// Provider name
    pub provider: String,
    /// Model name
    pub model: Option<String>,
    /// Chat history
    pub messages: Vec<ChatMessage>,
    /// Input buffer
    pub input: String,
    /// Is running
    pub running: bool,
    /// Currently thinking
    pub thinking: bool,
}

/// A message in the chat
pub struct ChatMessage {
    pub role: MessageRole,
    pub content: String,
    pub tool_calls: Vec<ToolCallDisplay>,
}

pub enum MessageRole {
    User,
    Assistant,
    System,
}

pub struct ToolCallDisplay {
    pub tool: String,
    pub status: ToolStatus,
    pub output_preview: Option<String>,
    pub diff: Option<String>,
}

pub enum ToolStatus {
    Pending,
    Running,
    Success,
    Error(String),
}

impl App {
    pub fn new(config: &Config) -> Self {
        Self {
            project: config
                .project_dir
                .clone()
                .unwrap_or_else(|| std::env::current_dir().unwrap_or_default()),
            provider: config.default_provider.clone(),
            model: config.default_model.clone(),
            messages: Vec::new(),
            input: String::new(),
            running: true,
            thinking: false,
        }
    }
}
