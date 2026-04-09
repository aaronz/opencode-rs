use async_trait::async_trait;
use reqwest::Client;
use serde::{Deserialize, Serialize};

use crate::provider::{Provider, StreamingCallback};
use opencode_core::OpenCodeError;

pub struct AnthropicProvider {
    client: Client,
    api_key: String,
    model: String,
}

#[derive(Serialize)]
struct AnthropicRequest {
    model: String,
    messages: Vec<AnthropicMessage>,
    max_tokens: u32,
    stream: bool,
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
        }
    }
}

#[async_trait]
impl Provider for AnthropicProvider {
    async fn complete(&self, prompt: &str, _context: Option<&str>) -> Result<String, OpenCodeError> {
        let messages = vec![AnthropicMessage {
            role: "user".to_string(),
            content: prompt.to_string(),
        }];

        let request = AnthropicRequest {
            model: self.model.clone(),
            messages,
            max_tokens: 4096,
            stream: false,
        };

        let response = self
            .client
            .post("https://api.anthropic.com/v1/messages")
            .header("x-api-key", &self.api_key)
            .header("anthropic-version", "2023-06-01")
            .header("Content-Type", "application/json")
            .json(&request)
            .send()
            .await
            .map_err(|e| OpenCodeError::Llm(e.to_string()))?;

        if !response.status().is_success() {
            let status = response.status();
            let error_text = response.text().await.unwrap_or_default();
            return Err(OpenCodeError::Llm(format!("Anthropic API error {}: {}", status, error_text)));
        }

        let result: AnthropicResponse = response
            .json()
            .await
            .map_err(|e| OpenCodeError::Llm(e.to_string()))?;

        let content = result
            .content
            .first()
            .map(|c| c.text.clone())
            .unwrap_or_default();

        Ok(content)
    }

    async fn complete_streaming(&self, prompt: &str, mut callback: StreamingCallback) -> Result<(), OpenCodeError> {
        let messages = vec![AnthropicMessage {
            role: "user".to_string(),
            content: prompt.to_string(),
        }];

        let request = AnthropicRequest {
            model: self.model.clone(),
            messages,
            max_tokens: 4096,
            stream: true,
        };

        let response = self
            .client
            .post("https://api.anthropic.com/v1/messages")
            .header("x-api-key", &self.api_key)
            .header("anthropic-version", "2023-06-01")
            .header("Content-Type", "application/json")
            .json(&request)
            .send()
            .await
            .map_err(|e| OpenCodeError::Llm(e.to_string()))?;

        if !response.status().is_success() {
            let status = response.status();
            let error_text = response.text().await.unwrap_or_default();
            return Err(OpenCodeError::Llm(format!("Anthropic API error {}: {}", status, error_text)));
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
