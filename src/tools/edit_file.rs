//! edit_file tool - Edit file with string replacement
//!
//! This is the key tool for making precise edits without rewriting entire files.
//! Uses exact string matching (old_string -> new_string).

use super::traits::{FilePatch, Tool, ToolContext, ToolError, ToolOutput};
use crate::permissions::Permission;
use crate::utils::diff::generate_unified_diff;
use async_trait::async_trait;
use serde_json::{json, Value};
use std::fs;

/// Tool for editing files with string replacement
pub struct EditFileTool;

#[async_trait]
impl Tool for EditFileTool {
    fn name(&self) -> &'static str {
        "edit_file"
    }

    fn description(&self) -> &'static str {
        "Edit a file by replacing an exact string with a new string. \
         The old_string must match exactly (including whitespace and indentation). \
         Use this for precise edits instead of rewriting entire files."
    }

    fn input_schema(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "path": {
                    "type": "string",
                    "description": "Path to the file (relative to project root)"
                },
                "old_string": {
                    "type": "string",
                    "description": "Exact string to find and replace (must match exactly)"
                },
                "new_string": {
                    "type": "string",
                    "description": "String to replace old_string with"
                },
                "replace_all": {
                    "type": "boolean",
                    "description": "Replace all occurrences (default: false, only first)",
                    "default": false
                }
            },
            "required": ["path", "old_string", "new_string"]
        })
    }

    fn required_permission(&self) -> Permission {
        Permission::WorkspaceWrite
    }

    async fn execute(&self, input: Value, ctx: &ToolContext) -> Result<ToolOutput, ToolError> {
        let path = input["path"]
            .as_str()
            .ok_or_else(|| ToolError::InvalidInput {
                reason: "path is required".to_string(),
            })?;

        let old_string = input["old_string"]
            .as_str()
            .ok_or_else(|| ToolError::InvalidInput {
                reason: "old_string is required".to_string(),
            })?;

        let new_string = input["new_string"]
            .as_str()
            .ok_or_else(|| ToolError::InvalidInput {
                reason: "new_string is required".to_string(),
            })?;

        let replace_all = input["replace_all"].as_bool().unwrap_or(false);

        // Resolve and validate path
        let full_path = ctx.resolve_path(path)?;

        // Check if file exists
        if !full_path.exists() {
            return Err(ToolError::ExecutionFailed {
                reason: format!("File not found: {}", path),
            });
        }

        if full_path.is_dir() {
            return Err(ToolError::ExecutionFailed {
                reason: format!("Path is a directory: {}", path),
            });
        }

        // Read current content
        let old_content = fs::read_to_string(&full_path).map_err(|e| ToolError::ExecutionFailed {
            reason: format!("Failed to read file: {}", e),
        })?;

        // Check if old_string exists
        if !old_content.contains(old_string) {
            // Provide helpful error with context
            let suggestion = find_similar_string(&old_content, old_string);
            let mut reason = format!("old_string not found in {}", path);
            if let Some(similar) = suggestion {
                reason.push_str(&format!("\n\nDid you mean:\n{}", similar));
            }
            return Err(ToolError::InvalidInput { reason });
        }

        // Count occurrences
        let match_count = old_content.matches(old_string).count();

        // Perform replacement
        let new_content = if replace_all {
            old_content.replace(old_string, new_string)
        } else {
            old_content.replacen(old_string, new_string, 1)
        };

        // Find first match line number
        let first_match_line = old_content
            .lines()
            .enumerate()
            .find(|(_, line)| line.contains(old_string))
            .map(|(i, _)| i + 1)
            .unwrap_or(0);

        // Write atomically
        let temp_path = full_path.with_extension("tmp.becode");
        fs::write(&temp_path, &new_content).map_err(|e| ToolError::ExecutionFailed {
            reason: format!("Failed to write file: {}", e),
        })?;

        fs::rename(&temp_path, &full_path).map_err(|e| {
            let _ = fs::remove_file(&temp_path);
            ToolError::ExecutionFailed {
                reason: format!("Failed to rename temp file: {}", e),
            }
        })?;

        // Generate diff
        let diff = generate_unified_diff(&old_content, &new_content, path);
        let lines_added = diff
            .lines()
            .filter(|l| l.starts_with('+') && !l.starts_with("+++"))
            .count() as u32;
        let lines_removed = diff
            .lines()
            .filter(|l| l.starts_with('-') && !l.starts_with("---"))
            .count() as u32;

        let patch = FilePatch {
            path: path.to_string(),
            diff,
            lines_added,
            lines_removed,
        };

        let replaced_count = if replace_all { match_count } else { 1 };

        let result = json!({
            "path": path,
            "replaced": true,
            "replaced_count": replaced_count,
            "match_count": match_count,
            "first_match_line": first_match_line,
            "lines_added": lines_added,
            "lines_removed": lines_removed
        });

        Ok(ToolOutput::success(self.name(), result, 0).with_patch(patch))
    }
}

/// Find a similar string in content (for helpful error messages)
fn find_similar_string(content: &str, target: &str) -> Option<String> {
    // Simple heuristic: find lines containing parts of the target
    let target_words: Vec<&str> = target.split_whitespace().collect();
    if target_words.is_empty() {
        return None;
    }

    // Look for lines containing the first significant word
    let first_word = target_words.iter().find(|w| w.len() > 3)?;

    let matching_lines: Vec<&str> = content
        .lines()
        .filter(|line| line.contains(*first_word))
        .take(3)
        .collect();

    if matching_lines.is_empty() {
        None
    } else {
        Some(matching_lines.join("\n"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::TempDir;

    #[tokio::test]
    async fn test_edit_file_basic() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("test.txt");

        // Create initial file
        {
            let mut file = fs::File::create(&file_path).unwrap();
            writeln!(file, "Hello, World!").unwrap();
            writeln!(file, "Goodbye, World!").unwrap();
        }

        let tool = EditFileTool;
        let ctx = create_test_context(temp_dir.path());

        let input = json!({
            "path": "test.txt",
            "old_string": "Hello",
            "new_string": "Hi"
        });

        let result = tool.execute(input, &ctx).await;
        assert!(result.is_ok());

        let output = result.unwrap();
        assert!(output.success);
        assert_eq!(output.result["replaced_count"], 1);

        // Verify content
        let content = fs::read_to_string(&file_path).unwrap();
        assert!(content.contains("Hi, World!"));
        assert!(content.contains("Goodbye, World!"));
    }

    #[tokio::test]
    async fn test_edit_file_replace_all() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("test.txt");

        // Create initial file with multiple occurrences
        {
            let mut file = fs::File::create(&file_path).unwrap();
            writeln!(file, "foo bar foo").unwrap();
            writeln!(file, "foo baz foo").unwrap();
        }

        let tool = EditFileTool;
        let ctx = create_test_context(temp_dir.path());

        let input = json!({
            "path": "test.txt",
            "old_string": "foo",
            "new_string": "qux",
            "replace_all": true
        });

        let result = tool.execute(input, &ctx).await;
        assert!(result.is_ok());

        let output = result.unwrap();
        assert!(output.success);
        assert_eq!(output.result["replaced_count"], 4);

        // Verify all replaced
        let content = fs::read_to_string(&file_path).unwrap();
        assert!(!content.contains("foo"));
        assert_eq!(content.matches("qux").count(), 4);
    }

    #[tokio::test]
    async fn test_edit_file_not_found() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("test.txt");

        // Create file
        fs::write(&file_path, "Hello World").unwrap();

        let tool = EditFileTool;
        let ctx = create_test_context(temp_dir.path());

        let input = json!({
            "path": "test.txt",
            "old_string": "Nonexistent",
            "new_string": "Replacement"
        });

        let result = tool.execute(input, &ctx).await;
        assert!(result.is_err());
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
