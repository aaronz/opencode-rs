use crate::provider::sealed;
use crate::provider::{Model, Provider, StreamingCallback};
use opencode_core::OpenCodeError;

/// Cloudflare AI Gateway provider for routing requests through Cloudflare's AI Gateway.
/// Base URL format: https://gateway.ai.cloudflare.com/v1/{account_id}/openai
pub struct AiGatewayProvider {
    account_id: String,
    api_key: String,
    model: String,
    client: reqwest::Client,
    base_url: String,
}

impl AiGatewayProvider {
    /// Create a new AI Gateway provider with the specified account ID and API key.
    pub fn new(account_id: String, api_key: String, model: String) -> Self {
        let base_url = format!("https://gateway.ai.cloudflare.com/v1/{}/openai", account_id);
        Self {
            account_id,
            api_key,
            model,
            client: reqwest::Client::new(),
            base_url,
        }
    }

    /// Returns the base URL for the AI Gateway.
    pub fn base_url(&self) -> &str {
        &self.base_url
    }

    /// Returns the account ID.
    pub fn account_id(&self) -> &str {
        &self.account_id
    }

    /// Returns the model being used.
    pub fn model(&self) -> &str {
        &self.model
    }
}

impl sealed::Sealed for AiGatewayProvider {}

#[async_trait::async_trait]
impl Provider for AiGatewayProvider {
    async fn complete(
        &self,
        prompt: &str,
        _context: Option<&str>,
    ) -> Result<String, OpenCodeError> {
        let messages = vec![serde_json::json!({
            "role": "user",
            "content": prompt
        })];

        let request_body = serde_json::json!({
            "model": self.model,
            "messages": messages,
            "stream": false,
        });

        let response = self
            .client
            .post(format!("{}/chat/completions", self.base_url))
            .header("Authorization", format!("Bearer {}", self.api_key))
            .header("Content-Type", "application/json")
            .json(&request_body)
            .send()
            .await
            .map_err(|e| OpenCodeError::Llm(e.to_string()))?;

        if !response.status().is_success() {
            let status = response.status();
            let error_text = response.text().await.unwrap_or_default();
            return Err(OpenCodeError::Llm(format!(
                "AI Gateway error {}: {}",
                status, error_text
            )));
        }

        let result: serde_json::Value = response
            .json()
            .await
            .map_err(|e| OpenCodeError::Llm(e.to_string()))?;

        result["choices"][0]["message"]["content"]
            .as_str()
            .map(|s| s.to_string())
            .ok_or_else(|| OpenCodeError::Llm("Invalid AI Gateway response format".to_string()))
    }

    async fn complete_streaming(
        &self,
        prompt: &str,
        mut callback: StreamingCallback,
    ) -> Result<(), OpenCodeError> {
        let messages = vec![serde_json::json!({
            "role": "user",
            "content": prompt
        })];

        let request_body = serde_json::json!({
            "model": self.model,
            "messages": messages,
            "stream": true,
        });

        let response = self
            .client
            .post(format!("{}/chat/completions", self.base_url))
            .header("Authorization", format!("Bearer {}", self.api_key))
            .header("Content-Type", "application/json")
            .json(&request_body)
            .send()
            .await
            .map_err(|e| OpenCodeError::Llm(e.to_string()))?;

        if !response.status().is_success() {
            let status = response.status();
            let error_text = response.text().await.unwrap_or_default();
            return Err(OpenCodeError::Llm(format!(
                "AI Gateway streaming error {}: {}",
                status, error_text
            )));
        }

        use futures_util::StreamExt;
        let mut lines = response.bytes_stream();
        while let Some(item) = lines.next().await {
            match item {
                Ok(bytes) => {
                    let text = String::from_utf8_lossy(&bytes);
                    for line in text.lines() {
                        if !line.starts_with("data: ") {
                            continue;
                        }
                        let data = line.strip_prefix("data: ").unwrap_or("");
                        if data == "[DONE]" {
                            callback(String::new());
                            return Ok(());
                        }
                        if let Ok(chunk) = serde_json::from_str::<serde_json::Value>(data) {
                            if let Some(content) = chunk["choices"][0]["delta"]["content"].as_str()
                            {
                                callback(content.to_string());
                            }
                        }
                    }
                }
                Err(e) => {
                    return Err(OpenCodeError::Llm(format!(
                        "AI Gateway stream error: {}",
                        e
                    )));
                }
            }
        }

        Ok(())
    }

    fn get_models(&self) -> Vec<Model> {
        vec![
            Model::new("gpt-4o", "GPT-4o"),
            Model::new("gpt-4o-mini", "GPT-4o Mini"),
            Model::new("claude-3.5-sonnet", "Claude 3.5 Sonnet"),
            Model::new("claude-3-haiku", "Claude 3 Haiku"),
        ]
    }

    fn provider_name(&self) -> &str {
        "ai-gateway"
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ai_gateway_provider_new() {
        let provider = AiGatewayProvider::new(
            "test-account".to_string(),
            "test-key".to_string(),
            "gpt-4o".to_string(),
        );

        assert_eq!(provider.account_id(), "test-account");
        assert_eq!(provider.model(), "gpt-4o");
        assert_eq!(
            provider.base_url(),
            "https://gateway.ai.cloudflare.com/v1/test-account/openai"
        );
    }

    #[test]
    fn test_ai_gateway_provider_name() {
        let provider = AiGatewayProvider::new(
            "account".to_string(),
            "key".to_string(),
            "model".to_string(),
        );
        assert_eq!(provider.provider_name(), "ai-gateway");
    }

    #[test]
    fn test_ai_gateway_provider_get_models() {
        let provider = AiGatewayProvider::new(
            "account".to_string(),
            "key".to_string(),
            "gpt-4o".to_string(),
        );
        let models = provider.get_models();
        assert!(!models.is_empty());
        assert!(models.iter().any(|m| m.id == "gpt-4o"));
    }

    #[test]
    fn test_ai_gateway_provider_base_url_format() {
        let account_id = "my-account-123";
        let provider = AiGatewayProvider::new(
            account_id.to_string(),
            "key".to_string(),
            "model".to_string(),
        );

        let expected_url = format!("https://gateway.ai.cloudflare.com/v1/{}/openai", account_id);
        assert_eq!(provider.base_url(), expected_url);
    }

    #[tokio::test]
    async fn test_ai_gateway_complete_returns_error_without_valid_key() {
        let provider = AiGatewayProvider::new(
            "test-account".to_string(),
            "invalid-key".to_string(),
            "gpt-4o".to_string(),
        );
        let result = provider.complete("test prompt", None).await;
        // Should fail due to invalid credentials
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_ai_gateway_streaming_returns_error_without_valid_key() {
        let provider = AiGatewayProvider::new(
            "test-account".to_string(),
            "invalid-key".to_string(),
            "gpt-4o".to_string(),
        );
        let result = provider.complete_streaming("test", Box::new(|_| {})).await;
        // Should fail due to invalid credentials
        assert!(result.is_err());
    }
}
