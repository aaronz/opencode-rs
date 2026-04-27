pub use super::types::{
    CentralAgentConfig, CentralConfig, CentralFormatterConfig, CentralProviderConfig,
    CentralToolConfig, EnterpriseConfig, EnterpriseError, SsoConfig, SsoProvider,
};

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