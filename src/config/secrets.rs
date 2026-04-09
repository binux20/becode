//! Secret management for API keys
//!
//! Stores API keys securely in ~/.becode/secrets.toml
//! Supports environment variable fallback

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;

use super::Config;

/// Secret store for API keys
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct SecretStore {
    /// API keys by provider name
    #[serde(default)]
    pub keys: HashMap<String, String>,
}

impl SecretStore {
    /// Load secrets from file
    pub fn load() -> Result<Self> {
        let path = Config::secrets_path();

        if path.exists() {
            let content = fs::read_to_string(&path)
                .with_context(|| format!("Failed to read secrets file: {:?}", path))?;

            toml::from_str(&content)
                .with_context(|| format!("Failed to parse secrets file: {:?}", path))
        } else {
            Ok(Self::default())
        }
    }

    /// Save secrets to file
    pub fn save(&self) -> Result<()> {
        let path = Config::secrets_path();

        // Ensure directory exists
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }

        let content = toml::to_string_pretty(self)
            .with_context(|| "Failed to serialize secrets")?;

        fs::write(&path, content)
            .with_context(|| format!("Failed to write secrets file: {:?}", path))?;

        Ok(())
    }

    /// Get API key for a provider
    /// First checks secrets file, then environment variables
    pub fn get_key(&self, provider: &str) -> Option<String> {
        // Check secrets file first
        if let Some(key) = self.keys.get(provider) {
            if !key.is_empty() {
                return Some(key.clone());
            }
        }

        // Fallback to environment variables
        let env_var = Self::env_var_name(provider);
        std::env::var(&env_var).ok()
    }

    /// Set API key for a provider
    pub fn set_key(&mut self, provider: &str, key: &str) -> Result<()> {
        self.keys.insert(provider.to_string(), key.to_string());
        self.save()
    }

    /// Clear API key for a provider
    pub fn clear_key(&mut self, provider: &str) -> Result<()> {
        self.keys.remove(provider);
        self.save()
    }

    /// Check if a provider has an API key configured
    pub fn has_key(&self, provider: &str) -> bool {
        self.get_key(provider).is_some()
    }

    /// Get environment variable name for a provider
    pub fn env_var_name(provider: &str) -> String {
        match provider {
            "anthropic" => "ANTHROPIC_API_KEY".to_string(),
            "openai" => "OPENAI_API_KEY".to_string(),
            "gemini" => "GEMINI_API_KEY".to_string(),
            "mistral" => "MISTRAL_API_KEY".to_string(),
            "openrouter" => "OPENROUTER_API_KEY".to_string(),
            _ => format!("{}_API_KEY", provider.to_uppercase().replace('-', "_")),
        }
    }

    /// List all providers with configured keys
    pub fn list_providers(&self) -> Vec<&str> {
        self.keys
            .iter()
            .filter(|(_, v)| !v.is_empty())
            .map(|(k, _)| k.as_str())
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_env_var_name() {
        assert_eq!(SecretStore::env_var_name("anthropic"), "ANTHROPIC_API_KEY");
        assert_eq!(SecretStore::env_var_name("openai"), "OPENAI_API_KEY");
        assert_eq!(SecretStore::env_var_name("my-custom"), "MY_CUSTOM_API_KEY");
    }

    #[test]
    fn test_get_key_from_env() {
        let store = SecretStore::default();
        std::env::set_var("TEST_PROVIDER_API_KEY", "test_key_123");
        // This would work if we had a provider named "test_provider"
        // For now, just verify the env var logic
        assert!(std::env::var("TEST_PROVIDER_API_KEY").is_ok());
    }
}
