//! Mistral AI provider

use super::traits::{
    Attachment, ContentPart, LLMProvider, LLMResponse, LLMToolCall,
    Message, MessageContent, MessageRole, ProviderError, StreamChunk, Usage,
};
use crate::tools::ToolSpec;
use async_trait::async_trait;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::pin::Pin;
use tokio_stream::Stream;

const API_URL: &str = "https://api.mistral.ai/v1/chat/completions";

pub struct MistralProvider {
    api_key: String,
    client: Client,
}

impl MistralProvider {
    pub fn new(api_key: String) -> Self {
        Self {
            api_key,
            client: Client::new(),
        }
    }

    fn convert_messages(&self, messages: &[Message]) -> Vec<ApiMessage> {
        messages.iter().map(|msg| {
            let role = match msg.role {
                MessageRole::System => "system",
                MessageRole::User => "user",
                MessageRole::Assistant => "assistant",
                MessageRole::Tool => "tool",
            };

            let content = match &msg.content {
                MessageContent::Text(text) => text.clone(),
                MessageContent::Parts(parts) => {
                    parts.iter().filter_map(|p| {
                        if let ContentPart::Text { text } = p {
                            Some(text.clone())
                        } else {
                            None
                        }
                    }).collect::<Vec<_>>().join("\n")
                }
            };

            ApiMessage {
                role: role.to_string(),
                content,
                tool_call_id: msg.tool_call_id.clone(),
                tool_calls: None,
            }
        }).collect()
    }

    fn convert_tools(&self, tools: &[ToolSpec]) -> Vec<ApiTool> {
        tools.iter().map(|t| ApiTool {
            tool_type: "function".to_string(),
            function: ApiFunctionDef {
                name: t.name.clone(),
                description: t.description.clone(),
                parameters: t.input_schema.clone(),
            },
        }).collect()
    }
}

#[derive(Debug, Serialize)]
struct ApiRequest {
    model: String,
    messages: Vec<ApiMessage>,
    #[serde(skip_serializing_if = "Option::is_none")]
    tools: Option<Vec<ApiTool>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    max_tokens: Option<u32>,
}

#[derive(Debug, Serialize)]
struct ApiMessage {
    role: String,
    content: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    tool_call_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    tool_calls: Option<Vec<ApiToolCall>>,
}

#[derive(Debug, Serialize)]
struct ApiTool {
    #[serde(rename = "type")]
    tool_type: String,
    function: ApiFunctionDef,
}

#[derive(Debug, Serialize)]
struct ApiFunctionDef {
    name: String,
    description: String,
    parameters: Value,
}

#[derive(Debug, Serialize, Deserialize)]
struct ApiToolCall {
    id: String,
    #[serde(rename = "type")]
    call_type: String,
    function: ApiFunctionCall,
}

#[derive(Debug, Serialize, Deserialize)]
struct ApiFunctionCall {
    name: String,
    arguments: String,
}

#[derive(Debug, Deserialize)]
struct ApiResponse {
    choices: Vec<Choice>,
    usage: Option<ApiUsage>,
}

#[derive(Debug, Deserialize)]
struct Choice {
    message: ResponseMessage,
    finish_reason: Option<String>,
}

#[derive(Debug, Deserialize)]
struct ResponseMessage {
    content: Option<String>,
    tool_calls: Option<Vec<ApiToolCall>>,
}

#[derive(Debug, Deserialize)]
struct ApiUsage {
    prompt_tokens: u32,
    completion_tokens: u32,
    total_tokens: u32,
}

#[async_trait]
impl LLMProvider for MistralProvider {
    fn name(&self) -> &str {
        "mistral"
    }

    fn default_model(&self) -> &str {
        "mistral-large-latest"
    }

    fn supports_tools(&self) -> bool {
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
        model: Option<&str>,
    ) -> Result<LLMResponse, ProviderError> {
        let model = model.unwrap_or(self.default_model());
        let api_messages = self.convert_messages(messages);
        let api_tools = tools.filter(|t| !t.is_empty()).map(|t| self.convert_tools(t));

        let request = ApiRequest {
            model: model.to_string(),
            messages: api_messages,
            tools: api_tools,
            max_tokens: Some(4096),
        };

        let response = self.client
            .post(API_URL)
            .header("Authorization", format!("Bearer {}", self.api_key))
            .header("Content-Type", "application/json")
            .json(&request)
            .send()
            .await
            .map_err(|e| ProviderError::NetworkError(e.to_string()))?;

        if !response.status().is_success() {
            let status = response.status().as_u16();
            let text = response.text().await.unwrap_or_default();
            return Err(ProviderError::ApiError { status, message: text });
        }

        let api_response: ApiResponse = response
            .json()
            .await
            .map_err(|e| ProviderError::ParseError(e.to_string()))?;

        let choice = api_response.choices.into_iter().next()
            .ok_or_else(|| ProviderError::InvalidResponse("No choices".to_string()))?;

        let content = choice.message.content.unwrap_or_default();

        let tool_calls: Vec<LLMToolCall> = choice.message.tool_calls
            .unwrap_or_default()
            .into_iter()
            .filter_map(|tc| {
                let args: Value = serde_json::from_str(&tc.function.arguments).ok()?;
                Some(LLMToolCall {
                    id: tc.id,
                    name: tc.function.name,
                    arguments: args,
                })
            })
            .collect();

        let usage = api_response.usage.map(|u| Usage {
            prompt_tokens: u.prompt_tokens,
            completion_tokens: u.completion_tokens,
            total_tokens: u.total_tokens,
        });

        Ok(LLMResponse {
            content,
            tool_calls,
            finish_reason: choice.finish_reason,
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
        let response = self.chat(messages, tools, attachments, model).await?;
        let chunks = vec![
            StreamChunk::Text(response.content),
            StreamChunk::Done,
        ];
        Ok(Box::pin(tokio_stream::iter(chunks)))
    }
}
