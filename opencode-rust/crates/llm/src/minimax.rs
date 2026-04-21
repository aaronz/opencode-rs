use crate::provider::sealed;
use crate::provider::{Model, Provider, StreamingCallback};
use opencode_core::OpenCodeError;

pub struct MiniMaxProvider {
    api_key: String,
    model: String,
    temperature: f32,
}

impl MiniMaxProvider {
    pub fn new(api_key: String, model: String) -> Self {
        Self {
            api_key,
            model,
            temperature: 0.7,
        }
    }
}

impl sealed::Sealed for MiniMaxProvider {}

#[async_trait::async_trait]
impl Provider for MiniMaxProvider {
    async fn complete(&self, prompt: &str, context: Option<&str>) -> Result<String, OpenCodeError> {
        let client = reqwest::Client::new();
        let url = "https://api.minimax.io/v1/chat/completions";

        let messages = if let Some(ctx) = context {
            vec![
                serde_json::json!({"role": "system", "content": ctx}),
                serde_json::json!({"role": "user", "content": prompt}),
            ]
        } else {
            vec![serde_json::json!({"role": "user", "content": prompt})]
        };

        let body = serde_json::json!({
            "model": self.model,
            "messages": messages,
            "temperature": self.temperature,
        });

        let response = client
            .post(url)
            .header("Authorization", format!("Bearer {}", self.api_key))
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
            .ok_or_else(|| OpenCodeError::Llm("Invalid MiniMax response".to_string()))
    }

    async fn complete_streaming(
        &self,
        prompt: &str,
        mut callback: StreamingCallback,
    ) -> Result<(), OpenCodeError> {
        let client = reqwest::Client::new();
        let url = "https://api.minimax.io/v1/chat/completions";

        let body = serde_json::json!({
            "model": self.model,
            "messages": [
                {"role": "user", "content": prompt}
            ],
            "temperature": self.temperature,
            "stream": true,
        });

        let response = client
            .post(url)
            .header("Authorization", format!("Bearer {}", self.api_key))
            .header("Content-Type", "application/json")
            .json(&body)
            .send()
            .await
            .map_err(|e| OpenCodeError::Llm(e.to_string()))?;

        if !response.status().is_success() {
            let status = response.status();
            let error_text = response.text().await.unwrap_or_default();
            return Err(OpenCodeError::Llm(format!(
                "MiniMax API error {}: {}",
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
                Err(e) => return Err(OpenCodeError::Llm(format!("MiniMax stream error: {}", e))),
            }
        }

        Ok(())
    }

    fn get_models(&self) -> Vec<Model> {
        vec![
            Model::new("MiniMax-M2.7", "MiniMax M2.7"),
            Model::new("MiniMax-M2.7-highspeed", "MiniMax M2.7 HighSpeed"),
            Model::new("MiniMax-M2.5", "MiniMax M2.5"),
            Model::new("MiniMax-M2.5-highspeed", "MiniMax M2.5 HighSpeed"),
        ]
    }

    fn provider_name(&self) -> &str {
        "minimax"
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_minimax_provider_new() {
        let provider = MiniMaxProvider::new("test-key".to_string(), "MiniMax-M2.7".to_string());
        assert_eq!(provider.provider_name(), "minimax");
    }

    #[test]
    fn test_minimax_provider_get_models() {
        let provider = MiniMaxProvider::new("test-key".to_string(), "MiniMax-M2.7".to_string());
        let models = provider.get_models();
        assert!(!models.is_empty());
        assert!(models.iter().any(|m| m.id == "MiniMax-M2.7"));
    }
}
