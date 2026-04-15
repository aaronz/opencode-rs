use serde::Deserialize;
use std::collections::BTreeMap;

pub type ModelsDevApiResponse = BTreeMap<String, ModelsDevProvider>;

#[derive(Debug, Clone, Deserialize)]
pub struct ModelsDevProvider {
    pub id: String,
    pub name: String,
    #[serde(default)]
    pub api: Option<String>,
    #[serde(default)]
    pub doc: Option<String>,
    #[serde(default)]
    pub npm: Option<String>,
    #[serde(default)]
    pub env: Vec<String>,
    #[serde(default)]
    pub models: BTreeMap<String, ModelsDevModel>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ModelsDevModel {
    pub id: String,
    pub name: String,
    #[serde(default)]
    pub family: Option<String>,
    #[serde(default)]
    pub release_date: Option<String>,
    #[serde(default)]
    pub attachment: bool,
    #[serde(default)]
    pub reasoning: bool,
    pub temperature: Option<bool>,
    #[serde(default)]
    pub tool_call: bool,
    pub modalities: Option<ModelsDevModalities>,
    #[serde(default)]
    pub open_weights: bool,
    #[serde(default)]
    pub interleaved: Option<serde_json::Value>,
    pub cost: Option<ModelsDevCost>,
    pub limit: Option<ModelsDevLimit>,
    #[serde(default)]
    pub experimental: Option<serde_json::Value>,
    #[serde(default)]
    pub status: Option<String>,
}

#[derive(Debug, Clone, Deserialize, Default)]
pub struct ModelsDevModalities {
    #[serde(default)]
    pub input: Vec<String>,
    #[serde(default)]
    pub output: Vec<String>,
}

#[derive(Debug, Clone, Deserialize, Default)]
pub struct ModelsDevCost {
    pub input: Option<f64>,
    pub output: Option<f64>,
    pub reasoning: Option<f64>,
    #[serde(rename = "cache_read", default)]
    pub cache_read: Option<f64>,
    #[serde(rename = "cache_write", default)]
    pub cache_write: Option<f64>,
    #[serde(rename = "input_audio", default)]
    pub input_audio: Option<f64>,
    #[serde(rename = "output_audio", default)]
    pub output_audio: Option<f64>,
    #[serde(rename = "context_over_200k", default)]
    pub context_over_200k: Option<f64>,
}

#[derive(Debug, Clone, Deserialize, Default)]
pub struct ModelsDevLimit {
    pub context: Option<u32>,
    pub input: Option<u32>,
    pub output: Option<u32>,
}
