//! Permission system for BeCode
//!
//! Three-level permission model:
//! - ReadOnly: Only read operations (file read, search, web fetch)
//! - WorkspaceWrite: Read + write within project directory
//! - DangerFullAccess: Full system access (requires explicit flag)

mod enforcer;
mod policy;

pub use enforcer::PermissionEnforcer;
pub use policy::{BashPolicy, CommandDecision};

use serde::{Deserialize, Serialize};
use std::fmt;

/// Permission levels for tool execution
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum Permission {
    /// Only read operations allowed
    ReadOnly = 0,

    /// Read + write within workspace (DEFAULT)
    WorkspaceWrite = 1,

    /// Full system access (dangerous)
    DangerFullAccess = 2,
}

impl Default for Permission {
    fn default() -> Self {
        Self::WorkspaceWrite
    }
}

impl fmt::Display for Permission {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::ReadOnly => write!(f, "read-only"),
            Self::WorkspaceWrite => write!(f, "workspace-write"),
            Self::DangerFullAccess => write!(f, "danger-full-access"),
        }
    }
}

impl Permission {
    /// Parse permission from string
    pub fn from_str(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "read-only" | "readonly" | "read" => Some(Self::ReadOnly),
            "workspace-write" | "workspacewrite" | "write" | "workspace" => {
                Some(Self::WorkspaceWrite)
            }
            "danger-full-access" | "dangerfullaccess" | "danger" | "full" => {
                Some(Self::DangerFullAccess)
            }
            _ => None,
        }
    }

    /// Check if this permission level allows another
    pub fn allows(&self, required: Permission) -> bool {
        *self >= required
    }

    /// Get description for this permission level
    pub fn description(&self) -> &'static str {
        match self {
            Self::ReadOnly => "Read-only access: file reading, searching, web fetching",
            Self::WorkspaceWrite => {
                "Workspace write: read + write/edit files, run safe commands"
            }
            Self::DangerFullAccess => {
                "Full access: all operations including system commands (dangerous!)"
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_permission_ordering() {
        assert!(Permission::ReadOnly < Permission::WorkspaceWrite);
        assert!(Permission::WorkspaceWrite < Permission::DangerFullAccess);
    }

    #[test]
    fn test_permission_allows() {
        assert!(Permission::DangerFullAccess.allows(Permission::ReadOnly));
        assert!(Permission::DangerFullAccess.allows(Permission::WorkspaceWrite));
        assert!(Permission::WorkspaceWrite.allows(Permission::ReadOnly));
        assert!(!Permission::ReadOnly.allows(Permission::WorkspaceWrite));
    }

    #[test]
    fn test_permission_from_str() {
        assert_eq!(Permission::from_str("read-only"), Some(Permission::ReadOnly));
        assert_eq!(
            Permission::from_str("workspace-write"),
            Some(Permission::WorkspaceWrite)
        );
        assert_eq!(
            Permission::from_str("danger"),
            Some(Permission::DangerFullAccess)
        );
    }
}
