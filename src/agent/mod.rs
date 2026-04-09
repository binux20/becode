//! Agent runtime for BeCode
//!
//! The agent orchestrates the conversation loop between user, LLM, and tools.

mod runtime;
mod parser;
mod recovery;

pub use runtime::AgentRuntime;
pub use parser::{ParsedResponse, ResponseParser};
pub use recovery::RecoveryStrategy;
