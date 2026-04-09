//! Main TUI application state

use crate::config::Config;
use crate::permissions::Permission;
use std::path::PathBuf;

/// TUI Application state
pub struct App {
    /// Current project directory
    pub project: PathBuf,
    /// Provider name
    pub provider_name: String,
    /// Model name
    pub model: Option<String>,
    /// Chat history
    pub messages: Vec<ChatMessage>,
    /// Input buffer
    pub input: String,
    /// Input cursor position
    pub cursor_pos: usize,
    /// Is running
    pub running: bool,
    /// Currently thinking/processing
    pub thinking: bool,
    /// Scroll offset for chat
    pub scroll_offset: usize,
    /// Current panel focus
    pub focus: PanelFocus,
    /// Status message
    pub status: String,
    /// Permission level
    pub permission: Permission,
}

#[derive(Clone, Copy, PartialEq)]
pub enum PanelFocus {
    Input,
    Chat,
    FileTree,
}

/// A message in the chat
#[derive(Clone)]
pub struct ChatMessage {
    pub role: MessageRole,
    pub content: String,
    pub tool_calls: Vec<ToolCallDisplay>,
    pub timestamp: chrono::DateTime<chrono::Local>,
}

#[derive(Clone, Copy, PartialEq)]
pub enum MessageRole {
    User,
    Assistant,
    System,
}

#[derive(Clone)]
pub struct ToolCallDisplay {
    pub tool: String,
    pub args_preview: String,
    pub status: ToolStatus,
    pub output_preview: Option<String>,
    pub diff: Option<String>,
    pub duration_ms: Option<u32>,
}

#[derive(Clone)]
pub enum ToolStatus {
    Pending,
    Running,
    Success,
    Error(String),
}

impl App {
    pub fn new(config: &Config, provider: Option<String>, model: Option<String>, permission: Permission) -> Self {
        Self {
            project: config
                .project_dir
                .clone()
                .unwrap_or_else(|| std::env::current_dir().unwrap_or_default()),
            provider_name: provider.unwrap_or_else(|| config.default_provider.clone()),
            model: model.or_else(|| config.default_model.clone()),
            messages: vec![ChatMessage {
                role: MessageRole::System,
                content: "Welcome to BeCode! Type your request and press Enter.".to_string(),
                tool_calls: vec![],
                timestamp: chrono::Local::now(),
            }],
            input: String::new(),
            cursor_pos: 0,
            running: true,
            thinking: false,
            scroll_offset: 0,
            focus: PanelFocus::Input,
            status: "Ready".to_string(),
            permission,
        }
    }

    pub fn add_user_message(&mut self, content: String) {
        self.messages.push(ChatMessage {
            role: MessageRole::User,
            content,
            tool_calls: vec![],
            timestamp: chrono::Local::now(),
        });
    }

    pub fn add_assistant_message(&mut self, content: String) {
        self.messages.push(ChatMessage {
            role: MessageRole::Assistant,
            content,
            tool_calls: vec![],
            timestamp: chrono::Local::now(),
        });
    }

    pub fn add_tool_call(&mut self, tool: String, args: String, status: ToolStatus) {
        if let Some(last) = self.messages.last_mut() {
            if last.role == MessageRole::Assistant {
                last.tool_calls.push(ToolCallDisplay {
                    tool,
                    args_preview: args,
                    status,
                    output_preview: None,
                    diff: None,
                    duration_ms: None,
                });
            }
        }
    }

    pub fn update_last_tool_status(&mut self, status: ToolStatus, output: Option<String>, duration: Option<u32>) {
        if let Some(last_msg) = self.messages.last_mut() {
            if let Some(last_tool) = last_msg.tool_calls.last_mut() {
                last_tool.status = status;
                last_tool.output_preview = output;
                last_tool.duration_ms = duration;
            }
        }
    }

    pub fn scroll_up(&mut self) {
        if self.scroll_offset > 0 {
            self.scroll_offset -= 1;
        }
    }

    pub fn scroll_down(&mut self) {
        self.scroll_offset += 1;
    }

    pub fn scroll_to_bottom(&mut self) {
        self.scroll_offset = self.messages.len().saturating_sub(1);
    }
}
