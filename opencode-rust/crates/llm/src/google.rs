use async_trait::async_trait;
use reqwest::Client;
use serde::{Deserialize, Serialize};

use crate::provider::sealed;
use crate::provider::{Provider, StreamingCallback};
use opencode_core::OpenCodeError;

pub struct GoogleProvider {
    client: Client,
    api_key: String,
    base_url: String,
    model: String,
    thinking_throttle: Option<String>,
}

#[derive(Serialize)]
struct GoogleRequest {
    contents: Vec<GoogleContent>,
    #[serde(skip_serializing_if = "Option::is_none")]
    thinking_config: Option<GoogleThinkingConfig>,
}

#[derive(Serialize)]
struct GoogleThinkingConfig {
    thinking_throttle: String,
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
            thinking_throttle: None,
        }
    }

    pub fn with_thinking_throttle(mut self, throttle: String) -> Self {
        self.thinking_throttle = Some(throttle);
        self
    }
}

impl sealed::Sealed for GoogleProvider {}

#[async_trait]
impl Provider for GoogleProvider {
    async fn complete(
        &self,
        prompt: &str,
        _context: Option<&str>,
    ) -> Result<String, OpenCodeError> {
        let contents = vec![GoogleContent {
            parts: vec![GooglePart {
                text: prompt.to_string(),
            }],
        }];

        let thinking_config = self
            .thinking_throttle
            .as_ref()
            .map(|t| GoogleThinkingConfig {
                thinking_throttle: t.clone(),
            });

        let request = GoogleRequest {
            contents,
            thinking_config,
        };

        let response = self
            .client
            .post(format!(
                "{}/models/{}:generateContent?key={}",
                self.base_url, self.model, self.api_key
            ))
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

    async fn complete_streaming(
        &self,
        prompt: &str,
        mut callback: StreamingCallback,
    ) -> Result<(), OpenCodeError> {
        let contents = vec![GoogleContent {
            parts: vec![GooglePart {
                text: prompt.to_string(),
            }],
        }];

        let thinking_config = self
            .thinking_throttle
            .as_ref()
            .map(|t| GoogleThinkingConfig {
                thinking_throttle: t.clone(),
            });

        let request = GoogleRequest {
            contents,
            thinking_config,
        };

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
            return Err(OpenCodeError::Llm(format!(
                "Google API error {}: {}",
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn google_provider_new() {
        let provider = GoogleProvider::new("test-key".to_string(), "gemini-pro".to_string());
        assert_eq!(provider.model, "gemini-pro");
        assert_eq!(provider.api_key, "test-key");
        assert!(provider.thinking_throttle.is_none());
    }

    #[test]
    fn google_provider_with_thinking_throttle() {
        let provider = GoogleProvider::new("test-key".to_string(), "gemini-pro".to_string())
            .with_thinking_throttle("low".to_string());
        assert_eq!(provider.thinking_throttle, Some("low".to_string()));
    }

    #[test]
    fn google_request_serialization() {
        let request = GoogleRequest {
            contents: vec![GoogleContent {
                parts: vec![GooglePart {
                    text: "Hello".to_string(),
                }],
            }],
            thinking_config: None,
        };
        let json = serde_json::to_string(&request).unwrap();
        assert!(json.contains("Hello"));
    }

    #[test]
    fn google_request_serialization_with_thinking() {
        let request = GoogleRequest {
            contents: vec![GoogleContent {
                parts: vec![GooglePart {
                    text: "Hello".to_string(),
                }],
            }],
            thinking_config: Some(GoogleThinkingConfig {
                thinking_throttle: "low".to_string(),
            }),
        };
        let json = serde_json::to_string(&request).unwrap();
        assert!(json.contains("thinking_throttle"));
        assert!(json.contains("low"));
    }

    #[test]
    fn google_response_deserialization() {
        let json = r#"{"candidates":[{"content":{"parts":[{"text":"Hello"}]}}]}"#;
        let response: GoogleResponse = serde_json::from_str(json).unwrap();
        assert!(response.candidates.is_some());
        let candidates = response.candidates.unwrap();
        assert_eq!(candidates.len(), 1);
    }

    #[test]
    fn google_response_deserialization_empty_candidates() {
        let json = r#"{"candidates":[]}"#;
        let response: GoogleResponse = serde_json::from_str(json).unwrap();
        assert!(response.candidates.is_some());
        assert!(response.candidates.unwrap().is_empty());
    }

    #[test]
    fn google_response_deserialization_no_candidates() {
        let json = r#"{}"#;
        let response: GoogleResponse = serde_json::from_str(json).unwrap();
        assert!(response.candidates.is_none());
    }
}
