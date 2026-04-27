use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ControlPlaneConfig {
    pub enabled: bool,
    pub endpoint: Option<String>,
    pub api_key: Option<String>,
    pub features: HashMap<String, bool>,
}

pub struct ControlPlane {
    pub(crate) config: ControlPlaneConfig,
}

impl ControlPlane {
    pub fn new(config: ControlPlaneConfig) -> Self {
        Self { config }
    }

    pub fn is_enabled(&self) -> bool {
        self.config.enabled
    }

    pub fn has_feature(&self, feature: &str) -> bool {
        self.config.features.get(feature).copied().unwrap_or(false)
    }

    pub fn endpoint(&self) -> Option<&str> {
        self.config.endpoint.as_deref()
    }
}

impl Default for ControlPlane {
    fn default() -> Self {
        Self::new(ControlPlaneConfig {
            enabled: false,
            endpoint: None,
            api_key: None,
            features: HashMap::new(),
        })
    }
}