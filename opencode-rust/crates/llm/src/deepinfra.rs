use crate::provider::sealed;
use crate::provider::{Model, Provider, ProviderConfig, StreamingCallback};
use opencode_core::OpenCodeError;

pub struct DeepInfraProvider {
    config: ProviderConfig,
}

impl DeepInfraProvider {
    pub fn new(config: ProviderConfig) -> Self {
        Self { config }
    }
}

impl sealed::Sealed for DeepInfraProvider {}

#[async_trait::async_trait]
impl Provider for DeepInfraProvider {
    async fn complete(&self, prompt: &str, context: Option<&str>) -> Result<String, OpenCodeError> {
        let client = reqwest::Client::new();
        let url = "https://api.deepinfra.com/v1/inference";

        let messages = if let Some(ctx) = context {
            vec![
                serde_json::json!({"role": "system", "content": ctx}),
                serde_json::json!({"role": "user", "content": prompt}),
            ]
        } else {
            vec![serde_json::json!({"role": "user", "content": prompt})]
        };

        let body = serde_json::json!({
            "model": self.config.model,
            "messages": messages,
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
        result["choices"][0]["message"]["content"]
            .as_str()
            .map(|s| s.to_string())
            .ok_or_else(|| OpenCodeError::Llm("Invalid DeepInfra response".to_string()))
    }

    async fn complete_streaming(
        &self,
        prompt: &str,
        mut callback: StreamingCallback,
    ) -> Result<(), OpenCodeError> {
        let client = reqwest::Client::new();
        let url = "https://api.deepinfra.com/v1/openai/chat/completions";

        let body = serde_json::json!({
            "model": self.config.model,
            "messages": [
                {"role": "user", "content": prompt}
            ],
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
                "DeepInfra API error {}: {}",
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
                Err(e) => return Err(OpenCodeError::Llm(format!("DeepInfra stream error: {}", e))),
            }
        }

        Ok(())
    }

    fn get_models(&self) -> Vec<Model> {
        vec![
            Model::new("meta-llama/Llama-3.1-70B-Instruct", "Llama 3.1 70B"),
            Model::new("meta-llama/Llama-3.1-405B-Instruct", "Llama 3.1 405B"),
            Model::new("mistralai/Mistral-7B-Instruct-v0.2", "Mistral 7B"),
            Model::new("Qwen/Qwen2-72B-Instruct", "Qwen 2 72B"),
        ]
    }

    fn provider_name(&self) -> &str {
        "deepinfra"
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_deepinfra_provider_new() {
        let config = ProviderConfig {
            model: "meta-llama/Llama-3.1-70B-Instruct".to_string(),
            api_key: "test-key".to_string(),
            temperature: 0.7,
            headers: std::collections::HashMap::new(),
        };
        let provider = DeepInfraProvider::new(config);
        assert_eq!(provider.provider_name(), "deepinfra");
    }

    #[test]
    fn test_deepinfra_provider_get_models() {
        let config = ProviderConfig::default();
        let provider = DeepInfraProvider::new(config);
        let models = provider.get_models();
        assert!(!models.is_empty());
        assert!(models
            .iter()
            .any(|m| m.id == "meta-llama/Llama-3.1-70B-Instruct"));
    }

    #[tokio::test]
    async fn test_deepinfra_complete_returns_error_without_api_key() {
        let config = ProviderConfig {
            model: "meta-llama/Llama-3.1-70B-Instruct".to_string(),
            api_key: "invalid-key".to_string(),
            temperature: 0.7,
            headers: std::collections::HashMap::new(),
        };
        let provider = DeepInfraProvider::new(config);
        let result = provider.complete("test prompt", None).await;
        assert!(result.is_err());
    }
}
