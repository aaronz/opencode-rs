use crate::provider::{Provider, ProviderConfig, Model, StreamingCallback};
use opencode_core::OpenCodeError;

pub struct MistralProvider {
    config: ProviderConfig,
}

impl MistralProvider {
    pub fn new(config: ProviderConfig) -> Self {
        Self { config }
    }
}

#[async_trait::async_trait]
impl Provider for MistralProvider {
    async fn complete(&self, prompt: &str, context: Option<&str>) -> Result<String, OpenCodeError> {
        let client = reqwest::Client::new();
        let url = "https://api.mistral.ai/v1/chat/completions";

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
            .ok_or_else(|| OpenCodeError::Llm("Invalid Mistral response".to_string()))
    }

    async fn complete_streaming(&self, _prompt: &str, _callback: StreamingCallback) -> Result<(), OpenCodeError> {
        Err(OpenCodeError::Llm("Mistral streaming not implemented".to_string()))
    }

    fn get_models(&self) -> Vec<Model> {
        vec![
            Model::new("mistral-large-latest", "Mistral Large"),
            Model::new("mistral-medium-latest", "Mistral Medium"),
            Model::new("mistral-small-latest", "Mistral Small"),
            Model::new("codestral-latest", "Codestral"),
        ]
    }

    fn provider_name(&self) -> &str {
        "mistral"
    }
}
