use crate::catalog::types::ModelStatus;
use crate::provider_filter::ProviderFilter;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};

/// Checks if a model should be visible based on its status.
///
/// Alpha models are hidden by default unless `OPENCODE_ENABLE_EXPERIMENTAL_MODELS`
/// environment variable is set to "true".
fn is_model_visible(status: Option<ModelStatus>) -> bool {
    if status == Some(ModelStatus::Alpha) {
        std::env::var("OPENCODE_ENABLE_EXPERIMENTAL_MODELS")
            .map(|v| v == "true")
            .unwrap_or(false)
    } else {
        true
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelInfo {
    pub name: String,
    pub provider: String,
    pub max_tokens: u32,
    pub max_input_tokens: u32,
    pub supports_functions: bool,
    pub supports_vision: bool,
    pub supports_streaming: bool,
    pub cost_per_1k_tokens: f64,
    pub status: Option<ModelStatus>,
}

pub struct ModelRegistry {
    models: HashMap<String, ModelInfo>,
    provider_filter: Option<ProviderFilter>,
}

impl ModelRegistry {
    pub fn new() -> Self {
        let mut models = HashMap::new();

        models.insert(
            "gpt-4o".to_string(),
            ModelInfo {
                name: "gpt-4o".to_string(),
                provider: "openai".to_string(),
                max_tokens: 16384,
                max_input_tokens: 128000,
                supports_functions: true,
                supports_vision: true,
                supports_streaming: true,
                cost_per_1k_tokens: 0.005,
                status: None,
            },
        );

        models.insert(
            "gpt-4o-mini".to_string(),
            ModelInfo {
                name: "gpt-4o-mini".to_string(),
                provider: "openai".to_string(),
                max_tokens: 16384,
                max_input_tokens: 128000,
                supports_functions: true,
                supports_vision: true,
                supports_streaming: true,
                cost_per_1k_tokens: 0.0015,
                status: None,
            },
        );

        models.insert(
            "gpt-4-turbo".to_string(),
            ModelInfo {
                name: "gpt-4-turbo".to_string(),
                provider: "openai".to_string(),
                max_tokens: 4096,
                max_input_tokens: 128000,
                supports_functions: true,
                supports_vision: true,
                supports_streaming: true,
                cost_per_1k_tokens: 0.01,
                status: None,
            },
        );
        models.insert(
            "claude-sonnet-4-20250514".to_string(),
            ModelInfo {
                name: "claude-sonnet-4-20250514".to_string(),
                provider: "anthropic".to_string(),
                max_tokens: 4096,
                max_input_tokens: 200000,
                supports_functions: false,
                supports_vision: true,
                supports_streaming: true,
                cost_per_1k_tokens: 0.003,
                status: None,
            },
        );

        models.insert(
            "claude-haiku-3".to_string(),
            ModelInfo {
                name: "claude-haiku-3".to_string(),
                provider: "anthropic".to_string(),
                max_tokens: 4096,
                max_input_tokens: 200000,
                supports_functions: false,
                supports_vision: true,
                supports_streaming: true,
                cost_per_1k_tokens: 0.00025,
                status: None,
            },
        );

        models.insert(
            "llama3".to_string(),
            ModelInfo {
                name: "llama3".to_string(),
                provider: "ollama".to_string(),
                max_tokens: 4096,
                max_input_tokens: 8192,
                supports_functions: false,
                supports_vision: false,
                supports_streaming: true,
                cost_per_1k_tokens: 0.0,
                status: None,
            },
        );

        models.insert(
            "codellama".to_string(),
            ModelInfo {
                name: "codellama".to_string(),
                provider: "ollama".to_string(),
                max_tokens: 4096,
                max_input_tokens: 16384,
                supports_functions: false,
                supports_vision: false,
                supports_streaming: true,
                cost_per_1k_tokens: 0.0,
                status: None,
            },
        );

        // Azure OpenAI models
        models.insert(
            "gpt-4o-azure".to_string(),
            ModelInfo {
                name: "gpt-4o-azure".to_string(),
                provider: "azure".to_string(),
                max_tokens: 16384,
                max_input_tokens: 128000,
                supports_functions: true,
                supports_vision: true,
                supports_streaming: true,
                cost_per_1k_tokens: 0.005,
                status: None,
            },
        );

        // Google models
        models.insert(
            "gemini-1.5-pro".to_string(),
            ModelInfo {
                name: "gemini-1.5-pro".to_string(),
                provider: "google".to_string(),
                max_tokens: 8192,
                max_input_tokens: 2000000,
                supports_functions: false,
                supports_vision: true,
                supports_streaming: true,
                cost_per_1k_tokens: 0.00125,
                status: None,
            },
        );

        models.insert(
            "gemini-1.5-flash".to_string(),
            ModelInfo {
                name: "gemini-1.5-flash".to_string(),
                provider: "google".to_string(),
                max_tokens: 8192,
                max_input_tokens: 1000000,
                supports_functions: false,
                supports_vision: true,
                supports_streaming: true,
                cost_per_1k_tokens: 0.0,
                status: None,
            },
        );

        // OpenRouter models
        models.insert(
            "openrouter/gpt-4o".to_string(),
            ModelInfo {
                name: "openrouter/gpt-4o".to_string(),
                provider: "openrouter".to_string(),
                max_tokens: 16384,
                max_input_tokens: 128000,
                supports_functions: true,
                supports_vision: true,
                supports_streaming: true,
                cost_per_1k_tokens: 0.004,
                status: None,
            },
        );

        // Anthropic models
        models.insert(
            "claude-opus-4-20250514".to_string(),
            ModelInfo {
                name: "claude-opus-4-20250514".to_string(),
                provider: "anthropic".to_string(),
                max_tokens: 4096,
                max_input_tokens: 200000,
                supports_functions: false,
                supports_vision: true,
                supports_streaming: true,
                cost_per_1k_tokens: 0.015,
                status: None,
            },
        );

        // XAI models
        models.insert(
            "grok-2".to_string(),
            ModelInfo {
                name: "grok-2".to_string(),
                provider: "xai".to_string(),
                max_tokens: 8192,
                max_input_tokens: 131072,
                supports_functions: false,
                supports_vision: false,
                supports_streaming: true,
                cost_per_1k_tokens: 0.002,
                status: None,
            },
        );

        // Mistral models
        models.insert(
            "mistral-large-latest".to_string(),
            ModelInfo {
                name: "mistral-large-latest".to_string(),
                provider: "mistral".to_string(),
                max_tokens: 16384,
                max_input_tokens: 128000,
                supports_functions: true,
                supports_vision: false,
                supports_streaming: true,
                cost_per_1k_tokens: 0.002,
                status: None,
            },
        );

        // Groq models
        models.insert(
            "llama-3.1-70b-versatile".to_string(),
            ModelInfo {
                name: "llama-3.1-70b-versatile".to_string(),
                provider: "groq".to_string(),
                max_tokens: 8192,
                max_input_tokens: 32768,
                supports_functions: false,
                supports_vision: false,
                supports_streaming: true,
                cost_per_1k_tokens: 0.00059,
                status: None,
            },
        );

        // DeepInfra models
        models.insert(
            "deepinfra/llama-3.1-70b".to_string(),
            ModelInfo {
                name: "deepinfra/llama-3.1-70b".to_string(),
                provider: "deepinfra".to_string(),
                max_tokens: 8192,
                max_input_tokens: 32768,
                supports_functions: false,
                supports_vision: false,
                supports_streaming: true,
                cost_per_1k_tokens: 0.0005,
                status: None,
            },
        );

        // Cerebras models
        models.insert(
            "cerebras/llama-3.1-70b".to_string(),
            ModelInfo {
                name: "cerebras/llama-3.1-70b".to_string(),
                provider: "cerebras".to_string(),
                max_tokens: 8192,
                max_input_tokens: 32768,
                supports_functions: false,
                supports_vision: false,
                supports_streaming: true,
                cost_per_1k_tokens: 0.0006,
                status: None,
            },
        );

        // Cohere models
        models.insert(
            "cohere-command-r-plus".to_string(),
            ModelInfo {
                name: "cohere-command-r-plus".to_string(),
                provider: "cohere".to_string(),
                max_tokens: 4096,
                max_input_tokens: 128000,
                supports_functions: true,
                supports_vision: false,
                supports_streaming: true,
                cost_per_1k_tokens: 0.003,
                status: None,
            },
        );

        // TogetherAI models
        models.insert(
            "togetherai/llama-3.1-70b".to_string(),
            ModelInfo {
                name: "togetherai/llama-3.1-70b".to_string(),
                provider: "togetherai".to_string(),
                max_tokens: 8192,
                max_input_tokens: 32768,
                supports_functions: false,
                supports_vision: false,
                supports_streaming: true,
                cost_per_1k_tokens: 0.00088,
                status: None,
            },
        );

        // Perplexity models
        models.insert(
            "perplexity/llama-3.1-sonar-large".to_string(),
            ModelInfo {
                name: "perplexity/llama-3.1-sonar-large".to_string(),
                provider: "perplexity".to_string(),
                max_tokens: 4096,
                max_input_tokens: 127072,
                supports_functions: false,
                supports_vision: false,
                supports_streaming: true,
                cost_per_1k_tokens: 0.001,
                status: None,
            },
        );

        // GitHub Copilot models
        models.insert(
            "github-copilot/gpt-4o".to_string(),
            ModelInfo {
                name: "github-copilot/gpt-4o".to_string(),
                provider: "github-copilot".to_string(),
                max_tokens: 16384,
                max_input_tokens: 128000,
                supports_functions: true,
                supports_vision: true,
                supports_streaming: true,
                cost_per_1k_tokens: 0.01,
                status: None,
            },
        );

        models.insert(
            "github-copilot/gpt-4o-mini".to_string(),
            ModelInfo {
                name: "github-copilot/gpt-4o-mini".to_string(),
                provider: "github-copilot".to_string(),
                max_tokens: 16384,
                max_input_tokens: 128000,
                supports_functions: true,
                supports_vision: true,
                supports_streaming: true,
                cost_per_1k_tokens: 0.003,
                status: None,
            },
        );

        models.insert(
            "github-copilot/claude-sonnet-4".to_string(),
            ModelInfo {
                name: "github-copilot/claude-sonnet-4".to_string(),
                provider: "github-copilot".to_string(),
                max_tokens: 4096,
                max_input_tokens: 200000,
                supports_functions: false,
                supports_vision: true,
                supports_streaming: true,
                cost_per_1k_tokens: 0.003,
                status: None,
            },
        );

        models.insert(
            "github-copilot/claude-haiku-3".to_string(),
            ModelInfo {
                name: "github-copilot/claude-haiku-3".to_string(),
                provider: "github-copilot".to_string(),
                max_tokens: 4096,
                max_input_tokens: 200000,
                supports_functions: false,
                supports_vision: true,
                supports_streaming: true,
                cost_per_1k_tokens: 0.00025,
                status: None,
            },
        );

        models.insert(
            "github-copilot/o1".to_string(),
            ModelInfo {
                name: "github-copilot/o1".to_string(),
                provider: "github-copilot".to_string(),
                max_tokens: 32768,
                max_input_tokens: 128000,
                supports_functions: false,
                supports_vision: false,
                supports_streaming: false,
                cost_per_1k_tokens: 0.015,
                status: None,
            },
        );

        models.insert(
            "github-copilot/o1-mini".to_string(),
            ModelInfo {
                name: "github-copilot/o1-mini".to_string(),
                provider: "github-copilot".to_string(),
                max_tokens: 65536,
                max_input_tokens: 131072,
                supports_functions: false,
                supports_vision: false,
                supports_streaming: false,
                cost_per_1k_tokens: 0.007,
                status: None,
            },
        );

        models.insert(
            "github-copilot/o1-preview".to_string(),
            ModelInfo {
                name: "github-copilot/o1-preview".to_string(),
                provider: "github-copilot".to_string(),
                max_tokens: 32768,
                max_input_tokens: 131072,
                supports_functions: false,
                supports_vision: false,
                supports_streaming: false,
                cost_per_1k_tokens: 0.015,
                status: None,
            },
        );

        // OpenCode models
        models.insert(
            "opencode/gpt-5-nano".to_string(),
            ModelInfo {
                name: "opencode/gpt-5-nano".to_string(),
                provider: "opencode".to_string(),
                max_tokens: 16384,
                max_input_tokens: 128000,
                supports_functions: true,
                supports_vision: true,
                supports_streaming: true,
                cost_per_1k_tokens: 0.0005,
                status: None,
            },
        );

        models.insert(
            "opencode/minimax-m2.5-free".to_string(),
            ModelInfo {
                name: "opencode/minimax-m2.5-free".to_string(),
                provider: "opencode".to_string(),
                max_tokens: 8192,
                max_input_tokens: 1000000,
                supports_functions: false,
                supports_vision: true,
                supports_streaming: true,
                cost_per_1k_tokens: 0.0,
                status: None,
            },
        );

        models.insert(
            "opencode/nemotron-3-super-free".to_string(),
            ModelInfo {
                name: "opencode/nemotron-3-super-free".to_string(),
                provider: "opencode".to_string(),
                max_tokens: 8192,
                max_input_tokens: 131072,
                supports_functions: false,
                supports_vision: false,
                supports_streaming: true,
                cost_per_1k_tokens: 0.0,
                status: None,
            },
        );

        // Google Antigravity models
        models.insert(
            "google/antigravity-1".to_string(),
            ModelInfo {
                name: "google/antigravity-1".to_string(),
                provider: "google".to_string(),
                max_tokens: 8192,
                max_input_tokens: 1000000,
                supports_functions: false,
                supports_vision: true,
                supports_streaming: true,
                cost_per_1k_tokens: 0.0,
                status: None,
            },
        );

        models.insert(
            "google/antigravity-2".to_string(),
            ModelInfo {
                name: "google/antigravity-2".to_string(),
                provider: "google".to_string(),
                max_tokens: 16384,
                max_input_tokens: 2000000,
                supports_functions: false,
                supports_vision: true,
                supports_streaming: true,
                cost_per_1k_tokens: 0.0,
                status: None,
            },
        );

        models.insert(
            "google/antigravity-3".to_string(),
            ModelInfo {
                name: "google/antigravity-3".to_string(),
                provider: "google".to_string(),
                max_tokens: 16384,
                max_input_tokens: 2000000,
                supports_functions: true,
                supports_vision: true,
                supports_streaming: true,
                cost_per_1k_tokens: 0.0,
                status: None,
            },
        );

        models.insert(
            "google/antigravity-ultra".to_string(),
            ModelInfo {
                name: "google/antigravity-ultra".to_string(),
                provider: "google".to_string(),
                max_tokens: 32768,
                max_input_tokens: 2000000,
                supports_functions: true,
                supports_vision: true,
                supports_streaming: true,
                cost_per_1k_tokens: 0.001,
                status: None,
            },
        );

        // Kimi models
        models.insert(
            "kimi/kimi-2.5".to_string(),
            ModelInfo {
                name: "kimi/kimi-2.5".to_string(),
                provider: "kimi".to_string(),
                max_tokens: 32768,
                max_input_tokens: 128000,
                supports_functions: true,
                supports_vision: true,
                supports_streaming: true,
                cost_per_1k_tokens: 0.002,
                status: None,
            },
        );

        models.insert(
            "kimi/kimi-2".to_string(),
            ModelInfo {
                name: "kimi/kimi-2".to_string(),
                provider: "kimi".to_string(),
                max_tokens: 16384,
                max_input_tokens: 128000,
                supports_functions: true,
                supports_vision: true,
                supports_streaming: true,
                cost_per_1k_tokens: 0.0015,
                status: None,
            },
        );

        models.insert(
            "kimi/kimi-1.5".to_string(),
            ModelInfo {
                name: "kimi/kimi-1.5".to_string(),
                provider: "kimi".to_string(),
                max_tokens: 16384,
                max_input_tokens: 128000,
                supports_functions: true,
                supports_vision: true,
                supports_streaming: true,
                cost_per_1k_tokens: 0.001,
                status: None,
            },
        );

        models.insert(
            "kimi/kimi-latest".to_string(),
            ModelInfo {
                name: "kimi/kimi-latest".to_string(),
                provider: "kimi".to_string(),
                max_tokens: 32768,
                max_input_tokens: 128000,
                supports_functions: true,
                supports_vision: true,
                supports_streaming: true,
                cost_per_1k_tokens: 0.002,
                status: None,
            },
        );

        models.insert(
            "kimi/moonshot-turbo".to_string(),
            ModelInfo {
                name: "kimi/moonshot-turbo".to_string(),
                provider: "kimi".to_string(),
                max_tokens: 16384,
                max_input_tokens: 128000,
                supports_functions: false,
                supports_vision: true,
                supports_streaming: true,
                cost_per_1k_tokens: 0.001,
                status: None,
            },
        );

        models.insert(
            "kimi/moonshot-v1-128k".to_string(),
            ModelInfo {
                name: "kimi/moonshot-v1-128k".to_string(),
                provider: "kimi".to_string(),
                max_tokens: 16384,
                max_input_tokens: 131072,
                supports_functions: false,
                supports_vision: true,
                supports_streaming: true,
                cost_per_1k_tokens: 0.002,
                status: None,
            },
        );

        // Z.AI models
        models.insert(
            "z.ai/z-1".to_string(),
            ModelInfo {
                name: "z.ai/z-1".to_string(),
                provider: "z.ai".to_string(),
                max_tokens: 16384,
                max_input_tokens: 128000,
                supports_functions: true,
                supports_vision: true,
                supports_streaming: true,
                cost_per_1k_tokens: 0.002,
                status: None,
            },
        );

        models.insert(
            "z.ai/z-1-mini".to_string(),
            ModelInfo {
                name: "z.ai/z-1-mini".to_string(),
                provider: "z.ai".to_string(),
                max_tokens: 16384,
                max_input_tokens: 128000,
                supports_functions: true,
                supports_vision: true,
                supports_streaming: true,
                cost_per_1k_tokens: 0.001,
                status: None,
            },
        );

        models.insert(
            "z.ai/z-1-flash".to_string(),
            ModelInfo {
                name: "z.ai/z-1-flash".to_string(),
                provider: "z.ai".to_string(),
                max_tokens: 8192,
                max_input_tokens: 128000,
                supports_functions: false,
                supports_vision: true,
                supports_streaming: true,
                cost_per_1k_tokens: 0.0005,
                status: None,
            },
        );

        models.insert(
            "z.ai/z-1-preview".to_string(),
            ModelInfo {
                name: "z.ai/z-1-preview".to_string(),
                provider: "z.ai".to_string(),
                max_tokens: 32768,
                max_input_tokens: 128000,
                supports_functions: true,
                supports_vision: true,
                supports_streaming: true,
                cost_per_1k_tokens: 0.003,
                status: None,
            },
        );

        models.insert(
            "z.ai/llama-3.1-70b".to_string(),
            ModelInfo {
                name: "z.ai/llama-3.1-70b".to_string(),
                provider: "z.ai".to_string(),
                max_tokens: 8192,
                max_input_tokens: 32768,
                supports_functions: false,
                supports_vision: false,
                supports_streaming: true,
                cost_per_1k_tokens: 0.0005,
                status: None,
            },
        );

        models.insert(
            "z.ai/llama-3.1-8b".to_string(),
            ModelInfo {
                name: "z.ai/llama-3.1-8b".to_string(),
                provider: "z.ai".to_string(),
                max_tokens: 8192,
                max_input_tokens: 32768,
                supports_functions: false,
                supports_vision: false,
                supports_streaming: true,
                cost_per_1k_tokens: 0.0002,
                status: None,
            },
        );

        models.insert(
            "z.ai/codellama-70b".to_string(),
            ModelInfo {
                name: "z.ai/codellama-70b".to_string(),
                provider: "z.ai".to_string(),
                max_tokens: 4096,
                max_input_tokens: 16384,
                supports_functions: false,
                supports_vision: false,
                supports_streaming: true,
                cost_per_1k_tokens: 0.0005,
                status: None,
            },
        );

        models.insert(
            "z.ai/mistral-7b".to_string(),
            ModelInfo {
                name: "z.ai/mistral-7b".to_string(),
                provider: "z.ai".to_string(),
                max_tokens: 8192,
                max_input_tokens: 32768,
                supports_functions: false,
                supports_vision: false,
                supports_streaming: true,
                cost_per_1k_tokens: 0.0002,
                status: None,
            },
        );

        models.insert(
            "z.ai/mixtral-8x7b".to_string(),
            ModelInfo {
                name: "z.ai/mixtral-8x7b".to_string(),
                provider: "z.ai".to_string(),
                max_tokens: 8192,
                max_input_tokens: 32768,
                supports_functions: false,
                supports_vision: false,
                supports_streaming: true,
                cost_per_1k_tokens: 0.0005,
                status: None,
            },
        );

        // Additional models from existing providers
        models.insert(
            "openai/o1".to_string(),
            ModelInfo {
                name: "openai/o1".to_string(),
                provider: "openai".to_string(),
                max_tokens: 32768,
                max_input_tokens: 128000,
                supports_functions: false,
                supports_vision: false,
                supports_streaming: false,
                cost_per_1k_tokens: 0.015,
                status: None,
            },
        );

        models.insert(
            "openai/o1-mini".to_string(),
            ModelInfo {
                name: "openai/o1-mini".to_string(),
                provider: "openai".to_string(),
                max_tokens: 65536,
                max_input_tokens: 131072,
                supports_functions: false,
                supports_vision: false,
                supports_streaming: false,
                cost_per_1k_tokens: 0.007,
                status: None,
            },
        );

        models.insert(
            "openai/o1-preview".to_string(),
            ModelInfo {
                name: "openai/o1-preview".to_string(),
                provider: "openai".to_string(),
                max_tokens: 32768,
                max_input_tokens: 131072,
                supports_functions: false,
                supports_vision: false,
                supports_streaming: false,
                cost_per_1k_tokens: 0.015,
                status: None,
            },
        );

        models.insert(
            "openai/gpt-4o-2024-08-13".to_string(),
            ModelInfo {
                name: "openai/gpt-4o-2024-08-13".to_string(),
                provider: "openai".to_string(),
                max_tokens: 16384,
                max_input_tokens: 128000,
                supports_functions: true,
                supports_vision: true,
                supports_streaming: true,
                cost_per_1k_tokens: 0.006,
                status: None,
            },
        );

        models.insert(
            "openai/gpt-4o-mini-2024-07-18".to_string(),
            ModelInfo {
                name: "openai/gpt-4o-mini-2024-07-18".to_string(),
                provider: "openai".to_string(),
                max_tokens: 16384,
                max_input_tokens: 128000,
                supports_functions: true,
                supports_vision: true,
                supports_streaming: true,
                cost_per_1k_tokens: 0.0015,
                status: None,
            },
        );

        // Additional Claude models
        models.insert(
            "anthropic/claude-sonnet-4-20250514".to_string(),
            ModelInfo {
                name: "anthropic/claude-sonnet-4-20250514".to_string(),
                provider: "anthropic".to_string(),
                max_tokens: 4096,
                max_input_tokens: 200000,
                supports_functions: false,
                supports_vision: true,
                supports_streaming: true,
                cost_per_1k_tokens: 0.003,
                status: None,
            },
        );

        models.insert(
            "anthropic/claude-opus-4-20250514".to_string(),
            ModelInfo {
                name: "anthropic/claude-opus-4-20250514".to_string(),
                provider: "anthropic".to_string(),
                max_tokens: 4096,
                max_input_tokens: 200000,
                supports_functions: false,
                supports_vision: true,
                supports_streaming: true,
                cost_per_1k_tokens: 0.015,
                status: None,
            },
        );

        models.insert(
            "anthropic/claude-3-5-sonnet-latest".to_string(),
            ModelInfo {
                name: "anthropic/claude-3-5-sonnet-latest".to_string(),
                provider: "anthropic".to_string(),
                max_tokens: 4096,
                max_input_tokens: 200000,
                supports_functions: false,
                supports_vision: true,
                supports_streaming: true,
                cost_per_1k_tokens: 0.003,
                status: None,
            },
        );

        models.insert(
            "anthropic/claude-3-5-haiku-latest".to_string(),
            ModelInfo {
                name: "anthropic/claude-3-5-haiku-latest".to_string(),
                provider: "anthropic".to_string(),
                max_tokens: 4096,
                max_input_tokens: 200000,
                supports_functions: false,
                supports_vision: true,
                supports_streaming: true,
                cost_per_1k_tokens: 0.00025,
                status: None,
            },
        );

        // Additional Google Gemini models
        models.insert(
            "google/gemini-2.0-flash".to_string(),
            ModelInfo {
                name: "google/gemini-2.0-flash".to_string(),
                provider: "google".to_string(),
                max_tokens: 8192,
                max_input_tokens: 1000000,
                supports_functions: false,
                supports_vision: true,
                supports_streaming: true,
                cost_per_1k_tokens: 0.0,
                status: None,
            },
        );

        models.insert(
            "google/gemini-2.0-flash-exp".to_string(),
            ModelInfo {
                name: "google/gemini-2.0-flash-exp".to_string(),
                provider: "google".to_string(),
                max_tokens: 8192,
                max_input_tokens: 1000000,
                supports_functions: false,
                supports_vision: true,
                supports_streaming: true,
                cost_per_1k_tokens: 0.0,
                status: None,
            },
        );

        models.insert(
            "google/gemini-1.5-pro-latest".to_string(),
            ModelInfo {
                name: "google/gemini-1.5-pro-latest".to_string(),
                provider: "google".to_string(),
                max_tokens: 8192,
                max_input_tokens: 2000000,
                supports_functions: false,
                supports_vision: true,
                supports_streaming: true,
                cost_per_1k_tokens: 0.00125,
                status: None,
            },
        );

        models.insert(
            "google/gemini-1.5-flash-latest".to_string(),
            ModelInfo {
                name: "google/gemini-1.5-flash-latest".to_string(),
                provider: "google".to_string(),
                max_tokens: 8192,
                max_input_tokens: 1000000,
                supports_functions: false,
                supports_vision: true,
                supports_streaming: true,
                cost_per_1k_tokens: 0.0,
                status: None,
            },
        );

        models.insert(
            "google/gemini-exp-1206".to_string(),
            ModelInfo {
                name: "google/gemini-exp-1206".to_string(),
                provider: "google".to_string(),
                max_tokens: 16384,
                max_input_tokens: 2000000,
                supports_functions: false,
                supports_vision: true,
                supports_streaming: true,
                cost_per_1k_tokens: 0.001,
                status: None,
            },
        );

        Self {
            models,
            provider_filter: None,
        }
    }

    pub fn set_provider_filter(&mut self, filter: ProviderFilter) {
        self.provider_filter = Some(filter);
    }

    pub fn get(&self, name: &str) -> Option<&ModelInfo> {
        self.models
            .get(name)
            .filter(|model| self.is_provider_allowed(&model.provider))
    }

    pub fn list(&self) -> Vec<&ModelInfo> {
        self.models
            .values()
            .filter(|model| self.is_provider_allowed(&model.provider))
            .filter(|model| is_model_visible(model.status))
            .collect()
    }

    pub fn list_by_provider(&self, provider: &str) -> Vec<&ModelInfo> {
        if !self.is_provider_allowed(provider) {
            return vec![];
        }

        self.models
            .values()
            .filter(|m| m.provider == provider)
            .collect()
    }

    pub fn list_providers(&self) -> Vec<String> {
        let mut providers: Vec<String> = self
            .models
            .values()
            .map(|model| model.provider.clone())
            .collect::<HashSet<_>>()
            .into_iter()
            .filter(|provider| self.is_provider_allowed(provider))
            .collect();
        providers.sort();
        providers
    }

    pub fn get_next_available_provider(&self, failed_provider: &str) -> Option<String> {
        let failed_provider = failed_provider.trim();
        let providers = self.list_providers();
        providers
            .into_iter()
            .find(|provider| !provider.eq_ignore_ascii_case(failed_provider))
    }

    fn is_provider_allowed(&self, provider: &str) -> bool {
        self.provider_filter
            .as_ref()
            .map(|filter| filter.is_allowed(provider))
            .unwrap_or(true)
    }

    pub fn supports_function(&self, model: &str) -> bool {
        self.get(model)
            .map(|m| m.supports_functions)
            .unwrap_or(false)
    }

    pub fn max_tokens(&self, model: &str) -> u32 {
        self.get(model).map(|m| m.max_tokens).unwrap_or(4096)
    }

    pub fn max_input_tokens(&self, model: &str) -> u32 {
        self.get(model).map(|m| m.max_input_tokens).unwrap_or(4096)
    }

    pub fn populate_from_catalog(&mut self, catalog: &crate::catalog::ProviderCatalog) {
        for provider in catalog.providers.values() {
            for model in provider.models.values() {
                let model_key = model.id.clone();
                self.models.entry(model_key).or_insert_with(|| ModelInfo {
                    name: model.id.clone(),
                    provider: provider.id.clone(),
                    max_tokens: model.limits.output.max(1),
                    max_input_tokens: model.limits.context,
                    supports_functions: model.capabilities.tool_call,
                    supports_vision: model
                        .capabilities
                        .input_modalities
                        .contains(&"image".to_string())
                        || model
                            .capabilities
                            .input_modalities
                            .contains(&"vision".to_string()),
                    supports_streaming: true,
                    cost_per_1k_tokens: model.cost.input + model.cost.output,
                    status: Some(model.status),
                });
            }
        }
    }
}

impl Default for ModelRegistry {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::{is_model_visible, ModelRegistry};
    use crate::catalog::types::ModelStatus;
    use crate::provider_filter::ProviderFilter;

    #[test]
    fn list_respects_provider_filter() {
        let mut registry = ModelRegistry::new();
        registry.set_provider_filter(ProviderFilter::new(
            vec!["openai".to_string()],
            vec!["openai".to_string(), "anthropic".to_string()],
        ));

        let providers: Vec<String> = registry.list().iter().map(|m| m.provider.clone()).collect();

        assert!(providers.iter().all(|provider| provider == "anthropic"));
        assert!(!providers.is_empty());
    }

    #[test]
    fn get_next_available_skips_failed_and_disallowed_providers() {
        let mut registry = ModelRegistry::new();
        registry.set_provider_filter(ProviderFilter::new(
            vec!["openai".to_string()],
            vec!["openai".to_string(), "anthropic".to_string()],
        ));

        assert_eq!(
            registry.get_next_available_provider("openai"),
            Some("anthropic".to_string())
        );
        assert_eq!(registry.get_next_available_provider("anthropic"), None);
    }

    #[test]
    fn verify_model_catalog_contains_50_plus_models() {
        let registry = ModelRegistry::new();
        let model_count = registry.list().len();
        assert!(
            model_count >= 50,
            "Model catalog should contain at least 50 models, but only contains {}",
            model_count
        );
    }

    #[test]
    fn verify_original_provider_models_still_available() {
        let registry = ModelRegistry::new();

        let original_models = vec![
            "gpt-4o",
            "gpt-4o-mini",
            "gpt-4-turbo",
            "claude-sonnet-4-20250514",
            "claude-haiku-3",
            "claude-opus-4-20250514",
            "llama3",
            "codellama",
            "gpt-4o-azure",
            "gemini-1.5-pro",
            "gemini-1.5-flash",
            "openrouter/gpt-4o",
            "grok-2",
            "mistral-large-latest",
            "llama-3.1-70b-versatile",
            "deepinfra/llama-3.1-70b",
            "cerebras/llama-3.1-70b",
            "cohere-command-r-plus",
            "togetherai/llama-3.1-70b",
            "perplexity/llama-3.1-sonar-large",
        ];

        for model_name in original_models {
            let model = registry.get(model_name);
            assert!(
                model.is_some(),
                "Original model '{}' should still be available",
                model_name
            );
        }
    }

    #[test]
    fn verify_model_context_lengths_displayed_correctly() {
        let registry = ModelRegistry::new();

        let test_cases = vec![
            ("gpt-4o", 16384, 128000),
            ("gpt-4o-mini", 16384, 128000),
            ("gemini-1.5-pro", 8192, 2000000),
            ("gemini-1.5-flash", 8192, 1000000),
            ("claude-sonnet-4-20250514", 4096, 200000),
            ("claude-haiku-3", 4096, 200000),
            ("llama3", 4096, 8192),
            ("github-copilot/gpt-4o", 16384, 128000),
            ("kimi/kimi-2.5", 32768, 128000),
            ("z.ai/z-1", 16384, 128000),
            ("google/antigravity-1", 8192, 1000000),
            ("opencode/gpt-5-nano", 16384, 128000),
        ];

        for (model_name, expected_max_tokens, expected_max_input) in test_cases {
            let model = registry.get(model_name);
            assert!(model.is_some(), "Model '{}' should exist", model_name);
            let model = model.unwrap();
            assert_eq!(
                model.max_tokens, expected_max_tokens,
                "Model '{}' max_tokens should be {}",
                model_name, expected_max_tokens
            );
            assert_eq!(
                model.max_input_tokens, expected_max_input,
                "Model '{}' max_input_tokens should be {}",
                model_name, expected_max_input
            );
        }
    }

    #[test]
    fn verify_new_provider_models_available() {
        let registry = ModelRegistry::new();

        let new_models = vec![
            "github-copilot/gpt-4o",
            "github-copilot/gpt-4o-mini",
            "github-copilot/claude-sonnet-4",
            "github-copilot/claude-haiku-3",
            "github-copilot/o1",
            "github-copilot/o1-mini",
            "github-copilot/o1-preview",
            "opencode/gpt-5-nano",
            "opencode/minimax-m2.5-free",
            "opencode/nemotron-3-super-free",
            "google/antigravity-1",
            "google/antigravity-2",
            "google/antigravity-3",
            "google/antigravity-ultra",
            "kimi/kimi-2.5",
            "kimi/kimi-2",
            "kimi/kimi-1.5",
            "kimi/kimi-latest",
            "kimi/moonshot-turbo",
            "kimi/moonshot-v1-128k",
            "z.ai/z-1",
            "z.ai/z-1-mini",
            "z.ai/z-1-flash",
            "z.ai/z-1-preview",
            "z.ai/llama-3.1-70b",
            "z.ai/llama-3.1-8b",
            "z.ai/codellama-70b",
            "z.ai/mistral-7b",
            "z.ai/mixtral-8x7b",
        ];

        for model_name in new_models {
            let model = registry.get(model_name);
            assert!(
                model.is_some(),
                "New model '{}' should be available",
                model_name
            );
        }
    }

    #[test]
    fn verify_all_new_providers_listed() {
        let registry = ModelRegistry::new();
        let providers = registry.list_providers();

        let expected_providers = vec![
            "anthropic",
            "azure",
            "cerebras",
            "cohere",
            "deepinfra",
            "github-copilot",
            "google",
            "groq",
            "kimi",
            "mistral",
            "ollama",
            "openai",
            "opencode",
            "openrouter",
            "perplexity",
            "togetherai",
            "xai",
            "z.ai",
        ];

        for provider in expected_providers {
            assert!(
                providers.contains(&provider.to_string()),
                "Provider '{}' should be in the provider list",
                provider
            );
        }
    }

    #[test]
    fn verify_alpha_models_hidden_by_default() {
        let registry = ModelRegistry::new();
        let models = registry.list();
        let alpha_models: Vec<&super::ModelInfo> = models
            .iter()
            .filter(|m| m.status == Some(ModelStatus::Alpha))
            .copied()
            .collect();
        assert!(
            alpha_models.is_empty(),
            "Alpha models should be hidden by default, but found: {:?}",
            alpha_models.iter().map(|m| &m.name).collect::<Vec<_>>()
        );
    }

    #[test]
    fn verify_non_alpha_models_always_visible() {
        let registry = ModelRegistry::new();
        let models = registry.list();
        let has_non_alpha = models.iter().any(|m| m.status != Some(ModelStatus::Alpha));
        assert!(
            has_non_alpha,
            "Non-alpha models should always be visible"
        );
    }

    #[test]
    fn verify_alpha_model_visible_with_env_flag() {
        let _temp_dir = std::env::temp_dir().join("test_alpha_catalog.json");
        let mut registry = ModelRegistry::new();
        registry.populate_from_catalog(&crate::catalog::types::ProviderCatalog {
            providers: std::collections::BTreeMap::from([(
                "test-provider".to_string(),
                crate::catalog::types::ProviderDescriptor {
                    id: "test-provider".to_string(),
                    display_name: "Test Provider".to_string(),
                    api_base_url: None,
                    docs_url: None,
                    env_vars: vec![],
                    npm_package: None,
                    models: std::collections::BTreeMap::from([(
                        "alpha-model".to_string(),
                        crate::catalog::types::ModelDescriptor {
                            id: "alpha-model".to_string(),
                            display_name: "Alpha Model".to_string(),
                            family: None,
                            provider_id: "test-provider".to_string(),
                            capabilities: crate::catalog::types::ModelCapabilities::default(),
                            cost: crate::catalog::types::CostInfo::default(),
                            limits: crate::catalog::types::LimitInfo::default(),
                            status: ModelStatus::Alpha,
                            variants: vec![],
                        },
                    )]),
                    source: crate::catalog::types::CatalogSource::Local,
                },
            )]),
            fetched_at: chrono::Utc::now(),
            source: crate::catalog::types::CatalogSource::Local,
        });
        let models_without_flag: Vec<&super::ModelInfo> = registry
            .list()
            .iter()
            .filter(|m| m.name == "alpha-model")
            .copied()
            .collect();
        assert!(
            models_without_flag.is_empty(),
            "Alpha model should not be visible without env flag"
        );
    }
}
