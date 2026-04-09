//! Settings commands for configuration management

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    pub default_provider: String,
    pub default_model: Option<String>,
    pub project_dir: Option<String>,
    pub permission: String,
    pub theme: String,
    pub sub_agents: SubAgentSettings,
    pub providers: HashMap<String, ProviderConfig>,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            default_provider: "anthropic".to_string(),
            default_model: Some("claude-sonnet-4-20250514".to_string()),
            project_dir: None,
            permission: "workspace-write".to_string(),
            theme: "dark".to_string(),
            sub_agents: SubAgentSettings::default(),
            providers: HashMap::new(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SubAgentSettings {
    pub enabled: bool,
    pub auto_compact: bool,
    pub auto_compact_threshold: u8,
    pub use_explorer: bool,
    pub use_planner: bool,
    pub use_reviewer: bool,
}

impl Default for SubAgentSettings {
    fn default() -> Self {
        Self {
            enabled: true,
            auto_compact: true,
            auto_compact_threshold: 80,
            use_explorer: true,
            use_planner: true,
            use_reviewer: false,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProviderConfig {
    pub provider_type: Option<String>,
    pub base_url: Option<String>,
    pub api_key: Option<String>,
    pub model: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct ProviderInfo {
    pub id: String,
    pub name: String,
    pub description: String,
    pub supports_tools: bool,
    pub supports_vision: bool,
    pub supports_streaming: bool,
}

#[derive(Debug, Clone, Serialize)]
pub struct ModelInfo {
    pub id: String,
    pub name: String,
    pub provider: String,
    pub context_window: u32,
}

fn get_config_path() -> PathBuf {
    dirs::home_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join(".becode")
        .join("config.toml")
}

fn get_secrets_path() -> PathBuf {
    dirs::home_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join(".becode")
        .join("secrets.toml")
}

/// Get current configuration
#[tauri::command]
pub async fn get_config() -> Result<AppConfig, String> {
    let config_path = get_config_path();

    if !config_path.exists() {
        // Create default config
        let config = AppConfig::default();
        save_config(config.clone()).await?;
        return Ok(config);
    }

    let content = std::fs::read_to_string(&config_path)
        .map_err(|e| format!("Failed to read config: {}", e))?;

    let config: AppConfig = toml::from_str(&content)
        .map_err(|e| format!("Failed to parse config: {}", e))?;

    Ok(config)
}

/// Save configuration
#[tauri::command]
pub async fn save_config(config: AppConfig) -> Result<(), String> {
    let config_path = get_config_path();

    // Ensure directory exists
    if let Some(parent) = config_path.parent() {
        std::fs::create_dir_all(parent)
            .map_err(|e| format!("Failed to create config directory: {}", e))?;
    }

    let content = toml::to_string_pretty(&config)
        .map_err(|e| format!("Failed to serialize config: {}", e))?;

    std::fs::write(&config_path, content)
        .map_err(|e| format!("Failed to write config: {}", e))?;

    Ok(())
}

/// Set API key for a provider (stored securely)
#[tauri::command]
pub async fn set_api_key(provider: String, key: String) -> Result<(), String> {
    let secrets_path = get_secrets_path();

    // Load existing secrets
    let mut secrets: HashMap<String, String> = if secrets_path.exists() {
        let content = std::fs::read_to_string(&secrets_path)
            .map_err(|e| format!("Failed to read secrets: {}", e))?;
        toml::from_str(&content).unwrap_or_default()
    } else {
        HashMap::new()
    };

    // Update key
    secrets.insert(provider, key);

    // Ensure directory exists
    if let Some(parent) = secrets_path.parent() {
        std::fs::create_dir_all(parent)
            .map_err(|e| format!("Failed to create secrets directory: {}", e))?;
    }

    // Save secrets
    let content = toml::to_string_pretty(&secrets)
        .map_err(|e| format!("Failed to serialize secrets: {}", e))?;

    std::fs::write(&secrets_path, content)
        .map_err(|e| format!("Failed to write secrets: {}", e))?;

    Ok(())
}

/// Get API key for a provider
#[tauri::command]
pub async fn get_api_key(provider: String) -> Result<Option<String>, String> {
    // First check environment variable
    let env_key = format!("{}_API_KEY", provider.to_uppercase());
    if let Ok(key) = std::env::var(&env_key) {
        if !key.is_empty() {
            return Ok(Some(key));
        }
    }

    // Then check secrets file
    let secrets_path = get_secrets_path();
    if !secrets_path.exists() {
        return Ok(None);
    }

    let content = std::fs::read_to_string(&secrets_path)
        .map_err(|e| format!("Failed to read secrets: {}", e))?;

    let secrets: HashMap<String, String> = toml::from_str(&content).unwrap_or_default();

    Ok(secrets.get(&provider).cloned())
}

/// List available providers
#[tauri::command]
pub async fn list_providers() -> Result<Vec<ProviderInfo>, String> {
    Ok(vec![
        ProviderInfo {
            id: "anthropic".to_string(),
            name: "Anthropic".to_string(),
            description: "Claude models (Opus, Sonnet, Haiku)".to_string(),
            supports_tools: true,
            supports_vision: true,
            supports_streaming: true,
        },
        ProviderInfo {
            id: "openai".to_string(),
            name: "OpenAI".to_string(),
            description: "GPT-4o and GPT-4 models".to_string(),
            supports_tools: true,
            supports_vision: true,
            supports_streaming: true,
        },
        ProviderInfo {
            id: "gemini".to_string(),
            name: "Google Gemini".to_string(),
            description: "Gemini Pro and Flash models".to_string(),
            supports_tools: true,
            supports_vision: true,
            supports_streaming: true,
        },
        ProviderInfo {
            id: "mistral".to_string(),
            name: "Mistral".to_string(),
            description: "Mistral Large and other models".to_string(),
            supports_tools: true,
            supports_vision: false,
            supports_streaming: true,
        },
        ProviderInfo {
            id: "openrouter".to_string(),
            name: "OpenRouter".to_string(),
            description: "Access multiple models via OpenRouter".to_string(),
            supports_tools: true,
            supports_vision: true,
            supports_streaming: true,
        },
        ProviderInfo {
            id: "openai-compatible".to_string(),
            name: "OpenAI Compatible".to_string(),
            description: "Any OpenAI-compatible API with tool calling".to_string(),
            supports_tools: true,
            supports_vision: true,
            supports_streaming: true,
        },
        ProviderInfo {
            id: "openai-compatible-no-tools".to_string(),
            name: "OpenAI Compatible (No Tools)".to_string(),
            description: "OpenAI-compatible API without native tool calling".to_string(),
            supports_tools: false,
            supports_vision: true,
            supports_streaming: true,
        },
    ])
}

/// List models for a provider
#[tauri::command]
pub async fn list_models(provider: String) -> Result<Vec<ModelInfo>, String> {
    let models = match provider.as_str() {
        "anthropic" => vec![
            ModelInfo {
                id: "claude-opus-4-20250514".to_string(),
                name: "Claude Opus 4".to_string(),
                provider: "anthropic".to_string(),
                context_window: 200000,
            },
            ModelInfo {
                id: "claude-sonnet-4-20250514".to_string(),
                name: "Claude Sonnet 4".to_string(),
                provider: "anthropic".to_string(),
                context_window: 200000,
            },
            ModelInfo {
                id: "claude-haiku-4-20250514".to_string(),
                name: "Claude Haiku 4".to_string(),
                provider: "anthropic".to_string(),
                context_window: 200000,
            },
        ],
        "openai" => vec![
            ModelInfo {
                id: "gpt-4o".to_string(),
                name: "GPT-4o".to_string(),
                provider: "openai".to_string(),
                context_window: 128000,
            },
            ModelInfo {
                id: "gpt-4o-mini".to_string(),
                name: "GPT-4o Mini".to_string(),
                provider: "openai".to_string(),
                context_window: 128000,
            },
            ModelInfo {
                id: "gpt-4-turbo".to_string(),
                name: "GPT-4 Turbo".to_string(),
                provider: "openai".to_string(),
                context_window: 128000,
            },
        ],
        "gemini" => vec![
            ModelInfo {
                id: "gemini-2.0-flash".to_string(),
                name: "Gemini 2.0 Flash".to_string(),
                provider: "gemini".to_string(),
                context_window: 1000000,
            },
            ModelInfo {
                id: "gemini-1.5-pro".to_string(),
                name: "Gemini 1.5 Pro".to_string(),
                provider: "gemini".to_string(),
                context_window: 2000000,
            },
        ],
        "mistral" => vec![
            ModelInfo {
                id: "mistral-large-latest".to_string(),
                name: "Mistral Large".to_string(),
                provider: "mistral".to_string(),
                context_window: 128000,
            },
            ModelInfo {
                id: "mistral-medium-latest".to_string(),
                name: "Mistral Medium".to_string(),
                provider: "mistral".to_string(),
                context_window: 32000,
            },
        ],
        _ => vec![],
    };

    Ok(models)
}
