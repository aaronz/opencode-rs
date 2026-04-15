use crate::provider::sealed;
use async_trait::async_trait;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::sync::Mutex;

use crate::provider::{Provider, StreamingCallback};
use crate::{OpenAiBrowserAuthStore, OpenAiBrowserSession};
use opencode_core::OpenCodeError;

const OPENAI_API_BASE_URL: &str = "https://api.openai.com/v1";
const OPENAI_CODEX_RESPONSES_URL: &str = "https://chatgpt.com/backend-api/codex/responses";
const OPENAI_CODEX_MODELS_URL: &str = "https://chatgpt.com/backend-api/codex/models";
const OPENAI_AUTH_ISSUER: &str = "https://auth.openai.com";
const OPENAI_CODEX_CLIENT_ID: &str = "app_EMoamEEZ73f0CkXaXp7hrann";

pub struct OpenAiProvider {
    client: Client,
    api_key: String,
    base_url: String,
    model: String,
    auth_mode: OpenAiAuthMode,
    reasoning_effort: Option<String>,
}

enum OpenAiAuthMode {
    ApiKey,
    Browser {
        session: Mutex<OpenAiBrowserSession>,
        store: OpenAiBrowserAuthStore,
    },
}

#[derive(Serialize)]
struct ChatRequest {
    model: String,
    messages: Vec<Message>,
    stream: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    reasoning: Option<ReasoningRequest>,
}

#[derive(Serialize)]
struct ReasoningRequest {
    effort: String,
}

#[derive(Serialize)]
struct Message {
    role: String,
    content: String,
}

#[derive(Deserialize)]
struct ChatCompletion {
    choices: Vec<Choice>,
}

#[derive(Deserialize)]
struct Choice {
    message: MessageContent,
}

#[derive(Deserialize)]
struct MessageContent {
    content: String,
}

#[derive(Deserialize)]
struct StreamChunk {
    choices: Vec<StreamChoice>,
}

#[derive(Deserialize)]
struct StreamChoice {
    delta: StreamDelta,
}

#[derive(Deserialize)]
struct StreamDelta {
    content: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct BrowserAuthModelInfo {
    pub id: String,
    pub name: String,
}

#[derive(Serialize)]
struct ResponsesRequest {
    model: String,
    input: Vec<ResponsesInputMessage>,
    stream: bool,
}

#[derive(Serialize)]
struct ResponsesInputMessage {
    role: String,
    content: Vec<ResponsesInputContent>,
}

#[derive(Serialize)]
struct ResponsesInputContent {
    #[serde(rename = "type")]
    kind: String,
    text: String,
}

#[derive(Deserialize)]
struct ResponsesResponse {
    output: Vec<ResponsesOutputItem>,
}

#[derive(Deserialize)]
struct ResponsesOutputItem {
    #[serde(default)]
    content: Vec<ResponsesOutputContent>,
}

#[derive(Deserialize)]
struct ResponsesOutputContent {
    #[serde(default)]
    text: Option<String>,
}

#[derive(Deserialize)]
struct RefreshTokenResponse {
    access_token: String,
    refresh_token: String,
    expires_in: Option<i64>,
}

#[derive(Deserialize)]
#[serde(untagged)]
enum BrowserModelsResponse {
    Direct(Vec<BrowserAuthModelInfo>),
    Wrapped { models: Vec<BrowserAuthModelInfo> },
}

impl OpenAiProvider {
    pub fn new(api_key: String, model: String) -> Self {
        Self {
            client: Client::new(),
            api_key,
            base_url: OPENAI_API_BASE_URL.to_string(),
            model,
            auth_mode: OpenAiAuthMode::ApiKey,
            reasoning_effort: None,
        }
    }

    pub fn with_reasoning_effort(mut self, effort: String) -> Self {
        self.reasoning_effort = Some(effort);
        self
    }

    pub fn new_browser_auth(
        session: OpenAiBrowserSession,
        model: String,
        store: OpenAiBrowserAuthStore,
    ) -> Self {
        Self {
            client: Client::new(),
            api_key: session.access_token.clone(),
            base_url: OPENAI_CODEX_RESPONSES_URL.to_string(),
            model,
            auth_mode: OpenAiAuthMode::Browser {
                session: Mutex::new(session),
                store,
            },
            reasoning_effort: None,
        }
    }

    pub fn uses_browser_auth(&self) -> bool {
        matches!(self.auth_mode, OpenAiAuthMode::Browser { .. })
    }

    pub fn models_url(&self) -> &str {
        OPENAI_CODEX_MODELS_URL
    }

    async fn auth_context(&self) -> Result<(String, Option<String>), OpenCodeError> {
        match &self.auth_mode {
            OpenAiAuthMode::ApiKey => Ok((self.api_key.clone(), None)),
            OpenAiAuthMode::Browser { session, store } => {
                let needs_refresh = {
                    let current = session.lock().map_err(|_| {
                        OpenCodeError::Llm("OpenAI browser auth mutex poisoned".to_string())
                    })?;
                    current.is_expired()
                };

                if needs_refresh {
                    let refreshed = self.refresh_browser_session(session, store).await?;
                    return Ok((refreshed.access_token.clone(), refreshed.account_id.clone()));
                }

                let current = session.lock().map_err(|_| {
                    OpenCodeError::Llm("OpenAI browser auth mutex poisoned".to_string())
                })?;
                Ok((current.access_token.clone(), current.account_id.clone()))
            }
        }
    }

    async fn refresh_browser_session(
        &self,
        session: &Mutex<OpenAiBrowserSession>,
        store: &OpenAiBrowserAuthStore,
    ) -> Result<OpenAiBrowserSession, OpenCodeError> {
        let refresh_token = {
            let current = session.lock().map_err(|_| {
                OpenCodeError::Llm("OpenAI browser auth mutex poisoned".to_string())
            })?;
            current.refresh_token.clone()
        };

        let response = self
            .client
            .post(format!("{}/oauth/token", OPENAI_AUTH_ISSUER))
            .header("Content-Type", "application/x-www-form-urlencoded")
            .form(&[
                ("grant_type", "refresh_token"),
                ("refresh_token", refresh_token.as_str()),
                ("client_id", OPENAI_CODEX_CLIENT_ID),
            ])
            .send()
            .await
            .map_err(|e| {
                OpenCodeError::Llm(format!("Failed to refresh OpenAI browser token: {}", e))
            })?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            return Err(OpenCodeError::Llm(format!(
                "OpenAI browser token refresh failed {}: {}",
                status, body
            )));
        }

        let payload: RefreshTokenResponse = response.json().await.map_err(|e| {
            OpenCodeError::Llm(format!(
                "Failed to decode OpenAI browser refresh response: {}",
                e
            ))
        })?;

        let mut current = session
            .lock()
            .map_err(|_| OpenCodeError::Llm("OpenAI browser auth mutex poisoned".to_string()))?;
        current.access_token = payload.access_token;
        current.refresh_token = payload.refresh_token;
        current.expires_at_epoch_ms =
            chrono::Utc::now().timestamp_millis() + payload.expires_in.unwrap_or(3600) * 1000;
        store.save(&current.clone())?;
        Ok(current.clone())
    }

    async fn complete_browser_auth(&self, prompt: &str) -> Result<String, OpenCodeError> {
        let (access_token, account_id) = self.auth_context().await?;
        let request = ResponsesRequest {
            model: self.model.clone(),
            input: vec![ResponsesInputMessage {
                role: "user".to_string(),
                content: vec![ResponsesInputContent {
                    kind: "input_text".to_string(),
                    text: prompt.to_string(),
                }],
            }],
            stream: false,
        };

        let mut builder = self
            .client
            .post(&self.base_url)
            .header("Authorization", format!("Bearer {}", access_token))
            .header("Content-Type", "application/json")
            .json(&request);

        if let Some(account_id) = account_id {
            builder = builder.header("ChatGPT-Account-Id", account_id);
        }

        let response = builder.send().await.map_err(|e| {
            OpenCodeError::Llm(format!("OpenAI browser auth request failed: {}", e))
        })?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            return Err(OpenCodeError::Llm(format!(
                "OpenAI browser auth error {}: {}",
                status, body
            )));
        }

        let payload: ResponsesResponse = response.json().await.map_err(|e| {
            OpenCodeError::Llm(format!("Invalid OpenAI browser auth response: {}", e))
        })?;

        Ok(payload
            .output
            .iter()
            .flat_map(|item| item.content.iter())
            .filter_map(|content| content.text.clone())
            .collect::<Vec<_>>()
            .join(""))
    }

    pub async fn list_browser_auth_models(
        &self,
    ) -> Result<Vec<BrowserAuthModelInfo>, OpenCodeError> {
        let (access_token, account_id) = self.auth_context().await?;

        let mut request = self
            .client
            .get(self.models_url())
            .header("Authorization", format!("Bearer {}", access_token));

        if let Some(account_id) = account_id {
            request = request.header("ChatGPT-Account-Id", account_id);
        }

        let response = request.send().await.map_err(|e| {
            OpenCodeError::Llm(format!("Failed to fetch OpenAI browser-auth models: {}", e))
        })?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            return Err(OpenCodeError::Llm(format!(
                "OpenAI browser-auth models error {}: {}",
                status, body
            )));
        }

        let payload: BrowserModelsResponse = response.json().await.map_err(|e| {
            OpenCodeError::Llm(format!(
                "Invalid OpenAI browser-auth models response: {}",
                e
            ))
        })?;

        Ok(match payload {
            BrowserModelsResponse::Direct(models) => models,
            BrowserModelsResponse::Wrapped { models } => models,
        })
    }
}

impl sealed::Sealed for OpenAiProvider {}

#[async_trait]
impl Provider for OpenAiProvider {
    async fn complete(
        &self,
        prompt: &str,
        _context: Option<&str>,
    ) -> Result<String, OpenCodeError> {
        if self.uses_browser_auth() {
            return self.complete_browser_auth(prompt).await;
        }

        let messages = vec![Message {
            role: "user".to_string(),
            content: prompt.to_string(),
        }];

        let reasoning = self
            .reasoning_effort
            .as_ref()
            .map(|e| ReasoningRequest { effort: e.clone() });

        let request = ChatRequest {
            model: self.model.clone(),
            messages,
            stream: false,
            reasoning,
        };

        let response = self
            .client
            .post(format!("{}/chat/completions", self.base_url))
            .header("Authorization", format!("Bearer {}", self.api_key))
            .header("Content-Type", "application/json")
            .json(&request)
            .send()
            .await
            .map_err(|e| OpenCodeError::Llm(e.to_string()))?;

        if !response.status().is_success() {
            let status = response.status();
            let error_text = response.text().await.unwrap_or_default();
            return Err(OpenCodeError::Llm(format!(
                "OpenAI API error {}: {}",
                status, error_text
            )));
        }

        let completion: ChatCompletion = response
            .json()
            .await
            .map_err(|e| OpenCodeError::Llm(e.to_string()))?;

        let content = completion
            .choices
            .first()
            .map(|c| c.message.content.clone())
            .unwrap_or_default();

        Ok(content)
    }

    async fn complete_streaming(
        &self,
        prompt: &str,
        mut callback: StreamingCallback,
    ) -> Result<(), OpenCodeError> {
        if self.uses_browser_auth() {
            let content = self.complete_browser_auth(prompt).await?;
            callback(content);
            return Ok(());
        }

        let messages = vec![Message {
            role: "user".to_string(),
            content: prompt.to_string(),
        }];

        let reasoning = self
            .reasoning_effort
            .as_ref()
            .map(|e| ReasoningRequest { effort: e.clone() });

        let request = ChatRequest {
            model: self.model.clone(),
            messages,
            stream: true,
            reasoning,
        };

        let response = self
            .client
            .post(format!("{}/chat/completions", self.base_url))
            .header("Authorization", format!("Bearer {}", self.api_key))
            .header("Content-Type", "application/json")
            .json(&request)
            .send()
            .await
            .map_err(|e| OpenCodeError::Llm(e.to_string()))?;

        if !response.status().is_success() {
            let status = response.status();
            let error_text = response.text().await.unwrap_or_default();
            return Err(OpenCodeError::Llm(format!(
                "OpenAI API error {}: {}",
                status, error_text
            )));
        }

        let mut lines = response.bytes_stream();

        use futures_util::StreamExt;
        while let Some(item) = lines.next().await {
            match item {
                Ok(bytes) => {
                    let text = String::from_utf8_lossy(&bytes);
                    for line in text.lines() {
                        if line.starts_with("data: ") {
                            let data = line.strip_prefix("data: ").unwrap_or("");
                            if data == "[DONE]" {
                                callback(String::new());
                                return Ok(());
                            }
                            if let Ok(chunk) = serde_json::from_str::<StreamChunk>(data) {
                                if let Some(content) =
                                    chunk.choices.first().and_then(|c| c.delta.content.clone())
                                {
                                    callback(content);
                                }
                            }
                        }
                    }
                }
                Err(e) => {
                    return Err(OpenCodeError::Llm(format!("Stream error: {}", e)));
                }
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{OpenAiBrowserAuthStore, OpenAiBrowserSession};

    #[test]
    fn test_openai_provider_new() {
        let provider = OpenAiProvider::new("test-key".to_string(), "gpt-4".to_string());
        assert_eq!(provider.model, "gpt-4");
        assert_eq!(provider.api_key, "test-key");
    }

    #[test]
    fn openai_provider_uses_codex_endpoint_for_browser_auth() {
        let provider = OpenAiProvider::new_browser_auth(
            OpenAiBrowserSession {
                access_token: "access-token".into(),
                refresh_token: "refresh-token".into(),
                expires_at_epoch_ms: chrono::Utc::now().timestamp_millis() + 60_000,
                account_id: Some("acct_123".into()),
            },
            "gpt-5.3-codex".into(),
            OpenAiBrowserAuthStore::new(std::env::temp_dir()),
        );

        assert_eq!(provider.base_url, OPENAI_CODEX_RESPONSES_URL);
        assert!(provider.uses_browser_auth());
    }

    #[tokio::test]
    async fn test_openai_complete_returns_error_without_api_key() {
        let provider = OpenAiProvider::new("invalid-key".to_string(), "gpt-4".to_string());
        let result = provider.complete("test prompt", None).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_openai_streaming_returns_error_without_valid_key() {
        let provider = OpenAiProvider::new("invalid-key".to_string(), "gpt-4".to_string());
        let result = provider.complete_streaming("test", Box::new(|_| {})).await;
        assert!(result.is_err());
    }

    #[test]
    fn browser_auth_provider_exposes_model_list_endpoint() {
        let provider = OpenAiProvider::new_browser_auth(
            OpenAiBrowserSession {
                access_token: "access".into(),
                refresh_token: "refresh".into(),
                expires_at_epoch_ms: chrono::Utc::now().timestamp_millis() + 60_000,
                account_id: Some("acct_123".into()),
            },
            "gpt-5.3-codex".into(),
            OpenAiBrowserAuthStore::new(std::env::temp_dir()),
        );

        assert!(provider.uses_browser_auth());
        assert_eq!(
            provider.models_url(),
            "https://chatgpt.com/backend-api/codex/models"
        );
    }
}
