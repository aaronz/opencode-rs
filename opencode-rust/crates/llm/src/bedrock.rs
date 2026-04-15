use crate::provider::sealed;
use crate::provider::{Model, Provider, ProviderConfig, StreamingCallback};
use opencode_core::OpenCodeError;
use std::env;

pub struct BedrockProvider {
    config: ProviderConfig,
    region: String,
}

impl BedrockProvider {
    pub fn new(config: ProviderConfig, region: String) -> Self {
        Self { config, region }
    }

    pub fn from_env() -> Option<Self> {
        let region = env::var("AWS_DEFAULT_REGION")
            .or_else(|_| env::var("AWS_REGION"))
            .unwrap_or_else(|_| "us-east-1".to_string());

        let config = ProviderConfig {
            model: env::var("BEDROCK_MODEL")
                .unwrap_or_else(|_| "anthropic.claude-3-sonnet-20240229-v1:0".to_string()),
            api_key: env::var("AWS_BEARER_TOKEN_BEDROCK")
                .or_else(|_| env::var("AWS_ACCESS_KEY_ID"))
                .unwrap_or_default(),
            temperature: 0.7,
        };

        Some(Self::new(config, region))
    }

    fn resolve_credentials(&self) -> Result<BedrockCredentials, OpenCodeError> {
        if let Ok(token) = env::var("AWS_BEARER_TOKEN_BEDROCK") {
            if !token.is_empty() {
                return Ok(BedrockCredentials::BearerToken(token));
            }
        }

        if let (Ok(access_key), Ok(secret_key)) = (
            env::var("AWS_ACCESS_KEY_ID"),
            env::var("AWS_SECRET_ACCESS_KEY"),
        ) {
            let session_token = env::var("AWS_SESSION_TOKEN").ok();
            return Ok(BedrockCredentials::AccessKey {
                access_key,
                secret_key,
                session_token,
            });
        }

        if let Ok(profile) = env::var("AWS_PROFILE") {
            return Ok(BedrockCredentials::Profile(profile));
        }

        if let (Ok(token_file), Ok(role_arn)) = (
            env::var("AWS_WEB_IDENTITY_TOKEN_FILE"),
            env::var("AWS_ROLE_ARN"),
        ) {
            return Ok(BedrockCredentials::Oidc {
                token_file,
                role_arn,
                session_name: env::var("AWS_ROLE_SESSION_NAME")
                    .unwrap_or_else(|_| "opencode".to_string()),
            });
        }

        Err(OpenCodeError::Llm(
            "No AWS credentials found. Set AWS_BEARER_TOKEN_BEDROCK, AWS_ACCESS_KEY_ID/AWS_SECRET_ACCESS_KEY, or AWS_PROFILE".to_string()
        ))
    }
}

#[derive(Debug)]
enum BedrockCredentials {
    BearerToken(String),
    AccessKey {
        access_key: String,
        secret_key: String,
        session_token: Option<String>,
    },
    Profile(String),
    Oidc {
        token_file: String,
        role_arn: String,
        session_name: String,
    },
}

impl sealed::Sealed for BedrockProvider {}

#[async_trait::async_trait]
impl Provider for BedrockProvider {
    async fn complete(
        &self,
        prompt: &str,
        _context: Option<&str>,
    ) -> Result<String, OpenCodeError> {
        let credentials = self.resolve_credentials()?;

        match credentials {
            BedrockCredentials::BearerToken(token) => {
                self.complete_with_bearer(&token, prompt).await
            }
            BedrockCredentials::AccessKey {
                access_key,
                secret_key,
                session_token,
            } => {
                self.complete_with_aws_sigv4(
                    &access_key,
                    &secret_key,
                    session_token.as_deref(),
                    prompt,
                )
                .await
            }
            BedrockCredentials::Profile(profile) => {
                self.complete_with_profile(&profile, prompt).await
            }
            BedrockCredentials::Oidc {
                token_file,
                role_arn,
                session_name,
            } => {
                self.complete_with_oidc(&token_file, &role_arn, &session_name, prompt)
                    .await
            }
        }
    }

    async fn complete_streaming(
        &self,
        prompt: &str,
        mut callback: StreamingCallback,
    ) -> Result<(), OpenCodeError> {
        let content = self.complete(prompt, None).await?;
        callback(content);
        Ok(())
    }

    fn get_models(&self) -> Vec<Model> {
        vec![
            Model::new(
                "anthropic.claude-3-5-sonnet-20241022-v2:0",
                "Claude 3.5 Sonnet",
            ),
            Model::new("anthropic.claude-3-sonnet-20240229-v1:0", "Claude 3 Sonnet"),
            Model::new("anthropic.claude-3-haiku-20240307-v1:0", "Claude 3 Haiku"),
            Model::new("anthropic.claude-3-opus-20240229-v1:0", "Claude 3 Opus"),
            Model::new("meta.llama3-70b-instruct-v1:0", "Llama 3 70B"),
            Model::new("meta.llama3-1-70b-instruct-v1:0", "Llama 3.1 70B"),
            Model::new("amazon.titan-text-express-v1", "Titan Text Express"),
            Model::new("mistral.mistral-large-2402-v1:0", "Mistral Large"),
        ]
    }

    fn provider_name(&self) -> &str {
        "bedrock"
    }
}

impl BedrockProvider {
    async fn complete_with_bearer(
        &self,
        token: &str,
        prompt: &str,
    ) -> Result<String, OpenCodeError> {
        let client = reqwest::Client::new();
        let url = format!(
            "https://bedrock-runtime.{}.amazonaws.com/model/{}/invoke",
            self.region, self.config.model
        );

        let body = serde_json::json!({
            "prompt": prompt,
            "max_tokens_to_sample": 4096,
            "temperature": self.config.temperature,
        });

        let response = client
            .post(&url)
            .header("Authorization", format!("Bearer {}", token))
            .header("Content-Type", "application/json")
            .json(&body)
            .send()
            .await
            .map_err(|e| OpenCodeError::Llm(format!("Bedrock request failed: {}", e)))?;

        if !response.status().is_success() {
            return Err(OpenCodeError::Llm(format!(
                "Bedrock API error {}: {}",
                response.status(),
                response.text().await.unwrap_or_default()
            )));
        }

        let result: serde_json::Value = response
            .json()
            .await
            .map_err(|e| OpenCodeError::Llm(format!("Failed to parse Bedrock response: {}", e)))?;

        result
            .get("completion")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string())
            .or_else(|| {
                result
                    .get("content")
                    .and_then(|c| c.as_array())
                    .and_then(|c| c.first())
                    .and_then(|c| c.get("text"))
                    .and_then(|t| t.as_str())
                    .map(|s| s.to_string())
            })
            .ok_or_else(|| OpenCodeError::Llm("Invalid Bedrock response".to_string()))
    }

    async fn complete_with_aws_sigv4(
        &self,
        _access_key: &str,
        _secret_key: &str,
        _session_token: Option<&str>,
        prompt: &str,
    ) -> Result<String, OpenCodeError> {
        self.complete_with_bearer("sigv4-placeholder", prompt).await
    }

    async fn complete_with_profile(
        &self,
        _profile: &str,
        prompt: &str,
    ) -> Result<String, OpenCodeError> {
        self.complete_with_bearer("profile-placeholder", prompt)
            .await
    }

    async fn complete_with_oidc(
        &self,
        _token_file: &str,
        _role_arn: &str,
        _session_name: &str,
        prompt: &str,
    ) -> Result<String, OpenCodeError> {
        self.complete_with_bearer("oidc-placeholder", prompt).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bedrock_provider_metadata() {
        let provider = BedrockProvider::new(
            ProviderConfig {
                model: "anthropic.claude-3-sonnet".to_string(),
                api_key: "test-key".to_string(),
                temperature: 0.7,
            },
            "us-east-1".to_string(),
        );

        assert_eq!(provider.provider_name(), "bedrock");
        assert!(!provider.get_models().is_empty());
    }

    #[test]
    fn test_bedrock_credential_resolution_bearer_token_priority() {
        let old_token = env::var("AWS_BEARER_TOKEN_BEDROCK").ok();
        let old_access = env::var("AWS_ACCESS_KEY_ID").ok();
        env::set_var("AWS_BEARER_TOKEN_BEDROCK", "bearer-token-123");
        env::set_var("AWS_ACCESS_KEY_ID", "AKIA123");
        env::set_var("AWS_SECRET_ACCESS_KEY", "secret123");

        let provider = BedrockProvider::new(ProviderConfig::default(), "us-east-1".to_string());
        let creds = provider.resolve_credentials().unwrap();
        assert!(matches!(creds, BedrockCredentials::BearerToken(_)));

        if let Some(old) = old_token {
            env::set_var("AWS_BEARER_TOKEN_BEDROCK", old);
        } else {
            env::remove_var("AWS_BEARER_TOKEN_BEDROCK");
        }
        if let Some(old) = old_access {
            env::set_var("AWS_ACCESS_KEY_ID", old);
        } else {
            env::remove_var("AWS_ACCESS_KEY_ID");
        }
    }

    #[test]
    fn test_bedrock_credential_resolution_access_key_fallback() {
        let old_token = env::var("AWS_BEARER_TOKEN_BEDROCK").ok();
        let old_access = env::var("AWS_ACCESS_KEY_ID").ok();
        let old_secret = env::var("AWS_SECRET_ACCESS_KEY").ok();
        env::remove_var("AWS_BEARER_TOKEN_BEDROCK");
        env::set_var("AWS_ACCESS_KEY_ID", "AKIA456");
        env::set_var("AWS_SECRET_ACCESS_KEY", "secret456");

        let provider = BedrockProvider::new(ProviderConfig::default(), "us-east-1".to_string());
        let creds = provider.resolve_credentials().unwrap();
        assert!(matches!(creds, BedrockCredentials::AccessKey { .. }));

        if let Some(old) = old_token {
            env::set_var("AWS_BEARER_TOKEN_BEDROCK", old);
        } else {
            env::remove_var("AWS_BEARER_TOKEN_BEDROCK");
        }
        if let Some(old) = old_access {
            env::set_var("AWS_ACCESS_KEY_ID", old);
        } else {
            env::remove_var("AWS_ACCESS_KEY_ID");
        }
        if let Some(old) = old_secret {
            env::set_var("AWS_SECRET_ACCESS_KEY", old);
        } else {
            env::remove_var("AWS_SECRET_ACCESS_KEY");
        }
    }
}
