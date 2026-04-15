use crate::provider::{Model, Provider, ProviderConfig, StreamingCallback};
use crate::provider::sealed;
use opencode_core::OpenCodeError;
use reqwest::Client;
use serde_json::Value;
use std::env;

pub struct SapAiCoreProvider {
    client: Client,
    config: ProviderConfig,
    service_key: SapAiCoreServiceKey,
}

#[derive(Debug, Clone)]
pub struct SapAiCoreServiceKey {
    pub client_id: String,
    pub client_secret: String,
    pub url: String,
    pub resource_group: Option<String>,
}

impl SapAiCoreServiceKey {
    pub fn from_env() -> Option<Self> {
        let service_key_json = env::var("AICORE_SERVICE_KEY").ok()?;
        let key: serde_json::Value = serde_json::from_str(&service_key_json).ok()?;

        Some(Self {
            client_id: key.get("clientid").and_then(|v| v.as_str())?.to_string(),
            client_secret: key
                .get("clientsecret")
                .and_then(|v| v.as_str())?
                .to_string(),
            url: key.get("url").and_then(|v| v.as_str())?.to_string(),
            resource_group: key
                .get("service_group")
                .or_else(|| key.get("resource_group"))
                .and_then(|v| v.as_str())
                .map(|s| s.to_string()),
        })
    }
}

impl SapAiCoreProvider {
    pub fn new(config: ProviderConfig, service_key: SapAiCoreServiceKey) -> Self {
        Self {
            client: Client::new(),
            config,
            service_key,
        }
    }

    pub fn from_env() -> Option<Self> {
        let service_key = SapAiCoreServiceKey::from_env()?;
        let config = ProviderConfig {
            model: env::var("SAP_AI_CORE_MODEL").unwrap_or_else(|_| "gpt-4".to_string()),
            api_key: String::new(),
            temperature: 0.7,
        };
        Some(Self::new(config, service_key))
    }

    async fn get_access_token(&self) -> Result<String, OpenCodeError> {
        let token_url = format!("{}/oauth/token", self.service_key.url);
        let response = self
            .client
            .post(&token_url)
            .form(&[
                ("grant_type", "client_credentials"),
                ("client_id", &self.service_key.client_id),
                ("client_secret", &self.service_key.client_secret),
            ])
            .send()
            .await
            .map_err(|e| OpenCodeError::Llm(format!("SAP AI Core token request failed: {}", e)))?;

        if !response.status().is_success() {
            return Err(OpenCodeError::Llm(format!(
                "SAP AI Core token request failed: {}",
                response.status()
            )));
        }

        let token_response: Value = response.json().await.map_err(|e| {
            OpenCodeError::Llm(format!("Failed to parse SAP AI Core token response: {}", e))
        })?;

        token_response
            .get("access_token")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string())
            .ok_or_else(|| {
                OpenCodeError::Llm("SAP AI Core token response missing access_token".to_string())
            })
    }
}

impl sealed::Sealed for SapAiCoreProvider {}

#[async_trait::async_trait]
impl Provider for SapAiCoreProvider {
    async fn complete(&self, prompt: &str, context: Option<&str>) -> Result<String, OpenCodeError> {
        let token = self.get_access_token().await?;
        let api_url = format!(
            "{}/inference/deployments/{}/completions",
            self.service_key.url, self.config.model
        );

        let input = if let Some(ctx) = context {
            format!("{}\n\n{}", ctx, prompt)
        } else {
            prompt.to_string()
        };

        let mut request = self
            .client
            .post(&api_url)
            .header("Authorization", format!("Bearer {}", token))
            .header("Content-Type", "application/json")
            .header(
                "AI-Resource-Group",
                self.service_key
                    .resource_group
                    .as_deref()
                    .unwrap_or("default"),
            );

        if let Some(rg) = &self.service_key.resource_group {
            request = request.header("AI-Resource-Group", rg);
        }

        let body = serde_json::json!({
            "messages": [
                {"role": "user", "content": input}
            ],
            "temperature": self.config.temperature,
        });

        let response = request
            .json(&body)
            .send()
            .await
            .map_err(|e| OpenCodeError::Llm(format!("SAP AI Core request failed: {}", e)))?;

        if !response.status().is_success() {
            return Err(OpenCodeError::Llm(format!(
                "SAP AI Core API error {}: {}",
                response.status(),
                response.text().await.unwrap_or_default()
            )));
        }

        let payload: Value = response.json().await.map_err(|e| {
            OpenCodeError::Llm(format!("Failed to parse SAP AI Core response: {}", e))
        })?;

        payload
            .get("choices")
            .and_then(|c| c.as_array())
            .and_then(|c| c.first())
            .and_then(|c| c.get("message"))
            .and_then(|m| m.get("content"))
            .and_then(|c| c.as_str())
            .map(|s| s.to_string())
            .ok_or_else(|| OpenCodeError::Llm("Invalid SAP AI Core response".to_string()))
    }

    async fn complete_streaming(
        &self,
        _prompt: &str,
        mut _callback: StreamingCallback,
    ) -> Result<(), OpenCodeError> {
        Err(OpenCodeError::Llm(
            "SAP AI Core streaming not yet supported".to_string(),
        ))
    }

    fn get_models(&self) -> Vec<Model> {
        vec![
            Model::new("gpt-4", "GPT-4"),
            Model::new("gpt-35-turbo", "GPT-3.5 Turbo"),
            Model::new("claude-3-sonnet", "Claude 3 Sonnet"),
        ]
    }

    fn provider_name(&self) -> &str {
        "sap-ai-core"
    }
}

pub struct CloudflareProvider {
    client: Client,
    config: ProviderConfig,
    account_id: String,
    gateway_id: Option<String>,
}

impl CloudflareProvider {
    pub fn new(config: ProviderConfig, account_id: String, gateway_id: Option<String>) -> Self {
        Self {
            client: Client::new(),
            config,
            account_id,
            gateway_id,
        }
    }

    pub fn from_env() -> Option<Self> {
        let account_id = env::var("CLOUDFLARE_ACCOUNT_ID").ok()?;
        let gateway_id = env::var("CLOUDFLARE_AI_GATEWAY_ID").ok();
        let config = ProviderConfig {
            model: env::var("CLOUDFLARE_MODEL")
                .unwrap_or_else(|_| "@cf/meta/llama-3.1-8b-instruct".to_string()),
            api_key: env::var("CLOUDFLARE_API_TOKEN").unwrap_or_default(),
            temperature: 0.7,
        };
        Some(Self::new(config, account_id, gateway_id))
    }

    fn api_url(&self) -> String {
        if let Some(ref gateway_id) = self.gateway_id {
            format!(
                "https://gateway.ai.cloudflare.com/v1/{}/{}/workers-ai/{}",
                self.account_id, gateway_id, self.config.model
            )
        } else {
            format!(
                "https://api.cloudflare.com/client/v4/accounts/{}/ai/run/{}",
                self.account_id, self.config.model
            )
        }
    }
}

impl sealed::Sealed for CloudflareProvider {}

#[async_trait::async_trait]
impl Provider for CloudflareProvider {
    async fn complete(&self, prompt: &str, context: Option<&str>) -> Result<String, OpenCodeError> {
        let input = if let Some(ctx) = context {
            format!("{}\n\n{}", ctx, prompt)
        } else {
            prompt.to_string()
        };

        let body = serde_json::json!({
            "messages": [
                {"role": "user", "content": input}
            ]
        });

        let response = self
            .client
            .post(self.api_url())
            .header("Authorization", format!("Bearer {}", self.config.api_key))
            .header("Content-Type", "application/json")
            .json(&body)
            .send()
            .await
            .map_err(|e| OpenCodeError::Llm(format!("Cloudflare request failed: {}", e)))?;

        if !response.status().is_success() {
            return Err(OpenCodeError::Llm(format!(
                "Cloudflare API error {}: {}",
                response.status(),
                response.text().await.unwrap_or_default()
            )));
        }

        let payload: Value = response.json().await.map_err(|e| {
            OpenCodeError::Llm(format!("Failed to parse Cloudflare response: {}", e))
        })?;

        payload
            .get("result")
            .and_then(|r| r.get("response"))
            .and_then(|r| r.as_str())
            .map(|s| s.to_string())
            .or_else(|| {
                payload
                    .get("response")
                    .and_then(|r| r.as_str())
                    .map(|s| s.to_string())
            })
            .ok_or_else(|| OpenCodeError::Llm("Invalid Cloudflare response".to_string()))
    }

    async fn complete_streaming(
        &self,
        _prompt: &str,
        mut _callback: StreamingCallback,
    ) -> Result<(), OpenCodeError> {
        Err(OpenCodeError::Llm(
            "Cloudflare streaming not yet supported".to_string(),
        ))
    }

    fn get_models(&self) -> Vec<Model> {
        vec![
            Model::new("@cf/meta/llama-3.1-8b-instruct", "Llama 3.1 8B"),
            Model::new("@cf/meta/llama-3.1-70b-instruct", "Llama 3.1 70B"),
            Model::new("@cf/qwen/qwen1.5-14b-chat-awq", "Qwen 1.5 14B"),
            Model::new("@cf/mistral/mistral-7b-instruct-v0.2", "Mistral 7B"),
        ]
    }

    fn provider_name(&self) -> &str {
        "cloudflare"
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cloudflare_provider_metadata() {
        let provider = CloudflareProvider::new(
            ProviderConfig {
                model: "@cf/meta/llama-3.1-8b-instruct".to_string(),
                api_key: "test-key".to_string(),
                temperature: 0.5,
            },
            "test-account".to_string(),
            Some("test-gateway".to_string()),
        );

        assert_eq!(provider.provider_name(), "cloudflare");
        assert_eq!(provider.get_models().len(), 4);
        assert!(provider.api_url().contains("gateway.ai.cloudflare.com"));
    }

    #[test]
    fn test_cloudflare_provider_without_gateway() {
        let provider = CloudflareProvider::new(
            ProviderConfig {
                model: "@cf/meta/llama-3.1-8b-instruct".to_string(),
                api_key: "test-key".to_string(),
                temperature: 0.5,
            },
            "test-account".to_string(),
            None,
        );

        assert!(provider.api_url().contains("api.cloudflare.com"));
    }

    #[test]
    fn test_sap_ai_core_service_key_from_env() {
        let old = env::var("AICORE_SERVICE_KEY").ok();
        env::set_var(
            "AICORE_SERVICE_KEY",
            r#"{"clientid":"cid","clientsecret":"csec","url":"https://api.ai.sap.com","service_group":"my-group"}"#,
        );

        let key = SapAiCoreServiceKey::from_env();
        assert!(key.is_some());
        let key = key.unwrap();
        assert_eq!(key.client_id, "cid");
        assert_eq!(key.client_secret, "csec");
        assert_eq!(key.url, "https://api.ai.sap.com");
        assert_eq!(key.resource_group, Some("my-group".to_string()));

        if let Some(old) = old {
            env::set_var("AICORE_SERVICE_KEY", old);
        } else {
            env::remove_var("AICORE_SERVICE_KEY");
        }
    }
}
