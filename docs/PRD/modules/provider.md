# provider.md — LLM Provider Module

## Module Overview

- **Crate**: `opencode-llm`
- **Source**: `crates/llm/src/lib.rs`
- **Status**: Fully implemented — PRD reflects actual Rust API
- **Purpose**: Multi-provider LLM abstraction with 20+ provider implementations (OpenAI, Anthropic, Ollama, Azure, Google, etc.), provider factory/registry pattern, budget tracking, model catalog, auth management, and request retry logic.

---

## Crate Layout

```
crates/llm/src/
├── lib.rs                        ← Public re-exports
├── provider.rs                   ← Provider trait, SimpleProvider, ChatMessage, Model, etc.
├── provider_abstraction.rs       ← DynProvider, ProviderFactory, ProviderManager, ProviderSpec
├── provider_adapter.rs           ← AnthropicAdapter, OpenAICompatibleAdapter, LocalEndpointAdapter
├── provider_filter.rs            ← ProviderFilter
├── provider_registry.rs          ← ProviderRegistry
├── error.rs                      ← LlmError, RetryConfig, with_retry
├── auth.rs                       ← AuthApplicator, AuthManager, AuthStrategy, Credential
├── auth_layered/                 ← OAuth flows (Google, Copilot)
├── budget.rs                     ← BudgetTracker, BudgetLimit, BudgetExceededError
├── catalog/                      ← ProviderCatalog, ProviderCatalogFetcher, ModelDescriptor
├── model_selection.rs            ← ModelSelection, ProviderType, UserModelConfig
├── models.rs                     ← ModelRegistry
├── transform.rs                  ← MessageTransform, TransformPipeline
├── [20+ provider implementations]:
│   ├── openai.rs                 ← OpenAiProvider
│   ├── anthropic.rs              ← AnthropicProvider
│   ├── ollama.rs                 ← OllamaProvider
│   ├── azure.rs                  ← AzureProvider
│   ├── google.rs                 ← GoogleProvider
│   ├── vertex.rs                  ← VertexProvider
│   ├── bedrock.rs                ← BedrockProvider
│   ├── openrouter.rs             ← OpenRouterProvider
│   ├── anthropic.rs              ← AnthropicProvider
│   ├── cohere.rs                 ← CohereProvider
│   ├── mistral.rs                ← MistralProvider
│   ├── huggingface.rs            ← HuggingFaceProvider
│   ├── lm_studio.rs              ← LmStudioProvider
│   ├── cerebras.rs               ← CerebrasProvider
│   ├── groq.rs                   ← GroqProvider
│   ├── deepinfra.rs              ← DeepInfraProvider
│   ├── togetherai.rs             ← TogetherAiProvider
│   ├── perplexity.rs             ← PerplexityProvider
│   ├── xai.rs                    ← XaiProvider
│   ├── ai_gateway.rs             ← AiGatewayProvider
│   ├── ai21.rs                   ← Ai21Provider
│   ├── copilot.rs                ← CopilotProvider
│   ├── gitlab.rs                 ← GitLabProvider
│   ├── vercel.rs                 ← VercelProvider
│   └── [more...]
```

**Key Cargo.toml dependencies**:
```toml
[dependencies]
async-trait = "0.1"
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
reqwest = { version = "0.12", features = ["json"] }
tokio = { version = "1.45", features = ["full"] }
thiserror = "2.0"
anyhow = "1.0"
tiktoken-rs = "0.6"

opencode-core = { path = "../core" }
```

**Public exports from lib.rs**:
```rust
pub use provider::{
    ChatMessage, ChatResponse, Model, Provider, ProviderConfig, SimpleProvider, StreamChunk,
    StreamingCallback, Usage,
};
pub use provider_abstraction::{
    AnthropicThinkingConfig, DynProvider, LmStudioProviderFactory, LocalInferenceProviderFactory,
    ProviderFactory, ProviderIdentity, ProviderManager, ProviderReasoningConfig, ProviderSpec,
    ReasoningBudget,
};
pub use error::{with_retry, LlmError, RetryConfig};
pub use auth::{AuthApplicator, AuthManager, AuthStrategy, Credential, CredentialStore, OAuthSessionManager,
    OAuthTokenResponse, ProviderAuthConfig};
pub use budget::{BudgetExceededError, BudgetLimit, BudgetTracker, ConversationBudgetState, RequestBudgetState, VariantCost};
pub use catalog::{merge_catalogs, CatalogSource, FetchError, ModelDescriptor, ModelStatus, ProviderCatalog,
    ProviderCatalogFetcher, ProviderDescriptor};
pub use model_selection::{ModelSelection, ProviderType, UserModelConfig};
pub use models::ModelRegistry;
// Plus all provider implementations...
```

---

## Core Types

### Provider Trait

```rust
pub mod sealed {
    pub trait Sealed {}
}

pub type StreamingCallback = Box<dyn FnMut(String) + Send>;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Model {
    pub id: String,
    pub name: String,
}

impl Model {
    pub fn new(id: &str, name: &str) -> Self { ... }
}

#[derive(Clone, Serialize, Deserialize)]
pub struct ProviderConfig {
    pub model: String,
    pub api_key: String,
    pub temperature: f32,
    #[serde(default)]
    pub headers: HashMap<String, String>,
}

impl ProviderConfig {
    pub fn sanitize_for_logging(&self) -> Self { ... }
}

impl Default for ProviderConfig {
    fn default() -> Self {
        Self {
            model: "gpt-4o".to_string(),
            api_key: String::new(),
            temperature: 0.7,
            headers: HashMap::new(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatMessage {
    pub role: String,
    pub content: String,
}

impl From<&Message> for ChatMessage { ... }

#[derive(Debug, Clone, Default)]
pub struct Usage {
    pub prompt_tokens: u64,
    pub completion_tokens: u64,
    pub total_tokens: u64,
}

impl Usage {
    pub fn new(prompt_tokens: u64, completion_tokens: u64) -> Self { ... }
    pub fn calculate_cost(&self, cost_per_1k_tokens: f64) -> f64 { ... }
}

#[derive(Debug, Clone)]
pub struct ChatResponse {
    pub content: String,
    pub model: String,
    pub usage: Option<Usage>,
}

impl ChatResponse {
    pub fn new(content: String, model: String) -> Self { ... }
    pub fn with_usage(mut self, usage: Usage) -> Self { ... }
}

#[derive(Debug, Clone)]
pub struct StreamChunk {
    pub content: String,
    pub done: bool,
}

#[async_trait]
pub trait Provider: Send + Sync + sealed::Sealed {
    async fn complete(&self, prompt: &str, context: Option<&str>) -> Result<String, OpenCodeError>;
    
    async fn complete_streaming(
        &self,
        prompt: &str,
        mut callback: StreamingCallback,
    ) -> Result<(), OpenCodeError> {
        let content = self.complete(prompt, None).await?;
        callback(content);
        Ok(())
    }

    async fn chat(&self, messages: &[ChatMessage]) -> Result<ChatResponse, OpenCodeError> {
        let prompt = messages.iter().map(|m| format!("{}: {}", m.role, m.content)).collect::<Vec<_>>().join("\n");
        let content = self.complete(&prompt, None).await?;
        Ok(ChatResponse::new(content, String::new()))
    }

    fn get_models(&self) -> Vec<Model> { vec![] }
    fn provider_name(&self) -> &str { "unknown" }
}

pub trait SimpleProvider: Send + Sync + sealed::Sealed {
    fn get_models(&self) -> Vec<Model>;
    fn provider_name(&self) -> &str;
}
```

### Provider Abstraction (DynProvider, ProviderManager)

```rust
// From provider_abstraction.rs
pub struct DynProvider(pub Box<dyn Provider>);

pub trait ProviderFactory: Send + Sync {
    fn create(&self, config: &ProviderConfig) -> Result<Box<dyn Provider>, LlmError>;
    fn provider_name(&self) -> &'static str;
    fn get_models(&self) -> Vec<Model>;
}

pub struct ProviderManager { ... }
impl ProviderManager {
    pub fn new() -> Self;
    pub fn register_factory(&self, factory: Box<dyn ProviderFactory>) -> Result<(), LlmError>;
    pub fn create_provider(&self, name: &str, config: &ProviderConfig) -> Result<DynProvider, LlmError>;
    pub fn list_providers(&self) -> Vec<String>;
}

pub struct ProviderIdentity {
    pub provider_name: String,
    pub model: String,
}

pub struct ProviderSpec {
    pub name: String,
    pub display_name: String,
    pub models: Vec<Model>,
    pub auth_type: AuthType,
}

pub enum AuthType {
    ApiKey,
    OAuth,
    Bearer,
    Azure,
    Vertex,
    Bedrock,
}

#[derive(Clone)]
pub struct ReasoningBudget {
    pub max_tokens: Option<u32>,
    pub prompt_tokens: Option<u32>,
}

pub struct AnthropicThinkingConfig {
    pub enabled: bool,
    pub budget_tokens: Option<u32>,
}
```

### LlmError

```rust
// From error.rs
#[derive(Debug, thiserror::Error)]
pub enum LlmError {
    #[error("Provider error: {0}")]
    Provider(String),
    #[error("Auth error: {0}")]
    Auth(String),
    #[error("Rate limit exceeded")]
    RateLimit,
    #[error("Context length exceeded: {0}")]
    ContextLengthExceeded(usize),
    #[error("Invalid request: {0}")]
    InvalidRequest(String),
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("HTTP error: {0}")]
    Http(#[from] reqwest::Error),
    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),
    #[error("Timeout after {0}s")]
    Timeout(u64),
    #[error("Budget exceeded: {0}")]
    BudgetExceeded(String),
    #[error("Unsupported provider: {0}")]
    UnsupportedProvider(String),
    #[error("Retry exhausted after {0} attempts")]
    RetryExhausted { attempts: u32, last_error: String },
}

#[derive(Debug, Clone, Copy)]
pub struct RetryConfig {
    pub max_attempts: u32,
    pub initial_backoff_ms: u64,
    pub max_backoff_ms: u64,
    pub backoff_multiplier: f64,
}

impl Default for RetryConfig {
    fn default() -> Self {
        Self {
            max_attempts: 3,
            initial_backoff_ms: 1000,
            max_backoff_ms: 30000,
            backoff_multiplier: 2.0,
        }
    }
}

pub async fn with_retry<F, Fut, T>(
    config: RetryConfig,
    operation: F,
) -> Result<T, LlmError>
where
    F: Fn() -> Fut,
    Fut: std::future::Future<Output = Result<T, LlmError>>,
{ ... }
```

### Budget Tracking

```rust
// From budget.rs
pub struct BudgetTracker { ... }
pub struct BudgetLimit {
    pub max_tokens: Option<u64>,
    pub max_cost: Option<f64>,
    pub warning_threshold: f64,  // 0.0-1.0
}

#[derive(Debug, Clone)]
pub struct ConversationBudgetState {
    pub total_tokens: u64,
    pub total_cost: f64,
    pub warning_triggered: bool,
}

#[derive(Debug, Clone)]
pub struct RequestBudgetState {
    pub prompt_tokens: u64,
    pub completion_tokens: u64,
    pub cost: f64,
}

pub struct VariantCost {
    pub variant_name: String,
    pub cost_per_1k_input: f64,
    pub cost_per_1k_output: f64,
}

#[derive(Debug, Clone, thiserror::Error)]
pub enum BudgetExceededError {
    #[error("Token budget exceeded: {0} > {1}")]
    TokenLimit { used: u64, limit: u64 },
    #[error("Cost budget exceeded: ${0} > ${1}")]
    CostLimit { used: f64, limit: f64 },
    #[error("Warning threshold reached: {0}%")]
    WarningThreshold { percent: f64 },
}
```

### Model Catalog

```rust
// From catalog/
pub struct ProviderCatalog {
    pub providers: Vec<ProviderDescriptor>,
}

pub struct ProviderDescriptor {
    pub name: String,
    pub display_name: String,
    pub models: Vec<ModelDescriptor>,
    pub enabled: bool,
}

pub struct ModelDescriptor {
    pub id: String,
    pub name: String,
    pub status: ModelStatus,
    pub context_length: Option<usize>,
    pub supports_streaming: bool,
    pub input_cost_per_1k: Option<f64>,
    pub output_cost_per_1k: Option<f64>,
    pub variants: Vec<String>,
}

pub enum ModelStatus {
    Stable,
    Beta,
    Deprecated,
    Legacy,
}

pub enum CatalogSource {
    Bundled,
    Remote,
    UserOverride,
}

pub struct ProviderCatalogFetcher { ... }
impl ProviderCatalogFetcher {
    pub async fn fetch(remote_url: &str) -> Result<ProviderCatalog, FetchError>;
    pub fn merge(local: ProviderCatalog, remote: ProviderCatalog) -> ProviderCatalog;
}
```

### Auth Types

```rust
// From auth.rs
pub struct Credential {
    pub provider: String,
    pub key: String,  // api_key or token
}

pub trait CredentialStore: Send + Sync {
    fn get(&self, provider: &str) -> Option<Credential>;
    fn set(&self, provider: &str, credential: Credential);
    fn remove(&self, provider: &str);
}

pub enum AuthStrategy {
    ApiKey(String),
    OAuth(OAuthSession),
    Bearer(String),
    Azure { tenant_id: String, client_id: String, client_secret: String },
}

pub struct AuthApplicator { ... }
pub struct AuthManager { ... }
```

---

## Inter-Crate Dependencies

| Dependant Crate | What it uses from `opencode-llm` |
|---|---|
| `opencode-agent` | `Provider` trait, `ChatMessage`, `DynProvider`, `ProviderManager` |
| `opencode-server` | `ProviderManager`, `BudgetTracker` |
| `opencode-core` | `LlmError`, `Message` conversion to `ChatMessage` |
| `opencode-config` | `ProviderConfig`, `ModelConfig` |

**Dependencies of `opencode-llm`**:
| Crate | Usage |
|---|---|
| `opencode-core` | `Message`, `OpenCodeError`, `Role` enum |

---

## Test Design

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_model_new() {
        let model = Model::new("gpt-4o", "GPT-4o");
        assert_eq!(model.id, "gpt-4o");
    }

    #[test]
    fn test_chat_message_from_message() {
        let msg = Message {
            role: Role::User,
            content: "Hello".into(),
            timestamp: chrono::Utc::now(),
            parts: None,
        };
        let chat = ChatMessage::from(&msg);
        assert_eq!(chat.role, "user");
        assert_eq!(chat.content, "Hello");
    }

    #[test]
    fn test_provider_config_default() {
        let config = ProviderConfig::default();
        assert_eq!(config.model, "gpt-4o");
        assert_eq!(config.temperature, 0.7);
        assert!(config.api_key.is_empty());
    }

    #[test]
    fn test_provider_config_sanitize() {
        let config = ProviderConfig { api_key: "secret".into(), ..Default::default() };
        let sanitized = config.sanitize_for_logging();
        assert_eq!(sanitized.api_key, "***REDACTED***");
    }

    #[test]
    fn test_usage_calculate_cost() {
        let usage = Usage::new(1000, 500);
        assert_eq!(usage.total_tokens, 1500);
        let cost = usage.calculate_cost(0.002);  // $2/1K tokens
        assert!((cost - 3.0).abs() < 0.001);
    }

    #[test]
    fn test_retry_config_default() {
        let config = RetryConfig::default();
        assert_eq!(config.max_attempts, 3);
        assert_eq!(config.initial_backoff_ms, 1000);
    }

    #[tokio::test]
    async fn test_with_retry_success_first_try() {
        let config = RetryConfig { max_attempts: 3, ..Default::default() };
        let result = with_retry(config, || async { Ok::<_, LlmError>(42) }).await;
        assert_eq!(result.unwrap(), 42);
    }

    #[tokio::test]
    async fn test_with_retry_eventually_succeeds() {
        let config = RetryConfig { max_attempts: 3, ..Default::default() };
        let attempts = Arc::new(std::sync::atomic::AtomicU32::new(0));
        let attempts_clone = attempts.clone();
        let result = with_retry(config, move || {
            let attempts = attempts_clone.clone();
            async move {
                attempts.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
                if attempts.load(std::sync::atomic::Ordering::SeqCst) < 2 {
                    Err(LlmError::RateLimit)
                } else {
                    Ok(42)
                }
            }
        }).await;
        assert_eq!(result.unwrap(), 42);
    }

    #[test]
    fn test_budget_exceeded_error_display() {
        let err = BudgetExceededError::TokenLimit { used: 100000, limit: 50000 };
        assert!(err.to_string().contains("100000"));
    }

    #[test]
    fn test_chat_response_with_usage() {
        let usage = Usage::new(100, 50);
        let response = ChatResponse::new("hello".into(), "gpt-4".into()).with_usage(usage);
        assert!(response.usage.is_some());
        assert_eq!(response.usage.unwrap().total_tokens, 150);
    }

    #[test]
    fn test_stream_chunk() {
        let chunk = StreamChunk { content: "Hello".into(), done: false };
        assert_eq!(chunk.content, "Hello");
        assert!(!chunk.done);
    }
}
```

---

## Usage Example

```rust
use opencode_llm::{
    Provider, ProviderManager, ProviderConfig, ChatMessage,
    Model, DynProvider, LlmError,
};

async fn chat_example() -> Result<(), LlmError> {
    let manager = ProviderManager::new();
    // Register providers via factories...
    
    let config = ProviderConfig {
        model: "gpt-4o".into(),
        api_key: std::env::var("OPENAI_API_KEY").unwrap(),
        temperature: 0.7,
        headers: Default::default(),
    };
    
    let provider = manager.create_provider("openai", &config)?;
    
    let messages = vec![
        ChatMessage { role: "user".into(), content: "Hello!".into() },
    ];
    
    let response = provider.chat(&messages).await?;
    println!("Response: {}", response.content);
    
    if let Some(usage) = response.usage {
        println!("Tokens: {} total", usage.total_tokens);
    }
    
    Ok(())
}

async fn streaming_example() -> Result<(), LlmError> {
    let provider: DynProvider = /* ... */;
    let callback = |chunk: String| {
        print!("{}", chunk);
    };
    provider.complete_streaming("Say hello", callback).await?;
    Ok(())
}
```
