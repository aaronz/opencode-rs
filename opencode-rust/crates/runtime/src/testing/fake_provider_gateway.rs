use async_trait::async_trait;
use futures_util::stream::{self, BoxStream};
use futures_util::StreamExt;
use std::sync::{Arc, RwLock};

use crate::provider_gateway::{
    ModelCapabilities, ModelInfo, ProviderError, ProviderErrorKind, ProviderGateway,
    ProviderRequest, ProviderStatus, ProviderStreamEvent, TokenUsage,
};

pub struct FakeProviderGateway {
    status: RwLock<ProviderStatus>,
    models: RwLock<Vec<ModelInfo>>,
    responses: RwLock<Vec<ProviderStreamEvent>>,
    should_error: RwLock<bool>,
    error_kind: RwLock<ProviderErrorKind>,
}

impl Default for FakeProviderGateway {
    fn default() -> Self {
        Self::new()
    }
}

impl FakeProviderGateway {
    pub fn new() -> Self {
        Self {
            status: RwLock::new(ProviderStatus::Available),
            models: RwLock::new(vec![ModelInfo {
                id: "test-model".to_string(),
                name: "Test Model".to_string(),
                provider: "test".to_string(),
                capabilities: ModelCapabilities {
                    context_window: Some(128_000),
                    max_output_tokens: Some(4096),
                    supports_streaming: true,
                    supports_tools: true,
                    supports_parallel_tool_calls: true,
                    supports_json_schema: true,
                    supports_images: true,
                    supports_cache_control: true,
                },
            }]),
            responses: RwLock::new(vec![
                ProviderStreamEvent::Started {
                    request_id: "test-request".to_string(),
                },
                ProviderStreamEvent::Token {
                    text: "Hello, world!".to_string(),
                },
                ProviderStreamEvent::Completed {
                    usage: Some(TokenUsage {
                        input_tokens: 10,
                        output_tokens: 20,
                        total_tokens: 30,
                    }),
                },
            ]),
            should_error: RwLock::new(false),
            error_kind: RwLock::new(ProviderErrorKind::Unknown),
        }
    }

    pub fn with_status(self: Arc<Self>, status: ProviderStatus) -> Arc<Self> {
        *self.status.write().unwrap() = status;
        self
    }

    pub fn with_model(self: Arc<Self>, model: ModelInfo) -> Arc<Self> {
        self.models.write().unwrap().push(model);
        self
    }

    pub fn with_response(self: Arc<Self>, event: ProviderStreamEvent) -> Arc<Self> {
        self.responses.write().unwrap().push(event);
        self
    }

    pub fn with_error(self: Arc<Self>, kind: ProviderErrorKind) -> Arc<Self> {
        *self.should_error.write().unwrap() = true;
        *self.error_kind.write().unwrap() = kind;
        self
    }

    pub fn into_boxed(self: Arc<Self>) -> Arc<dyn ProviderGateway> {
        self
    }
}

#[async_trait]
impl ProviderGateway for FakeProviderGateway {
    async fn validate_provider(
        &self,
        _provider: String,
    ) -> std::result::Result<ProviderStatus, ProviderError> {
        let status = self.status.read().unwrap();
        if *status == ProviderStatus::Available {
            Ok(ProviderStatus::Available)
        } else {
            Err(ProviderError::new(
                ProviderErrorKind::AuthenticationFailed,
                "Provider not available",
            ))
        }
    }

    async fn list_models(
        &self,
        _provider: String,
    ) -> std::result::Result<Vec<ModelInfo>, ProviderError> {
        let should_error = *self.should_error.read().unwrap();
        if should_error {
            let kind = self.error_kind.read().unwrap().clone();
            return Err(ProviderError::new(kind, "Failed to list models"));
        }
        let models = self.models.read().unwrap();
        Ok(models.clone())
    }

    async fn stream_chat(
        &self,
        _request: ProviderRequest,
    ) -> std::result::Result<
        BoxStream<'static, std::result::Result<ProviderStreamEvent, ProviderError>>,
        ProviderError,
    > {
        let should_error = *self.should_error.read().unwrap();
        if should_error {
            let kind = self.error_kind.read().unwrap().clone();
            return Err(ProviderError::new(kind, "Stream chat failed"));
        }

        let responses = self.responses.read().unwrap();
        let events: Vec<_> = responses.clone();

        let stream = stream::iter(events.into_iter().map(Ok::<_, ProviderError>));

        Ok(stream.boxed())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::provider_gateway::ToolChoice;
    use futures_util::StreamExt;

    #[tokio::test]
    async fn test_fake_provider_validates_available() {
        let gateway = Arc::new(FakeProviderGateway::new());
        let result = gateway.validate_provider("test".to_string()).await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), ProviderStatus::Available);
    }

    #[tokio::test]
    async fn test_fake_provider_lists_models() {
        let gateway = Arc::new(FakeProviderGateway::new());
        let result = gateway.list_models("test".to_string()).await;
        assert!(result.is_ok());
        let models = result.unwrap();
        assert!(!models.is_empty());
        assert_eq!(models[0].id, "test-model");
    }

    #[tokio::test]
    async fn test_fake_provider_streams_response() {
        let gateway = Arc::new(FakeProviderGateway::new());
        let request = ProviderRequest {
            provider: "test".to_string(),
            model: "test-model".to_string(),
            messages: vec![],
            tools: vec![],
            tool_choice: ToolChoice::Auto,
            temperature: None,
            max_tokens: None,
            metadata: Default::default(),
        };

        let stream = match gateway.stream_chat(request).await {
            Ok(s) => s,
            Err(e) => panic!("Unexpected error: {:?}", e),
        };
        let events: Vec<_> = stream.collect().await;

        assert!(!events.is_empty());
        assert!(events.iter().any(|e| match e {
            Ok(ProviderStreamEvent::Token { text }) => text == "Hello, world!",
            _ => false,
        }));
    }

    #[tokio::test]
    async fn test_fake_provider_returns_error_for_unavailable_status() {
        let gateway = Arc::new(FakeProviderGateway::new());
        *gateway.status.write().unwrap() = ProviderStatus::Unavailable;

        let result = gateway.validate_provider("test".to_string()).await;
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(matches!(err.kind, ProviderErrorKind::AuthenticationFailed));
    }

    #[tokio::test]
    async fn test_fake_provider_stream_returns_error_when_configured() {
        let gateway = Arc::new(FakeProviderGateway::new()).with_error(ProviderErrorKind::Network);

        let request = ProviderRequest {
            provider: "test".to_string(),
            model: "test-model".to_string(),
            messages: vec![],
            tools: vec![],
            tool_choice: ToolChoice::Auto,
            temperature: None,
            max_tokens: None,
            metadata: Default::default(),
        };

        match gateway.stream_chat(request).await {
            Err(err) => {
                assert!(matches!(err.kind, ProviderErrorKind::Network));
            }
            Ok(_) => panic!("Expected error"),
        }
    }
}
