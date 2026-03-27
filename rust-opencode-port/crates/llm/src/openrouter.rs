use crate::provider::{Provider, ProviderConfig, Model, StreamingCallback};
use opencode_core::OpenCodeError;

pub struct OpenRouterProvider {
    config: ProviderConfig,
}

impl OpenRouterProvider {
    pub fn new(config: ProviderConfig) -> Self {
        Self { config }
    }
}

#[async_trait::async_trait]
impl Provider for OpenRouterProvider {
    async fn complete(&self, prompt: &str, context: Option<&str>) -> Result<String, OpenCodeError> {
        let client = reqwest::Client::new();
        let url = "https://openrouter.ai/api/v1/chat/completions";
        
        // Build messages - system context first if provided
        let mut messages = vec![];
        if let Some(ctx) = context {
            messages.push(serde_json::json!({"role": "system", "content": ctx}));
        }
        messages.push(serde_json::json!({"role": "user", "content": prompt}));
        
        let body = serde_json::json!({
            "model": self.config.model,
            "messages": messages,
            "temperature": self.config.temperature,
        });

        let response = client
            .post(url)
            .header("Authorization", format!("Bearer {}", self.config.api_key))
            .header("Content-Type", "application/json")
            .json(&body)
            .send()
            .await
            .map_err(|e| OpenCodeError::Llm(e.to_string()))?;

        let result: serde_json::Value = response.json().await.map_err(|e| OpenCodeError::Llm(e.to_string()))?;
        result["choices"][0]["message"]["content"]
            .as_str()
            .map(|s| s.to_string())
            .ok_or_else(|| OpenCodeError::Llm("Invalid response format".to_string()))
    }

    async fn complete_streaming(&self, _prompt: &str, _callback: StreamingCallback) -> Result<(), OpenCodeError> {
        Err(OpenCodeError::Llm("Streaming not implemented".to_string()))
    }

    fn get_models(&self) -> Vec<Model> {
        vec![
            Model::new("openai/gpt-4o", "GPT-4o"),
            Model::new("openai/gpt-4o-mini", "GPT-4o Mini"),
            Model::new("anthropic/claude-3.5-sonnet", "Claude 3.5 Sonnet"),
            Model::new("meta-llama/llama-3.1-70b", "Llama 3.1 70B"),
            Model::new("google/gemini-pro-1.5", "Gemini Pro 1.5"),
            Model::new("mistralai/mistral-7b", "Mistral 7B"),
        ]
    }

    fn provider_name(&self) -> &str {
        "openrouter"
    }
}
