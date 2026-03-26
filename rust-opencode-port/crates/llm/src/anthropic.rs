use async_trait::async_trait;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use tokio::sync::mpsc;

use super::{ChatMessage, ChatResponse, Provider, StreamChunk};
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
    async fn chat(&self, messages: &[ChatMessage]) -> Result<ChatResponse, OpenCodeError> {
        let anthropic_messages: Vec<AnthropicMessage> = messages
            .iter()
            .map(|m| AnthropicMessage {
                role: m.role.clone(),
                content: m.content.clone(),
            })
            .collect();

        let request = AnthropicRequest {
            model: self.model.clone(),
            messages: anthropic_messages,
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

        let result: AnthropicResponse = response
            .json()
            .await
            .map_err(|e| OpenCodeError::Llm(e.to_string()))?;

        let content = result
            .content
            .first()
            .map(|c| c.text.clone())
            .unwrap_or_default();

        Ok(ChatResponse {
            content,
            model: self.model.clone(),
        })
    }

    async fn stream_chat(
        &self,
        _messages: &[ChatMessage],
    ) -> Result<mpsc::Receiver<Result<StreamChunk, OpenCodeError>>, OpenCodeError> {
        let (tx, rx) = mpsc::channel(100);
        tx.send(Err(OpenCodeError::Llm("Streaming not yet implemented".to_string())))
            .await
            .ok();
        Ok(rx)
    }
}
