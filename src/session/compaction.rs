//! Session compaction for managing context size
//!
//! When conversations get too long, older messages are summarized
//! to keep within token limits.

use super::{Session, SessionEvent};
use anyhow::Result;

/// Compact a session by summarizing old messages
/// Keeps first N and last M messages, summarizes the rest
pub fn compact_session(
    session: &mut Session,
    keep_start: usize,
    keep_end: usize,
    summary: &str,
) -> Result<()> {
    let total_turns = session.turns.len();

    if total_turns <= keep_start + keep_end {
        // Nothing to compact
        return Ok(());
    }

    let removed_count = total_turns - keep_start - keep_end;

    // Record compaction event
    session.append_event(SessionEvent::Compaction {
        removed_count,
        summary: summary.to_string(),
        timestamp: chrono::Utc::now().to_rfc3339(),
    })?;

    // Keep only the turns we want
    let mut new_turns = Vec::new();
    new_turns.extend(session.turns.iter().take(keep_start).cloned());
    new_turns.extend(
        session
            .turns
            .iter()
            .skip(total_turns - keep_end)
            .cloned(),
    );

    session.turns = new_turns;

    Ok(())
}

/// Estimate token count for a session
/// Uses rough approximation of 4 chars per token
pub fn estimate_tokens(session: &Session) -> usize {
    let mut char_count = 0;

    for turn in &session.turns {
        char_count += turn.user_message.len();
        char_count += turn.assistant_response.len();
        for tc in &turn.tool_calls {
            char_count += tc.input.to_string().len();
            char_count += tc.output.result.to_string().len();
        }
    }

    // Rough estimate: 4 characters per token
    char_count / 4
}

/// Check if session needs compaction
pub fn needs_compaction(session: &Session, max_tokens: usize) -> bool {
    estimate_tokens(session) > max_tokens
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::session::Turn;
    use std::path::PathBuf;

    #[test]
    fn test_estimate_tokens() {
        let session = Session {
            id: "test".to_string(),
            project: PathBuf::from("/test"),
            provider: "test".to_string(),
            model: None,
            turns: vec![Turn {
                user_message: "Hello world".to_string(), // 11 chars
                assistant_response: "Hi there!".to_string(), // 9 chars
                tool_calls: vec![],
                timestamp: "2024-01-01".to_string(),
            }],
            created_at: "2024-01-01".to_string(),
            updated_at: "2024-01-01".to_string(),
            file_path: None,
        };

        let tokens = estimate_tokens(&session);
        assert_eq!(tokens, 5); // (11 + 9) / 4 = 5
    }

    #[test]
    fn test_needs_compaction() {
        let session = Session {
            id: "test".to_string(),
            project: PathBuf::from("/test"),
            provider: "test".to_string(),
            model: None,
            turns: vec![Turn {
                user_message: "a".repeat(4000), // ~1000 tokens
                assistant_response: "b".repeat(4000),
                tool_calls: vec![],
                timestamp: "2024-01-01".to_string(),
            }],
            created_at: "2024-01-01".to_string(),
            updated_at: "2024-01-01".to_string(),
            file_path: None,
        };

        assert!(needs_compaction(&session, 1000));
        assert!(!needs_compaction(&session, 10000));
    }
}
