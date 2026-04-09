//! Tool system for BeCode
//!
//! Tools are the actions the agent can take:
//! - bash: Execute shell commands
//! - read_file: Read file contents
//! - write_file: Write file contents
//! - edit_file: Edit file with string replacement
//! - glob_search: Find files by pattern
//! - grep_search: Search file contents
//! - web_fetch: Fetch URL contents
//! - web_search: Search the web
//! - task_track: Track tasks in session

mod traits;
mod registry;
mod bash;
mod read_file;
mod write_file;
mod edit_file;
mod glob_search;
mod grep_search;
mod web_fetch;
mod web_search;
mod task_track;

pub use traits::{Tool, ToolCall, ToolContext, ToolError, ToolOutput, ToolSpec};
pub use registry::ToolRegistry;

// Re-export individual tools
pub use bash::BashTool;
pub use read_file::ReadFileTool;
pub use write_file::WriteFileTool;
pub use edit_file::EditFileTool;
pub use glob_search::GlobSearchTool;
pub use grep_search::GrepSearchTool;
pub use web_fetch::WebFetchTool;
pub use web_search::WebSearchTool;
pub use task_track::TaskTrackTool;
