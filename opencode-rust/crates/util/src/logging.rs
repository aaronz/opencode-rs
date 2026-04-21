use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use tracing::Level;
use tracing_appender::rolling::{RollingFileAppender, Rotation as LogRotation};
use tracing_subscriber::{
    fmt::{self, format::FmtSpan},
    layer::SubscriberExt,
    util::SubscriberInitExt,
    EnvFilter,
};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum LogLevel {
    Debug,
    #[default]
    Info,
    Warn,
    Error,
}

impl From<LogLevel> for Level {
    fn from(level: LogLevel) -> Self {
        match level {
            LogLevel::Debug => Level::DEBUG,
            LogLevel::Info => Level::INFO,
            LogLevel::Warn => Level::WARN,
            LogLevel::Error => Level::ERROR,
        }
    }
}

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct Rotation {
    max_size_bytes: usize,
    max_files: usize,
}

impl Rotation {
    pub fn new(max_size_mb: usize, max_files: usize) -> Self {
        Self {
            max_size_bytes: max_size_mb * 1024 * 1024,
            max_files,
        }
    }
}

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct Logger {
    level: LogLevel,
    file_path: Option<PathBuf>,
    console: bool,
}

impl Logger {
    pub fn new() -> Self {
        Self {
            level: LogLevel::Info,
            file_path: None,
            console: true,
        }
    }

    pub fn with_level(level: LogLevel) -> Self {
        Self {
            level,
            file_path: None,
            console: true,
        }
    }

    pub fn with_file(&mut self, path: impl Into<PathBuf>) -> &mut Self {
        self.file_path = Some(path.into());
        self
    }

    pub fn with_no_console(&mut self) -> &mut Self {
        self.console = false;
        self
    }

    pub fn init(&self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let env_filter =
            EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info"));

        let fmt_layer = fmt::layer()
            .with_target(true)
            .with_thread_ids(false)
            .with_file(true)
            .with_line_number(true)
            .with_span_events(FmtSpan::CLOSE);

        if let Some(file_path) = &self.file_path {
            let file_appender =
                RollingFileAppender::new(LogRotation::DAILY, file_path, "opencode.log");
            let (non_blocking, _guard) = tracing_appender::non_blocking(file_appender);

            let file_layer = fmt::layer()
                .with_target(true)
                .with_thread_ids(false)
                .with_file(true)
                .with_line_number(true)
                .with_span_events(FmtSpan::CLOSE)
                .with_writer(non_blocking);

            tracing_subscriber::registry()
                .with(env_filter)
                .with(fmt_layer)
                .with(file_layer)
                .try_init()?;
        } else {
            tracing_subscriber::registry()
                .with(env_filter)
                .with(fmt_layer)
                .try_init()?;
        }
        Ok(())
    }
}

impl Default for Logger {
    fn default() -> Self {
        Self::new()
    }
}

pub fn log_file_path() -> PathBuf {
    let home = std::env::var("HOME").unwrap_or_else(|_| ".".to_string());
    PathBuf::from(home)
        .join(".config")
        .join("opencode")
        .join("logs")
        .join("opencode.log")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_log_level_serde() {
        assert_eq!(
            serde_json::to_string(&LogLevel::Debug).unwrap(),
            "\"debug\""
        );
        assert_eq!(serde_json::to_string(&LogLevel::Info).unwrap(), "\"info\"");
        assert_eq!(serde_json::to_string(&LogLevel::Warn).unwrap(), "\"warn\"");
        assert_eq!(
            serde_json::to_string(&LogLevel::Error).unwrap(),
            "\"error\""
        );
    }

    #[test]
    fn test_log_level_from_tracing_level() {
        assert_eq!(Level::from(LogLevel::Debug), Level::DEBUG);
        assert_eq!(Level::from(LogLevel::Info), Level::INFO);
        assert_eq!(Level::from(LogLevel::Warn), Level::WARN);
        assert_eq!(Level::from(LogLevel::Error), Level::ERROR);
    }

    #[test]
    fn test_rotation_config() {
        let rotation = Rotation::new(10, 5);
        assert_eq!(rotation.max_size_bytes, 10 * 1024 * 1024);
        assert_eq!(rotation.max_files, 5);
    }
}
