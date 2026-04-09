//! write_file tool - Write content to a file

use super::traits::{FilePatch, Tool, ToolContext, ToolError, ToolOutput};
use crate::permissions::Permission;
use crate::utils::diff::generate_unified_diff;
use async_trait::async_trait;
use serde_json::{json, Value};
use std::fs;
use std::path::Path;

/// Tool for writing file contents
pub struct WriteFileTool;

#[async_trait]
impl Tool for WriteFileTool {
    fn name(&self) -> &'static str {
        "write_file"
    }

    fn description(&self) -> &'static str {
        "Write content to a file. Creates the file if it doesn't exist, or overwrites if it does."
    }

    fn input_schema(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "path": {
                    "type": "string",
                    "description": "Path to the file (relative to project root)"
                },
                "content": {
                    "type": "string",
                    "description": "Content to write to the file"
                },
                "create_directories": {
                    "type": "boolean",
                    "description": "Create parent directories if they don't exist",
                    "default": true
                }
            },
            "required": ["path", "content"]
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

        let content = input["content"]
            .as_str()
            .ok_or_else(|| ToolError::InvalidInput {
                reason: "content is required".to_string(),
            })?;

        let create_directories = input["create_directories"].as_bool().unwrap_or(true);

        // Resolve and validate path
        let full_path = ctx.resolve_path(path)?;

        // Read old content for diff (if file exists)
        let old_content = if full_path.exists() {
            fs::read_to_string(&full_path).unwrap_or_default()
        } else {
            String::new()
        };

        // Create parent directories if needed
        if create_directories {
            if let Some(parent) = full_path.parent() {
                fs::create_dir_all(parent).map_err(|e| ToolError::ExecutionFailed {
                    reason: format!("Failed to create directories: {}", e),
                })?;
            }
        }

        // Write content atomically (write to temp, then rename)
        let temp_path = full_path.with_extension("tmp.becode");
        fs::write(&temp_path, content).map_err(|e| ToolError::ExecutionFailed {
            reason: format!("Failed to write file: {}", e),
        })?;

        fs::rename(&temp_path, &full_path).map_err(|e| {
            // Clean up temp file on error
            let _ = fs::remove_file(&temp_path);
            ToolError::ExecutionFailed {
                reason: format!("Failed to rename temp file: {}", e),
            }
        })?;

        // Generate diff
        let diff = generate_unified_diff(&old_content, content, path);
        let lines_added = diff.lines().filter(|l| l.starts_with('+')).count() as u32;
        let lines_removed = diff.lines().filter(|l| l.starts_with('-')).count() as u32;

        let patch = FilePatch {
            path: path.to_string(),
            diff,
            lines_added,
            lines_removed,
        };

        let created = old_content.is_empty();
        let result = json!({
            "path": path,
            "bytes_written": content.len(),
            "lines": content.lines().count(),
            "created": created,
            "lines_added": lines_added,
            "lines_removed": lines_removed
        });

        Ok(ToolOutput::success(self.name(), result, 0).with_patch(patch))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[tokio::test]
    async fn test_write_new_file() {
        let temp_dir = TempDir::new().unwrap();
        let tool = WriteFileTool;
        let ctx = create_test_context(temp_dir.path());

        let input = json!({
            "path": "test.txt",
            "content": "Hello, World!"
        });

        let result = tool.execute(input, &ctx).await;
        assert!(result.is_ok());

        let output = result.unwrap();
        assert!(output.success);
        assert_eq!(output.result["created"], true);

        // Verify file was written
        let content = fs::read_to_string(temp_dir.path().join("test.txt")).unwrap();
        assert_eq!(content, "Hello, World!");
    }

    #[tokio::test]
    async fn test_write_with_directories() {
        let temp_dir = TempDir::new().unwrap();
        let tool = WriteFileTool;
        let ctx = create_test_context(temp_dir.path());

        let input = json!({
            "path": "subdir/nested/test.txt",
            "content": "Nested content"
        });

        let result = tool.execute(input, &ctx).await;
        assert!(result.is_ok());

        // Verify directories and file were created
        let content = fs::read_to_string(temp_dir.path().join("subdir/nested/test.txt")).unwrap();
        assert_eq!(content, "Nested content");
    }

    fn create_test_context(workspace: &Path) -> ToolContext {
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
