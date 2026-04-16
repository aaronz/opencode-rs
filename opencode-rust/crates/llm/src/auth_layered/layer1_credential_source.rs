use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum CredentialSource {
    AuthFile,
    EnvVar,
    DotEnv,
    ConfigInline,
    FileRef,
    OAuthStore,
    SystemKeychain,
    AwsCredentialChain,
    GcpServiceAccount,
    AzureIdentity,
}

impl CredentialSource {
    pub fn is_cloud_native(&self) -> bool {
        matches!(
            self,
            Self::AwsCredentialChain | Self::GcpServiceAccount | Self::AzureIdentity
        )
    }

    pub fn priority(&self) -> u8 {
        match self {
            Self::OAuthStore => 5,
            Self::AwsCredentialChain => 4,
            Self::GcpServiceAccount => 4,
            Self::AzureIdentity => 4,
            Self::SystemKeychain => 3,
            Self::AuthFile => 2,
            Self::DotEnv => 2,
            Self::EnvVar => 1,
            Self::ConfigInline => 0,
            Self::FileRef => 0,
        }
    }
}

#[derive(Debug, Clone)]
pub struct ResolvedCredential {
    pub provider: String,
    pub value: String,
    pub source: CredentialSource,
    pub metadata: HashMap<String, String>,
}

pub trait CredentialResolver: Send + Sync {
    fn resolve(&self, provider: &str, source: &CredentialSource) -> Option<ResolvedCredential>;

    fn resolve_with_fallback(
        &self,
        provider: &str,
        sources: &[CredentialSource],
    ) -> Option<ResolvedCredential> {
        for source in sources {
            if let Some(cred) = self.resolve(provider, source) {
                return Some(cred);
            }
        }
        None
    }
}

pub struct CompositeCredentialResolver {
    auth_file_path: Option<PathBuf>,
    dotenv_paths: Vec<PathBuf>,
    inline_credentials: HashMap<String, String>,
    oauth_store: Option<HashMap<String, String>>,
}

impl Default for CompositeCredentialResolver {
    fn default() -> Self {
        Self::new()
    }
}

impl CompositeCredentialResolver {
    pub fn new() -> Self {
        Self {
            auth_file_path: None,
            dotenv_paths: Vec::new(),
            inline_credentials: HashMap::new(),
            oauth_store: None,
        }
    }

    pub fn with_auth_file(mut self, path: PathBuf) -> Self {
        self.auth_file_path = Some(path);
        self
    }

    pub fn with_dotenv(mut self, paths: Vec<PathBuf>) -> Self {
        self.dotenv_paths = paths;
        self
    }

    pub fn with_inline(mut self, creds: HashMap<String, String>) -> Self {
        self.inline_credentials = creds;
        self
    }

    pub fn with_oauth_store(mut self, store: HashMap<String, String>) -> Self {
        self.oauth_store = Some(store);
        self
    }

    fn resolve_aws_credentials(&self, provider: &str) -> Option<ResolvedCredential> {
        if provider != "bedrock" && provider != "aws" {
            return None;
        }

        let mut metadata = HashMap::new();

        if let Ok(region) =
            std::env::var("AWS_DEFAULT_REGION").or_else(|_| std::env::var("AWS_REGION"))
        {
            metadata.insert("region".to_string(), region);
        }

        if std::env::var("AWS_ACCESS_KEY_ID").is_ok() {
            metadata.insert("type".to_string(), "access_key".to_string());
        } else if std::env::var("AWS_WEB_IDENTITY_TOKEN_FILE").is_ok() {
            metadata.insert("type".to_string(), "oidc".to_string());
        } else if std::env::var("AWS_PROFILE").is_ok() {
            metadata.insert("type".to_string(), "profile".to_string());
        } else {
            return None;
        }

        Some(ResolvedCredential {
            provider: provider.to_string(),
            value: "aws_credential_chain".to_string(),
            source: CredentialSource::AwsCredentialChain,
            metadata,
        })
    }

    fn resolve_gcp_credentials(&self, provider: &str) -> Option<ResolvedCredential> {
        if provider != "vertex" && provider != "gcp" {
            return None;
        }

        let mut metadata = HashMap::new();

        if let Ok(project) = std::env::var("GCP_PROJECT") {
            metadata.insert("project".to_string(), project);
        }

        if let Ok(location) = std::env::var("GCP_LOCATION") {
            metadata.insert("location".to_string(), location);
        }

        let has_creds = std::env::var("GOOGLE_APPLICATION_CREDENTIALS").is_ok()
            || std::env::var("GCP_SERVICE_ACCOUNT_JSON").is_ok();

        if has_creds {
            metadata.insert("type".to_string(), "service_account".to_string());
            Some(ResolvedCredential {
                provider: provider.to_string(),
                value: "gcp_service_account".to_string(),
                source: CredentialSource::GcpServiceAccount,
                metadata,
            })
        } else {
            None
        }
    }

    fn resolve_azure_credentials(&self, provider: &str) -> Option<ResolvedCredential> {
        if provider != "azure" && provider != "azure_openai" {
            return None;
        }

        let mut metadata = HashMap::new();

        if let Ok(tenant) = std::env::var("AZURE_TENANT_ID") {
            metadata.insert("tenant".to_string(), tenant);
        }

        let has_creds = std::env::var("AZURE_CLIENT_ID").is_ok()
            && std::env::var("AZURE_CLIENT_SECRET").is_ok()
            || std::env::var("AZURE_USE_MANAGED_IDENTITY").is_ok();

        if has_creds {
            metadata.insert("type".to_string(), "managed_identity".to_string());
            Some(ResolvedCredential {
                provider: provider.to_string(),
                value: "azure_identity".to_string(),
                source: CredentialSource::AzureIdentity,
                metadata,
            })
        } else {
            None
        }
    }

    fn load_from_auth_file(&self, provider: &str) -> Option<String> {
        let path = self.auth_file_path.as_ref()?;
        let content = std::fs::read_to_string(path).ok()?;
        let creds: HashMap<String, serde_json::Value> = serde_json::from_str(&content).ok()?;
        let value = creds.get(provider)?;
        value
            .get("key")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string())
            .or_else(|| value.as_str().map(|s| s.to_string()))
    }

    fn load_from_dotenv(&self, provider: &str) -> Option<String> {
        let env_var = Self::provider_to_env_var(provider);
        for path in &self.dotenv_paths {
            if let Ok(content) = std::fs::read_to_string(path) {
                for line in content.lines() {
                    let line = line.trim();
                    if line.starts_with('#') || line.is_empty() {
                        continue;
                    }
                    if let Some((key, value)) = line.split_once('=') {
                        if key.trim() == env_var {
                            return Some(
                                value
                                    .trim()
                                    .trim_matches('"')
                                    .trim_matches('\'')
                                    .to_string(),
                            );
                        }
                    }
                }
            }
        }
        std::env::var(&env_var).ok()
    }

    fn provider_to_env_var(provider: &str) -> String {
        let known = [
            ("openai", "OPENAI_API_KEY"),
            ("anthropic", "ANTHROPIC_API_KEY"),
            ("google", "GOOGLE_API_KEY"),
            ("azure", "AZURE_OPENAI_API_KEY"),
            ("ollama", "OLLAMA_HOST"),
            ("aws", "AWS_ACCESS_KEY_ID"),
            ("cohere", "COHERE_API_KEY"),
            ("mistral", "MISTRAL_API_KEY"),
            ("perplexity", "PERPLEXITY_API_KEY"),
            ("groq", "GROQ_API_KEY"),
            ("openrouter", "OPENROUTER_API_KEY"),
            ("huggingface", "HF_TOKEN"),
            ("ai21", "AI21_API_KEY"),
        ];
        known
            .iter()
            .find(|(p, _)| *p == provider)
            .map(|(_, v)| v.to_string())
            .unwrap_or_else(|| format!("OPENCODE_{}_API_KEY", provider.to_uppercase()))
    }
}

impl CredentialResolver for CompositeCredentialResolver {
    fn resolve(&self, provider: &str, source: &CredentialSource) -> Option<ResolvedCredential> {
        match source {
            CredentialSource::AuthFile => {
                self.load_from_auth_file(provider)
                    .map(|value| ResolvedCredential {
                        provider: provider.to_string(),
                        value,
                        source: CredentialSource::AuthFile,
                        metadata: HashMap::new(),
                    })
            }
            CredentialSource::EnvVar => {
                self.load_from_dotenv(provider)
                    .map(|value| ResolvedCredential {
                        provider: provider.to_string(),
                        value,
                        source: CredentialSource::EnvVar,
                        metadata: HashMap::new(),
                    })
            }
            CredentialSource::DotEnv => {
                self.load_from_dotenv(provider)
                    .map(|value| ResolvedCredential {
                        provider: provider.to_string(),
                        value,
                        source: CredentialSource::DotEnv,
                        metadata: HashMap::new(),
                    })
            }
            CredentialSource::ConfigInline => {
                self.inline_credentials
                    .get(provider)
                    .map(|value| ResolvedCredential {
                        provider: provider.to_string(),
                        value: value.clone(),
                        source: CredentialSource::ConfigInline,
                        metadata: HashMap::new(),
                    })
            }
            CredentialSource::FileRef => {
                self.load_from_auth_file(provider)
                    .map(|value| ResolvedCredential {
                        provider: provider.to_string(),
                        value,
                        source: CredentialSource::FileRef,
                        metadata: HashMap::new(),
                    })
            }
            CredentialSource::OAuthStore => self.oauth_store.as_ref().and_then(|store| {
                store.get(provider).map(|value| ResolvedCredential {
                    provider: provider.to_string(),
                    value: value.clone(),
                    source: CredentialSource::OAuthStore,
                    metadata: HashMap::new(),
                })
            }),
            CredentialSource::SystemKeychain => None,
            CredentialSource::AwsCredentialChain => self.resolve_aws_credentials(provider),
            CredentialSource::GcpServiceAccount => self.resolve_gcp_credentials(provider),
            CredentialSource::AzureIdentity => self.resolve_azure_credentials(provider),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn test_composite_resolver_inline_credential() {
        let mut creds = HashMap::new();
        creds.insert("openai".to_string(), "sk-test-inline".to_string());
        let resolver = CompositeCredentialResolver::new().with_inline(creds);

        let cred = resolver.resolve("openai", &CredentialSource::ConfigInline);
        assert!(cred.is_some());
        assert_eq!(cred.unwrap().value, "sk-test-inline");
    }

    #[test]
    fn test_composite_resolver_auth_file() {
        let temp_dir = std::env::temp_dir().join("opencode_auth_test");
        fs::create_dir_all(&temp_dir).unwrap();
        let auth_file = temp_dir.join("auth.json");
        fs::write(&auth_file, r#"{"openai": {"key": "sk-from-file"}}"#).unwrap();

        let resolver = CompositeCredentialResolver::new().with_auth_file(auth_file);

        let cred = resolver.resolve("openai", &CredentialSource::AuthFile);
        assert!(cred.is_some());
        assert_eq!(cred.unwrap().value, "sk-from-file");

        let _ = fs::remove_dir_all(temp_dir);
    }

    #[test]
    fn test_composite_resolver_fallback() {
        let mut inline = HashMap::new();
        inline.insert("openai".to_string(), "inline-value".to_string());
        let resolver = CompositeCredentialResolver::new().with_inline(inline);

        let cred = resolver.resolve_with_fallback(
            "openai",
            &[CredentialSource::OAuthStore, CredentialSource::ConfigInline],
        );
        assert!(cred.is_some());
        assert_eq!(cred.unwrap().source, CredentialSource::ConfigInline);
    }

    #[test]
    fn test_provider_to_env_var_known_mappings() {
        assert_eq!(
            CompositeCredentialResolver::provider_to_env_var("openai"),
            "OPENAI_API_KEY"
        );
        assert_eq!(
            CompositeCredentialResolver::provider_to_env_var("anthropic"),
            "ANTHROPIC_API_KEY"
        );
        assert_eq!(
            CompositeCredentialResolver::provider_to_env_var("huggingface"),
            "HF_TOKEN"
        );
    }

    #[test]
    fn test_provider_to_env_var_unknown_uses_default_pattern() {
        assert_eq!(
            CompositeCredentialResolver::provider_to_env_var("custom"),
            "OPENCODE_CUSTOM_API_KEY"
        );
    }

    #[test]
    fn test_resolve_returns_none_for_missing_provider() {
        let resolver = CompositeCredentialResolver::new();
        assert!(resolver
            .resolve("nonexistent", &CredentialSource::ConfigInline)
            .is_none());
    }

    #[test]
    fn test_credential_source_is_cloud_native() {
        assert!(CredentialSource::AwsCredentialChain.is_cloud_native());
        assert!(CredentialSource::GcpServiceAccount.is_cloud_native());
        assert!(CredentialSource::AzureIdentity.is_cloud_native());
        assert!(!CredentialSource::EnvVar.is_cloud_native());
        assert!(!CredentialSource::ConfigInline.is_cloud_native());
    }

    #[test]
    fn test_credential_source_priority_ordering() {
        assert!(CredentialSource::OAuthStore.priority() > CredentialSource::EnvVar.priority());
        assert!(
            CredentialSource::AwsCredentialChain.priority()
                > CredentialSource::SystemKeychain.priority()
        );
        assert!(
            CredentialSource::SystemKeychain.priority() > CredentialSource::AuthFile.priority()
        );
    }

    #[test]
    fn test_aws_credential_chain_resolves_for_bedrock() {
        std::env::remove_var("AWS_PROFILE");
        std::env::set_var("AWS_ACCESS_KEY_ID", "test-key");
        std::env::set_var("AWS_SECRET_ACCESS_KEY", "test-secret");

        let resolver = CompositeCredentialResolver::new();
        let cred = resolver.resolve("bedrock", &CredentialSource::AwsCredentialChain);

        assert!(cred.is_some());
        let c = cred.unwrap();
        assert_eq!(c.source, CredentialSource::AwsCredentialChain);
        assert_eq!(c.metadata.get("type"), Some(&"access_key".to_string()));

        std::env::remove_var("AWS_ACCESS_KEY_ID");
        std::env::remove_var("AWS_SECRET_ACCESS_KEY");
    }

    #[test]
    fn test_aws_credential_chain_skips_non_aws_providers() {
        let resolver = CompositeCredentialResolver::new();
        let cred = resolver.resolve("openai", &CredentialSource::AwsCredentialChain);
        assert!(cred.is_none());
    }

    #[test]
    fn test_oauth_store_resolves_credentials() {
        let mut store = HashMap::new();
        store.insert("openai".to_string(), "oauth-token-123".to_string());
        let resolver = CompositeCredentialResolver::new().with_oauth_store(store);

        let cred = resolver.resolve("openai", &CredentialSource::OAuthStore);
        assert!(cred.is_some());
        assert_eq!(cred.unwrap().value, "oauth-token-123");
    }

    #[test]
    fn test_oauth_store_returns_none_when_empty() {
        let resolver = CompositeCredentialResolver::new();
        let cred = resolver.resolve("openai", &CredentialSource::OAuthStore);
        assert!(cred.is_none());
    }

    #[test]
    fn test_dotenv_resolves_from_env_var() {
        std::env::set_var("OPENAI_API_KEY", "env-api-key-123");

        let resolver = CompositeCredentialResolver::new();
        let cred = resolver.resolve("openai", &CredentialSource::EnvVar);

        assert!(cred.is_some());
        assert_eq!(cred.unwrap().value, "env-api-key-123");

        std::env::remove_var("OPENAI_API_KEY");
    }

    #[test]
    fn test_system_keychain_returns_none() {
        let resolver = CompositeCredentialResolver::new();
        let cred = resolver.resolve("openai", &CredentialSource::SystemKeychain);
        assert!(cred.is_none());
    }

    #[test]
    fn test_gcp_credential_chain_resolves_for_vertex() {
        std::env::set_var("GCP_PROJECT", "test-project");
        std::env::set_var("GOOGLE_APPLICATION_CREDENTIALS", "/path/to/creds.json");

        let resolver = CompositeCredentialResolver::new();
        let cred = resolver.resolve("vertex", &CredentialSource::GcpServiceAccount);

        assert!(cred.is_some());
        let c = cred.unwrap();
        assert_eq!(c.source, CredentialSource::GcpServiceAccount);
        assert_eq!(c.metadata.get("type"), Some(&"service_account".to_string()));

        std::env::remove_var("GCP_PROJECT");
        std::env::remove_var("GOOGLE_APPLICATION_CREDENTIALS");
    }

    #[test]
    fn test_gcp_credential_chain_skips_non_gcp_providers() {
        let resolver = CompositeCredentialResolver::new();
        let cred = resolver.resolve("openai", &CredentialSource::GcpServiceAccount);
        assert!(cred.is_none());
    }

    #[test]
    fn test_azure_credential_chain_resolves_for_azure() {
        std::env::set_var("AZURE_CLIENT_ID", "client-123");
        std::env::set_var("AZURE_CLIENT_SECRET", "secret-456");

        let resolver = CompositeCredentialResolver::new();
        let cred = resolver.resolve("azure", &CredentialSource::AzureIdentity);

        assert!(cred.is_some());
        let c = cred.unwrap();
        assert_eq!(c.source, CredentialSource::AzureIdentity);
        assert_eq!(
            c.metadata.get("type"),
            Some(&"managed_identity".to_string())
        );

        std::env::remove_var("AZURE_CLIENT_ID");
        std::env::remove_var("AZURE_CLIENT_SECRET");
    }

    #[test]
    fn test_azure_credential_chain_skips_non_azure_providers() {
        let resolver = CompositeCredentialResolver::new();
        let cred = resolver.resolve("openai", &CredentialSource::AzureIdentity);
        assert!(cred.is_none());
    }

    #[test]
    fn test_azure_credential_chain_with_managed_identity() {
        std::env::set_var("AZURE_USE_MANAGED_IDENTITY", "true");

        let resolver = CompositeCredentialResolver::new();
        let cred = resolver.resolve("azure", &CredentialSource::AzureIdentity);

        assert!(cred.is_some());
        let c = cred.unwrap();
        assert_eq!(
            c.metadata.get("type"),
            Some(&"managed_identity".to_string())
        );

        std::env::remove_var("AZURE_USE_MANAGED_IDENTITY");
    }

    #[test]
    fn test_file_ref_loads_from_auth_file() {
        let temp_dir = std::env::temp_dir().join("opencode_auth_test2");
        fs::create_dir_all(&temp_dir).unwrap();
        let auth_file = temp_dir.join("auth.json");
        fs::write(&auth_file, r#"{"openai": {"key": "file-ref-key"}}"#).unwrap();

        let resolver = CompositeCredentialResolver::new().with_auth_file(auth_file);

        let cred = resolver.resolve("openai", &CredentialSource::FileRef);
        assert!(cred.is_some());
        assert_eq!(cred.unwrap().value, "file-ref-key");

        let _ = fs::remove_dir_all(temp_dir);
    }

    #[test]
    fn test_auth_file_returns_none_when_no_file() {
        let resolver = CompositeCredentialResolver::new();
        let cred = resolver.resolve("openai", &CredentialSource::AuthFile);
        assert!(cred.is_none());
    }

    #[test]
    fn test_dotenv_returns_none_when_not_found() {
        std::env::remove_var("NONEXISTENT_API_KEY");

        let resolver = CompositeCredentialResolver::new();
        let cred = resolver.resolve("nonexistent", &CredentialSource::EnvVar);
        assert!(cred.is_none());
    }

    #[test]
    fn test_resolved_credential_metadata() {
        std::env::set_var("AWS_ACCESS_KEY_ID", "test-key");
        std::env::set_var("AWS_SECRET_ACCESS_KEY", "test-secret");
        std::env::set_var("AWS_DEFAULT_REGION", "us-west-2");

        let resolver = CompositeCredentialResolver::new();
        let cred = resolver.resolve("bedrock", &CredentialSource::AwsCredentialChain);

        assert!(cred.is_some());
        let c = cred.unwrap();
        assert_eq!(c.provider, "bedrock");
        assert_eq!(c.metadata.get("region"), Some(&"us-west-2".to_string()));

        std::env::remove_var("AWS_ACCESS_KEY_ID");
        std::env::remove_var("AWS_SECRET_ACCESS_KEY");
        std::env::remove_var("AWS_DEFAULT_REGION");
    }

    #[test]
    fn test_credential_source_priority_all_variants() {
        let priorities: Vec<u8> = vec![
            CredentialSource::OAuthStore.priority(),
            CredentialSource::AwsCredentialChain.priority(),
            CredentialSource::GcpServiceAccount.priority(),
            CredentialSource::AzureIdentity.priority(),
            CredentialSource::SystemKeychain.priority(),
            CredentialSource::AuthFile.priority(),
            CredentialSource::DotEnv.priority(),
            CredentialSource::EnvVar.priority(),
            CredentialSource::ConfigInline.priority(),
            CredentialSource::FileRef.priority(),
        ];

        for i in 0..priorities.len() - 1 {
            assert!(
                priorities[i] >= priorities[i + 1],
                "Priorities should be in non-increasing order"
            );
        }
    }

    #[test]
    fn test_composite_resolver_with_multiple_sources() {
        let mut inline = HashMap::new();
        inline.insert("test".to_string(), "inline".to_string());
        let oauth_store: HashMap<String, String> = HashMap::new();

        let resolver = CompositeCredentialResolver::new()
            .with_inline(inline)
            .with_oauth_store(oauth_store);

        let cred = resolver.resolve("test", &CredentialSource::ConfigInline);
        assert!(cred.is_some());
        assert_eq!(cred.unwrap().value, "inline");
    }

    #[test]
    fn test_aws_web_identity_token_resolves() {
        std::env::remove_var("AWS_ACCESS_KEY_ID");
        std::env::remove_var("AWS_SECRET_ACCESS_KEY");
        std::env::remove_var("AWS_PROFILE");
        std::env::set_var("AWS_WEB_IDENTITY_TOKEN_FILE", "/path/to/token");
        std::env::set_var("AWS_ROLE_ARN", "arn:aws:iam::123:role/MyRole");

        let resolver = CompositeCredentialResolver::new();
        let cred = resolver.resolve("bedrock", &CredentialSource::AwsCredentialChain);

        assert!(cred.is_some());
        let c = cred.unwrap();
        assert_eq!(c.metadata.get("type"), Some(&"oidc".to_string()));

        std::env::remove_var("AWS_WEB_IDENTITY_TOKEN_FILE");
        std::env::remove_var("AWS_ROLE_ARN");
    }

    #[test]
    fn test_aws_profile_resolves() {
        std::env::remove_var("AWS_ACCESS_KEY_ID");
        std::env::remove_var("AWS_SECRET_ACCESS_KEY");
        std::env::remove_var("AWS_WEB_IDENTITY_TOKEN_FILE");
        std::env::remove_var("AWS_ROLE_ARN");
        std::env::remove_var("AWS_PROFILE");
        std::env::set_var("AWS_PROFILE", "my-profile");

        let resolver = CompositeCredentialResolver::new();
        let cred = resolver.resolve("bedrock", &CredentialSource::AwsCredentialChain);

        assert!(cred.is_some());
        let c = cred.unwrap();
        assert_eq!(c.metadata.get("type"), Some(&"profile".to_string()));

        std::env::remove_var("AWS_PROFILE");
    }

    #[test]
    fn test_gcp_service_account_json_env() {
        std::env::set_var("GCP_SERVICE_ACCOUNT_JSON", "{}");

        let resolver = CompositeCredentialResolver::new();
        let cred = resolver.resolve("gcp", &CredentialSource::GcpServiceAccount);

        assert!(cred.is_some());

        std::env::remove_var("GCP_SERVICE_ACCOUNT_JSON");
    }
}
