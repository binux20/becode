//! read_file tool - Read file contents with optional windowing

use super::traits::{Tool, ToolContext, ToolError, ToolOutput};
use crate::permissions::Permission;
use async_trait::async_trait;
use serde_json::{json, Value};
use std::fs;

/// Tool for reading file contents
pub struct ReadFileTool;

#[async_trait]
impl Tool for ReadFileTool {
    fn name(&self) -> &'static str {
        "read_file"
    }

    fn description(&self) -> &'static str {
        "Read the contents of a file. Supports offset and limit for large files."
    }

    fn input_schema(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "path": {
                    "type": "string",
                    "description": "Path to the file (relative to project root)"
                },
                "offset": {
                    "type": "integer",
                    "description": "Line number to start reading from (0-based)",
                    "default": 0
                },
                "limit": {
                    "type": "integer",
                    "description": "Maximum number of lines to read",
                    "default": 500
                }
            },
            "required": ["path"]
        })
    }

    fn required_permission(&self) -> Permission {
        Permission::ReadOnly
    }

    async fn execute(&self, input: Value, ctx: &ToolContext) -> Result<ToolOutput, ToolError> {
        let path = input["path"]
            .as_str()
            .ok_or_else(|| ToolError::InvalidInput {
                reason: "path is required".to_string(),
            })?;

        let offset = input["offset"].as_u64().unwrap_or(0) as usize;
        let limit = input["limit"].as_u64().unwrap_or(500) as usize;

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

        // Read file content
        let content = fs::read_to_string(&full_path).map_err(|e| ToolError::ExecutionFailed {
            reason: format!("Failed to read file: {}", e),
        })?;

        let lines: Vec<&str> = content.lines().collect();
        let total_lines = lines.len();

        // Apply offset and limit
        let start = offset.min(total_lines);
        let end = (start + limit).min(total_lines);
        let selected_lines: Vec<&str> = lines[start..end].to_vec();
        let is_truncated = end < total_lines;

        // Build result with line numbers
        let numbered_content: String = selected_lines
            .iter()
            .enumerate()
            .map(|(i, line)| format!("{:>4} | {}", start + i + 1, line))
            .collect::<Vec<_>>()
            .join("\n");

        let result = json!({
            "path": path,
            "content": numbered_content,
            "total_lines": total_lines,
            "start_line": start + 1,
            "end_line": end,
            "is_truncated": is_truncated,
            "bytes": content.len()
        });

        Ok(ToolOutput::success(self.name(), result, 0))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[tokio::test]
    async fn test_read_file_basic() {
        // Create temp file
        let mut file = NamedTempFile::new().unwrap();
        writeln!(file, "line 1").unwrap();
        writeln!(file, "line 2").unwrap();
        writeln!(file, "line 3").unwrap();

        let tool = ReadFileTool;
        let ctx = create_test_context(file.path().parent().unwrap());

        let input = json!({
            "path": file.path().file_name().unwrap().to_str().unwrap()
        });

        let result = tool.execute(input, &ctx).await;
        assert!(result.is_ok());

        let output = result.unwrap();
        assert!(output.success);
        assert_eq!(output.result["total_lines"], 3);
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
