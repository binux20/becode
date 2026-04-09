//! Agent runtime - the main conversation loop

use crate::config::Config;
use crate::permissions::{Permission, PermissionEnforcer};
use crate::providers::{LLMProvider, LLMResponse, Message};
use crate::session::Session;
use crate::tools::{ToolCall, ToolContext, ToolOutput, ToolRegistry};
use anyhow::Result;
use std::path::PathBuf;
use std::sync::Arc;

use super::parser::{ParsedResponse, ResponseParser};
use super::recovery::RecoveryStrategy;

/// Result of an agent run
#[derive(Debug)]
pub enum AgentResult {
    /// Task completed successfully
    Success(String),
    /// Max steps reached
    MaxStepsReached(String),
    /// Error occurred
    Error(String),
    /// User cancelled
    Cancelled,
}

/// Callback for agent events
pub trait AgentCallback: Send + Sync {
    fn on_thinking(&self) {}
    fn on_tool_start(&self, _tool: &str, _input: &serde_json::Value) {}
    fn on_tool_end(&self, _output: &ToolOutput) {}
    fn on_text(&self, _text: &str) {}
    fn on_done(&self, _message: &str) {}
    fn on_error(&self, _error: &str) {}
    fn should_cancel(&self) -> bool { false }
    fn confirm_command(&self, _command: &str, _reason: &str) -> bool { false }
}

/// No-op callback for when no UI is attached
pub struct NoopCallback;
impl AgentCallback for NoopCallback {}

/// The main agent runtime
pub struct AgentRuntime {
    pub provider: Arc<dyn LLMProvider>,
    pub tools: ToolRegistry,
    pub session: Session,
    pub permission: Permission,
    pub enforcer: Arc<PermissionEnforcer>,
    pub config: AgentConfig,
    pub parser: ResponseParser,
}

/// Agent configuration
#[derive(Debug, Clone)]
pub struct AgentConfig {
    pub max_steps: u32,
    pub context_limit: u32,
    pub model: Option<String>,
}

impl Default for AgentConfig {
    fn default() -> Self {
        Self {
            max_steps: 25,
            context_limit: 100_000,
            model: None,
        }
    }
}

impl AgentRuntime {
    /// Create a new agent runtime
    pub fn new(
        provider: Arc<dyn LLMProvider>,
        workspace: PathBuf,
        permission: Permission,
        config: AgentConfig,
    ) -> Self {
        let enforcer = Arc::new(PermissionEnforcer::new(permission, workspace.clone()));
        let session = Session::new(
            workspace,
            provider.name().to_string(),
            config.model.clone(),
        );

        Self {
            provider,
            tools: ToolRegistry::new(),
            session,
            permission,
            enforcer,
            config,
            parser: ResponseParser::new(provider.supports_tools()),
        }
    }

    /// Run the agent on a task
    pub async fn run(
        &mut self,
        task: &str,
        callback: &dyn AgentCallback,
    ) -> Result<AgentResult> {
        // Start session
        self.session.start()?;
        self.session.add_user_message(task)?;

        // Build initial messages
        let mut messages = vec![
            Message::system(self.build_system_prompt()),
            Message::user(task),
        ];

        // Agent loop
        for step in 1..=self.config.max_steps {
            if callback.should_cancel() {
                return Ok(AgentResult::Cancelled);
            }

            callback.on_thinking();

            // Get LLM response
            let response = self
                .provider
                .chat(
                    &messages,
                    Some(&self.tools.specs()),
                    None,
                    self.config.model.as_deref(),
                )
                .await?;

            // Parse response
            let parsed = self.parser.parse(&response);

            match parsed {
                ParsedResponse::ToolCalls(calls) => {
                    // Handle tool calls
                    let results = self.execute_tools(&calls, callback).await?;

                    // Add assistant message with tool calls
                    messages.push(Message::assistant(&response.content));

                    // Add tool results
                    for (call, result) in calls.iter().zip(results.iter()) {
                        let result_json = serde_json::to_string_pretty(&result.result)?;
                        messages.push(Message::tool_result(
                            call.id.as_deref().unwrap_or(&call.tool),
                            &result_json,
                        ));

                        // Record in session
                        self.session.add_tool_call(
                            &call.tool,
                            call.args.clone(),
                            result.result.clone(),
                            result.duration_ms,
                        )?;
                    }
                }
                ParsedResponse::Done(message) => {
                    self.session.add_assistant_message(&message)?;
                    self.session.end(Some(message.clone()))?;
                    callback.on_done(&message);
                    return Ok(AgentResult::Success(message));
                }
                ParsedResponse::Text(text) => {
                    callback.on_text(&text);
                    messages.push(Message::assistant(&text));
                    self.session.add_assistant_message(&text)?;

                    // If no tools and just text, might be done
                    if !self.provider.supports_tools() {
                        // For no-tools providers, check if this looks like a final answer
                        if !text.contains("```json") {
                            self.session.end(Some(text.clone()))?;
                            callback.on_done(&text);
                            return Ok(AgentResult::Success(text));
                        }
                    }
                }
                ParsedResponse::Invalid(raw) => {
                    // Recovery: ask model to try again
                    callback.on_error("Invalid response format, retrying...");
                    messages.push(Message::assistant(&raw));
                    messages.push(Message::user(
                        "Invalid response format. Please respond with tool calls or a final answer.",
                    ));
                }
            }
        }

        let msg = format!("Max steps ({}) reached", self.config.max_steps);
        self.session.end(Some(msg.clone()))?;
        Ok(AgentResult::MaxStepsReached(msg))
    }

    /// Execute tool calls (in parallel when possible)
    async fn execute_tools(
        &self,
        calls: &[ToolCall],
        callback: &dyn AgentCallback,
    ) -> Result<Vec<ToolOutput>> {
        let ctx = ToolContext {
            workspace_root: self.session.project.clone(),
            permission: self.permission,
            enforcer: self.enforcer.clone(),
        };

        // Execute all tools in parallel
        let mut results = Vec::new();

        for call in calls {
            callback.on_tool_start(&call.tool, &call.args);

            let result = self.tools.execute(call, &ctx).await?;

            callback.on_tool_end(&result);
            results.push(result);
        }

        Ok(results)
    }

    /// Build system prompt
    fn build_system_prompt(&self) -> String {
        let tools_info = if self.provider.supports_tools() {
            String::new()
        } else {
            // For no-tools providers, include tool instructions in system prompt
            format!(
                "\n\nYou have access to tools. To use them, respond with JSON blocks.\n\
                Available tools:\n{}",
                self.tools.format_for_no_tools_prompt()
            )
        };

        format!(
            r#"You are BeCode 🐝, an autonomous AI coding agent.

You help users with software development tasks by reading, writing, and editing code.

Workspace: {}
Permission level: {}

Guidelines:
- Read files before editing them
- Use edit_file for small changes, write_file for new files
- Run tests after making changes when appropriate
- Be concise in explanations
- Ask for clarification if the task is unclear{}
"#,
            self.session.project.display(),
            self.permission,
            tools_info
        )
    }
}
