use async_trait::async_trait;
use reqwest::Client;
use serde::{Deserialize, Serialize};

use crate::provider::{Provider, StreamingCallback};
use opencode_core::OpenCodeError;

pub struct GoogleProvider {
    client: Client,
    api_key: String,
    base_url: String,
    model: String,
}

#[derive(Serialize)]
struct GoogleRequest {
    contents: Vec<GoogleContent>,
}

#[derive(Serialize)]
struct GoogleContent {
    parts: Vec<GooglePart>,
}

#[derive(Serialize)]
struct GooglePart {
    text: String,
}

#[derive(Deserialize)]
struct GoogleResponse {
    candidates: Option<Vec<GoogleCandidate>>,
}

#[derive(Deserialize)]
struct GoogleCandidate {
    content: Option<GoogleContentResponse>,
}

#[derive(Deserialize)]
struct GoogleContentResponse {
    parts: Option<Vec<GooglePartResponse>>,
}

#[derive(Deserialize)]
struct GooglePartResponse {
    text: Option<String>,
}

impl GoogleProvider {
    pub fn new(api_key: String, model: String) -> Self {
        Self {
            client: Client::new(),
            api_key,
            base_url: "https://generativelanguage.googleapis.com/v1beta".to_string(),
            model,
        }
    }
}

#[async_trait]
impl Provider for GoogleProvider {
    async fn complete(&self, prompt: &str, _context: Option<&str>) -> Result<String, OpenCodeError> {
        let contents = vec![GoogleContent {
            parts: vec![GooglePart {
                text: prompt.to_string(),
            }],
        }];

        let request = GoogleRequest { contents };

        let response = self
            .client
            .post(format!("{}/models/{}:generateContent?key={}", self.base_url, self.model, self.api_key))
            .header("Content-Type", "application/json")
            .json(&request)
            .send()
            .await
            .map_err(|e| OpenCodeError::Llm(e.to_string()))?;

        let result: GoogleResponse = response
            .json()
            .await
            .map_err(|e| OpenCodeError::Llm(e.to_string()))?;

        let content = match result.candidates {
            Some(candidates) => {
                if let Some(first_candidate) = candidates.first() {
                    if let Some(content) = &first_candidate.content {
                        if let Some(parts) = &content.parts {
                            if let Some(first_part) = parts.first() {
                                if let Some(text) = &first_part.text {
                                    return Ok(text.clone());
                                }
                            }
                        }
                    }
                }
                String::new()
            }
            None => String::new(),
        };

        Ok(content)
    }

    async fn complete_streaming(&self, prompt: &str, mut callback: StreamingCallback) -> Result<(), OpenCodeError> {
        let contents = vec![GoogleContent {
            parts: vec![GooglePart {
                text: prompt.to_string(),
            }],
        }];

        let request = GoogleRequest { contents };

        let response = self
            .client
            .post(format!(
                "{}/models/{}:streamGenerateContent?alt=sse&key={}",
                self.base_url, self.model, self.api_key
            ))
            .header("Content-Type", "application/json")
            .json(&request)
            .send()
            .await
            .map_err(|e| OpenCodeError::Llm(e.to_string()))?;

        if !response.status().is_success() {
            let status = response.status();
            let error_text = response.text().await.unwrap_or_default();
            return Err(OpenCodeError::Llm(format!("Google API error {}: {}", status, error_text)));
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

                        if let Ok(chunk) = serde_json::from_str::<GoogleResponse>(data) {
                            if let Some(candidates) = chunk.candidates {
                                for candidate in candidates {
                                    if let Some(content) = candidate.content {
                                        if let Some(parts) = content.parts {
                                            for part in parts {
                                                if let Some(text) = part.text {
                                                    callback(text);
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
                Err(e) => return Err(OpenCodeError::Llm(format!("Google stream error: {}", e))),
            }
        }

        callback(String::new());
        Ok(())
    }
}
