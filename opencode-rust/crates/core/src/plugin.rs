use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::process::{Child as StdChild, Command as StdCommand, Stdio};

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

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
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

    fn get_tools(&self) -> Vec<crate::tool::ToolDefinition> {
        vec![]
    }
}

#[derive(Debug, Clone)]
pub struct SidecarConfig {
    pub command: String,
    pub args: Vec<String>,
    pub env: HashMap<String, String>,
    pub cwd: Option<String>,
}

impl SidecarConfig {
    pub fn new(command: impl Into<String>) -> Self {
        Self {
            command: command.into(),
            args: Vec::new(),
            env: HashMap::new(),
            cwd: None,
        }
    }

    pub fn args(mut self, args: Vec<String>) -> Self {
        self.args = args;
        self
    }

    pub fn env(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.env.insert(key.into(), value.into());
        self
    }

    pub fn cwd(mut self, path: impl Into<String>) -> Self {
        self.cwd = Some(path.into());
        self
    }
}

pub struct PluginManager {
    plugins: HashMap<String, Box<dyn Plugin>>,
    configs: HashMap<String, PluginConfig>,
    event_handlers: HashMap<String, Vec<String>>,
    enabled_plugins: std::collections::HashSet<String>,
    sidecar_processes: HashMap<String, StdChild>,
}

impl PluginManager {
    pub fn new() -> Self {
        Self {
            plugins: HashMap::new(),
            configs: HashMap::new(),
            event_handlers: HashMap::new(),
            enabled_plugins: std::collections::HashSet::new(),
            sidecar_processes: HashMap::new(),
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

    pub fn collect_plugin_tools(&self) -> Vec<(String, crate::tool::ToolDefinition)> {
        let mut tools = Vec::new();
        for (name, config) in &self.configs {
            if config.permissions.can(&PluginCapability::AddTools) && self.is_enabled(name) {
                if let Some(plugin) = self.plugins.get(name) {
                    for tool in plugin.get_tools() {
                        let prefixed_name = format!("{}/{}", name, tool.name);
                        tools.push((prefixed_name, tool));
                    }
                }
            }
        }
        tools
    }

    pub fn enable(&mut self, name: &str) -> Result<(), String> {
        if !self.plugins.contains_key(name) {
            return Err(format!("Plugin '{}' not found", name));
        }
        self.enabled_plugins.insert(name.to_string());
        if let Some(config) = self.configs.get_mut(name) {
            config.enabled = true;
        }
        Ok(())
    }

    pub fn disable(&mut self, name: &str) -> Result<(), String> {
        if !self.plugins.contains_key(name) {
            return Err(format!("Plugin '{}' not found", name));
        }
        self.enabled_plugins.remove(name);
        if let Some(config) = self.configs.get_mut(name) {
            config.enabled = false;
        }
        Ok(())
    }

    pub fn is_enabled(&self, name: &str) -> bool {
        self.enabled_plugins.contains(name)
    }

    pub fn enabled_plugins(&self) -> Vec<&str> {
        self.enabled_plugins.iter().map(|s| s.as_str()).collect()
    }

    pub fn start_sidecar(
        &mut self,
        plugin_name: &str,
        config: SidecarConfig,
    ) -> Result<(), String> {
        if self.sidecar_processes.contains_key(plugin_name) {
            return Err(format!("Sidecar for '{}' already running", plugin_name));
        }

        let mut cmd = StdCommand::new(&config.command);
        cmd.args(&config.args)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped());

        for (key, value) in &config.env {
            cmd.env(key, value);
        }

        if let Some(cwd) = &config.cwd {
            cmd.current_dir(cwd);
        }

        let child = cmd
            .spawn()
            .map_err(|e| format!("Failed to spawn sidecar: {}", e))?;

        self.sidecar_processes
            .insert(plugin_name.to_string(), child);
        Ok(())
    }

    pub fn stop_sidecar(&mut self, plugin_name: &str) -> Result<(), String> {
        match self.sidecar_processes.remove(plugin_name) {
            Some(mut child) => {
                child
                    .kill()
                    .map_err(|e| format!("Failed to kill sidecar: {}", e))?;
                Ok(())
            }
            None => Err(format!("No sidecar running for '{}'", plugin_name)),
        }
    }

    pub fn is_sidecar_running(&self, plugin_name: &str) -> bool {
        self.sidecar_processes.contains_key(plugin_name)
    }

    pub fn list_sidecars(&self) -> Vec<&str> {
        self.sidecar_processes.keys().map(|s| s.as_str()).collect()
    }
}

impl Default for PluginManager {
    fn default() -> Self {
        Self::new()
    }
}
