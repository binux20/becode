//! Session management for BeCode
//!
//! Sessions persist conversation history and state using JSONL format.

mod storage;
mod compaction;

pub use storage::{Session, SessionEvent, SessionStore};
pub use compaction::compact_session;

use serde::{Deserialize, Serialize};
use crate::providers::Message;
use crate::tools::ToolOutput;

/// A turn in the conversation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Turn {
    pub user_message: String,
    pub assistant_response: String,
    pub tool_calls: Vec<ToolCallRecord>,
    pub timestamp: String,
}

/// Record of a tool call
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolCallRecord {
    pub tool: String,
    pub input: serde_json::Value,
    pub output: ToolOutput,
    pub duration_ms: u32,
}
