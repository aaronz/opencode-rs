use std::collections::HashMap;
use std::sync::Arc;

use crate::auth_layered::layer1_credential_source::{
    CompositeCredentialResolver, ResolvedCredential,
};
use crate::auth_layered::layer3_provider_transport::AwsSigV4Transport;
use crate::auth_layered::layer3_provider_transport::OpenAICompatibleTransport;
use crate::auth_layered::{
    AccessControlResult, AuthMechanism, CredentialResolver, CredentialSource, RuntimeAccessControl,
    TransportLayer,
};
use opencode_core::OpenCodeError;

#[derive(Clone)]
pub struct ProviderAuthConfig {
    pub credential_sources: Vec<CredentialSource>,
    pub auth_mechanism: AuthMechanism,
    pub transport: Arc<TransportLayer>,
    pub access_control: Arc<RuntimeAccessControl>,
    pub base_url: Option<String>,
}

pub struct ResolvedAuth {
    pub credential: String,
    pub mechanism: AuthMechanism,
    pub transport: Arc<TransportLayer>,
}

pub struct ProviderRegistry {
    credential_resolver: CompositeCredentialResolver,
    provider_configs: HashMap<String, ProviderAuthConfig>,
    access_control: RuntimeAccessControl,
}

impl ProviderRegistry {
    pub fn new() -> Self {
        Self {
            credential_resolver: CompositeCredentialResolver::new(),
            provider_configs: HashMap::new(),
            access_control: RuntimeAccessControl::new(),
        }
    }

    pub fn with_credential_resolver(mut self, resolver: CompositeCredentialResolver) -> Self {
        self.credential_resolver = resolver;
        self
    }

    pub fn with_access_control(mut self, acl: RuntimeAccessControl) -> Self {
        self.access_control = acl;
        self
    }

    pub fn register_provider(&mut self, provider_id: &str, config: ProviderAuthConfig) {
        self.provider_configs
            .insert(provider_id.to_string(), config);
    }

    pub fn resolve_auth(&self, provider_id: &str) -> Result<ResolvedAuth, OpenCodeError> {
        match self.access_control.check_provider_access(provider_id) {
            AccessControlResult::Denied(reason) => {
                return Err(OpenCodeError::Provider(reason));
            }
            AccessControlResult::ProviderNotFound(reason) => {
                return Err(OpenCodeError::Provider(reason));
            }
            AccessControlResult::Allowed => {}
        }

        let provider_config = self.provider_configs.get(provider_id);

        let (credential, mechanism) = if let Some(config) = provider_config {
            let cred: Option<ResolvedCredential> = self
                .credential_resolver
                .resolve_with_fallback(provider_id, &config.credential_sources);
            let mech = if cred.is_some() {
                config.auth_mechanism.clone()
            } else {
                AuthMechanism::ApiKey
            };
            (cred, mech)
        } else {
            let cred: Option<ResolvedCredential> = self.credential_resolver.resolve_with_fallback(
                provider_id,
                &[CredentialSource::EnvVar, CredentialSource::ConfigInline],
            );
            (cred, AuthMechanism::ApiKey)
        };

        let credential = credential.map(|c| c.value).ok_or_else(|| {
            OpenCodeError::Provider(format!(
                "No credentials found for provider: {}",
                provider_id
            ))
        })?;

        let transport = if let Some(config) = provider_config {
            config.transport.clone()
        } else {
            Arc::new(TransportLayer::new(
                Box::new(OpenAICompatibleTransport),
                "https://api.openai.com".to_string(),
            ))
        };

        Ok(ResolvedAuth {
            credential,
            mechanism,
            transport,
        })
    }

    pub fn check_access(&self, provider_id: &str) -> AccessControlResult {
        self.access_control.check_provider_access(provider_id)
    }
}

impl Default for ProviderRegistry {
    fn default() -> Self {
        Self::new()
    }
}

pub mod examples {
    use super::*;

    pub fn openai_config() -> ProviderAuthConfig {
        ProviderAuthConfig {
            credential_sources: vec![
                CredentialSource::OAuthStore,
                CredentialSource::EnvVar,
                CredentialSource::ConfigInline,
            ],
            auth_mechanism: AuthMechanism::BearerToken,
            transport: Arc::new(TransportLayer::new(
                Box::new(OpenAICompatibleTransport),
                "https://api.openai.com".to_string(),
            )),
            access_control: Arc::new(RuntimeAccessControl::new()),
            base_url: None,
        }
    }

    pub fn aws_bedrock_config(region: &str) -> ProviderAuthConfig {
        ProviderAuthConfig {
            credential_sources: vec![CredentialSource::EnvVar, CredentialSource::SystemKeychain],
            auth_mechanism: AuthMechanism::AwsCredentialChain,
            transport: Arc::new(TransportLayer::new(
                Box::new(AwsSigV4Transport::new(
                    region.to_string(),
                    "bedrock".to_string(),
                )),
                format!("https://bedrock-runtime.{}", region),
            )),
            access_control: Arc::new(RuntimeAccessControl::new()),
            base_url: None,
        }
    }

    pub fn copilot_config() -> ProviderAuthConfig {
        ProviderAuthConfig {
            credential_sources: vec![CredentialSource::OAuthStore, CredentialSource::EnvVar],
            auth_mechanism: AuthMechanism::OAuthBrowser,
            transport: Arc::new(TransportLayer::new(
                Box::new(OpenAICompatibleTransport),
                "https://api.github.com".to_string(),
            )),
            access_control: Arc::new(RuntimeAccessControl::new()),
            base_url: None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashSet;

    #[test]
    fn test_provider_registry_resolve_auth() {
        let mut creds = HashMap::new();
        creds.insert("openai".to_string(), "sk-test-key".to_string());

        let resolver = CompositeCredentialResolver::new().with_inline(creds);
        let mut registry = ProviderRegistry::new().with_credential_resolver(resolver);
        registry.register_provider("openai", examples::openai_config());

        let auth = registry.resolve_auth("openai").unwrap();
        assert_eq!(auth.credential, "sk-test-key");
        assert!(matches!(auth.mechanism, AuthMechanism::BearerToken));
    }

    #[test]
    fn test_provider_registry_access_control_denylist() {
        let mut denylist = HashSet::new();
        denylist.insert("disabled-provider".to_string());

        let acl = RuntimeAccessControl::new().with_denylist(denylist);
        let registry = ProviderRegistry::new().with_access_control(acl);

        let result = registry.resolve_auth("disabled-provider");
        assert!(result.is_err());
    }

    #[test]
    fn test_provider_registry_missing_credentials() {
        let registry = ProviderRegistry::new();

        let result = registry.resolve_auth("nonexistent");
        assert!(result.is_err());
    }

    #[test]
    fn test_example_configs() {
        let openai = examples::openai_config();
        assert!(!openai.credential_sources.is_empty());

        let aws = examples::aws_bedrock_config("us-east-1");
        assert!(matches!(
            aws.auth_mechanism,
            AuthMechanism::AwsCredentialChain
        ));

        let copilot = examples::copilot_config();
        assert!(matches!(
            copilot.auth_mechanism,
            AuthMechanism::OAuthBrowser
        ));
    }
}
