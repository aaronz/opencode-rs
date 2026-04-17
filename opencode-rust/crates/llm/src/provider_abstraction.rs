use std::collections::HashMap;
use std::sync::Arc;

use serde::{Deserialize, Serialize};

use crate::error::LlmError;
use crate::google::GoogleProvider;
use crate::lm_studio::LmStudioProvider;
use crate::provider::{ChatMessage, ChatResponse, Model, Provider, StreamingCallback};
use crate::{AnthropicProvider, OllamaProvider, OpenAiProvider};

pub mod sealed {
    pub trait Sealed {}
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ProviderIdentity {
    pub provider_type: String,
    pub model: Option<String>,
    pub variant: Option<String>,
    pub reasoning_budget: Option<ReasoningBudget>,
}

impl ProviderIdentity {
    pub fn with_variant(mut self, variant: impl Into<String>) -> Self {
        self.variant = Some(variant.into());
        self
    }

    #[deprecated(
        since = "0.1.0",
        note = "Variant/Reasoning budget is experimental. Enable the `experimental-variant-reasoning` feature flag to use."
    )]
    pub fn with_reasoning_budget(mut self, budget: ReasoningBudget) -> Self {
        self.reasoning_budget = Some(budget);
        self
    }

    #[deprecated(
        since = "0.1.0",
        note = "Variant/Reasoning budget is experimental. Enable the `experimental-variant-reasoning` feature flag to use."
    )]
    pub fn get_reasoning_config(&self) -> Option<ProviderReasoningConfig> {
        self.reasoning_budget?.for_provider(&self.provider_type)
    }
}

/// ⚠️ **Experimental Feature**: Variant/Reasoning Budget support is experimental and subject to change.
///
/// This enum defines reasoning budget levels that can be applied to LLM providers that support
/// extended reasoning capabilities (e.g., Anthropic's thinking budgets, OpenAI's reasoning effort).
///
/// ## Provider Support
///
/// - **Anthropic**: Supports `Low`, `Medium`, `High`, `Max` via `thinking` config
/// - **OpenAI**: Supports `Minimal`, `Low`, `Medium`, `High`, `XHigh` via `reasoning_effort`
/// - **Google**: Supports `Low`, `Medium`, `High` via `thinking_throttle`
///
/// ## Usage Warning
///
/// - API surface may change in future releases
/// - Not all providers support all budget levels
/// - Feature availability depends on provider API capabilities
///
/// ## Feature Flag
///
/// This feature requires the `experimental-variant-reasoning` feature flag to be enabled.
/// Without the feature flag, the `with_reasoning_budget` and `get_reasoning_config` methods
/// will emit deprecation warnings.
///
/// # Example
///
/// ```ignore
/// use opencode_llm::{ProviderIdentity, ReasoningBudget};
///
/// let identity = ProviderIdentity::openai("o1-preview")
///     .with_reasoning_budget(ReasoningBudget::High);
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ReasoningBudget {
    None,
    Minimal,
    Low,
    Medium,
    High,
    XHigh,
    Max,
}

impl ReasoningBudget {
    #[allow(clippy::should_implement_trait)]
    #[deprecated(
        since = "0.1.0",
        note = "Variant/Reasoning budget is experimental. Enable the `experimental-variant-reasoning` feature flag to use."
    )]
    pub fn from_str(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "none" => Some(ReasoningBudget::None),
            "minimal" => Some(ReasoningBudget::Minimal),
            "low" => Some(ReasoningBudget::Low),
            "medium" => Some(ReasoningBudget::Medium),
            "high" => Some(ReasoningBudget::High),
            "xhigh" => Some(ReasoningBudget::XHigh),
            "max" => Some(ReasoningBudget::Max),
            _ => None,
        }
    }

    /// ⚠️ **Experimental**: This method is part of the experimental variant/reasoning budget feature.
    ///
    /// Converts this `ReasoningBudget` to a provider-specific `ProviderReasoningConfig`.
    ///
    /// Not all providers support all budget levels. Returns `None` if the provider
    /// does not support reasoning budgets or if the specific budget level is not
    /// supported by that provider.
    pub fn for_provider(&self, provider_type: &str) -> Option<ProviderReasoningConfig> {
        match provider_type {
            "anthropic" => match self {
                ReasoningBudget::None => {
                    Some(ProviderReasoningConfig::Anthropic { thinking: None })
                }
                ReasoningBudget::Low => Some(ProviderReasoningConfig::Anthropic {
                    thinking: Some(AnthropicThinkingConfig::Low),
                }),
                ReasoningBudget::High | ReasoningBudget::Medium => {
                    Some(ProviderReasoningConfig::Anthropic {
                        thinking: Some(AnthropicThinkingConfig::High),
                    })
                }
                ReasoningBudget::Max => Some(ProviderReasoningConfig::Anthropic {
                    thinking: Some(AnthropicThinkingConfig::Max),
                }),
                ReasoningBudget::Minimal => Some(ProviderReasoningConfig::Anthropic {
                    thinking: Some(AnthropicThinkingConfig::Low),
                }),
                ReasoningBudget::XHigh => Some(ProviderReasoningConfig::Anthropic {
                    thinking: Some(AnthropicThinkingConfig::Max),
                }),
            },
            "openai" => match self {
                ReasoningBudget::None => Some(ProviderReasoningConfig::OpenAI {
                    reasoning_effort: None,
                }),
                ReasoningBudget::Minimal => Some(ProviderReasoningConfig::OpenAI {
                    reasoning_effort: Some("minimal".to_string()),
                }),
                ReasoningBudget::Low => Some(ProviderReasoningConfig::OpenAI {
                    reasoning_effort: Some("low".to_string()),
                }),
                ReasoningBudget::Medium => Some(ProviderReasoningConfig::OpenAI {
                    reasoning_effort: Some("medium".to_string()),
                }),
                ReasoningBudget::High => Some(ProviderReasoningConfig::OpenAI {
                    reasoning_effort: Some("high".to_string()),
                }),
                ReasoningBudget::XHigh => Some(ProviderReasoningConfig::OpenAI {
                    reasoning_effort: Some("xhigh".to_string()),
                }),
                ReasoningBudget::Max => Some(ProviderReasoningConfig::OpenAI {
                    reasoning_effort: Some("xhigh".to_string()),
                }),
            },
            "google" => match self {
                ReasoningBudget::None => Some(ProviderReasoningConfig::Google {
                    thinking_throttle: Some("none".to_string()),
                }),
                ReasoningBudget::Low => Some(ProviderReasoningConfig::Google {
                    thinking_throttle: Some("low".to_string()),
                }),
                ReasoningBudget::Medium | ReasoningBudget::High | ReasoningBudget::XHigh => {
                    Some(ProviderReasoningConfig::Google {
                        thinking_throttle: Some("high".to_string()),
                    })
                }
                ReasoningBudget::Max => Some(ProviderReasoningConfig::Google {
                    thinking_throttle: Some("high".to_string()),
                }),
                ReasoningBudget::Minimal => Some(ProviderReasoningConfig::Google {
                    thinking_throttle: Some("low".to_string()),
                }),
            },
            _ => None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ProviderReasoningConfig {
    Anthropic {
        thinking: Option<AnthropicThinkingConfig>,
    },
    OpenAI {
        reasoning_effort: Option<String>,
    },
    Google {
        thinking_throttle: Option<String>,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AnthropicThinkingConfig {
    Low,
    High,
    Max,
}

impl ProviderIdentity {
    pub fn new(provider_type: &str, model: Option<&str>) -> Self {
        Self {
            provider_type: provider_type.to_string(),
            model: model.map(|s| s.to_string()),
            variant: None,
            reasoning_budget: None,
        }
    }

    pub fn openai(model: &str) -> Self {
        Self::new("openai", Some(model))
    }

    pub fn anthropic(model: &str) -> Self {
        Self::new("anthropic", Some(model))
    }

    pub fn google(model: &str) -> Self {
        Self::new("google", Some(model))
    }

    pub fn ollama(model: &str) -> Self {
        Self::new("ollama", Some(model))
    }

    pub fn lmstudio(model: &str) -> Self {
        Self::new("lmstudio", Some(model))
    }

    pub fn local(model: &str) -> Self {
        Self::new("local", Some(model))
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum ProviderSpec {
    #[serde(rename = "openai")]
    OpenAI {
        api_key: String,
        model: String,
        base_url: Option<String>,
    },
    #[serde(rename = "anthropic")]
    Anthropic {
        api_key: String,
        model: String,
        base_url: Option<String>,
    },
    #[serde(rename = "google")]
    Google { api_key: String, model: String },
    #[serde(rename = "ollama")]
    Ollama {
        base_url: Option<String>,
        model: String,
    },
    #[serde(rename = "lmstudio")]
    LmStudio {
        base_url: Option<String>,
        model: String,
    },
    #[serde(rename = "local")]
    LocalInference { base_url: String, model: String },
    #[serde(rename = "azure")]
    Azure {
        api_key: String,
        endpoint: String,
        deployment: String,
        api_version: Option<String>,
    },
    #[serde(rename = "openrouter")]
    OpenRouter {
        api_key: String,
        model: String,
        base_url: Option<String>,
    },
    #[serde(rename = "mistral")]
    Mistral { api_key: String, model: String },
    #[serde(rename = "groq")]
    Groq { api_key: String, model: String },
}

impl ProviderSpec {
    pub fn provider_type(&self) -> &str {
        match self {
            Self::OpenAI { .. } => "openai",
            Self::Anthropic { .. } => "anthropic",
            Self::Google { .. } => "google",
            Self::Ollama { .. } => "ollama",
            Self::LmStudio { .. } => "lmstudio",
            Self::LocalInference { .. } => "local",
            Self::Azure { .. } => "azure",
            Self::OpenRouter { .. } => "openrouter",
            Self::Mistral { .. } => "mistral",
            Self::Groq { .. } => "groq",
        }
    }

    pub fn model(&self) -> &str {
        match self {
            Self::OpenAI { model, .. } => model,
            Self::Anthropic { model, .. } => model,
            Self::Google { model, .. } => model,
            Self::Ollama { model, .. } => model,
            Self::LmStudio { model, .. } => model,
            Self::LocalInference { model, .. } => model,
            Self::Azure { deployment, .. } => deployment,
            Self::OpenRouter { model, .. } => model,
            Self::Mistral { model, .. } => model,
            Self::Groq { model, .. } => model,
        }
    }
}

#[derive(Debug, Clone)]
pub struct DynProviderError {
    pub message: String,
}

impl std::fmt::Display for DynProviderError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "DynProviderError: {}", self.message)
    }
}

impl std::error::Error for DynProviderError {}

pub struct DynProvider {
    inner: Arc<dyn Provider>,
    identity: ProviderIdentity,
}

impl DynProvider {
    pub fn new<P: Provider + 'static>(provider: P) -> Self {
        let identity = ProviderIdentity::new(
            provider.provider_name(),
            provider.get_models().first().map(|m| m.id.as_str()),
        );
        Self {
            inner: Arc::new(provider),
            identity,
        }
    }

    pub fn with_identity<P: Provider + 'static>(provider: P, identity: ProviderIdentity) -> Self {
        Self {
            inner: Arc::new(provider),
            identity,
        }
    }

    pub fn with_identity_and_inner(
        provider: Arc<dyn Provider>,
        identity: ProviderIdentity,
    ) -> Self {
        Self {
            inner: provider,
            identity,
        }
    }

    pub fn identity(&self) -> &ProviderIdentity {
        &self.identity
    }

    pub async fn complete(
        &self,
        prompt: &str,
        context: Option<&str>,
    ) -> Result<String, opencode_core::OpenCodeError> {
        self.inner.complete(prompt, context).await
    }

    pub async fn complete_streaming(
        &self,
        prompt: &str,
        callback: StreamingCallback,
    ) -> Result<(), opencode_core::OpenCodeError> {
        self.inner.complete_streaming(prompt, callback).await
    }

    pub async fn chat(
        &self,
        messages: &[ChatMessage],
    ) -> Result<ChatResponse, opencode_core::OpenCodeError> {
        self.inner.chat(messages).await
    }

    pub fn get_models(&self) -> Vec<Model> {
        if let Some(ref model) = self.identity.model {
            vec![Model::new(model, model)]
        } else {
            vec![]
        }
    }

    pub fn provider_name(&self) -> &str {
        &self.identity.provider_type
    }

    pub fn variant(&self) -> Option<&str> {
        self.identity.variant.as_deref()
    }

    pub fn reasoning_budget(&self) -> Option<ReasoningBudget> {
        self.identity.reasoning_budget
    }

    #[allow(deprecated)]
    pub fn reasoning_config(&self) -> Option<ProviderReasoningConfig> {
        self.identity.get_reasoning_config()
    }
}

impl std::fmt::Debug for DynProvider {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("DynProvider")
            .field("identity", &self.identity)
            .finish()
    }
}

#[derive(Debug, Clone)]
pub struct ProviderConfig {
    pub spec: ProviderSpec,
    pub reasoning_budget: Option<ReasoningBudget>,
    pub variant: Option<String>,
}

impl ProviderConfig {
    pub fn from_spec(spec: ProviderSpec) -> Self {
        Self {
            spec,
            reasoning_budget: None,
            variant: None,
        }
    }

    pub fn from_identity(identity: &ProviderIdentity) -> Option<Self> {
        let spec = match identity.provider_type.as_str() {
            "openai" => ProviderSpec::OpenAI {
                api_key: std::env::var("OPENAI_API_KEY").unwrap_or_default(),
                model: identity
                    .model
                    .clone()
                    .unwrap_or_else(|| "gpt-4o".to_string()),
                base_url: None,
            },
            "anthropic" => ProviderSpec::Anthropic {
                api_key: std::env::var("ANTHROPIC_API_KEY").unwrap_or_default(),
                model: identity
                    .model
                    .clone()
                    .unwrap_or_else(|| "claude-3-5-sonnet-20241022".to_string()),
                base_url: None,
            },
            "google" => ProviderSpec::Google {
                api_key: std::env::var("GOOGLE_API_KEY").unwrap_or_default(),
                model: identity
                    .model
                    .clone()
                    .unwrap_or_else(|| "gemini-1.5-pro".to_string()),
            },
            "ollama" => ProviderSpec::Ollama {
                base_url: Some("http://localhost:11434".to_string()),
                model: identity
                    .model
                    .clone()
                    .unwrap_or_else(|| "llama3".to_string()),
            },
            "lmstudio" => ProviderSpec::LmStudio {
                base_url: Some("http://localhost:1234".to_string()),
                model: identity
                    .model
                    .clone()
                    .unwrap_or_else(|| "llama3".to_string()),
            },
            "local" => ProviderSpec::LocalInference {
                base_url: "http://localhost:8080".to_string(),
                model: identity
                    .model
                    .clone()
                    .unwrap_or_else(|| "llama3".to_string()),
            },
            _ => return None,
        };

        Some(Self {
            spec,
            reasoning_budget: identity.reasoning_budget,
            variant: identity.variant.clone(),
        })
    }
}

pub trait ProviderFactory: Send + Sync + sealed::Sealed {
    fn name(&self) -> &str;
    fn create(&self, config: &ProviderConfig) -> Result<DynProvider, LlmError>;
    fn supports(&self, spec: &ProviderSpec) -> bool;
}

pub struct ProviderManager {
    factories: HashMap<String, Box<dyn ProviderFactory>>,
    default_provider: Option<String>,
}

impl ProviderManager {
    pub fn new() -> Self {
        let mut manager = Self {
            factories: HashMap::new(),
            default_provider: None,
        };
        manager.register_default_factories();
        manager
    }

    fn register_default_factories(&mut self) {
        self.register_factory(Box::new(OpenAIProviderFactory));
        self.register_factory(Box::new(AnthropicProviderFactory));
        self.register_factory(Box::new(GoogleProviderFactory));
        self.register_factory(Box::new(OllamaProviderFactory));
        self.register_factory(Box::new(LmStudioProviderFactory));
        self.register_factory(Box::new(LocalInferenceProviderFactory));
    }

    pub fn register_factory(&mut self, factory: Box<dyn ProviderFactory>) {
        self.factories.insert(factory.name().to_string(), factory);
    }

    pub fn set_default(&mut self, provider_type: &str) {
        self.default_provider = Some(provider_type.to_string());
    }

    pub fn get_default(&self) -> Option<&str> {
        self.default_provider.as_deref()
    }

    pub fn create_provider(&self, spec: &ProviderSpec) -> Result<DynProvider, LlmError> {
        let config = ProviderConfig::from_spec(spec.clone());
        let provider_type = config.spec.provider_type();

        let factory = match self.factories.get(provider_type) {
            Some(f) => f,
            None => {
                return Err(LlmError::Provider(format!(
                    "Unknown provider type: {}",
                    provider_type
                )))
            }
        };

        if !factory.supports(&config.spec) {
            return Err(LlmError::Provider(format!(
                "Factory '{}' does not support this configuration",
                provider_type
            )));
        }

        factory.create(&config)
    }

    pub fn create_provider_fallback(&self, spec: &ProviderSpec) -> Result<DynProvider, LlmError> {
        let config = ProviderConfig::from_spec(spec.clone());
        let provider_type = config.spec.provider_type();

        if let Some(factory) = self.factories.get(provider_type) {
            if factory.supports(spec) {
                return factory.create(&config);
            }
        }

        DynamicProviderFactory::new().create(&config)
    }

    pub fn create_provider_by_identity(
        &self,
        identity: &ProviderIdentity,
    ) -> Result<DynProvider, LlmError> {
        let config = ProviderConfig::from_identity(identity).ok_or_else(|| {
            LlmError::Provider(format!(
                "Cannot create provider '{}' from identity - missing configuration",
                identity.provider_type
            ))
        })?;

        let factory = self.factories.get(&identity.provider_type).ok_or_else(|| {
            LlmError::Provider(format!("Unknown provider type: {}", identity.provider_type))
        })?;

        factory.create(&config)
    }

    pub fn list_providers(&self) -> Vec<String> {
        self.factories.keys().cloned().collect()
    }

    pub fn has_provider(&self, provider_type: &str) -> bool {
        self.factories.contains_key(provider_type)
    }
}

impl Default for ProviderManager {
    fn default() -> Self {
        Self::new()
    }
}

pub struct OpenAIProviderFactory;

impl sealed::Sealed for OpenAIProviderFactory {}
impl ProviderFactory for OpenAIProviderFactory {
    fn name(&self) -> &str {
        "openai"
    }

    fn create(&self, config: &ProviderConfig) -> Result<DynProvider, LlmError> {
        match &config.spec {
            ProviderSpec::OpenAI { api_key, model, .. } => {
                let mut provider = OpenAiProvider::new(api_key.clone(), model.clone());

                if let Some(ProviderReasoningConfig::OpenAI {
                    reasoning_effort: Some(effort),
                }) = config
                    .reasoning_budget
                    .as_ref()
                    .and_then(|rb| rb.for_provider("openai"))
                {
                    provider = provider.with_reasoning_effort(effort.clone());
                }

                let mut identity = ProviderIdentity::openai(model);
                identity.reasoning_budget = config.reasoning_budget;
                identity.variant = config.variant.clone();
                Ok(DynProvider::with_identity(provider, identity))
            }
            _ => Err(LlmError::Provider(format!(
                "OpenAI factory cannot create provider from {:?}",
                config.spec
            ))),
        }
    }

    fn supports(&self, spec: &ProviderSpec) -> bool {
        matches!(spec, ProviderSpec::OpenAI { .. })
    }
}

pub struct AnthropicProviderFactory;

impl sealed::Sealed for AnthropicProviderFactory {}
impl ProviderFactory for AnthropicProviderFactory {
    fn name(&self) -> &str {
        "anthropic"
    }

    fn create(&self, config: &ProviderConfig) -> Result<DynProvider, LlmError> {
        match &config.spec {
            ProviderSpec::Anthropic { api_key, model, .. } => {
                let mut provider = AnthropicProvider::new(api_key.clone(), model.clone());

                if let Some(ProviderReasoningConfig::Anthropic {
                    thinking: Some(thinking_config),
                }) = config
                    .reasoning_budget
                    .as_ref()
                    .and_then(|rb| rb.for_provider("anthropic"))
                {
                    provider = provider.with_thinking_budget(thinking_config);
                }

                let mut identity = ProviderIdentity::anthropic(model);
                identity.reasoning_budget = config.reasoning_budget;
                identity.variant = config.variant.clone();
                Ok(DynProvider::with_identity(provider, identity))
            }
            _ => Err(LlmError::Provider(format!(
                "Anthropic factory cannot create provider from {:?}",
                config.spec
            ))),
        }
    }

    fn supports(&self, spec: &ProviderSpec) -> bool {
        matches!(spec, ProviderSpec::Anthropic { .. })
    }
}

pub struct GoogleProviderFactory;

impl sealed::Sealed for GoogleProviderFactory {}
impl ProviderFactory for GoogleProviderFactory {
    fn name(&self) -> &str {
        "google"
    }

    fn create(&self, config: &ProviderConfig) -> Result<DynProvider, LlmError> {
        match &config.spec {
            ProviderSpec::Google { api_key, model } => {
                let mut provider = GoogleProvider::new(api_key.clone(), model.clone());

                if let Some(ProviderReasoningConfig::Google {
                    thinking_throttle: Some(throttle),
                }) = config
                    .reasoning_budget
                    .as_ref()
                    .and_then(|rb| rb.for_provider("google"))
                {
                    provider = provider.with_thinking_throttle(throttle.clone());
                }

                let mut identity = ProviderIdentity::google(model);
                identity.reasoning_budget = config.reasoning_budget;
                identity.variant = config.variant.clone();
                Ok(DynProvider::with_identity(provider, identity))
            }
            _ => Err(LlmError::Provider(format!(
                "Google factory cannot create provider from {:?}",
                config.spec
            ))),
        }
    }

    fn supports(&self, spec: &ProviderSpec) -> bool {
        matches!(spec, ProviderSpec::Google { .. })
    }
}

pub struct OllamaProviderFactory;

impl sealed::Sealed for OllamaProviderFactory {}
impl ProviderFactory for OllamaProviderFactory {
    fn name(&self) -> &str {
        "ollama"
    }

    fn create(&self, config: &ProviderConfig) -> Result<DynProvider, LlmError> {
        match &config.spec {
            ProviderSpec::Ollama { base_url, model } => {
                let provider = OllamaProvider::new(model.clone(), base_url.clone());
                let mut identity = ProviderIdentity::ollama(model);
                identity.reasoning_budget = config.reasoning_budget;
                identity.variant = config.variant.clone();
                Ok(DynProvider::with_identity(provider, identity))
            }
            _ => Err(LlmError::Provider(format!(
                "Ollama factory cannot create provider from {:?}",
                config.spec
            ))),
        }
    }

    fn supports(&self, spec: &ProviderSpec) -> bool {
        matches!(spec, ProviderSpec::Ollama { .. })
    }
}

pub struct LmStudioProviderFactory;

impl sealed::Sealed for LmStudioProviderFactory {}
impl ProviderFactory for LmStudioProviderFactory {
    fn name(&self) -> &str {
        "lmstudio"
    }

    fn create(&self, config: &ProviderConfig) -> Result<DynProvider, LlmError> {
        match &config.spec {
            ProviderSpec::LmStudio { base_url, model } => {
                let provider = LmStudioProvider::new(model.clone(), base_url.clone());
                let mut identity = ProviderIdentity::new("lmstudio", Some(model));
                identity.reasoning_budget = config.reasoning_budget;
                identity.variant = config.variant.clone();
                Ok(DynProvider::with_identity(provider, identity))
            }
            _ => Err(LlmError::Provider(format!(
                "LmStudio factory cannot create provider from {:?}",
                config.spec
            ))),
        }
    }

    fn supports(&self, spec: &ProviderSpec) -> bool {
        matches!(spec, ProviderSpec::LmStudio { .. })
    }
}

pub struct LocalInferenceProviderFactory;

impl sealed::Sealed for LocalInferenceProviderFactory {}
impl ProviderFactory for LocalInferenceProviderFactory {
    fn name(&self) -> &str {
        "local"
    }

    fn create(&self, config: &ProviderConfig) -> Result<DynProvider, LlmError> {
        match &config.spec {
            ProviderSpec::LocalInference { base_url, model } => {
                let provider = LmStudioProvider::new(model.clone(), Some(base_url.clone()));
                let mut identity = ProviderIdentity::new("local", Some(model));
                identity.reasoning_budget = config.reasoning_budget;
                identity.variant = config.variant.clone();
                Ok(DynProvider::with_identity(provider, identity))
            }
            _ => Err(LlmError::Provider(format!(
                "LocalInference factory cannot create provider from {:?}",
                config.spec
            ))),
        }
    }

    fn supports(&self, spec: &ProviderSpec) -> bool {
        matches!(spec, ProviderSpec::LocalInference { .. })
    }
}

pub struct DynamicProviderFactory;

impl DynamicProviderFactory {
    pub fn new() -> Self {
        Self
    }
}

impl Default for DynamicProviderFactory {
    fn default() -> Self {
        Self::new()
    }
}

impl sealed::Sealed for DynamicProviderFactory {}
impl ProviderFactory for DynamicProviderFactory {
    fn name(&self) -> &str {
        "dynamic"
    }

    fn create(&self, config: &ProviderConfig) -> Result<DynProvider, LlmError> {
        let provider_type = config.spec.provider_type();
        let model = config.spec.model().to_string();

        let base_url = match &config.spec {
            ProviderSpec::OpenAI { base_url, .. } => base_url
                .clone()
                .unwrap_or_else(|| "https://api.openai.com/v1".to_string()),
            ProviderSpec::Anthropic { base_url, .. } => base_url
                .clone()
                .unwrap_or_else(|| "https://api.anthropic.com".to_string()),
            ProviderSpec::Google { .. } => "https://api.google.com".to_string(),
            ProviderSpec::Ollama { base_url, .. } => base_url
                .clone()
                .unwrap_or_else(|| "http://localhost:11434".to_string()),
            ProviderSpec::LmStudio { base_url, .. } => base_url
                .clone()
                .unwrap_or_else(|| "http://localhost:1234".to_string()),
            ProviderSpec::LocalInference { base_url, .. } => base_url.clone(),
            ProviderSpec::Azure { endpoint, .. } => endpoint.clone(),
            ProviderSpec::OpenRouter { base_url, .. } => base_url
                .clone()
                .unwrap_or_else(|| "https://openrouter.ai/api/v1".to_string()),
            ProviderSpec::Mistral { .. } => "https://api.mistral.ai".to_string(),
            ProviderSpec::Groq { .. } => "https://api.groq.com".to_string(),
        };

        let adapter = crate::provider_adapter::OpenAICompatibleAdapter::new(
            crate::auth::ProviderAuthConfig::new(
                provider_type.to_string(),
                format!("{}/chat/completions", base_url.trim_end_matches('/')),
                crate::auth::AuthStrategy::BearerApiKey { header_name: None },
            ),
            model.clone(),
        );

        let mut identity = ProviderIdentity::new(provider_type, Some(&model));
        identity.reasoning_budget = config.reasoning_budget;
        identity.variant = config.variant.clone();
        Ok(DynProvider::with_identity(adapter, identity))
    }

    fn supports(&self, _spec: &ProviderSpec) -> bool {
        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_provider_identity_creation() {
        let identity = ProviderIdentity::openai("gpt-4");
        assert_eq!(identity.provider_type, "openai");
        assert_eq!(identity.model, Some("gpt-4".to_string()));
    }

    #[test]
    fn test_provider_identity_equality() {
        let id1 = ProviderIdentity::anthropic("claude-3");
        let id2 = ProviderIdentity::anthropic("claude-3");
        let id3 = ProviderIdentity::openai("claude-3");

        assert_eq!(id1, id2);
        assert_ne!(id1, id3);
    }

    #[test]
    fn test_provider_spec_provider_type() {
        let spec = ProviderSpec::OpenAI {
            api_key: "test".to_string(),
            model: "gpt-4".to_string(),
            base_url: None,
        };
        assert_eq!(spec.provider_type(), "openai");
        assert_eq!(spec.model(), "gpt-4");
    }

    #[test]
    fn test_provider_manager_creation() {
        let manager = ProviderManager::new();
        assert!(manager.has_provider("openai"));
        assert!(manager.has_provider("anthropic"));
        assert!(manager.has_provider("google"));
        assert!(manager.has_provider("ollama"));
        assert!(!manager.has_provider("nonexistent"));
    }

    #[test]
    fn test_provider_manager_list_providers() {
        let manager = ProviderManager::new();
        let providers = manager.list_providers();
        assert!(providers.contains(&"openai".to_string()));
        assert!(providers.contains(&"anthropic".to_string()));
    }

    #[test]
    fn test_provider_manager_create_openai() {
        let manager = ProviderManager::new();
        let spec = ProviderSpec::OpenAI {
            api_key: "test-key".to_string(),
            model: "gpt-4".to_string(),
            base_url: None,
        };

        let result = manager.create_provider(&spec);
        assert!(result.is_ok());
        let provider = result.unwrap();
        assert_eq!(provider.provider_name(), "openai");
    }

    #[test]
    fn test_provider_manager_create_anthropic() {
        let manager = ProviderManager::new();
        let spec = ProviderSpec::Anthropic {
            api_key: "test-key".to_string(),
            model: "claude-3".to_string(),
            base_url: None,
        };

        let result = manager.create_provider(&spec);
        assert!(result.is_ok());
        let provider = result.unwrap();
        assert_eq!(provider.provider_name(), "anthropic");
    }

    #[test]
    fn test_provider_manager_create_ollama() {
        let manager = ProviderManager::new();
        let spec = ProviderSpec::Ollama {
            base_url: Some("http://localhost:11434".to_string()),
            model: "llama2".to_string(),
        };

        let result = manager.create_provider(&spec);
        assert!(result.is_ok());
        let provider = result.unwrap();
        assert_eq!(provider.provider_name(), "ollama");
    }

    #[test]
    fn test_provider_manager_default() {
        let mut manager = ProviderManager::new();
        manager.set_default("openai");
        assert_eq!(manager.get_default(), Some("openai"));
    }

    #[test]
    fn test_provider_manager_unknown_provider() {
        let manager = ProviderManager::new();
        let spec = ProviderSpec::OpenAI {
            api_key: "test".to_string(),
            model: "gpt-4".to_string(),
            base_url: None,
        };

        let result = manager.create_provider(&spec);
        assert!(result.is_ok());
    }

    #[test]
    fn test_dyn_provider_creation() {
        let spec = ProviderSpec::OpenAI {
            api_key: "test-key".to_string(),
            model: "gpt-4".to_string(),
            base_url: None,
        };

        let manager = ProviderManager::new();
        let provider = manager.create_provider(&spec).unwrap();

        assert_eq!(provider.identity().provider_type, "openai");
        assert_eq!(provider.identity().model, Some("gpt-4".to_string()));
    }

    #[test]
    fn test_dyn_provider_get_models() {
        let spec = ProviderSpec::OpenAI {
            api_key: "test-key".to_string(),
            model: "gpt-4".to_string(),
            base_url: None,
        };

        let manager = ProviderManager::new();
        let provider = manager.create_provider(&spec).unwrap();

        let models = provider.get_models();
        assert!(!models.is_empty());
    }

    #[test]
    fn test_provider_spec_serialization() {
        let spec = ProviderSpec::OpenAI {
            api_key: "test-key".to_string(),
            model: "gpt-4".to_string(),
            base_url: Some("https://api.openai.com".to_string()),
        };

        let json = serde_json::to_string(&spec).unwrap();
        assert!(json.contains("\"type\":\"openai\""));

        let deserialized: ProviderSpec = serde_json::from_str(&json).unwrap();
        match deserialized {
            ProviderSpec::OpenAI {
                api_key,
                model,
                base_url,
            } => {
                assert_eq!(api_key, "test-key");
                assert_eq!(model, "gpt-4");
                assert_eq!(base_url, Some("https://api.openai.com".to_string()));
            }
            _ => panic!("Expected OpenAI variant"),
        }
    }

    #[test]
    fn test_factory_supports() {
        let factory = OpenAIProviderFactory {};

        let openai_spec = ProviderSpec::OpenAI {
            api_key: "test".to_string(),
            model: "gpt-4".to_string(),
            base_url: None,
        };

        let anthropic_spec = ProviderSpec::Anthropic {
            api_key: "test".to_string(),
            model: "claude-3".to_string(),
            base_url: None,
        };

        assert!(factory.supports(&openai_spec));
        assert!(!factory.supports(&anthropic_spec));
    }

    #[test]
    fn test_create_provider_by_identity_with_model() {
        let manager = ProviderManager::new();

        let identity = ProviderIdentity::openai("gpt-4o");
        let result = manager.create_provider_by_identity(&identity);
        assert!(result.is_ok());
    }

    #[test]
    fn test_register_custom_factory() {
        struct CustomFactory;

        impl sealed::Sealed for CustomFactory {}
        impl ProviderFactory for CustomFactory {
            fn name(&self) -> &str {
                "custom"
            }

            fn create(&self, config: &ProviderConfig) -> Result<DynProvider, LlmError> {
                match &config.spec {
                    ProviderSpec::OpenAI { api_key, model, .. } => {
                        let provider = OpenAiProvider::new(api_key.clone(), model.clone());
                        Ok(DynProvider::new(provider))
                    }
                    _ => Err(LlmError::Provider(
                        "Custom factory only supports OpenAI".to_string(),
                    )),
                }
            }

            fn supports(&self, spec: &ProviderSpec) -> bool {
                matches!(spec, ProviderSpec::OpenAI { .. })
            }
        }

        let mut manager = ProviderManager::new();
        manager.register_factory(Box::new(CustomFactory));

        assert!(manager.has_provider("custom"));
    }
}
