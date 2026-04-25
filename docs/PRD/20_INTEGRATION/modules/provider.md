# provider.md — LLM Provider Module

> **User Documentation**: [providers.mdx](https://github.com/anomalyco/opencode/blob/main/packages/web/src/content/docs/zh-cn/providers.mdx) — Configuring LLM providers

**See Also**: [Glossary: Provider & Model](../../system/01_glossary.md#provider) | [System PRD: Provider/Model System](../../system/10-provider-model-system.md)

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

---

## Error Handling

### Provider Error Types

| Error Type | Code | Description |
|------------|------|-------------|
| `ProviderNotFound` | 3001 | Requested provider not configured |
| `ProviderAuthFailed` | 3002 | Provider authentication failed (invalid API key) |
| `ProviderUnavailable` | 3003 | Provider service temporarily unavailable |
| `RateLimit` | 3004 | Provider rate limit exceeded |
| `InvalidModel` | 3005 | Requested model not available |
| `ContextLengthExceeded` | 3006 | Request exceeds model's context length |

### Error Handling Matrix

| Scenario | Expected Behavior | Error Code |
|----------|------------------|------------|
| Invalid API key | Return `ProviderAuthFailed` with details | 3002 |
| Provider returns 401 | Return `ProviderAuthFailed` | 3002 |
| Provider returns 429 | Return `RateLimit`, trigger retry | 3004 |
| Provider returns 503 | Return `ProviderUnavailable`, trigger retry | 3003 |
| Model not found | Return `InvalidModel` | 3005 |
| Prompt too long | Return `ContextLengthExceeded` | 3006 |

---

## Provider Matrix

### Supported Providers

| Provider | Type | Auth Method | Streaming | Models |
|----------|------|-------------|-----------|--------|
| OpenAI | OpenAI | API Key | Yes | GPT-4, GPT-4o, GPT-3.5 |
| Anthropic | Anthropic | API Key | Yes | Claude 3.5, Claude 3 |
| Ollama | Local | None | Yes | Llama, Mistral, etc. |
| Azure OpenAI | Azure | API Key + Endpoint | Yes | GPT-4, GPT-35 |
| Google | Google | API Key | Yes | Gemini Pro, Flash |
| AWS Bedrock | AWS | AWS Credentials | Yes | Claude, Titan |
| OpenRouter | Proxy | API Key | Yes | 100+ models |
| Groq | Groq | API Key | Yes | Llama, Mixtral |
| DeepInfra | DeepInfra | API Key | Yes | Llama, Mistral |
| Cerebras | Cerebras | API Key | Yes | Cerebras-GLM |
| Cohere | Cohere | API Key | Yes | Command R+ |
| Mistral | Mistral | API Key | Yes | Mistral Large |
| TogetherAI | TogetherAI | API Key | Yes | Llama, Mixtral |
| Perplexity | Perplexity | API Key | Yes | Sonar |
| XAI | XAI | API Key | Yes | Grok |
| HuggingFace | HuggingFace | API Key | Yes | Inference Endpoints |
| LM Studio | Local | None | Yes | Local models |
| Vercel | Vercel | API Key | Yes | Vercel AI SDK |

---

## Acceptance Criteria

### Provider Trait

| ID | Criterion | Test Method |
|----|-----------|-------------|
| AC-P001 | `provider.chat()` returns `ChatResponse` | Unit test |
| AC-P002 | `provider.complete_streaming()` calls callback with chunks | Integration test |
| AC-P003 | `provider.get_model()` returns correct model info | Unit test |
| AC-P004 | Provider implements `Send + Sync` | Compile-time check |

### ProviderManager

| ID | Criterion | Test Method |
|----|-----------|-------------|
| AC-PM001 | `ProviderManager::new()` creates empty manager | Unit test |
| AC-PM002 | `register_provider()` makes provider available | Unit test |
| AC-PM003 | `create_provider()` returns configured provider | Unit test |
| AC-PM004 | `create_provider("nonexistent")` returns error | Unit test |
| AC-PM005 | Multiple providers can be registered | Unit test |

### OpenAI Provider

| ID | Criterion | Given-When-Then |
|----|-----------|------------------|
| AC-OPENAI001 | Valid API request | Given valid API key and model, When `chat()` with valid messages, Then return response |
| AC-OPENAI002 | Invalid API key | Given invalid API key, When `chat()`, Then return `ProviderAuthFailed` |
| AC-OPENAI003 | GPT-4o model | Given "gpt-4o" model, When `chat()`, Then use correct model endpoint |
| AC-OPENAI004 | Streaming response | Given streaming enabled, When `complete_streaming()`, Then receive chunks via callback |

### Anthropic Provider

| ID | Criterion | Given-When-Then |
|----|-----------|------------------|
| AC-ANTH001 | Valid API request | Given valid API key, When `chat()` with messages, Then return response |
| AC-ANTH002 | Missing API key | Given no API key, When `chat()`, Then return `ProviderAuthFailed` |
| AC-ANTH003 | Claude-3-5-Sonnet | Given "claude-3-5-sonnet-20240620", When `chat()`, Then use correct model |
| AC-ANTH004 | Streaming | Given streaming enabled, When `complete_streaming()`, Then receive chunks |

### Ollama Provider

| ID | Criterion | Given-When-Then |
|----|-----------|------------------|
| AC-OLLAMA001 | Local server available | Given Ollama running on localhost:11434, When `chat()`, Then return response |
| AC-OLLAMA002 | Local server unavailable | Given Ollama not running, When `chat()`, Then return `ProviderUnavailable` |
| AC-OLLAMA003 | Custom model | Given "llama3" model, When `chat()`, Then use correct model |
| AC-OLLAMA004 | Streaming | Given streaming enabled, When `complete_streaming()`, Then receive chunks |

### Azure OpenAI Provider

| ID | Criterion | Given-When-Then |
|----|-----------|------------------|
| AC-AZURE001 | Valid endpoint + key | Given valid endpoint and API key, When `chat()`, Then return response |
| AC-AZURE002 | Missing endpoint | Given missing endpoint, When `chat()`, Then return error |
| AC-AZURE003 | API version | Given "2024-02-15-preview", When `chat()`, Then use correct API version |

### Retry Logic

| ID | Criterion | Given-When-Then |
|----|-----------|------------------|
| AC-RETRY001 | Success on first try | Given operation succeeds first time, When `with_retry()`, Then return success immediately |
| AC-RETRY002 | Retry on rate limit | Given rate limit error, When `with_retry()` with max_attempts=3, Then retry up to 3 times |
| AC-RETRY003 | Backoff increases | Given retries, When operation fails, Then backoff doubles each retry |
| AC-RETRY004 | Max attempts exceeded | Given all retries fail, When `with_retry()`, Then return final error |
| AC-RETRY005 | No retry for auth errors | Given `ProviderAuthFailed`, When `with_retry()`, Then do NOT retry (fail fast) |

### Budget Tracking

| ID | Criterion | Given-When-Then |
|----|-----------|------------------|
| AC-BUDGET001 | Track token usage | Given `BudgetTracker` with limit=100000, When tokens used=50000, Then budget 50% remaining |
| AC-BUDGET002 | Exceeded limit | Given budget exhausted, When `chat()`, Then return `BudgetExceededError` |
| AC-BUDGET003 | Calculate cost | Given 1000 input + 500 output tokens at $0.002/1K, Then cost = $3.00 |
| AC-BUDGET004 | Reset budget | Given budget tracking, When reset called, Then budget cleared |

### Auth Management

| ID | Criterion | Test Method |
|----|-----------|-------------|
| AC-AUTH001 | API key stored securely | Verify API key not in logs | Security test |
| AC-AUTH002 | Keychain retrieval | Given keychain has key, When resolve_keychain_secret(), Then return key | Unit test |
| AC-AUTH003 | OAuth flow completion | Given OAuth provider, When auth flow completes, Then tokens stored | Integration test |
| AC-AUTH004 | Token refresh | Given expired token, When request, Then refresh token and retry | Integration test |

### Model Selection

| ID | Criterion | Given-When-Then |
|----|-----------|------------------|
| AC-MODEL001 | Default model | Given no model specified, When `chat()`, Then use provider default |
| AC-MODEL002 | Explicit model | Given model="gpt-4o", When `chat()`, Then use gpt-4o |
| AC-MODEL003 | Invalid model | Given model="nonexistent", When `chat()`, Then return `InvalidModel` |
| AC-MODEL004 | Model capabilities | Given model info requested, When `get_model()`, Then return context_window, supports_streaming, etc. |

### Message Transformation

| ID | Criterion | Test Method |
|----|-----------|-------------|
| AC-XFORM001 | Convert to provider format | Given ChatMessage, When transform for OpenAI, Then return OpenAI format | Unit test |
| AC-XFORM002 | System message handling | Given system message, When transform, Then place correctly per provider spec | Unit test |
| AC-XFORM003 | Tool results to assistant | Given ToolResult in messages, When transform, Then convert to assistant message | Unit test |

### Streaming

| ID | Criterion | Given-When-Then |
|----|-----------|------------------|
| AC-STREAM001 | Chunk received | Given streaming enabled, When `complete_streaming()`, Then callback called with chunks |
| AC-STREAM002 | Final chunk marked | Given streaming completes, When chunks received, Then final chunk has `done: true` |
| AC-STREAM003 | Error during stream | Given error mid-stream, When streaming, Then callback not called after error |
| AC-STREAM004 | Streaming abort | Given streaming in progress, When abort called, Then streaming stops |

### Catalog / Model Discovery

| ID | Criterion | Given-When-Then |
|----|-----------|------------------|
| AC-CATALOG001 | Fetch models.dev | Given models.dev available, When fetch catalog, Then return provider list |
| AC-CATALOG002 | Cache catalog | Given catalog fetched, When fetch again, Then use cache (no network) |
| AC-CATALOG003 | Cache expiry | Given cache expired, When fetch catalog, Then re-fetch from network |
| AC-CATALOG004 | Provider filter | Given catalog, When filter by "openai", Then return only OpenAI models |

### Performance

| ID | Criterion | Target | Test Method |
|----|-----------|--------|-------------|
| AC-PERF001 | Provider chat latency | P95 < 5s | E2E benchmark |
| AC-PERF002 | Streaming first token | < 500ms to first token | E2E benchmark |
| AC-PERF003 | Concurrent requests | 10+ concurrent chats | Load test |
| AC-PERF004 | Token counting | ±5% accuracy | Unit test |

---

## Cross-References

| Reference | Description |
|-----------|-------------|
| [Provider/Model System PRD](../../system/10-provider-model-system.md) | System-level provider architecture |
| [Glossary: Provider](../../system/01_glossary.md#provider) | Provider terminology |
| [Glossary: Model](../../system/01_glossary.md#model) | Model terminology |
| [ERROR_CODE_CATALOG.md](../../ERROR_CODE_CATALOG.md#3xxx) | Provider error codes (3001-3006) |
| [auth.md](../auth.md) | Authentication system |
```
