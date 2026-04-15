use async_trait::async_trait;
use reqwest::Client;
use serde::{Deserialize, Serialize};

use crate::provider::{ChatMessage, ChatResponse, Provider, StreamingCallback};
use crate::provider::sealed;
use opencode_core::OpenCodeError;

pub struct LmStudioProvider {
    client: Client,
    base_url: String,
    model: String,
}

#[derive(Serialize)]
struct LmStudioChatRequest {
    model: String,
    messages: Vec<LmStudioChatMessage>,
    stream: bool,
    temperature: Option<f32>,
}

#[derive(Serialize)]
struct LmStudioChatMessage {
    role: String,
    content: String,
}

#[derive(Deserialize)]
struct LmStudioChatResponse {
    choices: Vec<LmStudioChoice>,
    model: Option<String>,
}

#[derive(Deserialize)]
struct LmStudioChoice {
    message: LmStudioMessage,
}

#[derive(Deserialize)]
struct LmStudioMessage {
    content: String,
    #[allow(dead_code)]
    role: Option<String>,
}

#[derive(Deserialize)]
struct LmStudioStreamChunk {
    choices: Option<Vec<LmStudioStreamChoice>>,
}

#[derive(Deserialize)]
struct LmStudioStreamChoice {
    delta: Option<LmStudioDelta>,
    #[allow(dead_code)]
    finish_reason: Option<String>,
}

#[derive(Deserialize)]
struct LmStudioDelta {
    content: Option<String>,
}

impl LmStudioProvider {
    pub fn new(model: String, base_url: Option<String>) -> Self {
        Self {
            client: Client::new(),
            base_url: base_url.unwrap_or_else(|| "http://localhost:1234".to_string()),
            model,
        }
    }

    fn chat_url(&self) -> String {
        format!("{}/v1/chat/completions", self.base_url)
    }

    #[allow(dead_code)]
    fn models_url(&self) -> String {
        format!("{}/v1/models", self.base_url)
    }
}

impl sealed::Sealed for LmStudioProvider {}

#[async_trait]
impl Provider for LmStudioProvider {
    async fn complete(
        &self,
        prompt: &str,
        _context: Option<&str>,
    ) -> Result<String, OpenCodeError> {
        let messages = vec![ChatMessage {
            role: "user".to_string(),
            content: prompt.to_string(),
        }];
        let response = self.chat(&messages).await?;
        Ok(response.content)
    }

    async fn chat(&self, messages: &[ChatMessage]) -> Result<ChatResponse, OpenCodeError> {
        let request = LmStudioChatRequest {
            model: self.model.clone(),
            messages: messages
                .iter()
                .map(|m| LmStudioChatMessage {
                    role: m.role.clone(),
                    content: m.content.clone(),
                })
                .collect(),
            stream: false,
            temperature: None,
        };

        let response = self
            .client
            .post(self.chat_url())
            .header("Content-Type", "application/json")
            .json(&request)
            .send()
            .await
            .map_err(|e| OpenCodeError::Llm(e.to_string()))?;

        if !response.status().is_success() {
            let status = response.status();
            let error_text = response.text().await.unwrap_or_default();
            return Err(OpenCodeError::Llm(format!(
                "LM Studio API error {}: {}",
                status, error_text
            )));
        }

        let result: LmStudioChatResponse = response
            .json()
            .await
            .map_err(|e| OpenCodeError::Llm(e.to_string()))?;

        let content = result
            .choices
            .first()
            .map(|c| c.message.content.clone())
            .unwrap_or_default();

        Ok(ChatResponse {
            content,
            model: result.model.unwrap_or_else(|| self.model.clone()),
            usage: None,
        })
    }

    async fn complete_streaming(
        &self,
        prompt: &str,
        mut callback: StreamingCallback,
    ) -> Result<(), OpenCodeError> {
        let request = LmStudioChatRequest {
            model: self.model.clone(),
            messages: vec![LmStudioChatMessage {
                role: "user".to_string(),
                content: prompt.to_string(),
            }],
            stream: true,
            temperature: None,
        };

        let response = self
            .client
            .post(self.chat_url())
            .header("Content-Type", "application/json")
            .json(&request)
            .send()
            .await
            .map_err(|e| OpenCodeError::Llm(e.to_string()))?;

        if !response.status().is_success() {
            let status = response.status();
            let error_text = response.text().await.unwrap_or_default();
            return Err(OpenCodeError::Llm(format!(
                "LM Studio API error {}: {}",
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
                        let line = line.trim();
                        if line.starts_with("data: ") {
                            let data = line.strip_prefix("data: ").unwrap_or("");
                            if data == "[DONE]" {
                                return Ok(());
                            }
                            if let Ok(chunk) = serde_json::from_str::<LmStudioStreamChunk>(data) {
                                if let Some(choices) = chunk.choices {
                                    for choice in choices {
                                        if let Some(delta) = choice.delta {
                                            if let Some(content) = delta.content {
                                                callback(content);
                                            }
                                        }
                                    }
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

    fn provider_name(&self) -> &str {
        "lmstudio"
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_lmstudio_provider_new() {
        let provider = LmStudioProvider::new(
            "llama2".to_string(),
            Some("http://localhost:1234".to_string()),
        );
        assert_eq!(provider.model, "llama2");
        assert_eq!(provider.base_url, "http://localhost:1234");
    }

    #[test]
    fn test_lmstudio_provider_default_url() {
        let provider = LmStudioProvider::new("llama2".to_string(), None);
        assert_eq!(provider.base_url, "http://localhost:1234");
    }

    #[tokio::test]
    async fn test_lmstudio_chat_fails_without_server() {
        let provider = LmStudioProvider::new(
            "llama2".to_string(),
            Some("http://localhost:19999".to_string()),
        );
        let messages = vec![ChatMessage {
            role: "user".to_string(),
            content: "hello".to_string(),
        }];
        let result = provider.chat(&messages).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_lmstudio_streaming_fails_without_server() {
        let provider = LmStudioProvider::new(
            "llama2".to_string(),
            Some("http://localhost:19999".to_string()),
        );
        let result = provider.complete_streaming("hello", Box::new(|_| {})).await;
        assert!(result.is_err());
    }
}
