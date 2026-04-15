use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProviderCatalog {
    pub providers: BTreeMap<String, ProviderDescriptor>,
    pub fetched_at: chrono::DateTime<chrono::Utc>,
    pub source: CatalogSource,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum CatalogSource {
    ModelsDev,
    Config,
    Local,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProviderDescriptor {
    pub id: String,
    pub display_name: String,
    pub api_base_url: Option<String>,
    pub docs_url: Option<String>,
    pub env_vars: Vec<String>,
    pub npm_package: Option<String>,
    pub models: BTreeMap<String, ModelDescriptor>,
    pub source: CatalogSource,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelDescriptor {
    pub id: String,
    pub display_name: String,
    pub family: Option<String>,
    pub provider_id: String,
    pub capabilities: ModelCapabilities,
    pub cost: CostInfo,
    pub limits: LimitInfo,
    pub status: ModelStatus,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ModelCapabilities {
    pub attachment: bool,
    pub reasoning: bool,
    pub tool_call: bool,
    pub temperature: bool,
    pub structured_output: bool,
    pub interleaved: bool,
    pub open_weights: bool,
    pub input_modalities: Vec<String>,
    pub output_modalities: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct CostInfo {
    pub input: f64,
    pub output: f64,
    pub cache_read: f64,
    pub cache_write: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct LimitInfo {
    pub context: u32,
    pub input: Option<u32>,
    pub output: u32,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum ModelStatus {
    Active,
    Beta,
    Alpha,
    Deprecated,
}
