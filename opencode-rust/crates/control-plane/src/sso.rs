use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SsoConfig {
    pub id: String,
    pub provider: SsoProvider,
    pub entity_id: String,
    pub sso_url: String,
    pub certificate: Option<String>,
    pub client_id: Option<String>,
    pub client_secret: Option<String>,
    pub redirect_uri: Option<String>,
    pub enabled: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SsoProvider {
    Saml,
    Oidc,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OidcState {
    pub state: String,
    pub nonce: String,
    pub code_verifier: Option<String>,
    pub expires_at: chrono::DateTime<chrono::Utc>,
}

pub struct SsoManager {
    config: Option<SsoConfig>,
}

impl SsoManager {
    pub fn new() -> Self {
        Self { config: None }
    }

    pub fn set_config(&mut self, config: SsoConfig) {
        self.config = Some(config);
    }

    pub fn get_config(&self) -> Option<&SsoConfig> {
        self.config.as_ref()
    }

    pub fn is_enabled(&self) -> bool {
        self.config.as_ref().map(|c| c.enabled).unwrap_or(false)
    }
}

impl Default for SsoManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sso_config_creation() {
        let config = SsoConfig {
            id: "sso-1".to_string(),
            provider: SsoProvider::Oidc,
            entity_id: "opencode".to_string(),
            sso_url: "https://sso.example.com".to_string(),
            certificate: None,
            client_id: Some("client-id".to_string()),
            client_secret: Some("secret".to_string()),
            redirect_uri: Some("http://localhost:8080/callback".to_string()),
            enabled: true,
        };
        assert!(config.enabled);
    }

    #[test]
    fn test_sso_manager() {
        let mut manager = SsoManager::new();
        assert!(!manager.is_enabled());

        let config = SsoConfig {
            id: "sso-1".to_string(),
            provider: SsoProvider::Oidc,
            entity_id: "opencode".to_string(),
            sso_url: "https://sso.example.com".to_string(),
            certificate: None,
            client_id: None,
            client_secret: None,
            redirect_uri: None,
            enabled: true,
        };
        manager.set_config(config);
        assert!(manager.is_enabled());
    }
}
