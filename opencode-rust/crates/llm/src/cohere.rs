use crate::provider::sealed;
use crate::provider::{Model, Provider, ProviderConfig, StreamingCallback};
use opencode_core::OpenCodeError;

pub struct CohereProvider {
    config: ProviderConfig,
}

impl CohereProvider {
    pub fn new(config: ProviderConfig) -> Self {
        Self { config }
    }
}

impl sealed::Sealed for CohereProvider {}

#[async_trait::async_trait]
impl Provider for CohereProvider {
    async fn complete(&self, prompt: &str, context: Option<&str>) -> Result<String, OpenCodeError> {
        let client = reqwest::Client::new();
        let url = "https://api.cohere.ai/v1/chat";

        let _messages = if let Some(ctx) = context {
            vec![
                serde_json::json!({"role": "system", "content": ctx}),
                serde_json::json!({"role": "user", "content": prompt}),
            ]
        } else {
            vec![serde_json::json!({"role": "user", "content": prompt})]
        };

        let body = serde_json::json!({
            "model": self.config.model,
            "message": prompt,
            "temperature": self.config.temperature,
        });

        let response = client
            .post(url)
            .header("Authorization", format!("Bearer {}", self.config.api_key))
            .header("Content-Type", "application/json")
            .json(&body)
            .send()
            .await
            .map_err(|e| OpenCodeError::Llm(e.to_string()))?;

        let result: serde_json::Value = response
            .json()
            .await
            .map_err(|e| OpenCodeError::Llm(e.to_string()))?;
        result["text"]
            .as_str()
            .map(|s| s.to_string())
            .ok_or_else(|| OpenCodeError::Llm("Invalid Cohere response".to_string()))
    }

    async fn complete_streaming(
        &self,
        prompt: &str,
        mut callback: StreamingCallback,
    ) -> Result<(), OpenCodeError> {
        let client = reqwest::Client::new();
        let url = "https://api.cohere.ai/v1/chat";

        let body = serde_json::json!({
            "model": self.config.model,
            "message": prompt,
            "temperature": self.config.temperature,
            "stream": true,
        });

        let response = client
            .post(url)
            .header("Authorization", format!("Bearer {}", self.config.api_key))
            .header("Content-Type", "application/json")
            .json(&body)
            .send()
            .await
            .map_err(|e| OpenCodeError::Llm(e.to_string()))?;

        if !response.status().is_success() {
            let status = response.status();
            let error_text = response.text().await.unwrap_or_default();
            return Err(OpenCodeError::Llm(format!(
                "Cohere API error {}: {}",
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
                            if chunk["event_type"] == "stream-end" {
                                callback(String::new());
                                return Ok(());
                            }
                            if let Some(content) = chunk["text"].as_str() {
                                callback(content.to_string());
                            } else if let Some(content) =
                                chunk["delta"]["message"]["content"]["text"].as_str()
                            {
                                callback(content.to_string());
                            }
                        }
                    }
                }
                Err(e) => return Err(OpenCodeError::Llm(format!("Cohere stream error: {}", e))),
            }
        }

        Ok(())
    }

    fn get_models(&self) -> Vec<Model> {
        vec![
            Model::new("command-r-plus", "Command R+"),
            Model::new("command-r", "Command R"),
            Model::new("command", "Command"),
        ]
    }

    fn provider_name(&self) -> &str {
        "cohere"
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cohere_provider_new() {
        let config = ProviderConfig {
            model: "command-r-plus".to_string(),
            api_key: "test-key".to_string(),
            temperature: 0.7,
            headers: std::collections::HashMap::new(),
        };
        let provider = CohereProvider::new(config);
        assert_eq!(provider.provider_name(), "cohere");
    }

    #[test]
    fn test_cohere_provider_get_models() {
        let config = ProviderConfig::default();
        let provider = CohereProvider::new(config);
        let models = provider.get_models();
        assert!(!models.is_empty());
        assert!(models.iter().any(|m| m.id == "command-r-plus"));
    }

    #[tokio::test]
    async fn test_cohere_complete_returns_error_without_api_key() {
        let config = ProviderConfig {
            model: "command-r-plus".to_string(),
            api_key: "invalid-key".to_string(),
            temperature: 0.7,
            headers: std::collections::HashMap::new(),
        };
        let provider = CohereProvider::new(config);
        let result = provider.complete("test prompt", None).await;
        assert!(result.is_err());
    }
}
