use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;
use std::path::{Path, PathBuf};

mod directory_scanner;
mod jsonc;
mod merge;
mod schema;
pub use directory_scanner::{load_opencode_directory, OpencodeDirectoryScan};
pub use jsonc::{is_jsonc_extension, parse_jsonc, JsoncError};

/// Main configuration structure matching the TypeScript Config.Info schema
#[derive(Debug, Clone, Deserialize, Serialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct Config {
    /// JSON schema reference for configuration validation
    #[serde(rename = "$schema", skip_serializing_if = "Option::is_none")]
    pub schema: Option<String>,

    /// Log level
    #[serde(skip_serializing_if = "Option::is_none")]
    pub log_level: Option<LogLevel>,

    /// Server configuration for opencode serve and web commands
    #[serde(skip_serializing_if = "Option::is_none")]
    pub server: Option<ServerConfig>,

    /// Command configuration
    #[serde(skip_serializing_if = "Option::is_none")]
    pub command: Option<HashMap<String, CommandConfig>>,

    /// Additional skill folder paths
    #[serde(skip_serializing_if = "Option::is_none")]
    pub skills: Option<SkillsConfig>,

    /// File watcher configuration
    #[serde(skip_serializing_if = "Option::is_none")]
    pub watcher: Option<WatcherConfig>,

    /// Plugin list
    #[serde(skip_serializing_if = "Option::is_none")]
    pub plugin: Option<Vec<String>>,

    /// Enable or disable snapshot tracking
    #[serde(skip_serializing_if = "Option::is_none")]
    pub snapshot: Option<bool>,

    /// Control sharing behavior: 'manual', 'auto', or 'disabled'
    #[serde(skip_serializing_if = "Option::is_none")]
    pub share: Option<ShareMode>,

    /// Deprecated: Use 'share' field instead
    #[serde(skip_serializing_if = "Option::is_none")]
    pub autoshare: Option<bool>,

    /// Automatically update to the latest version
    #[serde(skip_serializing_if = "Option::is_none")]
    pub autoupdate: Option<AutoUpdate>,

    /// Disable providers that are loaded automatically
    #[serde(skip_serializing_if = "Option::is_none")]
    pub disabled_providers: Option<Vec<String>>,

    /// When set, ONLY these providers will be enabled
    #[serde(skip_serializing_if = "Option::is_none")]
    pub enabled_providers: Option<Vec<String>>,

    /// Model to use in the format of provider/model
    #[serde(skip_serializing_if = "Option::is_none")]
    pub model: Option<String>,

    /// Small model to use for tasks like title generation
    #[serde(skip_serializing_if = "Option::is_none")]
    pub small_model: Option<String>,

    /// Default agent to use when none is specified
    #[serde(skip_serializing_if = "Option::is_none")]
    pub default_agent: Option<String>,

    /// Custom username to display in conversations
    #[serde(skip_serializing_if = "Option::is_none")]
    pub username: Option<String>,

    /// Deprecated: Use `agent` field instead
    #[deprecated(since = "2.0.0", note = "Use 'agent' field instead")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mode: Option<HashMap<String, AgentConfig>>,

    /// Agent configuration
    #[serde(skip_serializing_if = "Option::is_none")]
    pub agent: Option<AgentMapConfig>,

    /// Custom provider configurations and model overrides
    #[serde(skip_serializing_if = "Option::is_none")]
    pub provider: Option<HashMap<String, ProviderConfig>>,

    /// MCP (Model Context Protocol) server configurations
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mcp: Option<HashMap<String, McpConfig>>,

    /// Formatter configuration
    #[serde(skip_serializing_if = "Option::is_none")]
    pub formatter: Option<FormatterConfig>,

    /// LSP server configuration
    #[serde(skip_serializing_if = "Option::is_none")]
    pub lsp: Option<LspConfig>,

    /// Additional instruction files or patterns to include
    #[serde(skip_serializing_if = "Option::is_none")]
    pub instructions: Option<Vec<String>>,

    /// Deprecated: Always uses stretch layout
    #[deprecated(since = "2.0.0", note = "Layout is always stretch now")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub layout: Option<Layout>,

    /// Permission configuration
    #[serde(skip_serializing_if = "Option::is_none")]
    pub permission: Option<PermissionConfig>,

    /// Legacy tools configuration (deprecated, use permission instead)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tools: Option<HashMap<String, bool>>,

    /// Enterprise configuration
    #[serde(skip_serializing_if = "Option::is_none")]
    pub enterprise: Option<EnterpriseConfig>,

    /// Compaction configuration
    #[serde(skip_serializing_if = "Option::is_none")]
    pub compaction: Option<CompactionConfig>,

    /// Experimental features
    #[serde(skip_serializing_if = "Option::is_none")]
    pub experimental: Option<ExperimentalConfig>,

    /// Keybind configuration
    #[deprecated(since = "3.0.0", note = "Move to tui.json")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub keybinds: Option<KeybindConfig>,

    /// Theme configuration
    #[deprecated(since = "3.0.0", note = "Move to tui.json")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub theme: Option<ThemeConfig>,

    /// TUI (Terminal UI) configuration
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tui: Option<TuiConfig>,

    // Legacy fields for backwards compatibility
    /// API key for the provider
    #[serde(skip_serializing_if = "Option::is_none")]
    pub api_key: Option<String>,

    /// Temperature setting
    #[serde(skip_serializing_if = "Option::is_none")]
    pub temperature: Option<f32>,

    /// Maximum tokens
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_tokens: Option<u32>,
}

/// Log level enumeration
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum LogLevel {
    Trace,
    Debug,
    Info,
    Warn,
    Error,
}

/// Server configuration
#[derive(Debug, Clone, Deserialize, Serialize, Default)]
pub struct ServerConfig {
    /// Port to listen on
    #[serde(skip_serializing_if = "Option::is_none")]
    pub port: Option<u16>,

    /// Hostname to listen on
    #[serde(skip_serializing_if = "Option::is_none")]
    pub hostname: Option<String>,

    /// Enable mDNS service discovery
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mdns: Option<bool>,

    /// Custom domain name for mDNS service
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mdns_domain: Option<String>,

    /// Additional domains to allow for CORS
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cors: Option<Vec<String>>,
}

/// Command configuration
#[derive(Debug, Clone, Deserialize, Serialize, Default)]
pub struct CommandConfig {
    /// Command template
    pub template: String,

    /// Command description
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,

    /// Agent to use for this command
    #[serde(skip_serializing_if = "Option::is_none")]
    pub agent: Option<String>,

    /// Model to use for this command
    #[serde(skip_serializing_if = "Option::is_none")]
    pub model: Option<String>,

    /// Whether this command runs as a subtask
    #[serde(skip_serializing_if = "Option::is_none")]
    pub subtask: Option<bool>,
}

/// Skills configuration
#[derive(Debug, Clone, Deserialize, Serialize, Default)]
pub struct SkillsConfig {
    /// Additional paths to skill folders
    #[serde(skip_serializing_if = "Option::is_none")]
    pub paths: Option<Vec<String>>,

    /// URLs to fetch skills from
    #[serde(skip_serializing_if = "Option::is_none")]
    pub urls: Option<Vec<String>>,
}

/// Watcher configuration
#[derive(Debug, Clone, Deserialize, Serialize, Default)]
pub struct WatcherConfig {
    /// Patterns to ignore
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ignore: Option<Vec<String>>,
}

/// Share mode enumeration
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum ShareMode {
    Manual,
    Auto,
    Disabled,
}

/// Auto-update setting
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(untagged)]
pub enum AutoUpdate {
    Bool(bool),
    Notify(String),
}

#[derive(Debug, Clone, Default)]
pub struct AgentMapConfig {
    pub agents: HashMap<String, AgentConfig>,
    pub default_agent: Option<String>,
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

impl<'de> Deserialize<'de> for AgentMapConfig {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let value = serde_json::Value::deserialize(deserializer)?;
        let obj = value
            .as_object()
            .ok_or_else(|| serde::de::Error::custom("agent config must be an object"))?;

        let mut agents: HashMap<String, AgentConfig> = HashMap::new();
        let mut default_agent = None;

        if let Some(raw_default) = obj.get("default_agent").or_else(|| obj.get("defaultAgent")) {
            default_agent = Some(
                serde_json::from_value::<String>(raw_default.clone())
                    .map_err(serde::de::Error::custom)?,
            );
        }

        for (key, raw_value) in obj {
            if key == "agents" || key == "default_agent" || key == "defaultAgent" {
                continue;
            }

            let agent = serde_json::from_value::<AgentConfig>(raw_value.clone())
                .map_err(serde::de::Error::custom)?;
            agents.insert(key.clone(), agent);
        }

        if let Some(raw_agents) = obj.get("agents") {
            let parsed_agents: HashMap<String, AgentConfig> =
                serde_json::from_value(raw_agents.clone()).map_err(serde::de::Error::custom)?;
            agents.extend(parsed_agents);
        }

        Ok(Self {
            agents,
            default_agent,
        })
    }
}

impl Serialize for AgentMapConfig {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        #[derive(Serialize)]
        struct AgentMapConfigOut<'a> {
            agents: &'a HashMap<String, AgentConfig>,
            #[serde(skip_serializing_if = "Option::is_none")]
            default_agent: &'a Option<String>,
        }

        AgentMapConfigOut {
            agents: &self.agents,
            default_agent: &self.default_agent,
        }
        .serialize(serializer)
    }
}

/// Agent configuration
#[derive(Debug, Clone, Deserialize, Serialize, Default)]
pub struct AgentConfig {
    /// Model to use
    #[serde(skip_serializing_if = "Option::is_none")]
    pub model: Option<String>,

    /// Model variant
    #[serde(skip_serializing_if = "Option::is_none")]
    pub variant: Option<String>,

    /// Temperature setting
    #[serde(skip_serializing_if = "Option::is_none")]
    pub temperature: Option<f32>,

    /// Top-p sampling
    #[serde(skip_serializing_if = "Option::is_none")]
    pub top_p: Option<f32>,

    /// Agent prompt
    #[serde(skip_serializing_if = "Option::is_none")]
    pub prompt: Option<String>,

    /// Deprecated: Use 'permission' field instead
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tools: Option<HashMap<String, bool>>,

    /// Disable this agent
    #[serde(skip_serializing_if = "Option::is_none")]
    pub disable: Option<bool>,

    /// Description of when to use the agent
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,

    /// Agent mode
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mode: Option<AgentMode>,

    /// Hide from @ autocomplete menu
    #[serde(skip_serializing_if = "Option::is_none")]
    pub hidden: Option<bool>,

    /// Additional options
    #[serde(skip_serializing_if = "Option::is_none")]
    pub options: Option<HashMap<String, serde_json::Value>>,

    /// Hex color code or theme color
    #[serde(skip_serializing_if = "Option::is_none")]
    pub color: Option<String>,

    /// Maximum number of agentic iterations
    #[serde(skip_serializing_if = "Option::is_none")]
    pub steps: Option<u32>,

    /// Deprecated: Use 'steps' field instead
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_steps: Option<u32>,

    /// Permission configuration
    #[serde(skip_serializing_if = "Option::is_none")]
    pub permission: Option<PermissionConfig>,
}

/// Agent mode enumeration
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum AgentMode {
    Subagent,
    Primary,
    All,
}

/// Provider configuration with model overrides and options
#[derive(Debug, Clone, Deserialize, Serialize, Default)]
pub struct ProviderConfig {
    /// Provider ID
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,

    /// Provider name
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,

    /// Whitelist of models
    #[serde(skip_serializing_if = "Option::is_none")]
    pub whitelist: Option<Vec<String>>,

    /// Blacklist of models
    #[serde(skip_serializing_if = "Option::is_none")]
    pub blacklist: Option<Vec<String>>,

    /// Model configurations
    #[serde(skip_serializing_if = "Option::is_none")]
    pub models: Option<HashMap<String, ModelConfig>>,

    /// Provider options
    #[serde(skip_serializing_if = "Option::is_none")]
    pub options: Option<ProviderOptions>,
}

/// Model configuration within a provider
#[derive(Debug, Clone, Deserialize, Serialize, Default)]
pub struct ModelConfig {
    /// Model ID
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,

    /// Model name
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,

    /// Variant-specific configurations
    #[serde(skip_serializing_if = "Option::is_none")]
    pub variants: Option<HashMap<String, VariantConfig>>,

    /// Visibility status
    #[serde(skip_serializing_if = "Option::is_none")]
    pub visible: Option<bool>,

    /// Additional model properties
    #[serde(flatten, skip_serializing_if = "Option::is_none")]
    pub extra: Option<HashMap<String, serde_json::Value>>,
}

/// Variant configuration
#[derive(Debug, Clone, Deserialize, Serialize, Default)]
pub struct VariantConfig {
    /// Disable this variant
    #[serde(skip_serializing_if = "Option::is_none")]
    pub disabled: Option<bool>,

    /// Additional variant properties
    #[serde(flatten, skip_serializing_if = "Option::is_none")]
    pub extra: Option<HashMap<String, serde_json::Value>>,
}

/// Provider options
#[derive(Debug, Clone, Deserialize, Serialize, Default)]
pub struct ProviderOptions {
    /// API key
    #[serde(skip_serializing_if = "Option::is_none")]
    pub api_key: Option<String>,

    /// Base URL
    #[serde(skip_serializing_if = "Option::is_none")]
    pub base_url: Option<String>,

    /// Enterprise URL (for GitHub Copilot)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub enterprise_url: Option<String>,

    /// Enable prompt cache key
    #[serde(skip_serializing_if = "Option::is_none")]
    pub set_cache_key: Option<bool>,

    /// Timeout in milliseconds
    #[serde(skip_serializing_if = "Option::is_none")]
    pub timeout: Option<TimeoutConfig>,

    /// Chunk timeout in milliseconds
    #[serde(skip_serializing_if = "Option::is_none")]
    pub chunk_timeout: Option<u64>,

    /// AWS region for Bedrock provider
    #[serde(skip_serializing_if = "Option::is_none", rename = "awsRegion")]
    pub aws_region: Option<String>,

    /// AWS profile for Bedrock provider
    #[serde(skip_serializing_if = "Option::is_none", rename = "awsProfile")]
    pub aws_profile: Option<String>,

    /// Custom endpoint for Bedrock provider
    #[serde(skip_serializing_if = "Option::is_none", rename = "awsEndpoint")]
    pub aws_endpoint: Option<String>,

    /// Additional options
    #[serde(flatten, skip_serializing_if = "Option::is_none")]
    pub extra: Option<HashMap<String, serde_json::Value>>,
}

/// Timeout configuration - can be a number or false to disable
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(untagged)]
pub enum TimeoutConfig {
    Milliseconds(u64),
    NoTimeout(bool),
}

/// MCP server configuration
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(tag = "type", rename_all = "lowercase")]
pub enum McpConfig {
    Local(McpLocalConfig),
    Remote(McpRemoteConfig),
    /// Simple enabled/disabled config
    Simple {
        enabled: bool,
    },
}

/// Local MCP server configuration
#[derive(Debug, Clone, Deserialize, Serialize, Default)]
pub struct McpLocalConfig {
    /// Command and arguments to run the MCP server
    pub command: Vec<String>,

    /// Environment variables
    #[serde(skip_serializing_if = "Option::is_none")]
    pub environment: Option<HashMap<String, String>>,

    /// Enable or disable the MCP server on startup
    #[serde(skip_serializing_if = "Option::is_none")]
    pub enabled: Option<bool>,

    /// Timeout in ms for MCP server requests
    #[serde(skip_serializing_if = "Option::is_none")]
    pub timeout: Option<u64>,
}

/// OAuth configuration for remote MCP
#[derive(Debug, Clone, Deserialize, Serialize, Default)]
pub struct McpOAuthConfig {
    /// OAuth client ID
    #[serde(skip_serializing_if = "Option::is_none")]
    pub client_id: Option<String>,

    /// OAuth client secret
    #[serde(skip_serializing_if = "Option::is_none")]
    pub client_secret: Option<String>,

    /// OAuth scopes
    #[serde(skip_serializing_if = "Option::is_none")]
    pub scope: Option<String>,
}

/// Remote MCP server configuration
#[derive(Debug, Clone, Deserialize, Serialize, Default)]
pub struct McpRemoteConfig {
    /// URL of the remote MCP server
    pub url: String,

    /// Enable or disable the MCP server on startup
    #[serde(skip_serializing_if = "Option::is_none")]
    pub enabled: Option<bool>,

    /// Headers to send with the request
    #[serde(skip_serializing_if = "Option::is_none")]
    pub headers: Option<HashMap<String, String>>,

    /// OAuth authentication configuration
    #[serde(skip_serializing_if = "Option::is_none")]
    pub oauth: Option<McpOAuthUnion>,

    /// Timeout in ms for MCP server requests
    #[serde(skip_serializing_if = "Option::is_none")]
    pub timeout: Option<u64>,
}

/// OAuth can be config or false to disable
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(untagged)]
pub enum McpOAuthUnion {
    Config(McpOAuthConfig),
    Disabled(bool),
}

/// Formatter configuration - false or record of formatters
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(untagged)]
pub enum FormatterConfig {
    Disabled(bool),
    Formatters(HashMap<String, FormatterEntry>),
}

/// Individual formatter entry
#[derive(Debug, Clone, Deserialize, Serialize, Default)]
pub struct FormatterEntry {
    /// Disable this formatter
    #[serde(skip_serializing_if = "Option::is_none")]
    pub disabled: Option<bool>,

    /// Command to run
    #[serde(skip_serializing_if = "Option::is_none")]
    pub command: Option<Vec<String>>,

    /// Environment variables
    #[serde(skip_serializing_if = "Option::is_none")]
    pub environment: Option<HashMap<String, String>>,

    /// File extensions to apply to
    #[serde(skip_serializing_if = "Option::is_none")]
    pub extensions: Option<Vec<String>>,
}

/// LSP configuration - false or record of LSP servers
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(untagged)]
pub enum LspConfig {
    Disabled(bool),
    Servers(HashMap<String, LspEntry>),
}

/// Individual LSP entry
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

/// Permission action
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum PermissionAction {
    Ask,
    Allow,
    Deny,
}

/// Permission rule - either an action or a nested object
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(untagged)]
pub enum PermissionRule {
    Action(PermissionAction),
    Object(HashMap<String, PermissionAction>),
}

/// Permission configuration
#[derive(Debug, Clone, Deserialize, Serialize, Default)]
pub struct PermissionConfig {
    /// Read permission
    #[serde(skip_serializing_if = "Option::is_none")]
    pub read: Option<PermissionRule>,

    /// Edit permission
    #[serde(skip_serializing_if = "Option::is_none")]
    pub edit: Option<PermissionRule>,

    /// Glob permission
    #[serde(skip_serializing_if = "Option::is_none")]
    pub glob: Option<PermissionRule>,

    /// Grep permission
    #[serde(skip_serializing_if = "Option::is_none")]
    pub grep: Option<PermissionRule>,

    /// List permission
    #[serde(skip_serializing_if = "Option::is_none")]
    pub list: Option<PermissionRule>,

    /// Bash permission
    #[serde(skip_serializing_if = "Option::is_none")]
    pub bash: Option<PermissionRule>,

    /// Task permission
    #[serde(skip_serializing_if = "Option::is_none")]
    pub task: Option<PermissionRule>,

    /// External directory permission
    #[serde(skip_serializing_if = "Option::is_none")]
    pub external_directory: Option<PermissionRule>,

    /// Todo write permission
    #[serde(skip_serializing_if = "Option::is_none")]
    pub todowrite: Option<PermissionAction>,

    /// Question permission
    #[serde(skip_serializing_if = "Option::is_none")]
    pub question: Option<PermissionAction>,

    /// Web fetch permission
    #[serde(skip_serializing_if = "Option::is_none")]
    pub webfetch: Option<PermissionAction>,

    /// Web search permission
    #[serde(skip_serializing_if = "Option::is_none")]
    pub websearch: Option<PermissionAction>,

    /// Code search permission
    #[serde(skip_serializing_if = "Option::is_none")]
    pub codesearch: Option<PermissionAction>,

    /// LSP permission
    #[serde(skip_serializing_if = "Option::is_none")]
    pub lsp: Option<PermissionRule>,

    /// Doom loop permission
    #[serde(skip_serializing_if = "Option::is_none")]
    pub doom_loop: Option<PermissionAction>,

    /// Skill permission
    #[serde(skip_serializing_if = "Option::is_none")]
    pub skill: Option<PermissionRule>,

    /// Catch-all for other permissions
    #[serde(flatten, skip_serializing_if = "Option::is_none")]
    pub extra: Option<HashMap<String, PermissionRule>>,
}

/// Layout enumeration
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum Layout {
    Auto,
    Stretch,
}

/// Enterprise configuration
#[derive(Debug, Clone, Deserialize, Serialize, Default)]
pub struct EnterpriseConfig {
    /// Enterprise URL
    #[serde(skip_serializing_if = "Option::is_none")]
    pub url: Option<String>,
}

/// Compaction configuration
#[derive(Debug, Clone, Deserialize, Serialize, Default)]
pub struct CompactionConfig {
    /// Enable automatic compaction when context is full
    #[serde(skip_serializing_if = "Option::is_none")]
    pub auto: Option<bool>,

    /// Enable pruning of old tool outputs
    #[serde(skip_serializing_if = "Option::is_none")]
    pub prune: Option<bool>,

    /// Token buffer for compaction
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reserved: Option<u32>,
}

/// Experimental features configuration
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

        let mut conflicts = Vec::new();
        let mut reverse: std::collections::HashMap<String, Vec<String>> = std::collections::HashMap::new();
        for (action, binding) in Self::bindings_with_labels(&merged) {
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

        (merged, conflicts)
    }

    fn bindings_with_labels(config: &KeybindConfig) -> Vec<(String, String)> {
        let mut out = Vec::new();
        if let Some(v) = &config.commands { out.push(("commands".to_string(), v.clone())); }
        if let Some(v) = &config.timeline { out.push(("timeline".to_string(), v.clone())); }
        if let Some(v) = &config.settings { out.push(("settings".to_string(), v.clone())); }
        if let Some(v) = &config.models { out.push(("models".to_string(), v.clone())); }
        if let Some(v) = &config.files { out.push(("files".to_string(), v.clone())); }
        if let Some(v) = &config.terminal { out.push(("terminal".to_string(), v.clone())); }
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

        let resolved = if raw == "~" {
            dirs::home_dir()?
        } else if let Some(stripped) = raw.strip_prefix("~/") {
            dirs::home_dir()?.join(stripped)
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

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct TuiConfig {
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
}

impl Default for TuiConfig {
    fn default() -> Self {
        Self {
            scroll_speed: None,
            scroll_acceleration: None,
            diff_style: None,
            theme: None,
            keybinds: None,
        }
    }
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
                formatter.write_str("a number (legacy) or {{ enabled: bool, speed?: f32 }}")
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
                        _ => { let _: serde::de::IgnoredAny = map.next_value()?; }
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

/// Legacy provider configuration enum for backwards compatibility
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum LegacyProvider {
    Openai,
    Anthropic,
    Ollama,
}

impl Default for LegacyProvider {
    fn default() -> Self {
        LegacyProvider::Openai
    }
}

impl Config {
    /// Load configuration from a file path
    pub fn load(path: &PathBuf) -> Result<Self, crate::OpenCodeError> {
        let mut config = if !path.exists() {
            Config::default()
        } else {
            let content = std::fs::read_to_string(path)?;
            let content = Self::substitute_variables(&content, path.parent());
            if path.extension().and_then(|s| s.to_str()) == Some("json")
                || path.extension().and_then(|s| s.to_str()) == Some("jsonc")
                || path.extension().and_then(|s| s.to_str()) == Some("json5")
            {
                Self::parse_json_content(&content)?
            } else {
                toml::from_str(&content).map_err(|e| crate::OpenCodeError::Config(e.to_string()))?
            }
        };

        Self::log_schema_validation(&config);

        config.apply_env_overrides();
        Ok(config)
    }

    /// Parse JSON content with JSONC fallback (handles comments)
    fn parse_json_content(content: &str) -> Result<Self, crate::OpenCodeError> {
        use crate::config::jsonc;

        if let Ok(config) = serde_json::from_str::<Config>(content) {
            return Ok(config);
        }

        let stripped = jsonc::strip_jsonc_comments(content);
        serde_json::from_str(&stripped).map_err(|e| crate::OpenCodeError::Config(e.to_string()))
    }

    /// Substitute {env:VAR} and {file:path} patterns in config content
    pub fn substitute_variables(input: &str, config_dir: Option<&Path>) -> String {
        let mut result = input.to_string();

        // Pattern: {env:VARIABLE_NAME}
        while let Some(start) = result.find("{env:") {
            if let Some(end) = result[start..].find('}') {
                let var_name = &result[start + 5..start + end];
                let replacement = std::env::var(var_name).unwrap_or_default();
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

        // Pattern: {file:path/to/file}
        while let Some(start) = result.find("{file:") {
            if let Some(end) = result[start..].find('}') {
                let file_path = &result[start + 6..start + end];
                let replacement = match Self::resolve_file_variable_path(file_path, config_dir) {
                    Some(path) => std::fs::read_to_string(&path)
                        .unwrap_or_else(|_| format!("{{file:{}}}", file_path)),
                    None => String::new(),
                };
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

        result
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

        let toml = config_root.join("config.toml");
        if toml.exists() {
            return toml;
        }

        json
    }

    fn warn_legacy_config_dir_if_exists() {
        if let Some(home) = dirs::home_dir().or_else(|| std::env::var("HOME").ok().map(PathBuf::from)) {
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
            "theme",
            "keybinds",
            "scroll_speed",
            "scrollSpeed",
            "scroll_acceleration",
            "scrollAcceleration",
            "diff_style",
            "diffStyle",
        ];

        obj.keys()
            .filter(|key| TUI_FIELDS.contains(&key.as_str()))
            .cloned()
            .collect()
    }

    fn parse_tui_config_file(path: &Path) -> Result<Option<TuiConfig>, crate::OpenCodeError> {
        if !path.exists() {
            return Ok(None);
        }

        let content = std::fs::read_to_string(path)?;
        let value = parse_jsonc(&content).map_err(|e| crate::OpenCodeError::Config(e.to_string()))?;

        let invalid_runtime_fields = Self::validate_tui_config_no_runtime_fields(&value);
        if !invalid_runtime_fields.is_empty() {
            tracing::warn!(
                "Ignoring runtime fields in tui config {}: {}",
                path.display(),
                invalid_runtime_fields.join(", ")
            );
        }

        let schema_errors = schema::validate_tui_schema(&value);
        if !schema_errors.is_empty() {
            return Err(crate::OpenCodeError::Config(format!(
                "Invalid TUI config {}: {}",
                path.display(),
                schema_errors.join("; ")
            )));
        }

        let config = serde_json::from_value::<TuiConfig>(value)
            .map_err(|e| crate::OpenCodeError::Config(format!("Invalid TUI config {}: {}", path.display(), e)))?;

        Ok(Some(config))
    }

    pub fn load_tui_config() -> Result<TuiConfig, crate::OpenCodeError> {
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
                let base = serde_json::to_value(&merged).unwrap_or(Value::Object(serde_json::Map::new()));
                let override_val = serde_json::to_value(&cfg).unwrap_or(Value::Object(serde_json::Map::new()));
                let merged_json = merge::deep_merge(&base, &override_val);
                merged = serde_json::from_value(merged_json).unwrap_or_default();
            }
        }

        Ok(merged)
    }

    fn warn_runtime_tui_fields(path: &Path) {
        if let Ok(content) = std::fs::read_to_string(path) {
            if let Ok(value) = parse_jsonc(&content) {
                let detected = Self::validate_runtime_no_tui_fields(&value);
                if !detected.is_empty() {
                    tracing::warn!(
                        "TUI fields in runtime config {} are deprecated and will be migrated: {}",
                        path.display(),
                        detected.join(", ")
                    );
                }
            }
        }
    }

    /// Load multi-tier config with precedence: remote → global → project → .opencode
    pub async fn load_multi() -> Result<Self, crate::OpenCodeError> {
        Self::warn_legacy_config_dir_if_exists();
        let mut configs: Vec<(String, Config)> = Vec::new();

        // Priority 1: Remote config from .well-known/opencode
        if let Ok(remote_url) = std::env::var("OPENCODE_REMOTE_CONFIG") {
            if let Ok(content) = Self::fetch_remote_config(&remote_url).await {
                if let Ok(config) = Self::parse_config_content(&content, "json") {
                    configs.push(("remote".to_string(), config));
                }
            }
        }

        // Priority 2: OPENCODE_CONFIG_CONTENT env var
        if let Ok(content) = std::env::var("OPENCODE_CONFIG_CONTENT") {
            let content = Self::substitute_variables(&content, None);
            if let Ok(config) = Self::parse_config_content(&content, "json") {
                configs.push(("env-content".to_string(), config));
            }
        }

        // Priority 3: OPENCODE_CONFIG path
        if let Ok(config_path) = std::env::var("OPENCODE_CONFIG") {
            let path = PathBuf::from(config_path);
            if path.exists() {
                Self::warn_runtime_tui_fields(&path);
                let config = Self::load(&path)?;
                configs.push(("env-path".to_string(), config));
            }
        }

        // Priority 4: Global config (~/.config/opencode/config.json)
        let global_path = Self::config_path();
        if global_path.exists() {
            Self::warn_runtime_tui_fields(&global_path);
            let config = Self::load(&global_path)?;
            configs.push(("global".to_string(), config));
        }

        // Priority 5: Project-level config (searching from current dir upwards)
        // Supports: opencode.json, opencode.json5, opencode.jsonc, .opencode/config.json, .opencode/config.json5, .opencode/config.jsonc
        if let Ok(cwd) = std::env::current_dir() {
            for ancestor in cwd.ancestors() {
                // Check for opencode.json, opencode.json5, opencode.jsonc
                for ext in &["json", "json5", "jsonc"] {
                    let project_config = ancestor.join(format!("opencode.{}", ext));
                    if project_config.exists() {
                        Self::warn_runtime_tui_fields(&project_config);
                        let config = Self::load(&project_config)?;
                        configs.push(("project".to_string(), config));
                        break;
                    }
                }
                // Also check for .opencode/config.json/json5/jsonc
                for ext in &["json", "json5", "jsonc"] {
                    let opencode_dir = ancestor.join(".opencode").join(format!("config.{}", ext));
                    if opencode_dir.exists() {
                        Self::warn_runtime_tui_fields(&opencode_dir);
                        let config = Self::load(&opencode_dir)?;
                        configs.push((".opencode".to_string(), config));
                        break;
                    }
                }
            }
        }

        // Merge all configs (later entries override earlier ones)
        let mut result = Config::default();
        for (_, config) in configs {
            result = Self::merge_configs(result, config);
        }

        // Priority 6: .opencode/ directory scan (agents, commands, modes, skills, tools, themes, plugins)
        // This merges discovered directory content into the final config.
        // Directory scanning is error-tolerant: missing directories log warnings, never block loading.
        Self::merge_opencode_directory_into_config(&mut result);

        let mut migrated_tui = result.tui.clone().unwrap_or_default();
        #[allow(deprecated)]
        {
            if result.theme.is_some() {
                tracing::warn!("'theme' in main config is deprecated since 3.0.0. Move it to tui.json.");
            }
            if result.keybinds.is_some() {
                tracing::warn!("'keybinds' in main config is deprecated since 3.0.0. Move it to tui.json.");
            }

            if migrated_tui.theme.is_none() {
                migrated_tui.theme = result.theme.clone();
            }
            if migrated_tui.keybinds.is_none() {
                migrated_tui.keybinds = result.keybinds.clone();
            }
        }

        let file_tui = Self::load_tui_config()?;
        let base = serde_json::to_value(&migrated_tui).unwrap_or(Value::Object(serde_json::Map::new()));
        let override_val = serde_json::to_value(&file_tui).unwrap_or(Value::Object(serde_json::Map::new()));
        let merged_tui = merge::deep_merge(&base, &override_val);
        result.tui = Some(serde_json::from_value(merged_tui).unwrap_or_default());

        result.apply_env_overrides();
        Ok(result)
    }

    /// Fetch remote configuration from URL
    async fn fetch_remote_config(url: &str) -> Result<String, crate::OpenCodeError> {
        let client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(10))
            .build()
            .map_err(|e| crate::OpenCodeError::Config(e.to_string()))?;

        let response = client
            .get(url)
            .header("Accept", "application/json")
            .send()
            .await
            .map_err(|e| crate::OpenCodeError::Config(e.to_string()))?;

        if response.status() == 401 || response.status() == 403 {
            return Err(crate::OpenCodeError::Config("Remote config authentication failed".to_string()));
        }

        let content = response
            .text()
            .await
            .map_err(|e| crate::OpenCodeError::Config(e.to_string()))?;

        Ok(content)
    }

    /// Build remote config URL from domain
    pub fn build_remote_url(domain: &str) -> String {
        let domain = domain.trim_end_matches('/');
        format!("{}/.well-known/opencode", domain)
    }

    /// Parse config content with specified format
    fn parse_config_content(content: &str, format: &str) -> Result<Self, crate::OpenCodeError> {
        if format == "json" || format == "jsonc" {
            if let Ok(config) = serde_json::from_str::<Config>(content) {
                Self::log_schema_validation(&config);
                return Ok(config);
            }
            let stripped = jsonc::strip_jsonc_comments(content);
            let config =
                serde_json::from_str(&stripped).map_err(|e| crate::OpenCodeError::Config(e.to_string()))?;
            Self::log_schema_validation(&config);
            Ok(config)
        } else {
            toml::from_str(content).map_err(|e| crate::OpenCodeError::Config(e.to_string()))
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
                agents.agents.entry(agent_info.name).or_insert_with(|| AgentConfig {
                    prompt: Some(agent_info.content),
                    ..Default::default()
                });
            }
        }

        if command_count > 0 {
            let commands = config.command.get_or_insert_with(HashMap::new);
            for cmd_info in scan.commands {
                let name = cmd_info.name.clone();
                let template = format!("# Command from {}\n{}", cmd_info.path.display(), cmd_info.content);
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

        if tool_count > 0 {
            let tools = config.tools.get_or_insert_with(HashMap::new);
            for tool_info in scan.tools {
                tools.entry(tool_info.name).or_insert(true);
            }
        }

        if theme_count > 0 {
            #[allow(deprecated)]
            if config.theme.is_none() {
                if let Some(first_theme) = scan.themes.first() {
                    config.theme = Some(ThemeConfig {
                        name: Some(first_theme.name.clone()),
                        path: Some(first_theme.path.clone()),
                        ..Default::default()
                    });
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

    /// Apply environment variable overrides
    fn apply_env_overrides(&mut self) {
        // Legacy provider override
        if let Ok(provider) = std::env::var("OPENCODE_PROVIDER") {
            // Set default provider configuration if not exists
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

        // New environment variable overrides
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

        // Experimental flags
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
                let config = providers.entry(provider_id.to_string()).or_insert_with(|| ProviderConfig {
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

    /// Get provider configuration for a given provider ID
    pub fn get_provider(&self, provider_id: &str) -> Option<&ProviderConfig> {
        self.provider.as_ref().and_then(|p| p.get(provider_id))
    }

    /// Check if a provider is enabled based on enabled_providers and disabled_providers lists
    pub fn is_provider_enabled(&self, provider_id: &str) -> bool {
        // If enabled_providers is set, provider must be in that list
        if let Some(enabled) = &self.enabled_providers {
            return enabled.iter().any(|p| p.eq_ignore_ascii_case(provider_id));
        }

        // Otherwise, check if provider is in disabled_providers
        if let Some(disabled) = &self.disabled_providers {
            return !disabled.iter().any(|p| p.eq_ignore_ascii_case(provider_id));
        }

        // Default: enabled
        true
    }

    /// Get experimental batch_tool flag
    pub fn batch_tool_enabled(&self) -> bool {
        self.experimental
            .as_ref()
            .and_then(|e| e.batch_tool)
            .unwrap_or(false)
    }

    /// Get experimental open_telemetry flag
    pub fn open_telemetry_enabled(&self) -> bool {
        self.experimental
            .as_ref()
            .and_then(|e| e.open_telemetry)
            .unwrap_or(false)
    }

    /// Validate the configuration and return a list of validation errors
    pub fn validate(&self) -> Vec<ValidationError> {
        let mut errors = Vec::new();

        // Validate model format (should be provider/model)
        if let Some(model) = &self.model {
            if !model.contains('/') {
                errors.push(ValidationError {
                    field: "model".to_string(),
                    message: format!("Model '{}' should be in format 'provider/model'", model),
                    severity: ValidationSeverity::Warning,
                });
            }
        }

        // Validate temperature range
        if let Some(temp) = self.temperature {
            if temp < 0.0 || temp > 2.0 {
                errors.push(ValidationError {
                    field: "temperature".to_string(),
                    message: format!("Temperature {} should be between 0.0 and 2.0", temp),
                    severity: ValidationSeverity::Error,
                });
            }
        }

        // Validate agent configurations
        if let Some(agents) = &self.agent {
            for (name, agent) in &agents.agents {
                if let Some(temp) = agent.temperature {
                    if temp < 0.0 || temp > 2.0 {
                        errors.push(ValidationError {
                            field: format!("agent.{}.temperature", name),
                            message: format!(
                                "Temperature {} should be between 0.0 and 2.0",
                                temp
                            ),
                            severity: ValidationSeverity::Error,
                        });
                    }
                }
                if let Some(top_p) = agent.top_p {
                    if top_p < 0.0 || top_p > 1.0 {
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

        // Validate provider configurations
        if let Some(providers) = &self.provider {
            for (name, provider) in providers {
                // Check for required fields in provider options
                if let Some(options) = &provider.options {
                    if name != "ollama" && options.api_key.is_none() {
                        // API key is typically required for cloud providers
                        // but we'll just warn since it might be set via env
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

        // Validate server configuration
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

        // Validate compaction configuration
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

    /// Check if configuration is valid (no errors)
    pub fn is_valid(&self) -> bool {
        self.validate().iter().all(|e| !e.is_error())
    }

    /// Validate config against JSON Schema
    pub fn validate_json_schema(&self, schema_url: Option<&str>) -> ValidationResult {
        let value = serde_json::to_value(self).unwrap_or(serde_json::Value::Null);
        schema::validate_json_schema(&value, schema_url.unwrap_or(""))
    }

    /// Save configuration to a file path
    pub fn save(&self, path: &PathBuf) -> Result<(), crate::OpenCodeError> {
        let content = if path.extension().and_then(|s| s.to_str()) == Some("json")
            || path.extension().and_then(|s| s.to_str()) == Some("jsonc")
        {
            serde_json::to_string_pretty(self)
                .map_err(|e| crate::OpenCodeError::Config(e.to_string()))?
        } else {
            toml::to_string_pretty(self)
                .map_err(|e| crate::OpenCodeError::Config(e.to_string()))?
        };

        // Ensure parent directory exists
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)
                .map_err(|e| crate::OpenCodeError::Config(e.to_string()))?;
        }

        std::fs::write(path, content).map_err(|e| crate::OpenCodeError::Config(e.to_string()))?;

        Ok(())
    }

    /// Save provider settings to config file
    pub fn save_provider_settings(
        &mut self,
        provider_id: &str,
        config: ProviderConfig,
    ) -> Result<(), crate::OpenCodeError> {
        let mut providers = self.provider.clone().unwrap_or_default();
        providers.insert(provider_id.to_string(), config);
        self.provider = Some(providers);

        // Save to default config path
        self.save(&Self::config_path())
    }

    /// Migrate from TypeScript-style JSON config format
    pub fn migrate_from_ts_format(json_content: &str) -> Result<Self, crate::OpenCodeError> {
        // Parse as generic JSON first
        let json_value: serde_json::Value = serde_json::from_str(json_content)
            .map_err(|e| crate::OpenCodeError::Config(e.to_string()))?;

        let mut config = Config::default();

        // Map common fields from TS format
        if let Some(obj) = json_value.as_object() {
            // Map logLevel -> log_level
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

            // Map model
            if let Some(model) = obj.get("model").and_then(|v| v.as_str()) {
                config.model = Some(model.to_string());
            }

            // Map smallModel -> small_model
            if let Some(small_model) = obj.get("smallModel").and_then(|v| v.as_str()) {
                config.small_model = Some(small_model.to_string());
            }

            // Map defaultAgent -> default_agent
            if let Some(default_agent) = obj.get("defaultAgent").and_then(|v| v.as_str()) {
                config.default_agent = Some(default_agent.to_string());
            }

            // Map username
            if let Some(username) = obj.get("username").and_then(|v| v.as_str()) {
                config.username = Some(username.to_string());
            }

            // Map apiKey -> api_key
            if let Some(api_key) = obj.get("apiKey").and_then(|v| v.as_str()) {
                config.api_key = Some(api_key.to_string());
            }

            // Map temperature
            if let Some(temp) = obj.get("temperature").and_then(|v| v.as_f64()) {
                config.temperature = Some(temp as f32);
            }

            // Map maxTokens -> max_tokens
            if let Some(max_tokens) = obj.get("maxTokens").and_then(|v| v.as_u64()) {
                config.max_tokens = Some(max_tokens as u32);
            }

            // Map providers
            if let Some(providers) = obj.get("providers").and_then(|v| v.as_object()) {
                let mut provider_map: HashMap<String, ProviderConfig> = HashMap::new();
                for (name, provider_json) in providers {
                    if let Some(provider_obj) = provider_json.as_object() {
                        let mut provider_config = ProviderConfig {
                            id: Some(name.clone()),
                            ..Default::default()
                        };

                        // Map provider options
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

            // Map disabledProviders -> disabled_providers
            if let Some(disabled) = obj.get("disabledProviders").and_then(|v| v.as_array()) {
                config.disabled_providers = Some(
                    disabled
                        .iter()
                        .filter_map(|v| v.as_str().map(|s| s.to_string()))
                        .collect(),
                );
            }

            // Map enabledProviders -> enabled_providers
            if let Some(enabled) = obj.get("enabledProviders").and_then(|v| v.as_array()) {
                config.enabled_providers = Some(
                    enabled
                        .iter()
                        .filter_map(|v| v.as_str().map(|s| s.to_string()))
                        .collect(),
                );
            }

            // Map share mode
            if let Some(share) = obj.get("share").and_then(|v| v.as_str()) {
                config.share = match share.to_lowercase().as_str() {
                    "manual" => Some(ShareMode::Manual),
                    "auto" => Some(ShareMode::Auto),
                    "disabled" => Some(ShareMode::Disabled),
                    _ => None,
                };
            }

            // Map autoUpdate -> autoupdate
            if let Some(autoupdate) = obj.get("autoUpdate") {
                if let Some(b) = autoupdate.as_bool() {
                    config.autoupdate = Some(AutoUpdate::Bool(b));
                } else if let Some(s) = autoupdate.as_str() {
                    config.autoupdate = Some(AutoUpdate::Notify(s.to_string()));
                }
            }

            // Map snapshot
            if let Some(snapshot) = obj.get("snapshot").and_then(|v| v.as_bool()) {
                config.snapshot = Some(snapshot);
            }
        }

        Ok(config)
    }
}

/// Validation error structure
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

/// Validation severity level
#[derive(Debug, Clone)]
pub enum ValidationSeverity {
    Error,
    Warning,
}

/// Result of validation
#[derive(Debug, Clone, Default)]
pub struct ValidationResult {
    pub valid: bool,
    pub errors: Vec<ValidationError>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::ffi::OsStr;
    use std::fs;
    use std::sync::{Arc, Mutex};
    use std::time::{SystemTime, UNIX_EPOCH};
    use tracing::field::{Field, Visit};
    use tracing::{Event, Subscriber};
    use tracing_subscriber::layer::{Context, Layer};
    use tracing_subscriber::prelude::*;
    use tracing_subscriber::registry::LookupSpan;

    #[derive(Default)]
    struct MessageVisitor {
        message: Option<String>,
    }

    impl Visit for MessageVisitor {
        fn record_str(&mut self, field: &Field, value: &str) {
            if field.name() == "message" {
                self.message = Some(value.to_string());
            }
        }

        fn record_debug(&mut self, field: &Field, value: &dyn std::fmt::Debug) {
            if field.name() == "message" && self.message.is_none() {
                self.message = Some(format!("{:?}", value));
            }
        }
    }

    struct WarnCaptureLayer {
        sink: Arc<Mutex<Vec<String>>>,
    }

    impl<S> Layer<S> for WarnCaptureLayer
    where
        S: Subscriber + for<'lookup> LookupSpan<'lookup>,
    {
        fn on_event(&self, event: &Event<'_>, _ctx: Context<'_, S>) {
            if *event.metadata().level() != tracing::Level::WARN {
                return;
            }

            let mut visitor = MessageVisitor::default();
            event.record(&mut visitor);
            if let Some(message) = visitor.message {
                self.sink.lock().unwrap().push(message);
            }
        }
    }

    fn unique_temp_dir(prefix: &str) -> PathBuf {
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let dir = std::env::temp_dir().join(format!("{}_{}", prefix, nanos));
        fs::create_dir_all(&dir).unwrap();
        dir
    }

    fn set_env<K: AsRef<OsStr>, V: AsRef<OsStr>>(key: K, value: V) {
        unsafe { std::env::set_var(key, value) }
    }

    fn remove_env<K: AsRef<OsStr>>(key: K) {
        unsafe { std::env::remove_var(key) }
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

        // Default: enabled
        assert!(config.is_provider_enabled("openai"));

        // Test enabled_providers
        config.enabled_providers = Some(vec!["anthropic".to_string()]);
        assert!(!config.is_provider_enabled("openai"));
        assert!(config.is_provider_enabled("anthropic"));

        // Test disabled_providers
        config.enabled_providers = None;
        config.disabled_providers = Some(vec!["ollama".to_string()]);
        assert!(config.is_provider_enabled("openai"));
        assert!(!config.is_provider_enabled("ollama"));
    }

    #[test]
    fn test_substitute_variables_env() {
        set_env("TEST_VAR", "test_value");
        let input = "key: {env:TEST_VAR}";
        let result = Config::substitute_variables(input, None);
        assert_eq!(result, "key: test_value");
        remove_env("TEST_VAR");
    }

    #[test]
    fn test_substitute_variables_missing_env() {
        remove_env("NONEXISTENT_VAR");
        let input = "key: {env:NONEXISTENT_VAR}";
        let result = Config::substitute_variables(input, None);
        assert_eq!(result, "key: ");
    }

    #[test]
    fn test_substitute_variables_multiple() {
        set_env("VAR1", "value1");
        set_env("VAR2", "value2");
        let input = "{env:VAR1} and {env:VAR2}";
        let result = Config::substitute_variables(input, None);
        assert_eq!(result, "value1 and value2");
        remove_env("VAR1");
        remove_env("VAR2");
    }

    #[test]
    fn test_substitute_file_tilde_expansion() {
        let temp_home = unique_temp_dir("opencode_home_expand");
        let file_path = temp_home.join(".test_file");
        fs::write(&file_path, "secret-value").unwrap();

        let old_home = std::env::var("HOME").ok();
        set_env("HOME", &temp_home);

        let result = Config::substitute_variables("{file:~/.test_file}", None);
        assert_eq!(result, "secret-value");

        if let Some(home) = old_home {
            set_env("HOME", home);
        } else {
            remove_env("HOME");
        }
        let _ = fs::remove_dir_all(temp_home);
    }

    #[test]
    fn test_substitute_file_tilde_expansion_failure_returns_empty() {
        let result = Config::substitute_variables("start-{file:~someone/path}-end", None);
        assert_eq!(result, "start--end");
    }

    #[test]
    fn test_substitute_file_relative_to_config_dir() {
        let config_dir = unique_temp_dir("opencode_config_dir_relative");
        let instructions = config_dir.join("instructions.md");
        fs::write(&instructions, "relative-content").unwrap();

        let result = Config::substitute_variables("{file:./instructions.md}", Some(&config_dir));
        assert_eq!(result, "relative-content");

        let _ = fs::remove_dir_all(config_dir);
    }

    #[test]
    fn test_substitute_file_parent_relative_to_config_dir() {
        let root_dir = unique_temp_dir("opencode_config_dir_parent");
        let config_dir = root_dir.join("config");
        let shared_dir = root_dir.join("shared");
        fs::create_dir_all(&config_dir).unwrap();
        fs::create_dir_all(&shared_dir).unwrap();
        fs::write(shared_dir.join("config.md"), "parent-content").unwrap();

        let result = Config::substitute_variables("{file:../shared/config.md}", Some(&config_dir));
        assert_eq!(result, "parent-content");

        let _ = fs::remove_dir_all(root_dir);
    }

    #[test]
    fn test_substitute_file_relative_without_config_dir_uses_cwd() {
        let cwd = std::env::current_dir().unwrap();
        let temp_dir = unique_temp_dir("opencode_config_dir_cwd");
        fs::write(temp_dir.join("instructions.md"), "cwd-content").unwrap();
        std::env::set_current_dir(&temp_dir).unwrap();

        let result = Config::substitute_variables("{file:instructions.md}", None);
        assert_eq!(result, "cwd-content");

        std::env::set_current_dir(cwd).unwrap();
        let _ = fs::remove_dir_all(temp_dir);
    }

    #[test]
    fn test_config_path_uses_json_by_default_in_custom_config_dir() {
        let temp_dir = unique_temp_dir("opencode_config_path_default_json");
        let old_dir = std::env::var("OPENCODE_CONFIG_DIR").ok();
        set_env("OPENCODE_CONFIG_DIR", &temp_dir);

        let path = Config::config_path();
        assert_eq!(path, temp_dir.join("config.json"));

        if let Some(dir) = old_dir {
            set_env("OPENCODE_CONFIG_DIR", dir);
        } else {
            remove_env("OPENCODE_CONFIG_DIR");
        }
        let _ = fs::remove_dir_all(temp_dir);
    }

    #[test]
    fn test_config_path_prefers_existing_jsonc_then_toml() {
        let temp_dir = unique_temp_dir("opencode_config_path_existing");
        let old_dir = std::env::var("OPENCODE_CONFIG_DIR").ok();
        set_env("OPENCODE_CONFIG_DIR", &temp_dir);

        fs::write(temp_dir.join("config.toml"), "model = \"x\"").unwrap();
        assert_eq!(Config::config_path(), temp_dir.join("config.toml"));

        fs::write(temp_dir.join("config.jsonc"), "{}").unwrap();
        assert_eq!(Config::config_path(), temp_dir.join("config.jsonc"));

        fs::write(temp_dir.join("config.json"), "{}").unwrap();
        assert_eq!(Config::config_path(), temp_dir.join("config.json"));

        if let Some(dir) = old_dir {
            set_env("OPENCODE_CONFIG_DIR", dir);
        } else {
            remove_env("OPENCODE_CONFIG_DIR");
        }
        let _ = fs::remove_dir_all(temp_dir);
    }

    #[test]
    fn test_scroll_acceleration_from_f32() {
        let config: ScrollAccelerationConfig = 1.5f32.into();
        assert!(config.enabled);
        assert_eq!(config.speed, Some(1.5));
    }

    #[test]
    fn test_scroll_acceleration_deserialize_legacy() {
        let config: ScrollAccelerationConfig = serde_json::from_str("2.5").unwrap();
        assert!(config.enabled);
        assert_eq!(config.speed, Some(2.5));
    }

    #[test]
    fn test_scroll_acceleration_deserialize_new_format() {
        let config: ScrollAccelerationConfig = serde_json::from_str(r#"{"enabled":true,"speed":3.0}"#).unwrap();
        assert!(config.enabled);
        assert_eq!(config.speed, Some(3.0));
    }

    #[test]
    fn test_scroll_acceleration_deserialize_minimal() {
        let config: ScrollAccelerationConfig = serde_json::from_str(r#"{"enabled":false}"#).unwrap();
        assert!(!config.enabled);
        assert_eq!(config.speed, None);
    }

    #[test]
    fn test_scroll_acceleration_default() {
        let config = ScrollAccelerationConfig::default();
        assert!(config.enabled);
        assert_eq!(config.speed, None);
    }

    #[test]
    fn test_agent_map_deserialize_old_format() {
        let agent_map: AgentMapConfig = serde_json::from_str(
            r#"{
                "plan": { "model": "openai/gpt-4o" },
                "build": { "temperature": 0.8 },
                "specialist": { "top_p": 0.7 }
            }"#,
        )
        .unwrap();

        assert!(agent_map.get_agent("plan").is_some());
        assert!(agent_map.get_agent("build").is_some());
        assert!(agent_map.get_agent("specialist").is_some());
        assert_eq!(agent_map.agents.len(), 3);
        assert!(agent_map.default_agent.is_none());
    }

    #[test]
    fn test_agent_map_deserialize_new_format() {
        let agent_map: AgentMapConfig = serde_json::from_str(
            r#"{
                "agents": {
                    "arbitrary": { "model": "anthropic/claude-3-7-sonnet" },
                    "deep_research": { "steps": 10 }
                },
                "default_agent": "arbitrary"
            }"#,
        )
        .unwrap();

        assert!(agent_map.get_agent("arbitrary").is_some());
        assert!(agent_map.get_agent("deep_research").is_some());
        assert!(agent_map.get_default_agent().is_some());
    }

    #[test]
    fn test_agent_map_get_agent_arbitrary_names() {
        let agent_map = AgentMapConfig {
            agents: HashMap::from([
                (
                    "my-custom-agent".to_string(),
                    AgentConfig {
                        model: Some("openai/gpt-4.1".to_string()),
                        ..Default::default()
                    },
                ),
                (
                    "agent_123".to_string(),
                    AgentConfig {
                        temperature: Some(0.5),
                        ..Default::default()
                    },
                ),
            ]),
            default_agent: Some("agent_123".to_string()),
        };

        assert!(agent_map.get_agent("my-custom-agent").is_some());
        assert!(agent_map.get_agent("agent_123").is_some());
        assert!(agent_map.get_agent("missing").is_none());
        assert_eq!(
            agent_map
                .get_default_agent()
                .and_then(|a| a.temperature),
            Some(0.5)
        );
    }

    #[test]
    fn test_agent_map_default_agent_validation_nonexistent_key() {
        let config = Config {
            agent: Some(AgentMapConfig {
                agents: HashMap::from([(
                    "plan".to_string(),
                    AgentConfig {
                        model: Some("openai/gpt-4o".to_string()),
                        ..Default::default()
                    },
                )]),
                default_agent: Some("does_not_exist".to_string()),
            }),
            ..Default::default()
        };

        let errors = config.validate();
        assert!(errors.iter().any(|e| {
            e.field == "agent.default_agent" && e.message.contains("does_not_exist")
        }));
    }

    #[test]
    fn test_agent_map_empty_agents_map() {
        let agent_map: AgentMapConfig = serde_json::from_str(r#"{"agents":{}}"#).unwrap();
        assert!(agent_map.agents.is_empty());
        assert!(agent_map.get_agent("anything").is_none());
        assert!(agent_map.get_default_agent().is_none());
    }

    #[test]
    fn test_agent_map_serialization_round_trip_new_format() {
        let original = AgentMapConfig {
            agents: HashMap::from([
                (
                    "plan".to_string(),
                    AgentConfig {
                        model: Some("openai/gpt-4.1".to_string()),
                        ..Default::default()
                    },
                ),
                (
                    "research".to_string(),
                    AgentConfig {
                        top_p: Some(0.9),
                        ..Default::default()
                    },
                ),
            ]),
            default_agent: Some("research".to_string()),
        };

        let serialized = serde_json::to_value(&original).unwrap();
        assert!(serialized.get("agents").is_some());
        assert!(serialized.get("default_agent").is_some());
        assert!(serialized.get("plan").is_none());

        let round_trip: AgentMapConfig = serde_json::from_value(serialized).unwrap();
        assert!(round_trip.get_agent("plan").is_some());
        assert!(round_trip.get_agent("research").is_some());
        assert_eq!(round_trip.default_agent.as_deref(), Some("research"));
    }

    #[test]
    fn test_load_tui_config_path_uses_env_when_set() {
        let old = std::env::var("OPENCODE_TUI_CONFIG").ok();
        set_env("OPENCODE_TUI_CONFIG", "/tmp/custom-tui.json");

        let path = Config::load_tui_config_path();
        assert_eq!(path, Some(PathBuf::from("/tmp/custom-tui.json")));

        if let Some(prev) = old {
            set_env("OPENCODE_TUI_CONFIG", prev);
        } else {
            remove_env("OPENCODE_TUI_CONFIG");
        }
    }

    #[test]
    fn test_load_tui_config_path_expands_tilde() {
        let old = std::env::var("OPENCODE_TUI_CONFIG").ok();
        let home = std::env::var("HOME").unwrap_or_else(|_| "/tmp".to_string());
        set_env("OPENCODE_TUI_CONFIG", "~/.config/opencode/test-tui.json");

        let path = Config::load_tui_config_path();
        assert_eq!(
            path,
            Some(PathBuf::from(home).join(".config/opencode/test-tui.json"))
        );

        if let Some(prev) = old {
            set_env("OPENCODE_TUI_CONFIG", prev);
        } else {
            remove_env("OPENCODE_TUI_CONFIG");
        }
    }

    #[test]
    fn test_load_tui_config_path_fallback_when_env_unset() {
        let old = std::env::var("OPENCODE_TUI_CONFIG").ok();
        remove_env("OPENCODE_TUI_CONFIG");

        let path = Config::load_tui_config_path();
        let expected = dirs::home_dir().map(|h| h.join(".config/opencode/tui.json"));
        assert_eq!(path, expected);

        if let Some(prev) = old {
            set_env("OPENCODE_TUI_CONFIG", prev);
        }
    }

    #[test]
    fn test_load_tui_config_missing_file_returns_default() {
        let old = std::env::var("OPENCODE_TUI_CONFIG").ok();
        let temp_dir = unique_temp_dir("opencode_tui_missing");
        set_env("OPENCODE_TUI_CONFIG", temp_dir.join("does-not-exist.json"));

        let tui = Config::load_tui_config().unwrap();
        assert!(tui.scroll_speed.is_none());
        assert!(tui.scroll_acceleration.is_none());
        assert!(tui.diff_style.is_none());
        assert!(tui.theme.is_none());
        assert!(tui.keybinds.is_none());

        if let Some(prev) = old {
            set_env("OPENCODE_TUI_CONFIG", prev);
        } else {
            remove_env("OPENCODE_TUI_CONFIG");
        }
        let _ = fs::remove_dir_all(temp_dir);
    }

    #[test]
    fn test_deprecation_warning_detection_fields_present() {
        let runtime_json = serde_json::json!({
            "model": "openai/gpt-4.1",
            "theme": { "name": "legacy-theme" },
            "keybinds": { "commands": "ctrl+k" }
        });

        let fields = Config::validate_runtime_no_tui_fields(&runtime_json);
        assert!(fields.contains(&"theme".to_string()));
        assert!(fields.contains(&"keybinds".to_string()));
    }

    #[test]
    fn test_migration_prefers_tui_values_over_legacy_main_fields() {
        #[allow(deprecated)]
        let runtime = Config {
            theme: Some(ThemeConfig {
                name: Some("legacy".to_string()),
                ..Default::default()
            }),
            keybinds: Some(KeybindConfig {
                commands: Some("ctrl+l".to_string()),
                ..Default::default()
            }),
            tui: Some(TuiConfig {
                theme: Some(ThemeConfig {
                    name: Some("new".to_string()),
                    ..Default::default()
                }),
                keybinds: Some(KeybindConfig {
                    commands: Some("ctrl+n".to_string()),
                    ..Default::default()
                }),
                ..Default::default()
            }),
            ..Default::default()
        };

        let mut migrated = runtime.tui.clone().unwrap_or_default();
        #[allow(deprecated)]
        {
            if migrated.theme.is_none() {
                migrated.theme = runtime.theme.clone();
            }
            if migrated.keybinds.is_none() {
                migrated.keybinds = runtime.keybinds.clone();
            }
        }

        assert_eq!(migrated.theme.and_then(|t| t.name), Some("new".to_string()));
        assert_eq!(
            migrated.keybinds.and_then(|k| k.commands),
            Some("ctrl+n".to_string())
        );
    }

    #[test]
    fn test_tui_runtime_field_separation_validation() {
        let tui_json = serde_json::json!({
            "scroll_speed": 3,
            "model": "openai/gpt-4.1"
        });
        let runtime_fields = Config::validate_tui_config_no_runtime_fields(&tui_json);
        assert!(runtime_fields.contains(&"model".to_string()));

        let runtime_json = serde_json::json!({
            "model": "openai/gpt-4.1",
            "tui": { "scroll_speed": 3 }
        });
        let tui_fields = Config::validate_runtime_no_tui_fields(&runtime_json);
        assert!(tui_fields.contains(&"tui".to_string()));
    }

    #[test]
    fn test_theme_config_resolve_path_relative_and_missing() {
        let config_dir = unique_temp_dir("opencode_theme_path_relative");
        let theme_file = config_dir.join("themes/custom.json");
        fs::create_dir_all(theme_file.parent().unwrap()).unwrap();
        fs::write(&theme_file, "{}").unwrap();

        let config = ThemeConfig {
            name: None,
            path: Some(PathBuf::from("themes/custom.json")),
            scan_dirs: None,
        };

        let resolved = config.resolve_path(Some(&config_dir));
        assert_eq!(resolved, Some(theme_file.clone()));

        let missing = ThemeConfig {
            name: None,
            path: Some(PathBuf::from("themes/does-not-exist.json")),
            scan_dirs: None,
        };
        assert!(missing.resolve_path(Some(&config_dir)).is_none());

        let _ = fs::remove_dir_all(config_dir);
    }

    #[test]
    fn test_theme_config_resolve_path_tilde_expansion() {
        let temp_home = unique_temp_dir("opencode_theme_home");
        let old_home = std::env::var("HOME").ok();
        set_env("HOME", &temp_home);

        let theme_file = temp_home.join(".config/opencode/themes/home-theme.json");
        fs::create_dir_all(theme_file.parent().unwrap()).unwrap();
        fs::write(&theme_file, "{}").unwrap();

        let config = ThemeConfig {
            name: None,
            path: Some(PathBuf::from("~/.config/opencode/themes/home-theme.json")),
            scan_dirs: None,
        };
        assert_eq!(config.resolve_path(None), Some(theme_file));

        if let Some(home) = old_home {
            set_env("HOME", home);
        } else {
            remove_env("HOME");
        }
        let _ = fs::remove_dir_all(temp_home);
    }

    #[test]
    fn test_load_with_hierarchy_emits_deprecation_warnings_for_theme_and_keybinds() {
        let temp_dir = unique_temp_dir("opencode_deprecation_warnings");
        let old_config_dir = std::env::var("OPENCODE_CONFIG_DIR").ok();

        let config_json = serde_json::json!({
            "model": "openai/gpt-4.1",
            "theme": { "name": "legacy-theme" },
            "keybinds": { "commands": "ctrl+k" }
        });
        fs::write(
            temp_dir.join("config.json"),
            serde_json::to_string_pretty(&config_json).unwrap(),
        )
        .unwrap();

        set_env("OPENCODE_CONFIG_DIR", &temp_dir);

        let sink = Arc::new(Mutex::new(Vec::<String>::new()));
        let subscriber = tracing_subscriber::registry().with(WarnCaptureLayer { sink: sink.clone() });

        tracing::subscriber::with_default(subscriber, || {
            let runtime = tokio::runtime::Runtime::new().unwrap();
            let _ = runtime.block_on(Config::load_multi()).unwrap();
        });

        let logs = sink.lock().unwrap().clone();
        assert!(logs.iter().any(|msg| msg.contains("'theme' in main config is deprecated")));
        assert!(logs.iter().any(|msg| msg.contains("'keybinds' in main config is deprecated")));

        if let Some(prev) = old_config_dir {
            set_env("OPENCODE_CONFIG_DIR", prev);
        } else {
            remove_env("OPENCODE_CONFIG_DIR");
        }
        let _ = fs::remove_dir_all(temp_dir);
    }

    #[test]
    fn test_keybind_merge_detects_conflicts_for_default_and_custom_actions() {
        let defaults = KeybindConfig {
            commands: Some("ctrl+k".to_string()),
            timeline: Some("ctrl+t".to_string()),
            ..Default::default()
        };
        let custom = KeybindConfig {
            settings: Some("ctrl+k".to_string()),
            custom: Some(std::collections::HashMap::from([(
                "my_action".to_string(),
                "ctrl+k".to_string(),
            )])),
            ..Default::default()
        };

        let (_merged, conflicts) = custom.merge_with_defaults(&defaults);

        assert!(
            conflicts
                .iter()
                .any(|c| c.contains("ctrl+k used by both 'commands' and 'settings'"))
        );
        assert!(
            conflicts
                .iter()
                .any(|c| c.contains("ctrl+k used by both 'commands' and 'custom 'my_action''"))
        );
    }
}
