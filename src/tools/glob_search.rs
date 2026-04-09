//! glob_search tool - Find files by pattern

use super::traits::{Tool, ToolContext, ToolError, ToolOutput};
use crate::permissions::Permission;
use async_trait::async_trait;
use ignore::WalkBuilder;
use serde_json::{json, Value};

/// Default limit for results
const DEFAULT_LIMIT: usize = 100;

/// Directories to skip by default
const SKIP_DIRS: &[&str] = &[
    ".git",
    "node_modules",
    "__pycache__",
    ".venv",
    "venv",
    "target",
    "dist",
    "build",
    ".next",
    ".nuxt",
    "coverage",
    ".pytest_cache",
    ".mypy_cache",
    ".ruff_cache",
];

/// Tool for finding files by glob pattern
pub struct GlobSearchTool;

#[async_trait]
impl Tool for GlobSearchTool {
    fn name(&self) -> &'static str {
        "glob_search"
    }

    fn description(&self) -> &'static str {
        "Find files matching a glob pattern. Examples: '*.rs', 'src/**/*.py', '**/test_*.js'"
    }

    fn input_schema(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "pattern": {
                    "type": "string",
                    "description": "Glob pattern to match (e.g., '*.rs', 'src/**/*.py')"
                },
                "path": {
                    "type": "string",
                    "description": "Directory to search in (default: project root)",
                    "default": "."
                },
                "limit": {
                    "type": "integer",
                    "description": "Maximum number of results (default: 100)",
                    "default": 100
                },
                "include_hidden": {
                    "type": "boolean",
                    "description": "Include hidden files/directories",
                    "default": false
                }
            },
            "required": ["pattern"]
        })
    }

    fn required_permission(&self) -> Permission {
        Permission::ReadOnly
    }

    async fn execute(&self, input: Value, ctx: &ToolContext) -> Result<ToolOutput, ToolError> {
        let pattern = input["pattern"]
            .as_str()
            .ok_or_else(|| ToolError::InvalidInput {
                reason: "pattern is required".to_string(),
            })?;

        let path = input["path"].as_str().unwrap_or(".");
        let limit = input["limit"].as_u64().unwrap_or(DEFAULT_LIMIT as u64) as usize;
        let include_hidden = input["include_hidden"].as_bool().unwrap_or(false);

        // Resolve search path
        let search_path = ctx.resolve_path(path)?;

        if !search_path.exists() {
            return Err(ToolError::ExecutionFailed {
                reason: format!("Path not found: {}", path),
            });
        }

        // Compile glob pattern
        let glob = glob::Pattern::new(pattern).map_err(|e| ToolError::InvalidInput {
            reason: format!("Invalid glob pattern: {}", e),
        })?;

        // Walk directory
        let mut matches: Vec<String> = Vec::new();
        let walker = WalkBuilder::new(&search_path)
            .hidden(!include_hidden)
            .git_ignore(true)
            .git_global(true)
            .git_exclude(true)
            .build();

        for entry in walker {
            if matches.len() >= limit {
                break;
            }

            let entry = match entry {
                Ok(e) => e,
                Err(_) => continue,
            };

            let path = entry.path();

            // Skip directories in skip list
            if path.is_dir() {
                if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
                    if SKIP_DIRS.contains(&name) {
                        continue;
                    }
                }
            }

            // Only match files
            if !path.is_file() {
                continue;
            }

            // Get relative path for matching
            let rel_path = path
                .strip_prefix(&search_path)
                .unwrap_or(path)
                .to_string_lossy();

            // Match against pattern
            let file_name = path.file_name().and_then(|n| n.to_str()).unwrap_or("");

            // Try matching both full relative path and just filename
            if glob.matches(&rel_path) || glob.matches(file_name) {
                matches.push(rel_path.to_string().replace('\\', "/"));
            }
        }

        let total_found = matches.len();
        let is_limited = total_found >= limit;

        let result = json!({
            "pattern": pattern,
            "path": path,
            "matches": matches,
            "total_found": total_found,
            "is_limited": is_limited,
            "limit": limit
        });

        Ok(ToolOutput::success(self.name(), result, 0))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    #[tokio::test]
    async fn test_glob_search_basic() {
        let temp_dir = TempDir::new().unwrap();

        // Create test files
        fs::write(temp_dir.path().join("test1.rs"), "content").unwrap();
        fs::write(temp_dir.path().join("test2.rs"), "content").unwrap();
        fs::write(temp_dir.path().join("test.py"), "content").unwrap();

        let tool = GlobSearchTool;
        let ctx = create_test_context(temp_dir.path());

        let input = json!({
            "pattern": "*.rs"
        });

        let result = tool.execute(input, &ctx).await;
        assert!(result.is_ok());

        let output = result.unwrap();
        assert!(output.success);
        assert_eq!(output.result["total_found"], 2);
    }

    #[tokio::test]
    async fn test_glob_search_nested() {
        let temp_dir = TempDir::new().unwrap();

        // Create nested structure
        fs::create_dir_all(temp_dir.path().join("src/utils")).unwrap();
        fs::write(temp_dir.path().join("src/main.rs"), "content").unwrap();
        fs::write(temp_dir.path().join("src/utils/helpers.rs"), "content").unwrap();

        let tool = GlobSearchTool;
        let ctx = create_test_context(temp_dir.path());

        let input = json!({
            "pattern": "**/*.rs"
        });

        let result = tool.execute(input, &ctx).await;
        assert!(result.is_ok());

        let output = result.unwrap();
        assert!(output.success);
        assert_eq!(output.result["total_found"], 2);
    }

    fn create_test_context(workspace: &std::path::Path) -> ToolContext {
        use crate::permissions::PermissionEnforcer;
        use std::sync::Arc;

        let enforcer = Arc::new(PermissionEnforcer::new(
            Permission::WorkspaceWrite,
            workspace.to_path_buf(),
        ));

        ToolContext {
            workspace_root: workspace.to_path_buf(),
            permission: Permission::WorkspaceWrite,
            enforcer,
        }
    }
}
