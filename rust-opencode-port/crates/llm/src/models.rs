use serde::{Deserialize, Serialize};
use std::collections::HashMap;

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
}

pub struct ModelRegistry {
    models: HashMap<String, ModelInfo>,
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
            },
        );

        Self { models }
    }

    pub fn get(&self, name: &str) -> Option<&ModelInfo> {
        self.models.get(name)
    }

    pub fn list(&self) -> Vec<&ModelInfo> {
        self.models.values().collect()
    }

    pub fn list_by_provider(&self, provider: &str) -> Vec<&ModelInfo> {
        self.models
            .values()
            .filter(|m| m.provider == provider)
            .collect()
    }

    pub fn supports_function(&self, model: &str) -> bool {
        self.models
            .get(model)
            .map(|m| m.supports_functions)
            .unwrap_or(false)
    }

    pub fn max_tokens(&self, model: &str) -> u32 {
        self.models.get(model).map(|m| m.max_tokens).unwrap_or(4096)
    }

    pub fn max_input_tokens(&self, model: &str) -> u32 {
        self.models
            .get(model)
            .map(|m| m.max_input_tokens)
            .unwrap_or(4096)
    }
}

impl Default for ModelRegistry {
    fn default() -> Self {
        Self::new()
    }
}
