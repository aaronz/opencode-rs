use crate::provider::{Provider, ProviderConfig, Model, StreamingCallback};
use opencode_core::OpenCodeError;

pub struct VertexProvider {
    config: ProviderConfig,
    project_id: String,
    location: String,
}

impl VertexProvider {
    pub fn new(config: ProviderConfig, project_id: String, location: String) -> Self {
        Self { config, project_id, location }
    }

    pub fn from_env() -> Option<Self> {
        let project_id = std::env::var("GOOGLE_CLOUD_PROJECT").ok()?;
        let location = std::env::var("GOOGLE_CLOUD_LOCATION").unwrap_or_else(|_| "us-central1".to_string());
        let config = ProviderConfig {
            model: std::env::var("VERTEX_MODEL").unwrap_or_else(|_| "gemini-1.5-pro".to_string()),
            api_key: std::env::var("GOOGLE_APPLICATION_CREDENTIALS").unwrap_or_default(),
            temperature: 0.7,
        };
        Some(Self::new(config, project_id, location))
    }
}

#[async_trait::async_trait]
impl Provider for VertexProvider {
    async fn complete(&self, prompt: &str, context: Option<&str>) -> Result<String, OpenCodeError> {
        let client = reqwest::Client::new();
        let url = format!(
            "https://{}-aiplatform.googleapis.com/v1/projects/{}/locations/{}/publishers/google/models/{}:predict",
            self.location,
            self.project_id,
            self.location,
            self.config.model
        );

        let instances = if let Some(ctx) = context {
            vec![
                serde_json::json!({"content": ctx}),
                serde_json::json!({"content": prompt})
            ]
        } else {
            vec![serde_json::json!({"content": prompt})]
        };

        let body = serde_json::json!({
            "instances": instances,
            "parameters": {
                "temperature": self.config.temperature,
            }
        });

        let response = client
            .post(&url)
            .header("Authorization", format!("Bearer {}", self.config.api_key))
            .header("Content-Type", "application/json")
            .json(&body)
            .send()
            .await
            .map_err(|e| OpenCodeError::Llm(e.to_string()))?;

        let result: serde_json::Value = response.json().await.map_err(|e| OpenCodeError::Llm(e.to_string()))?;
        result["predictions"][0]["content"]
            .as_str()
            .map(|s| s.to_string())
            .ok_or_else(|| OpenCodeError::Llm("Invalid Vertex response".to_string()))
    }

    async fn complete_streaming(&self, prompt: &str, mut callback: StreamingCallback) -> Result<(), OpenCodeError> {
        let client = reqwest::Client::new();
        let url = format!(
            "https://{}-aiplatform.googleapis.com/v1/projects/{}/locations/{}/publishers/google/models/{}:streamGenerateContent?alt=sse",
            self.location,
            self.project_id,
            self.location,
            self.config.model
        );

        let body = serde_json::json!({
            "contents": [
                {
                    "role": "user",
                    "parts": [
                        { "text": prompt }
                    ]
                }
            ],
            "generationConfig": {
                "temperature": self.config.temperature,
            }
        });

        let response = client
            .post(&url)
            .header("Authorization", format!("Bearer {}", self.config.api_key))
            .header("Content-Type", "application/json")
            .json(&body)
            .send()
            .await
            .map_err(|e| OpenCodeError::Llm(e.to_string()))?;

        if !response.status().is_success() {
            let status = response.status();
            let error_text = response.text().await.unwrap_or_default();
            return Err(OpenCodeError::Llm(format!("Vertex API error {}: {}", status, error_text)));
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
                        if data.trim().is_empty() || data == "[DONE]" {
                            continue;
                        }

                        if let Ok(chunk) = serde_json::from_str::<serde_json::Value>(data) {
                            if let Some(content) = chunk["candidates"][0]["content"]["parts"][0]["text"].as_str() {
                                callback(content.to_string());
                            }
                        }
                    }
                }
                Err(e) => return Err(OpenCodeError::Llm(format!("Vertex stream error: {}", e))),
            }
        }

        callback(String::new());
        Ok(())
    }

    fn get_models(&self) -> Vec<Model> {
        vec![
            Model::new("gemini-1.5-pro", "Gemini 1.5 Pro"),
            Model::new("gemini-1.5-flash", "Gemini 1.5 Flash"),
            Model::new("gemini-1.5-pro-002", "Gemini 1.5 Pro 002"),
            Model::new("gemini-1.5-flash-002", "Gemini 1.5 Flash 002"),
        ]
    }

    fn provider_name(&self) -> &str {
        "vertex"
    }
}
