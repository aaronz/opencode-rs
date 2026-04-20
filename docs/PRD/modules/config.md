# config.md — Configuration Module

## Module Overview

- **Crate**: `opencode-config`
- **Source**: `crates/config/src/lib.rs`
- **Status**: Fully implemented —PRD reflects actual Rust API
- **Purpose**: Centralized configuration management for the entire application. Loads config from JSON/JSONC files, environment variables, remote config servers, keychain secrets, and file variable expansion.

---

## Crate Layout

```
crates/config/src/
├── lib.rs              ← Main entry, Config struct, all config types, load/merge logic
├── directory_scanner.rs
├── jsonc.rs
├── merge.rs
├── remote_cache.rs
├── schema.rs
└── secret_storage.rs
```

**Key Cargo.toml dependencies**:
```toml
[dependencies]
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
thiserror = "2.0"
chrono = { version = "0.4", features = ["serde"] }
reqwest = { version = "0.12" }
sha2 = "0.10"
dirs = "5"
directories = "5"
```

**Public exports from lib.rs**:
```rust
pub use directory_scanner::{load_opencode_directory, DirectoryScanner, OpencodeDirectoryScan, ToolInfo};
pub use jsonc::{is_jsonc_extension, parse_jsonc, JsoncError};
pub use secret_storage::resolve_keychain_secret;
// Plus all config structs/enums directly in lib.rs
```

---

## Core Types

### Config (main struct)

```rust
#[derive(Debug, Clone, Deserialize, Serialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct Config {
    pub schema: Option<String>,
    pub log_level: Option<LogLevel>,
    pub server: Option<ServerConfig>,
    pub command: Option<HashMap<String, CommandConfig>>,
    pub skills: Option<SkillsConfig>,
    pub watcher: Option<WatcherConfig>,
    pub plugin: Option<Vec<String>>,
    pub snapshot: Option<bool>,
    pub share: Option<ShareMode>,
    pub autoshare: Option<bool>,
    pub autoupdate: Option<AutoUpdate>,
    pub disabled_providers: Option<Vec<String>>,
    pub enabled_providers: Option<Vec<String>>,
    pub model: Option<String>,
    pub small_model: Option<String>,
    pub default_agent: Option<String>,
    pub username: Option<String>,
    pub agent: Option<AgentMapConfig>,
    pub provider: Option<HashMap<String, ProviderConfig>>,
    pub mcp: Option<HashMap<String, McpConfig>>,
    pub formatter: Option<FormatterConfig>,
    pub lsp: Option<LspConfig>,
    pub instructions: Option<Vec<String>>,
    pub agents_md: Option<AgentsMdConfig>,
    pub permission: Option<PermissionConfig>,
    pub enterprise: Option<EnterpriseConfig>,
    pub compaction: Option<CompactionConfig>,
    pub experimental: Option<ExperimentalConfig>,
    pub tui: Option<TuiConfig>,  // #[serde(skip)] — runtime-only
    pub api_key: Option<String>,
    pub temperature: Option<f32>,
    pub max_tokens: Option<u32>,
}
```

**Key methods**:
```rust
impl Config {
    pub fn load(path: &PathBuf) -> Result<Self, ConfigError>
    pub fn substitute_variables(input: &str, config_dir: Option<&Path>) -> Result<String, ConfigError>
    pub fn contains_keychain_reference(s: &str) -> bool
    pub fn redact_keychain_references(s: &str) -> String
    pub fn expand_variables(value: &mut serde_json::Value) -> Result<(), ConfigError>
    pub fn config_path() -> PathBuf
    pub fn load_tui_config_path() -> Option<PathBuf>
}
```

### ConfigError

```rust
#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum ConfigError {
    #[error("Config error: {0}")]
    Config(String),
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),
}

impl From<ConfigError> for String { ... }
```

### LogLevel

```rust
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum LogLevel {
    Trace,
    Debug,
    Info,
    Warn,
    Error,
}
```

### ServerConfig

```rust
#[derive(Debug, Clone, Deserialize, Serialize, Default)]
pub struct ServerConfig {
    pub port: Option<u16>,
    pub hostname: Option<String>,
    pub mdns: Option<bool>,
    pub mdns_domain: Option<String>,
    pub cors: Option<Vec<String>>,
    pub desktop: Option<DesktopConfig>,
    pub acp: Option<AcpConfig>,
}

#[derive(Debug, Clone, Deserialize, Serialize, Default)]
pub struct DesktopConfig {
    pub enabled: Option<bool>,
    pub auto_open_browser: Option<bool>,
    pub port: Option<u16>,
    pub hostname: Option<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize, Default)]
pub struct AcpConfig {
    pub enabled: Option<bool>,
    pub server_id: Option<String>,
    pub version: Option<String>,
}
```

### ProviderConfig

```rust
#[derive(Debug, Clone, Deserialize, Serialize, Default)]
pub struct ProviderConfig {
    pub id: Option<String>,
    pub name: Option<String>,
    pub whitelist: Option<Vec<String>>,
    pub blacklist: Option<Vec<String>>,
    pub models: Option<HashMap<String, ModelConfig>>,
    pub options: Option<ProviderOptions>,
}

impl ProviderConfig {
    pub fn sanitize_for_logging(&self) -> Self { ... }
}

#[derive(Debug, Clone, Deserialize, Serialize, Default)]
pub struct ModelConfig {
    pub id: Option<String>,
    pub name: Option<String>,
    pub variants: Option<HashMap<String, VariantConfig>>,
    pub visible: Option<bool>,
    pub extra: Option<HashMap<String, serde_json::Value>>,
}

#[derive(Debug, Clone, Deserialize, Serialize, Default)]
pub struct VariantConfig {
    pub disabled: Option<bool>,
    pub extra: Option<HashMap<String, serde_json::Value>>,
}

#[derive(Clone, Deserialize, Serialize, Default)]
pub struct ProviderOptions {
    pub api_key: Option<String>,
    pub base_url: Option<String>,
    pub enterprise_url: Option<String>,
    pub set_cache_key: Option<bool>,
    pub timeout: Option<TimeoutConfig>,
    pub chunk_timeout: Option<u64>,
    pub aws_region: Option<String>,
    pub aws_profile: Option<String>,
    pub aws_endpoint: Option<String>,
    pub headers: Option<HashMap<String, String>>,
    pub extra: Option<HashMap<String, serde_json::Value>>,
}

impl ProviderOptions {
    pub fn sanitize_for_logging(&self) -> Self { ... }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(untagged)]
pub enum TimeoutConfig {
    Milliseconds(u64),
    NoTimeout(bool),
}
```

### McpConfig

```rust
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(tag = "type", rename_all = "lowercase")]
pub enum McpConfig {
    Local(McpLocalConfig),
    Remote(McpRemoteConfig),
    Simple { enabled: bool },
}

#[derive(Debug, Clone, Deserialize, Serialize, Default)]
pub struct McpLocalConfig {
    pub command: Vec<String>,
    pub environment: Option<HashMap<String, String>>,
    pub enabled: Option<bool>,
    pub timeout: Option<u64>,
    pub max_tokens: Option<usize>,
    pub cost_warning_threshold: Option<f64>,
    pub cost_limit_threshold: Option<f64>,
}

#[derive(Debug, Clone, Deserialize, Serialize, Default)]
pub struct McpRemoteConfig {
    pub url: String,
    pub enabled: Option<bool>,
    pub headers: Option<HashMap<String, String>>,
    pub oauth: Option<McpOAuthUnion>,
    pub timeout: Option<u64>,
    pub max_tokens: Option<usize>,
    pub cost_warning_threshold: Option<f64>,
    pub cost_limit_threshold: Option<f64>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(untagged)]
pub enum McpOAuthUnion {
    Config(McpOAuthConfig),
    Disabled(bool),
}

#[derive(Debug, Clone, Deserialize, Serialize, Default)]
pub struct McpOAuthConfig {
    pub client_id: Option<String>,
    pub client_secret: Option<String>,
    pub scope: Option<String>,
}
```

### PermissionConfig

```rust
#[derive(Debug, Clone, Deserialize, Serialize, Default)]
pub struct PermissionConfig {
    pub read: Option<PermissionRule>,
    pub edit: Option<PermissionRule>,
    pub glob: Option<PermissionRule>,
    pub grep: Option<PermissionRule>,
    pub list: Option<PermissionRule>,
    pub bash: Option<PermissionRule>,
    pub task: Option<PermissionRule>,
    pub external_directory: Option<PermissionRule>,
    pub todowrite: Option<PermissionAction>,
    pub question: Option<PermissionAction>,
    pub webfetch: Option<PermissionAction>,
    pub websearch: Option<PermissionAction>,
    pub codesearch: Option<PermissionAction>,
    pub lsp: Option<PermissionRule>,
    pub doom_loop: Option<PermissionAction>,
    pub skill: Option<PermissionRule>,
    pub extra: Option<HashMap<String, PermissionRule>>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum PermissionAction {
    Ask,
    Allow,
    Deny,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(untagged)]
pub enum PermissionRule {
    Action(PermissionAction),
    Object(HashMap<String, PermissionAction>),
}
```

### AgentMapConfig and AgentConfig

```rust
#[derive(Debug, Clone, Serialize, Default)]
pub struct AgentMapConfig {
    #[serde(default)]
    pub agents: HashMap<String, AgentConfig>,
    pub default_agent: Option<String>,
}

impl AgentMapConfig {
    pub fn get_agent(&self, name: &str) -> Option<&AgentConfig> { ... }
    pub fn get_default_agent(&self) -> Option<&AgentConfig> { ... }
}

#[derive(Debug, Clone, Deserialize, Serialize, Default)]
pub struct AgentConfig {
    pub model: Option<String>,
    pub variant: Option<String>,
    pub temperature: Option<f32>,
    pub top_p: Option<f32>,
    pub prompt: Option<String>,
    pub disable: Option<bool>,
    pub description: Option<String>,
    pub hidden: Option<bool>,
    pub options: Option<HashMap<String, serde_json::Value>>,
    pub color: Option<String>,
    pub steps: Option<u32>,
    pub max_steps: Option<u32>,
    pub permission: Option<PermissionConfig>,
}
```

### Other notable config types

```rust
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum ShareMode {
    Manual,
    Auto,
    Disabled,
    ReadOnly,
    Collaborative,
    Controlled,
}

#[derive(Debug, Clone, Deserialize, Serialize, Default)]
pub struct CompactionConfig {
    pub auto: Option<bool>,
    pub prune: Option<bool>,
    pub reserved: Option<u32>,
    pub warning_threshold: Option<f64>,
    pub compact_threshold: Option<f64>,
    pub continuation_threshold: Option<f64>,
    pub preserve_recent_messages: Option<usize>,
    pub preserve_system_messages: Option<bool>,
    pub summary_prefix: Option<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize, Default)]
pub struct ExperimentalConfig {
    pub disable_paste_summary: Option<bool>,
    pub batch_tool: Option<bool>,
    pub open_telemetry: Option<bool>,
    pub primary_tools: Option<Vec<String>>,
    pub continue_loop_on_deny: Option<bool>,
    pub mcp_timeout: Option<u64>,
}

#[derive(Debug, Clone, Deserialize, Serialize, Default)]
pub struct TuiConfig {
    pub schema: Option<String>,
    pub scroll_speed: Option<u32>,
    pub scroll_acceleration: Option<ScrollAccelerationConfig>,
    pub diff_style: Option<DiffStyle>,
    pub theme: Option<ThemeConfig>,
    pub keybinds: Option<KeybindConfig>,
    pub plugins: Option<TuiPluginConfig>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum DiffStyle {
    SideBySide,
    Inline,
    Unified,
    Auto,
    Stacked,
}
```

---

## Key Implementation Details

### Variable Expansion

The `Config` supports three variable substitution syntaxes in config files:

1. **`{env:VAR_NAME}`** — Expands from environment variables (recursive)
2. **`{file:/path/to/file}`** — Inlines file contents (recursive, path resolved relative to config dir or `~`)
3. **`{keychain:SECRET_NAME}`** — Resolves from OS keychain (macOS: Keychain, Linux: secret-service)

And JSON-value variable expansion with `${variable_name}` references within string values.

### Config Loading Priority

`Config::load(path)` does:
1. If path doesn't exist → return `Config::default()`
2. Read file content, substitute variables
3. Parse as JSON or JSONC (with comment stripping)
4. Check/migrate deprecated fields
5. Expand `${}` variables
6. Apply environment overrides
7. Return merged config

### Config Path Resolution

```rust
pub fn config_path() -> PathBuf  // Uses OPENCODE_CONFIG_DIR env or ~/.config/opencode/config.json
pub fn load_tui_config_path() -> Option<PathBuf>  // Uses OPENCODE_TUI_CONFIG env or ~/.config/opencode/tui.json
```

---

## Inter-Crate Dependencies

| Dependant Crate | What it uses from `opencode-config` |
|---|---|
| `opencode-core` | `Config` struct for global state initialization |
| `opencode-server` | `ServerConfig`, `Config` loading at startup |
| `opencode-tui` | `TuiConfig`, `ThemeConfig`, `KeybindConfig` |
| `opencode-cli` | Full `Config` for CLI argument overrides |
| `opencode-agent` | `AgentConfig`, `AgentMapConfig`, `PermissionConfig` |
| `opencode-llm` | `ProviderConfig`, `ModelConfig`, `ProviderOptions` |
| `opencode-mcp` | `McpConfig`, `McpLocalConfig`, `McpRemoteConfig` |
| `opencode-storage` | `CompactionConfig` |
| `opencode-plugin` | Plugin loading from config |

---

## Test Design

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_load_default() {
        let config = Config::default();
        assert!(config.log_level.is_none());
        assert!(config.server.is_none());
    }

    #[test]
    fn test_config_load_missing_file_returns_default() {
        let result = Config::load(&PathBuf::from("/nonexistent/path/config.json"));
        assert!(result.is_ok());
    }

    #[test]
    fn test_config_env_variable_substitution() {
        std::env::set_var("TEST_API_KEY", "secret123");
        let result = Config::substitute_variables("{env:TEST_API_KEY}", None);
        assert_eq!(result.unwrap(), "secret123");
    }

    #[test]
    fn test_config_keychain_reference_detection() {
        assert!(Config::contains_keychain_reference("{keychain:my-secret}"));
        assert!(!Config::contains_keychain_reference("plaintext"));
    }

    #[test]
    fn test_agent_map_config_get_agent() {
        let mut agents = HashMap::new();
        agents.insert("build".to_string(), AgentConfig::default());
        let map = AgentMapConfig { agents, default_agent: Some("build".to_string()) };
        assert!(map.get_agent("build").is_some());
        assert!(map.get_agent("nonexistent").is_none());
    }

    #[test]
    fn test_agent_map_config_get_default_agent() {
        let mut agents = HashMap::new();
        agents.insert("build".to_string(), AgentConfig::default());
        let map = AgentMapConfig { agents, default_agent: Some("build".to_string()) };
        assert!(map.get_default_agent().is_some());
    }

    #[test]
    fn test_provider_options_sanitize_for_logging() {
        let options = ProviderOptions {
            api_key: Some("secret".to_string()),
            base_url: Some("https://api.example.com".to_string()),
            ..Default::default()
        };
        let sanitized = options.sanitize_for_logging();
        assert_eq!(sanitized.api_key, Some("***REDACTED***".to_string()));
        assert_eq!(sanitized.base_url, Some("https://api.example.com".to_string()));
    }

    #[test]
    fn test_share_mode_variants() {
        assert_eq!(serde_json::to_string(&ShareMode::Auto).unwrap(), "\"auto\"");
        assert_eq!(serde_json::to_string(&ShareMode::Collaborative).unwrap(), "\"collaborative\"");
    }

    #[test]
    fn test_permission_action_serialization() {
        assert_eq!(serde_json::to_string(&PermissionAction::Allow).unwrap(), "\"allow\"");
        assert_eq!(serde_json::to_string(&PermissionAction::Deny).unwrap(), "\"deny\"");
    }

    #[test]
    fn test_tui_config_scroll_acceleration_legacy_number() {
        // ScrollAccelerationConfig can deserialize from a plain f64 (legacy format)
        let json = "0.5";
        let config: ScrollAccelerationConfig = serde_json::from_str(json).unwrap();
        assert!(config.enabled);
        assert_eq!(config.speed, Some(0.5));
    }

    #[test]
    fn test_tui_config_scroll_acceleration_object() {
        let json = r#"{"enabled": true, "speed": 1.5}"#;
        let config: ScrollAccelerationConfig = serde_json::from_str(json).unwrap();
        assert!(config.enabled);
        assert_eq!(config.speed, Some(1.5));
    }

    #[test]
    fn test_mcp_config_local_variant() {
        let json = r#"{"type": "local", "command": ["npx", "mcp-server"]}"#;
        let config: McpConfig = serde_json::from_str(json).unwrap();
        match config {
            McpConfig::Local(local) => assert_eq!(local.command, vec!["npx", "mcp-server"]),
            _ => panic!("expected Local variant"),
        }
    }

    #[test]
    fn test_mcp_config_remote_variant() {
        let json = r#"{"type": "remote", "url": "https://mcp.example.com"}"#;
        let config: McpConfig = serde_json::from_str(json).unwrap();
        match config {
            McpConfig::Remote(remote) => assert_eq!(remote.url, "https://mcp.example.com"),
            _ => panic!("expected Remote variant"),
        }
    }
}
```

---

## Usage Example

```rust
use opencode_config::Config;
use std::path::PathBuf;

fn main() -> Result<(), ConfigError> {
    let config_path = Config::config_path();
    let config = Config::load(&config_path)?;
    
    // Server config
    if let Some(server) = &config.server {
        println!("Server port: {:?}", server.port);
    }
    
    // Provider configs
    if let Some(providers) = &config.provider {
        for (name, provider) in providers {
            println!("Provider '{}': {:?}", name, provider.name);
        }
    }
    
    // Agent configs
    if let Some(agent_map) = &config.agent {
        if let Some(default) = agent_map.get_default_agent() {
            println!("Default agent model: {:?}", default.model);
        }
    }
    
    Ok(())
}
```
