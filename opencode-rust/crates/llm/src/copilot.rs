use crate::provider::{Model, Provider, ProviderConfig, StreamingCallback};
use opencode_core::OpenCodeError;

pub struct CopilotProvider {
    config: ProviderConfig,
}

impl CopilotProvider {
    pub fn new(config: ProviderConfig) -> Self {
        Self { config }
    }

    pub fn supports_copilot() -> bool {
        std::env::var("GITHUB_COPILOT_TOKEN").is_ok()
            || std::env::var("OPENCODE_COPILOT_TOKEN").is_ok()
    }
}

#[async_trait::async_trait]
impl Provider for CopilotProvider {
    async fn complete(&self, prompt: &str, context: Option<&str>) -> Result<String, OpenCodeError> {
        let token = std::env::var("GITHUB_COPILOT_TOKEN")
            .or_else(|_| std::env::var("OPENCODE_COPILOT_TOKEN"))
            .map_err(|_| OpenCodeError::Llm("No Copilot token found".to_string()))?;

        let client = reqwest::Client::new();
        let url = "https://api.github.com/copilot-interactive/v1/chat";

        let mut messages = vec![];
        if let Some(ctx) = context {
            messages.push(serde_json::json!({
                "role": "system",
                "content": ctx
            }));
        }
        messages.push(serde_json::json!({
            "role": "user",
            "content": prompt
        }));

        let body = serde_json::json!({
            "messages": messages,
            "model": self.config.model,
        });

        let response = client
            .post(url)
            .header("Authorization", format!("Bearer {}", token))
            .header("Content-Type", "application/json")
            .header("Accept", "application/vnd.github.copilot-chat-preview+json")
            .json(&body)
            .send()
            .await
            .map_err(|e| OpenCodeError::Llm(e.to_string()))?;

        let result: serde_json::Value = response
            .json()
            .await
            .map_err(|e| OpenCodeError::Llm(e.to_string()))?;
        result["choices"][0]["message"]["content"]
            .as_str()
            .map(|s| s.to_string())
            .ok_or_else(|| OpenCodeError::Llm("Invalid Copilot response".to_string()))
    }

    async fn complete_streaming(
        &self,
        prompt: &str,
        mut callback: StreamingCallback,
    ) -> Result<(), OpenCodeError> {
        let token = std::env::var("GITHUB_COPILOT_TOKEN")
            .or_else(|_| std::env::var("OPENCODE_COPILOT_TOKEN"))
            .map_err(|_| OpenCodeError::Llm("No Copilot token found".to_string()))?;

        let client = reqwest::Client::new();
        let url = "https://api.github.com/copilot-interactive/v1/chat";

        let body = serde_json::json!({
            "messages": [
                {"role": "user", "content": prompt}
            ],
            "model": self.config.model,
            "stream": true,
        });

        let response = client
            .post(url)
            .header("Authorization", format!("Bearer {}", token))
            .header("Content-Type", "application/json")
            .header("Accept", "application/vnd.github.copilot-chat-preview+json")
            .json(&body)
            .send()
            .await
            .map_err(|e| OpenCodeError::Llm(e.to_string()))?;

        if !response.status().is_success() {
            let status = response.status();
            let error_text = response.text().await.unwrap_or_default();
            return Err(OpenCodeError::Llm(format!(
                "Copilot API error {}: {}",
                status, error_text
            )));
        }

        use futures_util::StreamExt;
        let mut lines = response.bytes_stream();
        while let Some(item) = lines.next().await {
            match item {
                Ok(bytes) => {
                    let text = String::from_utf8_lossy(&bytes);
                    for line in text.lines() {
                        if !line.starts_with("data: ") {
                            continue;
                        }

                        let data = line.strip_prefix("data: ").unwrap_or("");
                        if data == "[DONE]" {
                            callback(String::new());
                            return Ok(());
                        }

                        if let Ok(chunk) = serde_json::from_str::<serde_json::Value>(data) {
                            if let Some(content) = chunk["choices"][0]["delta"]["content"].as_str()
                            {
                                callback(content.to_string());
                            }
                        }
                    }
                }
                Err(e) => return Err(OpenCodeError::Llm(format!("Copilot stream error: {}", e))),
            }
        }

        Ok(())
    }

    fn get_models(&self) -> Vec<Model> {
        vec![
            Model::new("gpt-4o", "GPT-4o"),
            Model::new("gpt-4o-mini", "GPT-4o Mini"),
            Model::new("claude-3.5-sonnet", "Claude 3.5 Sonnet"),
            Model::new("claude-3-haiku", "Claude 3 Haiku"),
        ]
    }

    fn provider_name(&self) -> &str {
        "copilot"
    }
}
