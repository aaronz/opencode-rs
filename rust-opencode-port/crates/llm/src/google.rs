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

    async fn complete_streaming(&self, _prompt: &str, _callback: StreamingCallback) -> Result<(), OpenCodeError> {
        Err(OpenCodeError::Llm("Streaming not implemented".to_string()))
    }
}
