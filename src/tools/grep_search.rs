//! grep_search tool - Search file contents

use super::traits::{Tool, ToolContext, ToolError, ToolOutput};
use crate::permissions::Permission;
use async_trait::async_trait;
use ignore::WalkBuilder;
use regex::Regex;
use serde_json::{json, Value};
use std::fs;

/// Default context lines
const DEFAULT_CONTEXT_LINES: usize = 2;

/// Default max results
const DEFAULT_MAX_RESULTS: usize = 50;

/// Max file size to search (10MB)
const MAX_FILE_SIZE: u64 = 10 * 1024 * 1024;

/// Tool for searching file contents
pub struct GrepSearchTool;

#[derive(Debug, Clone)]
struct SearchMatch {
    file: String,
    line: usize,
    content: String,
    before: Vec<String>,
    after: Vec<String>,
}

#[async_trait]
impl Tool for GrepSearchTool {
    fn name(&self) -> &'static str {
        "grep_search"
    }

    fn description(&self) -> &'static str {
        "Search for a pattern in file contents. Supports regex. \
         Returns matching lines with context."
    }

    fn input_schema(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "pattern": {
                    "type": "string",
                    "description": "Search pattern (regex supported)"
                },
                "path": {
                    "type": "string",
                    "description": "Directory to search in (default: project root)",
                    "default": "."
                },
                "glob": {
                    "type": "string",
                    "description": "File pattern to search (e.g., '*.rs', '*.py')"
                },
                "case_sensitive": {
                    "type": "boolean",
                    "description": "Case sensitive search",
                    "default": false
                },
                "context_lines": {
                    "type": "integer",
                    "description": "Number of context lines before/after match",
                    "default": 2
                },
                "max_results": {
                    "type": "integer",
                    "description": "Maximum number of matches to return",
                    "default": 50
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
        let glob_pattern = input["glob"].as_str();
        let case_sensitive = input["case_sensitive"].as_bool().unwrap_or(false);
        let context_lines = input["context_lines"]
            .as_u64()
            .unwrap_or(DEFAULT_CONTEXT_LINES as u64) as usize;
        let max_results = input["max_results"]
            .as_u64()
            .unwrap_or(DEFAULT_MAX_RESULTS as u64) as usize;

        // Resolve search path
        let search_path = ctx.resolve_path(path)?;

        if !search_path.exists() {
            return Err(ToolError::ExecutionFailed {
                reason: format!("Path not found: {}", path),
            });
        }

        // Compile regex
        let regex = if case_sensitive {
            Regex::new(pattern)
        } else {
            Regex::new(&format!("(?i){}", pattern))
        }
        .map_err(|e| ToolError::InvalidInput {
            reason: format!("Invalid regex pattern: {}", e),
        })?;

        // Compile glob if provided
        let glob = glob_pattern
            .map(|p| glob::Pattern::new(p))
            .transpose()
            .map_err(|e| ToolError::InvalidInput {
                reason: format!("Invalid glob pattern: {}", e),
            })?;

        // Walk directory and search
        let mut matches: Vec<SearchMatch> = Vec::new();
        let walker = WalkBuilder::new(&search_path)
            .hidden(false)
            .git_ignore(true)
            .build();

        'outer: for entry in walker {
            if matches.len() >= max_results {
                break;
            }

            let entry = match entry {
                Ok(e) => e,
                Err(_) => continue,
            };

            let file_path = entry.path();

            // Skip non-files
            if !file_path.is_file() {
                continue;
            }

            // Check file size
            if let Ok(metadata) = file_path.metadata() {
                if metadata.len() > MAX_FILE_SIZE {
                    continue;
                }
            }

            // Get relative path
            let rel_path = file_path
                .strip_prefix(&search_path)
                .unwrap_or(file_path)
                .to_string_lossy()
                .replace('\\', "/");

            // Check glob pattern if provided
            if let Some(ref g) = glob {
                let file_name = file_path.file_name().and_then(|n| n.to_str()).unwrap_or("");
                if !g.matches(&rel_path) && !g.matches(file_name) {
                    continue;
                }
            }

            // Read and search file
            let content = match fs::read_to_string(file_path) {
                Ok(c) => c,
                Err(_) => continue, // Skip binary/unreadable files
            };

            let lines: Vec<&str> = content.lines().collect();

            for (i, line) in lines.iter().enumerate() {
                if matches.len() >= max_results {
                    break 'outer;
                }

                if regex.is_match(line) {
                    // Get context
                    let start = i.saturating_sub(context_lines);
                    let end = (i + context_lines + 1).min(lines.len());

                    let before: Vec<String> = lines[start..i].iter().map(|s| s.to_string()).collect();
                    let after: Vec<String> = lines[(i + 1)..end].iter().map(|s| s.to_string()).collect();

                    matches.push(SearchMatch {
                        file: rel_path.clone(),
                        line: i + 1, // 1-indexed
                        content: line.to_string(),
                        before,
                        after,
                    });
                }
            }
        }

        let total_matches = matches.len();

        // Convert to JSON
        let results: Vec<Value> = matches
            .into_iter()
            .map(|m| {
                json!({
                    "file": m.file,
                    "line": m.line,
                    "content": m.content,
                    "before": m.before,
                    "after": m.after
                })
            })
            .collect();

        let result = json!({
            "pattern": pattern,
            "path": path,
            "results": results,
            "total_matches": total_matches,
            "is_limited": total_matches >= max_results
        });

        Ok(ToolOutput::success(self.name(), result, 0))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::TempDir;

    #[tokio::test]
    async fn test_grep_search_basic() {
        let temp_dir = TempDir::new().unwrap();

        // Create test file
        let mut file = fs::File::create(temp_dir.path().join("test.txt")).unwrap();
        writeln!(file, "Hello World").unwrap();
        writeln!(file, "Goodbye World").unwrap();
        writeln!(file, "Hello Again").unwrap();

        let tool = GrepSearchTool;
        let ctx = create_test_context(temp_dir.path());

        let input = json!({
            "pattern": "Hello"
        });

        let result = tool.execute(input, &ctx).await;
        assert!(result.is_ok());

        let output = result.unwrap();
        assert!(output.success);
        assert_eq!(output.result["total_matches"], 2);
    }

    #[tokio::test]
    async fn test_grep_search_with_glob() {
        let temp_dir = TempDir::new().unwrap();

        // Create test files
        fs::write(temp_dir.path().join("test.rs"), "fn main() {}").unwrap();
        fs::write(temp_dir.path().join("test.py"), "def main():").unwrap();

        let tool = GrepSearchTool;
        let ctx = create_test_context(temp_dir.path());

        let input = json!({
            "pattern": "main",
            "glob": "*.rs"
        });

        let result = tool.execute(input, &ctx).await;
        assert!(result.is_ok());

        let output = result.unwrap();
        assert!(output.success);
        assert_eq!(output.result["total_matches"], 1);
    }

    #[tokio::test]
    async fn test_grep_search_case_insensitive() {
        let temp_dir = TempDir::new().unwrap();

        fs::write(temp_dir.path().join("test.txt"), "HELLO world").unwrap();

        let tool = GrepSearchTool;
        let ctx = create_test_context(temp_dir.path());

        let input = json!({
            "pattern": "hello",
            "case_sensitive": false
        });

        let result = tool.execute(input, &ctx).await;
        assert!(result.is_ok());

        let output = result.unwrap();
        assert_eq!(output.result["total_matches"], 1);
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
