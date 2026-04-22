use async_trait::async_trait;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::provider::sealed;
use crate::provider::{Provider, StreamingCallback};
use crate::provider_abstraction::AnthropicThinkingConfig;
use opencode_core::OpenCodeError;

pub struct AnthropicProvider {
    client: Client,
    api_key: String,
    model: String,
    thinking_budget: Option<AnthropicThinkingConfig>,
    headers: HashMap<String, String>,
}

#[derive(Serialize)]
struct AnthropicRequest {
    model: String,
    messages: Vec<AnthropicMessage>,
    max_tokens: u32,
    stream: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    thinking: Option<AnthropicThinking>,
}

#[derive(Serialize)]
struct AnthropicThinking {
    #[serde(rename = "type")]
    thinking_type: String,
    budget_tokens: u32,
}

#[derive(Serialize)]
struct AnthropicMessage {
    role: String,
    content: String,
}

#[derive(Deserialize)]
struct AnthropicResponse {
    content: Vec<AnthropicContent>,
}

#[derive(Deserialize)]
struct AnthropicContent {
    text: String,
}

#[derive(Deserialize)]
struct StreamEvent {
    #[serde(rename = "type")]
    event_type: String,
    delta: Option<StreamDelta>,
}

#[derive(Deserialize)]
struct StreamDelta {
    text: Option<String>,
}

impl AnthropicProvider {
    pub fn new(api_key: String, model: String) -> Self {
        Self {
            client: Client::new(),
            api_key,
            model,
            thinking_budget: None,
            headers: HashMap::new(),
        }
    }

    pub fn with_headers(mut self, headers: HashMap<String, String>) -> Self {
        self.headers = headers;
        self
    }

    pub fn with_thinking_budget(mut self, config: AnthropicThinkingConfig) -> Self {
        self.thinking_budget = Some(config);
        self
    }
}

impl sealed::Sealed for AnthropicProvider {}

#[async_trait]
impl Provider for AnthropicProvider {
    async fn complete(
        &self,
        prompt: &str,
        _context: Option<&str>,
    ) -> Result<String, OpenCodeError> {
        tracing::debug!(provider = "anthropic", model = %self.model, prompt_len = prompt.len(), "Starting Anthropic completion");

        let messages = vec![AnthropicMessage {
            role: "user".to_string(),
            content: prompt.to_string(),
        }];

        let thinking = self.thinking_budget.map(|config| {
            let budget_tokens = match config {
                AnthropicThinkingConfig::Low => 1000,
                AnthropicThinkingConfig::High => 8000,
                AnthropicThinkingConfig::Max => 16000,
            };
            AnthropicThinking {
                thinking_type: "enabled".to_string(),
                budget_tokens,
            }
        });

        let request = AnthropicRequest {
            model: self.model.clone(),
            messages,
            max_tokens: 4096,
            stream: false,
            thinking,
        };

        let mut req = self
            .client
            .post("https://api.anthropic.com/v1/messages")
            .header("x-api-key", &self.api_key)
            .header("anthropic-version", "2023-06-01")
            .header("Content-Type", "application/json")
            .json(&request);

        for (key, value) in &self.headers {
            req = req.header(key, value);
        }

        let response = req.send().await.map_err(|e| {
            tracing::error!(provider = "anthropic", error = %e, "Anthropic request failed");
            OpenCodeError::Llm(e.to_string())
        })?;

        if !response.status().is_success() {
            let status = response.status();
            let error_text = response.text().await.unwrap_or_default();
            tracing::error!(provider = "anthropic", status = %status, error = %error_text, "Anthropic API error");
            return Err(OpenCodeError::Llm(format!(
                "Anthropic API error {}: {}",
                status, error_text
            )));
        }

        let result: AnthropicResponse = response
            .json()
            .await
            .map_err(|e| {
                tracing::error!(provider = "anthropic", error = %e, "Failed to parse Anthropic response");
                OpenCodeError::Llm(e.to_string())
            })?;

        let content = result
            .content
            .first()
            .map(|c| c.text.clone())
            .unwrap_or_default();

        tracing::info!(provider = "anthropic", model = %self.model, response_len = content.len(), "Anthropic completion successful");
        Ok(content)
    }

    async fn complete_streaming(
        &self,
        prompt: &str,
        mut callback: StreamingCallback,
    ) -> Result<(), OpenCodeError> {
        let messages = vec![AnthropicMessage {
            role: "user".to_string(),
            content: prompt.to_string(),
        }];

        let thinking = self.thinking_budget.map(|config| {
            let budget_tokens = match config {
                AnthropicThinkingConfig::Low => 1000,
                AnthropicThinkingConfig::High => 8000,
                AnthropicThinkingConfig::Max => 16000,
            };
            AnthropicThinking {
                thinking_type: "enabled".to_string(),
                budget_tokens,
            }
        });

        let request = AnthropicRequest {
            model: self.model.clone(),
            messages,
            max_tokens: 4096,
            stream: true,
            thinking,
        };

        let mut req = self
            .client
            .post("https://api.anthropic.com/v1/messages")
            .header("x-api-key", &self.api_key)
            .header("anthropic-version", "2023-06-01")
            .header("Content-Type", "application/json")
            .json(&request);

        for (key, value) in &self.headers {
            req = req.header(key, value);
        }

        let response = req
            .send()
            .await
            .map_err(|e| OpenCodeError::Llm(e.to_string()))?;

        if !response.status().is_success() {
            let status = response.status();
            let error_text = response.text().await.unwrap_or_default();
            return Err(OpenCodeError::Llm(format!(
                "Anthropic API error {}: {}",
                status, error_text
            )));
        }

        let mut lines = response.bytes_stream();

        use futures_util::StreamExt;
        while let Some(item) = lines.next().await {
            match item {
                Ok(bytes) => {
                    let text = String::from_utf8_lossy(&bytes);
                    for line in text.lines() {
                        if line.starts_with("data: ") {
                            let data = line.strip_prefix("data: ").unwrap_or("");
                            if let Ok(event) = serde_json::from_str::<StreamEvent>(data) {
                                if event.event_type == "content_block_delta" {
                                    if let Some(delta) = event.delta {
                                        if let Some(text) = delta.text {
                                            callback(text);
                                        }
                                    }
                                } else if event.event_type == "message_stop" {
                                    return Ok(());
                                }
                            }
                        }
                    }
                }
                Err(e) => {
                    return Err(OpenCodeError::Llm(format!("Stream error: {}", e)));
                }
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn anthropic_provider_new() {
        let provider = AnthropicProvider::new("test-key".to_string(), "claude-3".to_string());
        assert_eq!(provider.model, "claude-3");
        assert_eq!(provider.api_key, "test-key");
        assert!(provider.thinking_budget.is_none());
    }

    #[test]
    fn anthropic_provider_with_thinking_budget_low() {
        let provider = AnthropicProvider::new("test-key".to_string(), "claude-3".to_string())
            .with_thinking_budget(AnthropicThinkingConfig::Low);
        assert!(provider.thinking_budget.is_some());
    }

    #[test]
    fn anthropic_provider_with_thinking_budget_high() {
        let provider = AnthropicProvider::new("test-key".to_string(), "claude-3".to_string())
            .with_thinking_budget(AnthropicThinkingConfig::High);
        assert!(provider.thinking_budget.is_some());
    }

    #[test]
    fn anthropic_provider_with_thinking_budget_max() {
        let provider = AnthropicProvider::new("test-key".to_string(), "claude-3".to_string())
            .with_thinking_budget(AnthropicThinkingConfig::Max);
        assert!(provider.thinking_budget.is_some());
    }

    #[test]
    fn anthropic_request_serialization() {
        let request = AnthropicRequest {
            model: "claude-3".to_string(),
            messages: vec![AnthropicMessage {
                role: "user".to_string(),
                content: "Hello".to_string(),
            }],
            max_tokens: 4096,
            stream: false,
            thinking: None,
        };
        let json = serde_json::to_string(&request).unwrap();
        assert!(json.contains("claude-3"));
        assert!(json.contains("user"));
        assert!(json.contains("Hello"));
    }

    #[test]
    fn anthropic_request_serialization_with_thinking() {
        let request = AnthropicRequest {
            model: "claude-3".to_string(),
            messages: vec![AnthropicMessage {
                role: "user".to_string(),
                content: "Hello".to_string(),
            }],
            max_tokens: 4096,
            stream: false,
            thinking: Some(AnthropicThinking {
                thinking_type: "enabled".to_string(),
                budget_tokens: 1000,
            }),
        };
        let json = serde_json::to_string(&request).unwrap();
        assert!(json.contains("thinking"));
        assert!(json.contains("budget_tokens"));
    }

    #[test]
    fn anthropic_response_deserialization() {
        let json = r#"{"content":[{"text":"Hello"}]}"#;
        let response: AnthropicResponse = serde_json::from_str(json).unwrap();
        assert_eq!(response.content.len(), 1);
        assert_eq!(response.content[0].text, "Hello");
    }

    #[test]
    fn anthropic_response_deserialization_multiple_content() {
        let json = r#"{"content":[{"text":"Part1"},{"text":"Part2"}]}"#;
        let response: AnthropicResponse = serde_json::from_str(json).unwrap();
        assert_eq!(response.content.len(), 2);
    }

    #[test]
    fn stream_event_deserialization() {
        let json = r#"{"type":"content_block_delta","delta":{"text":"Hello"}}"#;
        let event: StreamEvent = serde_json::from_str(json).unwrap();
        assert_eq!(event.event_type, "content_block_delta");
        assert!(event.delta.is_some());
        assert_eq!(event.delta.unwrap().text, Some("Hello".to_string()));
    }

    #[test]
    fn stream_event_deserialization_message_stop() {
        let json = r#"{"type":"message_stop","delta":null}"#;
        let event: StreamEvent = serde_json::from_str(json).unwrap();
        assert_eq!(event.event_type, "message_stop");
        assert!(event.delta.is_none());
    }

    #[test]
    fn stream_delta_deserialization_with_text() {
        let json = r#"{"text":"Hello"}"#;
        let delta: StreamDelta = serde_json::from_str(json).unwrap();
        assert_eq!(delta.text, Some("Hello".to_string()));
    }

    #[test]
    fn stream_delta_deserialization_empty() {
        let json = r#"{"text":null}"#;
        let delta: StreamDelta = serde_json::from_str(json).unwrap();
        assert!(delta.text.is_none());
    }
}
