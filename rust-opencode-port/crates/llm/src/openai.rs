use async_trait::async_trait;
use reqwest::Client;
use serde::{Deserialize, Serialize};

use crate::provider::{Provider, StreamingCallback};
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
    messages: Vec<Message>,
    stream: bool,
}

#[derive(Serialize)]
struct Message {
    role: String,
    content: String,
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

#[derive(Deserialize)]
struct StreamChunk {
    choices: Vec<StreamChoice>,
}

#[derive(Deserialize)]
struct StreamChoice {
    delta: StreamDelta,
}

#[derive(Deserialize)]
struct StreamDelta {
    content: Option<String>,
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
    async fn complete(&self, prompt: &str, _context: Option<&str>) -> Result<String, OpenCodeError> {
        let messages = vec![Message {
            role: "user".to_string(),
            content: prompt.to_string(),
        }];
        
        let request = ChatRequest {
            model: self.model.clone(),
            messages,
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

        if !response.status().is_success() {
            let status = response.status();
            let error_text = response.text().await.unwrap_or_default();
            return Err(OpenCodeError::Llm(format!("OpenAI API error {}: {}", status, error_text)));
        }

        let completion: ChatCompletion = response
            .json()
            .await
            .map_err(|e| OpenCodeError::Llm(e.to_string()))?;

        let content = completion
            .choices
            .first()
            .map(|c| c.message.content.clone())
            .unwrap_or_default();

        Ok(content)
    }

    async fn complete_streaming(
        &self,
        prompt: &str,
        mut callback: StreamingCallback,
    ) -> Result<(), OpenCodeError> {
        let messages = vec![Message {
            role: "user".to_string(),
            content: prompt.to_string(),
        }];
        
        let request = ChatRequest {
            model: self.model.clone(),
            messages,
            stream: true,
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

        if !response.status().is_success() {
            let status = response.status();
            let error_text = response.text().await.unwrap_or_default();
            return Err(OpenCodeError::Llm(format!("OpenAI API error {}: {}", status, error_text)));
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
                            if data == "[DONE]" {
                                callback(String::new());
                                return Ok(());
                            }
                            if let Ok(chunk) = serde_json::from_str::<StreamChunk>(data) {
                                if let Some(content) = chunk.choices.first().and_then(|c| c.delta.content.clone()) {
                                    callback(content);
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
    fn test_openai_provider_new() {
        let provider = OpenAiProvider::new("test-key".to_string(), "gpt-4".to_string());
        assert_eq!(provider.model, "gpt-4");
        assert_eq!(provider.api_key, "test-key");
    }

    #[tokio::test]
    async fn test_openai_complete_returns_error_without_api_key() {
        let provider = OpenAiProvider::new("invalid-key".to_string(), "gpt-4".to_string());
        let result = provider.complete("test prompt", None).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_openai_streaming_returns_error_without_valid_key() {
        let provider = OpenAiProvider::new("invalid-key".to_string(), "gpt-4".to_string());
        let result = provider.complete_streaming("test", Box::new(|_| {})).await;
        assert!(result.is_err());
    }
}
