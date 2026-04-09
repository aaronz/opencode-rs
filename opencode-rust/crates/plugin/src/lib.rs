pub mod discovery;
pub mod loader;
pub mod registry;
pub mod wasm_runtime;

use discovery::PluginDiscovery;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;
use std::path::{Path, PathBuf};

#[derive(Debug, thiserror::Error)]
pub enum PluginError {
    #[error("plugin already registered: {0}")]
    DuplicatePlugin(String),
    #[error("plugin not found: {0}")]
    NotFound(String),
    #[error("plugin IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("plugin metadata parse error: {0}")]
    MetadataParse(#[from] serde_json::Error),
    #[error("plugin load error: {0}")]
    Load(String),
    #[error("plugin startup failed ({0}): {1}")]
    Startup(String, String),
    #[error("plugin shutdown failed ({0}): {1}")]
    Shutdown(String, String),
}

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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginConfig {
    pub name: String,
    pub version: String,
    pub enabled: bool,
    pub options: HashMap<String, Value>,
    pub permissions: PluginPermissions,
}

pub trait Plugin: Send + Sync {
    fn name(&self) -> &str;
    fn version(&self) -> &str;
    fn init(&mut self) -> Result<(), PluginError>;
    fn shutdown(&mut self) -> Result<(), PluginError>;
    fn description(&self) -> &str;

    /// Called after init() during plugin startup. Use for one-time setup.
    /// Default implementation: Ok(())
    fn on_init(&mut self) -> Result<(), PluginError> {
        Ok(())
    }

    /// Called when the runtime starts or a new session begins.
    /// Use for per-session setup. Default implementation: Ok(())
    fn on_start(&mut self) -> Result<(), PluginError> {
        Ok(())
    }

    /// Called before each tool execution. Return Err to block the tool call.
    /// Default implementation: Ok(())
    fn on_tool_call(
        &mut self,
        _tool_name: &str,
        _args: &Value,
        _session_id: &str,
    ) -> Result<(), PluginError> {
        Ok(())
    }

    /// Called when a message is received from the user or agent.
    /// Default implementation: Ok(())
    fn on_message(&mut self, _content: &str, _session_id: &str) -> Result<(), PluginError> {
        Ok(())
    }

    /// Called when a session ends (idle, error, or explicit close).
    /// Use for per-session cleanup. Default implementation: Ok(())
    fn on_session_end(&mut self, _session_id: &str) -> Result<(), PluginError> {
        Ok(())
    }
}

pub struct PluginManager {
    plugins: HashMap<String, Box<dyn Plugin>>,
    configs: HashMap<String, PluginConfig>,
    plugin_paths: HashMap<String, PathBuf>,
    loader: loader::PluginLoader,
    discovered_metadata: Vec<PathBuf>,
}

impl PluginManager {
    pub fn new() -> Self {
        Self {
            plugins: HashMap::new(),
            configs: HashMap::new(),
            plugin_paths: HashMap::new(),
            loader: loader::PluginLoader::new(),
            discovered_metadata: Vec::new(),
        }
    }

    pub fn register(&mut self, plugin: Box<dyn Plugin>) -> Result<(), PluginError> {
        let config = PluginConfig {
            name: plugin.name().to_string(),
            version: plugin.version().to_string(),
            enabled: true,
            options: HashMap::new(),
            permissions: PluginPermissions::default(),
        };

        self.register_with_config(plugin, config)
    }

    pub fn discover_default_dirs(&mut self) -> Result<(), PluginError> {
        self.discover_and_load(None).map(|_| ())
    }

    pub fn discover_from_dirs(&mut self, paths: &[PathBuf]) -> Result<(), PluginError> {
        let discovery = match paths {
            [] => PluginDiscovery::with_dirs(None, None),
            [single] => PluginDiscovery::with_dirs(None, Some(single.clone())),
            [global, project, ..] => {
                PluginDiscovery::with_dirs(Some(global.clone()), Some(project.clone()))
            }
        };

        self.load_discovered(discovery.discover()?)?;
        Ok(())
    }

    pub fn discover_and_load(
        &mut self,
        project_path: Option<&Path>,
    ) -> Result<Vec<String>, PluginError> {
        let discovery = PluginDiscovery::new(project_path);
        self.load_discovered(discovery.discover()?)
    }

    pub fn startup(&mut self) -> Result<(), PluginError> {
        let mut failed = Vec::new();

        for (name, plugin) in self.plugins.iter_mut() {
            if let Err(error) = plugin.init() {
                tracing::warn!(plugin = name, error = %error, "Plugin startup failed");
                failed.push(name.clone());
            }
        }

        if !failed.is_empty() {
            tracing::warn!(plugins = ?failed, "Some plugins failed startup");
        }

        Ok(())
    }

    pub fn shutdown(&mut self) -> Result<(), PluginError> {
        let mut failures = Vec::new();

        for (name, plugin) in self.plugins.iter_mut() {
            if let Err(error) = plugin.shutdown() {
                tracing::warn!(plugin = name, error = %error, "Plugin shutdown failed");
                failures.push((name.clone(), error.to_string()));
            }
        }

        self.plugins.clear();
        self.configs.clear();
        self.plugin_paths.clear();

        if let Some((name, message)) = failures.into_iter().next() {
            return Err(PluginError::Shutdown(name, message));
        }

        Ok(())
    }

    pub fn init_all(&mut self) -> Result<(), PluginError> {
        self.startup()
    }

    pub fn shutdown_all(&mut self) -> Result<(), PluginError> {
        self.shutdown()
    }

    pub fn on_start_all(&mut self) -> Result<(), PluginError> {
        let mut failed = Vec::new();

        for (name, plugin) in self.plugins.iter_mut() {
            if let Err(error) = plugin.on_start() {
                tracing::warn!(plugin = name, error = %error, "Plugin on_start hook failed");
                failed.push(name.clone());
            }
        }

        if !failed.is_empty() {
            tracing::warn!(plugins = ?failed, "Some plugins failed on_start");
        }

        Ok(())
    }

    pub fn on_tool_call_all(
        &mut self,
        tool_name: &str,
        args: &Value,
        session_id: &str,
    ) -> Result<(), PluginError> {
        for (name, plugin) in self.plugins.iter_mut() {
            if let Err(error) = plugin.on_tool_call(tool_name, args, session_id) {
                tracing::warn!(
                    plugin = name,
                    tool = tool_name,
                    error = %error,
                    "Plugin on_tool_call hook blocked execution"
                );
                return Err(error);
            }
        }
        Ok(())
    }

    pub fn on_message_all(&mut self, content: &str, session_id: &str) -> Result<(), PluginError> {
        for (name, plugin) in self.plugins.iter_mut() {
            if let Err(error) = plugin.on_message(content, session_id) {
                tracing::warn!(
                    plugin = name,
                    error = %error,
                    "Plugin on_message hook failed"
                );
            }
        }
        Ok(())
    }

    pub fn on_session_end_all(&mut self, session_id: &str) -> Result<(), PluginError> {
        for (name, plugin) in self.plugins.iter_mut() {
            if let Err(error) = plugin.on_session_end(session_id) {
                tracing::warn!(
                    plugin = name,
                    session_id = session_id,
                    error = %error,
                    "Plugin on_session_end hook failed"
                );
            }
        }
        Ok(())
    }

    pub fn get_plugin(&self, name: &str) -> Option<&dyn Plugin> {
        self.plugins.get(name).map(|p| p.as_ref())
    }

    pub fn get_config(&self, name: &str) -> Option<&PluginConfig> {
        self.configs.get(name)
    }

    pub fn discovered_metadata(&self) -> &[PathBuf] {
        &self.discovered_metadata
    }

    fn register_with_config(
        &mut self,
        plugin: Box<dyn Plugin>,
        config: PluginConfig,
    ) -> Result<(), PluginError> {
        let key = plugin.name().to_string();
        if self.plugins.contains_key(&key) {
            return Err(PluginError::DuplicatePlugin(key));
        }

        self.plugins.insert(key.clone(), plugin);
        self.configs.insert(key, config);
        Ok(())
    }

    fn load_discovered(
        &mut self,
        discovered: Vec<discovery::DiscoveredPlugin>,
    ) -> Result<Vec<String>, PluginError> {
        let mut loaded = Vec::new();

        for entry in discovered {
            self.discovered_metadata.push(entry.metadata_path.clone());

            if !entry.config.enabled {
                tracing::debug!(plugin = entry.config.name, "Skipping disabled plugin");
                continue;
            }

            let plugin_name = entry.config.name.clone();
            let plugin = unsafe { self.loader.load_plugin(&entry.library_path)? };
            self.register_with_config(plugin, entry.config)?;
            self.plugin_paths
                .insert(plugin_name.clone(), entry.library_path);
            loaded.push(plugin_name);
        }

        Ok(loaded)
    }
}

impl Default for PluginManager {
    fn default() -> Self {
        Self::new()
    }
}

pub fn initialize_plugins(project_path: Option<&Path>) -> Result<PluginManager, PluginError> {
    let mut manager = PluginManager::new();
    let loaded = manager.discover_and_load(project_path)?;
    tracing::info!(
        loaded_plugins = loaded.len(),
        "Plugins discovered and loaded"
    );
    manager.startup()?;
    Ok(manager)
}

#[cfg(test)]
mod tests {
    use super::*;

    struct TestPlugin {
        initialized: bool,
        shutdown_called: bool,
        fail_init: bool,
        fail_shutdown: bool,
    }

    impl Plugin for TestPlugin {
        fn name(&self) -> &str {
            "test-plugin"
        }

        fn version(&self) -> &str {
            "1.0.0"
        }

        fn init(&mut self) -> Result<(), PluginError> {
            self.initialized = true;
            if self.fail_init {
                return Err(PluginError::Load("init failed".to_string()));
            }
            Ok(())
        }

        fn shutdown(&mut self) -> Result<(), PluginError> {
            self.shutdown_called = true;
            if self.fail_shutdown {
                return Err(PluginError::Load("shutdown failed".to_string()));
            }
            Ok(())
        }

        fn description(&self) -> &str {
            "test plugin"
        }

        fn on_init(&mut self) -> Result<(), PluginError> {
            Ok(())
        }

        fn on_start(&mut self) -> Result<(), PluginError> {
            Ok(())
        }

        fn on_tool_call(
            &mut self,
            _tool_name: &str,
            _args: &serde_json::Value,
            _session_id: &str,
        ) -> Result<(), PluginError> {
            Ok(())
        }

        fn on_message(&mut self, _: &str, _: &str) -> Result<(), PluginError> {
            Ok(())
        }

        fn on_session_end(&mut self, _: &str) -> Result<(), PluginError> {
            Ok(())
        }
    }

    #[test]
    fn test_register_and_get_plugin() {
        let mut manager = PluginManager::new();
        manager
            .register(Box::new(TestPlugin {
                initialized: false,
                shutdown_called: false,
                fail_init: false,
                fail_shutdown: false,
            }))
            .unwrap();

        assert!(manager.get_plugin("test-plugin").is_some());
    }

    #[test]
    fn startup_non_fatal_when_plugin_init_fails() {
        let mut manager = PluginManager::new();
        manager
            .register(Box::new(TestPlugin {
                initialized: false,
                shutdown_called: false,
                fail_init: true,
                fail_shutdown: false,
            }))
            .unwrap();

        assert!(manager.startup().is_ok());
    }

    #[test]
    fn shutdown_clears_plugins() {
        let mut manager = PluginManager::new();
        manager
            .register(Box::new(TestPlugin {
                initialized: false,
                shutdown_called: false,
                fail_init: false,
                fail_shutdown: false,
            }))
            .unwrap();

        manager.shutdown().unwrap();
        assert!(manager.get_plugin("test-plugin").is_none());
    }

    #[test]
    fn shutdown_reports_failures() {
        let mut manager = PluginManager::new();
        manager
            .register(Box::new(TestPlugin {
                initialized: false,
                shutdown_called: false,
                fail_init: false,
                fail_shutdown: true,
            }))
            .unwrap();

        assert!(matches!(
            manager.shutdown(),
            Err(PluginError::Shutdown(_, _))
        ));
    }

    #[test]
    fn test_duplicate_plugin_registration_fails() {
        let mut manager = PluginManager::new();
        manager
            .register(Box::new(TestPlugin {
                initialized: false,
                shutdown_called: false,
                fail_init: false,
                fail_shutdown: false,
            }))
            .unwrap();

        let result = manager.register(Box::new(TestPlugin {
            initialized: false,
            shutdown_called: false,
            fail_init: false,
            fail_shutdown: false,
        }));

        assert!(matches!(result, Err(PluginError::DuplicatePlugin(_))));
    }

    #[test]
    fn test_get_config_returns_plugin_config() {
        let mut manager = PluginManager::new();
        manager
            .register(Box::new(TestPlugin {
                initialized: false,
                shutdown_called: false,
                fail_init: false,
                fail_shutdown: false,
            }))
            .unwrap();

        let config = manager.get_config("test-plugin");
        assert!(config.is_some());
        let config = config.unwrap();
        assert_eq!(config.name, "test-plugin");
        assert_eq!(config.version, "1.0.0");
        assert!(config.enabled);
    }

    #[test]
    fn test_get_config_returns_none_for_unknown_plugin() {
        let manager = PluginManager::new();
        let config = manager.get_config("non-existent-plugin");
        assert!(config.is_none());
    }

    #[test]
    fn test_init_all_and_shutdown_all() {
        let mut manager = PluginManager::new();
        manager
            .register(Box::new(TestPlugin {
                initialized: false,
                shutdown_called: false,
                fail_init: false,
                fail_shutdown: false,
            }))
            .unwrap();

        assert!(manager.init_all().is_ok());
        assert!(manager.shutdown_all().is_ok());
    }

    #[test]
    fn test_plugin_capabilities_default() {
        let perms = PluginPermissions::default();
        assert!(perms.capabilities.is_empty());
        assert!(perms.allowed_events.is_empty());
        assert!(perms.filesystem_scope.is_none());
        assert!(!perms.network_allowed);
    }

    #[test]
    fn test_plugin_capability_enum() {
        let caps = vec![
            PluginCapability::ListenEvents,
            PluginCapability::RewritePrompt,
            PluginCapability::InjectShellEnv,
            PluginCapability::AddTools,
            PluginCapability::AddContextSources,
            PluginCapability::InterceptSensitiveRead,
            PluginCapability::SendNotification,
        ];

        assert_eq!(caps.len(), 7);
    }

    #[test]
    fn test_plugin_config_with_permissions() {
        let config = PluginConfig {
            name: "test".to_string(),
            version: "2.0.0".to_string(),
            enabled: true,
            options: serde_json::json!({"key": "value"})
                .as_object()
                .unwrap()
                .clone()
                .into_iter()
                .collect(),
            permissions: PluginPermissions {
                capabilities: vec![PluginCapability::AddTools],
                allowed_events: vec!["session.created".to_string()],
                filesystem_scope: Some("/tmp".to_string()),
                network_allowed: true,
            },
        };

        assert_eq!(config.name, "test");
        assert_eq!(
            config.permissions.capabilities,
            vec![PluginCapability::AddTools]
        );
        assert!(config.permissions.network_allowed);
    }

    #[test]
    fn test_on_start_all() {
        let mut manager = PluginManager::new();
        manager
            .register(Box::new(TestPlugin {
                initialized: false,
                shutdown_called: false,
                fail_init: false,
                fail_shutdown: false,
            }))
            .unwrap();

        assert!(manager.on_start_all().is_ok());
    }

    #[test]
    fn test_on_tool_call_all() {
        let mut manager = PluginManager::new();
        manager
            .register(Box::new(TestPlugin {
                initialized: false,
                shutdown_called: false,
                fail_init: false,
                fail_shutdown: false,
            }))
            .unwrap();

        let args = serde_json::json!({"file": "test.txt"});
        let result = manager.on_tool_call_all("read", &args, "session-123");
        assert!(result.is_ok());
    }

    #[test]
    fn test_on_tool_call_all_blocks_on_error() {
        let mut manager = PluginManager::new();

        struct BlockingPlugin;
        impl Plugin for BlockingPlugin {
            fn name(&self) -> &str {
                "blocking-plugin"
            }
            fn version(&self) -> &str {
                "1.0.0"
            }
            fn init(&mut self) -> Result<(), PluginError> {
                Ok(())
            }
            fn shutdown(&mut self) -> Result<(), PluginError> {
                Ok(())
            }
            fn description(&self) -> &str {
                "blocks tool calls"
            }
            fn on_tool_call(
                &mut self,
                tool_name: &str,
                _: &Value,
                _: &str,
            ) -> Result<(), PluginError> {
                if tool_name == "dangerous" {
                    Err(PluginError::Load("blocked".to_string()))
                } else {
                    Ok(())
                }
            }
        }

        manager.register(Box::new(BlockingPlugin)).unwrap();

        let args = serde_json::json!({});
        let result = manager.on_tool_call_all("dangerous", &args, "session-123");
        assert!(result.is_err());
    }

    #[test]
    fn test_on_message_all() {
        let mut manager = PluginManager::new();
        manager
            .register(Box::new(TestPlugin {
                initialized: false,
                shutdown_called: false,
                fail_init: false,
                fail_shutdown: false,
            }))
            .unwrap();

        let result = manager.on_message_all("Hello world", "session-123");
        assert!(result.is_ok());
    }

    #[test]
    fn test_on_session_end_all() {
        let mut manager = PluginManager::new();
        manager
            .register(Box::new(TestPlugin {
                initialized: false,
                shutdown_called: false,
                fail_init: false,
                fail_shutdown: false,
            }))
            .unwrap();

        let result = manager.on_session_end_all("session-123");
        assert!(result.is_ok());
    }
}
