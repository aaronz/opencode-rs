//! Logging configuration.

use std::collections::HashMap;
use std::path::PathBuf;

use serde::{Deserialize, Serialize};

use crate::event::LogLevel;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoggingConfig {
    pub level: LogLevel,
    pub targets: HashMap<String, LogLevel>,
    pub file_path: Option<PathBuf>,
    pub max_file_size_mb: usize,
    pub max_rotated_files: usize,
    pub show_in_tui: bool,
    pub tui_position: TuiLogPosition,
    pub memory_buffer_size: usize,
    pub retention_days: u32,
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
            max_file_size_mb: 10,
            max_rotated_files: 5,
            show_in_tui: false,
            tui_position: TuiLogPosition::Bottom,
            memory_buffer_size: 1000,
            retention_days: 7,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = LoggingConfig::default();
        assert_eq!(config.level, LogLevel::Info);
        assert_eq!(config.memory_buffer_size, 1000);
        assert!(!config.show_in_tui);
    }

    #[test]
    fn test_tui_position_default() {
        assert_eq!(TuiLogPosition::default(), TuiLogPosition::Bottom);
    }
}