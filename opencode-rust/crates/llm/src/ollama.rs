use async_trait::async_trait;
use reqwest::Client;
use serde::{Deserialize, Serialize};

use crate::provider::sealed;
use crate::provider::{ChatMessage, ChatResponse, Model, Provider, StreamingCallback};
use opencode_core::OpenCodeError;

const OLLAMA_CONNECT_TIMEOUT_SECS: u64 = 10;
const OLLAMA_REQUEST_TIMEOUT_SECS: u64 = 120;

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
    #[serde(default)]
    #[allow(dead_code)]
    thinking: Option<String>,
    done: bool,
    #[serde(default)]
    error: Option<String>,
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
            .connect_timeout(std::time::Duration::from_secs(OLLAMA_CONNECT_TIMEOUT_SECS))
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
        let response = self
            .client
            .get(self.tags_url())
            .timeout(std::time::Duration::from_secs(OLLAMA_REQUEST_TIMEOUT_SECS))
            .send()
            .await
            .map_err(|e| {
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

fn process_stream_buffer<F>(buffer: &mut String, callback: &mut F) -> Result<bool, OpenCodeError>
where
    F: FnMut(&str),
{
    let mut done = false;

    while let Some(newline_idx) = buffer.find('\n') {
        let line = buffer[..newline_idx].trim_end_matches('\r').trim();
        let line = line.to_string();
        buffer.drain(..=newline_idx);

        if line.is_empty() {
            continue;
        }

        let chunk: StreamChunk = serde_json::from_str(&line)
            .map_err(|e| OpenCodeError::Llm(format!("Failed to parse Ollama stream chunk: {e}")))?;

        if let Some(error) = chunk.error {
            return Err(OpenCodeError::Llm(format!("Ollama stream error: {error}")));
        }

        if let Some(response) = chunk.response {
            if !response.is_empty() {
                callback(&response);
            }
        }

        if chunk.done {
            done = true;
            break;
        }
    }

    Ok(done)
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
            .timeout(std::time::Duration::from_secs(OLLAMA_REQUEST_TIMEOUT_SECS))
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
            .timeout(std::time::Duration::from_secs(OLLAMA_REQUEST_TIMEOUT_SECS))
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
            .timeout(std::time::Duration::from_secs(OLLAMA_REQUEST_TIMEOUT_SECS))
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
        let mut buffer = String::new();
        let mut emitted_any = false;

        use futures_util::StreamExt;
        while let Some(item) = lines.next().await {
            match item {
                Ok(bytes) => {
                    buffer.push_str(&String::from_utf8_lossy(&bytes));
                    let done = process_stream_buffer(&mut buffer, &mut |chunk: &str| {
                        emitted_any = true;
                        callback(chunk.to_string());
                    })?;

                    if done {
                        return Ok(());
                    }
                }
                Err(e) => {
                    if !emitted_any {
                        let fallback = self.complete(prompt, None).await?;
                        callback(fallback);
                        return Ok(());
                    }
                    return Err(OpenCodeError::Llm(format!("Stream error: {}", e)));
                }
            }
        }

        let done = process_stream_buffer(&mut buffer, &mut |chunk: &str| {
            emitted_any = true;
            callback(chunk.to_string());
        })?;

        if done || buffer.trim().is_empty() {
            return Ok(());
        }

        if !emitted_any {
            let fallback = self.complete(prompt, None).await?;
            callback(fallback);
            return Ok(());
        }

        tracing::warn!(
            provider = "ollama",
            partial = !buffer.is_empty(),
            "Ollama stream ended with partial data"
        );
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
    use std::io::{Read, Write};
    use std::net::TcpListener;
    use std::sync::{Arc, Mutex};
    use std::thread;

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

    #[test]
    fn test_process_stream_buffer_handles_split_json_lines() {
        let mut buffer = String::from("{\"response\":\"Hel");
        let emitted = Arc::new(Mutex::new(Vec::new()));
        let emitted_clone = emitted.clone();

        let result = process_stream_buffer(&mut buffer, &mut |chunk: &str| {
            emitted_clone.lock().unwrap().push(chunk.to_string());
        })
        .unwrap();

        assert!(!result);
        assert!(emitted.lock().unwrap().is_empty());

        buffer.push_str("lo\",\"done\":false}\n{\"done\":true}\n");
        let result = process_stream_buffer(&mut buffer, &mut |chunk: &str| {
            emitted.lock().unwrap().push(chunk.to_string());
        })
        .unwrap();

        assert!(result);
        assert_eq!(emitted.lock().unwrap().as_slice(), ["Hello"]);
        assert!(buffer.is_empty());
    }

    #[test]
    fn test_process_stream_buffer_ignores_thinking_only_chunks() {
        let mut buffer = String::from(
            "{\"response\":\"\",\"thinking\":\"step 1\",\"done\":false}\n{\"response\":\"done\",\"done\":false}\n{\"done\":true}\n",
        );
        let emitted = Arc::new(Mutex::new(Vec::new()));
        let emitted_clone = emitted.clone();

        let result = process_stream_buffer(&mut buffer, &mut |chunk: &str| {
            emitted_clone.lock().unwrap().push(chunk.to_string());
        })
        .unwrap();

        assert!(result);
        assert_eq!(emitted.lock().unwrap().as_slice(), ["done"]);
        assert!(buffer.is_empty());
    }

    #[tokio::test]
    async fn test_ollama_streaming_falls_back_after_stream_failure() {
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let addr = listener.local_addr().unwrap();
        let requests = Arc::new(Mutex::new(Vec::new()));
        let requests_clone = requests.clone();

        thread::spawn(move || {
            for _ in 0..2 {
                let (mut stream, _) = listener.accept().unwrap();
                let mut buffer = Vec::new();
                let mut chunk = [0_u8; 4096];

                loop {
                    let read = stream.read(&mut chunk).unwrap_or(0);
                    if read == 0 {
                        break;
                    }
                    buffer.extend_from_slice(&chunk[..read]);
                    if buffer.windows(4).any(|w| w == b"\r\n\r\n") {
                        break;
                    }
                }

                let request_text = String::from_utf8_lossy(&buffer).to_string();
                let body = request_text
                    .split("\r\n\r\n")
                    .nth(1)
                    .unwrap_or("")
                    .to_string();
                requests_clone.lock().unwrap().push(body.clone());

                if body.contains("\"stream\":true") {
                    let response = "HTTP/1.1 200 OK\r\nTransfer-Encoding: chunked\r\nContent-Type: application/json\r\n\r\n";
                    let _ = stream.write_all(response.as_bytes());
                    let _ = stream.flush();
                    continue;
                }

                let response_body = "{\"response\":\"fallback works\"}";
                let response = format!(
                    "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
                    response_body.len(),
                    response_body
                );
                let _ = stream.write_all(response.as_bytes());
                let _ = stream.flush();
            }
        });

        let provider =
            OllamaProvider::new("qwen3.5:9b".to_string(), Some(format!("http://{}", addr)));
        let output = Arc::new(Mutex::new(String::new()));
        let output_clone = output.clone();

        provider
            .complete_streaming(
                "hello",
                Box::new(move |chunk| output_clone.lock().unwrap().push_str(&chunk)),
            )
            .await
            .unwrap();

        assert_eq!(output.lock().unwrap().as_str(), "fallback works");
        assert_eq!(requests.lock().unwrap().len(), 2);
        assert!(requests.lock().unwrap()[0].contains("\"stream\":true"));
        assert!(requests.lock().unwrap()[1].contains("\"stream\":false"));
    }
}
