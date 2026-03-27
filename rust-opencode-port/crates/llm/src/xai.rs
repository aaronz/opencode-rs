use crate::provider::{Provider, ProviderConfig, Model, StreamingCallback};
use opencode_core::OpenCodeError;

pub struct XaiProvider {
    config: ProviderConfig,
}

impl XaiProvider {
    pub fn new(config: ProviderConfig) -> Self {
        Self { config }
    }
}

#[async_trait::async_trait]
impl Provider for XaiProvider {
    async fn complete(&self, prompt: &str, context: Option<&str>) -> Result<String, OpenCodeError> {
        let client = reqwest::Client::new();
        let url = "https://api.x.ai/v1/chat/completions";

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
            .ok_or_else(|| OpenCodeError::Llm("Invalid x.ai response".to_string()))
    }

    async fn complete_streaming(&self, _prompt: &str, _callback: StreamingCallback) -> Result<(), OpenCodeError> {
        Err(OpenCodeError::Llm("x.ai streaming not implemented".to_string()))
    }

    fn get_models(&self) -> Vec<Model> {
        vec![
            Model::new("grok-2-1212", "Grok 2"),
            Model::new("grok-2-vision-1212", "Grok 2 Vision"),
            Model::new("grok-beta", "Grok Beta"),
        ]
    }

    fn provider_name(&self) -> &str {
        "xai"
    }
}
