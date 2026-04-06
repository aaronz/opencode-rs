use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum PluginCapability {
    ListenEvents,
    RewritePrompt,
    InjectShellEnv,
    AddTools,
    AddContextSources,
    InterceptSensitiveRead,
    SendNotification,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[derive(Default)]
pub struct PluginPermissions {
    pub capabilities: Vec<PluginCapability>,
    pub allowed_events: Vec<String>,
    pub filesystem_scope: Option<String>,
    pub network_allowed: bool,
}


impl PluginPermissions {
    pub fn can(&self, capability: &PluginCapability) -> bool {
        self.capabilities.contains(capability)
    }

    pub fn can_listen_event(&self, event: &str) -> bool {
        self.allowed_events.contains(&event.to_string())
            || self.allowed_events.contains(&"*".to_string())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginConfig {
    pub name: String,
    pub version: String,
    pub enabled: bool,
    pub options: HashMap<String, serde_json::Value>,
    pub permissions: PluginPermissions,
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
    event_handlers: HashMap<String, Vec<String>>,
}

impl PluginManager {
    pub fn new() -> Self {
        Self {
            plugins: HashMap::new(),
            configs: HashMap::new(),
            event_handlers: HashMap::new(),
        }
    }

    pub fn register(
        &mut self,
        plugin: Box<dyn Plugin>,
        config: PluginConfig,
    ) -> Result<(), crate::OpenCodeError> {
        let name = plugin.name().to_string();
        self.plugins.insert(name.clone(), plugin);
        self.configs.insert(name.clone(), config);

        if let Some(plugin_config) = self.configs.get(&name) {
            for event in &plugin_config.permissions.allowed_events {
                self.event_handlers
                    .entry(event.clone())
                    .or_default()
                    .push(name.clone());
            }
        }

        Ok(())
    }

    pub fn get(&self, name: &str) -> Option<&dyn Plugin> {
        self.plugins.get(name).map(|p| p.as_ref())
    }

    pub fn get_config(&self, name: &str) -> Option<&PluginConfig> {
        self.configs.get(name)
    }

    pub fn get_permissions(&self, name: &str) -> Option<&PluginPermissions> {
        self.configs.get(name).map(|c| &c.permissions)
    }

    pub fn can_plugin(&self, name: &str, capability: &PluginCapability) -> bool {
        self.get_permissions(name)
            .map(|p| p.can(capability))
            .unwrap_or(false)
    }

    pub fn get_event_handlers(&self, event: &str) -> Vec<String> {
        self.event_handlers.get(event).cloned().unwrap_or_default()
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

    pub fn unregister(&mut self, name: &str) -> Option<PluginConfig> {
        self.plugins.remove(name);
        self.event_handlers.retain(|_, handlers| {
            handlers.retain(|h| h != name);
            !handlers.is_empty()
        });
        self.configs.remove(name)
    }
}

impl Default for PluginManager {
    fn default() -> Self {
        Self::new()
    }
}
