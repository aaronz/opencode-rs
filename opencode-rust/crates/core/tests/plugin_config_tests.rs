use opencode_core::config::TuiConfig;

#[cfg(test)]
mod plugin_config_tests {
    use super::*;

    #[test]
    fn plugin_config_tui_plugin_config_deserialization() {
        let json = r#"{
            "plugins": {
                "plugin_enabled": true,
                "plugins": {
                    "my-plugin": true,
                    "other-plugin": false
                }
            }
        }"#;

        let config: TuiConfig = serde_json::from_str(json).unwrap();
        let plugins = config.plugins.unwrap();
        assert!(plugins.plugin_enabled.unwrap_or(false));
        assert_eq!(plugins.plugins.as_ref().unwrap().len(), 2);
    }

    #[test]
    fn plugin_config_tui_plugin_enabled_defaults_to_true() {
        let json = r#"{
            "plugins": {
                "plugins": {
                    "my-plugin": true
                }
            }
        }"#;

        let config: TuiConfig = serde_json::from_str(json).unwrap();
        let plugins = config.plugins.unwrap();
        assert_eq!(
            plugins.plugin_enabled.unwrap_or(false),
            true,
            "plugin_enabled should default to true"
        );
    }

    #[test]
    fn plugin_config_empty_plugins_map_valid() {
        let json = r#"{
            "plugins": {
                "plugin_enabled": true,
                "plugins": {}
            }
        }"#;

        let config: TuiConfig = serde_json::from_str(json).unwrap();
        let plugins = config.plugins.unwrap();
        assert!(plugins.plugin_enabled.unwrap_or(false));
        assert!(plugins.plugins.as_ref().unwrap().is_empty());
    }

    #[test]
    fn plugin_config_plugin_enabled_false_disables_all() {
        let json = r#"{
            "plugins": {
                "plugin_enabled": false,
                "plugins": {
                    "my-plugin": true,
                    "other-plugin": true
                }
            }
        }"#;

        let config: TuiConfig = serde_json::from_str(json).unwrap();
        let plugins = config.plugins.unwrap();
        assert_eq!(
            plugins.plugin_enabled.unwrap_or(true),
            false,
            "plugin_enabled should be false"
        );
    }

    #[test]
    fn plugin_config_no_plugins_field_implies_default() {
        let json = r#"{
            "scroll_speed": 10
        }"#;

        let config: TuiConfig = serde_json::from_str(json).unwrap();
        assert!(config.plugins.is_none(), "No plugins field should be None");
    }
}
