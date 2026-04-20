use crate::provider::sealed;
use crate::provider::{Model, Provider, ProviderConfig, StreamingCallback};
use opencode_core::OpenCodeError;
use std::env;

/// AWS region prefix extracted from model ID
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RegionPrefix {
    Us,
    Eu,
    Apac,
    Au,
    Jp,
}

impl RegionPrefix {
    /// Returns the Bedrock endpoint URL for this region
    pub fn endpoint(&self) -> &'static str {
        match self {
            RegionPrefix::Us => "https://bedrock.us-east-1.amazonaws.com",
            RegionPrefix::Eu => "https://bedrock.eu-west-1.amazonaws.com",
            RegionPrefix::Apac | RegionPrefix::Au | RegionPrefix::Jp => {
                "https://bedrock.ap-northeast-1.amazonaws.com"
            }
        }
    }
}

/// Known region prefixes
const REGION_PREFIXES: &[&str] = &["us", "eu", "jp", "apac", "au"];

/// Extracts the region prefix from a Bedrock model ID.
///
/// The model ID format is typically: `{provider}.{model}` or `{region}.{provider}.{model}`
/// where the region prefix is the first dot-separated component.
///
/// # Examples
/// ```
/// assert_eq!(get_region_prefix("us.amazon.nova-pro"), Some("us"));
/// assert_eq!(get_region_prefix("eu.claude-3-sonnet"), Some("eu"));
/// assert_eq!(get_region_prefix("anthropic.claude-3-sonnet"), None);
/// ```
pub fn get_region_prefix(model_id: &str) -> Option<&str> {
    let prefix = model_id.split('.').next()?;
    if REGION_PREFIXES.contains(&prefix) {
        Some(prefix)
    } else {
        None
    }
}

/// Returns the Bedrock endpoint URL for a given model ID based on its region prefix.
///
/// If the model ID has a recognized region prefix (us, eu, jp, apac, au), returns
/// the appropriate regional endpoint. Otherwise, returns the default US endpoint.
///
/// # Examples
/// ```
/// assert_eq!(get_bedrock_endpoint("us.amazon.nova-pro"),
///            "https://bedrock.us-east-1.amazonaws.com");
/// assert_eq!(get_bedrock_endpoint("eu.claude-3-sonnet"),
///            "https://bedrock.eu-west-1.amazonaws.com");
/// assert_eq!(get_bedrock_endpoint("anthropic.claude-3-sonnet"),
///            "https://bedrock.us-east-1.amazonaws.com"); // default
/// ```
pub fn get_bedrock_endpoint(model_id: &str) -> &'static str {
    let prefix = get_region_prefix(model_id);
    match prefix {
        Some("eu") => RegionPrefix::Eu.endpoint(),
        Some("jp") | Some("apac") | Some("au") => RegionPrefix::Apac.endpoint(),
        _ => RegionPrefix::Us.endpoint(),
    }
}

/// Default Bedrock endpoint (US)
pub const DEFAULT_BEDROCK_ENDPOINT: &str = "https://bedrock.us-east-1.amazonaws.com";

pub struct BedrockProvider {
    config: ProviderConfig,
    #[allow(dead_code)]
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
        let endpoint = get_bedrock_endpoint(&self.config.model);
        let runtime_url = format!(
            "{}/model/{}/invoke",
            endpoint.replace("https://bedrock", "https://bedrock-runtime"),
            self.config.model
        );

        let body = serde_json::json!({
            "prompt": prompt,
            "max_tokens_to_sample": 4096,
            "temperature": self.config.temperature,
        });

        let response = client
            .post(&runtime_url)
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
        let old_session = env::var("AWS_SESSION_TOKEN").ok();
        env::remove_var("AWS_BEARER_TOKEN_BEDROCK");
        env::set_var("AWS_ACCESS_KEY_ID", "AKIA456");
        env::set_var("AWS_SECRET_ACCESS_KEY", "secret456");
        env::remove_var("AWS_SESSION_TOKEN");

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
        if let Some(old) = old_session {
            env::set_var("AWS_SESSION_TOKEN", old);
        } else {
            env::remove_var("AWS_SESSION_TOKEN");
        }
    }

    #[test]
    fn test_bedrock_credential_resolution_profile() {
        let old_token = env::var("AWS_BEARER_TOKEN_BEDROCK").ok();
        let old_access = env::var("AWS_ACCESS_KEY_ID").ok();
        let old_secret = env::var("AWS_SECRET_ACCESS_KEY").ok();
        let old_web_identity = env::var("AWS_WEB_IDENTITY_TOKEN_FILE").ok();
        let old_role = env::var("AWS_ROLE_ARN").ok();
        env::remove_var("AWS_BEARER_TOKEN_BEDROCK");
        env::remove_var("AWS_ACCESS_KEY_ID");
        env::remove_var("AWS_SECRET_ACCESS_KEY");
        env::remove_var("AWS_WEB_IDENTITY_TOKEN_FILE");
        env::remove_var("AWS_ROLE_ARN");
        env::set_var("AWS_PROFILE", "my-profile");

        let provider = BedrockProvider::new(ProviderConfig::default(), "us-east-1".to_string());
        let creds = provider.resolve_credentials().unwrap();
        assert!(matches!(creds, BedrockCredentials::Profile(p) if p == "my-profile"));

        if let Some(old) = old_token {
            env::set_var("AWS_BEARER_TOKEN_BEDROCK", old);
        }
        if let Some(old) = old_access {
            env::set_var("AWS_ACCESS_KEY_ID", old);
        }
        if let Some(old) = old_secret {
            env::set_var("AWS_SECRET_ACCESS_KEY", old);
        }
        if let Some(old) = old_web_identity {
            env::set_var("AWS_WEB_IDENTITY_TOKEN_FILE", old);
        }
        if let Some(old) = old_role {
            env::set_var("AWS_ROLE_ARN", old);
        }
        env::remove_var("AWS_PROFILE");
    }

    #[test]
    fn test_bedrock_credential_resolution_oidc() {
        let old_token = env::var("AWS_BEARER_TOKEN_BEDROCK").ok();
        let old_access = env::var("AWS_ACCESS_KEY_ID").ok();
        let old_secret = env::var("AWS_SECRET_ACCESS_KEY").ok();
        let old_profile = env::var("AWS_PROFILE").ok();
        let old_web_identity = env::var("AWS_WEB_IDENTITY_TOKEN_FILE").ok();
        let old_role = env::var("AWS_ROLE_ARN").ok();
        let old_session_name = env::var("AWS_ROLE_SESSION_NAME").ok();
        env::remove_var("AWS_BEARER_TOKEN_BEDROCK");
        env::remove_var("AWS_ACCESS_KEY_ID");
        env::remove_var("AWS_SECRET_ACCESS_KEY");
        env::remove_var("AWS_PROFILE");
        env::set_var("AWS_WEB_IDENTITY_TOKEN_FILE", "/path/to/token");
        env::set_var("AWS_ROLE_ARN", "arn:aws:iam::123456789012:role/MyRole");
        env::set_var("AWS_ROLE_SESSION_NAME", "my-session");

        let provider = BedrockProvider::new(ProviderConfig::default(), "us-east-1".to_string());
        let creds = provider.resolve_credentials().unwrap();
        match creds {
            BedrockCredentials::Oidc {
                token_file,
                role_arn,
                session_name,
            } => {
                assert_eq!(token_file, "/path/to/token");
                assert_eq!(role_arn, "arn:aws:iam::123456789012:role/MyRole");
                assert_eq!(session_name, "my-session");
            }
            other => panic!("Expected Oidc credentials, got {:?}", other),
        }

        if let Some(old) = old_token {
            env::set_var("AWS_BEARER_TOKEN_BEDROCK", old);
        }
        if let Some(old) = old_access {
            env::set_var("AWS_ACCESS_KEY_ID", old);
        }
        if let Some(old) = old_secret {
            env::set_var("AWS_SECRET_ACCESS_KEY", old);
        }
        if let Some(old) = old_profile {
            env::set_var("AWS_PROFILE", old);
        }
        if let Some(old) = old_web_identity {
            env::set_var("AWS_WEB_IDENTITY_TOKEN_FILE", old);
        } else {
            env::remove_var("AWS_WEB_IDENTITY_TOKEN_FILE");
        }
        if let Some(old) = old_role {
            env::set_var("AWS_ROLE_ARN", old);
        } else {
            env::remove_var("AWS_ROLE_ARN");
        }
        if let Some(old) = old_session_name {
            env::set_var("AWS_ROLE_SESSION_NAME", old);
        } else {
            env::remove_var("AWS_ROLE_SESSION_NAME");
        }
    }

    #[test]
    fn test_bedrock_credential_resolution_no_credentials() {
        let old_token = env::var("AWS_BEARER_TOKEN_BEDROCK").ok();
        let old_access = env::var("AWS_ACCESS_KEY_ID").ok();
        let old_secret = env::var("AWS_SECRET_ACCESS_KEY").ok();
        let old_profile = env::var("AWS_PROFILE").ok();
        let old_web_identity = env::var("AWS_WEB_IDENTITY_TOKEN_FILE").ok();
        let old_role = env::var("AWS_ROLE_ARN").ok();
        let old_session = env::var("AWS_SESSION_TOKEN").ok();
        env::remove_var("AWS_BEARER_TOKEN_BEDROCK");
        env::remove_var("AWS_ACCESS_KEY_ID");
        env::remove_var("AWS_SECRET_ACCESS_KEY");
        env::remove_var("AWS_PROFILE");
        env::remove_var("AWS_WEB_IDENTITY_TOKEN_FILE");
        env::remove_var("AWS_ROLE_ARN");
        env::remove_var("AWS_SESSION_TOKEN");

        let provider = BedrockProvider::new(ProviderConfig::default(), "us-east-1".to_string());
        let result = provider.resolve_credentials();
        assert!(result.is_err());

        if let Some(old) = old_token {
            env::set_var("AWS_BEARER_TOKEN_BEDROCK", old);
        }
        if let Some(old) = old_access {
            env::set_var("AWS_ACCESS_KEY_ID", old);
        }
        if let Some(old) = old_secret {
            env::set_var("AWS_SECRET_ACCESS_KEY", old);
        }
        if let Some(old) = old_profile {
            env::set_var("AWS_PROFILE", old);
        }
        if let Some(old) = old_web_identity {
            env::set_var("AWS_WEB_IDENTITY_TOKEN_FILE", old);
        }
        if let Some(old) = old_role {
            env::set_var("AWS_ROLE_ARN", old);
        }
        if let Some(old) = old_session {
            env::set_var("AWS_SESSION_TOKEN", old);
        }
    }

    #[test]
    fn test_bedrock_provider_from_env() {
        let old_region = env::var("AWS_DEFAULT_REGION").ok();
        let old_model = env::var("BEDROCK_MODEL").ok();
        env::set_var("AWS_DEFAULT_REGION", "eu-west-1");
        env::set_var("BEDROCK_MODEL", "custom.model");

        let provider = BedrockProvider::from_env();
        assert!(provider.is_some());

        if let Some(p) = provider {
            assert_eq!(p.region, "eu-west-1");
        }

        if let Some(old) = old_region {
            env::set_var("AWS_DEFAULT_REGION", old);
        } else {
            env::remove_var("AWS_DEFAULT_REGION");
        }
        if let Some(old) = old_model {
            env::set_var("BEDROCK_MODEL", old);
        } else {
            env::remove_var("BEDROCK_MODEL");
        }
    }

    #[test]
    fn test_bedrock_provider_from_env_default_region() {
        let old_region = env::var("AWS_DEFAULT_REGION").ok();
        let old_aws_region = env::var("AWS_REGION").ok();
        let old_model = env::var("BEDROCK_MODEL").ok();
        env::remove_var("AWS_DEFAULT_REGION");
        env::remove_var("AWS_REGION");
        env::remove_var("BEDROCK_MODEL");

        let provider = BedrockProvider::from_env();
        assert!(provider.is_some());

        if let Some(p) = provider {
            assert_eq!(p.region, "us-east-1");
        }

        if let Some(old) = old_region {
            env::set_var("AWS_DEFAULT_REGION", old);
        }
        if let Some(old) = old_aws_region {
            env::set_var("AWS_REGION", old);
        }
        if let Some(old) = old_model {
            env::set_var("BEDROCK_MODEL", old);
        }
    }

    #[test]
    fn test_bedrock_credentials_debug() {
        let creds = BedrockCredentials::BearerToken("token".to_string());
        let debug_str = format!("{:?}", creds);
        assert!(debug_str.contains("BearerToken"));
    }

    #[test]
    fn test_bedrock_get_models_contains_expected() {
        let provider = BedrockProvider::new(ProviderConfig::default(), "us-east-1".to_string());
        let models = provider.get_models();
        let model_ids: Vec<&str> = models.iter().map(|m| m.id.as_str()).collect();
        assert!(model_ids.contains(&"anthropic.claude-3-5-sonnet-20241022-v2:0"));
        assert!(model_ids.contains(&"anthropic.claude-3-sonnet-20240229-v1:0"));
        assert!(model_ids.contains(&"meta.llama3-70b-instruct-v1:0"));
    }

    #[test]
    fn test_get_region_prefix_us() {
        assert_eq!(get_region_prefix("us.amazon.nova-pro"), Some("us"));
        assert_eq!(get_region_prefix("us.anthropic.claude-3-sonnet"), Some("us"));
        assert_eq!(get_region_prefix("us.foo.bar.baz"), Some("us"));
    }

    #[test]
    fn test_get_region_prefix_eu() {
        assert_eq!(get_region_prefix("eu.claude-3-sonnet"), Some("eu"));
        assert_eq!(get_region_prefix("eu.amazon.nova-micro"), Some("eu"));
        assert_eq!(get_region_prefix("eu.deepseek.v3"), Some("eu"));
    }

    #[test]
    fn test_get_region_prefix_apac() {
        assert_eq!(get_region_prefix("apac.amazon.nova-lite"), Some("apac"));
        assert_eq!(get_region_prefix("apac.anthropic.claude-3-haiku"), Some("apac"));
    }

    #[test]
    fn test_get_region_prefix_au() {
        assert_eq!(get_region_prefix("au.amazon.nova-pro"), Some("au"));
        assert_eq!(get_region_prefix("au.claude-3-sonnet"), Some("au"));
    }

    #[test]
    fn test_get_region_prefix_jp() {
        assert_eq!(get_region_prefix("jp.amazon.nova-pro"), Some("jp"));
        assert_eq!(get_region_prefix("jp.anthropic.claude-3-sonnet"), Some("jp"));
    }

    #[test]
    fn test_get_region_prefix_no_prefix() {
        assert_eq!(get_region_prefix("anthropic.claude-3-sonnet-20240229-v1:0"), None);
        assert_eq!(get_region_prefix("meta.llama3-70b-instruct-v1:0"), None);
        assert_eq!(get_region_prefix("amazon.titan-text-express-v1"), None);
        assert_eq!(get_region_prefix("mistral.mistral-large-2402-v1:0"), None);
    }

    #[test]
    fn test_get_bedrock_endpoint_us() {
        assert_eq!(get_bedrock_endpoint("us.amazon.nova-pro"), "https://bedrock.us-east-1.amazonaws.com");
        assert_eq!(get_bedrock_endpoint("us.anthropic.claude-3-sonnet"), "https://bedrock.us-east-1.amazonaws.com");
    }

    #[test]
    fn test_get_bedrock_endpoint_eu() {
        assert_eq!(get_bedrock_endpoint("eu.claude-3-sonnet"), "https://bedrock.eu-west-1.amazonaws.com");
        assert_eq!(get_bedrock_endpoint("eu.amazon.nova-micro"), "https://bedrock.eu-west-1.amazonaws.com");
    }

    #[test]
    fn test_get_bedrock_endpoint_apac() {
        assert_eq!(get_bedrock_endpoint("apac.amazon.nova-lite"), "https://bedrock.ap-northeast-1.amazonaws.com");
        assert_eq!(get_bedrock_endpoint("jp.amazon.nova-pro"), "https://bedrock.ap-northeast-1.amazonaws.com");
        assert_eq!(get_bedrock_endpoint("au.amazon.nova-pro"), "https://bedrock.ap-northeast-1.amazonaws.com");
    }

    #[test]
    fn test_get_bedrock_endpoint_default() {
        assert_eq!(get_bedrock_endpoint("anthropic.claude-3-sonnet-20240229-v1:0"), "https://bedrock.us-east-1.amazonaws.com");
        assert_eq!(get_bedrock_endpoint("meta.llama3-70b-instruct-v1:0"), "https://bedrock.us-east-1.amazonaws.com");
        assert_eq!(get_bedrock_endpoint("amazon.titan-text-express-v1"), "https://bedrock.us-east-1.amazonaws.com");
    }

    #[test]
    fn test_region_prefix_endpoint_consistency() {
        for prefix in ["us", "eu", "jp", "apac", "au"] {
            let model_id = format!("{}.provider.model", prefix);
            let endpoint = get_bedrock_endpoint(&model_id);
            assert!(endpoint.starts_with("https://bedrock."), "Endpoint should start with https://bedrock. for prefix {}", prefix);
        }
    }

    #[test]
    fn test_invalid_region_prefix_falls_back_to_default() {
        assert_eq!(get_bedrock_endpoint("invalid.amazon.model"), "https://bedrock.us-east-1.amazonaws.com");
        assert_eq!(get_bedrock_endpoint("us-west-2.amazon.model"), "https://bedrock.us-east-1.amazonaws.com");
        assert_eq!(get_bedrock_endpoint("foo.bar.baz"), "https://bedrock.us-east-1.amazonaws.com");
    }
}
