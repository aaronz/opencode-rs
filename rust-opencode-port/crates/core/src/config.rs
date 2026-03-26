use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Config {
    pub provider: ProviderConfig,
    pub model: Option<String>,
    pub api_key: Option<String>,
    pub temperature: Option<f32>,
    pub max_tokens: Option<u32>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum ProviderConfig {
    Openai,
    Anthropic,
    Ollama,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            provider: ProviderConfig::Openai,
            model: Some("gpt-4o".to_string()),
            api_key: None,
            temperature: Some(0.7),
            max_tokens: None,
        }
    }
}

impl Config {
    pub fn load(path: &PathBuf) -> Result<Self, crate::OpenCodeError> {
        let mut config = if !path.exists() {
            Config::default()
        } else {
            let content = std::fs::read_to_string(path)?;
            toml::from_str(&content).map_err(|e| crate::OpenCodeError::Config(e.to_string()))?
        };

        config.apply_env_overrides();
        Ok(config)
    }

    pub fn config_path() -> PathBuf {
        directories::ProjectDirs::from("com", "opencode", "rs")
            .map(|dirs| dirs.config_dir().join("config.toml"))
            .unwrap_or_else(|| PathBuf::from("~/.config/opencode-rs/config.toml"))
    }

    fn apply_env_overrides(&mut self) {
        if let Ok(provider) = std::env::var("OPENCODE_PROVIDER") {
            self.provider = match provider.to_lowercase().as_str() {
                "openai" => ProviderConfig::Openai,
                "anthropic" => ProviderConfig::Anthropic,
                "ollama" => ProviderConfig::Ollama,
                _ => self.provider.clone(),
            };
        }

        if let Ok(model) = std::env::var("OPENCODE_MODEL") {
            self.model = Some(model);
        }

        if let Ok(api_key) = std::env::var("OPENCODE_API_KEY") {
            self.api_key = Some(api_key);
        }

        if let Ok(temp) = std::env::var("OPENCODE_TEMPERATURE") {
            if let Ok(t) = temp.parse() {
                self.temperature = Some(t);
            }
        }

        if let Ok(tokens) = std::env::var("OPENCODE_MAX_TOKENS") {
            if let Ok(t) = tokens.parse() {
                self.max_tokens = Some(t);
            }
        }
    }
}
