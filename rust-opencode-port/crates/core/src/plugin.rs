use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginConfig {
    pub name: String,
    pub version: String,
    pub enabled: bool,
    pub options: HashMap<String, serde_json::Value>,
}

pub trait Plugin: Send + Sync {
    fn name(&self) -> &str;
    fn version(&self) -> &str;
    fn initialize(&mut self, config: &PluginConfig) -> Result<(), crate::OpenCodeError>;
    fn execute(
        &self,
        action: &str,
        params: serde_json::Value,
    ) -> Result<serde_json::Value, crate::OpenCodeError>;
}

pub struct PluginManager {
    plugins: HashMap<String, Box<dyn Plugin>>,
    configs: HashMap<String, PluginConfig>,
}

impl PluginManager {
    pub fn new() -> Self {
        Self {
            plugins: HashMap::new(),
            configs: HashMap::new(),
        }
    }

    pub fn register(
        &mut self,
        plugin: Box<dyn Plugin>,
        config: PluginConfig,
    ) -> Result<(), crate::OpenCodeError> {
        let name = plugin.name().to_string();
        self.plugins.insert(name.clone(), plugin);
        self.configs.insert(name, config);
        Ok(())
    }

    pub fn get(&self, name: &str) -> Option<&dyn Plugin> {
        self.plugins.get(name).map(|p| p.as_ref())
    }

    pub fn execute(
        &self,
        plugin_name: &str,
        action: &str,
        params: serde_json::Value,
    ) -> Result<serde_json::Value, crate::OpenCodeError> {
        let plugin = self.get(plugin_name).ok_or_else(|| {
            crate::OpenCodeError::Tool(format!("Plugin not found: {}", plugin_name))
        })?;
        plugin.execute(action, params)
    }

    pub fn list(&self) -> Vec<&str> {
        self.plugins.keys().map(|k| k.as_str()).collect()
    }
}

impl Default for PluginManager {
    fn default() -> Self {
        Self::new()
    }
}
