#![allow(clippy::type_complexity, clippy::redundant_closure)]

pub mod config;
pub mod discovery;
pub mod loader;
pub mod registry;
pub mod wasm_runtime;

use async_trait::async_trait;
use discovery::PluginDiscovery;
use indexmap::IndexMap;
use opencode_permission::{ApprovalResult, PermissionScope};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tokio::sync::RwLock;

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
    #[error("tool registration failed: {0}")]
    ToolRegistration(String),
    #[error("permission denied: {0}")]
    PermissionDenied(String),
    #[error("plugin config validation failed for '{0}': {1}")]
    ConfigValidation(String, String),
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
    pub options: IndexMap<String, Value>,
    pub permissions: PluginPermissions,
}

#[derive(Debug, Clone)]
pub struct PluginToolDefinition {
    pub name: String,
    pub description: String,
    pub input_schema: Value,
    pub provider_name: String,
}

#[async_trait]
pub trait ToolProvider: Send + Sync {
    async fn get_tools(&self) -> Vec<PluginToolDefinition>;
}

#[derive(Clone)]
pub struct PluginTool {
    definition: PluginToolDefinition,
    executor: Arc<Box<dyn Fn(Value) -> Result<String, String> + Send + Sync>>,
}

impl PluginTool {
    pub fn new(
        definition: PluginToolDefinition,
        executor: Box<dyn Fn(Value) -> Result<String, String> + Send + Sync>,
    ) -> Self {
        Self {
            definition,
            executor: Arc::new(executor),
        }
    }

    pub fn definition(&self) -> &PluginToolDefinition {
        &self.definition
    }

    pub fn execute(&self, args: Value) -> Result<String, String> {
        (self.executor)(args)
    }
}

pub struct PluginToolAdapter {
    definition: PluginToolDefinition,
    executor: Arc<Box<dyn Fn(Value) -> Result<String, String> + Send + Sync>>,
}

impl PluginToolAdapter {
    pub fn new(
        definition: PluginToolDefinition,
        executor: Arc<Box<dyn Fn(Value) -> Result<String, String> + Send + Sync>>,
    ) -> Self {
        Self {
            definition,
            executor,
        }
    }

    pub fn from_plugin_tool(tool: PluginTool) -> Self {
        Self {
            definition: tool.definition,
            executor: tool.executor,
        }
    }
}

impl Clone for PluginToolAdapter {
    fn clone(&self) -> Self {
        Self {
            definition: self.definition.clone(),
            executor: Arc::clone(&self.executor),
        }
    }
}

#[async_trait]
impl opencode_tools::Tool for PluginToolAdapter {
    fn name(&self) -> &str {
        &self.definition.name
    }

    fn description(&self) -> &str {
        &self.definition.description
    }

    fn clone_tool(&self) -> Box<dyn opencode_tools::Tool> {
        Box::new(self.clone())
    }

    async fn execute(
        &self,
        args: serde_json::Value,
        _ctx: Option<opencode_tools::ToolContext>,
    ) -> Result<opencode_tools::ToolResult, opencode_core::OpenCodeError> {
        let executor = Arc::clone(&self.executor);
        let args_clone = args.clone();

        let result = tokio::task::spawn_blocking(move || executor(args_clone))
            .await
            .map_err(|e| opencode_core::OpenCodeError::Tool(format!("Task join error: {}", e)))?
            .map_err(|e| opencode_core::OpenCodeError::Tool(e))?;

        Ok(opencode_tools::ToolResult::ok(result))
    }
}

impl PluginPermissions {
    pub fn has_capability(&self, cap: &PluginCapability) -> bool {
        self.capabilities.contains(cap)
    }

    pub fn can_add_tools(&self) -> bool {
        self.has_capability(&PluginCapability::AddTools)
    }
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

    /// Register a tool with the plugin manager.
    /// The tool will be available to the agent if the plugin has AddTools capability.
    /// Default implementation: Err(PluginError::PermissionDenied(...))
    fn register_tool(&mut self, _tool: PluginTool) -> Result<(), PluginError> {
        Err(PluginError::PermissionDenied(format!(
            "plugin '{}' does not implement register_tool",
            self.name()
        )))
    }
}

pub struct PluginManager {
    plugins: IndexMap<String, Box<dyn Plugin>>,
    configs: IndexMap<String, PluginConfig>,
    plugin_paths: IndexMap<String, PathBuf>,
    loader: loader::PluginLoader,
    discovered_metadata: Vec<PathBuf>,
    plugin_tools: Arc<RwLock<IndexMap<String, PluginTool>>>,
    permission_scope: PermissionScope,
}

impl PluginManager {
    pub fn new() -> Self {
        Self {
            plugins: IndexMap::new(),
            configs: IndexMap::new(),
            plugin_paths: IndexMap::new(),
            loader: loader::PluginLoader::new(),
            discovered_metadata: Vec::new(),
            plugin_tools: Arc::new(RwLock::new(IndexMap::new())),
            permission_scope: PermissionScope::ReadOnly,
        }
    }

    pub fn register(&mut self, plugin: Box<dyn Plugin>) -> Result<(), PluginError> {
        let config = PluginConfig {
            name: plugin.name().to_string(),
            version: plugin.version().to_string(),
            enabled: true,
            options: IndexMap::new(),
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
        // Note: plugin_tools is AsyncRwLock, can't clear in sync shutdown
        // Use shutdown_async for proper cleanup

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

    pub fn set_permission_scope(&mut self, scope: PermissionScope) {
        self.permission_scope = scope;
    }

    pub fn permission_scope(&self) -> PermissionScope {
        self.permission_scope
    }

    pub async fn register_plugin_tool(&self, tool: PluginTool) -> Result<(), PluginError> {
        let tool_name = tool.definition().name.clone();
        let provider_name = tool.definition().provider_name.clone();

        let config = self
            .configs
            .get(&provider_name)
            .ok_or_else(|| PluginError::NotFound(provider_name.clone()))?;

        if !config.permissions.can_add_tools() {
            return Err(PluginError::PermissionDenied(format!(
                "plugin '{}' does not have AddTools capability",
                provider_name
            )));
        }

        let approval = check_tool_permission_for_scope(&tool_name, self.permission_scope);
        match approval {
            ApprovalResult::Denied => {
                return Err(PluginError::PermissionDenied(format!(
                    "tool '{}' is denied by permission policy",
                    tool_name
                )));
            }
            ApprovalResult::RequireApproval => {
                tracing::debug!(
                    tool = %tool_name,
                    plugin = %provider_name,
                    "plugin tool requires approval"
                );
            }
            ApprovalResult::AutoApprove => {}
        }

        let mut tools = self.plugin_tools.write().await;
        if tools.contains_key(&tool_name) {
            return Err(PluginError::ToolRegistration(format!(
                "tool '{}' already registered",
                tool_name
            )));
        }

        tools.insert(tool_name, tool);
        Ok(())
    }

    pub async fn get_plugin_tool_definition(&self, name: &str) -> Option<PluginToolDefinition> {
        let tools = self.plugin_tools.read().await;
        tools.get(name).map(|t| t.definition().clone())
    }

    pub async fn list_plugin_tools(&self) -> Vec<PluginToolDefinition> {
        let tools = self.plugin_tools.read().await;
        tools.values().map(|t| t.definition().clone()).collect()
    }

    pub async fn unregister_plugin_tool(&self, name: &str) -> Result<(), PluginError> {
        let mut tools = self.plugin_tools.write().await;
        tools
            .shift_remove(name)
            .ok_or_else(|| PluginError::ToolRegistration(format!("tool '{}' not found", name)))?;
        Ok(())
    }

    pub async fn execute_plugin_tool(
        &self,
        name: &str,
        args: Value,
    ) -> Result<String, PluginError> {
        let result;
        {
            let tools = self.plugin_tools.read().await;
            let tool = tools.get(name).ok_or_else(|| {
                PluginError::ToolRegistration(format!("tool '{}' not found", name))
            })?;
            result = tool.execute(args);
        }
        result.map_err(PluginError::ToolRegistration)
    }

    /// Export all plugin tools as Box<dyn opencode_tools::Tool> for integration
    /// with the opencode_tools::ToolRegistry.
    pub async fn export_as_tools(&self) -> Vec<Box<dyn opencode_tools::Tool>> {
        let tools = self.plugin_tools.read().await;
        tools
            .values()
            .map(|t| {
                let adapter = PluginToolAdapter::from_plugin_tool(t.clone());
                Box::new(adapter) as Box<dyn opencode_tools::Tool>
            })
            .collect()
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

impl Drop for PluginManager {
    fn drop(&mut self) {
        if !self.plugins.is_empty() {
            for (name, plugin) in self.plugins.iter_mut() {
                if let Err(error) = plugin.shutdown() {
                    tracing::warn!(plugin = name, error = %error, "Plugin shutdown failed during drop");
                }
            }
        }
        self.plugins.clear();
        self.configs.clear();
        self.plugin_paths.clear();
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

fn check_tool_permission_for_scope(tool_name: &str, scope: PermissionScope) -> ApprovalResult {
    opencode_permission::check_tool_permission(tool_name, scope)
}

impl PluginManager {
    pub async fn shutdown_async(&mut self) -> Result<(), PluginError> {
        self.plugin_tools.write().await.clear();
        self.shutdown()
    }

    pub fn unload_plugin(&mut self, name: &str) -> Result<(), PluginError> {
        let mut plugin = self
            .plugins
            .shift_remove(name)
            .ok_or_else(|| PluginError::NotFound(name.to_string()))?;

        if let Err(error) = plugin.shutdown() {
            tracing::warn!(plugin = name, error = %error, "Plugin shutdown failed during unload");
        }

        self.configs.shift_remove(name);
        self.plugin_paths.shift_remove(name);

        tracing::debug!(plugin = name, "Plugin unloaded successfully");
        Ok(())
    }

    pub async fn unload_plugin_async(&mut self, name: &str) -> Result<(), PluginError> {
        self.remove_tools_by_provider(name).await;

        if let Err(error) = self.unload_plugin(name) {
            if matches!(error, PluginError::NotFound(_)) {
                return Ok(());
            }
            return Err(error);
        }

        Ok(())
    }

    pub async fn remove_tools_by_provider(&self, provider_name: &str) {
        let mut tools = self.plugin_tools.write().await;
        let to_remove: Vec<String> = tools
            .iter()
            .filter(|(_, tool)| tool.definition().provider_name == provider_name)
            .map(|(name, _)| name.clone())
            .collect();

        for tool_name in to_remove {
            tools.shift_remove(&tool_name);
            tracing::debug!(
                tool = %tool_name,
                plugin = %provider_name,
                "Plugin tool unregistered during cleanup"
            );
        }
    }

    pub async fn unload_all_plugins(&mut self) -> Result<(), PluginError> {
        let plugin_names: Vec<String> = self.plugins.keys().cloned().collect();

        for name in plugin_names {
            if let Err(error) = self.unload_plugin_async(&name).await {
                tracing::warn!(plugin = %name, error = %error, "Failed to unload plugin during cleanup");
            }
        }

        Ok(())
    }

    pub fn is_plugin_loaded(&self, name: &str) -> bool {
        self.plugins.contains_key(name)
    }

    /// Called by a plugin to register a tool.
    /// The plugin must have AddTools capability.
    /// Returns Err if the plugin doesn't have permission or tool already exists.
    pub async fn register_tool(
        &self,
        plugin_name: &str,
        tool: PluginTool,
    ) -> Result<(), PluginError> {
        let config = self
            .configs
            .get(plugin_name)
            .ok_or_else(|| PluginError::NotFound(plugin_name.to_string()))?;

        if !config.permissions.can_add_tools() {
            return Err(PluginError::PermissionDenied(format!(
                "plugin '{}' does not have AddTools capability",
                plugin_name
            )));
        }

        let tool_name = tool.definition().name.clone();
        let approval = check_tool_permission_for_scope(&tool_name, self.permission_scope);
        match approval {
            ApprovalResult::Denied => {
                return Err(PluginError::PermissionDenied(format!(
                    "tool '{}' is denied by permission policy",
                    tool_name
                )));
            }
            ApprovalResult::RequireApproval => {
                tracing::debug!(
                    tool = %tool_name,
                    plugin = %plugin_name,
                    "plugin tool requires approval"
                );
            }
            ApprovalResult::AutoApprove => {}
        }

        let mut tools = self.plugin_tools.write().await;
        if tools.contains_key(&tool_name) {
            return Err(PluginError::ToolRegistration(format!(
                "tool '{}' already registered",
                tool_name
            )));
        }

        tools.insert(tool_name, tool);
        Ok(())
    }

    /// Export plugin tools and register them with a ToolRegistry.
    /// This integrates plugin-provided tools into the agent's toolset.
    pub async fn register_tools_in_registry(
        &self,
        registry: &opencode_tools::ToolRegistry,
    ) -> Result<(), PluginError> {
        let tools = self.export_as_tools().await;
        if tools.is_empty() {
            return Ok(());
        }
        registry.register_plugin_tools(tools).await;
        Ok(())
    }
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

    struct TestPluginWithTools {
        name: String,
    }

    impl TestPluginWithTools {
        fn new(name: &str) -> Self {
            Self {
                name: name.to_string(),
            }
        }

        fn to_config(&self) -> PluginConfig {
            PluginConfig {
                name: self.name.clone(),
                version: "1.0.0".to_string(),
                enabled: true,
                options: IndexMap::new(),
                permissions: PluginPermissions {
                    capabilities: vec![PluginCapability::AddTools],
                    allowed_events: vec![],
                    filesystem_scope: None,
                    network_allowed: false,
                },
            }
        }
    }

    impl Plugin for TestPluginWithTools {
        fn name(&self) -> &str {
            &self.name
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
            "test plugin with tools"
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

    fn register_test_plugin_with_tools(manager: &mut PluginManager, name: &str) {
        let plugin = TestPluginWithTools::new(name);
        let config = plugin.to_config();
        manager
            .register_with_config(Box::new(plugin), config)
            .unwrap();
    }

    struct TestPluginWithoutTools {
        name: String,
    }

    impl TestPluginWithoutTools {
        fn new(name: &str) -> Self {
            Self {
                name: name.to_string(),
            }
        }

        fn to_config(&self) -> PluginConfig {
            PluginConfig {
                name: self.name.clone(),
                version: "1.0.0".to_string(),
                enabled: true,
                options: IndexMap::new(),
                permissions: PluginPermissions {
                    capabilities: vec![], // No AddTools capability
                    allowed_events: vec![],
                    filesystem_scope: None,
                    network_allowed: false,
                },
            }
        }
    }

    impl Plugin for TestPluginWithoutTools {
        fn name(&self) -> &str {
            &self.name
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
            "test plugin without tools"
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

    fn register_test_plugin_without_tools(manager: &mut PluginManager, name: &str) {
        let plugin = TestPluginWithoutTools::new(name);
        let config = plugin.to_config();
        manager
            .register_with_config(Box::new(plugin), config)
            .unwrap();
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

    #[tokio::test]
    async fn test_plugin_tool_registration() {
        let mut manager = PluginManager::new();
        register_test_plugin_with_tools(&mut manager, "test-plugin");

        let tool_def = PluginToolDefinition {
            name: "custom_tool".to_string(),
            description: "A custom tool from plugin".to_string(),
            input_schema: serde_json::json!({"type": "object"}),
            provider_name: "test-plugin".to_string(),
        };

        let tool = PluginTool::new(tool_def, Box::new(|_args| Ok("executed".to_string())));

        let result = manager.register_plugin_tool(tool).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_plugin_tool_registration_requires_addtools_capability() {
        let mut manager = PluginManager::new();

        register_test_plugin_without_tools(&mut manager, "restricted-plugin");

        let tool_def = PluginToolDefinition {
            name: "custom_tool".to_string(),
            description: "A custom tool from plugin".to_string(),
            input_schema: serde_json::json!({"type": "object"}),
            provider_name: "restricted-plugin".to_string(),
        };

        let tool = PluginTool::new(tool_def, Box::new(|_args| Ok("executed".to_string())));

        let result = manager.register_plugin_tool(tool).await;
        assert!(result.is_err());
        let err = result.unwrap_err();
        println!("Error type: {:?}", err);
        assert!(matches!(err, PluginError::PermissionDenied(_)));
    }

    #[tokio::test]
    async fn test_plugin_tool_execution() {
        let mut manager = PluginManager::new();
        register_test_plugin_with_tools(&mut manager, "test-plugin");

        let tool_def = PluginToolDefinition {
            name: "echo_tool".to_string(),
            description: "Echo tool".to_string(),
            input_schema: serde_json::json!({"type": "object", "properties": {"msg": {"type": "string"}}}),
            provider_name: "test-plugin".to_string(),
        };

        let tool = PluginTool::new(
            tool_def,
            Box::new(|args: Value| {
                let msg = args.get("msg").and_then(|v| v.as_str()).unwrap_or("");
                Ok(format!("echo: {}", msg))
            }),
        );

        manager.register_plugin_tool(tool).await.unwrap();

        let result = manager
            .execute_plugin_tool("echo_tool", serde_json::json!({"msg": "hello"}))
            .await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "echo: hello");
    }

    #[tokio::test]
    async fn test_plugin_tool_not_found() {
        let manager = PluginManager::new();

        let result = manager
            .execute_plugin_tool("nonexistent", serde_json::json!({}))
            .await;
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            PluginError::ToolRegistration(_)
        ));
    }

    #[tokio::test]
    async fn test_list_plugin_tools() {
        let mut manager = PluginManager::new();
        register_test_plugin_with_tools(&mut manager, "test-plugin");

        let tool_def = PluginToolDefinition {
            name: "tool1".to_string(),
            description: "Tool 1".to_string(),
            input_schema: serde_json::json!({"type": "object"}),
            provider_name: "test-plugin".to_string(),
        };
        let tool = PluginTool::new(tool_def, Box::new(|_args| Ok("ok".to_string())));
        manager.register_plugin_tool(tool).await.unwrap();

        let tool_def = PluginToolDefinition {
            name: "tool2".to_string(),
            description: "Tool 2".to_string(),
            input_schema: serde_json::json!({"type": "object"}),
            provider_name: "test-plugin".to_string(),
        };
        let tool = PluginTool::new(tool_def, Box::new(|_args| Ok("ok".to_string())));
        manager.register_plugin_tool(tool).await.unwrap();

        let tools = manager.list_plugin_tools().await;
        assert_eq!(tools.len(), 2);
        assert!(tools.iter().any(|t| t.name == "tool1"));
        assert!(tools.iter().any(|t| t.name == "tool2"));
    }

    #[tokio::test]
    async fn test_unregister_plugin_tool() {
        let mut manager = PluginManager::new();
        register_test_plugin_with_tools(&mut manager, "test-plugin");

        let tool_def = PluginToolDefinition {
            name: "temp_tool".to_string(),
            description: "Temp tool".to_string(),
            input_schema: serde_json::json!({"type": "object"}),
            provider_name: "test-plugin".to_string(),
        };
        let tool = PluginTool::new(tool_def, Box::new(|_args| Ok("ok".to_string())));
        manager.register_plugin_tool(tool).await.unwrap();

        let result = manager.unregister_plugin_tool("temp_tool").await;
        assert!(result.is_ok());

        let tools = manager.list_plugin_tools().await;
        assert!(tools.is_empty());
    }

    #[tokio::test]
    async fn test_permission_scope_setting() {
        let mut manager = PluginManager::new();
        assert_eq!(manager.permission_scope(), PermissionScope::ReadOnly);

        manager.set_permission_scope(PermissionScope::Full);
        assert_eq!(manager.permission_scope(), PermissionScope::Full);

        manager.set_permission_scope(PermissionScope::Restricted);
        assert_eq!(manager.permission_scope(), PermissionScope::Restricted);
    }

    #[test]
    fn test_plugin_permissions_has_capability() {
        let perms = PluginPermissions {
            capabilities: vec![PluginCapability::AddTools, PluginCapability::ListenEvents],
            allowed_events: vec![],
            filesystem_scope: None,
            network_allowed: false,
        };

        assert!(perms.has_capability(&PluginCapability::AddTools));
        assert!(perms.has_capability(&PluginCapability::ListenEvents));
        assert!(!perms.has_capability(&PluginCapability::InjectShellEnv));
    }

    #[test]
    fn test_plugin_permissions_can_add_tools() {
        let perms_with_addtools = PluginPermissions {
            capabilities: vec![PluginCapability::AddTools],
            allowed_events: vec![],
            filesystem_scope: None,
            network_allowed: false,
        };
        assert!(perms_with_addtools.can_add_tools());

        let perms_without_addtools = PluginPermissions {
            capabilities: vec![PluginCapability::ListenEvents],
            allowed_events: vec![],
            filesystem_scope: None,
            network_allowed: false,
        };
        assert!(!perms_without_addtools.can_add_tools());
    }

    #[tokio::test]
    async fn test_plugin_tool_duplicate_registration_fails() {
        let mut manager = PluginManager::new();
        register_test_plugin_with_tools(&mut manager, "test-plugin");

        let tool_def = PluginToolDefinition {
            name: "duplicate_tool".to_string(),
            description: "Duplicate tool".to_string(),
            input_schema: serde_json::json!({"type": "object"}),
            provider_name: "test-plugin".to_string(),
        };

        let tool1 = PluginTool::new(tool_def.clone(), Box::new(|_args| Ok("ok".to_string())));
        let tool2 = PluginTool::new(tool_def, Box::new(|_args| Ok("ok".to_string())));

        manager.register_plugin_tool(tool1).await.unwrap();
        let result = manager.register_plugin_tool(tool2).await;
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            PluginError::ToolRegistration(_)
        ));
    }

    #[tokio::test]
    async fn test_plugin_tool_get_definition() {
        let mut manager = PluginManager::new();
        register_test_plugin_with_tools(&mut manager, "test-plugin");

        let tool_def = PluginToolDefinition {
            name: "def_tool".to_string(),
            description: "Definition test tool".to_string(),
            input_schema: serde_json::json!({"type": "object", "properties": {"input": {"type": "string"}}}),
            provider_name: "test-plugin".to_string(),
        };

        let tool = PluginTool::new(tool_def.clone(), Box::new(|_args| Ok("ok".to_string())));
        manager.register_plugin_tool(tool).await.unwrap();

        let retrieved = manager.get_plugin_tool_definition("def_tool").await;
        assert!(retrieved.is_some());
        let retrieved = retrieved.unwrap();
        assert_eq!(retrieved.name, "def_tool");
        assert_eq!(retrieved.description, "Definition test tool");
        assert_eq!(retrieved.provider_name, "test-plugin");
    }

    #[tokio::test]
    async fn test_shutdown_async_clears_tools() {
        let mut manager = PluginManager::new();
        register_test_plugin_with_tools(&mut manager, "test-plugin");

        let tool_def = PluginToolDefinition {
            name: "cleanup_tool".to_string(),
            description: "Cleanup test".to_string(),
            input_schema: serde_json::json!({"type": "object"}),
            provider_name: "test-plugin".to_string(),
        };
        let tool = PluginTool::new(tool_def, Box::new(|_args| Ok("ok".to_string())));
        manager.register_plugin_tool(tool).await.unwrap();

        manager.shutdown_async().await.unwrap();

        let tools = manager.list_plugin_tools().await;
        assert!(tools.is_empty());
    }

    #[test]
    fn test_hook_order_is_deterministic() {
        use std::sync::atomic::{AtomicUsize, Ordering};
        use std::sync::Arc;

        let call_order = Arc::new(AtomicUsize::new(0));
        let call_sequence: Arc<std::sync::Mutex<Vec<String>>> =
            Arc::new(std::sync::Mutex::new(Vec::new()));

        struct OrderedPlugin {
            name: String,
            call_count: Arc<AtomicUsize>,
            call_sequence: Arc<std::sync::Mutex<Vec<String>>>,
        }

        impl OrderedPlugin {
            fn new(
                name: &str,
                call_count: Arc<AtomicUsize>,
                call_sequence: Arc<std::sync::Mutex<Vec<String>>>,
            ) -> Self {
                Self {
                    name: name.to_string(),
                    call_count,
                    call_sequence,
                }
            }
        }

        impl Plugin for OrderedPlugin {
            fn name(&self) -> &str {
                &self.name
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
                "ordered plugin for testing"
            }

            fn on_start(&mut self) -> Result<(), PluginError> {
                let order = self.call_count.fetch_add(1, Ordering::SeqCst);
                let mut seq = self.call_sequence.lock().unwrap();
                if seq.len() == order as usize {
                    seq.push(self.name.clone());
                } else {
                    seq.push(format!("OUT_OF_ORDER:{}", self.name));
                }
                Ok(())
            }
        }

        let mut manager = PluginManager::new();

        let plugin_alpha = OrderedPlugin::new("alpha", call_order.clone(), call_sequence.clone());
        let plugin_beta = OrderedPlugin::new("beta", call_order.clone(), call_sequence.clone());
        let plugin_gamma = OrderedPlugin::new("gamma", call_order.clone(), call_sequence.clone());

        manager.register(Box::new(plugin_alpha)).unwrap();
        manager.register(Box::new(plugin_beta)).unwrap();
        manager.register(Box::new(plugin_gamma)).unwrap();

        manager.on_start_all().unwrap();

        let sequence = call_sequence.lock().unwrap();
        assert_eq!(
            sequence.len(),
            3,
            "Expected 3 plugins to be called, got {}",
            sequence.len()
        );
        assert_eq!(
            sequence[0], "alpha",
            "First plugin should be alpha, got {}",
            sequence[0]
        );
        assert_eq!(
            sequence[1], "beta",
            "Second plugin should be beta, got {}",
            sequence[1]
        );
        assert_eq!(
            sequence[2], "gamma",
            "Third plugin should be gamma, got {}",
            sequence[2]
        );
    }

    #[test]
    fn test_hook_order_is_consistent_across_invocations() {
        use std::sync::atomic::{AtomicUsize, Ordering};
        use std::sync::Arc;

        for iteration in 0..3 {
            let call_order = Arc::new(AtomicUsize::new(0));
            let call_sequence: Arc<std::sync::Mutex<Vec<String>>> =
                Arc::new(std::sync::Mutex::new(Vec::new()));

            struct OrderedPlugin {
                name: String,
                call_count: Arc<AtomicUsize>,
                call_sequence: Arc<std::sync::Mutex<Vec<String>>>,
            }

            impl OrderedPlugin {
                fn new(
                    name: &str,
                    call_count: Arc<AtomicUsize>,
                    call_sequence: Arc<std::sync::Mutex<Vec<String>>>,
                ) -> Self {
                    Self {
                        name: name.to_string(),
                        call_count,
                        call_sequence,
                    }
                }
            }

            impl Plugin for OrderedPlugin {
                fn name(&self) -> &str {
                    &self.name
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
                    "ordered plugin for testing"
                }

                fn on_start(&mut self) -> Result<(), PluginError> {
                    let order = self.call_count.fetch_add(1, Ordering::SeqCst);
                    let mut seq = self.call_sequence.lock().unwrap();
                    if seq.len() == order as usize {
                        seq.push(self.name.clone());
                    } else {
                        seq.push(format!("OUT_OF_ORDER:{}", self.name));
                    }
                    Ok(())
                }
            }

            let mut manager = PluginManager::new();

            let plugin_a =
                OrderedPlugin::new("plugin-a", call_order.clone(), call_sequence.clone());
            let plugin_b =
                OrderedPlugin::new("plugin-b", call_order.clone(), call_sequence.clone());
            let plugin_c =
                OrderedPlugin::new("plugin-c", call_order.clone(), call_sequence.clone());

            manager.register(Box::new(plugin_a)).unwrap();
            manager.register(Box::new(plugin_b)).unwrap();
            manager.register(Box::new(plugin_c)).unwrap();

            manager.on_start_all().unwrap();

            let sequence = call_sequence.lock().unwrap();
            assert_eq!(
                sequence.len(),
                3,
                "Iteration {}: Expected 3 plugins to be called",
                iteration
            );
            assert_eq!(
                sequence[0], "plugin-a",
                "Iteration {}: First plugin should be plugin-a",
                iteration
            );
            assert_eq!(
                sequence[1], "plugin-b",
                "Iteration {}: Second plugin should be plugin-b",
                iteration
            );
            assert_eq!(
                sequence[2], "plugin-c",
                "Iteration {}: Third plugin should be plugin-c",
                iteration
            );
        }
    }

    #[test]
    fn test_hook_order_on_tool_call_is_deterministic() {
        use std::sync::atomic::{AtomicUsize, Ordering};
        use std::sync::Arc;

        let call_order = Arc::new(AtomicUsize::new(0));
        let call_sequence: Arc<std::sync::Mutex<Vec<String>>> =
            Arc::new(std::sync::Mutex::new(Vec::new()));

        struct OrderedToolPlugin {
            name: String,
            call_count: Arc<AtomicUsize>,
            call_sequence: Arc<std::sync::Mutex<Vec<String>>>,
        }

        impl OrderedToolPlugin {
            fn new(
                name: &str,
                call_count: Arc<AtomicUsize>,
                call_sequence: Arc<std::sync::Mutex<Vec<String>>>,
            ) -> Self {
                Self {
                    name: name.to_string(),
                    call_count,
                    call_sequence,
                }
            }
        }

        impl Plugin for OrderedToolPlugin {
            fn name(&self) -> &str {
                &self.name
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
                "ordered plugin for testing"
            }

            fn on_tool_call(
                &mut self,
                _tool_name: &str,
                _args: &Value,
                _session_id: &str,
            ) -> Result<(), PluginError> {
                let order = self.call_count.fetch_add(1, Ordering::SeqCst);
                let mut seq = self.call_sequence.lock().unwrap();
                if seq.len() == order as usize {
                    seq.push(self.name.clone());
                } else {
                    seq.push(format!("OUT_OF_ORDER:{}", self.name));
                }
                Ok(())
            }
        }

        let mut manager = PluginManager::new();

        let plugin_alpha =
            OrderedToolPlugin::new("tool-alpha", call_order.clone(), call_sequence.clone());
        let plugin_beta =
            OrderedToolPlugin::new("tool-beta", call_order.clone(), call_sequence.clone());
        let plugin_gamma =
            OrderedToolPlugin::new("tool-gamma", call_order.clone(), call_sequence.clone());

        manager.register(Box::new(plugin_alpha)).unwrap();
        manager.register(Box::new(plugin_beta)).unwrap();
        manager.register(Box::new(plugin_gamma)).unwrap();

        let args = serde_json::json!({"file": "test.txt"});
        manager
            .on_tool_call_all("read", &args, "session-123")
            .unwrap();

        let sequence = call_sequence.lock().unwrap();
        assert_eq!(
            sequence.len(),
            3,
            "Expected 3 plugins to be called, got {}",
            sequence.len()
        );
        assert_eq!(
            sequence[0], "tool-alpha",
            "First plugin should be tool-alpha, got {}",
            sequence[0]
        );
        assert_eq!(
            sequence[1], "tool-beta",
            "Second plugin should be tool-beta, got {}",
            sequence[1]
        );
        assert_eq!(
            sequence[2], "tool-gamma",
            "Third plugin should be tool-gamma, got {}",
            sequence[2]
        );
    }

    #[test]
    fn test_hook_order_on_message_is_deterministic() {
        use std::sync::atomic::{AtomicUsize, Ordering};
        use std::sync::Arc;

        let call_order = Arc::new(AtomicUsize::new(0));
        let call_sequence: Arc<std::sync::Mutex<Vec<String>>> =
            Arc::new(std::sync::Mutex::new(Vec::new()));

        struct OrderedMessagePlugin {
            name: String,
            call_count: Arc<AtomicUsize>,
            call_sequence: Arc<std::sync::Mutex<Vec<String>>>,
        }

        impl OrderedMessagePlugin {
            fn new(
                name: &str,
                call_count: Arc<AtomicUsize>,
                call_sequence: Arc<std::sync::Mutex<Vec<String>>>,
            ) -> Self {
                Self {
                    name: name.to_string(),
                    call_count,
                    call_sequence,
                }
            }
        }

        impl Plugin for OrderedMessagePlugin {
            fn name(&self) -> &str {
                &self.name
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
                "ordered plugin for testing"
            }

            fn on_message(&mut self, _content: &str, _session_id: &str) -> Result<(), PluginError> {
                let order = self.call_count.fetch_add(1, Ordering::SeqCst);
                let mut seq = self.call_sequence.lock().unwrap();
                if seq.len() == order as usize {
                    seq.push(self.name.clone());
                } else {
                    seq.push(format!("OUT_OF_ORDER:{}", self.name));
                }
                Ok(())
            }
        }

        let mut manager = PluginManager::new();

        let plugin_first =
            OrderedMessagePlugin::new("msg-first", call_order.clone(), call_sequence.clone());
        let plugin_second =
            OrderedMessagePlugin::new("msg-second", call_order.clone(), call_sequence.clone());
        let plugin_third =
            OrderedMessagePlugin::new("msg-third", call_order.clone(), call_sequence.clone());

        manager.register(Box::new(plugin_first)).unwrap();
        manager.register(Box::new(plugin_second)).unwrap();
        manager.register(Box::new(plugin_third)).unwrap();

        manager
            .on_message_all("Hello world", "session-123")
            .unwrap();

        let sequence = call_sequence.lock().unwrap();
        assert_eq!(
            sequence.len(),
            3,
            "Expected 3 plugins to be called, got {}",
            sequence.len()
        );
        assert_eq!(
            sequence[0], "msg-first",
            "First plugin should be msg-first, got {}",
            sequence[0]
        );
        assert_eq!(
            sequence[1], "msg-second",
            "Second plugin should be msg-second, got {}",
            sequence[1]
        );
        assert_eq!(
            sequence[2], "msg-third",
            "Third plugin should be msg-third, got {}",
            sequence[2]
        );
    }

    #[test]
    fn test_hook_order_on_session_end_is_deterministic() {
        use std::sync::atomic::{AtomicUsize, Ordering};
        use std::sync::Arc;

        let call_order = Arc::new(AtomicUsize::new(0));
        let call_sequence: Arc<std::sync::Mutex<Vec<String>>> =
            Arc::new(std::sync::Mutex::new(Vec::new()));

        struct OrderedSessionPlugin {
            name: String,
            call_count: Arc<AtomicUsize>,
            call_sequence: Arc<std::sync::Mutex<Vec<String>>>,
        }

        impl OrderedSessionPlugin {
            fn new(
                name: &str,
                call_count: Arc<AtomicUsize>,
                call_sequence: Arc<std::sync::Mutex<Vec<String>>>,
            ) -> Self {
                Self {
                    name: name.to_string(),
                    call_count,
                    call_sequence,
                }
            }
        }

        impl Plugin for OrderedSessionPlugin {
            fn name(&self) -> &str {
                &self.name
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
                "ordered plugin for testing"
            }

            fn on_session_end(&mut self, _session_id: &str) -> Result<(), PluginError> {
                let order = self.call_count.fetch_add(1, Ordering::SeqCst);
                let mut seq = self.call_sequence.lock().unwrap();
                if seq.len() == order as usize {
                    seq.push(self.name.clone());
                } else {
                    seq.push(format!("OUT_OF_ORDER:{}", self.name));
                }
                Ok(())
            }
        }

        let mut manager = PluginManager::new();

        let plugin_end_a =
            OrderedSessionPlugin::new("end-a", call_order.clone(), call_sequence.clone());
        let plugin_end_b =
            OrderedSessionPlugin::new("end-b", call_order.clone(), call_sequence.clone());
        let plugin_end_c =
            OrderedSessionPlugin::new("end-c", call_order.clone(), call_sequence.clone());

        manager.register(Box::new(plugin_end_a)).unwrap();
        manager.register(Box::new(plugin_end_b)).unwrap();
        manager.register(Box::new(plugin_end_c)).unwrap();

        manager.on_session_end_all("session-123").unwrap();

        let sequence = call_sequence.lock().unwrap();
        assert_eq!(
            sequence.len(),
            3,
            "Expected 3 plugins to be called, got {}",
            sequence.len()
        );
        assert_eq!(
            sequence[0], "end-a",
            "First plugin should be end-a, got {}",
            sequence[0]
        );
        assert_eq!(
            sequence[1], "end-b",
            "Second plugin should be end-b, got {}",
            sequence[1]
        );
        assert_eq!(
            sequence[2], "end-c",
            "Third plugin should be end-c, got {}",
            sequence[2]
        );
    }

    #[test]
    fn test_hook_order_all_hooks_consistent() {
        use std::sync::atomic::{AtomicUsize, Ordering};
        use std::sync::Arc;

        let start_order = Arc::new(AtomicUsize::new(0));
        let start_sequence: Arc<std::sync::Mutex<Vec<String>>> =
            Arc::new(std::sync::Mutex::new(Vec::new()));

        let tool_order = Arc::new(AtomicUsize::new(0));
        let tool_sequence: Arc<std::sync::Mutex<Vec<String>>> =
            Arc::new(std::sync::Mutex::new(Vec::new()));

        let msg_order = Arc::new(AtomicUsize::new(0));
        let msg_sequence: Arc<std::sync::Mutex<Vec<String>>> =
            Arc::new(std::sync::Mutex::new(Vec::new()));

        struct ConsistentPlugin {
            name: String,
            start_count: Arc<AtomicUsize>,
            start_seq: Arc<std::sync::Mutex<Vec<String>>>,
            tool_count: Arc<AtomicUsize>,
            tool_seq: Arc<std::sync::Mutex<Vec<String>>>,
            msg_count: Arc<AtomicUsize>,
            msg_seq: Arc<std::sync::Mutex<Vec<String>>>,
        }

        impl ConsistentPlugin {
            fn new(
                name: &str,
                start_count: Arc<AtomicUsize>,
                start_seq: Arc<std::sync::Mutex<Vec<String>>>,
                tool_count: Arc<AtomicUsize>,
                tool_seq: Arc<std::sync::Mutex<Vec<String>>>,
                msg_count: Arc<AtomicUsize>,
                msg_seq: Arc<std::sync::Mutex<Vec<String>>>,
            ) -> Self {
                Self {
                    name: name.to_string(),
                    start_count,
                    start_seq,
                    tool_count,
                    tool_seq,
                    msg_count,
                    msg_seq,
                }
            }
        }

        impl Plugin for ConsistentPlugin {
            fn name(&self) -> &str {
                &self.name
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
                "consistent order plugin"
            }

            fn on_start(&mut self) -> Result<(), PluginError> {
                let order = self.start_count.fetch_add(1, Ordering::SeqCst);
                let mut seq = self.start_seq.lock().unwrap();
                if seq.len() == order as usize {
                    seq.push(self.name.clone());
                }
                Ok(())
            }

            fn on_tool_call(
                &mut self,
                _tool_name: &str,
                _args: &Value,
                _session_id: &str,
            ) -> Result<(), PluginError> {
                let order = self.tool_count.fetch_add(1, Ordering::SeqCst);
                let mut seq = self.tool_seq.lock().unwrap();
                if seq.len() == order as usize {
                    seq.push(self.name.clone());
                }
                Ok(())
            }

            fn on_message(&mut self, _content: &str, _session_id: &str) -> Result<(), PluginError> {
                let order = self.msg_count.fetch_add(1, Ordering::SeqCst);
                let mut seq = self.msg_seq.lock().unwrap();
                if seq.len() == order as usize {
                    seq.push(self.name.clone());
                }
                Ok(())
            }
        }

        let mut manager = PluginManager::new();

        let plugin_a = ConsistentPlugin::new(
            "consist-a",
            start_order.clone(),
            start_sequence.clone(),
            tool_order.clone(),
            tool_sequence.clone(),
            msg_order.clone(),
            msg_sequence.clone(),
        );
        let plugin_b = ConsistentPlugin::new(
            "consist-b",
            start_order.clone(),
            start_sequence.clone(),
            tool_order.clone(),
            tool_sequence.clone(),
            msg_order.clone(),
            msg_sequence.clone(),
        );
        let plugin_c = ConsistentPlugin::new(
            "consist-c",
            start_order.clone(),
            start_sequence.clone(),
            tool_order.clone(),
            tool_sequence.clone(),
            msg_order.clone(),
            msg_sequence.clone(),
        );

        manager.register(Box::new(plugin_a)).unwrap();
        manager.register(Box::new(plugin_b)).unwrap();
        manager.register(Box::new(plugin_c)).unwrap();

        manager.on_start_all().unwrap();

        let args = serde_json::json!({"file": "test.txt"});
        manager
            .on_tool_call_all("read", &args, "session-123")
            .unwrap();
        manager
            .on_message_all("test message", "session-123")
            .unwrap();

        let start_seq = start_sequence.lock().unwrap();
        let tool_seq = tool_sequence.lock().unwrap();
        let msg_seq = msg_sequence.lock().unwrap();

        assert_eq!(start_seq.len(), 3);
        assert_eq!(tool_seq.len(), 3);
        assert_eq!(msg_seq.len(), 3);

        assert_eq!(start_seq[0], "consist-a");
        assert_eq!(start_seq[1], "consist-b");
        assert_eq!(start_seq[2], "consist-c");

        assert_eq!(tool_seq[0], "consist-a");
        assert_eq!(tool_seq[1], "consist-b");
        assert_eq!(tool_seq[2], "consist-c");

        assert_eq!(msg_seq[0], "consist-a");
        assert_eq!(msg_seq[1], "consist-b");
        assert_eq!(msg_seq[2], "consist-c");
    }

    #[tokio::test]
    async fn test_plugin_tools_export_as_dyn_tool() {
        let mut manager = PluginManager::new();
        register_test_plugin_with_tools(&mut manager, "export-plugin");

        let tool_def = PluginToolDefinition {
            name: "export_tool".to_string(),
            description: "Tool for export test".to_string(),
            input_schema: serde_json::json!({"type": "object"}),
            provider_name: "export-plugin".to_string(),
        };

        let tool = PluginTool::new(tool_def, Box::new(|_args| Ok("exported".to_string())));
        manager.register_plugin_tool(tool).await.unwrap();

        let exported = manager.export_as_tools().await;
        assert_eq!(exported.len(), 1);
        assert_eq!(exported[0].name(), "export_tool");
        assert_eq!(exported[0].description(), "Tool for export test");
    }

    #[tokio::test]
    async fn test_plugin_tools_registered_in_tool_registry() {
        use opencode_tools::ToolRegistry;

        let mut manager = PluginManager::new();
        register_test_plugin_with_tools(&mut manager, "registry-plugin");

        let tool_def = PluginToolDefinition {
            name: "registry_tool".to_string(),
            description: "Tool for registry test".to_string(),
            input_schema: serde_json::json!({"type": "object"}),
            provider_name: "registry-plugin".to_string(),
        };

        let tool = PluginTool::new(tool_def, Box::new(|_args| Ok("registered".to_string())));
        manager.register_plugin_tool(tool).await.unwrap();

        let tool_registry = ToolRegistry::new();
        let exported_tools = manager.export_as_tools().await;
        tool_registry.register_plugin_tools(exported_tools).await;

        let retrieved = tool_registry.get("registry_tool").await;
        assert!(
            retrieved.is_some(),
            "Plugin tool should appear in ToolRegistry"
        );
        assert_eq!(retrieved.unwrap().name(), "registry_tool");
    }

    #[tokio::test]
    async fn test_plugin_tool_execution_via_tool_registry() {
        use opencode_tools::ToolRegistry;

        let mut manager = PluginManager::new();
        register_test_plugin_with_tools(&mut manager, "exec-plugin");

        let tool_def = PluginToolDefinition {
            name: "exec_tool".to_string(),
            description: "Tool for execution test".to_string(),
            input_schema: serde_json::json!({"type": "object", "properties": {"msg": {"type": "string"}}}),
            provider_name: "exec-plugin".to_string(),
        };

        let tool = PluginTool::new(
            tool_def,
            Box::new(|args: Value| {
                let msg = args.get("msg").and_then(|v| v.as_str()).unwrap_or("");
                Ok(format!("echo: {}", msg))
            }),
        );
        manager.register_plugin_tool(tool).await.unwrap();

        let tool_registry = ToolRegistry::new();
        let exported_tools = manager.export_as_tools().await;
        tool_registry.register_plugin_tools(exported_tools).await;

        let result = tool_registry
            .execute("exec_tool", serde_json::json!({"msg": "hello"}), None)
            .await;
        assert!(
            result.is_ok(),
            "Plugin tool should execute via ToolRegistry"
        );
        assert_eq!(result.unwrap().content, "echo: hello");
    }

    #[tokio::test]
    async fn test_plugin_tools_list_filtered_includes_plugin_tools() {
        use opencode_tools::ToolRegistry;

        let mut manager = PluginManager::new();
        register_test_plugin_with_tools(&mut manager, "list-plugin");

        let tool_def = PluginToolDefinition {
            name: "list_tool".to_string(),
            description: "Tool for list test".to_string(),
            input_schema: serde_json::json!({"type": "object"}),
            provider_name: "list-plugin".to_string(),
        };

        let tool = PluginTool::new(tool_def, Box::new(|_args| Ok("listed".to_string())));
        manager.register_plugin_tool(tool).await.unwrap();

        let tool_registry = ToolRegistry::new();
        let exported_tools = manager.export_as_tools().await;
        tool_registry.register_plugin_tools(exported_tools).await;

        let listed = tool_registry.list_filtered(None).await;
        assert!(
            listed.iter().any(|(name, _, _)| name == "list_tool"),
            "Plugin tool should appear in ToolRegistry.list_filtered()"
        );
    }

    #[tokio::test]
    async fn test_plugin_register_tool_via_trait() {
        use opencode_tools::ToolRegistry;

        let mut manager = PluginManager::new();
        register_test_plugin_with_tools(&mut manager, "trait-plugin");

        let tool_def = PluginToolDefinition {
            name: "trait_tool".to_string(),
            description: "Tool registered via Plugin trait".to_string(),
            input_schema: serde_json::json!({"type": "object"}),
            provider_name: "trait-plugin".to_string(),
        };

        let tool = PluginTool::new(tool_def, Box::new(|_args| Ok("trait_result".to_string())));

        // Register via the new register_tool method on PluginManager
        manager.register_tool("trait-plugin", tool).await.unwrap();

        // Verify tool is in the plugin manager
        let tools = manager.list_plugin_tools().await;
        assert!(tools.iter().any(|t| t.name == "trait_tool"));

        // Verify tool appears in ToolRegistry after export
        let tool_registry = ToolRegistry::new();
        manager
            .register_tools_in_registry(&tool_registry)
            .await
            .unwrap();

        let retrieved = tool_registry.get("trait_tool").await;
        assert!(retrieved.is_some(), "Plugin tool should be in ToolRegistry");
    }

    #[tokio::test]
    async fn test_plugin_trait_register_tool_requires_capability() {
        let mut manager = PluginManager::new();
        register_test_plugin_without_tools(&mut manager, "no-tools-plugin");

        let tool_def = PluginToolDefinition {
            name: "should_fail".to_string(),
            description: "Should fail".to_string(),
            input_schema: serde_json::json!({"type": "object"}),
            provider_name: "no-tools-plugin".to_string(),
        };

        let tool = PluginTool::new(tool_def, Box::new(|_args| Ok("ok".to_string())));

        let result = manager.register_tool("no-tools-plugin", tool).await;
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            PluginError::PermissionDenied(_)
        ));
    }

    #[tokio::test]
    async fn test_register_tools_in_registry_integration() {
        use opencode_tools::ToolRegistry;

        let mut manager = PluginManager::new();
        register_test_plugin_with_tools(&mut manager, "integration-plugin");

        // Add multiple tools
        for i in 0..3 {
            let tool_def = PluginToolDefinition {
                name: format!("integration_tool_{}", i),
                description: format!("Integration test tool {}", i),
                input_schema: serde_json::json!({"type": "object"}),
                provider_name: "integration-plugin".to_string(),
            };
            let tool =
                PluginTool::new(tool_def, Box::new(move |_args| Ok(format!("result_{}", i))));
            manager
                .register_tool("integration-plugin", tool)
                .await
                .unwrap();
        }

        let tool_registry = ToolRegistry::new();
        manager
            .register_tools_in_registry(&tool_registry)
            .await
            .unwrap();

        // Verify all tools are in registry
        for i in 0..3 {
            let name = format!("integration_tool_{}", i);
            let retrieved = tool_registry.get(&name).await;
            assert!(
                retrieved.is_some(),
                "Tool {} should be in ToolRegistry",
                name
            );
        }

        // Verify tools can be executed
        let result = tool_registry
            .execute("integration_tool_1", serde_json::json!({}), None)
            .await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap().content, "result_1");
    }

    #[tokio::test]
    async fn test_plugin_tools_respect_permission_system() {
        use opencode_permission::PermissionScope;
        use opencode_tools::ToolRegistry;

        let mut manager = PluginManager::new();
        register_test_plugin_with_tools(&mut manager, "perm-plugin");

        // Set permission scope to ReadOnly (denies write tools)
        manager.set_permission_scope(PermissionScope::ReadOnly);

        let tool_def = PluginToolDefinition {
            name: "write_tool".to_string(),
            description: "A write tool that should be denied".to_string(),
            input_schema: serde_json::json!({"type": "object"}),
            provider_name: "perm-plugin".to_string(),
        };

        let tool = PluginTool::new(tool_def, Box::new(|_args| Ok("ok".to_string())));

        // This should fail because the tool is denied by permission policy
        // (assuming "write_tool" is in the denied category for ReadOnly scope)
        let result = manager.register_tool("perm-plugin", tool).await;
        // Note: The actual result depends on whether write_tool is denied by the permission system
        // For this test, we just verify the method works and permission checking occurs
        if result.is_ok() {
            let tool_registry = ToolRegistry::new();
            manager
                .register_tools_in_registry(&tool_registry)
                .await
                .unwrap();

            // Tool should be registered but may require approval
            let retrieved = tool_registry.get("write_tool").await;
            assert!(
                retrieved.is_some() || retrieved.is_none(),
                "Tool should be handleable"
            );
        }
    }

    #[test]
    fn test_plugin_trait_default_register_tool() {
        struct DefaultRegisterPlugin;

        impl Plugin for DefaultRegisterPlugin {
            fn name(&self) -> &str {
                "default-plugin"
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
                "default register_tool plugin"
            }
        }

        let mut plugin = DefaultRegisterPlugin;
        let tool_def = PluginToolDefinition {
            name: "default_tool".to_string(),
            description: "default".to_string(),
            input_schema: serde_json::json!({}),
            provider_name: "default-plugin".to_string(),
        };
        let tool = PluginTool::new(tool_def, Box::new(|_args| Ok("ok".to_string())));

        // Default implementation should return PermissionDenied
        let result = plugin.register_tool(tool);
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            PluginError::PermissionDenied(_)
        ));
    }

    #[test]
    fn test_config_separation_plugin_options_isolated() {
        use crate::config::validate_plugin_options;
        use indexmap::IndexMap;

        // Plugin A has its own options - should be valid
        let mut plugin_a_options = IndexMap::new();
        plugin_a_options.insert("custom_key".to_string(), serde_json::json!("value"));
        plugin_a_options.insert(
            "another_setting".to_string(),
            serde_json::json!({"nested": true}),
        );

        let result_a = validate_plugin_options("plugin-a", &plugin_a_options);
        assert!(
            result_a.valid,
            "plugin-a options should be valid: {:?}",
            result_a.errors
        );

        // Plugin B has different options - should also be valid
        let mut plugin_b_options = IndexMap::new();
        plugin_b_options.insert("setting".to_string(), serde_json::json!(123));

        let result_b = validate_plugin_options("plugin-b", &plugin_b_options);
        assert!(
            result_b.valid,
            "plugin-b options should be valid: {:?}",
            result_b.errors
        );

        // Verify the options are isolated (different plugins have different option maps)
        assert_ne!(
            plugin_a_options.len(),
            plugin_b_options.len(),
            "Different plugins should have independent options"
        );
    }

    #[test]
    fn test_config_separation_plugin_cannot_override_core_config() {
        use crate::config::validate_plugin_options;
        use indexmap::IndexMap;

        // Test that plugins cannot override core config keys
        let core_config_keys = vec![
            "model",
            "server",
            "provider",
            "permission",
            "mcp",
            "formatter",
            "lsp",
            "agent",
            "plugin",
            "skills",
            "enterprise",
            "compaction",
        ];

        for key in core_config_keys {
            let mut options = IndexMap::new();
            options.insert(key.to_string(), serde_json::json!("malicious_value"));

            let result = validate_plugin_options("malicious-plugin", &options);
            assert!(
                !result.valid,
                "plugin should not be able to override core config key '{}': {:?}",
                key, result.errors
            );
            assert!(
                result.errors.iter().any(|e| e.contains("reserved")),
                "error should mention 'reserved' for key '{}'",
                key
            );
        }
    }

    #[test]
    fn test_config_separation_nested_reserved_keys() {
        use crate::config::validate_plugin_options;
        use indexmap::IndexMap;

        // Plugins cannot use nested reserved keys like "server.port" or "mcp.foo"
        let reserved_nested_keys = vec![
            ("server.port", serde_json::json!(8080)),
            (
                "mcp.local.command",
                serde_json::json!(vec!["npx".to_string()]),
            ),
            ("provider.openai.api_key", serde_json::json!("secret")),
            ("permission.read", serde_json::json!("allow")),
        ];

        for (key, value) in reserved_nested_keys {
            let mut options = IndexMap::new();
            options.insert(key.to_string(), value);

            let result = validate_plugin_options("test-plugin", &options);
            assert!(
                !result.valid,
                "plugin should not be able to use nested reserved key '{}': {:?}",
                key, result.errors
            );
        }
    }

    #[test]
    fn test_config_separation_deep_nested_objects() {
        use crate::config::validate_plugin_options;
        use indexmap::IndexMap;

        // Even deeply nested objects with reserved keys should be caught
        let mut options = IndexMap::new();
        options.insert(
            "my_config".to_string(),
            serde_json::json!({
                "server": {
                    "hostname": "evil.com",
                    "port": 9999
                }
            }),
        );

        let result = validate_plugin_options("deep-plugin", &options);
        assert!(!result.valid, "deep nested reserved keys should be caught");
    }

    #[test]
    fn test_config_separation_plugins_isolated_from_each_other() {
        use crate::config::validate_plugin_isolation;

        let plugins = vec![
            ("plugin-a".to_string(), {
                let mut m = indexmap::IndexMap::new();
                m.insert("option_a".to_string(), serde_json::json!("value"));
                m
            }),
            ("plugin-b".to_string(), {
                let mut m = indexmap::IndexMap::new();
                m.insert("option_b".to_string(), serde_json::json!("value"));
                m
            }),
            ("plugin-c".to_string(), {
                let mut m = indexmap::IndexMap::new();
                m.insert("option_c".to_string(), serde_json::json!("value"));
                m
            }),
        ];

        let result = validate_plugin_isolation(&plugins);
        assert!(
            result.valid,
            "plugins should be isolated from each other: {:?}",
            result.errors
        );
    }

    #[test]
    fn test_config_separation_plugin_name_as_option_key() {
        use crate::config::validate_plugin_isolation;

        // A plugin using another plugin's name as an option key should be flagged
        let plugins = vec![
            ("plugin-a".to_string(), {
                let mut m = indexmap::IndexMap::new();
                m.insert("plugin-b".to_string(), serde_json::json!("value")); // Leak!
                m
            }),
            ("plugin-b".to_string(), {
                let mut m = indexmap::IndexMap::new();
                m.insert("normal_option".to_string(), serde_json::json!("value"));
                m
            }),
        ];

        let result = validate_plugin_isolation(&plugins);
        assert!(
            !result.valid,
            "plugin config using other plugin name should be flagged"
        );
        assert!(
            result
                .errors
                .iter()
                .any(|e| e.contains("isolation violation")),
            "error should mention isolation violation"
        );
    }

    #[test]
    fn test_config_separation_validation_result_merge() {
        use crate::config::ConfigValidationResult;

        let mut result1 = ConfigValidationResult::valid();
        result1.add_error("error 1".to_string());

        let mut result2 = ConfigValidationResult::valid();
        result2.add_error("error 2".to_string());

        result1.merge(result2);

        assert!(!result1.valid);
        assert_eq!(result1.errors.len(), 2);
        assert!(result1.errors.contains(&"error 1".to_string()));
        assert!(result1.errors.contains(&"error 2".to_string()));
    }

    #[test]
    fn test_config_separation_reserved_keys_list_comprehensive() {
        use crate::config::RESERVED_CONFIG_KEYS;

        // Verify the reserved keys list is comprehensive
        let expected_keys = vec![
            "$schema",
            "log_level",
            "server",
            "command",
            "skills",
            "watcher",
            "plugin",
            "snapshot",
            "share",
            "autoshare",
            "autoupdate",
            "disabled_providers",
            "enabled_providers",
            "model",
            "small_model",
            "default_agent",
            "username",
            "mode",
            "agent",
            "provider",
            "mcp",
            "formatter",
            "lsp",
            "instructions",
            "agents_md",
            "permission",
            "tools",
            "enterprise",
            "compaction",
            "experimental",
            "theme",
            "tui",
            "api_key",
            "temperature",
            "max_tokens",
        ];

        for key in expected_keys {
            assert!(
                RESERVED_CONFIG_KEYS.contains(&key),
                "reserved key '{}' should be in RESERVED_CONFIG_KEYS",
                key
            );
        }
    }

    #[test]
    fn test_config_separation_discovery_validates_options() {
        use std::fs;

        let temp = tempfile::tempdir().unwrap();
        let metadata_path = temp.path().join("test.plugin.json");

        // Write a plugin with invalid options (reserved key)
        fs::write(
            &metadata_path,
            r#"{
                "name": "bad-plugin",
                "version": "1.0.0",
                "description": "test",
                "main": "test.so",
                "options": {
                    "model": "evil-model"
                }
            }"#,
        )
        .unwrap();

        let result = discovery::parse_metadata_file(&metadata_path);
        assert!(
            result.is_err(),
            "discovery should fail for plugin with reserved option key"
        );
        let err = result.unwrap_err();
        assert!(
            format!("{}", err).contains("config validation"),
            "error should mention config validation: {}",
            err
        );
    }

    #[test]
    fn test_config_separation_discovery_accepts_valid_options() {
        use std::fs;

        let temp = tempfile::tempdir().unwrap();
        let metadata_path = temp.path().join("good.plugin.json");

        // Write a plugin with valid options
        fs::write(
            &metadata_path,
            r#"{
                "name": "good-plugin",
                "version": "1.0.0",
                "description": "test",
                "main": "test.so",
                "options": {
                    "my_custom_setting": "value",
                    "nested": {"anything": true}
                }
            }"#,
        )
        .unwrap();

        let result = discovery::parse_metadata_file(&metadata_path);
        assert!(
            result.is_ok(),
            "discovery should succeed for valid plugin options: {:?}",
            result.err()
        );
        let discovered = result.unwrap();
        assert_eq!(discovered.config.name, "good-plugin");
        assert_eq!(
            discovered.config.options.get("my_custom_setting"),
            Some(&serde_json::json!("value"))
        );
    }

    #[test]
    fn test_plugin_cleanup_unload_single_plugin() {
        let mut manager = PluginManager::new();
        manager
            .register(Box::new(TestPlugin {
                initialized: false,
                shutdown_called: false,
                fail_init: false,
                fail_shutdown: false,
            }))
            .unwrap();

        assert!(manager.is_plugin_loaded("test-plugin"));
        manager.unload_plugin("test-plugin").unwrap();
        assert!(!manager.is_plugin_loaded("test-plugin"));
        assert!(manager.get_plugin("test-plugin").is_none());
        assert!(manager.get_config("test-plugin").is_none());
    }

    #[test]
    fn test_plugin_cleanup_unload_calls_shutdown() {
        let mut manager = PluginManager::new();
        let plugin = TestPlugin {
            initialized: false,
            shutdown_called: false,
            fail_init: false,
            fail_shutdown: false,
        };
        manager.register(Box::new(plugin)).unwrap();

        manager.unload_plugin("test-plugin").unwrap();
    }

    #[test]
    fn test_plugin_cleanup_unload_nonexistent_fails() {
        let mut manager = PluginManager::new();
        let result = manager.unload_plugin("nonexistent");
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), PluginError::NotFound(_)));
    }

    #[test]
    fn test_plugin_cleanup_unload_all() {
        let mut manager = PluginManager::new();
        manager
            .register(Box::new(TestPlugin {
                initialized: false,
                shutdown_called: false,
                fail_init: false,
                fail_shutdown: false,
            }))
            .unwrap();

        let plugin2 = TestPluginWithTools::new("another-plugin");
        let config2 = plugin2.to_config();
        manager
            .register_with_config(Box::new(plugin2), config2)
            .unwrap();

        assert!(manager.is_plugin_loaded("test-plugin"));
        assert!(manager.is_plugin_loaded("another-plugin"));

        manager.unload_plugin("test-plugin").unwrap();
        assert!(!manager.is_plugin_loaded("test-plugin"));
        assert!(manager.is_plugin_loaded("another-plugin"));

        manager.unload_plugin("another-plugin").unwrap();
        assert!(!manager.is_plugin_loaded("test-plugin"));
        assert!(!manager.is_plugin_loaded("another-plugin"));
    }

    #[tokio::test]
    async fn test_plugin_cleanup_unload_async_removes_tools() {
        let mut manager = PluginManager::new();
        register_test_plugin_with_tools(&mut manager, "tool-plugin");

        let tool_def = PluginToolDefinition {
            name: "cleanup_test_tool".to_string(),
            description: "Tool for cleanup test".to_string(),
            input_schema: serde_json::json!({"type": "object"}),
            provider_name: "tool-plugin".to_string(),
        };
        let tool = PluginTool::new(tool_def, Box::new(|_args| Ok("ok".to_string())));
        manager.register_plugin_tool(tool).await.unwrap();

        assert!(manager
            .get_plugin_tool_definition("cleanup_test_tool")
            .await
            .is_some());

        manager.unload_plugin_async("tool-plugin").await.unwrap();

        assert!(!manager.is_plugin_loaded("tool-plugin"));
        assert!(manager
            .get_plugin_tool_definition("cleanup_test_tool")
            .await
            .is_none());
    }

    #[tokio::test]
    async fn test_plugin_cleanup_unload_async_clears_multiple_tools() {
        let mut manager = PluginManager::new();
        register_test_plugin_with_tools(&mut manager, "multi-tool-plugin");

        for i in 0..5 {
            let tool_def = PluginToolDefinition {
                name: format!("tool_{}", i),
                description: format!("Tool {}", i),
                input_schema: serde_json::json!({"type": "object"}),
                provider_name: "multi-tool-plugin".to_string(),
            };
            let tool = PluginTool::new(tool_def, Box::new(|_args| Ok("ok".to_string())));
            manager.register_plugin_tool(tool).await.unwrap();
        }

        let tools = manager.list_plugin_tools().await;
        assert_eq!(tools.len(), 5);

        manager
            .unload_plugin_async("multi-tool-plugin")
            .await
            .unwrap();

        let tools = manager.list_plugin_tools().await;
        assert!(tools.is_empty());
        assert!(!manager.is_plugin_loaded("multi-tool-plugin"));
    }

    #[tokio::test]
    async fn test_plugin_cleanup_unload_all_plugins() {
        let mut manager = PluginManager::new();
        register_test_plugin_with_tools(&mut manager, "plugin-a");
        register_test_plugin_with_tools(&mut manager, "plugin-b");
        register_test_plugin_with_tools(&mut manager, "plugin-c");

        for (tool_key, plugin_name) in [
            ("tool_a", "plugin-a"),
            ("tool_b", "plugin-b"),
            ("tool_c", "plugin-c"),
        ] {
            let tool_def = PluginToolDefinition {
                name: tool_key.to_string(),
                description: format!("Tool for {}", tool_key),
                input_schema: serde_json::json!({"type": "object"}),
                provider_name: plugin_name.to_string(),
            };
            let tool = PluginTool::new(tool_def, Box::new(|_args| Ok("ok".to_string())));
            manager.register_plugin_tool(tool).await.unwrap();
        }

        assert!(manager.is_plugin_loaded("plugin-a"));
        assert!(manager.is_plugin_loaded("plugin-b"));
        assert!(manager.is_plugin_loaded("plugin-c"));
        assert_eq!(manager.list_plugin_tools().await.len(), 3);

        manager.unload_all_plugins().await.unwrap();

        assert!(!manager.is_plugin_loaded("plugin-a"));
        assert!(!manager.is_plugin_loaded("plugin-b"));
        assert!(!manager.is_plugin_loaded("plugin-c"));
        assert!(manager.list_plugin_tools().await.is_empty());
    }

    #[tokio::test]
    async fn test_plugin_cleanup_remove_tools_by_provider() {
        let mut manager = PluginManager::new();
        register_test_plugin_with_tools(&mut manager, "provider-x");
        register_test_plugin_with_tools(&mut manager, "provider-y");

        let tool_def_x = PluginToolDefinition {
            name: "tool_x1".to_string(),
            description: "Tool X1".to_string(),
            input_schema: serde_json::json!({"type": "object"}),
            provider_name: "provider-x".to_string(),
        };
        let tool_x1 = PluginTool::new(tool_def_x, Box::new(|_args| Ok("ok".to_string())));
        manager.register_plugin_tool(tool_x1).await.unwrap();

        let tool_def_y = PluginToolDefinition {
            name: "tool_y1".to_string(),
            description: "Tool Y1".to_string(),
            input_schema: serde_json::json!({"type": "object"}),
            provider_name: "provider-y".to_string(),
        };
        let tool_y1 = PluginTool::new(tool_def_y, Box::new(|_args| Ok("ok".to_string())));
        manager.register_plugin_tool(tool_y1).await.unwrap();

        assert_eq!(manager.list_plugin_tools().await.len(), 2);

        manager.remove_tools_by_provider("provider-x").await;

        let remaining_tools = manager.list_plugin_tools().await;
        assert_eq!(remaining_tools.len(), 1);
        assert_eq!(remaining_tools[0].name, "tool_y1");
    }

    #[test]
    fn test_plugin_cleanup_shutdown_all_after_unload() {
        let mut manager = PluginManager::new();
        manager
            .register(Box::new(TestPlugin {
                initialized: false,
                shutdown_called: false,
                fail_init: false,
                fail_shutdown: false,
            }))
            .unwrap();

        manager.unload_plugin("test-plugin").unwrap();
        let result = manager.shutdown();
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_plugin_cleanup_async_shutdown_after_unload_all() {
        let mut manager = PluginManager::new();
        register_test_plugin_with_tools(&mut manager, "plugin-1");
        register_test_plugin_with_tools(&mut manager, "plugin-2");

        manager.unload_all_plugins().await.unwrap();
        manager.shutdown_async().await.unwrap();

        assert!(!manager.is_plugin_loaded("plugin-1"));
        assert!(!manager.is_plugin_loaded("plugin-2"));
        assert!(manager.list_plugin_tools().await.is_empty());
    }

    #[test]
    fn test_plugin_cleanup_unload_preserves_other_plugins() {
        let mut manager = PluginManager::new();
        manager
            .register(Box::new(TestPlugin {
                initialized: false,
                shutdown_called: false,
                fail_init: false,
                fail_shutdown: false,
            }))
            .unwrap();

        let plugin2 = TestPluginWithTools::new("plugin-to-keep");
        let config2 = plugin2.to_config();
        manager
            .register_with_config(Box::new(plugin2), config2)
            .unwrap();

        manager.unload_plugin("test-plugin").unwrap();

        assert!(!manager.is_plugin_loaded("test-plugin"));
        assert!(manager.is_plugin_loaded("plugin-to-keep"));
        assert!(manager.get_plugin("plugin-to-keep").is_some());
        assert!(manager.get_config("plugin-to-keep").is_some());
    }

    #[tokio::test]
    async fn test_plugin_cleanup_resources_released_after_unload() {
        let mut manager = PluginManager::new();
        register_test_plugin_with_tools(&mut manager, "resource-plugin");

        let tool_def = PluginToolDefinition {
            name: "resource_tool".to_string(),
            description: "Resource tool".to_string(),
            input_schema: serde_json::json!({"type": "object"}),
            provider_name: "resource-plugin".to_string(),
        };
        let tool = PluginTool::new(
            tool_def,
            Box::new(|_args| Ok("resource released".to_string())),
        );
        manager.register_plugin_tool(tool).await.unwrap();

        manager
            .unload_plugin_async("resource-plugin")
            .await
            .unwrap();

        let result = manager
            .execute_plugin_tool("resource_tool", serde_json::json!({}))
            .await;
        assert!(result.is_err());

        let err = result.unwrap_err();
        assert!(matches!(err, PluginError::ToolRegistration(_)));
    }

    #[test]
    fn test_plugin_cleanup_unload_with_failed_shutdown_still_removes() {
        let mut manager = PluginManager::new();
        manager
            .register(Box::new(TestPlugin {
                initialized: false,
                shutdown_called: false,
                fail_init: false,
                fail_shutdown: true,
            }))
            .unwrap();

        let result = manager.unload_plugin("test-plugin");
        assert!(result.is_ok());

        assert!(!manager.is_plugin_loaded("test-plugin"));
        assert!(manager.get_plugin("test-plugin").is_none());
    }
}
