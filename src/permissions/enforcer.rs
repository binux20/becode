//! Permission enforcer for tool execution
//!
//! Checks permissions before allowing tool execution

use super::{BashPolicy, CommandDecision, Permission};
use std::path::{Path, PathBuf};
use thiserror::Error;

/// Errors from permission enforcement
#[derive(Debug, Error)]
pub enum PermissionError {
    #[error("Permission denied: {tool} requires {required}, current mode is {current}")]
    InsufficientPermission {
        tool: String,
        required: Permission,
        current: Permission,
    },

    #[error("Path escapes workspace: {path}")]
    PathEscapesWorkspace { path: String },

    #[error("Command blocked: {reason}")]
    CommandBlocked { reason: String },

    #[error("Command requires confirmation: {reason}")]
    CommandNeedsConfirmation { reason: String },
}

/// Permission enforcer context
pub struct PermissionEnforcer {
    /// Current permission level
    pub current_permission: Permission,

    /// Workspace root directory
    pub workspace_root: PathBuf,

    /// Callback for confirmation prompts
    confirm_callback: Option<Box<dyn Fn(&str, &str) -> bool + Send + Sync>>,
}

impl PermissionEnforcer {
    /// Create a new permission enforcer
    pub fn new(permission: Permission, workspace_root: PathBuf) -> Self {
        Self {
            current_permission: permission,
            workspace_root,
            confirm_callback: None,
        }
    }

    /// Set confirmation callback
    pub fn with_confirm_callback<F>(mut self, callback: F) -> Self
    where
        F: Fn(&str, &str) -> bool + Send + Sync + 'static,
    {
        self.confirm_callback = Some(Box::new(callback));
        self
    }

    /// Check if a tool can be executed
    pub fn check_tool(&self, tool_name: &str, required: Permission) -> Result<(), PermissionError> {
        if !self.current_permission.allows(required) {
            return Err(PermissionError::InsufficientPermission {
                tool: tool_name.to_string(),
                required,
                current: self.current_permission,
            });
        }
        Ok(())
    }

    /// Check if a path is within the workspace
    pub fn check_path(&self, path: &Path) -> Result<PathBuf, PermissionError> {
        // Allow absolute paths only in DangerFullAccess mode
        let resolved = if path.is_absolute() {
            if self.current_permission != Permission::DangerFullAccess {
                // Check if it's within workspace
                let canonical = path.canonicalize().unwrap_or_else(|_| path.to_path_buf());
                let workspace_canonical = self
                    .workspace_root
                    .canonicalize()
                    .unwrap_or_else(|_| self.workspace_root.clone());

                if !canonical.starts_with(&workspace_canonical) {
                    return Err(PermissionError::PathEscapesWorkspace {
                        path: path.display().to_string(),
                    });
                }
                canonical
            } else {
                path.to_path_buf()
            }
        } else {
            // Relative path - resolve from workspace
            let full_path = self.workspace_root.join(path);
            let canonical = full_path
                .canonicalize()
                .unwrap_or_else(|_| full_path.clone());
            let workspace_canonical = self
                .workspace_root
                .canonicalize()
                .unwrap_or_else(|_| self.workspace_root.clone());

            // Check for path traversal (..)
            if !canonical.starts_with(&workspace_canonical) {
                return Err(PermissionError::PathEscapesWorkspace {
                    path: path.display().to_string(),
                });
            }
            canonical
        };

        Ok(resolved)
    }

    /// Check if a bash command can be executed
    pub fn check_bash_command(&self, command: &str) -> Result<(), PermissionError> {
        // In DangerFullAccess mode, allow everything
        if self.current_permission == Permission::DangerFullAccess {
            return Ok(());
        }

        match BashPolicy::classify(command) {
            CommandDecision::Safe => Ok(()),
            CommandDecision::Blocked { reason } => {
                Err(PermissionError::CommandBlocked { reason })
            }
            CommandDecision::Confirm { reason } => {
                // If we have a confirmation callback, use it
                if let Some(ref callback) = self.confirm_callback {
                    if callback(command, &reason) {
                        return Ok(());
                    }
                }
                Err(PermissionError::CommandNeedsConfirmation { reason })
            }
        }
    }

    /// Resolve a path relative to workspace
    pub fn resolve_path(&self, path: &str) -> Result<PathBuf, PermissionError> {
        self.check_path(Path::new(path))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env;

    fn test_enforcer() -> PermissionEnforcer {
        PermissionEnforcer::new(Permission::WorkspaceWrite, env::current_dir().unwrap())
    }

    #[test]
    fn test_check_tool_permission() {
        let enforcer = test_enforcer();

        // Should allow ReadOnly tools
        assert!(enforcer.check_tool("read_file", Permission::ReadOnly).is_ok());

        // Should allow WorkspaceWrite tools
        assert!(enforcer
            .check_tool("write_file", Permission::WorkspaceWrite)
            .is_ok());

        // Should deny DangerFullAccess tools
        assert!(enforcer
            .check_tool("dangerous_tool", Permission::DangerFullAccess)
            .is_err());
    }

    #[test]
    fn test_check_path_relative() {
        let enforcer = test_enforcer();

        // Relative paths within workspace should work
        assert!(enforcer.check_path(Path::new("src/main.rs")).is_ok());

        // Path traversal should be blocked
        assert!(enforcer.check_path(Path::new("../../etc/passwd")).is_err());
    }

    #[test]
    fn test_check_bash_safe() {
        let enforcer = test_enforcer();

        assert!(enforcer.check_bash_command("git status").is_ok());
        assert!(enforcer.check_bash_command("cargo test").is_ok());
    }

    #[test]
    fn test_check_bash_blocked() {
        let enforcer = test_enforcer();

        assert!(enforcer.check_bash_command("rm -rf /").is_err());
    }

    #[test]
    fn test_danger_mode_allows_all() {
        let enforcer =
            PermissionEnforcer::new(Permission::DangerFullAccess, env::current_dir().unwrap());

        // Even dangerous commands should be allowed
        assert!(enforcer.check_bash_command("rm -rf /tmp/test").is_ok());
    }
}
