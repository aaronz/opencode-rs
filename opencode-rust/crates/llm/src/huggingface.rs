use async_trait::async_trait;
use futures_util::StreamExt;
use reqwest::Client;
use serde_json::Value;

use crate::provider::{Model, Provider, ProviderConfig, StreamingCallback};
use opencode_core::OpenCodeError;

const HUGGINGFACE_INFERENCE_URL: &str = "https://api-inference.huggingface.co/models";

pub struct HuggingFaceProvider {
    client: Client,
    config: ProviderConfig,
}

impl HuggingFaceProvider {
    pub fn new(config: ProviderConfig) -> Self {
        Self {
            client: Client::new(),
            config,
        }
    }

    fn endpoint(&self) -> String {
        format!("{}/{}", HUGGINGFACE_INFERENCE_URL, self.config.model)
    }

    fn extract_completion_text(payload: &Value) -> Option<String> {
        if let Some(text) = payload
            .as_array()
            .and_then(|arr| arr.first())
            .and_then(|item| item.get("generated_text"))
            .and_then(Value::as_str)
        {
            return Some(text.to_string());
        }

        if let Some(text) = payload.get("generated_text").and_then(Value::as_str) {
            return Some(text.to_string());
        }

        payload
            .get("choices")
            .and_then(Value::as_array)
            .and_then(|choices| choices.first())
            .and_then(|choice| {
                choice
                    .get("text")
                    .and_then(Value::as_str)
                    .or_else(|| {
                        choice
                            .get("message")
                            .and_then(|message| message.get("content"))
                            .and_then(Value::as_str)
                    })
            })
            .map(ToString::to_string)
    }

    fn extract_stream_text(payload: &Value) -> Option<String> {
        payload
            .get("token")
            .and_then(|token| token.get("text"))
            .and_then(Value::as_str)
            .map(ToString::to_string)
            .or_else(|| {
                payload
                    .get("generated_text")
                    .and_then(Value::as_str)
                    .map(ToString::to_string)
            })
            .or_else(|| {
                payload
                    .get("choices")
                    .and_then(Value::as_array)
                    .and_then(|choices| choices.first())
                    .and_then(|choice| {
                        choice
                            .get("delta")
                            .and_then(|delta| delta.get("content"))
                            .and_then(Value::as_str)
                            .or_else(|| choice.get("text").and_then(Value::as_str))
                    })
                    .map(ToString::to_string)
            })
    }
}

#[async_trait]
impl Provider for HuggingFaceProvider {
    async fn complete(&self, prompt: &str, context: Option<&str>) -> Result<String, OpenCodeError> {
        let input = if let Some(ctx) = context {
            format!("{}\n\n{}", ctx, prompt)
        } else {
            prompt.to_string()
        };

        let body = serde_json::json!({
            "inputs": input,
            "parameters": {
                "temperature": self.config.temperature,
            }
        });

        let response = self
            .client
            .post(self.endpoint())
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
                "HuggingFace API error {}: {}",
                status, error_text
            )));
        }

        let payload: Value = response
            .json()
            .await
            .map_err(|e| OpenCodeError::Llm(e.to_string()))?;

        Self::extract_completion_text(&payload)
            .ok_or_else(|| OpenCodeError::Llm("Invalid HuggingFace response".to_string()))
    }

    async fn complete_streaming(
        &self,
        prompt: &str,
        mut callback: StreamingCallback,
    ) -> Result<(), OpenCodeError> {
        let body = serde_json::json!({
            "inputs": prompt,
            "stream": true,
            "parameters": {
                "temperature": self.config.temperature,
            }
        });

        let response = self
            .client
            .post(self.endpoint())
            .header("Authorization", format!("Bearer {}", self.config.api_key))
            .header("Accept", "text/event-stream")
            .header("Content-Type", "application/json")
            .json(&body)
            .send()
            .await
            .map_err(|e| OpenCodeError::Llm(e.to_string()))?;

        if !response.status().is_success() {
            let status = response.status();
            let error_text = response.text().await.unwrap_or_default();
            return Err(OpenCodeError::Llm(format!(
                "HuggingFace API error {}: {}",
                status, error_text
            )));
        }

        let mut lines = response.bytes_stream();
        while let Some(item) = lines.next().await {
            let bytes = item.map_err(|e| OpenCodeError::Llm(format!("Stream error: {}", e)))?;
            let text = String::from_utf8_lossy(&bytes);

            for line in text.lines() {
                if !line.starts_with("data: ") {
                    continue;
                }

                let data = line.strip_prefix("data: ").unwrap_or("").trim();
                if data == "[DONE]" {
                    callback(String::new());
                    return Ok(());
                }

                if let Ok(payload) = serde_json::from_str::<Value>(data) {
                    if let Some(chunk) = Self::extract_stream_text(&payload) {
                        callback(chunk);
                    }
                }
            }
        }

        Ok(())
    }

    fn get_models(&self) -> Vec<Model> {
        vec![
            Model::new("meta-llama/Llama-3.1-8B-Instruct", "Llama 3.1 8B Instruct"),
            Model::new("mistralai/Mistral-7B-Instruct-v0.3", "Mistral 7B Instruct"),
            Model::new("google/gemma-2-9b-it", "Gemma 2 9B IT"),
        ]
    }

    fn provider_name(&self) -> &str {
        "huggingface"
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn provider() -> HuggingFaceProvider {
        HuggingFaceProvider::new(ProviderConfig {
            model: "meta-llama/Llama-3.1-8B-Instruct".to_string(),
            api_key: "test-key".to_string(),
            temperature: 0.2,
        })
    }

    #[test]
    fn provider_metadata_is_exposed() {
        let provider = provider();
        assert_eq!(provider.provider_name(), "huggingface");
        assert_eq!(provider.get_models().len(), 3);
        assert_eq!(provider.endpoint(), "https://api-inference.huggingface.co/models/meta-llama/Llama-3.1-8B-Instruct");
    }

    #[test]
    fn extract_completion_text_supports_inference_shapes() {
        let array_payload = serde_json::json!([
            {
                "generated_text": "hello world"
            }
        ]);
        assert_eq!(
            HuggingFaceProvider::extract_completion_text(&array_payload),
            Some("hello world".to_string())
        );

        let choices_payload = serde_json::json!({
            "choices": [
                {
                    "message": {"content": "chat shape"}
                }
            ]
        });
        assert_eq!(
            HuggingFaceProvider::extract_completion_text(&choices_payload),
            Some("chat shape".to_string())
        );
    }

    #[test]
    fn extract_stream_text_supports_token_and_delta_shapes() {
        let token_payload = serde_json::json!({
            "token": {"text": "abc"}
        });
        assert_eq!(
            HuggingFaceProvider::extract_stream_text(&token_payload),
            Some("abc".to_string())
        );

        let delta_payload = serde_json::json!({
            "choices": [{
                "delta": {"content": "chunk"}
            }]
        });
        assert_eq!(
            HuggingFaceProvider::extract_stream_text(&delta_payload),
            Some("chunk".to_string())
        );
    }
}
