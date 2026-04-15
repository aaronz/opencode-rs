use async_trait::async_trait;
use opencode_core::OpenCodeError;

use crate::auth::{AuthStrategy, ProviderAuthConfig};
use crate::provider::sealed;
use crate::provider::Provider;

/// Unified provider adapter that uses AuthStrategy for authentication
pub struct ProviderAdapter {
    client: reqwest::Client,
    config: ProviderAuthConfig,
    auth_strategy: AuthStrategy,
}

impl ProviderAdapter {
    pub fn new(config: ProviderAuthConfig, auth_strategy: AuthStrategy) -> Self {
        Self {
            client: reqwest::Client::new(),
            config,
            auth_strategy,
        }
    }

    pub fn config(&self) -> &ProviderAuthConfig {
        &self.config
    }

    pub fn auth_strategy(&self) -> &AuthStrategy {
        &self.auth_strategy
    }
}

/// OpenAI-compatible provider adapter
pub struct OpenAICompatibleAdapter {
    adapter: ProviderAdapter,
    model: String,
}

impl OpenAICompatibleAdapter {
    pub fn new(config: ProviderAuthConfig, model: String) -> Self {
        Self {
            adapter: ProviderAdapter::new(config, AuthStrategy::BearerApiKey { header_name: None }),
            model,
        }
    }

    pub fn with_auth_strategy(mut self, strategy: AuthStrategy) -> Self {
        self.adapter.auth_strategy = strategy;
        self
    }
}

impl sealed::Sealed for OpenAICompatibleAdapter {}

#[async_trait]
impl Provider for OpenAICompatibleAdapter {
    async fn complete(
        &self,
        prompt: &str,
        _context: Option<&str>,
    ) -> Result<String, OpenCodeError> {
        let request = serde_json::json!({
            "model": self.model,
            "messages": [{"role": "user", "content": prompt}],
            "max_tokens": 4096
        });

        let mut req = self
            .adapter
            .client
            .post(&self.adapter.config.endpoint)
            .header("Content-Type", "application/json")
            .json(&request);

        // Apply custom headers from config
        for (key, value) in &self.adapter.config.headers {
            req = req.header(key, value);
        }

        let response = req
            .send()
            .await
            .map_err(|e| OpenCodeError::Llm(e.to_string()))?;

        if !response.status().is_success() {
            return Err(OpenCodeError::Llm(format!(
                "API error: {}",
                response.status()
            )));
        }

        let result: serde_json::Value = response
            .json()
            .await
            .map_err(|e| OpenCodeError::Llm(e.to_string()))?;

        result["choices"][0]["message"]["content"]
            .as_str()
            .map(|s| s.to_string())
            .ok_or_else(|| OpenCodeError::Llm("No content in response".to_string()))
    }

    fn provider_name(&self) -> &str {
        "openai-compatible"
    }
}

/// Anthropic-compatible provider adapter
pub struct AnthropicAdapter {
    adapter: ProviderAdapter,
    model: String,
}

impl AnthropicAdapter {
    pub fn new(config: ProviderAuthConfig, model: String) -> Self {
        Self {
            adapter: ProviderAdapter::new(
                config,
                AuthStrategy::HeaderApiKey {
                    header_name: "x-api-key".to_string(),
                },
            ),
            model,
        }
    }
}

impl sealed::Sealed for AnthropicAdapter {}

#[async_trait]
impl Provider for AnthropicAdapter {
    async fn complete(
        &self,
        prompt: &str,
        _context: Option<&str>,
    ) -> Result<String, OpenCodeError> {
        let request = serde_json::json!({
            "model": self.model,
            "messages": [{"role": "user", "content": prompt}],
            "max_tokens": 4096
        });

        let mut req = self
            .adapter
            .client
            .post(&self.adapter.config.endpoint)
            .header("Content-Type", "application/json")
            .header("anthropic-version", "2023-06-01")
            .json(&request);

        for (key, value) in &self.adapter.config.headers {
            req = req.header(key, value);
        }

        let response = req
            .send()
            .await
            .map_err(|e| OpenCodeError::Llm(e.to_string()))?;

        if !response.status().is_success() {
            return Err(OpenCodeError::Llm(format!(
                "API error: {}",
                response.status()
            )));
        }

        let result: serde_json::Value = response
            .json()
            .await
            .map_err(|e| OpenCodeError::Llm(e.to_string()))?;

        result["content"][0]["text"]
            .as_str()
            .map(|s| s.to_string())
            .ok_or_else(|| OpenCodeError::Llm("No content in response".to_string()))
    }

    fn provider_name(&self) -> &str {
        "anthropic"
    }
}

/// Local endpoint provider adapter (no auth)
pub struct LocalEndpointAdapter {
    adapter: ProviderAdapter,
    model: String,
}

impl LocalEndpointAdapter {
    pub fn new(config: ProviderAuthConfig, model: String) -> Self {
        Self {
            adapter: ProviderAdapter::new(config, AuthStrategy::None),
            model,
        }
    }
}

impl sealed::Sealed for LocalEndpointAdapter {}

#[async_trait]
impl Provider for LocalEndpointAdapter {
    async fn complete(
        &self,
        prompt: &str,
        _context: Option<&str>,
    ) -> Result<String, OpenCodeError> {
        let request = serde_json::json!({
            "model": self.model,
            "messages": [{"role": "user", "content": prompt}],
            "max_tokens": 4096
        });

        let req = self
            .adapter
            .client
            .post(&self.adapter.config.endpoint)
            .header("Content-Type", "application/json")
            .json(&request);

        let response = req
            .send()
            .await
            .map_err(|e| OpenCodeError::Llm(e.to_string()))?;

        if !response.status().is_success() {
            return Err(OpenCodeError::Llm(format!(
                "API error: {}",
                response.status()
            )));
        }

        let result: serde_json::Value = response
            .json()
            .await
            .map_err(|e| OpenCodeError::Llm(e.to_string()))?;

        result["choices"][0]["message"]["content"]
            .as_str()
            .map(|s| s.to_string())
            .ok_or_else(|| OpenCodeError::Llm("No content in response".to_string()))
    }

    fn provider_name(&self) -> &str {
        "local"
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_openai_adapter_creation() {
        let config = ProviderAuthConfig::new(
            "openai".to_string(),
            "https://api.openai.com/v1/chat/completions".to_string(),
            AuthStrategy::BearerApiKey { header_name: None },
        );
        let adapter = OpenAICompatibleAdapter::new(config, "gpt-4".to_string());
        assert_eq!(adapter.provider_name(), "openai-compatible");
    }

    #[test]
    fn test_anthropic_adapter_creation() {
        let config = ProviderAuthConfig::new(
            "anthropic".to_string(),
            "https://api.anthropic.com/v1/messages".to_string(),
            AuthStrategy::HeaderApiKey {
                header_name: "x-api-key".to_string(),
            },
        );
        let adapter = AnthropicAdapter::new(config, "claude-3".to_string());
        assert_eq!(adapter.provider_name(), "anthropic");
    }

    #[test]
    fn test_local_adapter_creation() {
        let config = ProviderAuthConfig::new(
            "ollama".to_string(),
            "http://localhost:11434/api/chat".to_string(),
            AuthStrategy::None,
        );
        let adapter = LocalEndpointAdapter::new(config, "llama2".to_string());
        assert_eq!(adapter.provider_name(), "local");
    }
}
