use async_trait::async_trait;
use reqwest::Client;
use serde::{Deserialize, Serialize};

use crate::provider::sealed;
use crate::provider::{ChatMessage, ChatResponse, Model, Provider, StreamingCallback};
use opencode_core::OpenCodeError;

pub struct OllamaProvider {
    client: Client,
    base_url: String,
    model: String,
}

#[derive(Serialize)]
struct OllamaGenerateRequest {
    model: String,
    prompt: String,
    stream: bool,
}

#[derive(Serialize)]
struct OllamaChatRequest {
    model: String,
    messages: Vec<OllamaChatMessage>,
    stream: bool,
}

#[derive(Serialize)]
struct OllamaChatMessage {
    role: String,
    content: String,
}

#[derive(Deserialize)]
struct OllamaGenerateResponse {
    response: String,
}

#[derive(Deserialize)]
struct OllamaChatResponse {
    message: OllamaMessage,
    model: Option<String>,
}

#[derive(Deserialize)]
struct OllamaMessage {
    content: String,
}

#[derive(Deserialize)]
struct StreamChunk {
    response: Option<String>,
    done: bool,
}

#[derive(Deserialize)]
struct OllamaTagsResponse {
    models: Vec<OllamaModelInfo>,
}

#[derive(Deserialize)]
#[allow(dead_code)]
struct OllamaModelInfo {
    name: String,
    #[serde(default)]
    model: String,
    #[serde(default)]
    modified_at: String,
    #[serde(default)]
    size: u64,
    #[serde(default)]
    digest: String,
}

impl OllamaProvider {
    pub fn new(model: String, base_url: Option<String>) -> Self {
        let client = Client::builder()
            .timeout(std::time::Duration::from_secs(30))
            .build()
            .unwrap_or_default();
        Self {
            client,
            base_url: base_url.unwrap_or_else(|| "http://localhost:11434".to_string()),
            model,
        }
    }

    fn generate_url(&self) -> String {
        format!("{}/api/generate", self.base_url)
    }

    fn chat_url(&self) -> String {
        format!("{}/api/chat", self.base_url)
    }

    fn tags_url(&self) -> String {
        format!("{}/api/tags", self.base_url)
    }

    pub async fn get_local_models(&self) -> Result<Vec<Model>, OpenCodeError> {
        let response = self.client.get(self.tags_url()).send().await.map_err(|e| {
            tracing::error!(provider = "ollama", error = %e, "Failed to fetch Ollama models");
            OpenCodeError::Llm(e.to_string())
        })?;

        if !response.status().is_success() {
            let status = response.status();
            tracing::error!(provider = "ollama", status = %status, "Failed to fetch Ollama models");
            return Err(OpenCodeError::Llm(format!(
                "Ollama API error {}: failed to list models",
                status
            )));
        }

        let tags: OllamaTagsResponse = response.json().await.map_err(|e| {
            tracing::error!(provider = "ollama", error = %e, "Failed to parse Ollama models response");
            OpenCodeError::Llm(e.to_string())
        })?;

        let models: Vec<Model> = tags
            .models
            .into_iter()
            .map(|info| Model::new(&info.name, &info.name))
            .collect();

        tracing::debug!(
            provider = "ollama",
            model_count = models.len(),
            "Fetched local Ollama models"
        );
        Ok(models)
    }
}

impl sealed::Sealed for OllamaProvider {}

#[async_trait]
impl Provider for OllamaProvider {
    async fn complete(
        &self,
        prompt: &str,
        _context: Option<&str>,
    ) -> Result<String, OpenCodeError> {
        tracing::debug!(provider = "ollama", model = %self.model, prompt_len = prompt.len(), "Starting Ollama completion");

        let request = OllamaGenerateRequest {
            model: self.model.clone(),
            prompt: prompt.to_string(),
            stream: false,
        };

        let response = self
            .client
            .post(self.generate_url())
            .json(&request)
            .send()
            .await
            .map_err(|e| {
                tracing::error!(provider = "ollama", error = %e, "Ollama request failed");
                OpenCodeError::Llm(e.to_string())
            })?;

        if !response.status().is_success() {
            let status = response.status();
            let error_text = response.text().await.unwrap_or_default();
            tracing::error!(provider = "ollama", status = %status, error = %error_text, "Ollama API error");
            return Err(OpenCodeError::Llm(format!(
                "Ollama API error {}: {}",
                status, error_text
            )));
        }

        let result: OllamaGenerateResponse = response.json().await.map_err(|e| {
            tracing::error!(provider = "ollama", error = %e, "Failed to parse Ollama response");
            OpenCodeError::Llm(e.to_string())
        })?;

        tracing::info!(provider = "ollama", model = %self.model, response_len = result.response.len(), "Ollama completion successful");
        Ok(result.response)
    }

    async fn chat(&self, messages: &[ChatMessage]) -> Result<ChatResponse, OpenCodeError> {
        tracing::debug!(provider = "ollama", model = %self.model, message_count = messages.len(), "Starting Ollama chat");

        let request = OllamaChatRequest {
            model: self.model.clone(),
            messages: messages
                .iter()
                .map(|m| OllamaChatMessage {
                    role: m.role.clone(),
                    content: m.content.clone(),
                })
                .collect(),
            stream: false,
        };

        let response = self
            .client
            .post(self.chat_url())
            .json(&request)
            .send()
            .await
            .map_err(|e| {
                tracing::error!(provider = "ollama", error = %e, "Ollama chat request failed");
                OpenCodeError::Llm(e.to_string())
            })?;

        if !response.status().is_success() {
            let status = response.status();
            let error_text = response.text().await.unwrap_or_default();
            tracing::error!(provider = "ollama", status = %status, error = %error_text, "Ollama chat API error");
            return Err(OpenCodeError::Llm(format!(
                "Ollama API error {}: {}",
                status, error_text
            )));
        }

        let result: OllamaChatResponse = response
            .json()
            .await
            .map_err(|e| {
                tracing::error!(provider = "ollama", error = %e, "Failed to parse Ollama chat response");
                OpenCodeError::Llm(e.to_string())
            })?;

        tracing::info!(provider = "ollama", model = %self.model, response_len = result.message.content.len(), "Ollama chat successful");
        Ok(ChatResponse::new(
            result.message.content,
            result.model.unwrap_or_default(),
        ))
    }

    async fn complete_streaming(
        &self,
        prompt: &str,
        mut callback: StreamingCallback,
    ) -> Result<(), OpenCodeError> {
        let request = OllamaGenerateRequest {
            model: self.model.clone(),
            prompt: prompt.to_string(),
            stream: true,
        };

        let response = self
            .client
            .post(self.generate_url())
            .json(&request)
            .send()
            .await
            .map_err(|e| OpenCodeError::Llm(e.to_string()))?;

        if !response.status().is_success() {
            let status = response.status();
            let error_text = response.text().await.unwrap_or_default();
            return Err(OpenCodeError::Llm(format!(
                "Ollama API error {}: {}",
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
                        if let Ok(chunk) = serde_json::from_str::<StreamChunk>(line) {
                            if let Some(response) = chunk.response {
                                callback(response);
                            }
                            if chunk.done {
                                return Ok(());
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
        "ollama"
    }

    fn get_models(&self) -> Vec<Model> {
        // Return common Ollama models as fallback.
        // For actual local model discovery, use get_local_models() which is async
        // and can properly query the Ollama server.
        vec![
            Model::new("llama3", "Llama 3"),
            Model::new("llama3.1", "Llama 3.1"),
            Model::new("llama3.2", "Llama 3.2"),
            Model::new("mistral", "Mistral"),
            Model::new("codellama", "Code Llama"),
            Model::new("qwen2.5", "Qwen 2.5"),
            Model::new("qwen2.5-coder", "Qwen 2.5 Coder"),
            Model::new("phi3", "Phi-3"),
            Model::new("gemma2", "Gemma 2"),
        ]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ollama_provider_new() {
        let provider = OllamaProvider::new(
            "llama2".to_string(),
            Some("http://localhost:11434".to_string()),
        );
        assert_eq!(provider.model, "llama2");
    }

    #[tokio::test]
    async fn test_ollama_complete_fails_without_server() {
        let provider = OllamaProvider::new(
            "llama2".to_string(),
            Some("http://localhost:19999".to_string()),
        );
        let result = provider.complete("hello", None).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_ollama_chat_fails_without_server() {
        let provider = OllamaProvider::new(
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
    async fn test_ollama_get_local_models() {
        let provider = OllamaProvider::new(
            "qwen3.5:9b".to_string(),
            Some("http://localhost:11434".to_string()),
        );
        let result = provider.get_local_models().await;
        assert!(result.is_ok());
        let models = result.unwrap();
        assert!(!models.is_empty());
        // Should contain qwen3.5:9b which is the installed model
        assert!(models.iter().any(|m| m.id == "qwen3.5:9b"));
    }

    #[test]
    fn test_ollama_get_models_returns_fallback() {
        let provider = OllamaProvider::new(
            "llama2".to_string(),
            Some("http://localhost:19999".to_string()),
        );
        // get_models returns fallback models since sync context can't query server
        let models = provider.get_models();
        assert!(!models.is_empty());
        // Should contain common fallback models
        assert!(models.iter().any(|m| m.id == "llama3"));
    }
}
