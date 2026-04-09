//! Session storage using JSONL format

use super::{Turn, ToolCallRecord};
use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::fs::{self, File, OpenOptions};
use std::io::{BufRead, BufReader, Write};
use std::path::PathBuf;
use uuid::Uuid;

use crate::config::Config;

/// Session event types for JSONL storage
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum SessionEvent {
    #[serde(rename = "session_start")]
    SessionStart {
        id: String,
        timestamp: String,
        project: String,
        provider: String,
        model: Option<String>,
    },

    #[serde(rename = "user_message")]
    UserMessage { content: String, timestamp: String },

    #[serde(rename = "assistant_message")]
    AssistantMessage { content: String, timestamp: String },

    #[serde(rename = "tool_call")]
    ToolCall {
        tool: String,
        input: serde_json::Value,
        output: serde_json::Value,
        duration_ms: u32,
        timestamp: String,
    },

    #[serde(rename = "compaction")]
    Compaction {
        removed_count: usize,
        summary: String,
        timestamp: String,
    },

    #[serde(rename = "session_end")]
    SessionEnd {
        timestamp: String,
        final_message: Option<String>,
    },
}

/// A session containing conversation history
#[derive(Debug, Clone)]
pub struct Session {
    pub id: String,
    pub project: PathBuf,
    pub provider: String,
    pub model: Option<String>,
    pub turns: Vec<Turn>,
    pub created_at: String,
    pub updated_at: String,
    file_path: Option<PathBuf>,
}

impl Session {
    /// Create a new session
    pub fn new(project: PathBuf, provider: String, model: Option<String>) -> Self {
        let now = chrono::Utc::now().to_rfc3339();
        Self {
            id: Uuid::new_v4().to_string(),
            project,
            provider,
            model,
            turns: Vec::new(),
            created_at: now.clone(),
            updated_at: now,
            file_path: None,
        }
    }

    /// Get session file path
    pub fn file_path(&self) -> PathBuf {
        self.file_path.clone().unwrap_or_else(|| {
            Config::sessions_dir().join(format!("session-{}.jsonl", self.id))
        })
    }

    /// Append an event to the session file
    pub fn append_event(&mut self, event: SessionEvent) -> Result<()> {
        let path = self.file_path();

        // Ensure directory exists
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }

        // Append to file
        let mut file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&path)
            .with_context(|| format!("Failed to open session file: {:?}", path))?;

        let line = serde_json::to_string(&event)?;
        writeln!(file, "{}", line)?;

        self.updated_at = chrono::Utc::now().to_rfc3339();
        self.file_path = Some(path);

        Ok(())
    }

    /// Add a user message
    pub fn add_user_message(&mut self, content: &str) -> Result<()> {
        self.append_event(SessionEvent::UserMessage {
            content: content.to_string(),
            timestamp: chrono::Utc::now().to_rfc3339(),
        })
    }

    /// Add an assistant message
    pub fn add_assistant_message(&mut self, content: &str) -> Result<()> {
        self.append_event(SessionEvent::AssistantMessage {
            content: content.to_string(),
            timestamp: chrono::Utc::now().to_rfc3339(),
        })
    }

    /// Add a tool call record
    pub fn add_tool_call(
        &mut self,
        tool: &str,
        input: serde_json::Value,
        output: serde_json::Value,
        duration_ms: u32,
    ) -> Result<()> {
        self.append_event(SessionEvent::ToolCall {
            tool: tool.to_string(),
            input,
            output,
            duration_ms,
            timestamp: chrono::Utc::now().to_rfc3339(),
        })
    }

    /// Start the session (write initial event)
    pub fn start(&mut self) -> Result<()> {
        self.append_event(SessionEvent::SessionStart {
            id: self.id.clone(),
            timestamp: self.created_at.clone(),
            project: self.project.display().to_string(),
            provider: self.provider.clone(),
            model: self.model.clone(),
        })
    }

    /// End the session
    pub fn end(&mut self, final_message: Option<String>) -> Result<()> {
        self.append_event(SessionEvent::SessionEnd {
            timestamp: chrono::Utc::now().to_rfc3339(),
            final_message,
        })
    }
}

/// Session store for managing multiple sessions
pub struct SessionStore;

impl SessionStore {
    /// List all session IDs
    pub fn list() -> Result<Vec<String>> {
        let sessions_dir = Config::sessions_dir();
        if !sessions_dir.exists() {
            return Ok(Vec::new());
        }

        let mut sessions = Vec::new();
        for entry in fs::read_dir(sessions_dir)? {
            let entry = entry?;
            let path = entry.path();
            if path.extension().map(|e| e == "jsonl").unwrap_or(false) {
                if let Some(name) = path.file_stem() {
                    let name = name.to_string_lossy();
                    if let Some(id) = name.strip_prefix("session-") {
                        sessions.push(id.to_string());
                    }
                }
            }
        }

        // Sort by modification time (newest first)
        sessions.sort_by(|a, b| b.cmp(a));
        Ok(sessions)
    }

    /// Load a session by ID
    pub fn load(id: &str) -> Result<Session> {
        let path = Config::sessions_dir().join(format!("session-{}.jsonl", id));
        Self::load_from_path(&path)
    }

    /// Load a session from a file path
    pub fn load_from_path(path: &PathBuf) -> Result<Session> {
        let file = File::open(path)
            .with_context(|| format!("Failed to open session file: {:?}", path))?;

        let reader = BufReader::new(file);
        let mut session: Option<Session> = None;
        let mut current_turn: Option<Turn> = None;
        let mut tool_calls: Vec<ToolCallRecord> = Vec::new();

        for line in reader.lines() {
            let line = line?;
            if line.trim().is_empty() {
                continue;
            }

            let event: SessionEvent = serde_json::from_str(&line)
                .with_context(|| format!("Failed to parse session event: {}", line))?;

            match event {
                SessionEvent::SessionStart {
                    id,
                    timestamp,
                    project,
                    provider,
                    model,
                } => {
                    session = Some(Session {
                        id,
                        project: PathBuf::from(project),
                        provider,
                        model,
                        turns: Vec::new(),
                        created_at: timestamp.clone(),
                        updated_at: timestamp,
                        file_path: Some(path.clone()),
                    });
                }
                SessionEvent::UserMessage { content, timestamp } => {
                    // Save previous turn if exists
                    if let Some(turn) = current_turn.take() {
                        if let Some(ref mut s) = session {
                            s.turns.push(turn);
                        }
                    }
                    // Start new turn
                    current_turn = Some(Turn {
                        user_message: content,
                        assistant_response: String::new(),
                        tool_calls: Vec::new(),
                        timestamp,
                    });
                    tool_calls.clear();
                }
                SessionEvent::AssistantMessage { content, timestamp } => {
                    if let Some(ref mut turn) = current_turn {
                        turn.assistant_response = content;
                        turn.tool_calls = std::mem::take(&mut tool_calls);
                    }
                    if let Some(ref mut s) = session {
                        s.updated_at = timestamp;
                    }
                }
                SessionEvent::ToolCall {
                    tool,
                    input,
                    output,
                    duration_ms,
                    ..
                } => {
                    tool_calls.push(ToolCallRecord {
                        tool,
                        input,
                        output: crate::tools::ToolOutput {
                            tool: String::new(),
                            success: true,
                            result: output,
                            duration_ms,
                            patch: None,
                            call_id: None,
                        },
                        duration_ms,
                    });
                }
                SessionEvent::Compaction { .. } => {
                    // Compaction events don't affect loaded state
                }
                SessionEvent::SessionEnd { timestamp, .. } => {
                    if let Some(ref mut s) = session {
                        s.updated_at = timestamp;
                    }
                }
            }
        }

        // Save last turn
        if let Some(turn) = current_turn {
            if let Some(ref mut s) = session {
                s.turns.push(turn);
            }
        }

        session.ok_or_else(|| anyhow::anyhow!("No session_start event found in file"))
    }

    /// Get the most recent session
    pub fn latest() -> Result<Option<Session>> {
        let sessions = Self::list()?;
        if let Some(id) = sessions.first() {
            Ok(Some(Self::load(id)?))
        } else {
            Ok(None)
        }
    }

    /// Delete a session
    pub fn delete(id: &str) -> Result<()> {
        let path = Config::sessions_dir().join(format!("session-{}.jsonl", id));
        if path.exists() {
            fs::remove_file(path)?;
        }
        Ok(())
    }
}
