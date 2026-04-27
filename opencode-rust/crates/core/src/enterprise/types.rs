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