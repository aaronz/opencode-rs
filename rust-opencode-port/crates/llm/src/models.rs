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
