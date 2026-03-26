use opencode_core::OpenCodeError;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiKey {
    pub provider: String,
    pub key: String,
    pub expires_at: Option<chrono::DateTime<chrono::Utc>>,
}

impl ApiKey {
    pub fn new(provider: String, key: String) -> Self {
        Self {
            provider,
            key,
            expires_at: None,
        }
    }

    pub fn is_valid(&self) -> bool {
        if let Some(expires_at) = self.expires_at {
            chrono::Utc::now() < expires_at
        } else {
            !self.key.is_empty()
        }
    }
}

pub struct AuthManager {
    keys: std::collections::HashMap<String, ApiKey>,
}

impl AuthManager {
    pub fn new() -> Self {
        Self {
            keys: std::collections::HashMap::new(),
        }
    }

    pub fn set_key(&mut self, provider: String, key: String) {
        self.keys
            .insert(provider.clone(), ApiKey::new(provider, key));
    }

    pub fn get_key(&self, provider: &str) -> Option<&ApiKey> {
        self.keys.get(provider)
    }

    pub fn validate_key(&self, provider: &str) -> Result<String, OpenCodeError> {
        let key = self.keys.get(provider).ok_or_else(|| {
            OpenCodeError::Llm(format!("No API key found for provider: {}", provider))
        })?;

        if !key.is_valid() {
            return Err(OpenCodeError::Llm(format!(
                "API key for provider {} is invalid or expired",
                provider
            )));
        }

        Ok(key.key.clone())
    }

    pub fn load_from_env(&mut self) {
        let providers = ["OPENAI_API_KEY", "ANTHROPIC_API_KEY", "OLLAMA_API_KEY"];

        for env_var in providers {
            if let Ok(key) = std::env::var(env_var) {
                let provider = env_var.replace("_API_KEY", "").to_lowercase();
                self.set_key(provider, key);
            }
        }
    }
}

impl Default for AuthManager {
    fn default() -> Self {
        Self::new()
    }
}
