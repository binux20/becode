//! Configuration management for BeCode
//!
//! Handles loading, saving, and managing configuration from ~/.becode/config.toml

mod secrets;

pub use secrets::SecretStore;

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

/// Main configuration structure
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct Config {
    /// Default provider to use
    pub default_provider: String,

    /// Default model (if not specified, provider's default is used)
    pub default_model: Option<String>,

    /// Project directory (defaults to current dir)
    pub project_dir: Option<PathBuf>,

    /// Agent configuration
    pub agent: AgentConfig,

    /// UI configuration
    pub ui: UiConfig,

    /// Provider configurations
    #[serde(default)]
    pub providers: HashMap<String, ProviderConfig>,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            default_provider: "anthropic".to_string(),
            default_model: None,
            project_dir: None,
            agent: AgentConfig::default(),
            ui: UiConfig::default(),
            providers: HashMap::new(),
        }
    }
}

/// Agent-specific configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct AgentConfig {
    /// Maximum steps before stopping
    pub max_steps: u32,

    /// Enable web search tool
    pub enable_web_search: bool,

    /// Context token limit (for compaction)
    pub context_limit: u32,

    /// Auto-approve safe commands
    pub auto_approve_safe: bool,
}

impl Default for AgentConfig {
    fn default() -> Self {
        Self {
            max_steps: 25,
            enable_web_search: true,
            context_limit: 100_000,
            auto_approve_safe: true,
        }
    }
}

/// UI configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct UiConfig {
    /// Theme name
    pub theme: String,

    /// Show file tree panel
    pub show_file_tree: bool,

    /// Show token count
    pub show_token_count: bool,

    /// Enable mascot
    pub mascot_enabled: bool,

    /// Show mascot phrases
    pub mascot_phrases: bool,
}

impl Default for UiConfig {
    fn default() -> Self {
        Self {
            theme: "dark".to_string(),
            show_file_tree: true,
            show_token_count: true,
            mascot_enabled: true,
            mascot_phrases: true,
        }
    }
}

/// Provider-specific configuration
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ProviderConfig {
    /// Provider type (anthropic, openai, openai-compatible, openai-compatible-no-tools)
    #[serde(rename = "type")]
    pub provider_type: Option<String>,

    /// Base URL for API
    pub base_url: Option<String>,

    /// Model to use
    pub model: Option<String>,

    /// API key (can use ${ENV_VAR} syntax)
    pub api_key: Option<String>,

    /// Additional headers
    #[serde(default)]
    pub headers: HashMap<String, String>,

    /// Request timeout in seconds
    pub timeout_secs: Option<u64>,
}

impl Config {
    /// Get the config directory path (~/.becode)
    pub fn config_dir() -> PathBuf {
        dirs::home_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join(".becode")
    }

    /// Get the config file path (~/.becode/config.toml)
    pub fn config_path() -> PathBuf {
        Self::config_dir().join("config.toml")
    }

    /// Get the sessions directory path (~/.becode/sessions)
    pub fn sessions_dir() -> PathBuf {
        Self::config_dir().join("sessions")
    }

    /// Get the secrets file path (~/.becode/secrets.toml)
    pub fn secrets_path() -> PathBuf {
        Self::config_dir().join("secrets.toml")
    }

    /// Load configuration from file, creating default if not exists
    pub fn load() -> Result<Self> {
        let config_path = Self::config_path();

        // Ensure config directory exists
        if let Some(parent) = config_path.parent() {
            fs::create_dir_all(parent)
                .with_context(|| format!("Failed to create config directory: {:?}", parent))?;
        }

        // Ensure sessions directory exists
        fs::create_dir_all(Self::sessions_dir())
            .with_context(|| "Failed to create sessions directory")?;

        // Load or create config
        if config_path.exists() {
            let content = fs::read_to_string(&config_path)
                .with_context(|| format!("Failed to read config file: {:?}", config_path))?;

            toml::from_str(&content)
                .with_context(|| format!("Failed to parse config file: {:?}", config_path))
        } else {
            // Create default config
            let config = Config::default();
            config.save()?;
            Ok(config)
        }
    }

    /// Save configuration to file
    pub fn save(&self) -> Result<()> {
        let config_path = Self::config_path();

        // Ensure directory exists
        if let Some(parent) = config_path.parent() {
            fs::create_dir_all(parent)?;
        }

        let content = toml::to_string_pretty(self)
            .with_context(|| "Failed to serialize config")?;

        fs::write(&config_path, content)
            .with_context(|| format!("Failed to write config file: {:?}", config_path))?;

        Ok(())
    }

    /// Get provider configuration by name
    pub fn get_provider(&self, name: &str) -> Option<&ProviderConfig> {
        self.providers.get(name)
    }

    /// Resolve environment variables in a string (${VAR} syntax)
    pub fn resolve_env(value: &str) -> String {
        let mut result = value.to_string();

        // Find all ${VAR} patterns
        let re = regex::Regex::new(r"\$\{([^}]+)\}").unwrap();
        for cap in re.captures_iter(value) {
            let var_name = &cap[1];
            if let Ok(var_value) = std::env::var(var_name) {
                result = result.replace(&cap[0], &var_value);
            }
        }

        result
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = Config::default();
        assert_eq!(config.default_provider, "anthropic");
        assert_eq!(config.agent.max_steps, 25);
        assert_eq!(config.ui.theme, "dark");
    }

    #[test]
    fn test_resolve_env() {
        std::env::set_var("TEST_VAR", "test_value");
        let result = Config::resolve_env("prefix_${TEST_VAR}_suffix");
        assert_eq!(result, "prefix_test_value_suffix");
    }

    #[test]
    fn test_config_serialization() {
        let config = Config::default();
        let toml_str = toml::to_string_pretty(&config).unwrap();
        let parsed: Config = toml::from_str(&toml_str).unwrap();
        assert_eq!(config.default_provider, parsed.default_provider);
    }
}
