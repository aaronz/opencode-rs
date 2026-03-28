use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;

/// Main configuration structure matching the TypeScript Config.Info schema
#[derive(Debug, Clone, Deserialize, Serialize, Default)]
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
    #[serde(skip_serializing_if = "Option::is_none")]
    pub keybinds: Option<KeybindConfig>,

    /// Theme configuration
    #[serde(skip_serializing_if = "Option::is_none")]
    pub theme: Option<ThemeConfig>,

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
#[derive(Debug, Clone, Deserialize, Serialize)]
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

/// Agent map configuration with predefined agents
#[derive(Debug, Clone, Deserialize, Serialize, Default)]
pub struct AgentMapConfig {
    /// Primary agents
    #[serde(skip_serializing_if = "Option::is_none")]
    pub plan: Option<AgentConfig>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub build: Option<AgentConfig>,

    /// Subagents
    #[serde(skip_serializing_if = "Option::is_none")]
    pub general: Option<AgentConfig>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub explore: Option<AgentConfig>,

    /// Specialized agents
    #[serde(skip_serializing_if = "Option::is_none")]
    pub title: Option<AgentConfig>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub summary: Option<AgentConfig>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub compaction: Option<AgentConfig>,

    /// Additional custom agents
    #[serde(flatten, skip_serializing_if = "Option::is_none")]
    pub custom: Option<HashMap<String, AgentConfig>>,
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

    /// Additional options
    #[serde(flatten, skip_serializing_if = "Option::is_none")]
    pub extra: Option<HashMap<String, serde_json::Value>>,
}

/// Timeout configuration - can be a number or false to disable
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(untagged)]
pub enum TimeoutConfig {
    Milliseconds(u64),
    Disabled(bool),
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

#[derive(Debug, Clone, Deserialize, Serialize, Default)]
pub struct ThemeConfig {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub path: Option<std::path::PathBuf>,
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
            toml::from_str(&content).map_err(|e| crate::OpenCodeError::Config(e.to_string()))?
        };

        config.apply_env_overrides();
        Ok(config)
    }

    pub fn config_path() -> PathBuf {
        directories::ProjectDirs::from("com", "opencode", "rs")
            .map(|dirs| dirs.config_dir().join("config.toml"))
            .unwrap_or_else(|| PathBuf::from("~/.config/opencode-rs/config.toml"))
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
}

#[cfg(test)]
mod tests {
    use super::*;

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
}
