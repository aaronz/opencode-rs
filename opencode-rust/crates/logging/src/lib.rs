//! OpenCode Logging System
//!
//! Structured logging for the OpenCode coding agent.

pub mod config;
pub mod error;
pub mod event;
pub mod logger;
pub mod macros;
pub mod query;
pub mod sanitizer;
pub mod store;

#[cfg(feature = "tui")]
pub mod tui;

pub use config::{LoggingConfig, TuiLogPosition};
pub use error::LogError;
pub use event::{
    CauseInfo, ErrorContext, ErrorFrame, LogEvent, LogFields, LogLevel, ReasoningLog,
    SanitizedValue, ToolConsideration, ToolExecutionLog, ToolResult,
};
pub use logger::{AgentLogger, AgentLoggerImpl, ChildLogger};
pub use query::LogQuery;
pub use sanitizer::Sanitizer;
pub use store::{LogStore, SessionLogBuffer};

#[cfg(feature = "tui")]
pub use tui::log_panel::LogPanel;

#[cfg(feature = "tui")]
pub use tui::log_renderer::LogRenderer;
