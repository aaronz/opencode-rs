use async_trait::async_trait;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use tokio::sync::mpsc;

use super::{ChatMessage, ChatResponse, Provider, StreamChunk};
use opencode_core::OpenCodeError;

pub struct OllamaProvider {
    client: Client,
    base_url: String,
    model: String,
}

#[derive(Serialize)]
struct OllamaRequest {
    model: String,
    messages: Vec<ChatMessage>,
    stream: bool,
}

#[derive(Deserialize)]
struct OllamaResponse {
    message: OllamaMessage,
}

#[derive(Deserialize)]
struct OllamaMessage {
    content: String,
}

impl OllamaProvider {
    pub fn new(model: String, base_url: Option<String>) -> Self {
        Self {
            client: Client::new(),
            base_url: base_url.unwrap_or_else(|| "http://localhost:11434".to_string()),
            model,
        }
    }
}

#[async_trait]
impl Provider for OllamaProvider {
    async fn chat(&self, messages: &[ChatMessage]) -> Result<ChatResponse, OpenCodeError> {
        let request = OllamaRequest {
            model: self.model.clone(),
            messages: messages.to_vec(),
            stream: false,
        };

        let response = self
            .client
            .post(format!("{}/api/chat", self.base_url))
            .json(&request)
            .send()
            .await
            .map_err(|e| OpenCodeError::Llm(e.to_string()))?;

        let result: OllamaResponse = response
            .json()
            .await
            .map_err(|e| OpenCodeError::Llm(e.to_string()))?;

        Ok(ChatResponse {
            content: result.message.content,
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
