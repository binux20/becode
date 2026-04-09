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
            r#"You are BeCode, an autonomous AI coding agent.

You have access to the following tools:

{tools_json}

## How to use tools

To use a tool, respond with a JSON block like this:

```json
{{"tool": "tool_name", "args": {{"param1": "value1", "param2": "value2"}}}}
```

You can call MULTIPLE tools in a single response by including multiple JSON blocks.

After each tool call, you will receive the result. Then you can continue with more tool calls or provide a final response.

## When you're done

When the task is complete, respond with:

```json
{{"done": true, "message": "Summary of what was accomplished"}}
```

## Important rules

1. ALWAYS read files before editing them
2. Use `edit_file` for small changes (preferred), `write_file` for new files or complete rewrites
3. Use `glob_search` to find files, `grep_search` to search content
4. After making changes, verify by reading the file or running tests
5. For multi-step tasks, plan your approach first"#
        )
    }

    /// Make HTTP request to the API
    async fn make_request(
        &self,
        messages: Vec<ChatMessage>,
        model: &str,
    ) -> Result<String, ProviderError> {
        let client = reqwest::Client::new();

        let request = ChatRequest {
            model: model.to_string(),
            messages,
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

        Ok(content)
    }

    /// Parse tool calls from response text (extracts JSON blocks)
    fn parse_tool_calls(&self, response: &str) -> Vec<LLMToolCall> {
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
        let args = value.get("args").cloned().unwrap_or_else(|| json!({}));

        Ok(LLMToolCall {
            id: format!("call_{}", uuid::Uuid::new_v4()),
            name: tool_name,
            arguments: args,
        })
    }

    /// Check if response contains a "done" signal
    fn is_done_response(&self, response: &str) -> Option<String> {
        let json_block_re = Regex::new(r"```json\s*\n([\s\S]*?)\n```").unwrap();
        for cap in json_block_re.captures_iter(response) {
            if let Some(json_str) = cap.get(1) {
                if let Ok(value) = serde_json::from_str::<Value>(json_str.as_str()) {
                    if value.get("done").and_then(|v| v.as_bool()).unwrap_or(false) {
                        return value
                            .get("message")
                            .and_then(|v| v.as_str())
                            .map(|s| s.to_string());
                    }
                }
            }
        }
        None
    }

    /// Extract non-JSON content from response (text before/after JSON blocks)
    fn extract_non_json_content(&self, response: &str) -> String {
        let re = Regex::new(r"```(?:json)?\s*\n[\s\S]*?\n```").unwrap();
        let cleaned = re.replace_all(response, "");
        cleaned.trim().to_string()
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

#[derive(Debug, Serialize, Deserialize, Clone)]
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
    #[allow(dead_code)]
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
        true // We support tools via JSON blocks
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
        let model = self.model.as_deref().unwrap_or("llama3.1:8b");

        // Build messages with tool system prompt
        let mut api_messages: Vec<ChatMessage> = Vec::new();

        // Add tool system prompt if tools provided
        if let Some(tools) = tools {
            if !tools.is_empty() {
                api_messages.push(ChatMessage {
                    role: "system".to_string(),
                    content: self.build_tool_system_prompt(tools),
                });
            }
        }

        // Add conversation messages
        for msg in messages {
            let role = match msg.role {
                MessageRole::System => "system",
                MessageRole::User => "user",
                MessageRole::Assistant => "assistant",
                MessageRole::ToolResult => "user", // Tool results sent as user messages
            };

            let content = match &msg.content {
                MessageContent::Text(t) => t.clone(),
                MessageContent::ToolResult { tool_use_id, result } => {
                    format!("Tool result for {}:\n{}", tool_use_id, result)
                }
                MessageContent::MultiPart(parts) => {
                    // Extract text from parts
                    parts
                        .iter()
                        .filter_map(|p| {
                            if let super::traits::ContentPart::Text { text } = p {
                                Some(text.clone())
                            } else {
                                None
                            }
                        })
                        .collect::<Vec<_>>()
                        .join("\n")
                }
            };

            api_messages.push(ChatMessage {
                role: role.to_string(),
                content,
            });
        }

        // Make request
        let response_text = self.make_request(api_messages, model).await?;

        // Parse tool calls from response
        let tool_calls = self.parse_tool_calls(&response_text);

        // Check for done signal
        let stop_reason = if self.is_done_response(&response_text).is_some() {
            Some("done".to_string())
        } else if tool_calls.is_empty() {
            Some("stop".to_string())
        } else {
            Some("tool_calls".to_string())
        };

        Ok(LLMResponse {
            content: if tool_calls.is_empty() {
                Some(response_text)
            } else {
                let text_content = self.extract_non_json_content(&response_text);
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
            usage: None,
            model: Some(model.to_string()),
            stop_reason,
        })
    }

    async fn chat_stream(
        &self,
        messages: &[Message],
        tools: Option<&[ToolSpec]>,
        attachments: Option<&[Attachment]>,
    ) -> Result<Pin<Box<dyn Stream<Item = StreamChunk> + Send>>, ProviderError> {
        // For now, use non-streaming and return as single chunk
        let response = self.chat(messages, tools, attachments).await?;

        let chunks = vec![
            StreamChunk::Content(response.content.unwrap_or_default()),
            StreamChunk::Done,
        ];

        Ok(Box::pin(tokio_stream::iter(chunks)))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_tool_calls() {
        let provider = OpenAICompatibleNoToolsProvider::new(
            "test".to_string(),
            "http://localhost:11434/v1".to_string(),
            None,
            None,
        );

        let response = r#"
Let me read the file first.

```json
{"tool": "read_file", "args": {"path": "src/main.rs"}}
```
"#;

        let calls = provider.parse_tool_calls(response);
        assert_eq!(calls.len(), 1);
        assert_eq!(calls[0].name, "read_file");
        assert_eq!(calls[0].arguments["path"], "src/main.rs");
    }

    #[test]
    fn test_parse_multiple_tool_calls() {
        let provider = OpenAICompatibleNoToolsProvider::new(
            "test".to_string(),
            "http://localhost:11434/v1".to_string(),
            None,
            None,
        );

        let response = r#"
Let me read multiple files.

```json
{"tool": "read_file", "args": {"path": "src/main.rs"}}
```

```json
{"tool": "read_file", "args": {"path": "Cargo.toml"}}
```
"#;

        let calls = provider.parse_tool_calls(response);
        assert_eq!(calls.len(), 2);
    }

    #[test]
    fn test_parse_done_response() {
        let provider = OpenAICompatibleNoToolsProvider::new(
            "test".to_string(),
            "http://localhost:11434/v1".to_string(),
            None,
            None,
        );

        let response = r#"
Task completed!

```json
{"done": true, "message": "Successfully fixed the bug"}
```
"#;

        let done_msg = provider.is_done_response(response);
        assert!(done_msg.is_some());
        assert_eq!(done_msg.unwrap(), "Successfully fixed the bug");
    }
}
