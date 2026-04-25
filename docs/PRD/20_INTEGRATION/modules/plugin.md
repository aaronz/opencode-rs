# plugin.md — Plugin Module

## Module Overview

- **Crate**: `opencode-plugin`
- **Source**: `crates/plugin/src/lib.rs`
- **Status**: Fully implemented — PRD reflects actual Rust API
- **Purpose**: WASM-based plugin system with lifecycle management, tool registration, permission enforcement, and ABI versioning.

---

## Crate Layout

```
crates/plugin/src/
├── lib.rs              ← Plugin, PluginManager, PluginTool, PluginToolAdapter, all types
├── config.rs           ← PluginConfig validation
├── discovery.rs        ← PluginDiscovery, DiscoveredPlugin
├── loader.rs           ← PluginLoader (WASM loading)
├── registry.rs         ← PluginRegistry
└── wasm_runtime.rs    ← WASM runtime execution
```

**Key Cargo.toml dependencies**:
```toml
[dependencies]
async-trait = "0.1"
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
tokio = { version = "1.45", features = ["full"] }
thiserror = "2.0"
indexmap = "2"
wasmtime = "20"

opencode-permission = { path = "../permission" }
opencode-tools = { path = "../tools" }
opencode-core = { path = "../core" }
```

---

## Core Types

### PluginAbiVersion

```rust
pub const PLUGIN_ABI_VERSION: PluginAbiVersion = PluginAbiVersion {
    major: 1,
    minor: 0,
    patch: 0,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct PluginAbiVersion {
    pub major: u32,
    pub minor: u32,
    pub patch: u32,
}

impl PluginAbiVersion {
    pub fn new(major: u32, minor: u32, patch: u32) -> Self;
    pub fn from_string(s: &str) -> Result<Self, PluginAbiVersionError>;
    pub fn is_compatible_with(&self, other: &PluginAbiVersion) -> bool;  // major must match
    pub fn supports_abi(&self, min_abi: &PluginAbiVersion) -> bool;  // self >= min_abi
}

#[derive(Debug, thiserror::Error)]
pub enum PluginAbiVersionError {
    #[error("invalid version format: '{0}', expected major.minor.patch")]
    InvalidFormat(String),
    #[error("version component out of range: {0}")]
    OutOfRange(#[from] ParseIntError),
}
```

### PluginCapability

```rust
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
    pub fn has_capability(&self, cap: &PluginCapability) -> bool;
    pub fn can_add_tools(&self) -> bool;
}
```

### PluginDomain

```rust
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Default)]
#[serde(rename_all = "lowercase")]
pub enum PluginDomain {
    #[default]
    Runtime,
    Tui,
}
```

### PluginConfig

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginConfig {
    pub name: String,
    pub version: String,
    pub enabled: bool,
    pub priority: i32,
    pub domain: PluginDomain,
    pub options: IndexMap<String, Value>,
    pub permissions: PluginPermissions,
}
```

### PluginTool and PluginToolDefinition

```rust
#[derive(Debug, Clone)]
pub struct PluginToolDefinition {
    pub name: String,
    pub description: String,
    pub input_schema: Value,
    pub provider_name: String,
}

pub struct PluginTool {
    definition: PluginToolDefinition,
    executor: Arc<Box<dyn Fn(Value) -> Result<String, String> + Send + Sync>>,
}

impl PluginTool {
    pub fn new(
        definition: PluginToolDefinition,
        executor: Box<dyn Fn(Value) -> Result<String, String> + Send + Sync>,
    ) -> Self;
    pub fn definition(&self) -> &PluginToolDefinition;
    pub fn execute(&self, args: Value) -> Result<String, String>;
}
```

### PluginToolAdapter

```rust
// Adapts PluginTool to opencode_tools::Tool trait
pub struct PluginToolAdapter { ... }

impl PluginToolAdapter {
    pub fn new(
        definition: PluginToolDefinition,
        executor: Arc<Box<dyn Fn(Value) -> Result<String, String> + Send + Sync>>,
    ) -> Self;
    pub fn from_plugin_tool(tool: PluginTool) -> Self;
}

impl opencode_tools::Tool for PluginToolAdapter {
    fn name(&self) -> &str;
    fn description(&self) -> &str;
    fn clone_tool(&self) -> Box<dyn opencode_tools::Tool>;
    async fn execute(&self, args: serde_json::Value, ctx: Option<ToolContext>) -> Result<ToolResult, OpenCodeError>;
}
```

### ToolProvider Trait

```rust
#[async_trait]
pub trait ToolProvider: Send + Sync + sealed::SealedToolProvider {
    async fn get_tools(&self) -> Vec<PluginToolDefinition>;
}
```

### Plugin Trait

```rust
pub trait Plugin: Send + Sync + sealed::SealedPlugin {
    fn name(&self) -> &str;
    fn version(&self) -> &str;
    fn init(&mut self) -> Result<(), PluginError>;
    fn shutdown(&mut self) -> Result<(), PluginError>;
    fn description(&self) -> &str;

    // Lifecycle hooks (all have default implementations)
    fn on_init(&mut self) -> Result<(), PluginError> { Ok(()) }
    fn on_start(&mut self) -> Result<(), PluginError> { Ok(()) }
    fn on_tool_call(&mut self, tool_name: &str, args: &Value, session_id: &str) -> Result<(), PluginError> { Ok(()) }
    fn on_message(&mut self, content: &str, session_id: &str) -> Result<(), PluginError> { Ok(()) }
    fn on_session_end(&mut self, session_id: &str) -> Result<(), PluginError> { Ok(()) }
    fn register_tool(&mut self, tool: PluginTool) -> Result<(), PluginError> {
        Err(PluginError::PermissionDenied(format!("plugin '{}' does not implement register_tool", self.name())))
    }
}
```

### PluginError

```rust
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
    #[error("plugin ABI version mismatch: plugin requires {plugin_abi}, runtime supports {runtime_abi}")]
    AbiVersionMismatch { plugin_abi: PluginAbiVersion, runtime_abi: PluginAbiVersion },
}
```

### PluginManager

```rust
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
    pub fn new() -> Self;

    // Registration
    pub fn register(&mut self, plugin: Box<dyn Plugin>) -> Result<(), PluginError>;
    pub fn register_with_config(&mut self, plugin: Box<dyn Plugin>, config: PluginConfig) -> Result<(), PluginError>;

    // Discovery
    pub fn discover_default_dirs(&mut self) -> Result<(), PluginError>;
    pub fn discover_from_dirs(&mut self, paths: &[PathBuf]) -> Result<(), PluginError>;
    pub fn discover_and_load(&mut self, project_path: Option<&Path>) -> Result<Vec<String>, PluginError>;

    // Lifecycle
    pub fn startup(&mut self) -> Result<(), PluginError>;  // calls init() on all plugins
    pub fn shutdown(&mut self) -> Result<(), PluginError>;  // calls shutdown() on all plugins
    pub fn init_all(&mut self) -> Result<(), PluginError> { self.startup() }
    pub fn shutdown_all(&mut self) -> Result<(), PluginError> { self.shutdown() }

    // Async shutdown
    pub async fn shutdown_async(&mut self) -> Result<(), PluginError>;
    pub async fn unload_plugin_async(&mut self, name: &str) -> Result<(), PluginError>;
    pub async fn unload_all_plugins(&mut self) -> Result<(), PluginError>;

    // Sync unload
    pub fn unload_plugin(&mut self, name: &str) -> Result<(), PluginError>;

    // Hooks
    pub fn on_start_all(&mut self) -> Result<(), PluginError>;
    pub fn on_tool_call_all(&mut self, tool_name: &str, args: &Value, session_id: &str) -> Result<(), PluginError>;
    pub fn on_message_all(&mut self, content: &str, session_id: &str) -> Result<(), PluginError>;
    pub fn on_session_end_all(&mut self, session_id: &str) -> Result<(), PluginError>;

    // Accessors
    pub fn get_plugin(&self, name: &str) -> Option<&dyn Plugin>;
    pub fn get_config(&self, name: &str) -> Option<&PluginConfig>;
    pub fn discovered_metadata(&self) -> &[PathBuf];
    pub fn sorted_plugin_names(&self) -> Vec<String>;  // by priority ascending
    pub fn is_plugin_loaded(&self, name: &str) -> bool;

    // Permissions
    pub fn set_permission_scope(&mut self, scope: PermissionScope);
    pub fn permission_scope(&self) -> PermissionScope;

    // Tool management
    pub async fn register_plugin_tool(&self, tool: PluginTool) -> Result<(), PluginError>;
    pub async fn register_tool(&self, plugin_name: &str, tool: PluginTool) -> Result<(), PluginError>;
    pub async fn unregister_plugin_tool(&self, name: &str) -> Result<(), PluginError>;
    pub async fn get_plugin_tool_definition(&self, name: &str) -> Option<PluginToolDefinition>;
    pub async fn list_plugin_tools(&self) -> Vec<PluginToolDefinition>;
    pub async fn execute_plugin_tool(&self, name: &str, args: Value) -> Result<String, PluginError>;
    pub async fn export_as_tools(&self) -> Vec<Box<dyn opencode_tools::Tool>>;
    pub async fn register_tools_in_registry(&self, registry: &opencode_tools::ToolRegistry) -> Result<(), PluginError>;
    pub async fn remove_tools_by_provider(&self, provider_name: &str);
}
```

---

## Inter-Crate Dependencies

| Dependant Crate | What it uses from `opencode-plugin` |
|---|---|
| `opencode-server` | `PluginManager` for plugin lifecycle |
| `opencode-tui` | `PluginDomain::Tui` plugins for UI extensions |
| `opencode-tools` | `PluginToolAdapter` implements `Tool` trait |
| `opencode-config` | `PluginConfig` loaded from config |

**Dependencies of `opencode-plugin`**:
| Crate | Usage |
|---|---|
| `opencode-core` | `OpenCodeError` |
| `opencode-tools` | `Tool`, `ToolContext`, `ToolResult`, `ToolRegistry` |
| `opencode-permission` | `ApprovalResult`, `PermissionScope` |

---

## Test Design

```rust
#[cfg(test)]
mod tests {
    // Plugin ABI version
    #[test]
    fn test_abi_version_compatible_same_major() {
        let v1 = PluginAbiVersion::new(1, 2, 0);
        let v2 = PluginAbiVersion::new(1, 3, 0);
        assert!(v1.is_compatible_with(&v2));
    }

    #[test]
    fn test_abi_version_incompatible_different_major() {
        let v1 = PluginAbiVersion::new(1, 0, 0);
        let v2 = PluginAbiVersion::new(2, 0, 0);
        assert!(!v1.is_compatible_with(&v2));
    }

    #[test]
    fn test_abi_version_supports_abi() {
        let runtime = PluginAbiVersion::new(1, 3, 0);
        let min_required = PluginAbiVersion::new(1, 2, 0);
        assert!(runtime.supports_abi(&min_required));
    }

    // PluginManager registration
    #[test]
    fn test_register_and_get_plugin() { ... }
    #[test]
    fn test_duplicate_plugin_registration_fails() { ... }
    #[test]
    fn test_get_config_returns_plugin_config() { ... }

    // Lifecycle
    #[test]
    fn test_startup_non_fatal_when_plugin_init_fails() { ... }
    #[test]
    fn test_shutdown_clears_plugins() { ... }
    #[test]
    fn test_shutdown_reports_failures() { ... }
    #[test]
    fn test_init_all_and_shutdown_all() { ... }

    // Hooks
    #[test]
    fn test_on_start_all() { ... }
    #[test]
    fn test_on_tool_call_all_blocks_on_error() { ... }
    #[test]
    fn test_on_message_all_logs_errors() { ... }
    #[test]
    fn test_on_session_end_all() { ... }

    // Permissions
    #[test]
    fn test_plugin_permissions_can_add_tools() {
        let perms = PluginPermissions {
            capabilities: vec![PluginCapability::AddTools],
            ..Default::default()
        };
        assert!(perms.can_add_tools());
    }

    // Tool registration
    #[tokio::test]
    async fn test_register_plugin_tool() { ... }
    #[tokio::test]
    async fn test_plugin_tool_adapter_executes() { ... }
    #[tokio::test]
    async fn test_export_as_tools() { ... }

    // Priority ordering
    #[test]
    fn test_sorted_plugin_names_by_priority() { ... }
}
```

---

## Usage Example

```rust
use opencode_plugin::{
    Plugin, PluginManager, PluginConfig, PluginPermissions,
    PluginCapability, PluginTool, PluginToolDefinition,
};

struct MyPlugin;

impl Plugin for MyPlugin {
    fn name(&self) -> &str { "my-plugin" }
    fn version(&self) -> &str { "1.0.0" }
    fn init(&mut self) -> Result<(), PluginError> { Ok(()) }
    fn shutdown(&mut self) -> Result<(), PluginError> { Ok(()) }
    fn description(&self) -> &str { "My custom plugin" }
}

fn main() -> Result<(), PluginError> {
    let mut manager = PluginManager::new();
    
    // Register plugin
    manager.register(Box::new(MyPlugin))?;
    
    // Start all plugins
    manager.startup()?;
    
    // Run hooks
    manager.on_start_all()?;
    manager.on_tool_call_all("read", &serde_json::json!({}), "session-1")?;
    manager.on_message_all("Hello", "session-1")?;
    manager.on_session_end_all("session-1")?;
    
    // Shutdown
    manager.shutdown()?;
    
    Ok(())
}
```
