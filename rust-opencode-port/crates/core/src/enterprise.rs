use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[derive(Default)]
pub struct EnterpriseConfig {
    pub enabled: bool,
    pub central_config_url: Option<String>,
    pub sso_enabled: bool,
    pub sso_provider: Option<SsoProvider>,
    pub forced_gateway_only: bool,
    pub disabled_external_providers: Vec<String>,
    pub api_key: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SsoProvider {
    Okta,
    AzureAD,
    Auth0,
    GenericOIDC,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SsoConfig {
    pub provider: SsoProvider,
    pub client_id: String,
    pub client_secret: Option<String>,
    pub issuer_url: String,
    pub scopes: Vec<String>,
}


impl EnterpriseConfig {
    pub fn is_provider_allowed(&self, provider: &str) -> bool {
        if self.forced_gateway_only {
            return false;
        }
        !self
            .disabled_external_providers
            .contains(&provider.to_string())
    }

    pub fn fetch_central_config(&self) -> Result<CentralConfig, EnterpriseError> {
        let url = self
            .central_config_url
            .as_ref()
            .ok_or_else(|| EnterpriseError::Config("central_config_url not set".to_string()))?;

        let client = reqwest::blocking::Client::new();
        let mut request = client.get(url);

        if let Some(ref key) = self.api_key {
            request = request.header("Authorization", format!("Bearer {}", key));
        }

        let response = request
            .send()
            .map_err(|e| EnterpriseError::Network(e.to_string()))?;

        if !response.status().is_success() {
            return Err(EnterpriseError::Network(format!(
                "central config fetch failed: {}",
                response.status()
            )));
        }

        let config: CentralConfig = response
            .json()
            .map_err(|e| EnterpriseError::Parse(e.to_string()))?;

        Ok(config)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct CentralConfig {
    pub providers: Option<Vec<CentralProviderConfig>>,
    pub agents: Option<Vec<CentralAgentConfig>>,
    pub tools: Option<Vec<CentralToolConfig>>,
    pub formatters: Option<Vec<CentralFormatterConfig>>,
    pub instructions: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CentralProviderConfig {
    pub name: String,
    pub api_base: Option<String>,
    pub api_key: Option<String>,
    pub model: Option<String>,
    pub disabled: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CentralAgentConfig {
    pub name: String,
    pub model: Option<String>,
    pub temperature: Option<f32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CentralToolConfig {
    pub name: String,
    pub enabled: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CentralFormatterConfig {
    pub extensions: Vec<String>,
    pub command: String,
}

#[derive(Debug, thiserror::Error)]
pub enum EnterpriseError {
    #[error("enterprise config error: {0}")]
    Config(String),
    #[error("network error: {0}")]
    Network(String),
    #[error("parse error: {0}")]
    Parse(String),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_enterprise_config_default_disabled() {
        let config = EnterpriseConfig::default();
        assert!(!config.enabled);
        assert!(!config.sso_enabled);
        assert!(!config.forced_gateway_only);
    }

    #[test]
    fn test_provider_blocked_in_gateway_mode() {
        let config = EnterpriseConfig {
            forced_gateway_only: true,
            ..Default::default()
        };

        assert!(!config.is_provider_allowed("openai"));
        assert!(!config.is_provider_allowed("anthropic"));
    }

    #[test]
    fn test_disabled_external_providers() {
        let config = EnterpriseConfig {
            disabled_external_providers: vec!["openai".to_string()],
            forced_gateway_only: false,
            ..Default::default()
        };

        assert!(!config.is_provider_allowed("openai"));
        assert!(config.is_provider_allowed("anthropic"));
    }
}
