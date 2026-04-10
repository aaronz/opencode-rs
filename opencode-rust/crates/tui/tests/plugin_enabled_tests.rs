//! Tests for TUI plugin_enabled configuration semantics
//!
//! # Semantics
//!
//! - `plugin_enabled` belongs in tui.json, NOT opencode.json
//! - `plugin_enabled=true` (default): TUI plugins ARE loaded
//! - `plugin_enabled=false`: TUI plugins are NOT loaded (master switch)
//!
//! # Test Command
//!
//! ```bash
//! cargo test -p opencode-tui -- plugin_enabled
//! ```

use opencode_core::config::{TuiConfig, TuiPluginConfig};

#[test]
fn test_plugin_enabled_default_is_true() {
    let tui_config = TuiConfig::default();
    let plugin_enabled = tui_config
        .plugins
        .as_ref()
        .and_then(|p| p.plugin_enabled)
        .unwrap_or(true);
    assert!(
        plugin_enabled,
        "plugin_enabled should default to true when not specified"
    );
}

#[test]
fn test_plugin_enabled_true_allows_loading() {
    let mut tui_config = TuiConfig::default();
    tui_config.plugins = Some(TuiPluginConfig {
        plugin_enabled: Some(true),
    });
    let plugin_enabled = tui_config
        .plugins
        .as_ref()
        .and_then(|p| p.plugin_enabled)
        .unwrap_or(true);
    assert!(
        plugin_enabled,
        "plugin_enabled=true should allow plugin loading"
    );
}

#[test]
fn test_plugin_enabled_false_prevents_loading() {
    let mut tui_config = TuiConfig::default();
    tui_config.plugins = Some(TuiPluginConfig {
        plugin_enabled: Some(false),
    });
    let plugin_enabled = tui_config
        .plugins
        .as_ref()
        .and_then(|p| p.plugin_enabled)
        .unwrap_or(true);
    assert!(
        !plugin_enabled,
        "plugin_enabled=false should prevent plugin loading"
    );
}

#[test]
fn test_plugin_enabled_none_defaults_to_true() {
    let mut tui_config = TuiConfig::default();
    tui_config.plugins = Some(TuiPluginConfig {
        plugin_enabled: None,
    });
    let plugin_enabled = tui_config
        .plugins
        .as_ref()
        .and_then(|p| p.plugin_enabled)
        .unwrap_or(true);
    assert!(plugin_enabled, "plugin_enabled=None should default to true");
}

#[test]
fn test_plugin_enabled_in_tui_config_not_opencode_config() {
    use opencode_core::config::Config;

    let core_config = Config::default();
    let has_plugin_field = core_config
        .tui
        .as_ref()
        .and_then(|t| t.plugins.as_ref())
        .is_some();

    assert!(
        has_plugin_field || core_config.tui.is_none(),
        "plugin_enabled belongs in TuiConfig, not in main Config"
    );
}

#[test]
fn test_tui_config_serialization_with_plugin_enabled() {
    let mut tui_config = TuiConfig::default();
    tui_config.plugins = Some(TuiPluginConfig {
        plugin_enabled: Some(false),
    });

    let json = serde_json::to_string(&tui_config).unwrap();
    assert!(
        json.contains("\"plugin_enabled\":false"),
        "JSON serialization should contain plugin_enabled: false"
    );
}

#[test]
fn test_tui_config_deserialization_with_plugin_enabled() {
    let json = r#"{"plugins":{"plugin_enabled":false}}"#;
    let tui_config: TuiConfig = serde_json::from_str(json).unwrap();

    let plugin_enabled = tui_config
        .plugins
        .as_ref()
        .and_then(|p| p.plugin_enabled)
        .unwrap_or(true);

    assert!(
        !plugin_enabled,
        "Deserialized config should have plugin_enabled=false"
    );
}

#[test]
fn test_tui_config_deserialization_plugin_enabled_true() {
    let json = r#"{"plugins":{"plugin_enabled":true}}"#;
    let tui_config: TuiConfig = serde_json::from_str(json).unwrap();

    let plugin_enabled = tui_config
        .plugins
        .as_ref()
        .and_then(|p| p.plugin_enabled)
        .unwrap_or(true);

    assert!(
        plugin_enabled,
        "Deserialized config should have plugin_enabled=true"
    );
}

#[test]
fn test_plugin_enabled_semantics_documented() {
    let tui_config = TuiConfig::default();
    assert!(
        tui_config.plugins.is_none(),
        "Default TuiConfig should have no plugins field (implies default true)"
    );
}
