//! Plugin configuration separation and validation.
//!
//! This module ensures that plugin configs are properly isolated from:
//! - Core config values (opencode.json)
//! - Other plugin configs
//!
//! # Config Ownership
//!
//! Plugins store their configuration in the `options` field of `PluginConfig`.
//! These options are isolated per-plugin and cannot:
//! - Override core config keys (like `model`, `server`, `permission`, etc.)
//! - Leak into other plugins' configurations
//!
//! # Reserved Keys
//!
//! The following top-level keys are reserved and cannot be used in plugin options:
//! - Core runtime keys: `log_level`, `server`, `command`, `skills`, `watcher`, `plugin`,
//!   `snapshot`, `share`, `autoshare`, `autoupdate`, `disabled_providers`, `enabled_providers`
//! - Model/agent keys: `model`, `small_model`, `default_agent`, `username`, `mode`, `agent`
//! - Provider/integration keys: `provider`, `mcp`, `formatter`, `lsp`, `instructions`, `agents_md`
//! - Permission/security keys: `permission`, `tools`, `enterprise`
//! - Feature keys: `compaction`, `experimental`
//! - Legacy/theme keys: `theme`, `tui`
//! - Auth/connection keys: `api_key`, `temperature`, `max_tokens`

use indexmap::IndexMap;
use serde_json::Value;
use thiserror::Error;

/// Errors that can occur during plugin config validation.
#[derive(Debug, Error)]
pub enum ConfigValidationError {
    #[error("plugin '{plugin}' has reserved option key '{key}' - cannot override core config")]
    ReservedKey { plugin: String, key: String },

    #[error("plugin '{plugin}' has nested reserved key '{key}' - cannot override core config")]
    NestedReservedKey { plugin: String, key: String },

    #[error(
        "plugin config isolation violation: '{from_plugin}' options leak into plugin '{to_plugin}'"
    )]
    ConfigLeak {
        from_plugin: String,
        to_plugin: String,
    },
}

/// Result of plugin config validation.
#[derive(Debug, Clone, Default)]
pub struct ConfigValidationResult {
    /// Whether the validation passed.
    pub valid: bool,
    /// List of validation errors if any.
    pub errors: Vec<String>,
}

impl ConfigValidationResult {
    /// Create a new valid result.
    pub fn valid() -> Self {
        Self {
            valid: true,
            errors: Vec::new(),
        }
    }

    /// Create a new invalid result with the given errors.
    pub fn invalid(errors: Vec<String>) -> Self {
        Self {
            valid: false,
            errors,
        }
    }

    /// Add an error to the result.
    pub fn add_error(&mut self, error: String) {
        self.valid = false;
        self.errors.push(error);
    }

    /// Merge another result into this one.
    pub fn merge(&mut self, other: ConfigValidationResult) {
        if !other.valid {
            self.valid = false;
            self.errors.extend(other.errors);
        }
    }
}

/// Top-level reserved config keys that plugins cannot override.
/// These correspond to the core Config struct fields.
pub const RESERVED_CONFIG_KEYS: &[&str] = &[
    // Schema and meta
    "$schema",
    // Core runtime
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
    // Model/agent
    "model",
    "small_model",
    "default_agent",
    "username",
    "mode",
    "agent",
    // Provider/integration
    "provider",
    "mcp",
    "formatter",
    "lsp",
    "instructions",
    "agents_md",
    // Permission/security
    "permission",
    "tools",
    "enterprise",
    // Feature
    "compaction",
    "experimental",
    // Legacy/theme
    "theme",
    "tui",
    // Auth/connection
    "api_key",
    "temperature",
    "max_tokens",
];

/// Reserved key prefixes that indicate nested core config.
pub const RESERVED_KEY_PREFIXES: &[&str] = &[
    "server.",
    "mcp.",
    "provider.",
    "agent.",
    "permission.",
    "formatter.",
    "lsp.",
    "command.",
    "skills.",
    "watcher.",
    "enterprise.",
    "compaction.",
    "experimental.",
];

/// Validates that a plugin's options don't use reserved config keys.
///
/// # Arguments
/// * `plugin_name` - The name of the plugin (for error reporting)
/// * `options` - The plugin's options to validate
///
/// # Returns
/// A `ConfigValidationResult` indicating whether validation passed.
pub fn validate_plugin_options(
    plugin_name: &str,
    options: &IndexMap<String, Value>,
) -> ConfigValidationResult {
    let mut result = ConfigValidationResult::valid();

    // Check for reserved top-level keys
    for key in options.keys() {
        if RESERVED_CONFIG_KEYS.contains(&key.as_str()) {
            result.add_error(format!(
                "plugin '{}' has reserved option key '{}' - cannot override core config",
                plugin_name, key
            ));
        }
    }

    // Check for nested reserved keys (e.g., "server.port", "mcp.foo")
    for (key, value) in options {
        // Check key prefix
        for prefix in RESERVED_KEY_PREFIXES {
            if key.starts_with(prefix) {
                result.add_error(format!(
                    "plugin '{}' has nested reserved key '{}' - cannot override core config",
                    plugin_name, key
                ));
                break;
            }
        }

        // Recursively check nested objects
        if let Value::Object(nested) = value {
            validate_nested_reserved_keys(plugin_name, nested, &mut result);
        }
    }

    result
}

/// Recursively validates nested objects for reserved keys.
fn validate_nested_reserved_keys(
    plugin_name: &str,
    obj: &serde_json::Map<String, Value>,
    result: &mut ConfigValidationResult,
) {
    for (key, value) in obj {
        if RESERVED_CONFIG_KEYS.contains(&key.as_str()) {
            result.add_error(format!(
                "plugin '{}' has nested reserved key '{}' - cannot override core config",
                plugin_name, key
            ));
        }

        // Check if this key starts with a reserved prefix
        for prefix in RESERVED_KEY_PREFIXES {
            if key.starts_with(prefix) {
                result.add_error(format!(
                    "plugin '{}' has nested reserved key '{}' - cannot override core config",
                    plugin_name, key
                ));
            }
        }

        // Recurse into nested objects
        if let Value::Object(nested) = value {
            validate_nested_reserved_keys(plugin_name, nested, result);
        }
    }
}

/// Validates that a plugin config doesn't have options that conflict with core config.
///
/// This is called during plugin discovery/loading to ensure config boundaries are respected.
pub fn validate_plugin_config(
    plugin_name: &str,
    options: &IndexMap<String, Value>,
) -> ConfigValidationResult {
    validate_plugin_options(plugin_name, options)
}

/// Validates config isolation between multiple plugins.
///
/// Ensures that no plugin's options leak into or conflict with other plugins.
pub fn validate_plugin_isolation(
    plugins: &[(String, IndexMap<String, Value>)],
) -> ConfigValidationResult {
    let mut result = ConfigValidationResult::valid();

    // Each plugin's options should be independent - no cross-contamination
    // Since options are stored per-plugin in PluginConfig.options, isolation is
    // naturally enforced at the data structure level. We verify that no plugin
    // has options that reference another plugin's name as a config key.

    for (plugin_name, options) in plugins {
        for key in options.keys() {
            // Check if any plugin option key matches another plugin's name
            for (other_name, _) in plugins {
                if key == other_name && plugin_name != other_name {
                    result.add_error(format!(
                        "plugin config isolation violation: '{}' options contain key '{}' which matches plugin '{}'",
                        plugin_name, key, other_name
                    ));
                }
            }
        }
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;
    use indexmap::IndexMap;

    #[test]
    fn test_valid_plugin_options() {
        let mut options = IndexMap::new();
        options.insert("custom_setting".to_string(), serde_json::json!("value"));
        options.insert("my_option".to_string(), serde_json::json!({"nested": true}));

        let result = validate_plugin_options("test-plugin", &options);
        assert!(result.valid);
        assert!(result.errors.is_empty());
    }

    #[test]
    fn test_reserved_top_level_key() {
        let mut options = IndexMap::new();
        options.insert("model".to_string(), serde_json::json!("gpt-5"));
        options.insert("custom_setting".to_string(), serde_json::json!("value"));

        let result = validate_plugin_options("test-plugin", &options);
        assert!(!result.valid);
        assert!(result
            .errors
            .iter()
            .any(|e| e.contains("reserved option key 'model'")));
    }

    #[test]
    fn test_reserved_nested_key() {
        let mut options = IndexMap::new();
        options.insert("server".to_string(), serde_json::json!({"port": 8080}));

        let result = validate_plugin_options("test-plugin", &options);
        assert!(!result.valid);
        // server is a reserved top-level key so it should be caught there
        assert!(result
            .errors
            .iter()
            .any(|e| e.contains("reserved option key 'server'")));
    }

    #[test]
    fn test_reserved_key_prefix() {
        let mut options = IndexMap::new();
        options.insert("server.port".to_string(), serde_json::json!(9090));

        let result = validate_plugin_options("test-plugin", &options);
        assert!(!result.valid);
        assert!(result
            .errors
            .iter()
            .any(|e| e.contains("nested reserved key 'server.port'")));
    }

    #[test]
    fn test_multiple_reserved_keys() {
        let mut options = IndexMap::new();
        options.insert("model".to_string(), serde_json::json!("gpt-5"));
        options.insert("provider".to_string(), serde_json::json!({"openai": true}));
        options.insert(
            "permission".to_string(),
            serde_json::json!({"read": "allow"}),
        );

        let result = validate_plugin_options("test-plugin", &options);
        assert!(!result.valid);
        assert_eq!(result.errors.len(), 3);
    }

    #[test]
    fn test_deeply_nested_reserved_key() {
        let mut options = IndexMap::new();
        options.insert(
            "custom".to_string(),
            serde_json::json!({
                "server": {
                    "port": 8080
                }
            }),
        );

        let result = validate_plugin_options("test-plugin", &options);
        assert!(!result.valid);
        assert!(result
            .errors
            .iter()
            .any(|e| e.contains("nested reserved key")));
    }

    #[test]
    fn test_plugin_isolation_no_conflicts() {
        let plugins = vec![
            ("plugin-a".to_string(), {
                let mut m = IndexMap::new();
                m.insert("option_a".to_string(), serde_json::json!("value"));
                m
            }),
            ("plugin-b".to_string(), {
                let mut m = IndexMap::new();
                m.insert("option_b".to_string(), serde_json::json!("value"));
                m
            }),
        ];

        let result = validate_plugin_isolation(&plugins);
        assert!(result.valid);
    }

    #[test]
    fn test_plugin_isolation_with_name_conflict() {
        let plugins = vec![
            ("plugin-a".to_string(), {
                let mut m = IndexMap::new();
                m.insert("plugin-b".to_string(), serde_json::json!("value"));
                m
            }),
            ("plugin-b".to_string(), {
                let mut m = IndexMap::new();
                m.insert("option_b".to_string(), serde_json::json!("value"));
                m
            }),
        ];

        let result = validate_plugin_isolation(&plugins);
        assert!(!result.valid);
        assert!(result
            .errors
            .iter()
            .any(|e| e.contains("config isolation violation")));
    }

    #[test]
    fn test_all_reserved_keys_defined() {
        // Ensure our reserved keys list is comprehensive
        // These should match the core Config struct fields
        let expected_reserved = vec![
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

        for key in expected_reserved {
            assert!(
                RESERVED_CONFIG_KEYS.contains(&key),
                "reserved key '{}' should be in RESERVED_CONFIG_KEYS",
                key
            );
        }
    }
}
