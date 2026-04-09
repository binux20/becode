//! bash tool - Execute shell commands
//!
//! Executes commands with safety checks and timeout support.

use super::traits::{Tool, ToolContext, ToolError, ToolOutput};
use crate::permissions::Permission;
use async_trait::async_trait;
use serde_json::{json, Value};
use std::process::Stdio;
use std::time::Duration;
use tokio::process::Command;
use tokio::time::timeout;

/// Default timeout in seconds
const DEFAULT_TIMEOUT_SECS: u64 = 120;

/// Maximum output size in bytes
const MAX_OUTPUT_SIZE: usize = 100_000;

/// Tool for executing bash/shell commands
pub struct BashTool;

#[async_trait]
impl Tool for BashTool {
    fn name(&self) -> &'static str {
        "bash"
    }

    fn description(&self) -> &'static str {
        "Execute a shell command. Use for running tests, builds, git operations, etc. \
         Commands are executed in the project directory."
    }

    fn input_schema(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "command": {
                    "type": "string",
                    "description": "The command to execute"
                },
                "timeout_secs": {
                    "type": "integer",
                    "description": "Timeout in seconds (default: 120)",
                    "default": 120
                },
                "cwd": {
                    "type": "string",
                    "description": "Working directory (relative to project root)"
                }
            },
            "required": ["command"]
        })
    }

    fn required_permission(&self) -> Permission {
        Permission::WorkspaceWrite
    }

    async fn execute(&self, input: Value, ctx: &ToolContext) -> Result<ToolOutput, ToolError> {
        let command = input["command"]
            .as_str()
            .ok_or_else(|| ToolError::InvalidInput {
                reason: "command is required".to_string(),
            })?;

        let timeout_secs = input["timeout_secs"]
            .as_u64()
            .unwrap_or(DEFAULT_TIMEOUT_SECS);

        // Determine working directory
        let cwd = if let Some(cwd_str) = input["cwd"].as_str() {
            ctx.resolve_path(cwd_str)?
        } else {
            ctx.workspace_root.clone()
        };

        // Check command against policy
        ctx.enforcer
            .check_bash_command(command)
            .map_err(|e| ToolError::PermissionDenied {
                reason: e.to_string(),
            })?;

        // Prepare command based on OS
        let (shell, shell_arg) = if cfg!(windows) {
            ("cmd", "/C")
        } else {
            ("sh", "-c")
        };

        // Execute command
        let child = Command::new(shell)
            .arg(shell_arg)
            .arg(command)
            .current_dir(&cwd)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .map_err(|e| ToolError::ExecutionFailed {
                reason: format!("Failed to spawn command: {}", e),
            })?;

        // Wait with timeout
        let result = timeout(Duration::from_secs(timeout_secs), child.wait_with_output()).await;

        match result {
            Ok(Ok(output)) => {
                let stdout = String::from_utf8_lossy(&output.stdout);
                let stderr = String::from_utf8_lossy(&output.stderr);
                let exit_code = output.status.code().unwrap_or(-1);

                // Truncate output if too large
                let stdout_truncated = truncate_output(&stdout, MAX_OUTPUT_SIZE);
                let stderr_truncated = truncate_output(&stderr, MAX_OUTPUT_SIZE);

                let result = json!({
                    "command": command,
                    "exit_code": exit_code,
                    "stdout": stdout_truncated,
                    "stderr": stderr_truncated,
                    "success": exit_code == 0,
                    "cwd": cwd.display().to_string()
                });

                Ok(ToolOutput::success(self.name(), result, 0))
            }
            Ok(Err(e)) => Err(ToolError::ExecutionFailed {
                reason: format!("Command failed: {}", e),
            }),
            Err(_) => {
                // Timeout - process already consumed by wait_with_output
                Err(ToolError::Timeout {
                    command: command.to_string(),
                    timeout_secs: timeout_secs as u32,
                })
            }
        }
    }
}

/// Truncate output to max size with indicator
fn truncate_output(output: &str, max_size: usize) -> String {
    if output.len() <= max_size {
        output.to_string()
    } else {
        format!(
            "{}\n... [truncated, {} bytes total]",
            &output[..max_size],
            output.len()
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env;

    #[tokio::test]
    async fn test_bash_echo() {
        let tool = BashTool;
        let ctx = create_test_context();

        let input = json!({
            "command": if cfg!(windows) { "echo hello" } else { "echo hello" }
        });

        let result = tool.execute(input, &ctx).await;
        assert!(result.is_ok());

        let output = result.unwrap();
        assert!(output.success);
        assert_eq!(output.result["exit_code"], 0);
        assert!(output.result["stdout"]
            .as_str()
            .unwrap()
            .contains("hello"));
    }

    #[tokio::test]
    async fn test_bash_exit_code() {
        let tool = BashTool;
        let ctx = create_test_context();

        let input = json!({
            "command": if cfg!(windows) { "exit 1" } else { "exit 1" }
        });

        let result = tool.execute(input, &ctx).await;
        assert!(result.is_ok());

        let output = result.unwrap();
        assert_eq!(output.result["exit_code"], 1);
    }

    fn create_test_context() -> ToolContext {
        use crate::permissions::PermissionEnforcer;
        use std::sync::Arc;

        let workspace = env::current_dir().unwrap();
        let enforcer = Arc::new(PermissionEnforcer::new(
            Permission::WorkspaceWrite,
            workspace.clone(),
        ));

        ToolContext {
            workspace_root: workspace,
            permission: Permission::WorkspaceWrite,
            enforcer,
        }
    }
}
