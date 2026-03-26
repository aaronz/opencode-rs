use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use opencode_core::{Message, OpenCodeError};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatMessage {
    pub role: String,
    pub content: String,
}

impl From<&Message> for ChatMessage {
    fn from(msg: &Message) -> Self {
        Self {
            role: match msg.role {
                opencode_core::Role::System => "system".to_string(),
                opencode_core::Role::User => "user".to_string(),
                opencode_core::Role::Assistant => "assistant".to_string(),
            },
            content: msg.content.clone(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct ChatResponse {
    pub content: String,
    pub model: String,
}

#[derive(Debug, Clone)]
pub struct StreamChunk {
    pub content: String,
    pub done: bool,
}

#[async_trait]
pub trait Provider: Send + Sync {
    async fn chat(&self, messages: &[ChatMessage]) -> Result<ChatResponse, OpenCodeError>;
    async fn stream_chat(
        &self,
        messages: &[ChatMessage],
    ) -> Result<tokio::sync::mpsc::Receiver<Result<StreamChunk, OpenCodeError>>, OpenCodeError>;
}
