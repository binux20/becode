//! OpenRouter provider (multi-model aggregator)

use super::traits::{
    Attachment, LLMProvider, LLMResponse, Message, ProviderError, StreamChunk,
};
use crate::tools::ToolSpec;
use async_trait::async_trait;
use std::pin::Pin;
use tokio_stream::Stream;

pub struct OpenRouterProvider {
    api_key: String,
}

impl OpenRouterProvider {
    pub fn new(api_key: String) -> Self {
        Self { api_key }
    }
}

#[async_trait]
impl LLMProvider for OpenRouterProvider {
    fn name(&self) -> &str {
        "openrouter"
    }

    fn default_model(&self) -> &str {
        "anthropic/claude-sonnet-4"
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
        _messages: &[Message],
        _tools: Option<&[ToolSpec]>,
        _attachments: Option<&[Attachment]>,
        _model: Option<&str>,
    ) -> Result<LLMResponse, ProviderError> {
        todo!("OpenRouter provider not yet implemented")
    }

    async fn chat_stream(
        &self,
        _messages: &[Message],
        _tools: Option<&[ToolSpec]>,
        _attachments: Option<&[Attachment]>,
        _model: Option<&str>,
    ) -> Result<Pin<Box<dyn Stream<Item = StreamChunk> + Send>>, ProviderError> {
        todo!("OpenRouter streaming not yet implemented")
    }
}
