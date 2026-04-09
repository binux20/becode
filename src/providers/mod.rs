//! LLM Provider system for BeCode
//!
//! Supports multiple providers:
//! - Anthropic (Claude)
//! - OpenAI (GPT)
//! - Gemini (Google)
//! - Mistral
//! - OpenRouter
//! - OpenAI-Compatible (generic)
//! - OpenAI-Compatible-NoTools (for models without function calling)

mod traits;
mod anthropic;
mod openai;
mod gemini;
mod mistral;
mod openrouter;
mod openai_compatible;
mod openai_compatible_no_tools;

pub use traits::{
    Attachment, AttachmentData, LLMProvider, LLMResponse, Message, MessageContent,
    ProviderError, StreamChunk,
};

pub use anthropic::AnthropicProvider;
pub use openai::OpenAIProvider;
pub use gemini::GeminiProvider;
pub use mistral::MistralProvider;
pub use openrouter::OpenRouterProvider;
pub use openai_compatible::OpenAICompatibleProvider;
pub use openai_compatible_no_tools::OpenAICompatibleNoToolsProvider;

use crate::config::{Config, ProviderConfig, SecretStore};
use std::sync::Arc;

/// Create a provider from configuration
pub fn create_provider(
    name: &str,
    config: &Config,
    secrets: &SecretStore,
) -> Result<Arc<dyn LLMProvider>, ProviderError> {
    // Check for custom provider config first
    if let Some(provider_config) = config.providers.get(name) {
        return create_from_config(name, provider_config, secrets);
    }

    // Built-in providers
    match name {
        "anthropic" => {
            let api_key = secrets
                .get_key("anthropic")
                .ok_or_else(|| ProviderError::MissingApiKey("anthropic".to_string()))?;
            Ok(Arc::new(AnthropicProvider::new(api_key)))
        }
        "openai" => {
            let api_key = secrets
                .get_key("openai")
                .ok_or_else(|| ProviderError::MissingApiKey("openai".to_string()))?;
            Ok(Arc::new(OpenAIProvider::new(api_key)))
        }
        "gemini" => {
            let api_key = secrets
                .get_key("gemini")
                .ok_or_else(|| ProviderError::MissingApiKey("gemini".to_string()))?;
            Ok(Arc::new(GeminiProvider::new(api_key)))
        }
        "mistral" => {
            let api_key = secrets
                .get_key("mistral")
                .ok_or_else(|| ProviderError::MissingApiKey("mistral".to_string()))?;
            Ok(Arc::new(MistralProvider::new(api_key)))
        }
        "openrouter" => {
            let api_key = secrets
                .get_key("openrouter")
                .ok_or_else(|| ProviderError::MissingApiKey("openrouter".to_string()))?;
            Ok(Arc::new(OpenRouterProvider::new(api_key)))
        }
        _ => Err(ProviderError::UnknownProvider(name.to_string())),
    }
}

/// Create provider from explicit config
fn create_from_config(
    name: &str,
    config: &ProviderConfig,
    secrets: &SecretStore,
) -> Result<Arc<dyn LLMProvider>, ProviderError> {
    let provider_type = config
        .provider_type
        .as_deref()
        .unwrap_or("openai-compatible");

    // Get API key (from config, secrets, or env)
    let api_key = config
        .api_key
        .as_ref()
        .map(|k| Config::resolve_env(k))
        .or_else(|| secrets.get_key(name));

    let base_url = config
        .base_url
        .as_ref()
        .map(|u| Config::resolve_env(u))
        .unwrap_or_default();

    let model = config.model.clone();

    match provider_type {
        "anthropic" => {
            let key = api_key.ok_or_else(|| ProviderError::MissingApiKey(name.to_string()))?;
            Ok(Arc::new(AnthropicProvider::new(key)))
        }
        "openai" => {
            let key = api_key.ok_or_else(|| ProviderError::MissingApiKey(name.to_string()))?;
            Ok(Arc::new(OpenAIProvider::new(key)))
        }
        "openai-compatible" => {
            Ok(Arc::new(OpenAICompatibleProvider::new(
                name.to_string(),
                base_url,
                api_key,
                model,
            )))
        }
        "openai-compatible-no-tools" => {
            Ok(Arc::new(OpenAICompatibleNoToolsProvider::new(
                name.to_string(),
                base_url,
                api_key,
                model,
            )))
        }
        _ => Err(ProviderError::UnknownProvider(format!(
            "{}:{}",
            name, provider_type
        ))),
    }
}
