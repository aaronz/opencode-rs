use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::sync::OnceLock;
use tracing::Level;
use tracing_appender::non_blocking::WorkerGuard;
use tracing_appender::rolling::{RollingFileAppender, Rotation as LogRotation};
use tracing_subscriber::{
    fmt::{format::FmtSpan, writer::MakeWriterExt},
    layer::SubscriberExt,
    util::SubscriberInitExt,
    EnvFilter,
};

static FILE_LOG_GUARD: OnceLock<WorkerGuard> = OnceLock::new();

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
            console: false,
        }
    }

    pub fn with_level(level: LogLevel) -> Self {
        Self {
            level,
            file_path: None,
            console: false,
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

    pub fn with_console(&mut self) -> &mut Self {
        self.console = true;
        self
    }

    pub fn init(&self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let env_filter =
            EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info"));

        if let Some(file_path) = &self.file_path {
            let log_dir = file_path.parent().expect("log file path must have a parent directory");
            let log_prefix = file_path
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("opencode.log");
            std::fs::create_dir_all(log_dir)?;

            let file_appender =
                RollingFileAppender::new(LogRotation::DAILY, log_dir, log_prefix);
            let (non_blocking, guard) = tracing_appender::non_blocking(file_appender);

            let _ = FILE_LOG_GUARD.set(guard);

            if self.console {
                let writer = std::io::stderr.and(non_blocking);
                tracing_subscriber::fmt()
                    .with_env_filter(env_filter)
                    .with_target(true)
                    .with_thread_ids(false)
                    .with_file(true)
                    .with_line_number(true)
                    .with_span_events(FmtSpan::CLOSE)
                    .with_writer(writer)
                    .try_init()?;
            } else {
                tracing_subscriber::fmt()
                    .with_env_filter(env_filter)
                    .with_target(true)
                    .with_thread_ids(false)
                    .with_file(true)
                    .with_line_number(true)
                    .with_span_events(FmtSpan::CLOSE)
                    .with_writer(non_blocking)
                    .try_init()?;
            }
        } else if self.console {
            tracing_subscriber::fmt()
                .with_env_filter(env_filter)
                .with_target(true)
                .with_thread_ids(false)
                .with_file(true)
                .with_line_number(true)
                .with_span_events(FmtSpan::CLOSE)
                .with_writer(std::io::stderr)
                .try_init()?;
        } else {
            tracing_subscriber::registry().with(env_filter).try_init()?;
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
    use opencode_core::paths::Paths;
    Paths::log_file()
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

    #[test]
    fn test_log_file_path_uses_opencode_rs_paths() {
        let log_path = log_file_path();
        let log_str = log_path.to_string_lossy();
        assert!(
            log_str.contains("opencode-rs"),
            "log_file_path should use opencode-rs paths, got: {}",
            log_str
        );
        assert!(
            !log_str.contains("/opencode/"),
            "log_file_path should NOT use /opencode/ paths, got: {}",
            log_str
        );
    }
}
