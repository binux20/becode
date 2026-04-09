//! Tool traits and types
//!
//! Defines the core Tool trait that all tools implement

use crate::permissions::Permission;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::sync::Arc;
use thiserror::Error;

/// Tool specification for LLM
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolSpec {
    /// Tool name (unique identifier)
    pub name: String,

    /// Human-readable description
    pub description: String,

    /// JSON Schema for input parameters
    pub input_schema: Value,

    /// Required permission level
    pub required_permission: Permission,
}

/// A tool call from the LLM
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolCall {
    /// Tool name
    pub tool: String,

    /// Arguments as JSON
    pub args: Value,

    /// Optional call ID (for tracking)
    #[serde(default)]
    pub id: Option<String>,
}

impl ToolCall {
    /// Create a new tool call
    pub fn new(tool: impl Into<String>, args: Value) -> Self {
        Self {
            tool: tool.into(),
            args,
            id: None,
        }
    }

    /// Create with ID
    pub fn with_id(mut self, id: impl Into<String>) -> Self {
        self.id = Some(id.into());
        self
    }
}

/// Output from a tool execution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolOutput {
    /// Tool name
    pub tool: String,

    /// Whether execution succeeded
    pub success: bool,

    /// Result data
    pub result: Value,

    /// Execution duration in milliseconds
    pub duration_ms: u32,

    /// File patch (for write/edit operations)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub patch: Option<FilePatch>,

    /// Call ID (if provided)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub call_id: Option<String>,
}

impl ToolOutput {
    /// Create a success output
    pub fn success(tool: impl Into<String>, result: Value, duration_ms: u32) -> Self {
        Self {
            tool: tool.into(),
            success: true,
            result,
            duration_ms,
            patch: None,
            call_id: None,
        }
    }

    /// Create a failure output
    pub fn failure(tool: impl Into<String>, error: impl Into<String>, duration_ms: u32) -> Self {
        Self {
            tool: tool.into(),
            success: false,
            result: serde_json::json!({ "error": error.into() }),
            duration_ms,
            patch: None,
            call_id: None,
        }
    }

    /// Add patch information
    pub fn with_patch(mut self, patch: FilePatch) -> Self {
        self.patch = Some(patch);
        self
    }

    /// Add call ID
    pub fn with_call_id(mut self, id: impl Into<String>) -> Self {
        self.call_id = Some(id.into());
        self
    }
}

/// File patch information for diff display
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FilePatch {
    /// File path
    pub path: String,

    /// Unified diff
    pub diff: String,

    /// Lines added
    pub lines_added: u32,

    /// Lines removed
    pub lines_removed: u32,
}

/// Errors from tool execution
#[derive(Debug, Error)]
pub enum ToolError {
    #[error("Permission denied: {reason}")]
    PermissionDenied { reason: String },

    #[error("Invalid input: {reason}")]
    InvalidInput { reason: String },

    #[error("Execution failed: {reason}")]
    ExecutionFailed { reason: String },

    #[error("Timeout after {timeout_secs}s: {command}")]
    Timeout { command: String, timeout_secs: u32 },

    #[error("Tool not found: {tool}")]
    NotFound { tool: String },

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),

    #[error("HTTP error: {0}")]
    Http(String),

    #[error("Path error: {reason}")]
    PathError { reason: String },
}

/// Context for tool execution
pub struct ToolContext {
    /// Workspace root directory
    pub workspace_root: std::path::PathBuf,

    /// Current permission level
    pub permission: Permission,

    /// Permission enforcer
    pub enforcer: Arc<crate::permissions::PermissionEnforcer>,
}

impl ToolContext {
    /// Resolve a path relative to workspace
    pub fn resolve_path(&self, path: &str) -> Result<std::path::PathBuf, ToolError> {
        self.enforcer
            .resolve_path(path)
            .map_err(|e| ToolError::PathError {
                reason: e.to_string(),
            })
    }
}

/// The core Tool trait
#[async_trait]
pub trait Tool: Send + Sync {
    /// Get tool name
    fn name(&self) -> &'static str;

    /// Get tool description
    fn description(&self) -> &'static str;

    /// Get JSON schema for input parameters
    fn input_schema(&self) -> Value;

    /// Get required permission level
    fn required_permission(&self) -> Permission;

    /// Execute the tool
    async fn execute(&self, input: Value, ctx: &ToolContext) -> Result<ToolOutput, ToolError>;

    /// Get tool specification for LLM
    fn spec(&self) -> ToolSpec {
        ToolSpec {
            name: self.name().to_string(),
            description: self.description().to_string(),
            input_schema: self.input_schema(),
            required_permission: self.required_permission(),
        }
    }
}
