//! Logging configuration.

use std::collections::HashMap;
use std::path::PathBuf;

use serde::{Deserialize, Serialize};

use crate::event::LogLevel;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoggingConfig {
    #[serde(default = "default_level")]
    pub level: LogLevel,
    #[serde(default)]
    pub targets: HashMap<String, LogLevel>,
    #[serde(default)]
    pub file_path: Option<PathBuf>,
    #[serde(default = "default_max_file_size_mb")]
    pub max_file_size_mb: usize,
    #[serde(default = "default_max_rotated_files")]
    pub max_rotated_files: usize,
    #[serde(default)]
    pub show_in_tui: bool,
    #[serde(default)]
    pub tui_position: TuiLogPosition,
    #[serde(default = "default_memory_buffer_size")]
    pub memory_buffer_size: usize,
    #[serde(default = "default_retention_days")]
    pub retention_days: u32,
}

fn default_level() -> LogLevel {
    LogLevel::Info
}

fn default_max_file_size_mb() -> usize {
    50
}

fn default_max_rotated_files() -> usize {
    5
}

fn default_memory_buffer_size() -> usize {
    10000
}

fn default_retention_days() -> u32 {
    30
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum TuiLogPosition {
    #[default]
    Bottom,
    Right,
    Overlay,
}

impl Default for LoggingConfig {
    fn default() -> Self {
        Self {
            level: LogLevel::Info,
            targets: HashMap::new(),
            file_path: None,
            max_file_size_mb: 50,
            max_rotated_files: 5,
            show_in_tui: false,
            tui_position: TuiLogPosition::Bottom,
            memory_buffer_size: 10000,
            retention_days: 30,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn test_default_config_provides_sensible_values() {
        let config = LoggingConfig::default();

        assert_eq!(config.level, LogLevel::Info);
        assert_eq!(config.max_file_size_mb, 50);
        assert_eq!(config.max_rotated_files, 5);
        assert_eq!(config.memory_buffer_size, 10000);
        assert_eq!(config.retention_days, 30);
        assert!(!config.show_in_tui);
        assert_eq!(config.tui_position, TuiLogPosition::Bottom);
        assert!(config.targets.is_empty());
        assert!(config.file_path.is_none());
    }

    #[test]
    fn test_tui_position_enum_variants() {
        assert_ne!(TuiLogPosition::Bottom, TuiLogPosition::Right);
        assert_ne!(TuiLogPosition::Bottom, TuiLogPosition::Overlay);
        assert_ne!(TuiLogPosition::Right, TuiLogPosition::Overlay);
        assert_eq!(TuiLogPosition::default(), TuiLogPosition::Bottom);
    }

    #[test]
    fn test_targets_hashmap_allows_per_component_levels() {
        let mut config = LoggingConfig::default();

        config.targets.insert("agent".to_string(), LogLevel::Debug);
        config.targets.insert("llm.*".to_string(), LogLevel::Info);
        config.targets.insert("tool.read".to_string(), LogLevel::Trace);
        config.targets.insert("tool.*".to_string(), LogLevel::Debug);
        config.targets.insert("error".to_string(), LogLevel::Error);

        assert_eq!(config.targets.get("agent"), Some(&LogLevel::Debug));
        assert_eq!(config.targets.get("llm.*"), Some(&LogLevel::Info));
        assert_eq!(config.targets.get("tool.read"), Some(&LogLevel::Trace));
        assert_eq!(config.targets.get("tool.*"), Some(&LogLevel::Debug));
        assert_eq!(config.targets.get("error"), Some(&LogLevel::Error));
        assert_eq!(config.targets.get("nonexistent"), None);

        config.targets.insert("agent".to_string(), LogLevel::Warn);
        assert_eq!(config.targets.get("agent"), Some(&LogLevel::Warn));

        config.targets.remove("tool.read");
        assert_eq!(config.targets.get("tool.read"), None);
    }

    #[test]
    fn test_toml_deserialization_parses_all_fields() {
        let toml_content = r#"
            level = "debug"
            file_path = "/var/log/opencode.log"
            max_file_size_mb = 100
            max_rotated_files = 10
            show_in_tui = true
            tui_position = "right"
            memory_buffer_size = 50000
            retention_days = 60

            [targets]
            "agent" = "debug"
            "llm.*" = "info"
            "tool.*" = "trace"
        "#;

        let config: LoggingConfig = toml::from_str(toml_content).expect("Failed to parse TOML");

        assert_eq!(config.level, LogLevel::Debug);
        assert_eq!(config.file_path, Some(PathBuf::from("/var/log/opencode.log")));
        assert_eq!(config.max_file_size_mb, 100);
        assert_eq!(config.max_rotated_files, 10);
        assert!(config.show_in_tui);
        assert_eq!(config.tui_position, TuiLogPosition::Right);
        assert_eq!(config.memory_buffer_size, 50000);
        assert_eq!(config.retention_days, 60);
        assert_eq!(config.targets.len(), 3);
        assert_eq!(config.targets.get("agent"), Some(&LogLevel::Debug));
        assert_eq!(config.targets.get("llm.*"), Some(&LogLevel::Info));
        assert_eq!(config.targets.get("tool.*"), Some(&LogLevel::Trace));
    }

    #[test]
    fn test_toml_deserialization_with_minimal_config() {
        let toml_content = r#"
            level = "warn"
        "#;

        let config: LoggingConfig = toml::from_str(toml_content).expect("Failed to parse TOML");

        assert_eq!(config.level, LogLevel::Warn);
        assert_eq!(config.max_file_size_mb, 50);
        assert_eq!(config.max_rotated_files, 5);
        assert_eq!(config.memory_buffer_size, 10000);
        assert_eq!(config.retention_days, 30);
        assert!(!config.show_in_tui);
        assert_eq!(config.tui_position, TuiLogPosition::Bottom);
        assert!(config.targets.is_empty());
        assert!(config.file_path.is_none());
    }

    #[test]
    fn test_toml_deserialization_all_tui_positions() {
        let bottom_toml = r#"tui_position = "bottom""#;
        let config: LoggingConfig = toml::from_str(bottom_toml).expect("Failed to parse Bottom");
        assert_eq!(config.tui_position, TuiLogPosition::Bottom);

        let right_toml = r#"tui_position = "right""#;
        let config: LoggingConfig = toml::from_str(right_toml).expect("Failed to parse Right");
        assert_eq!(config.tui_position, TuiLogPosition::Right);

        let overlay_toml = r#"tui_position = "overlay""#;
        let config: LoggingConfig = toml::from_str(overlay_toml).expect("Failed to parse Overlay");
        assert_eq!(config.tui_position, TuiLogPosition::Overlay);
    }

    #[test]
    fn test_toml_deserialization_all_log_levels() {
        let trace_toml = r#"level = "trace""#;
        let config: LoggingConfig = toml::from_str(trace_toml).expect("Failed to parse trace");
        assert_eq!(config.level, LogLevel::Trace);

        let debug_toml = r#"level = "debug""#;
        let config: LoggingConfig = toml::from_str(debug_toml).expect("Failed to parse debug");
        assert_eq!(config.level, LogLevel::Debug);

        let info_toml = r#"level = "info""#;
        let config: LoggingConfig = toml::from_str(info_toml).expect("Failed to parse info");
        assert_eq!(config.level, LogLevel::Info);

        let warn_toml = r#"level = "warn""#;
        let config: LoggingConfig = toml::from_str(warn_toml).expect("Failed to parse warn");
        assert_eq!(config.level, LogLevel::Warn);

        let error_toml = r#"level = "error""#;
        let config: LoggingConfig = toml::from_str(error_toml).expect("Failed to parse error");
        assert_eq!(config.level, LogLevel::Error);
    }

    #[test]
    fn test_default_config_satisfies_spec_requirements() {
        let config = LoggingConfig::default();
        assert_eq!(config.max_file_size_mb, 50);
        assert_eq!(config.max_rotated_files, 5);
        assert_eq!(config.memory_buffer_size, 10000);
        assert_eq!(config.retention_days, 30);
    }

    #[test]
    fn test_tui_position_serde_roundtrip() {
        for position in &[TuiLogPosition::Bottom, TuiLogPosition::Right, TuiLogPosition::Overlay] {
            let json = serde_json::to_string(position).expect("Failed to serialize");
            let deserialized: TuiLogPosition = serde_json::from_str(&json).expect("Failed to deserialize");
            assert_eq!(*position, deserialized);
        }
    }
}