//! Tool registry for managing available tools

use super::{
    traits::{Tool, ToolCall, ToolContext, ToolError, ToolOutput, ToolSpec},
    BashTool, EditFileTool, GlobSearchTool, GrepSearchTool, ReadFileTool,
    TaskTrackTool, WebFetchTool, WebSearchTool, WriteFileTool,
};
use crate::permissions::PermissionEnforcer;
use serde_json::Value;
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Instant;

/// Registry of available tools
pub struct ToolRegistry {
    tools: HashMap<String, Arc<dyn Tool>>,
}

impl ToolRegistry {
    /// Create a new registry with all default tools
    pub fn new() -> Self {
        let mut registry = Self {
            tools: HashMap::new(),
        };

        // Register all built-in tools
        registry.register(Arc::new(BashTool));
        registry.register(Arc::new(ReadFileTool));
        registry.register(Arc::new(WriteFileTool));
        registry.register(Arc::new(EditFileTool));
        registry.register(Arc::new(GlobSearchTool));
        registry.register(Arc::new(GrepSearchTool));
        registry.register(Arc::new(WebFetchTool));
        registry.register(Arc::new(WebSearchTool));
        registry.register(Arc::new(TaskTrackTool::new()));

        registry
    }

    /// Register a tool
    pub fn register(&mut self, tool: Arc<dyn Tool>) {
        self.tools.insert(tool.name().to_string(), tool);
    }

    /// Get a tool by name
    pub fn get(&self, name: &str) -> Option<Arc<dyn Tool>> {
        self.tools.get(name).cloned()
    }

    /// Get all tool specifications for LLM
    pub fn specs(&self) -> Vec<ToolSpec> {
        self.tools.values().map(|t| t.spec()).collect()
    }

    /// Get tool names
    pub fn names(&self) -> Vec<&str> {
        self.tools.keys().map(|s| s.as_str()).collect()
    }

    /// Execute a tool call
    pub async fn execute(
        &self,
        call: &ToolCall,
        ctx: &ToolContext,
    ) -> Result<ToolOutput, ToolError> {
        let tool = self.get(&call.tool).ok_or_else(|| ToolError::NotFound {
            tool: call.tool.clone(),
        })?;

        // Check permission before execution
        ctx.enforcer
            .check_tool(&call.tool, tool.required_permission())
            .map_err(|e| ToolError::PermissionDenied {
                reason: e.to_string(),
            })?;

        // Execute with timing
        let start = Instant::now();
        let result = tool.execute(call.args.clone(), ctx).await;
        let duration_ms = start.elapsed().as_millis() as u32;

        match result {
            Ok(mut output) => {
                output.duration_ms = duration_ms;
                if let Some(ref id) = call.id {
                    output.call_id = Some(id.clone());
                }
                Ok(output)
            }
            Err(e) => {
                let mut output = ToolOutput::failure(&call.tool, e.to_string(), duration_ms);
                if let Some(ref id) = call.id {
                    output.call_id = Some(id.clone());
                }
                Ok(output)
            }
        }
    }

    /// Execute multiple tool calls
    /// Independent calls are executed in parallel
    pub async fn execute_many(
        &self,
        calls: Vec<ToolCall>,
        ctx: &ToolContext,
    ) -> Vec<ToolOutput> {
        // For now, execute all in parallel
        // Future: analyze dependencies and batch appropriately
        let futures: Vec<_> = calls
            .into_iter()
            .map(|call| {
                let registry = self;
                async move { registry.execute(&call, ctx).await }
            })
            .collect();

        // Execute all futures concurrently
        let results = futures::future::join_all(futures).await;

        // Convert Results to ToolOutputs
        results
            .into_iter()
            .map(|r| r.unwrap_or_else(|e| ToolOutput::failure("unknown", e.to_string(), 0)))
            .collect()
    }

    /// Format tool specs for NoTools provider (JSON block format)
    pub fn format_for_no_tools_prompt(&self) -> String {
        let specs: Vec<Value> = self
            .specs()
            .iter()
            .map(|spec| {
                serde_json::json!({
                    "name": spec.name,
                    "description": spec.description,
                    "parameters": spec.input_schema,
                })
            })
            .collect();

        serde_json::to_string_pretty(&specs).unwrap_or_default()
    }
}

impl Default for ToolRegistry {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_registry_has_all_tools() {
        let registry = ToolRegistry::new();
        let names = registry.names();

        assert!(names.contains(&"bash"));
        assert!(names.contains(&"read_file"));
        assert!(names.contains(&"write_file"));
        assert!(names.contains(&"edit_file"));
        assert!(names.contains(&"glob_search"));
        assert!(names.contains(&"grep_search"));
        assert!(names.contains(&"web_fetch"));
        assert!(names.contains(&"web_search"));
        assert!(names.contains(&"task_track"));
    }

    #[test]
    fn test_get_tool() {
        let registry = ToolRegistry::new();

        assert!(registry.get("bash").is_some());
        assert!(registry.get("nonexistent").is_none());
    }

    #[test]
    fn test_specs() {
        let registry = ToolRegistry::new();
        let specs = registry.specs();

        assert!(!specs.is_empty());
        assert!(specs.iter().any(|s| s.name == "bash"));
    }
}
