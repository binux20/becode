//! Anthropic Claude provider

use super::traits::{
    Attachment, AttachmentData, ContentPart, LLMProvider, LLMResponse, LLMToolCall,
    Message, MessageContent, MessageRole, ProviderError, StreamChunk, Usage,
};
use crate::tools::ToolSpec;
use async_trait::async_trait;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::pin::Pin;
use tokio_stream::Stream;

const API_URL: &str = "https://api.anthropic.com/v1/messages";
const API_VERSION: &str = "2023-06-01";

pub struct AnthropicProvider {
    api_key: String,
    client: Client,
}

impl AnthropicProvider {
    pub fn new(api_key: String) -> Self {
        Self {
            api_key,
            client: Client::new(),
        }
    }

    fn convert_messages(&self, messages: &[Message]) -> Vec<ApiMessage> {
        let mut api_messages = Vec::new();

        for msg in messages {
            if msg.role == MessageRole::System {
                continue; // System handled separately
            }

            let role = match msg.role {
                MessageRole::User => "user",
                MessageRole::Assistant => "assistant",
                MessageRole::Tool => "user",
                MessageRole::System => continue,
            };

            let content = match &msg.content {
                MessageContent::Text(text) => {
                    if msg.role == MessageRole::Tool {
                        vec![ApiContent::ToolResult {
                            tool_use_id: msg.tool_call_id.clone().unwrap_or_default(),
                            content: text.clone(),
                        }]
                    } else {
                        vec![ApiContent::Text { text: text.clone() }]
                    }
                }
                MessageContent::Parts(parts) => {
                    parts.iter().map(|p| match p {
                        ContentPart::Text { text } => ApiContent::Text { text: text.clone() },
                        ContentPart::ImageUrl { image_url } => ApiContent::Image {
                            source: ImageSource {
                                source_type: "url".to_string(),
                                url: Some(image_url.url.clone()),
                                media_type: None,
                                data: None,
                            },
                        },
                    }).collect()
                }
            };

            api_messages.push(ApiMessage {
                role: role.to_string(),
                content,
            });
        }

        api_messages
    }

    fn convert_tools(&self, tools: &[ToolSpec]) -> Vec<ApiTool> {
        tools.iter().map(|t| ApiTool {
            name: t.name.clone(),
            description: t.description.clone(),
            input_schema: t.input_schema.clone(),
        }).collect()
    }

    fn extract_system(&self, messages: &[Message]) -> Option<String> {
        messages.iter()
            .find(|m| m.role == MessageRole::System)
            .and_then(|m| m.content.as_text().map(|s| s.to_string()))
    }
}

#[derive(Debug, Serialize)]
struct ApiRequest {
    model: String,
    messages: Vec<ApiMessage>,
    max_tokens: u32,
    #[serde(skip_serializing_if = "Option::is_none")]
    system: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    tools: Option<Vec<ApiTool>>,
}

#[derive(Debug, Serialize)]
struct ApiMessage {
    role: String,
    content: Vec<ApiContent>,
}

#[derive(Debug, Serialize)]
#[serde(tag = "type")]
enum ApiContent {
    #[serde(rename = "text")]
    Text { text: String },
    #[serde(rename = "image")]
    Image { source: ImageSource },
    #[serde(rename = "tool_use")]
    ToolUse {
        id: String,
        name: String,
        input: Value,
    },
    #[serde(rename = "tool_result")]
    ToolResult {
        tool_use_id: String,
        content: String,
    },
}

#[derive(Debug, Serialize)]
struct ImageSource {
    #[serde(rename = "type")]
    source_type: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    url: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    media_type: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    data: Option<String>,
}

#[derive(Debug, Serialize)]
struct ApiTool {
    name: String,
    description: String,
    input_schema: Value,
}

#[derive(Debug, Deserialize)]
struct ApiResponse {
    content: Vec<ResponseContent>,
    stop_reason: Option<String>,
    usage: Option<ApiUsage>,
}

#[derive(Debug, Deserialize)]
#[serde(tag = "type")]
enum ResponseContent {
    #[serde(rename = "text")]
    Text { text: String },
    #[serde(rename = "tool_use")]
    ToolUse { id: String, name: String, input: Value },
}

#[derive(Debug, Deserialize)]
struct ApiUsage {
    input_tokens: u32,
    output_tokens: u32,
}

#[async_trait]
impl LLMProvider for AnthropicProvider {
    fn name(&self) -> &str {
        "anthropic"
    }

    fn default_model(&self) -> &str {
        "claude-sonnet-4-20250514"
    }

    fn supports_tools(&self) -> bool {
        true
    }

    fn supports_streaming(&self) -> bool {
        true
    }

    fn supports_vision(&self) -> bool {
        true
    }

    async fn chat(
        &self,
        messages: &[Message],
        tools: Option<&[ToolSpec]>,
        _attachments: Option<&[Attachment]>,
        model: Option<&str>,
    ) -> Result<LLMResponse, ProviderError> {
        let model = model.unwrap_or(self.default_model());
        let system = self.extract_system(messages);
        let api_messages = self.convert_messages(messages);
        let api_tools = tools.map(|t| self.convert_tools(t));

        let request = ApiRequest {
            model: model.to_string(),
            messages: api_messages,
            max_tokens: 4096,
            system,
            tools: api_tools,
        };

        let response = self.client
            .post(API_URL)
            .header("x-api-key", &self.api_key)
            .header("anthropic-version", API_VERSION)
            .header("content-type", "application/json")
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

        let mut content = String::new();
        let mut tool_calls = Vec::new();

        for item in api_response.content {
            match item {
                ResponseContent::Text { text } => {
                    content.push_str(&text);
                }
                ResponseContent::ToolUse { id, name, input } => {
                    tool_calls.push(LLMToolCall {
                        id,
                        name,
                        arguments: input,
                    });
                }
            }
        }

        let usage = api_response.usage.map(|u| Usage {
            prompt_tokens: u.input_tokens,
            completion_tokens: u.output_tokens,
            total_tokens: u.input_tokens + u.output_tokens,
        });

        Ok(LLMResponse {
            content,
            tool_calls,
            finish_reason: api_response.stop_reason,
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
        let response = self.chat(messages, tools, attachments, model).await?;
        let chunks = vec![
            StreamChunk::Text(response.content),
            StreamChunk::Done,
        ];
        Ok(Box::pin(tokio_stream::iter(chunks)))
    }
}
