//! task_track tool - Track tasks in session
//!
//! Allows the agent to create, list, and manage tasks during a session.

use super::traits::{Tool, ToolContext, ToolError, ToolOutput};
use crate::permissions::Permission;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::sync::{Arc, RwLock};

/// A tracked task
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Task {
    pub id: u32,
    pub text: String,
    pub completed: bool,
    pub created_at: String,
}

/// Tool for tracking tasks within a session
pub struct TaskTrackTool {
    tasks: Arc<RwLock<Vec<Task>>>,
    next_id: Arc<RwLock<u32>>,
}

impl TaskTrackTool {
    pub fn new() -> Self {
        Self {
            tasks: Arc::new(RwLock::new(Vec::new())),
            next_id: Arc::new(RwLock::new(1)),
        }
    }

    fn add_task(&self, text: &str) -> Task {
        let mut tasks = self.tasks.write().unwrap();
        let mut next_id = self.next_id.write().unwrap();

        let task = Task {
            id: *next_id,
            text: text.to_string(),
            completed: false,
            created_at: chrono::Utc::now().to_rfc3339(),
        };

        *next_id += 1;
        tasks.push(task.clone());
        task
    }

    fn list_tasks(&self) -> Vec<Task> {
        self.tasks.read().unwrap().clone()
    }

    fn complete_task(&self, id: u32) -> Option<Task> {
        let mut tasks = self.tasks.write().unwrap();
        if let Some(task) = tasks.iter_mut().find(|t| t.id == id) {
            task.completed = true;
            return Some(task.clone());
        }
        None
    }

    fn clear_completed(&self) -> usize {
        let mut tasks = self.tasks.write().unwrap();
        let before = tasks.len();
        tasks.retain(|t| !t.completed);
        before - tasks.len()
    }

    fn clear_all(&self) -> usize {
        let mut tasks = self.tasks.write().unwrap();
        let count = tasks.len();
        tasks.clear();
        count
    }
}

impl Default for TaskTrackTool {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl Tool for TaskTrackTool {
    fn name(&self) -> &'static str {
        "task_track"
    }

    fn description(&self) -> &'static str {
        "Track tasks during the session. Use to plan and track progress on multi-step work."
    }

    fn input_schema(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "action": {
                    "type": "string",
                    "enum": ["add", "list", "complete", "clear", "clear_all"],
                    "description": "Action to perform"
                },
                "text": {
                    "type": "string",
                    "description": "Task description (for 'add' action)"
                },
                "id": {
                    "type": "integer",
                    "description": "Task ID (for 'complete' action)"
                }
            },
            "required": ["action"]
        })
    }

    fn required_permission(&self) -> Permission {
        Permission::WorkspaceWrite
    }

    async fn execute(&self, input: Value, _ctx: &ToolContext) -> Result<ToolOutput, ToolError> {
        let action = input["action"]
            .as_str()
            .ok_or_else(|| ToolError::InvalidInput {
                reason: "action is required".to_string(),
            })?;

        match action {
            "add" => {
                let text = input["text"]
                    .as_str()
                    .ok_or_else(|| ToolError::InvalidInput {
                        reason: "text is required for 'add' action".to_string(),
                    })?;

                let task = self.add_task(text);
                let result = json!({
                    "action": "add",
                    "task": task,
                    "message": format!("Added task #{}: {}", task.id, task.text)
                });
                Ok(ToolOutput::success(self.name(), result, 0))
            }
            "list" => {
                let tasks = self.list_tasks();
                let pending: Vec<_> = tasks.iter().filter(|t| !t.completed).collect();
                let completed: Vec<_> = tasks.iter().filter(|t| t.completed).collect();

                let result = json!({
                    "action": "list",
                    "tasks": tasks,
                    "pending_count": pending.len(),
                    "completed_count": completed.len(),
                    "total_count": tasks.len()
                });
                Ok(ToolOutput::success(self.name(), result, 0))
            }
            "complete" => {
                let id = input["id"]
                    .as_u64()
                    .ok_or_else(|| ToolError::InvalidInput {
                        reason: "id is required for 'complete' action".to_string(),
                    })? as u32;

                if let Some(task) = self.complete_task(id) {
                    let result = json!({
                        "action": "complete",
                        "task": task,
                        "message": format!("Completed task #{}: {}", task.id, task.text)
                    });
                    Ok(ToolOutput::success(self.name(), result, 0))
                } else {
                    Err(ToolError::ExecutionFailed {
                        reason: format!("Task #{} not found", id),
                    })
                }
            }
            "clear" => {
                let cleared = self.clear_completed();
                let result = json!({
                    "action": "clear",
                    "cleared_count": cleared,
                    "message": format!("Cleared {} completed tasks", cleared)
                });
                Ok(ToolOutput::success(self.name(), result, 0))
            }
            "clear_all" => {
                let cleared = self.clear_all();
                let result = json!({
                    "action": "clear_all",
                    "cleared_count": cleared,
                    "message": format!("Cleared all {} tasks", cleared)
                });
                Ok(ToolOutput::success(self.name(), result, 0))
            }
            _ => Err(ToolError::InvalidInput {
                reason: format!("Unknown action: {}", action),
            }),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_add_and_list_tasks() {
        let tool = TaskTrackTool::new();
        let ctx = create_test_context();

        // Add a task
        let input = json!({
            "action": "add",
            "text": "Fix the bug"
        });
        let result = tool.execute(input, &ctx).await.unwrap();
        assert!(result.success);
        assert_eq!(result.result["task"]["id"], 1);

        // Add another task
        let input = json!({
            "action": "add",
            "text": "Write tests"
        });
        tool.execute(input, &ctx).await.unwrap();

        // List tasks
        let input = json!({ "action": "list" });
        let result = tool.execute(input, &ctx).await.unwrap();
        assert_eq!(result.result["total_count"], 2);
        assert_eq!(result.result["pending_count"], 2);
    }

    #[tokio::test]
    async fn test_complete_task() {
        let tool = TaskTrackTool::new();
        let ctx = create_test_context();

        // Add a task
        let input = json!({
            "action": "add",
            "text": "Fix the bug"
        });
        tool.execute(input, &ctx).await.unwrap();

        // Complete it
        let input = json!({
            "action": "complete",
            "id": 1
        });
        let result = tool.execute(input, &ctx).await.unwrap();
        assert!(result.success);
        assert!(result.result["task"]["completed"].as_bool().unwrap());

        // List to verify
        let input = json!({ "action": "list" });
        let result = tool.execute(input, &ctx).await.unwrap();
        assert_eq!(result.result["completed_count"], 1);
    }

    fn create_test_context() -> ToolContext {
        use crate::permissions::PermissionEnforcer;
        use std::env;
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
