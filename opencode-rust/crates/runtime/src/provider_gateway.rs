use async_trait::async_trait;
use futures_util::stream::BoxStream;
use serde::{Deserialize, Serialize};

pub type ProviderRef = String;
pub type ModelRef = String;
pub type ProviderRequestId = String;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProviderMessage {
    pub role: String,
    pub content: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolDescriptor {
    pub name: String,
    pub description: String,
    pub input_schema: serde_json::Value,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Default)]
pub enum ToolChoice {
    #[default]
    Auto,
    Required,
    None,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ProviderRequestMetadata {
    pub session_id: Option<String>,
    pub trace_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProviderRequest {
    pub provider: ProviderRef,
    pub model: ModelRef,
    pub messages: Vec<ProviderMessage>,
    pub tools: Vec<ToolDescriptor>,
    pub tool_choice: ToolChoice,
    pub temperature: Option<f32>,
    pub max_tokens: Option<u32>,
    pub metadata: ProviderRequestMetadata,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NormalizedToolCall {
    pub call_id: String,
    pub name: String,
    pub arguments: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenUsage {
    pub input_tokens: u32,
    pub output_tokens: u32,
    pub total_tokens: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ProviderErrorKind {
    AuthenticationFailed,
    ModelNotFound,
    RateLimited,
    QuotaExceeded,
    ContextLengthExceeded,
    InvalidRequest,
    Network,
    Timeout,
    ServerError,
    UnsupportedCapability,
    Unknown,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProviderError {
    pub kind: ProviderErrorKind,
    pub message: String,
    pub provider: Option<String>,
    pub model: Option<String>,
}

impl ProviderError {
    pub fn new(kind: ProviderErrorKind, message: impl Into<String>) -> Self {
        Self {
            kind,
            message: message.into(),
            provider: None,
            model: None,
        }
    }

    pub fn with_provider(mut self, provider: ProviderRef) -> Self {
        self.provider = Some(provider);
        self
    }

    pub fn with_model(mut self, model: ModelRef) -> Self {
        self.model = Some(model);
        self
    }
}

impl std::fmt::Display for ProviderError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}: {}", self.kind, self.message)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ProviderStreamEvent {
    Started { request_id: ProviderRequestId },
    Token { text: String },
    ToolCallDelta {
        call_id: String,
        name: Option<String>,
        arguments_delta: String,
    },
    ToolCallCompleted { call: NormalizedToolCall },
    Completed { usage: Option<TokenUsage> },
    Error { error: ProviderError },
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum ProviderStatus {
    Available,
    Unavailable,
    AuthRequired,
    RateLimited,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelInfo {
    pub id: String,
    pub name: String,
    pub provider: String,
    pub capabilities: ModelCapabilities,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ModelCapabilities {
    pub context_window: Option<usize>,
    pub max_output_tokens: Option<usize>,
    pub supports_streaming: bool,
    pub supports_tools: bool,
    pub supports_parallel_tool_calls: bool,
    pub supports_json_schema: bool,
    pub supports_images: bool,
    pub supports_cache_control: bool,
}

#[async_trait]
pub trait ProviderGateway: Send + Sync {
    async fn validate_provider(
        &self,
        provider: ProviderRef,
    ) -> std::result::Result<ProviderStatus, ProviderError>;

    async fn list_models(
        &self,
        provider: ProviderRef,
    ) -> std::result::Result<Vec<ModelInfo>, ProviderError>;

    async fn stream_chat(
        &self,
        request: ProviderRequest,
    ) -> std::result::Result<BoxStream<'static, std::result::Result<ProviderStreamEvent, ProviderError>>, ProviderError>;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_provider_request_default_metadata() {
        let request = ProviderRequest {
            provider: "openai".to_string(),
            model: "gpt-4".to_string(),
            messages: vec![],
            tools: vec![],
            tool_choice: ToolChoice::Auto,
            temperature: None,
            max_tokens: None,
            metadata: ProviderRequestMetadata::default(),
        };
        assert!(request.metadata.session_id.is_none());
        assert!(request.metadata.trace_id.is_none());
    }

    #[test]
    fn test_provider_error_display() {
        let error = ProviderError::new(ProviderErrorKind::ModelNotFound, "Model not found");
        assert!(error.to_string().contains("ModelNotFound"));
    }

    #[test]
    fn test_tool_choice_default() {
        assert_eq!(ToolChoice::default(), ToolChoice::Auto);
    }

    #[test]
    fn test_model_capabilities_default() {
        let caps = ModelCapabilities::default();
        assert!(!caps.supports_streaming);
        assert!(!caps.supports_tools);
    }
}
