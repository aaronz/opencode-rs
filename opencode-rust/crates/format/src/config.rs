use opencode_config::FormatterConfig;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FormatConfig {
    #[serde(default)]
    pub formatter: Option<FormatterConfig>,
}

impl FormatConfig {
    pub fn formatter_enabled(&self) -> bool {
        match &self.formatter {
            Some(FormatterConfig::Disabled(false)) => true,
            Some(FormatterConfig::Disabled(true)) => false,
            Some(FormatterConfig::Formatters(map)) => !map.is_empty(),
            None => false,
        }
    }

    pub fn is_disabled(&self) -> bool {
        match &self.formatter {
            Some(FormatterConfig::Disabled(true)) => true,
            Some(FormatterConfig::Disabled(false)) => false,
            Some(FormatterConfig::Formatters(_)) => false,
            None => true,
        }
    }

    pub fn get_formatter_config(&self) -> Option<&FormatterConfig> {
        self.formatter.as_ref()
    }
}

impl Default for FormatConfig {
    fn default() -> Self {
        Self {
            formatter: Some(FormatterConfig::Disabled(false)),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    #[test]
    fn test_formatter_config_parsing_true() {
        let json = serde_json::json!({"formatter": true});
        let config: FormatConfig = serde_json::from_value(json).unwrap();
        assert!(config.is_disabled());
        assert!(!config.formatter_enabled());
    }

    #[test]
    fn test_formatter_config_parsing_false() {
        let json = serde_json::json!({"formatter": false});
        let config: FormatConfig = serde_json::from_value(json).unwrap();
        assert!(!config.is_disabled());
        assert!(config.formatter_enabled());
    }

    #[test]
    fn test_disabled_formatters_list_parsing() {
        let json = serde_json::json!({
            "formatter": {
                "prettier": {"disabled": true},
                "rustfmt": {"disabled": true}
            }
        });
        let config: FormatConfig = serde_json::from_value(json).unwrap();
        match &config.formatter {
            Some(FormatterConfig::Formatters(map)) => {
                assert_eq!(map.len(), 2);
                assert_eq!(map.get("prettier").unwrap().disabled, Some(true));
                assert_eq!(map.get("rustfmt").unwrap().disabled, Some(true));
            }
            _ => panic!("Expected Formatters variant"),
        }
    }

    #[test]
    fn test_formatter_config_per_formatter_settings() {
        let json = serde_json::json!({
            "formatter": {
                "prettier": {
                    "command": ["prettier", "--write"],
                    "extensions": [".js", ".ts"]
                }
            }
        });
        let config: FormatConfig = serde_json::from_value(json).unwrap();
        match &config.formatter {
            Some(FormatterConfig::Formatters(map)) => {
                let entry = map.get("prettier").unwrap();
                assert_eq!(
                    entry.command,
                    Some(vec!["prettier".to_string(), "--write".to_string()])
                );
                assert_eq!(
                    entry.extensions,
                    Some(vec![".js".to_string(), ".ts".to_string()])
                );
            }
            _ => panic!("Expected Formatters variant"),
        }
    }

    #[test]
    fn test_formatter_config_empty_formatters() {
        let json = serde_json::json!({
            "formatter": {}
        });
        let config: FormatConfig = serde_json::from_value(json).unwrap();
        match &config.formatter {
            Some(FormatterConfig::Formatters(map)) => {
                assert!(map.is_empty());
            }
            _ => panic!("Expected Formatters variant"),
        }
    }

    #[test]
    fn test_formatter_config_null_formatter() {
        let json = serde_json::json!({"formatter": null});
        let config: FormatConfig = serde_json::from_value(json).unwrap();
        assert!(config.formatter.is_none());
        assert!(config.is_disabled());
        assert!(!config.formatter_enabled());
    }

    #[test]
    fn test_formatter_config_missing_formatter() {
        let json = serde_json::json!({});
        let config: FormatConfig = serde_json::from_value(json).unwrap();
        assert!(config.formatter.is_none());
    }

    #[test]
    fn test_formatter_config_environment_variables() {
        let json = serde_json::json!({
            "formatter": {
                "custom": {
                    "command": ["customFormatter"],
                    "environment": {"VAR1": "value1", "VAR2": "value2"}
                }
            }
        });
        let config: FormatConfig = serde_json::from_value(json).unwrap();
        match &config.formatter {
            Some(FormatterConfig::Formatters(map)) => {
                let entry = map.get("custom").unwrap();
                assert_eq!(
                    entry.environment,
                    Some(HashMap::from([
                        ("VAR1".to_string(), "value1".to_string()),
                        ("VAR2".to_string(), "value2".to_string())
                    ]))
                );
            }
            _ => panic!("Expected Formatters variant"),
        }
    }
}
