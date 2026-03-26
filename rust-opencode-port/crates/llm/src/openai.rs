use async_trait::async_trait;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use tokio::sync::mpsc;

use super::{ChatMessage, ChatResponse, Provider, StreamChunk};
use opencode_core::OpenCodeError;

pub struct OpenAiProvider {
    client: Client,
    api_key: String,
    base_url: String,
    model: String,
}

#[derive(Serialize)]
struct ChatRequest {
    model: String,
    messages: Vec<ChatMessage>,
    stream: bool,
}

#[derive(Deserialize)]
struct ChatCompletion {
    choices: Vec<Choice>,
}

#[derive(Deserialize)]
struct Choice {
    message: MessageContent,
}

#[derive(Deserialize)]
struct MessageContent {
    content: String,
}

impl OpenAiProvider {
    pub fn new(api_key: String, model: String) -> Self {
        Self {
            client: Client::new(),
            api_key,
            base_url: "https://api.openai.com/v1".to_string(),
            model,
        }
    }
}

#[async_trait]
impl Provider for OpenAiProvider {
    async fn chat(&self, messages: &[ChatMessage]) -> Result<ChatResponse, OpenCodeError> {
        let request = ChatRequest {
            model: self.model.clone(),
            messages: messages.to_vec(),
            stream: false,
        };

        let response = self
            .client
            .post(format!("{}/chat/completions", self.base_url))
            .header("Authorization", format!("Bearer {}", self.api_key))
            .header("Content-Type", "application/json")
            .json(&request)
            .send()
            .await
            .map_err(|e| OpenCodeError::Llm(e.to_string()))?;

        let completion: ChatCompletion = response
            .json()
            .await
            .map_err(|e| OpenCodeError::Llm(e.to_string()))?;

        let content = completion
            .choices
            .first()
            .map(|c| c.message.content.clone())
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
        
        tx.send(Err(OpenCodeError::Llm("Streaming not implemented".to_string())))
            .await
            .ok();
            
        Ok(rx)
    }
}
