use crate::provider::{Provider, ProviderConfig, Model, StreamingCallback};
use opencode_core::OpenCodeError;

pub struct GroqProvider {
    config: ProviderConfig,
}

impl GroqProvider {
    pub fn new(config: ProviderConfig) -> Self {
        Self { config }
    }
}

#[async_trait::async_trait]
impl Provider for GroqProvider {
    async fn complete(&self, prompt: &str, context: Option<&str>) -> Result<String, OpenCodeError> {
        let client = reqwest::Client::new();
        let url = "https://api.groq.com/openai/v1/chat/completions";

        let messages = if let Some(ctx) = context {
            vec![
                serde_json::json!({"role": "system", "content": ctx}),
                serde_json::json!({"role": "user", "content": prompt})
            ]
        } else {
            vec![serde_json::json!({"role": "user", "content": prompt})]
        };

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
            .ok_or_else(|| OpenCodeError::Llm("Invalid Groq response".to_string()))
    }

    async fn complete_streaming(&self, _prompt: &str, _callback: StreamingCallback) -> Result<(), OpenCodeError> {
        Err(OpenCodeError::Llm("Groq streaming not implemented".to_string()))
    }

    fn get_models(&self) -> Vec<Model> {
        vec![
            Model::new("llama-3.1-70b-versatile", "Llama 3.1 70B"),
            Model::new("llama-3.1-405b-reasoning", "Llama 3.1 405B"),
            Model::new("mixtral-8x7b-32768", "Mixtral 8x7B"),
            Model::new("gemma2-9b-it", "Gemma 2 9B"),
        ]
    }

    fn provider_name(&self) -> &str {
        "groq"
    }
}
