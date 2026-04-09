//! Recovery strategies for agent errors
//!
//! Handles various failure scenarios with automatic recovery attempts.

use serde::{Deserialize, Serialize};

/// Types of failures that can occur
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum FailureType {
    /// Invalid response format from model
    InvalidFormat,
    /// Tool execution failed
    ToolFailed { tool: String, error: String },
    /// Permission denied
    PermissionDenied { reason: String },
    /// Network/API error
    NetworkError { error: String },
    /// Rate limited
    RateLimited { retry_after: Option<u32> },
    /// Context too long
    ContextTooLong,
    /// Model refused request
    Refused { reason: String },
}

/// Recovery strategy to apply
#[derive(Debug, Clone)]
pub enum RecoveryStrategy {
    /// Retry with same input
    Retry { max_attempts: u32 },
    /// Retry with modified prompt
    RetryWithHint { hint: String },
    /// Wait and retry (for rate limits)
    WaitAndRetry { delay_secs: u32 },
    /// Compact context and retry
    CompactAndRetry,
    /// Ask user for help
    AskUser { question: String },
    /// Give up with error message
    GiveUp { message: String },
}

impl RecoveryStrategy {
    /// Determine recovery strategy for a failure
    pub fn for_failure(failure: &FailureType, attempt: u32) -> Self {
        match failure {
            FailureType::InvalidFormat => {
                if attempt < 3 {
                    Self::RetryWithHint {
                        hint: "Please respond with valid JSON tool calls or a final answer."
                            .to_string(),
                    }
                } else {
                    Self::GiveUp {
                        message: "Model repeatedly returned invalid format".to_string(),
                    }
                }
            }
            FailureType::ToolFailed { tool, error } => {
                if attempt < 2 {
                    Self::RetryWithHint {
                        hint: format!(
                            "Tool '{}' failed with error: {}. Please try a different approach.",
                            tool, error
                        ),
                    }
                } else {
                    Self::AskUser {
                        question: format!(
                            "Tool '{}' keeps failing. Should I try a different approach?",
                            tool
                        ),
                    }
                }
            }
            FailureType::PermissionDenied { reason } => Self::AskUser {
                question: format!(
                    "Permission denied: {}. Would you like to grant permission?",
                    reason
                ),
            },
            FailureType::NetworkError { error } => {
                if attempt < 3 {
                    Self::WaitAndRetry { delay_secs: 5 }
                } else {
                    Self::GiveUp {
                        message: format!("Network error after {} attempts: {}", attempt, error),
                    }
                }
            }
            FailureType::RateLimited { retry_after } => Self::WaitAndRetry {
                delay_secs: retry_after.unwrap_or(60),
            },
            FailureType::ContextTooLong => Self::CompactAndRetry,
            FailureType::Refused { reason } => Self::AskUser {
                question: format!(
                    "Model refused the request: {}. Would you like to rephrase?",
                    reason
                ),
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_recovery_for_invalid_format() {
        let failure = FailureType::InvalidFormat;

        // First attempts should retry
        match RecoveryStrategy::for_failure(&failure, 1) {
            RecoveryStrategy::RetryWithHint { .. } => {}
            _ => panic!("Expected RetryWithHint"),
        }

        // After 3 attempts, give up
        match RecoveryStrategy::for_failure(&failure, 3) {
            RecoveryStrategy::GiveUp { .. } => {}
            _ => panic!("Expected GiveUp"),
        }
    }

    #[test]
    fn test_recovery_for_rate_limit() {
        let failure = FailureType::RateLimited {
            retry_after: Some(30),
        };

        match RecoveryStrategy::for_failure(&failure, 1) {
            RecoveryStrategy::WaitAndRetry { delay_secs } => {
                assert_eq!(delay_secs, 30);
            }
            _ => panic!("Expected WaitAndRetry"),
        }
    }

    #[test]
    fn test_recovery_for_context_too_long() {
        let failure = FailureType::ContextTooLong;

        match RecoveryStrategy::for_failure(&failure, 1) {
            RecoveryStrategy::CompactAndRetry => {}
            _ => panic!("Expected CompactAndRetry"),
        }
    }
}
