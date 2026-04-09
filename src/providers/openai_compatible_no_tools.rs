//! OpenAI-Compatible provider WITHOUT native tool calling
//!
//! This is a KEY feature of BeCode - supports models that don't have function calling
//! by using JSON blocks in the prompt for tool calls.
//!
//! Tool calls are extracted from ```json blocks in the response.

use super::traits::{
    Attachment, LLMProvider, LLMResponse, LLMToolCall, Message, MessageContent,
    MessageRole, ProviderError, StreamChunk, Usage,
};
use crate::tools::ToolSpec;
use async_trait::async_trait;
use regex::Regex;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::pin::Pin;
use tokio_stream::Stream;

/// OpenAI-Compatible provider for models WITHOUT native tool calling
/// Uses JSON blocks in responses for tool calls
pub struct OpenAICompatibleNoToolsProvider {
    name: String,
    base_url: String,
    api_key: Option<String>,
    model: Option<String>,
}

impl OpenAICompatibleNoToolsProvider {
    pub fn new(
        name: String,
        base_url: String,
        api_key: Option<String>,
        model: Option<String>,
    ) -> Self {
        Self {
            name,
            base_url,
            api_key,
            model,
        }
    }

    /// Build the system prompt that teaches the model how to use tools via JSON blocks
    fn build_tool_system_prompt(&self, tools: &[ToolSpec]) -> String {
        let tools_description: Vec<Value> = tools
            .iter()
            .map(|t| {
                json!({
                    "name": t.name,
                    "description": t.description,
                    "parameters": t.input_schema
                })
            })
            .collect();

        let tools_json = serde_json::to_string_pretty(&tools_description).unwrap_or_default();

        format!(
            r#"You are BeCode, an autonomous coding agent.

You have access to these tools:
{}

To use a tool, respond with a JSON block:
```json
{{"tool": "tool_name", "args": {{...}}}}
```

You can call multiple tools by returning multiple JSON blocks.

When the task is complete, respond with:
```json
{{"done": true, "message": "What was accomplished"}}
```

IMPORTANT:
- Always read files before editing
- Use edit_file for small changes, write_file for new files
- After each tool call, you'll see the result and can continue"#,
            tools_json
        )
    }

    /// Parse tool calls from response text (extracts JSON blocks)
    fn parse_tool_calls_from_text(&self, response: &str) -> Vec<LLMToolCall> {
        let mut calls = Vec::new();

        // Pattern 1: ```json blocks
        let json_block_re = Regex::new(r"```json\s*\n([\s\S]*?)\n```").unwrap();
        for cap in json_block_re.captures_iter(response) {
            if let Some(json_str) = cap.get(1) {
                if let Ok(parsed) = self.parse_single_tool_call(json_str.as_str()) {
                    calls.push(parsed);
                }
            }
        }

        // Pattern 2: ``` blocks without json marker
        if calls.is_empty() {
            let block_re = Regex::new(r"```\s*\n([\s\S]*?)\n```").unwrap();
            for cap in block_re.captures_iter(response) {
                if let Some(content) = cap.get(1) {
                    let trimmed = content.as_str().trim();
                    if trimmed.starts_with('{') {
                        if let Ok(parsed) = self.parse_single_tool_call(trimmed) {
                            calls.push(parsed);
                        }
                    }
                }
            }
        }

        // Pattern 3: Inline JSON objects (fallback)
        if calls.is_empty() {
            let inline_re = Regex::new(r#"\{\s*"tool"\s*:\s*"[^"]+"[^}]+\}"#).unwrap();
            for mat in inline_re.find_iter(response) {
                if let Ok(parsed) = self.parse_single_tool_call(mat.as_str()) {
                    calls.push(parsed);
                }
            }
        }

        calls
    }

    /// Parse a single tool call from JSON string
    fn parse_single_tool_call(&self, json_str: &str) -> Result<LLMToolCall, ()> {
        let json_str = json_str.trim();

        // Try to parse as JSON
        let value: Value = serde_json::from_str(json_str).map_err(|_| ())?;

        // Check for "done" response
        if value.get("done").and_then(|v| v.as_bool()).unwrap_or(false) {
            let message = value
                .get("message")
                .and_then(|v| v.as_str())
                .unwrap_or("Task completed")
                .to_string();

            return Ok(LLMToolCall {
                id: format!("done_{}", uuid::Uuid::new_v4()),
                name: "__done__".to_string(),
                arguments: json!({ "message": message }),
            });
        }

        // Extract tool name
        let tool_name = value
            .get("tool")
            .and_then(|v| v.as_str())
            .ok_or(())?
            .to_string();

        // Extract arguments
        let args = value
            .get("args")
            .cloned()
            .unwrap_or_else(|| json!({}));

        Ok(LLMToolCall {
            id: format!("call_{}", uuid::Uuid::new_v4()),
            name: tool_name,
            arguments: args,
        })
    }
}

#[derive(Debug, Serialize)]
struct ChatRequest {
    model: String,
    messages: Vec<ChatMessage>,
    #[serde(skip_serializing_if = "Option::is_none")]
    max_tokens: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    temperature: Option<f32>,
    stream: bool,
}

#[derive(Debug, Serialize, Deserialize)]
struct ChatMessage {
    role: String,
    content: String,
}

#[derive(Debug, Deserialize)]
struct ChatResponse {
    choices: Vec<Choice>,
    usage: Option<ApiUsage>,
}

#[derive(Debug, Deserialize)]
struct Choice {
    message: ResponseMessage,
}

#[derive(Debug, Deserialize)]
struct ResponseMessage {
    content: Option<String>,
}

#[derive(Debug, Deserialize)]
struct ApiUsage {
    prompt_tokens: u32,
    completion_tokens: u32,
    total_tokens: u32,
}

#[async_trait]
impl LLMProvider for OpenAICompatibleNoToolsProvider {
    fn name(&self) -> &str {
        &self.name
    }

    fn default_model(&self) -> &str {
        self.model.as_deref().unwrap_or("llama3.1:8b")
    }

    fn supports_tools(&self) -> bool {
        // We support tools via JSON blocks, not native function calling
        true
    }

    fn supports_streaming(&self) -> bool {
        true
    }

    fn supports_vision(&self) -> bool {
        false
    }

    async fn chat(
        &self,
        messages: &[Message],
        tools: Option<&[ToolSpec]>,
        _attachments: Option<&[Attachment]>,
    ) -> Result<LLMResponse, ProviderError> {
        let client = reqwest::Client::new();

        // Build messages with tool system prompt
        let mut chat_messages: Vec<ChatMessage> = Vec::new();

        // Add tool system prompt if tools are provided
        if let Some(tool_specs) = tools {
            if !tool_specs.is_empty() {
                chat_messages.push(ChatMessage {
                    role: "system".to_string(),
                    content: self.build_tool_system_prompt(tool_specs),
                });
            }
        }

        // Convert messages
        for msg in messages {
            let role = match msg.role {
                MessageRole::User => "user",
                MessageRole::Assistant => "assistant",
                MessageRole::System => "system",
                MessageRole::ToolResult => "user", // Tool results go as user messages
            };

            let content = match &msg.content {
                MessageContent::Text(text) => text.clone(),
                MessageContent::ToolResult { tool_use_id, result } => {
                    format!("Tool result for {}:\n{}", tool_use_id, result)
                }
                MessageContent::MultiPart(_) => continue,
            };

            chat_messages.push(ChatMessage {
                role: role.to_string(),
                content,
            });
        }

        let request = ChatRequest {
            model: self.model.clone().unwrap_or_else(|| "llama3.1:8b".to_string()),
            messages: chat_messages,
            max_tokens: Some(4096),
            temperature: Some(0.7),
            stream: false,
        };

        let url = format!("{}/chat/completions", self.base_url.trim_end_matches('/'));

        let mut req_builder = client.post(&url).json(&request);

        if let Some(ref key) = self.api_key {
            if !key.is_empty() {
                req_builder = req_builder.header("Authorization", format!("Bearer {}", key));
            }
        }

        let response = req_builder.send().await.map_err(|e| ProviderError::Network {
            message: e.to_string(),
        })?;

        if !response.status().is_success() {
            let status = response.status();
            let text = response.text().await.unwrap_or_default();
            return Err(ProviderError::Api {
                status_code: status.as_u16(),
                message: text,
            });
        }

        let chat_response: ChatResponse =
            response.json().await.map_err(|e| ProviderError::Parse {
                message: e.to_string(),
            })?;

        let content = chat_response
            .choices
            .first()
            .and_then(|c| c.message.content.clone())
            .unwrap_or_default();

        // Parse tool calls from the response text
        let tool_calls = self.parse_tool_calls_from_text(&content);

        let usage = chat_response.usage.map(|u| Usage {
            input_tokens: u.prompt_tokens,
            output_tokens: u.completion_tokens,
        });

        Ok(LLMResponse {
            content: if tool_calls.is_empty() {
                Some(content)
            } else {
                // If we have tool calls, the content might just be the JSON blocks
                // Return the non-JSON part as content
                let text_content = self.extract_non_json_content(&content);
                if text_content.is_empty() {
                    None
                } else {
                    Some(text_content)
                }
            },
            tool_calls: if tool_calls.is_empty() {
                None
            } else {
                Some(tool_calls)
            },
            usage,
            model: self.model.clone(),
            stop_reason: None,
        })
    }

    async fn chat_stream(
        &self,
        messages: &[Message],
        tools: Option<&[ToolSpec]>,
        attachments: Option<&[Attachment]>,
    ) -> Result<Pin<Box<dyn Stream<Item = StreamChunk> + Send>>, ProviderError> {
        // For simplicity, use non-streaming and convert to stream
        let response = self.chat(messages, tools, attachments).await?;

        let chunks: Vec<StreamChunk> = vec![
            StreamChunk::Content(response.content.unwrap_or_default()),
            StreamChunk::Done,
        ];

        Ok(Box::pin(tokio_stream::iter(chunks)))
    }
}

impl OpenAICompatibleNoToolsProvider {
    /// Extract non-JSON content from response (text before/after JSON blocks)
    fn extract_non_json_content(&self, response: &str) -> String {
        // Remove JSON blocks
        let re = Regex::new(r"```(?:json)?\s*\n[\s\S]*?\n```").unwrap();
        let cleaned = re.replace_all(response, "");
        cleaned.trim().to_string()
    }
}
