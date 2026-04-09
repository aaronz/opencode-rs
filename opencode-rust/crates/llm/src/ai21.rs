use async_trait::async_trait;
use futures_util::StreamExt;
use reqwest::Client;
use serde_json::Value;

use crate::provider::{Model, Provider, ProviderConfig, StreamingCallback};
use opencode_core::OpenCodeError;

const AI21_API_BASE: &str = "https://api.ai21.com/studio/v1";

pub struct Ai21Provider {
    client: Client,
    config: ProviderConfig,
}

impl Ai21Provider {
    pub fn new(config: ProviderConfig) -> Self {
        Self {
            client: Client::new(),
            config,
        }
    }

    fn endpoint(&self) -> String {
        format!("{}/{}/complete", AI21_API_BASE, self.config.model)
    }

    fn extract_completion_text(payload: &Value) -> Option<String> {
        payload
            .get("completions")
            .and_then(Value::as_array)
            .and_then(|completions| completions.first())
            .and_then(|completion| completion.get("data"))
            .and_then(|data| data.get("text"))
            .and_then(Value::as_str)
            .map(ToString::to_string)
            .or_else(|| {
                payload
                    .get("choices")
                    .and_then(Value::as_array)
                    .and_then(|choices| choices.first())
                    .and_then(|choice| choice.get("text"))
                    .and_then(Value::as_str)
                    .map(ToString::to_string)
            })
    }

    fn extract_stream_text(payload: &Value) -> Option<String> {
        payload
            .get("data")
            .and_then(|data| data.get("text"))
            .and_then(Value::as_str)
            .map(ToString::to_string)
            .or_else(|| {
                payload
                    .get("token")
                    .and_then(|token| token.get("text"))
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
                    })
                    .map(ToString::to_string)
            })
    }
}

#[async_trait]
impl Provider for Ai21Provider {
    async fn complete(&self, prompt: &str, context: Option<&str>) -> Result<String, OpenCodeError> {
        let prompt = if let Some(ctx) = context {
            format!("{}\n\n{}", ctx, prompt)
        } else {
            prompt.to_string()
        };

        let body = serde_json::json!({
            "prompt": prompt,
            "temperature": self.config.temperature,
            "numResults": 1,
            "maxTokens": 512,
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
                "AI21 API error {}: {}",
                status, error_text
            )));
        }

        let payload: Value = response
            .json()
            .await
            .map_err(|e| OpenCodeError::Llm(e.to_string()))?;

        Self::extract_completion_text(&payload)
            .ok_or_else(|| OpenCodeError::Llm("Invalid AI21 response".to_string()))
    }

    async fn complete_streaming(
        &self,
        prompt: &str,
        mut callback: StreamingCallback,
    ) -> Result<(), OpenCodeError> {
        let body = serde_json::json!({
            "prompt": prompt,
            "temperature": self.config.temperature,
            "stream": true,
            "numResults": 1,
            "maxTokens": 512,
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
                "AI21 API error {}: {}",
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
            Model::new("j2-ultra", "Jurassic-2 Ultra"),
            Model::new("j2-mid", "Jurassic-2 Mid"),
            Model::new("j2-light", "Jurassic-2 Light"),
        ]
    }

    fn provider_name(&self) -> &str {
        "ai21"
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn provider() -> Ai21Provider {
        Ai21Provider::new(ProviderConfig {
            model: "j2-ultra".to_string(),
            api_key: "test-key".to_string(),
            temperature: 0.5,
        })
    }

    #[test]
    fn provider_metadata_is_exposed() {
        let provider = provider();
        assert_eq!(provider.provider_name(), "ai21");
        assert_eq!(provider.get_models().len(), 3);
        assert_eq!(provider.endpoint(), "https://api.ai21.com/studio/v1/j2-ultra/complete");
    }

    #[test]
    fn extract_completion_text_handles_jurassic_shape() {
        let payload = serde_json::json!({
            "completions": [
                { "data": { "text": "hello jurassic" } }
            ]
        });

        assert_eq!(
            Ai21Provider::extract_completion_text(&payload),
            Some("hello jurassic".to_string())
        );
    }

    #[test]
    fn extract_stream_text_handles_multiple_shapes() {
        let payload = serde_json::json!({"data": {"text": "stream-chunk"}});
        assert_eq!(
            Ai21Provider::extract_stream_text(&payload),
            Some("stream-chunk".to_string())
        );

        let fallback = serde_json::json!({
            "choices": [{ "delta": { "content": "fallback" }}]
        });
        assert_eq!(
            Ai21Provider::extract_stream_text(&fallback),
            Some("fallback".to_string())
        );
    }
}
