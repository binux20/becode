//! Provider traits and types

use crate::tools::ToolSpec;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::pin::Pin;
use thiserror::Error;
use tokio_stream::Stream;

/// Provider errors
#[derive(Debug, Error)]
pub enum ProviderError {
    #[error("Missing API key for provider: {0}")]
    MissingApiKey(String),

    #[error("Unknown provider: {0}")]
    UnknownProvider(String),

    #[error("API error: {status} - {message}")]
    ApiError { status: u16, message: String },

    #[error("Network error: {0}")]
    NetworkError(String),

    #[error("Parse error: {0}")]
    ParseError(String),

    #[error("Rate limited: {0}")]
    RateLimited(String),

    #[error("Timeout")]
    Timeout,

    #[error("Invalid response: {0}")]
    InvalidResponse(String),
}

/// Message role
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum MessageRole {
    System,
    User,
    Assistant,
    Tool,
}

/// Message content types
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum MessageContent {
    Text(String),
    Parts(Vec<ContentPart>),
}

impl MessageContent {
    pub fn text(s: impl Into<String>) -> Self {
        Self::Text(s.into())
    }

    pub fn as_text(&self) -> Option<&str> {
        match self {
            Self::Text(s) => Some(s),
            Self::Parts(parts) => parts.iter().find_map(|p| {
                if let ContentPart::Text { text } = p {
                    Some(text.as_str())
                } else {
                    None
                }
            }),
        }
    }
}

/// Content part for multimodal messages
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum ContentPart {
    #[serde(rename = "text")]
    Text { text: String },

    #[serde(rename = "image_url")]
    ImageUrl { image_url: ImageUrl },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImageUrl {
    pub url: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub detail: Option<String>,
}

/// A conversation message
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Message {
    pub role: MessageRole,
    pub content: MessageContent,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool_call_id: Option<String>,
}

impl Message {
    pub fn system(content: impl Into<String>) -> Self {
        Self {
            role: MessageRole::System,
            content: MessageContent::text(content),
            name: None,
            tool_call_id: None,
        }
    }

    pub fn user(content: impl Into<String>) -> Self {
        Self {
            role: MessageRole::User,
            content: MessageContent::text(content),
            name: None,
            tool_call_id: None,
        }
    }

    pub fn assistant(content: impl Into<String>) -> Self {
        Self {
            role: MessageRole::Assistant,
            content: MessageContent::text(content),
            name: None,
            tool_call_id: None,
        }
    }

    pub fn tool_result(tool_call_id: impl Into<String>, content: impl Into<String>) -> Self {
        Self {
            role: MessageRole::Tool,
            content: MessageContent::text(content),
            name: None,
            tool_call_id: Some(tool_call_id.into()),
        }
    }

    pub fn with_image(mut self, image_url: impl Into<String>) -> Self {
        let text = match &self.content {
            MessageContent::Text(t) => t.clone(),
            MessageContent::Parts(parts) => parts
                .iter()
                .find_map(|p| {
                    if let ContentPart::Text { text } = p {
                        Some(text.clone())
                    } else {
                        None
                    }
                })
                .unwrap_or_default(),
        };

        self.content = MessageContent::Parts(vec![
            ContentPart::Text { text },
            ContentPart::ImageUrl {
                image_url: ImageUrl {
                    url: image_url.into(),
                    detail: Some("auto".to_string()),
                },
            },
        ]);
        self
    }
}

/// Attachment for messages (images, files)
#[derive(Debug, Clone)]
pub struct Attachment {
    pub filename: String,
    pub mime_type: String,
    pub data: AttachmentData,
}

/// Attachment data variants
#[derive(Debug, Clone)]
pub enum AttachmentData {
    Base64(String),
    FilePath(PathBuf),
    Url(String),
}

impl Attachment {
    /// Create from file path
    pub fn from_file(path: PathBuf) -> Self {
        let filename = path
            .file_name()
            .map(|n| n.to_string_lossy().to_string())
            .unwrap_or_default();
        let mime_type = mime_guess::from_path(&path)
            .first_or_octet_stream()
            .to_string();

        Self {
            filename,
            mime_type,
            data: AttachmentData::FilePath(path),
        }
    }

    /// Create from URL
    pub fn from_url(url: impl Into<String>) -> Self {
        let url = url.into();
        Self {
            filename: url.split('/').last().unwrap_or("image").to_string(),
            mime_type: "image/png".to_string(), // Default, will be detected
            data: AttachmentData::Url(url),
        }
    }

    /// Convert to base64 data URL
    pub async fn to_data_url(&self) -> Result<String, ProviderError> {
        match &self.data {
            AttachmentData::Base64(data) => {
                Ok(format!("data:{};base64,{}", self.mime_type, data))
            }
            AttachmentData::FilePath(path) => {
                let data = tokio::fs::read(path)
                    .await
                    .map_err(|e| ProviderError::ParseError(e.to_string()))?;
                let encoded = base64::Engine::encode(&base64::engine::general_purpose::STANDARD, &data);
                Ok(format!("data:{};base64,{}", self.mime_type, encoded))
            }
            AttachmentData::Url(url) => Ok(url.clone()),
        }
    }
}

/// Response from LLM
#[derive(Debug, Clone)]
pub struct LLMResponse {
    pub content: String,
    pub tool_calls: Vec<LLMToolCall>,
    pub finish_reason: Option<String>,
    pub usage: Option<Usage>,
}

/// Tool call from LLM response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LLMToolCall {
    pub id: String,
    pub name: String,
    pub arguments: serde_json::Value,
}

/// Token usage
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Usage {
    pub prompt_tokens: u32,
    pub completion_tokens: u32,
    pub total_tokens: u32,
}

/// Stream chunk for streaming responses
#[derive(Debug, Clone)]
pub enum StreamChunk {
    Text(String),
    ToolCallStart { id: String, name: String },
    ToolCallDelta { id: String, arguments_delta: String },
    ToolCallEnd { id: String },
    Done,
    Error(String),
}

/// The core LLM Provider trait
#[async_trait]
pub trait LLMProvider: Send + Sync {
    /// Provider name
    fn name(&self) -> &str;

    /// Default model for this provider
    fn default_model(&self) -> &str;

    /// Whether this provider supports tool/function calling natively
    fn supports_tools(&self) -> bool;

    /// Whether this provider supports streaming
    fn supports_streaming(&self) -> bool;

    /// Whether this provider supports vision (images)
    fn supports_vision(&self) -> bool;

    /// Send a chat request
    async fn chat(
        &self,
        messages: &[Message],
        tools: Option<&[ToolSpec]>,
        attachments: Option<&[Attachment]>,
        model: Option<&str>,
    ) -> Result<LLMResponse, ProviderError>;

    /// Send a streaming chat request
    async fn chat_stream(
        &self,
        messages: &[Message],
        tools: Option<&[ToolSpec]>,
        attachments: Option<&[Attachment]>,
        model: Option<&str>,
    ) -> Result<Pin<Box<dyn Stream<Item = StreamChunk> + Send>>, ProviderError>;
}
