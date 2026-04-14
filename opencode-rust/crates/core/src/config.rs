pub use opencode_config::{
    is_jsonc_extension, load_opencode_directory, parse_jsonc, AcpConfig, AgentConfig,
    AgentMapConfig, AgentsMdConfig, AutoUpdate, CliOverrideConfig, CommandConfig, CompactionConfig,
    Config, ConfigError, DesktopConfig, DiffStyle, DirectoryScanner, EnterpriseConfig,
    ExperimentalConfig, FormatterConfig, FormatterEntry, JsoncError, KeybindConfig, LegacyProvider,
    LogLevel, LspConfig, LspEntry, McpConfig, McpLocalConfig, McpOAuthConfig, McpOAuthUnion,
    McpRemoteConfig, ModelConfig, OpencodeDirectoryScan, PermissionAction, PermissionConfig,
    PermissionRule, ProviderConfig, ProviderOptions, ScrollAccelerationConfig, ServerConfig,
    ShareMode, SkillsConfig, ThemeConfig, TimeoutConfig, ToolInfo, TuiConfig, TuiPluginConfig,
    ValidationError, ValidationResult, ValidationSeverity, VariantConfig, WatcherConfig,
};

pub use opencode_config::Config as ConfigTrait;

pub type OpenCodeConfigError = opencode_config::ConfigError;

impl From<opencode_config::ConfigError> for crate::OpenCodeError {
    fn from(err: opencode_config::ConfigError) -> Self {
        crate::OpenCodeError::Config(err.to_string())
    }
}
