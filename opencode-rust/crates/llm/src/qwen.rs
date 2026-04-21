use crate::provider::sealed;
use crate::provider::{Model, Provider, StreamingCallback};
use opencode_core::OpenCodeError;

pub struct QwenProvider {
    api_key: String,
    model: String,
    temperature: f32,
}

impl QwenProvider {
    pub fn new(api_key: String, model: String) -> Self {
        Self {
            api_key,
            model,
            temperature: 0.7,
        }
    }
}

impl sealed::Sealed for QwenProvider {}

#[async_trait::async_trait]
impl Provider for QwenProvider {
    async fn complete(&self, prompt: &str, context: Option<&str>) -> Result<String, OpenCodeError> {
        let client = reqwest::Client::new();
        let url = "https://dashscope.aliyuncs.com/api/v1/chat/completions";

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
            .ok_or_else(|| OpenCodeError::Llm("Invalid Qwen response".to_string()))
    }

    async fn complete_streaming(
        &self,
        prompt: &str,
        mut callback: StreamingCallback,
    ) -> Result<(), OpenCodeError> {
        let client = reqwest::Client::new();
        let url = "https://dashscope.aliyuncs.com/api/v1/chat/completions";

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
                "Qwen API error {}: {}",
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
                Err(e) => return Err(OpenCodeError::Llm(format!("Qwen stream error: {}", e))),
            }
        }

        Ok(())
    }

    fn get_models(&self) -> Vec<Model> {
        vec![
            Model::new("qwen-plus", "Qwen Plus"),
            Model::new("qwen-max", "Qwen Max"),
            Model::new("qwen-flash", "Qwen Flash"),
            Model::new("qwen3-plus", "Qwen3 Plus"),
            Model::new("qwen3-max", "Qwen3 Max"),
            Model::new("qwen3.6-plus", "Qwen3.6 Plus"),
        ]
    }

    fn provider_name(&self) -> &str {
        "qwen"
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_qwen_provider_new() {
        let provider = QwenProvider::new("test-key".to_string(), "qwen-plus".to_string());
        assert_eq!(provider.provider_name(), "qwen");
    }

    #[test]
    fn test_qwen_provider_get_models() {
        let provider = QwenProvider::new("test-key".to_string(), "qwen-plus".to_string());
        let models = provider.get_models();
        assert!(!models.is_empty());
        assert!(models.iter().any(|m| m.id == "qwen-plus"));
    }
}
