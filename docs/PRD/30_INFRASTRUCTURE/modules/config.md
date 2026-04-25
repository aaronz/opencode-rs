# config.md — Configuration Module

> **User Documentation**: [config.mdx](https://github.com/anomalyco/opencode/blob/main/packages/web/src/content/docs/zh-cn/config.mdx) — Configuration guide for users

**See Also**: [Glossary: Config](../../system/01_glossary.md#config) | [System PRD: Configuration System](../../system/06-configuration-system.md)

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

---

## Error Handling

### Config Error Types

| Error Type | Code | Description |
|------------|------|-------------|
| `ConfigMissing` | 6001 | Required configuration key missing |
| `ConfigInvalid` | 6002 | Configuration value is invalid |
| `ConfigLoadFailed` | 6003 | Failed to load configuration |
| `JsoncParseError` | 6004 | Failed to parse JSONC file |
| `SecretResolveError` | 6005 | Failed to resolve secret from keychain |

### Error Handling Matrix

| Scenario | Expected Behavior | Error Code |
|----------|------------------|------------|
| Config file not found | Use defaults or env vars | 6003 |
| Invalid JSON syntax | Return `JsoncParseError` with line number | 6004 |
| Invalid value type | Return `ConfigInvalid` with field path | 6002 |
| Missing required field | Return `ConfigMissing` | 6001 |
| Keychain unavailable | Return `SecretResolveError` | 6005 |
| Permission denied on config file | Return `ConfigLoadFailed` | 6003 |

---

## Configuration Precedence

Configuration is loaded in order of precedence (later overrides earlier):

| Priority | Source | Description |
|----------|--------|-------------|
| 1 (lowest) | Built-in defaults | Hardcoded defaults in code |
| 2 | System config file | `/etc/opencode-rs/config.toml` |
| 3 | User config file | `~/.config/opencode-rs/config.toml` |
| 4 | Project config file | `./.opencode/config.toml` |
| 5 | Environment variables | `OPENCODE_*` prefixed vars |
| 6 | Command-line arguments | CLI flags |

### Environment Variable Mapping

| Env Var | Config Path | Type |
|---------|-------------|------|
| `OPENCODE_CONFIG` | - | Path to config file |
| `OPENCODE_DATA_DIR` | - | Data directory path |
| `OPENCODE_LOG_LEVEL` | `log_level` | trace, debug, info, warn, error |
| `OPENCODE_SERVER_PORT` | `server.port` | u16 |
| `OPENCODE_SERVER_HOST` | `server.host` | String |
| `OPENAI_API_KEY` | `provider.openai.api_key` | String |
| `ANTHROPIC_API_KEY` | `provider.anthropic.api_key` | String |

---

## Acceptance Criteria

### Config Loading

| ID | Criterion | Given-When-Then |
|----|-----------|------------------|
| AC-LOAD001 | Load default config | Given no config file exists, When `Config::load()` called, Then return default config |
| AC-LOAD002 | Load from config file | Given valid `config.toml` exists, When `Config::load()`, Then return merged config |
| AC-LOAD003 | JSONC with comments | Given config file has `// comments` and `/* block comments */`, When `Config::load()`, Then parse succeeds ignoring comments |
| AC-LOAD004 | Nested config merge | Given user config has `server.port=3000` and default has `server.port=8080`, When loaded, Then final `port=3000` |
| AC-LOAD005 | Unknown fields ignored | Given config has unknown field `"unknown_field": true`, When `Config::load()`, Then load succeeds (ignore unknown) |

### Environment Variable Override

| ID | Criterion | Given-When-Then |
|----|-----------|------------------|
| AC-ENV001 | Env var overrides file | Given `OPENCODE_SERVER_PORT=9000` and config file has `port=8080`, When `Config::load()`, Then final `port=9000` |
| AC-ENV002 | Env var for nested config | Given `OPENAI_API_KEY=sk-test`, When config loaded, Then `provider.openai.api_key=sk-test` |
| AC-ENV003 | Missing env var uses default | Given `OPENCODE_SERVER_PORT` not set, When config loaded, Then use file or default value |
| AC-ENV004 | Invalid env var type | Given `OPENCODE_SERVER_PORT=not-a-number`, When config loaded, Then return `ConfigInvalid` |

### Secret Resolution

| ID | Criterion | Given-When-Then |
|----|-----------|------------------|
| AC-SECRET001 | Resolve from keychain | Given config has `"api_key": "keychain://openai-api-key"`, When loaded, Then resolve to actual key from keychain |
| AC-SECRET002 | Keychain key not found | Given keychain entry missing, When `Config::load()`, Then return `SecretResolveError` |
| AC-SECRET003 | Env var fallback | Given keychain fails but `OPENAI_API_KEY` env var set, When load, Then use env var |
| AC-SECRET004 | Secret not exposed in logs | Given secret in config, When logging config, Then secret value masked as `***` |

### File Variable Expansion

| ID | Criterion | Given-When-Then |
|----|-----------|------------------|
| AC-EXPAND001 | Expand `$HOME` | Given config has `"data_dir": "$HOME/.opencode"`, When loaded, Then expand to actual home path |
| AC-EXPAND002 | Expand `$WORKSPACE` | Given config has `"root": "$WORKSPACE"`, When loaded, Then expand to workspace env var |
| AC-EXPAND003 | Nested expansion | Given `"path": "$DATA_DIR/sessions"` and `DATA_DIR=/opt/data`, When loaded, Then path=/opt/data/sessions |
| AC-EXPAND004 | Unknown variable | Given `"path": "$UNKNOWN_VAR/file"` and var not set, When loaded, Then leave as-is or return error |

### Validation

| ID | Criterion | Given-When-Then |
|----|-----------|------------------|
| AC-VALID001 | Port range | Given `port: 70000` (invalid), When `Config::load()`, Then return `ConfigInvalid` |
| AC-VALID002 | URL format | Given `server.url: "not-a-url"`, When `Config::load()`, Then return `ConfigInvalid` |
| AC-VALID003 | Required field missing | Given required `server.port` missing, When `Config::load()`, Then return `ConfigMissing` |
| AC-VALID004 | Duplicate keys | Given config has duplicate keys, When loaded, Then later value wins (no error) |

### Directory Scanning

| ID | Criterion | Given-When-Then |
|----|-----------|------------------|
| AC-SCAN001 | Scan `.opencode/` | Given project has `.opencode/` directory with `tools.json`, When scan, Then load tool definitions |
| AC-SCAN002 | Merge multiple configs | Given `.opencode/` has multiple config files, When scan, Then merge in order |
| AC-SCAN003 | Ignore hidden files | Given `.opencode/` has `.hidden.toml`, When scan, Then ignore hidden files |
| AC-SCAN004 | Nested directory scan | Given deep nested `.opencode/` dirs, When scan, Then only scan immediate `.opencode/` |

### Remote Config

| ID | Criterion | Given-When-Then |
|----|-----------|------------------|
| AC-REMOTE001 | Fetch remote config | Given `remote_config_url` set, When `Config::load()`, Then fetch and merge remote config |
| AC-REMOTE002 | Remote takes precedence | Given remote has `port=9000` and local has `port=8080`, When loaded, Then final `port=9000` |
| AC-REMOTE003 | Remote unavailable | Given remote URL unreachable, When `Config::load()`, Then fail with `ConfigLoadFailed` or use local only |
| AC-REMOTE004 | Remote cache | Given remote fetched, When load again within TTL, Then use cached remote config |

### Server Config

| ID | Criterion | Given-When-Then |
|----|-----------|------------------|
| AC-SERVER001 | Default port | Given no server config, When `Config::load()`, Then `server.port=8080` |
| AC-SERVER002 | Custom port | Given `server.port=3000`, When loaded, Then use 3000 |
| AC-SERVER003 | Host binding | Given `server.host="127.0.0.1"`, When loaded, Then bind to localhost only |
| AC-SERVER004 | CORS settings | Given `server.cors.enabled=true`, When loaded, Then enable CORS with config |

### Provider Config

| ID | Criterion | Given-When-Then |
|----|-----------|------------------|
| AC-PROV001 | Provider array syntax | Given `providers=["openai", "anthropic"]`, When loaded, Then enable both providers |
| AC-PROV002 | Provider object syntax | Given `providers.openai={api_key: "..."}`, When loaded, Then configure OpenAI |
| AC-PROV003 | Provider priority | Given multiple providers, When `select_provider()` called, Then use priority order |
| AC-PROV004 | Disabled provider | Given `disabled_providers=["openai"]`, When load, Then OpenAI not available |

### Agent Config

| ID | Criterion | Given-When-Then |
|----|-----------|------------------|
| AC-AGENT001 | Default agent | Given `agents.default="build"`, When `Config::load()`, Then default agent is BuildAgent |
| AC-AGENT002 | Agent options | Given `agents.build.model="gpt-4o"`, When loaded, Then BuildAgent uses gpt-4o |
| AC-AGENT003 | Agent permission | Given `agents.build.permission="read-only"`, When loaded, Then BuildAgent restricted |

### MCP Config

| ID | Criterion | Given-When-Then |
|----|-----------|------------------|
| AC-MCP001 | Local MCP server | Given `mcp.type="local"` with command, When loaded, Then spawn local MCP server |
| AC-MCP002 | Remote MCP server | Given `mcp.type="remote"` with URL, When loaded, Then connect to remote MCP |
| AC-MCP003 | Multiple MCP servers | Given array of MCP configs, When loaded, Then start all servers |
| AC-MCP004 | MCP auth | Given `mcp.bearer_token="..."`, When loaded, Then configure auth header |

### Performance

| ID | Criterion | Target | Test Method |
|----|-----------|--------|-------------|
| AC-PERF001 | Config load time | < 100ms | Benchmark |
| AC-PERF002 | Env var scan | < 10ms | Benchmark |
| AC-PERF003 | Secret resolution | < 50ms per secret | Benchmark |

---

## Cross-References

| Reference | Description |
|-----------|-------------|
| [Configuration System PRD](../../system/06-configuration-system.md) | System-level config architecture |
| [Glossary: Config](../../system/01_glossary.md#config) | Config terminology |
| [ERROR_CODE_CATALOG.md](../../ERROR_CODE_CATALOG.md#6xxx) | Config error codes (6001-6005) |
| [auth.md](../auth.md) | Secret/keychain integration |
```
