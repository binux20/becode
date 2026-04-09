//! Generic OpenAI-Compatible provider (with native tool calling)

use super::traits::{
    Attachment, LLMProvider, LLMResponse, LLMToolCall, Message, MessageContent,
    MessageRole, ProviderError, StreamChunk, Usage,
};
use crate::tools::ToolSpec;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::pin::Pin;
use tokio_stream::Stream;

/// Generic OpenAI-Compatible provider with native tool calling support
pub struct OpenAICompatibleProvider {
    name: String,
    base_url: String,
    api_key: Option<String>,
    model: Option<String>,
}

impl OpenAICompatibleProvider {
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

    /// Convert tools to OpenAI function format
    fn tools_to_functions(tools: &[ToolSpec]) -> Vec<Value> {
        tools
            .iter()
            .map(|t| {
                json!({
                    "type": "function",
                    "function": {
                        "name": t.name,
                        "description": t.description,
                        "parameters": t.input_schema
                    }
                })
            })
            .collect()
    }

    /// Convert messages to OpenAI format
    fn messages_to_api(messages: &[Message]) -> Vec<Value> {
        messages
            .iter()
            .map(|msg| {
                let role = match msg.role {
                    MessageRole::System => "system",
                    MessageRole::User => "user",
                    MessageRole::Assistant => "assistant",
                    MessageRole::Tool => "tool",
                };

                let content = match &msg.content {
                    MessageContent::Text(t) => json!(t),
                    MessageContent::Parts(parts) => {
                        let api_parts: Vec<Value> = parts
                            .iter()
                            .map(|p| match p {
                                super::traits::ContentPart::Text { text } => {
                                    json!({ "type": "text", "text": text })
                                }
                                super::traits::ContentPart::ImageUrl { image_url } => {
                                    json!({
                                        "type": "image_url",
                                        "image_url": {
                                            "url": image_url.url,
                                            "detail": image_url.detail
                                        }
                                    })
                                }
                            })
                            .collect();
                        json!(api_parts)
                    }
                };

                let mut msg_json = json!({
                    "role": role,
                    "content": content
                });

                if let Some(ref tool_call_id) = msg.tool_call_id {
                    msg_json["tool_call_id"] = json!(tool_call_id);
                }

                msg_json
            })
            .collect()
    }
}

#[derive(Debug, Deserialize)]
struct ChatCompletionResponse {
    choices: Vec<ChatChoice>,
    usage: Option<UsageResponse>,
}

#[derive(Debug, Deserialize)]
struct ChatChoice {
    message: ChatMessageResponse,
    finish_reason: Option<String>,
}

#[derive(Debug, Deserialize)]
struct ChatMessageResponse {
    content: Option<String>,
    tool_calls: Option<Vec<ToolCallResponse>>,
}

#[derive(Debug, Deserialize)]
struct ToolCallResponse {
    id: String,
    function: FunctionCallResponse,
}

#[derive(Debug, Deserialize)]
struct FunctionCallResponse {
    name: String,
    arguments: String,
}

#[derive(Debug, Deserialize)]
struct UsageResponse {
    prompt_tokens: u32,
    completion_tokens: u32,
    total_tokens: u32,
}

#[async_trait]
impl LLMProvider for OpenAICompatibleProvider {
    fn name(&self) -> &str {
        &self.name
    }

    fn default_model(&self) -> &str {
        self.model.as_deref().unwrap_or("gpt-4o")
    }

    fn supports_tools(&self) -> bool {
        true
    }

    fn supports_streaming(&self) -> bool {
        true
    }

    fn supports_vision(&self) -> bool {
        true // Assume yes, user can configure
    }

    async fn chat(
        &self,
        messages: &[Message],
        tools: Option<&[ToolSpec]>,
        _attachments: Option<&[Attachment]>,
        model: Option<&str>,
    ) -> Result<LLMResponse, ProviderError> {
        let model = model.unwrap_or_else(|| self.default_model());
        let client = reqwest::Client::new();

        let url = format!("{}/chat/completions", self.base_url.trim_end_matches('/'));

        let mut request = client
            .post(&url)
            .header("Content-Type", "application/json");

        if let Some(ref api_key) = self.api_key {
            if !api_key.is_empty() {
                request = request.header("Authorization", format!("Bearer {}", api_key));
            }
        }

        let mut body = json!({
            "model": model,
            "messages": Self::messages_to_api(messages),
            "temperature": 0.1
        });

        if let Some(tools) = tools {
            if !tools.is_empty() {
                body["tools"] = json!(Self::tools_to_functions(tools));
                body["tool_choice"] = json!("auto");
            }
        }

        let response = request
            .json(&body)
            .send()
            .await
            .map_err(|e| ProviderError::NetworkError(e.to_string()))?;

        let status = response.status();
        if !status.is_success() {
            let error_text = response.text().await.unwrap_or_default();
            return Err(ProviderError::ApiError {
                status: status.as_u16(),
                message: error_text,
            });
        }

        let response_json: ChatCompletionResponse = response
            .json()
            .await
            .map_err(|e| ProviderError::ParseError(e.to_string()))?;

        let choice = response_json
            .choices
            .first()
            .ok_or_else(|| ProviderError::InvalidResponse("No choices in response".to_string()))?;

        let content = choice.message.content.clone().unwrap_or_default();

        let tool_calls: Vec<LLMToolCall> = choice
            .message
            .tool_calls
            .as_ref()
            .map(|calls| {
                calls
                    .iter()
                    .filter_map(|tc| {
                        let args: Value = serde_json::from_str(&tc.function.arguments).ok()?;
                        Some(LLMToolCall {
                            id: tc.id.clone(),
                            name: tc.function.name.clone(),
                            arguments: args,
                        })
                    })
                    .collect()
            })
            .unwrap_or_default();

        let usage = response_json.usage.map(|u| Usage {
            prompt_tokens: u.prompt_tokens,
            completion_tokens: u.completion_tokens,
            total_tokens: u.total_tokens,
        });

        Ok(LLMResponse {
            content,
            tool_calls,
            finish_reason: choice.finish_reason.clone(),
            usage,
        })
    }

    async fn chat_stream(
        &self,
        messages: &[Message],
        tools: Option<&[ToolSpec]>,
        attachments: Option<&[Attachment]>,
        model: Option<&str>,
    ) -> Result<Pin<Box<dyn Stream<Item = StreamChunk> + Send>>, ProviderError> {
        // For now, use non-streaming
        // TODO: Implement SSE streaming
        let response = self.chat(messages, tools, attachments, model).await?;

        let mut chunks = vec![StreamChunk::Text(response.content)];

        for tc in response.tool_calls {
            chunks.push(StreamChunk::ToolCallStart {
                id: tc.id.clone(),
                name: tc.name,
            });
            chunks.push(StreamChunk::ToolCallDelta {
                id: tc.id.clone(),
                arguments_delta: tc.arguments.to_string(),
            });
            chunks.push(StreamChunk::ToolCallEnd { id: tc.id });
        }

        chunks.push(StreamChunk::Done);

        Ok(Box::pin(tokio_stream::iter(chunks)))
    }
}
