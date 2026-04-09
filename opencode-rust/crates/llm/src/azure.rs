use async_trait::async_trait;
use reqwest::Client;
use serde::{Deserialize, Serialize};

use crate::provider::{Provider, StreamingCallback};
use opencode_core::OpenCodeError;

pub struct AzureProvider {
    client: Client,
    api_key: String,
    endpoint: String,
    deployment: String,
    api_version: String,
}

#[derive(Serialize)]
struct AzureRequest {
    prompt: String,
}

#[derive(Deserialize)]
struct AzureResponse {
    choices: Vec<AzureChoice>,
}

#[derive(Deserialize)]
struct AzureChoice {
    text: String,
}

impl AzureProvider {
    pub fn new(api_key: String, endpoint: String, deployment: String) -> Self {
        Self {
            client: Client::new(),
            api_key,
            endpoint: endpoint.trim_end_matches('/').to_string(),
            deployment,
            api_version: "2024-02-01".to_string(),
        }
    }

    fn build_url(&self) -> String {
        format!(
            "{}/openai/deployments/{}/completions?api-version={}",
            self.endpoint, self.deployment, self.api_version
        )
    }

    fn build_chat_stream_url(&self) -> String {
        format!(
            "{}/openai/deployments/{}/chat/completions?api-version={}",
            self.endpoint, self.deployment, self.api_version
        )
    }
}

#[async_trait]
impl Provider for AzureProvider {
    async fn complete(&self, prompt: &str, _context: Option<&str>) -> Result<String, OpenCodeError> {
        let request = AzureRequest {
            prompt: prompt.to_string(),
        };

        let response = self
            .client
            .post(self.build_url())
            .header("Authorization", format!("Bearer {}", self.api_key))
            .header("Content-Type", "application/json")
            .json(&request)
            .send()
            .await
            .map_err(|e| OpenCodeError::Llm(e.to_string()))?;

        let completion: AzureResponse = response
            .json()
            .await
            .map_err(|e| OpenCodeError::Llm(e.to_string()))?;

        let content = completion
            .choices
            .first()
            .map(|c| c.text.clone())
            .unwrap_or_default();

        Ok(content)
    }

    async fn complete_streaming(&self, prompt: &str, mut callback: StreamingCallback) -> Result<(), OpenCodeError> {
        let body = serde_json::json!({
            "messages": [
                {"role": "user", "content": prompt}
            ],
            "stream": true,
        });

        let response = self
            .client
            .post(self.build_chat_stream_url())
            .header("Authorization", format!("Bearer {}", self.api_key))
            .header("api-key", &self.api_key)
            .header("Content-Type", "application/json")
            .json(&body)
            .send()
            .await
            .map_err(|e| OpenCodeError::Llm(e.to_string()))?;

        if !response.status().is_success() {
            let status = response.status();
            let error_text = response.text().await.unwrap_or_default();
            return Err(OpenCodeError::Llm(format!("Azure API error {}: {}", status, error_text)));
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
                            if let Some(content) = chunk["choices"][0]["delta"]["content"].as_str() {
                                callback(content.to_string());
                            }
                        }
                    }
                }
                Err(e) => return Err(OpenCodeError::Llm(format!("Azure stream error: {}", e))),
            }
        }

        Ok(())
    }
}
