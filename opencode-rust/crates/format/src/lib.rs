//! OpenCode Format Crate
//!
//! Provides code formatting services with support for 25+ formatters across many languages.

pub use opencode_config::{FormatterConfig, FormatterEntry};
pub use opencode_core::formatter::{FormatterEngine, FormatterError};

pub use config::FormatConfig;

pub mod config;
pub mod formatters;
pub mod glob;
pub mod service;

pub use formatters::{
    all_formatters, formatter_names, Formatter, FormatterContext, FormatterStatus,
};
pub use glob::{entry_matches_file, matches_patterns};
pub use service::{FormatService, FormatServiceState, InstanceState, InstanceStateManager};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn format_service_creates() {
        let _service = service::FormatService::new();
        assert!(true);
    }

    #[test]
    fn formatter_config_exports() {
        let _disabled = FormatterConfig::Disabled(false);
        let _formatters = FormatterConfig::Formatters(std::collections::HashMap::new());
    }

    #[test]
    fn formatter_entry_exports() {
        let entry = FormatterEntry {
            disabled: None,
            command: Some(vec!["rustfmt".to_string()]),
            environment: None,
            extensions: Some(vec!["rs".to_string()]),
        };
        assert!(entry.command.is_some());
    }

    #[test]
    fn formatter_engine_exports() {
        let engine = FormatterEngine::new(FormatterConfig::Disabled(false));
        assert!(!engine.is_enabled());
    }

    #[test]
    fn formatter_error_exports() {
        let _disabled = FormatterError::Disabled;
        let _no_match = FormatterError::NoMatch("test.rs".to_string());
    }
}
