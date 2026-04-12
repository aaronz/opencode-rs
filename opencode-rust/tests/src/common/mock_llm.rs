use std::sync::{Arc, Mutex};

use async_trait::async_trait;
use opencode_core::OpenCodeError;
use opencode_llm::provider::{
    ChatMessage, ChatResponse, Model, Provider, ProviderConfig, StreamingCallback,
};

struct MockState {
    responses: Vec<String>,
    streaming_chunks: Vec<Vec<String>>,
    errors: Vec<String>,
    last_prompt: Option<String>,
    call_count: usize,
    response_index: usize,
    stream_index: usize,
    error_index: usize,
}

pub struct MockLLMProvider {
    state: Arc<Mutex<MockState>>,
    config: ProviderConfig,
}

impl MockLLMProvider {
    pub fn new() -> Self {
        Self {
            state: Arc::new(Mutex::new(MockState {
                responses: Vec::new(),
                streaming_chunks: Vec::new(),
                errors: Vec::new(),
                last_prompt: None,
                call_count: 0,
                response_index: 0,
                stream_index: 0,
                error_index: 0,
            })),
            config: ProviderConfig::default(),
        }
    }

    pub fn with_model(mut self, model: &str) -> Self {
        self.config.model = model.to_string();
        self
    }

    pub fn with_response(self, content: &str) -> Self {
        self.state
            .lock()
            .unwrap()
            .responses
            .push(content.to_string());
        self
    }

    pub fn with_streaming_chunks(self, chunks: Vec<&str>) -> Self {
        self.state
            .lock()
            .unwrap()
            .streaming_chunks
            .push(chunks.iter().map(|s| s.to_string()).collect());
        self
    }

    pub fn with_error(self, error: &str) -> Self {
        self.state.lock().unwrap().errors.push(error.to_string());
        self
    }

    pub fn last_prompt(&self) -> Option<String> {
        self.state.lock().unwrap().last_prompt.clone()
    }

    pub fn call_count(&self) -> usize {
        self.state.lock().unwrap().call_count
    }
}

impl Default for MockLLMProvider {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl Provider for MockLLMProvider {
    async fn complete(
        &self,
        prompt: &str,
        _context: Option<&str>,
    ) -> Result<String, OpenCodeError> {
        let mut state = self.state.lock().unwrap();
        state.last_prompt = Some(prompt.to_string());
        state.call_count += 1;

        if state.error_index < state.errors.len() {
            let error = state.errors[state.error_index].clone();
            state.error_index += 1;
            return Err(OpenCodeError::Llm(error));
        }

        if state.response_index < state.responses.len() {
            let response = state.responses[state.response_index].clone();
            state.response_index += 1;
            Ok(response)
        } else {
            Ok("mock response".to_string())
        }
    }

    async fn complete_streaming(
        &self,
        prompt: &str,
        mut callback: StreamingCallback,
    ) -> Result<(), OpenCodeError> {
        let mut state = self.state.lock().unwrap();
        state.last_prompt = Some(prompt.to_string());
        state.call_count += 1;

        if state.error_index < state.errors.len() {
            let error = state.errors[state.error_index].clone();
            state.error_index += 1;
            return Err(OpenCodeError::Llm(error));
        }

        if state.stream_index < state.streaming_chunks.len() {
            let chunks = state.streaming_chunks[state.stream_index].clone();
            state.stream_index += 1;
            for chunk in chunks {
                callback(chunk);
            }
        } else {
            callback("mock streaming response".to_string());
        }

        Ok(())
    }

    async fn chat(&self, messages: &[ChatMessage]) -> Result<ChatResponse, OpenCodeError> {
        let prompt = messages
            .iter()
            .map(|m| format!("{}: {}", m.role, m.content))
            .collect::<Vec<_>>()
            .join("\n");

        let content = self.complete(&prompt, None).await?;
        Ok(ChatResponse {
            content,
            model: self.config.model.clone(),
            usage: None,
        })
    }

    fn get_models(&self) -> Vec<Model> {
        vec![Model::new("mock-model", "Mock Model")]
    }

    fn provider_name(&self) -> &str {
        "mock"
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_mock_provider_returns_queued_response() {
        let provider = MockLLMProvider::new().with_response("hello");
        let result = provider.complete("test", None).await.unwrap();
        assert_eq!(result, "hello");
        assert_eq!(provider.call_count(), 1);
        assert_eq!(provider.last_prompt(), Some("test".to_string()));
    }

    #[tokio::test]
    async fn test_mock_provider_returns_error() {
        let provider = MockLLMProvider::new().with_error("test error");
        let result = provider.complete("test", None).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_mock_provider_streaming() {
        use std::sync::Arc;
        let chunks = Arc::new(std::sync::Mutex::new(Vec::new()));
        let chunks_clone = chunks.clone();

        let callback: Box<dyn FnMut(String) + Send> = Box::new(move |chunk: String| {
            if let Ok(mut guard) = chunks_clone.lock() {
                guard.push(chunk);
            }
        });

        let provider = MockLLMProvider::new().with_streaming_chunks(vec!["a", "b", "c"]);
        provider.complete_streaming("test", callback).await.unwrap();

        let result = chunks.lock().unwrap();
        assert_eq!(
            *result,
            vec!["a".to_string(), "b".to_string(), "c".to_string()]
        );
    }

    #[tokio::test]
    async fn test_mock_provider_chat() {
        let provider = MockLLMProvider::new()
            .with_model("mock-model")
            .with_response("chat response");

        let messages = vec![ChatMessage {
            role: "user".to_string(),
            content: "hi".to_string(),
        }];
        let result = provider.chat(&messages).await.unwrap();
        assert_eq!(result.content, "chat response");
        assert_eq!(result.model, "mock-model");
    }

    #[tokio::test]
    async fn test_mock_provider_default_response() {
        let provider = MockLLMProvider::new();
        let result = provider.complete("test", None).await.unwrap();
        assert_eq!(result, "mock response");
    }
}
