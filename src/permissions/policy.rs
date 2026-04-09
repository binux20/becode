//! Bash command policy for safety checks
//!
//! Classifies commands as safe, confirm-required, or blocked

/// Decision for a command execution
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CommandDecision {
    /// Command is safe to execute
    Safe,

    /// Command requires user confirmation
    Confirm { reason: String },

    /// Command is blocked entirely
    Blocked { reason: String },
}

/// Policy for bash command execution
pub struct BashPolicy;

impl BashPolicy {
    /// Patterns that are always blocked (dangerous system commands)
    const BLOCKED_PATTERNS: &'static [&'static str] = &[
        "rm -rf /",
        "rm -rf ~",
        "rm -rf $HOME",
        "rm -rf %USERPROFILE%",
        "dd if=/dev/",
        "mkfs",
        ": () { :",       // Fork bomb
        ":(){ :",         // Fork bomb variant
        "shutdown",
        "reboot",
        "halt",
        "poweroff",
        "init 0",
        "init 6",
        "format c:",
        "format d:",
        "del /f /s /q c:",
        "rd /s /q c:",
        "> /dev/sda",
        "| /dev/sda",
        "curl | bash",
        "curl | sh",
        "wget | bash",
        "wget | sh",
    ];

    /// Patterns that require user confirmation
    const CONFIRM_PATTERNS: &'static [&'static str] = &[
        // Destructive file operations
        "rm ",
        "rm -r",
        "rmdir",
        "del ",
        "erase ",
        "rd ",
        // Git dangerous operations
        "git push",
        "git push -f",
        "git push --force",
        "git reset --hard",
        "git clean",
        "git checkout -- .",
        "git rebase",
        "git merge",
        // Package operations
        "pip install",
        "pip uninstall",
        "npm install -g",
        "npm publish",
        "cargo publish",
        // Permission changes
        "chmod ",
        "chown ",
        "icacls ",
        // Database operations
        "drop database",
        "drop table",
        "truncate table",
        "delete from",
        // Service operations
        "systemctl stop",
        "systemctl restart",
        "service stop",
        "net stop",
        // Output redirection (can overwrite files)
        " > ",
        " >> ",
    ];

    /// Patterns that are always safe
    const SAFE_PATTERNS: &'static [&'static str] = &[
        // File viewing
        "cat ",
        "type ",
        "head ",
        "tail ",
        "less ",
        "more ",
        // Directory listing
        "ls",
        "dir",
        "tree",
        "find ",
        "fd ",
        // Search
        "grep ",
        "rg ",
        "ag ",
        "findstr ",
        // Git read operations
        "git status",
        "git diff",
        "git log",
        "git show",
        "git branch",
        "git remote",
        "git rev-parse",
        "git ls-files",
        // Build/test commands
        "cargo check",
        "cargo build",
        "cargo test",
        "cargo clippy",
        "cargo fmt --check",
        "npm test",
        "npm run test",
        "npm run lint",
        "npm run build",
        "pnpm test",
        "pnpm run",
        "yarn test",
        "yarn run",
        "pytest",
        "python -m pytest",
        "py -m pytest",
        "go test",
        "go build",
        "make test",
        "make check",
        // Linters
        "ruff check",
        "ruff format --check",
        "mypy ",
        "flake8 ",
        "eslint ",
        "prettier --check",
        "rustfmt --check",
        // Info commands
        "echo ",
        "pwd",
        "whoami",
        "hostname",
        "date",
        "which ",
        "where ",
        "env",
        "printenv",
        // Python/Node execution (typically safe in dev context)
        "python ",
        "python3 ",
        "py ",
        "node ",
    ];

    /// Classify a command
    pub fn classify(command: &str) -> CommandDecision {
        let cmd = command.trim();
        let cmd_lower = cmd.to_lowercase();

        if cmd.is_empty() {
            return CommandDecision::Blocked {
                reason: "Empty command".to_string(),
            };
        }

        // Check blocked patterns first
        for pattern in Self::BLOCKED_PATTERNS {
            if cmd_lower.contains(&pattern.to_lowercase()) {
                return CommandDecision::Blocked {
                    reason: format!("Dangerous pattern detected: {}", pattern),
                };
            }
        }

        // Check safe patterns
        for pattern in Self::SAFE_PATTERNS {
            if cmd_lower.starts_with(&pattern.to_lowercase()) {
                return CommandDecision::Safe;
            }
        }

        // Check confirm patterns
        for pattern in Self::CONFIRM_PATTERNS {
            if cmd_lower.contains(&pattern.to_lowercase()) {
                return CommandDecision::Confirm {
                    reason: format!("Potentially risky operation: {}", pattern.trim()),
                };
            }
        }

        // Default: require confirmation for unknown commands
        CommandDecision::Confirm {
            reason: "Unknown command - please review".to_string(),
        }
    }

    /// Check if a command modifies files
    pub fn is_write_operation(command: &str) -> bool {
        let cmd_lower = command.to_lowercase();
        let write_indicators = [
            ">",
            ">>",
            "rm ",
            "del ",
            "mv ",
            "move ",
            "cp ",
            "copy ",
            "mkdir ",
            "touch ",
            "echo ",
            "tee ",
            "sed -i",
            "awk -i",
        ];

        write_indicators
            .iter()
            .any(|ind| cmd_lower.contains(ind))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_blocked_commands() {
        assert!(matches!(
            BashPolicy::classify("rm -rf /"),
            CommandDecision::Blocked { .. }
        ));
        assert!(matches!(
            BashPolicy::classify(":(){ :|:& };:"),
            CommandDecision::Blocked { .. }
        ));
    }

    #[test]
    fn test_safe_commands() {
        assert_eq!(BashPolicy::classify("git status"), CommandDecision::Safe);
        assert_eq!(BashPolicy::classify("cargo test"), CommandDecision::Safe);
        assert_eq!(BashPolicy::classify("ls -la"), CommandDecision::Safe);
        assert_eq!(BashPolicy::classify("cat file.txt"), CommandDecision::Safe);
    }

    #[test]
    fn test_confirm_commands() {
        assert!(matches!(
            BashPolicy::classify("rm file.txt"),
            CommandDecision::Confirm { .. }
        ));
        assert!(matches!(
            BashPolicy::classify("git push origin main"),
            CommandDecision::Confirm { .. }
        ));
    }

    #[test]
    fn test_empty_command() {
        assert!(matches!(
            BashPolicy::classify(""),
            CommandDecision::Blocked { .. }
        ));
    }
}
