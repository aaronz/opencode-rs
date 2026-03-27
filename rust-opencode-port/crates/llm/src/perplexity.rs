use crate::provider::{Provider, ProviderConfig, Model, StreamingCallback};
use opencode_core::OpenCodeError;

pub struct PerplexityProvider {
    config: ProviderConfig,
}

impl PerplexityProvider {
    pub fn new(config: ProviderConfig) -> Self {
        Self { config }
    }
}

#[async_trait::async_trait]
impl Provider for PerplexityProvider {
    async fn complete(&self, prompt: &str, context: Option<&str>) -> Result<String, OpenCodeError> {
        let client = reqwest::Client::new();
        let url = "https://api.perplexity.ai/chat/completions";

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
            .ok_or_else(|| OpenCodeError::Llm("Invalid Perplexity response".to_string()))
    }

    async fn complete_streaming(&self, _prompt: &str, _callback: StreamingCallback) -> Result<(), OpenCodeError> {
        Err(OpenCodeError::Llm("Perplexity streaming not implemented".to_string()))
    }

    fn get_models(&self) -> Vec<Model> {
        vec![
            Model::new("llama-3.1-sonar-large-128k-online", "Llama 3.1 Sonar Large"),
            Model::new("llama-3.1-sonar-small-128k-online", "Llama 3.1 Sonar Small"),
            Model::new("llama-3-sonar-large-32k-online", "Llama 3 Sonar Large"),
            Model::new("llama-3-sonar-small-32k-online", "Llama 3 Sonar Small"),
        ]
    }

    fn provider_name(&self) -> &str {
        "perplexity"
    }
}
