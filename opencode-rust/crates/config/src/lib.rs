use chrono::{DateTime, Duration, Utc};
use reqwest::header::{CACHE_CONTROL, ETAG, IF_MODIFIED_SINCE, IF_NONE_MATCH, LAST_MODIFIED};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use sha2::{Digest, Sha256};
use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};

mod directory_scanner;
mod jsonc;
mod merge;
mod remote_cache;
mod schema;
mod secret_storage;
pub use directory_scanner::{
    load_opencode_directory, DirectoryScanner, OpencodeDirectoryScan, ToolInfo,
};
pub use jsonc::{is_jsonc_extension, parse_jsonc, JsoncError};
use remote_cache::{load_cache, save_cache, RemoteConfigCache};
use secret_storage::resolve_keychain_secret;

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

impl From<ConfigError> for String {
    fn from(err: ConfigError) -> String {
        err.to_string()
    }
}

/// Main configuration structure matching the TypeScript Config.Info schema
#[derive(Debug, Clone, Deserialize, Serialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct Config {
    #[serde(rename = "$schema", skip_serializing_if = "Option::is_none")]
    pub schema: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub log_level: Option<LogLevel>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub server: Option<ServerConfig>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub command: Option<HashMap<String, CommandConfig>>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub skills: Option<SkillsConfig>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub watcher: Option<WatcherConfig>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub plugin: Option<Vec<String>>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub snapshot: Option<bool>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub share: Option<ShareMode>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub autoshare: Option<bool>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub autoupdate: Option<AutoUpdate>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub disabled_providers: Option<Vec<String>>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub enabled_providers: Option<Vec<String>>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub model: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub small_model: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub default_agent: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub username: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub agent: Option<AgentMapConfig>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub provider: Option<HashMap<String, ProviderConfig>>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub mcp: Option<HashMap<String, McpConfig>>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub formatter: Option<FormatterConfig>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub lsp: Option<LspConfig>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub instructions: Option<Vec<String>>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub agents_md: Option<AgentsMdConfig>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub permission: Option<PermissionConfig>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub enterprise: Option<EnterpriseConfig>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub compaction: Option<CompactionConfig>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub experimental: Option<ExperimentalConfig>,

    #[serde(skip)]
    pub tui: Option<TuiConfig>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub api_key: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub temperature: Option<f32>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_tokens: Option<u32>,
}

#[derive(Debug, Clone, Default)]
pub struct CliOverrideConfig {
    pub model: Option<String>,
    pub provider: Option<String>,
    pub temperature: Option<f32>,
    pub max_tokens: Option<u32>,
    pub default_agent: Option<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum LogLevel {
    Trace,
    Debug,
    Info,
    Warn,
    Error,
}

#[derive(Debug, Clone, Deserialize, Serialize, Default)]
pub struct ServerConfig {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub port: Option<u16>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub hostname: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub mdns: Option<bool>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub mdns_domain: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub cors: Option<Vec<String>>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub desktop: Option<DesktopConfig>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub acp: Option<AcpConfig>,
}

#[derive(Debug, Clone, Deserialize, Serialize, Default)]
pub struct DesktopConfig {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub enabled: Option<bool>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub auto_open_browser: Option<bool>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub port: Option<u16>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub hostname: Option<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize, Default)]
pub struct AcpConfig {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub enabled: Option<bool>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub server_id: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub version: Option<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize, Default)]
pub struct CommandConfig {
    pub template: String,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub agent: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub model: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub subtask: Option<bool>,
}

#[derive(Debug, Clone, Deserialize, Serialize, Default)]
pub struct SkillsConfig {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub paths: Option<Vec<String>>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub urls: Option<Vec<String>>,
}

#[derive(Debug, Clone, Deserialize, Serialize, Default)]
pub struct AgentsMdConfig {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub enabled: Option<bool>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub stop_at_worktree_root: Option<bool>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub include_hidden: Option<bool>,
}

impl AgentsMdConfig {
    pub fn to_scan_config(&self) -> crate_default::AgentsMdScanConfig {
        crate_default::AgentsMdScanConfig {
            enabled: self.enabled.unwrap_or(true),
            stop_at_worktree_root: self.stop_at_worktree_root.unwrap_or(true),
            include_hidden: self.include_hidden.unwrap_or(false),
        }
    }
}

mod crate_default {
    pub struct AgentsMdScanConfig {
        pub enabled: bool,
        pub stop_at_worktree_root: bool,
        pub include_hidden: bool,
    }
}

#[derive(Debug, Clone, Deserialize, Serialize, Default)]
pub struct WatcherConfig {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ignore: Option<Vec<String>>,
}

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

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
#[serde(untagged)]
pub enum AutoUpdate {
    Bool(bool),
    Notify(String),
}

#[derive(Debug, Clone, Serialize, Default)]
pub struct AgentMapConfig {
    #[serde(default)]
    pub agents: HashMap<String, AgentConfig>,
    #[serde(alias = "defaultAgent", skip_serializing_if = "Option::is_none")]
    pub default_agent: Option<String>,
}

impl<'de> Deserialize<'de> for AgentMapConfig {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        #[derive(Deserialize)]
        struct NewFormat {
            #[serde(default)]
            agents: HashMap<String, AgentConfig>,
            #[serde(alias = "defaultAgent", default)]
            default_agent: Option<String>,
        }

        let value = serde_json::Value::deserialize(deserializer)?;
        if let Some(obj) = value.as_object() {
            if obj.contains_key("agents") {
                let nf: NewFormat =
                    serde_json::from_value(value).map_err(serde::de::Error::custom)?;
                Ok(AgentMapConfig {
                    agents: nf.agents,
                    default_agent: nf.default_agent,
                })
            } else {
                let agents: HashMap<String, AgentConfig> =
                    serde_json::from_value(value).map_err(serde::de::Error::custom)?;
                Ok(AgentMapConfig {
                    agents,
                    default_agent: None,
                })
            }
        } else {
            Err(serde::de::Error::custom(
                "expected object for AgentMapConfig",
            ))
        }
    }
}

impl AgentMapConfig {
    pub fn get_agent(&self, name: &str) -> Option<&AgentConfig> {
        self.agents.get(name)
    }

    pub fn get_default_agent(&self) -> Option<&AgentConfig> {
        self.default_agent
            .as_deref()
            .and_then(|name| self.get_agent(name))
    }
}

#[derive(Debug, Clone, Deserialize, Serialize, Default)]
pub struct AgentConfig {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub model: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub variant: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub temperature: Option<f32>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub top_p: Option<f32>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub prompt: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub disable: Option<bool>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub hidden: Option<bool>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub options: Option<HashMap<String, serde_json::Value>>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub color: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub steps: Option<u32>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_steps: Option<u32>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub permission: Option<PermissionConfig>,
}

#[derive(Debug, Clone, Deserialize, Serialize, Default)]
pub struct ProviderConfig {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub whitelist: Option<Vec<String>>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub blacklist: Option<Vec<String>>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub models: Option<HashMap<String, ModelConfig>>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub options: Option<ProviderOptions>,
}

impl ProviderConfig {
    pub fn sanitize_for_logging(&self) -> Self {
        let mut sanitized = self.clone();
        if let Some(options) = &sanitized.options {
            sanitized.options = Some(options.sanitize_for_logging());
        }
        sanitized
    }
}

#[derive(Debug, Clone, Deserialize, Serialize, Default)]
pub struct ModelConfig {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub variants: Option<HashMap<String, VariantConfig>>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub visible: Option<bool>,

    #[serde(flatten, skip_serializing_if = "Option::is_none")]
    pub extra: Option<HashMap<String, serde_json::Value>>,
}

#[derive(Debug, Clone, Deserialize, Serialize, Default)]
pub struct VariantConfig {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub disabled: Option<bool>,

    #[serde(flatten, skip_serializing_if = "Option::is_none")]
    pub extra: Option<HashMap<String, serde_json::Value>>,
}

#[derive(Clone, Deserialize, Serialize, Default)]
pub struct ProviderOptions {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub api_key: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub base_url: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub enterprise_url: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub set_cache_key: Option<bool>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub timeout: Option<TimeoutConfig>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub chunk_timeout: Option<u64>,

    #[serde(skip_serializing_if = "Option::is_none", rename = "awsRegion")]
    pub aws_region: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none", rename = "awsProfile")]
    pub aws_profile: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none", rename = "awsEndpoint")]
    pub aws_endpoint: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub headers: Option<HashMap<String, String>>,

    #[serde(flatten, skip_serializing_if = "Option::is_none")]
    pub extra: Option<HashMap<String, serde_json::Value>>,
}

impl ProviderOptions {
    pub fn sanitize_for_logging(&self) -> Self {
        let mut sanitized = self.clone();
        if sanitized.api_key.is_some() {
            sanitized.api_key = Some("***REDACTED***".to_string());
        }
        sanitized
    }
}

impl std::fmt::Debug for ProviderOptions {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let sanitized = self.sanitize_for_logging();
        f.debug_struct("ProviderOptions")
            .field("api_key", &sanitized.api_key)
            .field("base_url", &self.base_url)
            .field("enterprise_url", &self.enterprise_url)
            .field("set_cache_key", &self.set_cache_key)
            .field("timeout", &self.timeout)
            .field("chunk_timeout", &self.chunk_timeout)
            .field("aws_region", &self.aws_region)
            .field("aws_profile", &self.aws_profile)
            .field("aws_endpoint", &self.aws_endpoint)
            .field("headers", &self.headers)
            .field("extra", &self.extra)
            .finish()
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(untagged)]
pub enum TimeoutConfig {
    Milliseconds(u64),
    NoTimeout(bool),
}

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

    #[serde(skip_serializing_if = "Option::is_none")]
    pub environment: Option<HashMap<String, String>>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub enabled: Option<bool>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub timeout: Option<u64>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_tokens: Option<usize>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub cost_warning_threshold: Option<f64>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub cost_limit_threshold: Option<f64>,
}

#[derive(Debug, Clone, Deserialize, Serialize, Default)]
pub struct McpOAuthConfig {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub client_id: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub client_secret: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub scope: Option<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize, Default)]
pub struct McpRemoteConfig {
    pub url: String,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub enabled: Option<bool>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub headers: Option<HashMap<String, String>>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub oauth: Option<McpOAuthUnion>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub timeout: Option<u64>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_tokens: Option<usize>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub cost_warning_threshold: Option<f64>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub cost_limit_threshold: Option<f64>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(untagged)]
pub enum McpOAuthUnion {
    Config(McpOAuthConfig),
    Disabled(bool),
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(untagged)]
pub enum FormatterConfig {
    Disabled(bool),
    Formatters(HashMap<String, FormatterEntry>),
}

#[derive(Debug, Clone, Deserialize, Serialize, Default)]
pub struct FormatterEntry {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub disabled: Option<bool>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub command: Option<Vec<String>>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub environment: Option<HashMap<String, String>>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub extensions: Option<Vec<String>>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(untagged)]
pub enum LspConfig {
    Disabled(bool),
    Servers(HashMap<String, LspEntry>),
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(untagged)]
pub enum LspEntry {
    Disabled {
        disabled: bool,
    },
    Config {
        command: Vec<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        extensions: Option<Vec<String>>,
        #[serde(skip_serializing_if = "Option::is_none")]
        disabled: Option<bool>,
        #[serde(skip_serializing_if = "Option::is_none")]
        env: Option<HashMap<String, String>>,
        #[serde(skip_serializing_if = "Option::is_none")]
        initialization: Option<HashMap<String, serde_json::Value>>,
    },
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

#[derive(Debug, Clone, Deserialize, Serialize, Default)]
pub struct PermissionConfig {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub read: Option<PermissionRule>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub edit: Option<PermissionRule>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub glob: Option<PermissionRule>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub grep: Option<PermissionRule>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub list: Option<PermissionRule>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub bash: Option<PermissionRule>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub task: Option<PermissionRule>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub external_directory: Option<PermissionRule>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub todowrite: Option<PermissionAction>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub question: Option<PermissionAction>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub webfetch: Option<PermissionAction>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub websearch: Option<PermissionAction>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub codesearch: Option<PermissionAction>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub lsp: Option<PermissionRule>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub doom_loop: Option<PermissionAction>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub skill: Option<PermissionRule>,

    #[serde(flatten, skip_serializing_if = "Option::is_none")]
    pub extra: Option<HashMap<String, PermissionRule>>,
}

#[derive(Debug, Clone, Deserialize, Serialize, Default)]
pub struct EnterpriseConfig {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub url: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none", rename = "remoteConfigDomain")]
    pub remote_config_domain: Option<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize, Default)]
pub struct CompactionConfig {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub auto: Option<bool>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub prune: Option<bool>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub reserved: Option<u32>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub warning_threshold: Option<f64>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub compact_threshold: Option<f64>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub continuation_threshold: Option<f64>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub preserve_recent_messages: Option<usize>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub preserve_system_messages: Option<bool>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub summary_prefix: Option<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize, Default)]
pub struct ExperimentalConfig {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub disable_paste_summary: Option<bool>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub batch_tool: Option<bool>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub open_telemetry: Option<bool>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub primary_tools: Option<Vec<String>>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub continue_loop_on_deny: Option<bool>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub mcp_timeout: Option<u64>,
}

#[derive(Debug, Clone, Deserialize, Serialize, Default)]
pub struct KeybindConfig {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub commands: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub timeline: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub settings: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub models: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub files: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub terminal: Option<String>,
    #[serde(flatten, skip_serializing_if = "Option::is_none")]
    pub custom: Option<std::collections::HashMap<String, String>>,
}

impl KeybindConfig {
    pub fn merge_with_defaults(&self, defaults: &KeybindConfig) -> (KeybindConfig, Vec<String>) {
        let mut merged = defaults.clone();

        macro_rules! merge_field {
            ($field:ident) => {
                if self.$field.is_some() {
                    merged.$field = self.$field.clone();
                }
            };
        }
        merge_field!(commands);
        merge_field!(timeline);
        merge_field!(settings);
        merge_field!(models);
        merge_field!(files);
        merge_field!(terminal);

        if let Some(ref custom) = self.custom {
            merged
                .custom
                .get_or_insert_with(std::collections::HashMap::new)
                .extend(custom.clone());
        }

        let conflicts = merged.detect_conflicts();

        (merged, conflicts)
    }

    pub fn detect_conflicts(&self) -> Vec<String> {
        let mut conflicts = Vec::new();
        let mut reverse: std::collections::HashMap<String, Vec<String>> =
            std::collections::HashMap::new();
        for (action, binding) in Self::bindings_with_labels(self) {
            reverse.entry(binding).or_default().push(action);
        }
        for (binding, mut actions) in reverse {
            if actions.len() > 1 {
                actions.sort();
                actions.dedup();
                for i in 1..actions.len() {
                    conflicts.push(format!(
                        "{} used by both '{}' and '{}'",
                        binding, actions[0], actions[i]
                    ));
                }
            }
        }
        conflicts
    }

    fn bindings_with_labels(config: &KeybindConfig) -> Vec<(String, String)> {
        let mut out = Vec::new();
        if let Some(v) = &config.commands {
            out.push(("commands".to_string(), v.clone()));
        }
        if let Some(v) = &config.timeline {
            out.push(("timeline".to_string(), v.clone()));
        }
        if let Some(v) = &config.settings {
            out.push(("settings".to_string(), v.clone()));
        }
        if let Some(v) = &config.models {
            out.push(("models".to_string(), v.clone()));
        }
        if let Some(v) = &config.files {
            out.push(("files".to_string(), v.clone()));
        }
        if let Some(v) = &config.terminal {
            out.push(("terminal".to_string(), v.clone()));
        }
        if let Some(custom) = &config.custom {
            for (action, binding) in custom {
                out.push((format!("custom '{}'", action), binding.clone()));
            }
        }
        out
    }
}

#[derive(Debug, Clone, Deserialize, Serialize, Default)]
pub struct ThemeConfig {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub path: Option<std::path::PathBuf>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub scan_dirs: Option<Vec<String>>,
}

impl ThemeConfig {
    pub fn resolve_path(&self, config_dir: Option<&Path>) -> Option<PathBuf> {
        let configured = self.path.as_ref()?;
        let raw = configured.to_string_lossy();

        let home_dir = std::env::var("HOME")
            .ok()
            .map(PathBuf::from)
            .or_else(dirs_next::home_dir);

        let resolved = if raw == "~" {
            home_dir?
        } else if let Some(stripped) = raw.strip_prefix("~/") {
            home_dir?.join(stripped)
        } else if configured.is_relative() {
            config_dir
                .map(|dir| dir.join(configured))
                .unwrap_or_else(|| configured.clone())
        } else {
            configured.clone()
        };

        if resolved.exists() {
            Some(resolved)
        } else {
            None
        }
    }
}

#[derive(Debug, Clone, Deserialize, Serialize, Default)]
pub struct TuiPluginConfig {
    #[serde(
        default = "default_plugin_enabled",
        skip_serializing_if = "Option::is_none"
    )]
    pub plugin_enabled: Option<bool>,

    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub plugins: Option<std::collections::HashMap<String, bool>>,
}

fn default_plugin_enabled() -> Option<bool> {
    Some(true)
}

#[derive(Debug, Clone, Deserialize, Serialize, Default)]
pub struct TuiConfig {
    #[serde(
        rename = "$schema",
        alias = "$schema",
        skip_serializing_if = "Option::is_none"
    )]
    pub schema: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub scroll_speed: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub scroll_acceleration: Option<ScrollAccelerationConfig>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub diff_style: Option<DiffStyle>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub theme: Option<ThemeConfig>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub keybinds: Option<KeybindConfig>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub plugins: Option<TuiPluginConfig>,
}

#[derive(Debug, Clone, Serialize)]
pub struct ScrollAccelerationConfig {
    pub enabled: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub speed: Option<f32>,
}

impl<'de> Deserialize<'de> for ScrollAccelerationConfig {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        use serde::de::{self, Visitor};

        struct ScrollVisitor;

        impl<'de> Visitor<'de> for ScrollVisitor {
            type Value = ScrollAccelerationConfig;

            fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                formatter.write_str("a number (legacy) or { enabled: bool, speed?: f32 }")
            }

            fn visit_f64<E>(self, value: f64) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                Ok(ScrollAccelerationConfig {
                    enabled: true,
                    speed: Some(value as f32),
                })
            }

            fn visit_map<M>(self, mut map: M) -> Result<Self::Value, M::Error>
            where
                M: de::MapAccess<'de>,
            {
                let mut enabled = true;
                let mut speed = None;

                while let Some(key) = map.next_key::<String>()? {
                    match key.as_str() {
                        "enabled" => enabled = map.next_value()?,
                        "speed" => speed = map.next_value()?,
                        _ => {
                            let _: serde::de::IgnoredAny = map.next_value()?;
                        }
                    }
                }

                Ok(ScrollAccelerationConfig { enabled, speed })
            }
        }

        deserializer.deserialize_any(ScrollVisitor)
    }
}

impl Default for ScrollAccelerationConfig {
    fn default() -> Self {
        ScrollAccelerationConfig {
            enabled: true,
            speed: None,
        }
    }
}

impl From<f32> for ScrollAccelerationConfig {
    fn from(val: f32) -> Self {
        ScrollAccelerationConfig {
            enabled: true,
            speed: Some(val),
        }
    }
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

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "lowercase")]
#[derive(Default)]
pub enum LegacyProvider {
    #[default]
    Openai,
    Anthropic,
    Ollama,
}

impl Config {
    pub fn load(path: &PathBuf) -> Result<Self, ConfigError> {
        tracing::info!(path = %path.display(), "Loading configuration");

        let mut config = if !path.exists() {
            tracing::info!("Config file not found, using defaults");
            Config::default()
        } else {
            tracing::debug!(path = %path.display(), "Reading config file");
            let content = std::fs::read_to_string(path)?;
            let content = Self::substitute_variables(&content, path.parent())?;
            let ext = path.extension().and_then(|s| s.to_str());
            if ext == Some("json") || ext == Some("jsonc") || ext == Some("json5") {
                Config::parse_json_content(&content)?
            } else {
                return Err(ConfigError::Config(format!(
                    "Unsupported config format for {}. Expected .json, .jsonc, or .json5 extension.",
                    path.display()
                )));
            }
        };

        Self::log_schema_validation(&config);

        config.apply_env_overrides();
        tracing::info!("Configuration loaded successfully");
        Ok(config)
    }

    fn parse_json_content(content: &str) -> Result<Self, ConfigError> {
        let value = if let Ok(v) = serde_json::from_str::<serde_json::Value>(content) {
            v
        } else {
            let stripped = jsonc::strip_jsonc_comments(content);
            serde_json::from_str(&stripped).map_err(|e| ConfigError::Config(e.to_string()))?
        };

        let mut value = value;
        Self::check_and_migrate_deprecated_fields(&mut value);
        Self::expand_variables(&mut value).map_err(|e| ConfigError::Config(e.to_string()))?;
        serde_json::from_value(value).map_err(|e| ConfigError::Config(e.to_string()))
    }

    fn check_and_migrate_deprecated_fields(value: &mut serde_json::Value) {
        if let Some(obj) = value.as_object_mut() {
            let deprecated_fields = [
                ("mode", "Use 'agent[].permission' instead. Will be removed in v4.0."),
                ("tools", "Use 'permission' field instead. Will be removed in v4.0."),
                ("theme", "Theme configuration has moved to 'tui.json'. Will be removed from opencode.json in v4.0."),
                ("keybinds", "Keybinds configuration has moved to 'tui.json'. Will be removed from opencode.json in v4.0."),
            ];

            for (field, message) in deprecated_fields {
                if obj.contains_key(field) {
                    tracing::warn!(
                        "Deprecated config field '{}' detected: {}. \
                        See https://docs.opencode.ai/config/migration for migration guide.",
                        field,
                        message
                    );
                }
            }

            if let Some(agents) = obj.get("agent").and_then(|v| v.as_object()) {
                for (agent_name, agent_value) in agents {
                    if let Some(agent_obj) = agent_value.as_object() {
                        if agent_obj.contains_key("mode") {
                            tracing::warn!(
                                "Deprecated config field 'agent.{}.mode' detected: \
                                Use 'permission' field instead. Will be removed in v4.0. \
                                See https://docs.opencode.ai/config/migration for migration guide.",
                                agent_name
                            );
                        }
                    }
                }
            }

            Self::migrate_deprecated_tools_field(obj);
        }
    }

    fn migrate_deprecated_tools_field(obj: &mut serde_json::Map<String, serde_json::Value>) {
        if let Some(tools_value) = obj.remove("tools") {
            if let Some(tools_array) = tools_value.as_array() {
                let mut permission_obj = serde_json::Map::new();

                for tool_name in tools_array {
                    if let Some(name) = tool_name.as_str() {
                        let permission_value = match name {
                            "read" | "edit" | "glob" | "grep" | "list" | "bash" | "task"
                            | "lsp" | "skill" | "external_directory" => {
                                serde_json::json!({"action": "allow"})
                            }
                            _ => {
                                serde_json::json!({"action": "allow"})
                            }
                        };
                        permission_obj.insert(name.to_string(), permission_value);
                    }
                }

                if !permission_obj.is_empty() {
                    if let Some(existing_permission) = obj.get("permission") {
                        if let Some(existing_obj) = existing_permission.as_object() {
                            let mut merged = existing_obj.clone();
                            for (k, v) in permission_obj {
                                merged.entry(k).or_insert(v);
                            }
                            obj.insert("permission".to_string(), serde_json::Value::Object(merged));
                        }
                    } else {
                        obj.insert(
                            "permission".to_string(),
                            serde_json::Value::Object(permission_obj),
                        );
                    }
                }

                tracing::warn!(
                    "Deprecated 'tools' field has been migrated to 'permission'. \
                    Please update your config to use the new 'permission' structure. \
                    See https://docs.opencode.ai/config/migration for migration guide."
                );
            }
        }
    }

    pub fn substitute_variables(
        input: &str,
        config_dir: Option<&Path>,
    ) -> Result<String, ConfigError> {
        Self::substitute_variables_inner(input, config_dir, &mut HashSet::new())
    }

    fn substitute_variables_inner(
        input: &str,
        config_dir: Option<&Path>,
        expanding: &mut HashSet<String>,
    ) -> Result<String, ConfigError> {
        let mut result = input.to_string();

        for _ in 0..3 {
            let before = result.clone();
            result = Self::substitute_variables_single_pass(&result, config_dir, expanding)?;
            if result == before {
                break;
            }
        }

        Ok(result)
    }

    fn substitute_variables_single_pass(
        input: &str,
        config_dir: Option<&Path>,
        expanding: &mut HashSet<String>,
    ) -> Result<String, ConfigError> {
        let mut result = input.to_string();

        while let Some(start) = result.find("{env:") {
            if let Some(end) = result[start..].find('}') {
                let var_name = result[start + 5..start + end].to_string();

                if expanding.contains(&var_name) {
                    let chain: Vec<&str> = expanding
                        .iter()
                        .chain(std::iter::once(&var_name))
                        .map(|s| s.as_str())
                        .collect();
                    return Err(ConfigError::Config(format!(
                        "Circular environment variable reference detected: {{env:{}}}",
                        chain.join(" -> {env:")
                    )));
                }

                expanding.insert(var_name.clone());
                let replacement = std::env::var(&var_name).unwrap_or_default();
                let expansion_result =
                    Self::substitute_variables_inner(&replacement, config_dir, expanding);
                expanding.remove(&var_name);
                let expansion_result = expansion_result?;
                result = format!(
                    "{}{}{}",
                    &result[..start],
                    expansion_result,
                    &result[start + end + 1..]
                );
            } else {
                break;
            }
        }

        while let Some(start) = result.find("{file:") {
            if let Some(end) = result[start..].find('}') {
                let file_path = result[start + 6..start + end].to_string();

                if expanding.contains(&file_path) {
                    let chain: Vec<&str> = expanding
                        .iter()
                        .chain(std::iter::once(&file_path))
                        .map(|s| s.as_str())
                        .collect();
                    return Err(ConfigError::Config(format!(
                        "Circular file variable reference detected: {{file:{}}}",
                        chain.join(" -> {file:")
                    )));
                }

                expanding.insert(file_path.clone());
                let replacement = match Self::resolve_file_variable_path(&file_path, config_dir) {
                    Some(path) => {
                        let path_str = path.to_string_lossy().to_string();
                        let content = std::fs::read_to_string(&path)
                            .unwrap_or_else(|_| format!("{{file:{}}}", &file_path));
                        let expanded_content =
                            Self::substitute_variables_inner(&content, config_dir, expanding)?;
                        (path_str, expanded_content)
                    }
                    _ => (file_path.clone(), String::new()),
                };
                expanding.remove(&file_path);

                result = format!(
                    "{}{}{}",
                    &result[..start],
                    replacement.1,
                    &result[start + end + 1..]
                );
            } else {
                break;
            }
        }

        while let Some(start) = result.find("{keychain:") {
            if let Some(end) = result[start..].find('}') {
                let secret_name = result[start + 10..start + end].to_string();
                let replacement = resolve_keychain_secret(&secret_name)
                    .unwrap_or_else(|| format!("{{keychain:{}}}", secret_name));
                result = format!(
                    "{}{}{}",
                    &result[..start],
                    replacement,
                    &result[start + end + 1..]
                );
            } else {
                break;
            }
        }

        Ok(result)
    }

    pub fn contains_keychain_reference(s: &str) -> bool {
        s.contains("{keychain:")
    }

    pub fn redact_keychain_references(s: &str) -> String {
        let mut result = s.to_string();
        while let Some(start) = result.find("{keychain:") {
            if let Some(end) = result[start..].find('}') {
                let secret_name = &result[start + 10..start + end];
                result = format!(
                    "{}[keychain:{}]{}",
                    &result[..start],
                    secret_name,
                    &result[start + end + 1..]
                );
            } else {
                break;
            }
        }
        result
    }

    pub fn expand_variables(value: &mut serde_json::Value) -> Result<(), ConfigError> {
        let config_values = Self::collect_config_values(value);
        Self::expand_variables_inner(value, &config_values, &mut Vec::new())
    }

    fn collect_config_values(
        value: &serde_json::Value,
    ) -> std::collections::HashMap<String, serde_json::Value> {
        let mut map = std::collections::HashMap::new();
        Self::collect_values_recursive(value, String::new(), &mut map);
        map
    }

    fn collect_values_recursive(
        value: &serde_json::Value,
        prefix: String,
        map: &mut std::collections::HashMap<String, serde_json::Value>,
    ) {
        match value {
            serde_json::Value::Object(obj) => {
                for (key, val) in obj {
                    let new_prefix = if prefix.is_empty() {
                        key.clone()
                    } else {
                        format!("{}.{}", prefix, key)
                    };
                    Self::collect_values_recursive(val, new_prefix, map);
                }
            }
            serde_json::Value::Array(arr) => {
                for (i, val) in arr.iter().enumerate() {
                    let new_prefix = format!("{}[{}]", prefix, i);
                    Self::collect_values_recursive(val, new_prefix, map);
                }
            }
            _ => {
                map.insert(prefix, value.clone());
            }
        }
    }

    fn expand_variables_inner(
        value: &mut serde_json::Value,
        config_values: &std::collections::HashMap<String, serde_json::Value>,
        path: &mut Vec<String>,
    ) -> Result<(), ConfigError> {
        match value {
            serde_json::Value::String(s) => Self::expand_string_variable(s, config_values, path),
            serde_json::Value::Object(obj) => {
                for (_, v) in obj {
                    Self::expand_variables_inner(v, config_values, path)?;
                }
                Ok(())
            }
            serde_json::Value::Array(arr) => {
                for v in arr {
                    Self::expand_variables_inner(v, config_values, path)?;
                }
                Ok(())
            }
            _ => Ok(()),
        }
    }

    fn expand_string_variable(
        s: &mut String,
        config_values: &std::collections::HashMap<String, serde_json::Value>,
        path: &mut Vec<String>,
    ) -> Result<(), ConfigError> {
        while let Some(start) = s.find("${") {
            if let Some(end) = s[start..].find('}') {
                let var_name = s[start + 2..start + end].to_string();

                if let Some(circular_start) = path.iter().position(|v| v == &var_name) {
                    let chain: Vec<&str> = path[circular_start..]
                        .iter()
                        .chain(std::iter::once(&var_name))
                        .map(|s| s.as_str())
                        .collect();
                    return Err(ConfigError::Config(format!(
                        "Circular config variable reference detected: ${}",
                        chain.join(" -> $")
                    )));
                }

                let var_value = config_values.get(&var_name).ok_or_else(|| {
                    ConfigError::Config(format!("Undefined config variable: ${}", var_name))
                })?;

                path.push(var_name.clone());

                let replacement = match var_value {
                    serde_json::Value::String(v) => v.clone(),
                    serde_json::Value::Number(n) => n.to_string(),
                    serde_json::Value::Bool(b) => b.to_string(),
                    serde_json::Value::Null => String::new(),
                    _ => {
                        path.pop();
                        let type_name = match var_value {
                            serde_json::Value::Object(_) => "object",
                            serde_json::Value::Array(_) => "array",
                            serde_json::Value::Null => "null",
                            _ => "unknown",
                        };
                        return Err(ConfigError::Config(format!(
                            "Config variable ${} does not resolve to a string (got {})",
                            var_name, type_name
                        )));
                    }
                };

                let end_pos = start + end + 1;
                s.replace_range(start..end_pos, &replacement);

                Self::expand_string_variable(s, config_values, path)?;
                path.pop();
            } else {
                break;
            }
        }
        Ok(())
    }

    fn resolve_file_variable_path(file_path: &str, config_dir: Option<&Path>) -> Option<PathBuf> {
        if file_path.starts_with('~') {
            let home = dirs::home_dir().or_else(|| std::env::var("HOME").ok().map(PathBuf::from));
            let Some(home) = home else {
                tracing::error!(
                    "Failed to expand home directory for file variable: {}",
                    file_path
                );
                return None;
            };

            if file_path == "~" {
                return Some(home);
            }

            if let Some(stripped) = file_path.strip_prefix("~/") {
                return Some(home.join(stripped));
            }

            tracing::error!(
                "Unsupported home-relative file variable path: {}",
                file_path
            );
            return None;
        }

        let path = Path::new(file_path);
        if path.is_absolute() {
            return Some(path.to_path_buf());
        }

        if let Some(base) = config_dir {
            return Some(base.join(path));
        }

        tracing::warn!(
            "Relative file variable path without config directory context: {}",
            file_path
        );

        match std::env::current_dir() {
            Ok(cwd) => Some(cwd.join(path)),
            Err(err) => {
                tracing::error!(
                    "Failed to resolve current directory for file variable {}: {}",
                    file_path,
                    err
                );
                None
            }
        }
    }

    fn preferred_config_path(config_root: &Path) -> PathBuf {
        let json = config_root.join("config.json");
        if json.exists() {
            return json;
        }

        let jsonc = config_root.join("config.jsonc");
        if jsonc.exists() {
            return jsonc;
        }

        config_root.join("config.json")
    }

    fn warn_legacy_config_dir_if_exists() {
        if let Some(home) =
            dirs::home_dir().or_else(|| std::env::var("HOME").ok().map(PathBuf::from))
        {
            let legacy_dir = home.join(".config").join("opencode-rs");
            if legacy_dir.exists() {
                tracing::warn!(
                    "Legacy config directory detected at {}. Please migrate to ~/.config/opencode/",
                    legacy_dir.display()
                );
            }
        }
    }

    pub fn config_path() -> PathBuf {
        if let Ok(config_dir) = std::env::var("OPENCODE_CONFIG_DIR") {
            return Self::preferred_config_path(Path::new(&config_dir));
        }

        directories::ProjectDirs::from("ai", "opencode", "opencode")
            .map(|dirs| Self::preferred_config_path(dirs.config_dir()))
            .unwrap_or_else(|| PathBuf::from("~/.config/opencode/config.json"))
    }

    fn default_tui_config_path() -> Option<PathBuf> {
        dirs::home_dir().map(|home| home.join(".config/opencode/tui.json"))
    }

    fn expand_tilde_path(path: &str) -> PathBuf {
        let home = dirs::home_dir().or_else(|| std::env::var("HOME").ok().map(PathBuf::from));

        if path == "~" {
            return home.unwrap_or_else(|| PathBuf::from(path));
        }

        if let Some(stripped) = path.strip_prefix("~/") {
            return home
                .map(|h| h.join(stripped))
                .unwrap_or_else(|| PathBuf::from(path));
        }

        PathBuf::from(path)
    }

    fn load_tui_config_path_from_env() -> Option<PathBuf> {
        std::env::var("OPENCODE_TUI_CONFIG")
            .ok()
            .map(|p| p.trim().to_string())
            .filter(|p| !p.is_empty())
            .map(|p| Self::expand_tilde_path(&p))
    }

    pub fn load_tui_config_path() -> Option<PathBuf> {
        Self::load_tui_config_path_from_env().or_else(Self::default_tui_config_path)
    }

    fn find_project_config_directory() -> Option<PathBuf> {
        let cwd = std::env::current_dir().ok()?;

        for ancestor in cwd.ancestors() {
            for ext in ["json", "json5", "jsonc"] {
                let project_config = ancestor.join(format!("opencode.{}", ext));
                if project_config.exists() {
                    return project_config.parent().map(PathBuf::from);
                }

                let opencode_dir = ancestor.join(".opencode").join(format!("config.{}", ext));
                if opencode_dir.exists() {
                    return opencode_dir.parent().map(PathBuf::from);
                }
            }
        }

        None
    }

    pub fn validate_tui_config_no_runtime_fields(value: &Value) -> Vec<String> {
        let Some(obj) = value.as_object() else {
            return Vec::new();
        };

        const ALLOWED_TUI_FIELDS: &[&str] = &[
            "scroll_speed",
            "scrollSpeed",
            "scroll_acceleration",
            "scrollAcceleration",
            "diff_style",
            "diffStyle",
            "theme",
            "keybinds",
            "plugin_enabled",
            "plugins",
        ];

        obj.keys()
            .filter(|key| !ALLOWED_TUI_FIELDS.contains(&key.as_str()))
            .cloned()
            .collect()
    }

    pub fn validate_runtime_no_tui_fields(value: &Value) -> Vec<String> {
        let Some(obj) = value.as_object() else {
            return Vec::new();
        };

        const TUI_FIELDS: &[&str] = &[
            "tui",
            "scroll_speed",
            "scrollSpeed",
            "scroll_acceleration",
            "scrollAcceleration",
            "diff_style",
            "diffStyle",
            "plugin_enabled",
            "plugins",
        ];

        obj.keys()
            .filter(|key| TUI_FIELDS.contains(&key.as_str()))
            .cloned()
            .collect()
    }

    fn parse_tui_config_file(path: &Path) -> Result<Option<TuiConfig>, ConfigError> {
        if !path.exists() {
            return Ok(None);
        }

        let content = std::fs::read_to_string(path)?;
        let value = parse_jsonc(&content).map_err(|e| {
            ConfigError::Config(format!(
                "Failed to parse config file {}: {}. Ensure valid JSON/JSONC syntax.",
                path.display(),
                e
            ))
        })?;

        let invalid_runtime_fields = Self::validate_tui_config_no_runtime_fields(&value);
        if !invalid_runtime_fields.is_empty() {
            return Err(ConfigError::Config(format!(
                "TUI config file {} contains runtime-specific fields that belong in opencode.json: {}. \
                Please move these fields to opencode.json or remove them from tui.json.",
                path.display(),
                invalid_runtime_fields.join(", ")
            )));
        }

        let schema_errors = schema::validate_tui_schema(&value);
        if !schema_errors.is_empty() {
            return Err(ConfigError::Config(format!(
                "Invalid TUI config {}: {}",
                path.display(),
                schema_errors.join("; ")
            )));
        }

        let config = serde_json::from_value::<TuiConfig>(value).map_err(|e| {
            ConfigError::Config(format!("Invalid TUI config {}: {}", path.display(), e))
        })?;

        Ok(Some(config))
    }

    pub fn load_tui_config() -> Result<TuiConfig, ConfigError> {
        let mut paths: Vec<PathBuf> = Vec::new();

        if let Some(primary) = Self::load_tui_config_path() {
            paths.push(primary);
        }

        if let Some(home) = Self::default_tui_config_path() {
            if !paths.contains(&home) {
                paths.push(home);
            }
        }

        if let Some(project_dir) = Self::find_project_config_directory() {
            let project_tui = project_dir.join("tui.json");
            if !paths.contains(&project_tui) {
                paths.push(project_tui);
            }
        }

        let mut merged = TuiConfig::default();
        for path in paths {
            if let Some(cfg) = Self::parse_tui_config_file(&path)? {
                let base =
                    serde_json::to_value(&merged).unwrap_or(Value::Object(serde_json::Map::new()));
                let override_val =
                    serde_json::to_value(&cfg).unwrap_or(Value::Object(serde_json::Map::new()));
                let merged_json = merge::deep_merge(&base, &override_val);
                merged = serde_json::from_value(merged_json).unwrap_or_default();
            }
        }

        Ok(merged)
    }

    fn validate_runtime_tui_fields(path: &Path) -> Result<(), Vec<String>> {
        let content = std::fs::read_to_string(path).map_err(|e| {
            vec![format!(
                "Failed to read config file {}: {}",
                path.display(),
                e
            )]
        })?;
        let value = parse_jsonc(&content).map_err(|e| {
            vec![format!(
                "Failed to parse config file {}: {}",
                path.display(),
                e
            )]
        })?;
        let detected = Self::validate_runtime_no_tui_fields(&value);
        if !detected.is_empty() {
            return Err(detected);
        }
        Ok(())
    }

    pub async fn load_multi(
        cli_overrides: Option<&CliOverrideConfig>,
    ) -> Result<Self, ConfigError> {
        Self::warn_legacy_config_dir_if_exists();
        let mut configs: Vec<(String, Config)> = Vec::new();

        if let Ok(remote_url) = std::env::var("OPENCODE_REMOTE_CONFIG") {
            if let Ok(content) = Self::fetch_remote_config_with_fallback(&remote_url).await {
                if let Ok(config) = Self::parse_config_content(&content, "json") {
                    configs.push(("remote".to_string(), config));
                }
            }
        }

        if let Ok(domain) = std::env::var("OPENCODE_REMOTE_CONFIG_DOMAIN") {
            if !domain.trim().is_empty() {
                let url = Self::build_remote_url(&domain);
                if let Ok(content) = Self::fetch_remote_config_with_fallback(&url).await {
                    if let Ok(config) = Self::parse_config_content(&content, "json") {
                        configs.push(("remote-auto-discover".to_string(), config));
                    } else {
                        tracing::warn!(
                            "Remote config auto-discovery: failed to parse config from {}",
                            url
                        );
                    }
                } else {
                    tracing::warn!("Remote config auto-discovery: failed to fetch from {}", url);
                }
            }
        }

        if let Ok(cwd) = std::env::current_dir() {
            for ancestor in cwd.ancestors() {
                for ext in ["json", "json5", "jsonc"] {
                    let project_config = ancestor.join(format!("opencode.{}", ext));
                    if project_config.exists() {
                        if let Ok(content) = std::fs::read_to_string(&project_config) {
                            if let Ok(value) = serde_json::from_str::<serde_json::Value>(&content) {
                                if let Some(domain) = value
                                    .get("enterprise")
                                    .and_then(|e| e.get("remoteConfigDomain"))
                                    .and_then(|d| d.as_str())
                                {
                                    let url = Self::build_remote_url(domain);
                                    if let Ok(content) =
                                        Self::fetch_remote_config_with_fallback(&url).await
                                    {
                                        if let Ok(config) =
                                            Self::parse_config_content(&content, "json")
                                        {
                                            configs.push(("remote-enterprise".to_string(), config));
                                        }
                                    }
                                }
                            }
                        }
                        break;
                    }
                }
            }
        }

        if let Ok(config_path) = std::env::var("OPENCODE_CONFIG") {
            let path = PathBuf::from(config_path);
            if path.exists() {
                if let Err(tui_fields) = Self::validate_runtime_tui_fields(&path) {
                    return Err(ConfigError::Config(format!(
                        "Config file {} contains TUI-specific fields that belong in tui.json: {}. \
                        Please move these fields to tui.json or remove them from opencode.json.",
                        path.display(),
                        tui_fields.join(", ")
                    )));
                }
                let config = Self::load(&path)?;
                configs.push(("env-path".to_string(), config));
            }
        }

        if let Ok(content) = std::env::var("OPENCODE_CONFIG_CONTENT") {
            if let Ok(content) = Self::substitute_variables(&content, None) {
                if let Ok(config) = Self::parse_config_content(&content, "json") {
                    configs.push(("env-content".to_string(), config));
                }
            }
        }

        let global_path = Self::config_path();
        if global_path.exists() {
            if let Err(tui_fields) = Self::validate_runtime_tui_fields(&global_path) {
                return Err(ConfigError::Config(format!(
                    "Config file {} contains TUI-specific fields that belong in tui.json: {}. \
                    Please move these fields to tui.json or remove them from opencode.json.",
                    global_path.display(),
                    tui_fields.join(", ")
                )));
            }
            let config = Self::load(&global_path)?;
            configs.push(("global".to_string(), config));
        }

        if let Ok(cwd) = std::env::current_dir() {
            for ancestor in cwd.ancestors() {
                for ext in &["json", "json5", "jsonc"] {
                    let project_config = ancestor.join(format!("opencode.{}", ext));
                    if project_config.exists() {
                        if let Err(tui_fields) = Self::validate_runtime_tui_fields(&project_config)
                        {
                            return Err(ConfigError::Config(format!(
                                "Config file {} contains TUI-specific fields that belong in tui.json: {}. \
                                Please move these fields to tui.json or remove them from opencode.json.",
                                project_config.display(),
                                tui_fields.join(", ")
                            )));
                        }
                        let config = Self::load(&project_config)?;
                        configs.push(("project".to_string(), config));
                        break;
                    }
                }
                for ext in &["json", "json5", "jsonc"] {
                    let opencode_dir = ancestor.join(".opencode").join(format!("config.{}", ext));
                    if opencode_dir.exists() {
                        if let Err(tui_fields) = Self::validate_runtime_tui_fields(&opencode_dir) {
                            return Err(ConfigError::Config(format!(
                                "Config file {} contains TUI-specific fields that belong in tui.json: {}. \
                                Please move these fields to tui.json or remove them from opencode.json.",
                                opencode_dir.display(),
                                tui_fields.join(", ")
                            )));
                        }
                        let config = Self::load(&opencode_dir)?;
                        configs.push((".opencode".to_string(), config));
                        break;
                    }
                }
            }
        }

        let mut result = Config::default();
        for (_, config) in configs {
            result = Self::merge_configs(result, config);
        }

        Self::merge_opencode_directory_into_config(&mut result);

        let file_tui = Self::load_tui_config()?;
        let base = serde_json::to_value(result.tui.clone().unwrap_or_default())
            .unwrap_or(Value::Object(serde_json::Map::new()));
        let override_val =
            serde_json::to_value(&file_tui).unwrap_or(Value::Object(serde_json::Map::new()));
        let merged_tui = merge::deep_merge(&base, &override_val);
        result.tui = Some(serde_json::from_value(merged_tui).unwrap_or_default());

        result.apply_env_overrides();

        if let Some(cli_overrides) = cli_overrides {
            result.apply_cli_overrides(
                cli_overrides.model.clone(),
                cli_overrides.provider.clone(),
                cli_overrides.temperature,
                cli_overrides.max_tokens,
                cli_overrides.default_agent.clone(),
            );
        }

        Ok(result)
    }

    async fn fetch_remote_config(url: &str) -> Result<String, ConfigError> {
        let cache_dir = Self::remote_cache_dir();
        Self::fetch_remote_config_from_cache_dir(url, &cache_dir).await
    }

    async fn fetch_remote_config_from_cache_dir(
        url: &str,
        cache_dir: &Path,
    ) -> Result<String, ConfigError> {
        let cached = load_cache(url, cache_dir);

        let client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(10))
            .build()
            .map_err(|e| ConfigError::Config(e.to_string()))?;

        let mut request = client.get(url).header("Accept", "application/json");

        if let Ok(token) = std::env::var("OPENCODE_REMOTE_CONFIG_TOKEN") {
            if !token.trim().is_empty() {
                request = request.header("Authorization", format!("Bearer {}", token.trim()));
            }
        }

        if let Some(cache) = cached.as_ref() {
            if let Some(etag) = &cache.etag {
                request = request.header(IF_NONE_MATCH, etag);
            }
            if let Some(last_modified) = &cache.last_modified {
                request = request.header(IF_MODIFIED_SINCE, last_modified);
            }
        }

        let response = request
            .send()
            .await
            .map_err(|e| ConfigError::Config(e.to_string()))?;

        if response.status() == reqwest::StatusCode::NOT_MODIFIED {
            if let Some(cache) = cached {
                return Ok(cache.content);
            }

            return Err(ConfigError::Config(
                "Remote config returned 304 but no cache is available".to_string(),
            ));
        }

        if !response.status().is_success() {
            return Err(ConfigError::Config(format!(
                "Remote config request failed with status {}",
                response.status()
            )));
        }

        let etag = response
            .headers()
            .get(ETAG)
            .and_then(|value| value.to_str().ok())
            .map(|value| value.to_string());

        let last_modified = response
            .headers()
            .get(LAST_MODIFIED)
            .and_then(|value| value.to_str().ok())
            .map(|value| value.to_string());

        let expires_at = Self::parse_cache_expiration(response.headers(), Utc::now());

        let content = response
            .text()
            .await
            .map_err(|e| ConfigError::Config(e.to_string()))?;

        let content_hash = Self::compute_content_hash(&content);
        if let Some(cache) = cached.as_ref() {
            if !Self::verify_integrity(&content, &cache.content_hash) {
                tracing::warn!(
                    "Remote config hash mismatch for {} (previous={}, current={})",
                    url,
                    cache.content_hash,
                    content_hash
                );
            }
        }

        let cache_entry = RemoteConfigCache {
            url: url.to_string(),
            content: content.clone(),
            etag,
            last_modified,
            fetched_at: Utc::now(),
            expires_at,
            content_hash,
        };

        if let Err(err) = save_cache(&cache_entry, cache_dir) {
            tracing::warn!("Failed to persist remote config cache for {}: {}", url, err);
        }

        Ok(content)
    }

    async fn fetch_remote_config_with_fallback(url: &str) -> Result<String, ConfigError> {
        match Self::fetch_remote_config(url).await {
            Ok(content) => Ok(content),
            Err(err) => {
                let cache_dir = Self::remote_cache_dir();
                if let Some(cache) = load_cache(url, &cache_dir) {
                    tracing::warn!(
                        "Remote config fetch failed for {} ({}); using cached content{}",
                        url,
                        err,
                        if cache.is_expired() { " (expired)" } else { "" }
                    );
                    return Ok(cache.content);
                }

                Err(err)
            }
        }
    }

    fn remote_cache_dir() -> PathBuf {
        let config_path = Self::config_path();
        config_path
            .parent()
            .map(Path::to_path_buf)
            .unwrap_or_else(|| PathBuf::from(".opencode"))
    }

    fn parse_cache_expiration(
        headers: &reqwest::header::HeaderMap,
        fetched_at: DateTime<Utc>,
    ) -> Option<DateTime<Utc>> {
        let header = headers
            .get(CACHE_CONTROL)
            .and_then(|value| value.to_str().ok())?;

        for directive in header.split(',') {
            let directive = directive.trim();
            if let Some(max_age) = directive.strip_prefix("max-age=") {
                if let Ok(seconds) = max_age.parse::<i64>() {
                    return Some(fetched_at + Duration::seconds(seconds.max(0)));
                }
            }
        }

        None
    }

    fn compute_content_hash(content: &str) -> String {
        let mut hasher = Sha256::new();
        hasher.update(content.as_bytes());
        format!("{:x}", hasher.finalize())
    }

    fn verify_integrity(content: &str, expected_hash: &str) -> bool {
        Self::compute_content_hash(content) == expected_hash
    }

    pub fn build_remote_url(domain: &str) -> String {
        let domain = domain.trim_end_matches('/');
        format!("{}/.well-known/opencode", domain)
    }

    fn parse_config_content(content: &str, format: &str) -> Result<Self, ConfigError> {
        if format == "json" || format == "jsonc" {
            if let Ok(config) = serde_json::from_str::<Config>(content) {
                Self::log_schema_validation(&config);
                return Ok(config);
            }
            let stripped = jsonc::strip_jsonc_comments(content);
            let config =
                serde_json::from_str(&stripped).map_err(|e| ConfigError::Config(e.to_string()))?;
            Self::log_schema_validation(&config);
            Ok(config)
        } else {
            Err(ConfigError::Config(format!(
                "Unsupported config format: {}. Expected json or jsonc.",
                format
            )))
        }
    }

    fn log_schema_validation(config: &Config) {
        if let Some(schema_url) = config.schema.as_deref() {
            let validation = config.validate_json_schema(Some(schema_url));
            if !validation.valid {
                for error in validation.errors {
                    tracing::warn!(
                        "config schema validation error at {}: {}",
                        error.field,
                        error.message
                    );
                }
            }
        }
    }

    fn merge_opencode_directory_into_config(config: &mut Config) {
        let Ok(cwd) = std::env::current_dir() else {
            return;
        };

        let mut found_opencode_dir = None;
        for ancestor in cwd.ancestors() {
            let project_opencode = ancestor.join(".opencode");
            if project_opencode.exists() && project_opencode.is_dir() {
                found_opencode_dir = Some(project_opencode);
                break;
            }
        }

        let Some(opencode_path) = found_opencode_dir else {
            return;
        };

        let scanner = directory_scanner::DirectoryScanner::new();
        let scan = scanner.scan_all(&opencode_path);

        let agent_count = scan.agents.len();
        let command_count = scan.commands.len();
        let mode_count = scan.modes.len();
        let skill_count = scan.skills.len();
        let tool_count = scan.tools.len();
        let theme_count = scan.themes.len();
        let plugin_count = scan.plugins.len();

        if agent_count > 0 {
            let agents = config.agent.get_or_insert_with(AgentMapConfig::default);
            for agent_info in scan.agents {
                agents
                    .agents
                    .entry(agent_info.name)
                    .or_insert_with(|| AgentConfig {
                        prompt: Some(agent_info.content),
                        ..Default::default()
                    });
            }
        }

        if command_count > 0 {
            let commands = config.command.get_or_insert_with(HashMap::new);
            for cmd_info in scan.commands {
                let name = cmd_info.name.clone();
                let template = format!(
                    "# Command from {}\n{}",
                    cmd_info.path.display(),
                    cmd_info.content
                );
                let description = format!("Loaded from .opencode/commands/{name}");
                commands.entry(name).or_insert_with(|| CommandConfig {
                    template,
                    description: Some(description),
                    ..Default::default()
                });
            }
        }

        if skill_count > 0 {
            let skills = config.skills.get_or_insert_with(SkillsConfig::default);
            let paths = skills.paths.get_or_insert_with(Vec::new);
            for skill_info in scan.skills {
                if let Some(parent) = skill_info.path.parent() {
                    if let Some(path_str) = parent.to_str() {
                        if !paths.iter().any(|p| p == path_str) {
                            paths.push(path_str.to_string());
                        }
                    }
                }
            }
        }

        if plugin_count > 0 {
            let plugins = config.plugin.get_or_insert_with(Vec::new);
            for plugin_info in scan.plugins {
                if let Some(path_str) = plugin_info.path.to_str() {
                    if !plugins.iter().any(|p| p == path_str) {
                        plugins.push(path_str.to_string());
                    }
                }
            }
        }

        if agent_count > 0 || mode_count > 0 {
            tracing::info!(
                "Loaded .opencode/ directory: {agent_count} agents, {command_count} commands, {mode_count} modes, {skill_count} skills, {tool_count} tools, {theme_count} themes, {plugin_count} plugins"
            );
        }
    }

    fn merge_configs(base: Config, override_config: Config) -> Config {
        merge::merge_configs(&base, &override_config)
    }

    fn apply_env_overrides(&mut self) {
        if let Ok(provider) = std::env::var("OPENCODE_PROVIDER") {
            let provider_config = ProviderConfig {
                id: Some(provider.to_lowercase()),
                ..Default::default()
            };
            let mut providers = self.provider.clone().unwrap_or_default();
            providers.insert(provider.to_lowercase(), provider_config);
            self.provider = Some(providers);
        }

        if let Ok(model) = std::env::var("OPENCODE_MODEL") {
            self.model = Some(model);
        }

        if let Ok(api_key) = std::env::var("OPENCODE_API_KEY") {
            self.api_key = Some(api_key);
        }

        if let Ok(temp) = std::env::var("OPENCODE_TEMPERATURE") {
            if let Ok(t) = temp.parse() {
                self.temperature = Some(t);
            }
        }

        if let Ok(tokens) = std::env::var("OPENCODE_MAX_TOKENS") {
            if let Ok(t) = tokens.parse() {
                self.max_tokens = Some(t);
            }
        }

        if let Ok(small_model) = std::env::var("OPENCODE_SMALL_MODEL") {
            self.small_model = Some(small_model);
        }

        if let Ok(username) = std::env::var("OPENCODE_USERNAME") {
            self.username = Some(username);
        }

        if let Ok(default_agent) = std::env::var("OPENCODE_DEFAULT_AGENT") {
            self.default_agent = Some(default_agent);
        }

        if let Ok(log_level) = std::env::var("OPENCODE_LOG_LEVEL") {
            self.log_level = match log_level.to_lowercase().as_str() {
                "trace" => Some(LogLevel::Trace),
                "debug" => Some(LogLevel::Debug),
                "info" => Some(LogLevel::Info),
                "warn" => Some(LogLevel::Warn),
                "error" => Some(LogLevel::Error),
                _ => self.log_level.clone(),
            };
        }

        if let Ok(exp_flags) = std::env::var("OPENCODE_EXPERIMENTAL") {
            let mut exp = self.experimental.clone().unwrap_or_default();
            for flag in exp_flags.split(',') {
                match flag.trim() {
                    "batch_tool" => exp.batch_tool = Some(true),
                    "open_telemetry" => exp.open_telemetry = Some(true),
                    "continue_loop_on_deny" => exp.continue_loop_on_deny = Some(true),
                    "disable_paste_summary" => exp.disable_paste_summary = Some(true),
                    _ => {}
                }
            }
            self.experimental = Some(exp);
        }

        let provider_api_keys = [
            ("openai", "OPENAI_API_KEY"),
            ("anthropic", "ANTHROPIC_API_KEY"),
            ("google", "GOOGLE_API_KEY"),
            ("azure", "AZURE_OPENAI_API_KEY"),
            ("ollama", "OLLAMA_HOST"),
            ("aws", "AWS_ACCESS_KEY_ID"),
            ("cohere", "COHERE_API_KEY"),
            ("mistral", "MISTRAL_API_KEY"),
            ("perplexity", "PERPLEXITY_API_KEY"),
            ("groq", "GROQ_API_KEY"),
        ];

        let mut providers = self.provider.clone().unwrap_or_default();
        for (provider_id, env_var) in provider_api_keys {
            if let Ok(api_key) = std::env::var(env_var) {
                let config =
                    providers
                        .entry(provider_id.to_string())
                        .or_insert_with(|| ProviderConfig {
                            id: Some(provider_id.to_string()),
                            ..Default::default()
                        });
                let mut opts = config.options.clone().unwrap_or_default();
                opts.api_key = Some(api_key);
                config.options = Some(opts);
            }
        }
        if !providers.is_empty() {
            self.provider = Some(providers);
        }
    }

    pub fn apply_cli_overrides(
        &mut self,
        model: Option<String>,
        provider: Option<String>,
        temperature: Option<f32>,
        max_tokens: Option<u32>,
        default_agent: Option<String>,
    ) {
        if let Some(m) = model {
            self.model = Some(m);
        }

        if let Some(p) = provider {
            let provider_config = ProviderConfig {
                id: Some(p.to_lowercase()),
                ..Default::default()
            };
            let mut providers = self.provider.clone().unwrap_or_default();
            providers.insert(p.to_lowercase(), provider_config);
            self.provider = Some(providers);
        }

        if let Some(t) = temperature {
            self.temperature = Some(t);
        }

        if let Some(t) = max_tokens {
            self.max_tokens = Some(t);
        }

        if let Some(da) = default_agent {
            self.default_agent = Some(da);
        }
    }

    pub fn get_provider(&self, provider_id: &str) -> Option<&ProviderConfig> {
        self.provider.as_ref().and_then(|p| p.get(provider_id))
    }

    pub fn get_provider_filter(&self) -> Option<(Vec<String>, Vec<String>)> {
        if self.disabled_providers.is_none() && self.enabled_providers.is_none() {
            return None;
        }

        Some((
            self.disabled_providers.clone().unwrap_or_default(),
            self.enabled_providers.clone().unwrap_or_default(),
        ))
    }

    pub fn is_provider_enabled(&self, provider_id: &str) -> bool {
        let Some((disabled, enabled)) = self.get_provider_filter() else {
            return true;
        };

        if disabled
            .iter()
            .any(|provider| provider.eq_ignore_ascii_case(provider_id))
        {
            return false;
        }

        if enabled.is_empty() {
            return true;
        }

        enabled
            .iter()
            .any(|provider| provider.eq_ignore_ascii_case(provider_id))
    }

    pub fn batch_tool_enabled(&self) -> bool {
        self.experimental
            .as_ref()
            .and_then(|e| e.batch_tool)
            .unwrap_or(false)
    }

    pub fn open_telemetry_enabled(&self) -> bool {
        self.experimental
            .as_ref()
            .and_then(|e| e.open_telemetry)
            .unwrap_or(false)
    }

    pub fn get_disabled_tools(&self) -> HashSet<String> {
        if let Some(permission) = &self.permission {
            let mut disabled = HashSet::new();

            fn extract_action(
                rule: &PermissionRule,
                field_name: &str,
                disabled: &mut HashSet<String>,
            ) {
                match rule {
                    PermissionRule::Action(PermissionAction::Deny) => {
                        disabled.insert(field_name.to_string());
                    }
                    PermissionRule::Action(PermissionAction::Allow)
                    | PermissionRule::Action(PermissionAction::Ask) => {}
                    PermissionRule::Object(obj) => {
                        for (name, action) in obj {
                            if matches!(action, PermissionAction::Deny) {
                                disabled.insert(name.clone());
                            }
                        }
                    }
                }
            }

            if let Some(read) = &permission.read {
                extract_action(read, "read", &mut disabled);
            }
            if let Some(edit) = &permission.edit {
                extract_action(edit, "edit", &mut disabled);
            }
            if let Some(glob) = &permission.glob {
                extract_action(glob, "glob", &mut disabled);
            }
            if let Some(grep) = &permission.grep {
                extract_action(grep, "grep", &mut disabled);
            }
            if let Some(list) = &permission.list {
                extract_action(list, "list", &mut disabled);
            }
            if let Some(bash) = &permission.bash {
                extract_action(bash, "bash", &mut disabled);
            }
            if let Some(task) = &permission.task {
                extract_action(task, "task", &mut disabled);
            }
            if let Some(external_directory) = &permission.external_directory {
                extract_action(external_directory, "external_directory", &mut disabled);
            }
            if let Some(todowrite) = &permission.todowrite {
                if matches!(todowrite, PermissionAction::Deny) {
                    disabled.insert("todowrite".to_string());
                }
            }
            if let Some(question) = &permission.question {
                if matches!(question, PermissionAction::Deny) {
                    disabled.insert("question".to_string());
                }
            }
            if let Some(webfetch) = &permission.webfetch {
                if matches!(webfetch, PermissionAction::Deny) {
                    disabled.insert("webfetch".to_string());
                }
            }
            if let Some(websearch) = &permission.websearch {
                if matches!(websearch, PermissionAction::Deny) {
                    disabled.insert("websearch".to_string());
                }
            }
            if let Some(codesearch) = &permission.codesearch {
                if matches!(codesearch, PermissionAction::Deny) {
                    disabled.insert("codesearch".to_string());
                }
            }
            if let Some(lsp) = &permission.lsp {
                extract_action(lsp, "lsp", &mut disabled);
            }
            if let Some(doom_loop) = &permission.doom_loop {
                if matches!(doom_loop, PermissionAction::Deny) {
                    disabled.insert("doom_loop".to_string());
                }
            }
            if let Some(skill) = &permission.skill {
                extract_action(skill, "skill", &mut disabled);
            }

            if let Some(extra) = &permission.extra {
                for (name, rule) in extra {
                    extract_action(rule, name, &mut disabled);
                }
            }

            return disabled;
        }

        HashSet::new()
    }

    pub fn validate(&self) -> Vec<ValidationError> {
        let mut errors = Vec::new();

        if let Some(model) = &self.model {
            if !model.contains('/') {
                errors.push(ValidationError {
                    field: "model".to_string(),
                    message: format!("Model '{}' should be in format 'provider/model'", model),
                    severity: ValidationSeverity::Warning,
                });
            }
        }

        if let Some(temp) = self.temperature {
            if !(0.0..=2.0).contains(&temp) {
                errors.push(ValidationError {
                    field: "temperature".to_string(),
                    message: format!("Temperature {} should be between 0.0 and 2.0", temp),
                    severity: ValidationSeverity::Error,
                });
            }
        }

        if let Some(agents) = &self.agent {
            for (name, agent) in &agents.agents {
                if let Some(temp) = agent.temperature {
                    if !(0.0..=2.0).contains(&temp) {
                        errors.push(ValidationError {
                            field: format!("agent.{}.temperature", name),
                            message: format!("Temperature {} should be between 0.0 and 2.0", temp),
                            severity: ValidationSeverity::Error,
                        });
                    }
                }
                if let Some(top_p) = agent.top_p {
                    if !(0.0..=1.0).contains(&top_p) {
                        errors.push(ValidationError {
                            field: format!("agent.{}.top_p", name),
                            message: format!("Top-p {} should be between 0.0 and 1.0", top_p),
                            severity: ValidationSeverity::Error,
                        });
                    }
                }
            }

            if let Some(default_agent) = &agents.default_agent {
                if !agents.agents.contains_key(default_agent) {
                    errors.push(ValidationError {
                        field: "agent.default_agent".to_string(),
                        message: format!(
                            "Default agent '{}' does not exist in agent map",
                            default_agent
                        ),
                        severity: ValidationSeverity::Error,
                    });
                }
            }
        }

        if let (Some(disabled), Some(enabled)) = (&self.disabled_providers, &self.enabled_providers)
        {
            let conflicts: Vec<&String> = disabled
                .iter()
                .filter(|d| enabled.iter().any(|e| e.eq_ignore_ascii_case(d)))
                .collect();
            if !conflicts.is_empty() {
                errors.push(ValidationError {
                    field: "disabled_providers/enabled_providers".to_string(),
                    message: format!(
                        "Providers appear in both disabled_providers and enabled_providers (disabled takes precedence): {}",
                        conflicts.iter().map(|s| s.as_str()).collect::<Vec<_>>().join(", ")
                    ),
                    severity: ValidationSeverity::Warning,
                });
            }
        }

        if let Some(providers) = &self.provider {
            for (name, provider) in providers {
                if let Some(options) = &provider.options {
                    if name != "ollama" && options.api_key.is_none() {
                        errors.push(ValidationError {
                            field: format!("provider.{}.options.api_key", name),
                            message: format!(
                                "API key not set for provider '{}' (may be set via environment)",
                                name
                            ),
                            severity: ValidationSeverity::Warning,
                        });
                    }
                }
            }
        }

        if let Some(server) = &self.server {
            if let Some(port) = server.port {
                if port < 1024 {
                    errors.push(ValidationError {
                        field: "server.port".to_string(),
                        message: "Server port must be in range 1024-65535".to_string(),
                        severity: ValidationSeverity::Error,
                    });
                }
            }
        }

        if let Some(compaction) = &self.compaction {
            if let Some(reserved) = compaction.reserved {
                if reserved > 10000 {
                    errors.push(ValidationError {
                        field: "compaction.reserved".to_string(),
                        message: format!("Reserved tokens {} seems excessively high", reserved),
                        severity: ValidationSeverity::Warning,
                    });
                }
            }
        }

        errors
    }

    pub fn is_valid(&self) -> bool {
        self.validate().iter().all(|e| !e.is_error())
    }

    pub fn validate_json_schema(&self, schema_url: Option<&str>) -> ValidationResult {
        let value = serde_json::to_value(self).unwrap_or(serde_json::Value::Null);
        schema::validate_json_schema(&value, schema_url.unwrap_or(""))
    }

    pub fn save(&self, path: &PathBuf) -> Result<(), ConfigError> {
        let ext = path.extension().and_then(|s| s.to_str());
        if ext != Some("json") && ext != Some("jsonc") {
            return Err(ConfigError::Config(format!(
                "Unsupported config format for {}. Expected .json or .jsonc extension.",
                path.display()
            )));
        }

        let content = serde_json::to_string_pretty(self).map_err(|e| {
            ConfigError::Config(format!("Failed to serialize config to JSON: {}", e))
        })?;

        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent).map_err(|e| {
                ConfigError::Config(format!(
                    "Failed to create directory {}: {}",
                    parent.display(),
                    e
                ))
            })?;
        }

        std::fs::write(path, content).map_err(|e| {
            ConfigError::Config(format!(
                "Failed to write config to {}: {}",
                path.display(),
                e
            ))
        })?;

        Ok(())
    }

    pub fn save_provider_settings(
        &mut self,
        provider_id: &str,
        config: ProviderConfig,
    ) -> Result<(), ConfigError> {
        let mut providers = self.provider.clone().unwrap_or_default();
        providers.insert(provider_id.to_string(), config);
        self.provider = Some(providers);

        self.save(&Self::config_path())
    }

    pub fn migrate_from_ts_format(json_content: &str) -> Result<Self, ConfigError> {
        let json_value: serde_json::Value =
            serde_json::from_str(json_content).map_err(|e| ConfigError::Config(e.to_string()))?;

        let mut config = Config::default();

        if let Some(obj) = json_value.as_object() {
            if let Some(log_level) = obj.get("logLevel").and_then(|v| v.as_str()) {
                config.log_level = match log_level.to_lowercase().as_str() {
                    "trace" => Some(LogLevel::Trace),
                    "debug" => Some(LogLevel::Debug),
                    "info" => Some(LogLevel::Info),
                    "warn" => Some(LogLevel::Warn),
                    "error" => Some(LogLevel::Error),
                    _ => None,
                };
            }

            if let Some(model) = obj.get("model").and_then(|v| v.as_str()) {
                config.model = Some(model.to_string());
            }

            if let Some(small_model) = obj.get("smallModel").and_then(|v| v.as_str()) {
                config.small_model = Some(small_model.to_string());
            }

            if let Some(default_agent) = obj.get("defaultAgent").and_then(|v| v.as_str()) {
                config.default_agent = Some(default_agent.to_string());
            }

            if let Some(username) = obj.get("username").and_then(|v| v.as_str()) {
                config.username = Some(username.to_string());
            }

            if let Some(api_key) = obj.get("apiKey").and_then(|v| v.as_str()) {
                config.api_key = Some(api_key.to_string());
            }

            if let Some(temp) = obj.get("temperature").and_then(|v| v.as_f64()) {
                config.temperature = Some(temp as f32);
            }

            if let Some(max_tokens) = obj.get("maxTokens").and_then(|v| v.as_u64()) {
                config.max_tokens = Some(max_tokens as u32);
            }

            if let Some(providers) = obj.get("providers").and_then(|v| v.as_object()) {
                let mut provider_map: HashMap<String, ProviderConfig> = HashMap::new();
                for (name, provider_json) in providers {
                    if let Some(provider_obj) = provider_json.as_object() {
                        let mut provider_config = ProviderConfig {
                            id: Some(name.clone()),
                            ..Default::default()
                        };

                        let mut options = ProviderOptions::default();
                        if let Some(api_key) = provider_obj.get("apiKey").and_then(|v| v.as_str()) {
                            options.api_key = Some(api_key.to_string());
                        }
                        if let Some(base_url) = provider_obj.get("baseUrl").and_then(|v| v.as_str())
                        {
                            options.base_url = Some(base_url.to_string());
                        }
                        provider_config.options = Some(options);

                        provider_map.insert(name.clone(), provider_config);
                    }
                }
                config.provider = Some(provider_map);
            }

            if let Some(disabled) = obj.get("disabledProviders").and_then(|v| v.as_array()) {
                config.disabled_providers = Some(
                    disabled
                        .iter()
                        .filter_map(|v| v.as_str().map(|s| s.to_string()))
                        .collect(),
                );
            }

            if let Some(enabled) = obj.get("enabledProviders").and_then(|v| v.as_array()) {
                config.enabled_providers = Some(
                    enabled
                        .iter()
                        .filter_map(|v| v.as_str().map(|s| s.to_string()))
                        .collect(),
                );
            }

            if let Some(share) = obj.get("share").and_then(|v| v.as_str()) {
                config.share = match share.to_lowercase().as_str() {
                    "manual" => Some(ShareMode::Manual),
                    "auto" => Some(ShareMode::Auto),
                    "disabled" => Some(ShareMode::Disabled),
                    _ => None,
                };
            }

            if let Some(autoupdate) = obj.get("autoUpdate") {
                if let Some(b) = autoupdate.as_bool() {
                    config.autoupdate = Some(AutoUpdate::Bool(b));
                } else if let Some(s) = autoupdate.as_str() {
                    config.autoupdate = Some(AutoUpdate::Notify(s.to_string()));
                }
            }

            if let Some(snapshot) = obj.get("snapshot").and_then(|v| v.as_bool()) {
                config.snapshot = Some(snapshot);
            }
        }

        Ok(config)
    }
}

#[derive(Debug, Clone)]
pub struct ValidationError {
    pub field: String,
    pub message: String,
    pub severity: ValidationSeverity,
}

impl ValidationError {
    pub fn is_error(&self) -> bool {
        matches!(self.severity, ValidationSeverity::Error)
    }

    pub fn is_warning(&self) -> bool {
        matches!(self.severity, ValidationSeverity::Warning)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum ValidationSeverity {
    Error,
    Warning,
}

#[derive(Debug, Clone, Default)]
pub struct ValidationResult {
    pub valid: bool,
    pub errors: Vec<ValidationError>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_content_hash_is_deterministic() {
        let content = r#"{"model":"openai/gpt-4.1"}"#;
        let hash1 = Config::compute_content_hash(content);
        let hash2 = Config::compute_content_hash(content);

        assert_eq!(hash1, hash2);
        assert!(!hash1.is_empty());
    }

    #[test]
    fn test_verify_integrity_with_matching_hash() {
        let content = r#"{"k":"v"}"#;
        let expected = Config::compute_content_hash(content);
        assert!(Config::verify_integrity(content, &expected));
    }

    #[test]
    fn test_verify_integrity_with_mismatched_hash() {
        let content = r#"{"k":"v"}"#;
        let expected = Config::compute_content_hash("different");
        assert!(!Config::verify_integrity(content, &expected));
    }

    #[test]
    fn test_default_config() {
        let config = Config::default();
        assert!(config.model.is_none());
        assert!(config.provider.is_none());
    }

    #[test]
    fn test_provider_enabled() {
        let mut config = Config::default();

        assert!(config.is_provider_enabled("openai"));

        config.enabled_providers = Some(vec!["anthropic".to_string()]);
        assert!(!config.is_provider_enabled("openai"));
        assert!(config.is_provider_enabled("anthropic"));

        config.enabled_providers = None;
        config.disabled_providers = Some(vec!["ollama".to_string()]);
        assert!(config.is_provider_enabled("openai"));
        assert!(!config.is_provider_enabled("ollama"));

        config.enabled_providers = Some(vec!["openai".to_string(), "anthropic".to_string()]);
        config.disabled_providers = Some(vec!["openai".to_string()]);
        assert!(!config.is_provider_enabled("openai"));
        assert!(config.is_provider_enabled("anthropic"));
    }

    #[test]
    fn test_provider_options_sanitize_for_logging_redacts_api_key() {
        let options = ProviderOptions {
            api_key: Some("super-secret".to_string()),
            base_url: Some("https://example.com".to_string()),
            ..Default::default()
        };

        let sanitized = options.sanitize_for_logging();
        assert_eq!(sanitized.api_key.as_deref(), Some("***REDACTED***"));
        assert_eq!(sanitized.base_url, options.base_url);
    }

    #[test]
    fn test_variable_expansion_basic() {
        let mut value = serde_json::json!({
            "model": "openai/gpt-4o",
            "api_key": "${model}"
        });
        let result = Config::expand_variables(&mut value);
        assert!(result.is_ok());
        assert_eq!(value["api_key"], "openai/gpt-4o");
    }

    #[test]
    fn test_variable_expansion_nested() {
        let mut value = serde_json::json!({
            "a": "first",
            "b": "${a}",
            "c": "${b}"
        });
        let result = Config::expand_variables(&mut value);
        assert!(result.is_ok());
        assert_eq!(value["c"], "first");
    }

    #[test]
    fn test_precedence_env_overrides_config_file() {
        let temp_dir = tempfile::tempdir().unwrap();
        let config_path = temp_dir.path().join("config.json");
        let config_content = serde_json::json!({
            "model": "config-file-model",
            "temperature": 0.3,
            "maxTokens": 1000
        })
        .to_string();
        std::fs::write(&config_path, config_content).unwrap();

        std::env::set_var("OPENCODE_MODEL", "env-model");
        std::env::set_var("OPENCODE_TEMPERATURE", "0.8");

        let mut config = Config::load(&config_path).unwrap();
        config.apply_env_overrides();

        assert_eq!(config.model, Some("env-model".to_string()));
        assert_eq!(config.temperature, Some(0.8));
        assert_eq!(config.max_tokens, Some(1000));

        std::env::remove_var("OPENCODE_MODEL");
        std::env::remove_var("OPENCODE_TEMPERATURE");
    }

    #[test]
    fn test_precedence_cli_overrides_env() {
        std::env::set_var("OPENCODE_MODEL", "env-model");
        std::env::set_var("OPENCODE_TEMPERATURE", "0.8");

        let mut config = Config::default();
        config.apply_env_overrides();

        assert_eq!(config.model, Some("env-model".to_string()));
        assert_eq!(config.temperature, Some(0.8));

        let cli_overrides = CliOverrideConfig {
            model: Some("cli-model".to_string()),
            provider: Some("anthropic".to_string()),
            temperature: Some(0.1),
            max_tokens: Some(4000),
            default_agent: Some("build".to_string()),
        };
        config.apply_cli_overrides(
            cli_overrides.model,
            cli_overrides.provider,
            cli_overrides.temperature,
            cli_overrides.max_tokens,
            cli_overrides.default_agent.clone(),
        );

        assert_eq!(config.model, Some("cli-model".to_string()));
        assert_eq!(config.temperature, Some(0.1));
        assert_eq!(config.max_tokens, Some(4000));
        assert_eq!(config.default_agent, Some("build".to_string()));

        std::env::remove_var("OPENCODE_MODEL");
        std::env::remove_var("OPENCODE_TEMPERATURE");
    }

    #[tokio::test]
    async fn test_precedence_full_chain_integration() {
        let temp_dir = tempfile::tempdir().unwrap();
        let config_path = temp_dir.path().join("config.json");
        let config_content = serde_json::json!({
            "model": "file-model",
            "temperature": 0.3,
            "provider": {
                "openai": {"id": "openai"}
            }
        })
        .to_string();
        std::fs::write(&config_path, config_content).unwrap();

        std::env::set_var("OPENCODE_CONFIG_DIR", temp_dir.path().to_str().unwrap());
        std::env::set_var("OPENCODE_MODEL", "env-model");
        std::env::set_var("OPENCODE_TEMPERATURE", "0.8");

        let cli_overrides = CliOverrideConfig {
            model: Some("cli-model".to_string()),
            provider: Some("google".to_string()),
            temperature: Some(0.1),
            max_tokens: Some(8000),
            default_agent: Some("plan".to_string()),
        };
        let config = Config::load_multi(Some(&cli_overrides)).await.unwrap();

        assert_eq!(config.model, Some("cli-model".to_string()));
        assert!(config.provider.as_ref().unwrap().contains_key("google"));
        assert_eq!(config.temperature, Some(0.1));
        assert_eq!(config.max_tokens, Some(8000));
        assert_eq!(config.default_agent, Some("plan".to_string()));

        std::env::remove_var("OPENCODE_CONFIG_DIR");
        std::env::remove_var("OPENCODE_MODEL");
        std::env::remove_var("OPENCODE_TEMPERATURE");
    }

    #[test]
    fn test_precedence_provider_api_keys_from_env() {
        std::env::set_var("OPENAI_API_KEY", "sk-openai-test");
        std::env::set_var("ANTHROPIC_API_KEY", "sk-ant-test");
        std::env::set_var("GOOGLE_API_KEY", "google-test-key");

        let mut config = Config::default();
        config.apply_env_overrides();

        let providers = config.provider.as_ref().unwrap();
        assert_eq!(
            providers
                .get("openai")
                .unwrap()
                .options
                .as_ref()
                .unwrap()
                .api_key
                .as_ref()
                .unwrap(),
            "sk-openai-test"
        );
        assert_eq!(
            providers
                .get("anthropic")
                .unwrap()
                .options
                .as_ref()
                .unwrap()
                .api_key
                .as_ref()
                .unwrap(),
            "sk-ant-test"
        );
        assert_eq!(
            providers
                .get("google")
                .unwrap()
                .options
                .as_ref()
                .unwrap()
                .api_key
                .as_ref()
                .unwrap(),
            "google-test-key"
        );

        std::env::remove_var("OPENAI_API_KEY");
        std::env::remove_var("ANTHROPIC_API_KEY");
        std::env::remove_var("GOOGLE_API_KEY");
    }

    #[test]
    fn test_json_parsing() {
        let json_content = r#"{
            "model": "openai/gpt-4o",
            "temperature": 0.7
        }"#;
        let config: Config = serde_json::from_str(json_content).unwrap();
        assert_eq!(config.model, Some("openai/gpt-4o".to_string()));
        assert_eq!(config.temperature, Some(0.7));
    }

    #[test]
    fn test_jsonc_parsing_with_comments() {
        let jsonc_content = r#"{
            // This is a comment
            "model": "openai/gpt-4o",
            /* Multi-line comment */
            "temperature": 0.7
        }"#;
        let result = Config::parse_json_content(jsonc_content);
        assert!(result.is_ok());
        let config = result.unwrap();
        assert_eq!(config.model, Some("openai/gpt-4o".to_string()));
        assert_eq!(config.temperature, Some(0.7));
    }

    #[test]
    fn test_env_variable_expansion() {
        std::env::set_var("TEST_MODEL", "test-model-from-env");
        let input = "model: {env:TEST_MODEL}";
        let result = Config::substitute_variables(input, None);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "model: test-model-from-env");
        std::env::remove_var("TEST_MODEL");
    }

    #[test]
    fn test_scroll_acceleration_from_f32() {
        let config: ScrollAccelerationConfig = 1.5f32.into();
        assert!(config.enabled);
        assert_eq!(config.speed, Some(1.5));
    }

    #[test]
    fn test_scroll_acceleration_default() {
        let config = ScrollAccelerationConfig::default();
        assert!(config.enabled);
        assert_eq!(config.speed, None);
    }

    #[test]
    fn test_deprecated_mode_field_still_parses() {
        let json_content = r#"{
            "model": "openai/gpt-4o",
            "mode": "agent"
        }"#;
        let result = Config::parse_json_content(json_content);
        assert!(
            result.is_ok(),
            "Deprecated 'mode' field should not cause parse error"
        );
        let config = result.unwrap();
        assert_eq!(config.model, Some("openai/gpt-4o".to_string()));
    }

    #[test]
    fn test_deprecated_mode_field_emits_warning() {
        use tracing::Level;

        let json_content = r#"{
            "model": "openai/gpt-4o",
            "mode": "agent"
        }"#;

        let temp_file = tempfile::NamedTempFile::new().unwrap();
        let path = temp_file.path().to_path_buf();

        let subscriber = tracing_subscriber::fmt()
            .with_max_level(Level::WARN)
            .with_writer(std::fs::File::create(&path).unwrap())
            .with_ansi(false)
            .finish();

        tracing::subscriber::with_default(subscriber, || {
            let result = Config::parse_json_content(json_content);
            assert!(result.is_ok());
        });

        let output = std::fs::read_to_string(&path).unwrap();
        assert!(
            output.contains("Deprecated config field 'mode'"),
            "Warning about deprecated 'mode' field should have been emitted, but got: {}",
            output
        );
    }

    #[test]
    fn test_deprecated_mode_agent_mode_field_emits_warning() {
        use tracing::Level;

        let json_content = r#"{
            "model": "openai/gpt-4o",
            "agent": {
                "build": {
                    "mode": "agent"
                }
            }
        }"#;

        let temp_file = tempfile::NamedTempFile::new().unwrap();
        let path = temp_file.path().to_path_buf();

        let subscriber = tracing_subscriber::fmt()
            .with_max_level(Level::WARN)
            .with_writer(std::fs::File::create(&path).unwrap())
            .with_ansi(false)
            .finish();

        tracing::subscriber::with_default(subscriber, || {
            let result = Config::parse_json_content(json_content);
            assert!(result.is_ok());
        });

        let output = std::fs::read_to_string(&path).unwrap();
        assert!(
            output.contains("agent.build.mode"),
            "Warning about deprecated 'agent.<name>.mode' field should have been emitted, but got: {}",
            output
        );
    }

    #[test]
    fn test_deprecated_tools_field_still_parses() {
        let json_content = r#"{
            "model": "test-model",
            "tools": ["read", "write", "bash"]
        }"#;
        let result = Config::parse_json_content(json_content);
        assert!(
            result.is_ok(),
            "Deprecated 'tools' field should not cause parse error"
        );
        let config = result.unwrap();
        assert_eq!(config.model, Some("test-model".to_string()));
    }

    #[test]
    fn test_deprecated_tools_field_migrates_to_permission() {
        let json_content = r#"{
            "model": "test-model",
            "tools": ["read", "write", "bash"]
        }"#;
        let result = Config::parse_json_content(json_content);
        assert!(result.is_ok());
        let config = result.unwrap();
        assert!(
            config.permission.is_some(),
            "permission field should be created from migration"
        );
    }

    #[test]
    fn test_deprecated_tools_field_migration_content() {
        let json_content = r#"{
            "tools": ["read", "bash"]
        }"#;
        let result = Config::parse_json_content(json_content);
        assert!(result.is_ok());
        let config = result.unwrap();
        let permission = config
            .permission
            .expect("permission should exist after migration");
        assert!(permission.read.is_some(), "read permission should be set");
        assert!(permission.bash.is_some(), "bash permission should be set");
    }

    #[test]
    fn test_deprecated_tools_merges_with_existing_permission() {
        let json_content = r#"{
            "permission": {
                "read": {"action": "deny"}
            },
            "tools": ["write", "bash"]
        }"#;
        let result = Config::parse_json_content(json_content);
        assert!(result.is_ok());
        let config = result.unwrap();
        let permission = config.permission.expect("permission should exist");
        assert!(permission.read.is_some(), "read permission should exist");
        assert!(
            permission.bash.is_some(),
            "bash permission should exist from migration"
        );
    }

    #[test]
    fn test_deprecated_tools_empty_array() {
        let json_content = r#"{
            "tools": []
        }"#;
        let result = Config::parse_json_content(json_content);
        assert!(result.is_ok());
        let config = result.unwrap();
        assert!(
            config.permission.is_none(),
            "permission should not be created for empty tools array"
        );
    }

    #[test]
    fn test_deprecated_tools_preserves_other_fields() {
        let json_content = r#"{
            "model": "gpt-4",
            "temperature": 0.7,
            "tools": ["read"],
            "provider": {
                "openai": {}
            }
        }"#;
        let result = Config::parse_json_content(json_content);
        assert!(result.is_ok());
        let config = result.unwrap();
        assert_eq!(config.model, Some("gpt-4".to_string()));
        assert_eq!(config.temperature, Some(0.7));
        assert!(config.provider.is_some());
        assert!(config.permission.is_some());
    }

    #[test]
    fn test_deprecated_tools_emits_warning() {
        use tracing::Level;

        let json_content = r#"{
            "tools": ["read", "write"]
        }"#;

        let temp_file = tempfile::NamedTempFile::new().unwrap();
        let path = temp_file.path().to_path_buf();

        let subscriber = tracing_subscriber::fmt()
            .with_max_level(Level::WARN)
            .with_writer(std::fs::File::create(&path).unwrap())
            .with_ansi(false)
            .finish();

        tracing::subscriber::with_default(subscriber, || {
            let result = Config::parse_json_content(json_content);
            assert!(result.is_ok());
        });

        let output = std::fs::read_to_string(&path).unwrap();
        assert!(
            output.contains("Deprecated config field 'tools'"),
            "Warning about deprecated 'tools' field should have been emitted, but got: {}",
            output
        );
    }

    #[test]
    fn test_deprecated_tools_migration_emits_warning() {
        use tracing::Level;

        let json_content = r#"{
            "tools": ["read", "bash"]
        }"#;

        let temp_file = tempfile::NamedTempFile::new().unwrap();
        let path = temp_file.path().to_path_buf();

        let subscriber = tracing_subscriber::fmt()
            .with_max_level(Level::WARN)
            .with_writer(std::fs::File::create(&path).unwrap())
            .with_ansi(false)
            .finish();

        tracing::subscriber::with_default(subscriber, || {
            let result = Config::parse_json_content(json_content);
            assert!(result.is_ok());
        });

        let output = std::fs::read_to_string(&path).unwrap();
        assert!(
            output.contains("migrated to 'permission'"),
            "Warning about migration should have been emitted, but got: {}",
            output
        );
    }

    #[test]
    fn test_validation_error_is_error() {
        let error = ValidationError {
            field: "test".to_string(),
            message: "test error".to_string(),
            severity: ValidationSeverity::Error,
        };
        assert!(error.is_error());
        assert!(!error.is_warning());
    }

    #[test]
    fn test_validation_error_is_warning() {
        let error = ValidationError {
            field: "test".to_string(),
            message: "test warning".to_string(),
            severity: ValidationSeverity::Warning,
        };
        assert!(error.is_warning());
        assert!(!error.is_error());
    }

    #[test]
    fn test_agent_map_config_get_agent() {
        let mut agents = HashMap::new();
        agents.insert(
            "build".to_string(),
            AgentConfig {
                model: Some("gpt-4".to_string()),
                ..Default::default()
            },
        );
        let config = AgentMapConfig {
            agents,
            default_agent: None,
        };
        assert!(config.get_agent("build").is_some());
        assert!(config.get_agent("nonexistent").is_none());
    }

    #[test]
    fn test_agent_map_config_get_default_agent() {
        let mut agents = HashMap::new();
        agents.insert(
            "build".to_string(),
            AgentConfig {
                model: Some("gpt-4".to_string()),
                ..Default::default()
            },
        );
        let config = AgentMapConfig {
            agents,
            default_agent: Some("build".to_string()),
        };
        assert!(config.get_default_agent().is_some());
        assert_eq!(
            config.get_default_agent().unwrap().model.as_deref(),
            Some("gpt-4")
        );
    }

    #[test]
    fn test_agent_map_config_get_default_agent_nonexistent() {
        let mut agents = HashMap::new();
        agents.insert(
            "build".to_string(),
            AgentConfig {
                model: Some("gpt-4".to_string()),
                ..Default::default()
            },
        );
        let config = AgentMapConfig {
            agents,
            default_agent: Some("nonexistent".to_string()),
        };
        assert!(config.get_default_agent().is_none());
    }

    #[test]
    fn test_config_validate_model_without_slash() {
        let config = Config {
            model: Some("invalid-model".to_string()),
            ..Default::default()
        };
        let errors = config.validate();
        assert!(!errors.is_empty());
        assert!(errors.iter().any(|e| e.field == "model"));
    }

    #[test]
    fn test_config_validate_temperature_out_of_range() {
        let config = Config {
            temperature: Some(5.0),
            ..Default::default()
        };
        let errors = config.validate();
        assert!(!errors.is_empty());
        assert!(errors.iter().any(|e| e.field == "temperature"));
    }

    #[test]
    fn test_config_validate_agent_temperature_out_of_range() {
        let mut agents = HashMap::new();
        agents.insert(
            "build".to_string(),
            AgentConfig {
                temperature: Some(5.0),
                ..Default::default()
            },
        );
        let config = Config {
            agent: Some(AgentMapConfig {
                agents,
                default_agent: None,
            }),
            ..Default::default()
        };
        let errors = config.validate();
        assert!(!errors.is_empty());
        assert!(errors
            .iter()
            .any(|e| e.field.contains("agent.build.temperature")));
    }

    #[test]
    fn test_config_validate_agent_top_p_out_of_range() {
        let mut agents = HashMap::new();
        agents.insert(
            "build".to_string(),
            AgentConfig {
                top_p: Some(2.0),
                ..Default::default()
            },
        );
        let config = Config {
            agent: Some(AgentMapConfig {
                agents,
                default_agent: None,
            }),
            ..Default::default()
        };
        let errors = config.validate();
        assert!(!errors.is_empty());
        assert!(errors.iter().any(|e| e.field.contains("agent.build.top_p")));
    }

    #[test]
    fn test_config_validate_disabled_and_enabled_conflict() {
        let config = Config {
            disabled_providers: Some(vec!["openai".to_string()]),
            enabled_providers: Some(vec!["openai".to_string(), "anthropic".to_string()]),
            ..Default::default()
        };
        let errors = config.validate();
        assert!(!errors.is_empty());
        assert!(errors
            .iter()
            .any(|e| e.field.contains("disabled_providers/enabled_providers")));
    }

    #[test]
    fn test_config_validate_server_port_low() {
        let config = Config {
            server: Some(ServerConfig {
                port: Some(80),
                ..Default::default()
            }),
            ..Default::default()
        };
        let errors = config.validate();
        assert!(!errors.is_empty());
        assert!(errors.iter().any(|e| e.field == "server.port"));
    }

    #[test]
    fn test_config_validate_compaction_reserved_high() {
        let config = Config {
            compaction: Some(CompactionConfig {
                reserved: Some(20000),
                ..Default::default()
            }),
            ..Default::default()
        };
        let errors = config.validate();
        assert!(!errors.is_empty());
        assert!(errors.iter().any(|e| e.field == "compaction.reserved"));
    }

    #[test]
    fn test_config_is_valid() {
        let config = Config {
            model: Some("openai/gpt-4".to_string()),
            temperature: Some(0.7),
            ..Default::default()
        };
        assert!(config.is_valid());
    }

    #[test]
    fn test_config_is_not_valid() {
        let config = Config {
            model: Some("invalid".to_string()),
            temperature: Some(5.0),
            ..Default::default()
        };
        assert!(!config.is_valid());
    }

    #[test]
    fn test_config_batch_tool_enabled() {
        let config = Config {
            experimental: Some(ExperimentalConfig {
                batch_tool: Some(true),
                ..Default::default()
            }),
            ..Default::default()
        };
        assert!(config.batch_tool_enabled());

        let config = Config::default();
        assert!(!config.batch_tool_enabled());
    }

    #[test]
    fn test_config_open_telemetry_enabled() {
        let config = Config {
            experimental: Some(ExperimentalConfig {
                open_telemetry: Some(true),
                ..Default::default()
            }),
            ..Default::default()
        };
        assert!(config.open_telemetry_enabled());

        let config = Config::default();
        assert!(!config.open_telemetry_enabled());
    }

    #[test]
    fn test_config_get_disabled_tools_permission_action_deny() {
        let config = Config {
            permission: Some(PermissionConfig {
                read: Some(PermissionRule::Action(PermissionAction::Deny)),
                edit: Some(PermissionRule::Action(PermissionAction::Allow)),
                todowrite: Some(PermissionAction::Deny),
                ..Default::default()
            }),
            ..Default::default()
        };
        let disabled = config.get_disabled_tools();
        assert!(disabled.contains("read"));
        assert!(disabled.contains("todowrite"));
        assert!(!disabled.contains("edit"));
    }

    #[test]
    fn test_config_get_disabled_tools_permission_object_deny() {
        let config = Config {
            permission: Some(PermissionConfig {
                read: Some(PermissionRule::Object(HashMap::from([
                    ("file1".to_string(), PermissionAction::Deny),
                    ("file2".to_string(), PermissionAction::Allow),
                ]))),
                ..Default::default()
            }),
            ..Default::default()
        };
        let disabled = config.get_disabled_tools();
        assert!(disabled.contains("file1"));
        assert!(!disabled.contains("file2"));
    }

    #[test]
    fn test_config_get_disabled_tools_permission_extra() {
        let config = Config {
            permission: Some(PermissionConfig {
                extra: Some(HashMap::from([(
                    "custom_tool".to_string(),
                    PermissionRule::Action(PermissionAction::Deny),
                )])),
                ..Default::default()
            }),
            ..Default::default()
        };
        let disabled = config.get_disabled_tools();
        assert!(disabled.contains("custom_tool"));
    }

    #[test]
    fn test_config_get_provider() {
        let mut providers = HashMap::new();
        providers.insert(
            "openai".to_string(),
            ProviderConfig {
                id: Some("openai".to_string()),
                ..Default::default()
            },
        );
        let config = Config {
            provider: Some(providers),
            ..Default::default()
        };
        assert!(config.get_provider("openai").is_some());
        assert!(config.get_provider("nonexistent").is_none());
    }

    #[test]
    fn test_config_get_provider_filter() {
        let config = Config {
            disabled_providers: Some(vec!["openai".to_string()]),
            enabled_providers: Some(vec!["anthropic".to_string()]),
            ..Default::default()
        };
        let filter = config.get_provider_filter();
        assert!(filter.is_some());
        let (disabled, enabled) = filter.unwrap();
        assert!(disabled.contains(&"openai".to_string()));
        assert!(enabled.contains(&"anthropic".to_string()));
    }

    #[test]
    fn test_config_get_provider_filter_none() {
        let config = Config::default();
        assert!(config.get_provider_filter().is_none());
    }

    #[test]
    fn test_config_provider_enabled_empty_enabled_list() {
        let config = Config {
            disabled_providers: Some(vec!["openai".to_string()]),
            enabled_providers: Some(vec![]),
            ..Default::default()
        };
        assert!(!config.is_provider_enabled("openai"));
        assert!(config.is_provider_enabled("anthropic"));
    }

    #[test]
    fn test_contains_keychain_reference() {
        assert!(Config::contains_keychain_reference("{keychain:secret}"));
        assert!(Config::contains_keychain_reference(
            "some {keychain:test} text"
        ));
        assert!(!Config::contains_keychain_reference("no keychain here"));
    }

    #[test]
    fn test_redact_keychain_references() {
        let input = "before {keychain:my-secret} after";
        let result = Config::redact_keychain_references(input);
        assert_eq!(result, "before [keychain:my-secret] after");
    }

    #[test]
    fn test_redact_keychain_references_multiple() {
        let input = "{keychain:one} and {keychain:two}";
        let result = Config::redact_keychain_references(input);
        assert_eq!(result, "[keychain:one] and [keychain:two]");
    }

    #[test]
    fn test_redact_keychain_references_no_match() {
        let input = "no secrets here";
        let result = Config::redact_keychain_references(input);
        assert_eq!(result, input);
    }

    #[test]
    fn test_keybind_config_detect_conflicts() {
        let config = KeybindConfig {
            commands: Some("ctrl-c".to_string()),
            timeline: Some("ctrl-c".to_string()),
            ..Default::default()
        };
        let conflicts = config.detect_conflicts();
        assert!(!conflicts.is_empty());
    }

    #[test]
    fn test_keybind_config_no_conflicts() {
        let config = KeybindConfig {
            commands: Some("ctrl-c".to_string()),
            timeline: Some("ctrl-v".to_string()),
            ..Default::default()
        };
        let conflicts = config.detect_conflicts();
        assert!(conflicts.is_empty());
    }

    #[test]
    fn test_keybind_config_merge_with_defaults() {
        let user = KeybindConfig {
            commands: Some("ctrl-x".to_string()),
            ..Default::default()
        };
        let defaults = KeybindConfig {
            commands: Some("ctrl-c".to_string()),
            timeline: Some("ctrl-v".to_string()),
            ..Default::default()
        };
        let (merged, conflicts) = user.merge_with_defaults(&defaults);
        assert_eq!(merged.commands, Some("ctrl-x".to_string()));
        assert_eq!(merged.timeline, Some("ctrl-v".to_string()));
        assert!(conflicts.is_empty());
    }

    #[test]
    fn test_keybind_config_merge_custom() {
        let user = KeybindConfig {
            custom: Some(HashMap::from([(
                "custom1".to_string(),
                "ctrl-q".to_string(),
            )])),
            ..Default::default()
        };
        let defaults = KeybindConfig {
            custom: Some(HashMap::from([(
                "custom2".to_string(),
                "ctrl-w".to_string(),
            )])),
            ..Default::default()
        };
        let (merged, _conflicts) = user.merge_with_defaults(&defaults);
        assert!(merged.custom.is_some());
        let custom = merged.custom.unwrap();
        assert!(custom.contains_key("custom1"));
        assert!(custom.contains_key("custom2"));
    }

    #[test]
    fn test_theme_config_resolve_path_absolute() {
        let temp = tempfile::tempdir().unwrap();
        let config = ThemeConfig {
            path: Some(temp.path().to_path_buf()),
            ..Default::default()
        };
        assert!(config.resolve_path(None).is_some());
    }

    #[test]
    fn test_theme_config_resolve_path_relative() {
        let temp = tempfile::tempdir().unwrap();
        std::env::set_var("HOME", temp.path().to_str().unwrap());
        let config = ThemeConfig {
            path: Some(PathBuf::from("~/themes/my-theme")),
            ..Default::default()
        };
        let resolved = config.resolve_path(None);
        std::env::remove_var("HOME");
        if let Some(path) = resolved {
            if path.exists() {
                assert!(path.to_string_lossy().contains("my-theme"));
            }
        }
    }

    #[test]
    fn test_theme_config_resolve_path_tilde() {
        let temp = tempfile::tempdir().unwrap();
        std::env::set_var("HOME", temp.path().to_str().unwrap());
        let config = ThemeConfig {
            path: Some(PathBuf::from("~")),
            ..Default::default()
        };
        let resolved = config.resolve_path(None);
        std::env::remove_var("HOME");
        assert!(resolved.is_some());
    }

    #[test]
    fn test_theme_config_resolve_path_not_exists() {
        let config = ThemeConfig {
            path: Some(PathBuf::from("/nonexistent/path")),
            ..Default::default()
        };
        assert!(config.resolve_path(None).is_none());
    }

    #[test]
    fn test_config_save_json() {
        let temp = tempfile::tempdir().unwrap();
        let path = temp.path().join("config.json");
        let config = Config {
            model: Some("openai/gpt-4".to_string()),
            temperature: Some(0.7),
            ..Default::default()
        };
        let result = config.save(&path);
        assert!(result.is_ok());
        assert!(path.exists());
        let content = std::fs::read_to_string(&path).unwrap();
        assert!(content.contains("gpt-4"));
    }

    #[test]
    fn test_config_save_jsonc() {
        let temp = tempfile::tempdir().unwrap();
        let path = temp.path().join("config.jsonc");
        let config = Config {
            model: Some("openai/gpt-4".to_string()),
            ..Default::default()
        };
        let result = config.save(&path);
        assert!(result.is_ok());
        assert!(path.exists());
    }

    #[test]
    fn test_config_save_creates_parent_dirs() {
        let temp = tempfile::tempdir().unwrap();
        let path = temp.path().join("deep").join("nested").join("config.json");
        let config = Config::default();
        let result = config.save(&path);
        assert!(result.is_ok());
        assert!(path.exists());
    }

    #[test]
    fn test_config_load_json5_file() {
        let temp = tempfile::tempdir().unwrap();
        let path = temp.path().join("config.json5");
        std::fs::write(&path, r#"{"model": "openai/gpt-4"}"#).unwrap();
        let config = Config::load(&path);
        assert!(config.is_ok());
    }

    #[test]
    fn test_config_load_nonexistent_file() {
        let temp = tempfile::tempdir().unwrap();
        let path = temp.path().join("nonexistent.json");
        let config = Config::load(&path);
        assert!(config.is_ok());
        assert!(config.unwrap().model.is_none());
    }

    #[test]
    fn test_config_migrate_from_ts_format() {
        let json = r#"{
            "logLevel": "debug",
            "model": "openai/gpt-4",
            "smallModel": "openai/gpt-3.5",
            "defaultAgent": "build",
            "username": "testuser",
            "apiKey": "secret-key",
            "temperature": 0.7,
            "maxTokens": 1000,
            "providers": {
                "openai": {
                    "apiKey": "provider-key",
                    "baseUrl": "https://api.openai.com"
                }
            },
            "disabledProviders": ["ollama"],
            "enabledProviders": ["openai", "anthropic"],
            "share": "auto",
            "autoUpdate": true,
            "snapshot": false
        }"#;
        let config = Config::migrate_from_ts_format(json);
        assert!(config.is_ok());
        let config = config.unwrap();
        assert_eq!(config.log_level, Some(LogLevel::Debug));
        assert_eq!(config.model, Some("openai/gpt-4".to_string()));
        assert_eq!(config.small_model, Some("openai/gpt-3.5".to_string()));
        assert_eq!(config.username, Some("testuser".to_string()));
        assert_eq!(config.api_key, Some("secret-key".to_string()));
        assert_eq!(config.temperature, Some(0.7));
        assert_eq!(config.max_tokens, Some(1000));
        assert!(config.provider.is_some());
        assert_eq!(config.disabled_providers, Some(vec!["ollama".to_string()]));
        assert_eq!(
            config.enabled_providers,
            Some(vec!["openai".to_string(), "anthropic".to_string()])
        );
        assert_eq!(config.share, Some(ShareMode::Auto));
        assert_eq!(config.autoupdate, Some(AutoUpdate::Bool(true)));
        assert_eq!(config.snapshot, Some(false));
    }

    #[test]
    fn test_config_migrate_from_ts_format_autoupdate_notify() {
        let json = r#"{"autoUpdate": "notify"}"#;
        let config = Config::migrate_from_ts_format(json);
        assert!(config.is_ok());
        let config = config.unwrap();
        assert_eq!(
            config.autoupdate,
            Some(AutoUpdate::Notify("notify".to_string()))
        );
    }

    #[test]
    fn test_agents_md_config_to_scan_config() {
        let config = AgentsMdConfig {
            enabled: Some(false),
            stop_at_worktree_root: Some(false),
            include_hidden: Some(true),
        };
        let scan = config.to_scan_config();
        assert!(!scan.enabled);
        assert!(!scan.stop_at_worktree_root);
        assert!(scan.include_hidden);
    }

    #[test]
    fn test_agents_md_config_to_scan_defaults() {
        let config = AgentsMdConfig::default();
        let scan = config.to_scan_config();
        assert!(scan.enabled);
        assert!(scan.stop_at_worktree_root);
        assert!(!scan.include_hidden);
    }

    #[test]
    fn test_config_error_from_string() {
        let err: String = ConfigError::Config("test error".to_string()).into();
        assert!(err.contains("test error"));
    }

    #[test]
    fn test_timeout_config_serialization() {
        let json = serde_json::json!(5000);
        let config: TimeoutConfig = serde_json::from_value(json).unwrap();
        assert!(matches!(config, TimeoutConfig::Milliseconds(5000)));
    }

    #[test]
    fn test_mcp_config_local() {
        let json = serde_json::json!({
            "type": "local",
            "command": ["npx", "mcp-server"]
        });
        let config: McpConfig = serde_json::from_value(json).unwrap();
        assert!(matches!(config, McpConfig::Local(_)));
    }

    #[test]
    fn test_mcp_config_remote() {
        let json = serde_json::json!({
            "type": "remote",
            "url": "https://mcp.example.com"
        });
        let config: McpConfig = serde_json::from_value(json).unwrap();
        assert!(matches!(config, McpConfig::Remote(_)));
    }

    #[test]
    fn test_mcp_config_simple() {
        let json = serde_json::json!({"type": "simple", "enabled": true});
        let config: McpConfig = serde_json::from_value(json).unwrap();
        assert!(matches!(config, McpConfig::Simple { enabled: true }));
    }

    #[test]
    fn test_mcp_oauth_union_config() {
        let json = serde_json::json!({
            "client_id": "test-id",
            "client_secret": "test-secret"
        });
        let oauth: McpOAuthUnion = serde_json::from_value(json).unwrap();
        assert!(matches!(oauth, McpOAuthUnion::Config(_)));
    }

    #[test]
    fn test_mcp_oauth_union_disabled() {
        let json = serde_json::json!(false);
        let oauth: McpOAuthUnion = serde_json::from_value(json).unwrap();
        assert!(matches!(oauth, McpOAuthUnion::Disabled(false)));
    }

    #[test]
    fn test_lsp_config_disabled() {
        let json = serde_json::json!(true);
        let config: LspConfig = serde_json::from_value(json).unwrap();
        assert!(matches!(config, LspConfig::Disabled(true)));
    }

    #[test]
    fn test_lsp_config_servers() {
        let json = serde_json::json!({
            "rust": {"command": ["rust-analyzer"]}
        });
        let config: LspConfig = serde_json::from_value(json).unwrap();
        assert!(matches!(config, LspConfig::Servers(_)));
    }

    #[test]
    fn test_formatter_config_disabled() {
        let json = serde_json::json!(true);
        let config: FormatterConfig = serde_json::from_value(json).unwrap();
        assert!(matches!(config, FormatterConfig::Disabled(true)));
    }

    #[test]
    fn test_formatter_config_formatters() {
        let json = serde_json::json!({
            "prettier": {"command": ["prettier"]}
        });
        let config: FormatterConfig = serde_json::from_value(json).unwrap();
        assert!(matches!(config, FormatterConfig::Formatters(_)));
    }

    #[test]
    fn test_diff_style_serialization() {
        let json = serde_json::json!("sidebyside");
        let config: DiffStyle = serde_json::from_value(json).unwrap();
        assert!(matches!(config, DiffStyle::SideBySide));
    }

    #[test]
    fn test_tui_config_scroll_acceleration_legacy_number() {
        let json = serde_json::json!(1.5);
        let config: ScrollAccelerationConfig = serde_json::from_value(json).unwrap();
        assert!(config.enabled);
        assert_eq!(config.speed, Some(1.5));
    }

    #[test]
    fn test_scroll_acceleration_deserialize_map() {
        let json = serde_json::json!({"enabled": false, "speed": 2.0});
        let config: ScrollAccelerationConfig = serde_json::from_value(json).unwrap();
        assert!(!config.enabled);
        assert_eq!(config.speed, Some(2.0));
    }

    #[test]
    fn test_scroll_acceleration_deserialize_map_unknown_key() {
        let json = serde_json::json!({"enabled": true, "unknown": 123, "speed": 1.0});
        let config: ScrollAccelerationConfig = serde_json::from_value(json).unwrap();
        assert!(config.enabled);
        assert_eq!(config.speed, Some(1.0));
    }

    #[test]
    fn test_tui_plugin_config_default() {
        let config = TuiPluginConfig::default();
        assert_eq!(config.plugin_enabled, None);
    }

    #[test]
    fn test_tui_config_deserialization() {
        let json = serde_json::json!({
            "scroll_speed": 50,
            "diff_style": "inline"
        });
        let config: TuiConfig = serde_json::from_value(json).unwrap();
        assert_eq!(config.scroll_speed, Some(50));
        assert!(matches!(config.diff_style, Some(DiffStyle::Inline)));
    }

    #[test]
    fn test_provider_config_sanitize_for_logging() {
        let config = ProviderConfig {
            id: Some("test".to_string()),
            options: Some(ProviderOptions {
                api_key: Some("secret".to_string()),
                base_url: Some("https://api.test.com".to_string()),
                ..Default::default()
            }),
            ..Default::default()
        };
        let sanitized = config.sanitize_for_logging();
        assert_eq!(
            sanitized.options.as_ref().unwrap().api_key.as_deref(),
            Some("***REDACTED***")
        );
        assert_eq!(
            sanitized.options.as_ref().unwrap().base_url.as_deref(),
            Some("https://api.test.com")
        );
    }

    #[test]
    fn test_server_config_default() {
        let config = ServerConfig::default();
        assert!(config.port.is_none());
        assert!(config.hostname.is_none());
    }

    #[test]
    fn test_desktop_config_default() {
        let config = DesktopConfig::default();
        assert!(config.enabled.is_none());
        assert!(config.auto_open_browser.is_none());
    }

    #[test]
    fn test_acp_config_default() {
        let config = AcpConfig::default();
        assert!(config.enabled.is_none());
        assert!(config.server_id.is_none());
    }

    #[test]
    fn test_log_level_serialization() {
        let json = serde_json::json!("debug");
        let level: LogLevel = serde_json::from_value(json).unwrap();
        assert!(matches!(level, LogLevel::Debug));
    }

    #[test]
    fn test_share_mode_variants() {
        assert_eq!(serde_json::to_string(&ShareMode::Auto).unwrap(), "\"auto\"");
        assert_eq!(
            serde_json::to_string(&ShareMode::Collaborative).unwrap(),
            "\"collaborative\""
        );
    }

    #[test]
    fn test_share_mode_serialization() {
        let json = serde_json::json!("collaborative");
        let mode: ShareMode = serde_json::from_value(json).unwrap();
        assert!(matches!(mode, ShareMode::Collaborative));
    }

    #[test]
    fn test_auto_update_serialization_bool() {
        let json = serde_json::json!(true);
        let update: AutoUpdate = serde_json::from_value(json).unwrap();
        assert!(matches!(update, AutoUpdate::Bool(true)));
    }

    #[test]
    fn test_auto_update_serialization_string() {
        let json = serde_json::json!("notify");
        let update: AutoUpdate = serde_json::from_value(json).unwrap();
        assert!(matches!(update, AutoUpdate::Notify(s) if s == "notify"));
    }

    #[test]
    fn test_permission_action_serialization() {
        let json = serde_json::json!("allow");
        let action: PermissionAction = serde_json::from_value(json).unwrap();
        assert!(matches!(action, PermissionAction::Allow));
    }

    #[test]
    fn test_permission_rule_action() {
        let json = serde_json::json!("deny");
        let rule: PermissionRule = serde_json::from_value(json).unwrap();
        assert!(matches!(
            rule,
            PermissionRule::Action(PermissionAction::Deny)
        ));
    }

    #[test]
    fn test_permission_rule_object() {
        let json = serde_json::json!({"file1": "allow", "file2": "deny"});
        let rule: PermissionRule = serde_json::from_value(json).unwrap();
        assert!(matches!(rule, PermissionRule::Object(_)));
    }

    #[test]
    fn test_cli_override_config() {
        let config = CliOverrideConfig {
            model: Some("gpt-4".to_string()),
            provider: Some("openai".to_string()),
            temperature: Some(0.5),
            max_tokens: Some(2000),
            default_agent: Some("build".to_string()),
        };
        assert_eq!(config.model, Some("gpt-4".to_string()));
        assert_eq!(config.provider, Some("openai".to_string()));
    }

    #[test]
    fn test_model_config_extra() {
        let json = serde_json::json!({
            "id": "gpt-4",
            "name": "GPT-4",
            "extra_field": "extra_value"
        });
        let config: ModelConfig = serde_json::from_value(json).unwrap();
        assert!(config.extra.is_some());
    }

    #[test]
    fn test_variant_config_extra() {
        let json = serde_json::json!({
            "disabled": false,
            "custom": "value"
        });
        let config: VariantConfig = serde_json::from_value(json).unwrap();
        assert!(config.extra.is_some());
    }

    #[test]
    fn test_command_config() {
        let json = serde_json::json!({
            "template": "echo hello",
            "description": "A test command",
            "agent": "build",
            "model": "gpt-4",
            "subtask": true
        });
        let config: CommandConfig = serde_json::from_value(json).unwrap();
        assert_eq!(config.template, "echo hello");
        assert!(config.description.is_some());
        assert!(config.agent.is_some());
        assert!(config.model.is_some());
        assert!(config.subtask.is_some());
    }

    #[test]
    fn test_skills_config() {
        let json = serde_json::json!({
            "paths": ["/path/to/skills"],
            "urls": ["https://example.com/skills"]
        });
        let config: SkillsConfig = serde_json::from_value(json).unwrap();
        assert!(config.paths.is_some());
        assert!(config.urls.is_some());
    }

    #[test]
    fn test_watcher_config() {
        let json = serde_json::json!({
            "ignore": ["*.log", "node_modules"]
        });
        let config: WatcherConfig = serde_json::from_value(json).unwrap();
        assert!(config.ignore.is_some());
        assert_eq!(config.ignore.as_ref().unwrap().len(), 2);
    }

    #[test]
    fn test_agent_config_all_fields() {
        let json = serde_json::json!({
            "model": "gpt-4",
            "variant": "chat",
            "temperature": 0.7,
            "top_p": 0.9,
            "prompt": "You are helpful",
            "disable": false,
            "description": "A build agent",
            "hidden": false,
            "color": "#00ff00",
            "steps": 10,
            "max_steps": 100
        });
        let config: AgentConfig = serde_json::from_value(json).unwrap();
        assert_eq!(config.model, Some("gpt-4".to_string()));
        assert_eq!(config.variant, Some("chat".to_string()));
        assert_eq!(config.temperature, Some(0.7));
        assert_eq!(config.top_p, Some(0.9));
        assert_eq!(config.prompt, Some("You are helpful".to_string()));
        assert!(!config.disable.unwrap());
        assert_eq!(config.description, Some("A build agent".to_string()));
        assert!(!config.hidden.unwrap());
        assert_eq!(config.color, Some("#00ff00".to_string()));
        assert_eq!(config.steps, Some(10));
        assert_eq!(config.max_steps, Some(100));
    }

    #[test]
    fn test_enterprise_config() {
        let json = serde_json::json!({
            "url": "https://enterprise.example.com",
            "remoteConfigDomain": "config.enterprise.example.com"
        });
        let config: EnterpriseConfig = serde_json::from_value(json).unwrap();
        assert!(config.url.is_some());
        assert!(config.remote_config_domain.is_some());
    }

    #[test]
    fn test_compaction_config() {
        let json = serde_json::json!({
            "auto": true,
            "prune": false,
            "reserved": 1000,
            "warning_threshold": 0.8,
            "compact_threshold": 0.9,
            "continuation_threshold": 0.5,
            "preserve_recent_messages": 50,
            "preserve_system_messages": true,
            "summary_prefix": "Summary:"
        });
        let config: CompactionConfig = serde_json::from_value(json).unwrap();
        assert!(config.auto.unwrap());
        assert!(!config.prune.unwrap());
        assert_eq!(config.reserved, Some(1000));
    }

    #[test]
    fn test_experimental_config() {
        let json = serde_json::json!({
            "disable_paste_summary": true,
            "batch_tool": true,
            "open_telemetry": false,
            "primary_tools": ["read", "edit"],
            "continue_loop_on_deny": true,
            "mcp_timeout": 30000
        });
        let config: ExperimentalConfig = serde_json::from_value(json).unwrap();
        assert!(config.disable_paste_summary.unwrap());
        assert!(config.batch_tool.unwrap());
        assert!(!config.open_telemetry.unwrap());
    }

    #[test]
    fn test_keybind_config_serialization() {
        let json = serde_json::json!({
            "commands": "ctrl-x",
            "timeline": "ctrl-t",
            "settings": "ctrl-,",
            "custom": {"custom1": "ctrl-q"}
        });
        let config: KeybindConfig = serde_json::from_value(json).unwrap();
        assert_eq!(config.commands, Some("ctrl-x".to_string()));
        assert_eq!(config.timeline, Some("ctrl-t".to_string()));
    }

    #[test]
    fn test_theme_config_with_scan_dirs() {
        let json = serde_json::json!({
            "name": "dark",
            "path": "/themes/dark",
            "scan_dirs": ["/usr/share/themes"]
        });
        let config: ThemeConfig = serde_json::from_value(json).unwrap();
        assert!(config.scan_dirs.is_some());
    }

    #[test]
    fn test_validate_tui_config_no_runtime_fields() {
        let json = serde_json::json!({
            "scroll_speed": 50,
            "theme": {"name": "dark"}
        });
        let errors = Config::validate_tui_config_no_runtime_fields(&json);
        assert!(errors.is_empty());
    }

    #[test]
    fn test_validate_tui_config_with_runtime_fields() {
        let json = serde_json::json!({
            "scroll_speed": 50,
            "tui": {"theme": "dark"}
        });
        let errors = Config::validate_tui_config_no_runtime_fields(&json);
        assert!(!errors.is_empty());
        assert!(errors.contains(&"tui".to_string()));
    }

    #[test]
    fn test_validate_runtime_no_tui_fields_empty() {
        let json = serde_json::json!({
            "model": "gpt-4",
            "temperature": 0.7
        });
        let errors = Config::validate_runtime_no_tui_fields(&json);
        assert!(errors.is_empty());
    }

    #[test]
    fn test_validate_runtime_no_tui_fields_with_tui_fields() {
        let json = serde_json::json!({
            "model": "gpt-4",
            "scroll_speed": 50,
            "tui": {}
        });
        let errors = Config::validate_runtime_no_tui_fields(&json);
        assert!(!errors.is_empty());
        assert!(errors.contains(&"scroll_speed".to_string()));
        assert!(errors.contains(&"tui".to_string()));
    }

    #[test]
    fn test_tui_config_default() {
        let config = TuiConfig::default();
        assert!(config.schema.is_none());
        assert!(config.scroll_speed.is_none());
    }

    #[test]
    fn test_validation_result_default() {
        let result = ValidationResult::default();
        assert!(!result.valid);
        assert!(result.errors.is_empty());
    }
}
