//! Core Logger trait and implementation.

use std::fmt::Debug;
use std::fs::{self, File, OpenOptions};
use std::io::Write;
use std::path::PathBuf;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;

use async_trait::async_trait;
use tokio::sync::RwLock;

use crate::config::LoggingConfig;
use crate::error::LogError;
use crate::event::{LogEvent, LogFields, LogLevel};
use crate::query::LogQuery;
use crate::store::SessionLogBuffer;

pub type AgentLoggerImpl = Logger;

pub struct ChildLogger {
    parent: Arc<dyn AgentLogger>,
    context: LogFields,
}

impl Debug for ChildLogger {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ChildLogger")
            .field("context", &self.context)
            .finish()
    }
}

impl ChildLogger {
    pub fn new(parent: Arc<dyn AgentLogger>, context: LogFields) -> Self {
        Self { parent, context }
    }

    pub fn context(&self) -> &LogFields {
        &self.context
    }

    #[allow(clippy::field_reassign_with_default)]
    fn merge_fields(&self, fields: LogFields) -> LogFields {
        let caller_fields = fields;
        let ctx = &self.context;

        let mut merged = LogFields::default();

        merged.session_id = caller_fields.session_id.clone().or(ctx.session_id.clone());
        merged.tool_name = caller_fields.tool_name.clone().or(ctx.tool_name.clone());
        merged.latency_ms = caller_fields.latency_ms.or(ctx.latency_ms);
        merged.model = caller_fields.model.clone().or(ctx.model.clone());
        merged.provider = caller_fields.provider.clone().or(ctx.provider.clone());
        merged.token_count = caller_fields.token_count.or(ctx.token_count);
        merged.error_code = caller_fields.error_code.clone().or(ctx.error_code.clone());
        merged.file_path = caller_fields.file_path.clone().or(ctx.file_path.clone());
        merged.line = caller_fields.line.or(ctx.line);

        let mut extra = ctx.extra.clone();
        extra.extend(caller_fields.extra);
        merged.extra = extra;

        merged
    }
}

#[async_trait]
pub trait AgentLogger: Send + Sync {
    fn trace(&self, target: &str, message: &str, fields: LogFields);
    fn debug(&self, target: &str, message: &str, fields: LogFields);
    fn info(&self, target: &str, message: &str, fields: LogFields);
    fn warn(&self, target: &str, message: &str, fields: LogFields);
    fn error(&self, target: &str, message: &str, fields: LogFields);

    fn with_context(&self, context: LogFields) -> ChildLogger;

    async fn query(&self, criteria: LogQuery) -> Result<Vec<LogEvent>, LogError>;
}

pub struct Logger {
    config: LoggingConfig,
    buffer: Arc<RwLock<SessionLogBuffer>>,
    next_seq: Arc<AtomicU64>,
    file_path: Option<PathBuf>,
    file: Arc<RwLock<Option<File>>>,
}

impl Debug for Logger {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Logger")
            .field("config", &self.config)
            .finish()
    }
}

impl Clone for Logger {
    fn clone(&self) -> Self {
        Self {
            config: self.config.clone(),
            buffer: Arc::clone(&self.buffer),
            next_seq: Arc::clone(&self.next_seq),
            file_path: self.file_path.clone(),
            file: Arc::clone(&self.file),
        }
    }
}

impl Logger {
    pub fn new(config: LoggingConfig) -> Result<Self, LogError> {
        let buffer = SessionLogBuffer::new(config.memory_buffer_size);

        let file_path = config.file_path.clone();

        if let Some(ref path) = file_path {
            if let Some(dir) = path.parent() {
                std::fs::create_dir_all(dir).ok();
            }
        }

        let file = Self::open_log_file(file_path.as_ref())?;

        Ok(Self {
            config,
            buffer: Arc::new(RwLock::new(buffer)),
            next_seq: Arc::new(AtomicU64::new(1)),
            file_path,
            file: Arc::new(RwLock::new(file)),
        })
    }

    fn open_log_file(path: Option<&PathBuf>) -> Result<Option<File>, LogError> {
        match path {
            Some(p) => {
                let file = OpenOptions::new()
                    .create(true)
                    .append(true)
                    .open(p)
                    .map_err(|e| {
                        LogError::Io(std::io::Error::other(format!(
                            "Failed to open log file: {}",
                            e
                        )))
                    })?;
                Ok(Some(file))
            }
            None => Ok(None),
        }
    }

    fn max_file_size_bytes(&self) -> u64 {
        (self.config.max_file_size_mb * 1024 * 1024) as u64
    }

    async fn check_and_rotate(&self) -> Result<(), LogError> {
        let path = match &self.file_path {
            Some(p) => p.clone(),
            None => return Ok(()),
        };

        let file_size = fs::metadata(&path).map(|m| m.len()).unwrap_or(0);
        if file_size < self.max_file_size_bytes() {
            return Ok(());
        }

        {
            let mut file_guard = self.file.write().await;
            *file_guard = None;
        }

        self.rotate_logs(&path).await?;

        Ok(())
    }

    async fn rotate_logs(&self, path: &PathBuf) -> Result<(), LogError> {
        let max_rotated = self.config.max_rotated_files;

        for n in (1..=max_rotated).rev() {
            let old_path = if n == 1 {
                format!("{}.1", path.display())
            } else {
                format!("{}.{}", path.display(), n)
            };

            let existing = PathBuf::from(&old_path);
            if existing.exists() {
                let new_path = if n == max_rotated {
                    existing.clone()
                } else {
                    PathBuf::from(format!("{}.{}", path.display(), n + 1))
                };

                fs::rename(&existing, &new_path).map_err(|e| {
                    LogError::Io(std::io::Error::other(format!(
                        "Failed to rotate log file {}: {}",
                        old_path, e
                    )))
                })?;
            }
        }

        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).ok();
        }
        let backup_path = format!("{}.1", path.display());
        fs::rename(path, &backup_path).map_err(|e| {
            LogError::Io(std::io::Error::other(format!(
                "Failed to rotate log file to {}: {}",
                backup_path, e
            )))
        })?;

        let new_file = OpenOptions::new()
            .create(true)
            .write(true)
            .truncate(true)
            .open(path)
            .map_err(|e| {
                LogError::Io(std::io::Error::other(format!(
                    "Failed to create new log file: {}",
                    e
                )))
            })?;

        let mut file_guard = self.file.write().await;
        *file_guard = Some(new_file);

        Ok(())
    }

    pub fn should_log(&self, target: &str, level: LogLevel) -> bool {
        if let Some(target_level) = self.config.targets.get(target) {
            return level >= *target_level;
        }

        for (pattern, target_level) in &self.config.targets {
            if glob_match_pattern(pattern, target) {
                return level >= *target_level;
            }
        }

        level >= self.config.level
    }

    async fn log_event_async(
        &self,
        level: LogLevel,
        target: &str,
        message: &str,
        fields: LogFields,
    ) {
        if !self.should_log(target, level) {
            return;
        }

        let seq = self.next_seq.fetch_add(1, Ordering::SeqCst);
        let timestamp = chrono::Utc::now();
        let target_str = target.to_string();
        let message_str = message.to_string();
        let level_str = format!("{:?}", level);

        let event = LogEvent {
            seq,
            timestamp,
            level,
            target: target_str.clone(),
            message: message_str.clone(),
            fields,
            span_id: None,
            parent_seq: None,
        };

        let log_line = if self.file_path.is_some() {
            Some(format!(
                "[{}] {} {}: {}\n",
                event.timestamp.to_rfc3339(),
                level_str,
                target_str,
                message_str
            ))
        } else {
            None
        };

        let mut buffer = self.buffer.write().await;
        buffer.push(event);

        if let Some(ref path) = self.file_path {
            let log_line_str = log_line.clone();

            {
                let mut file_guard = self.file.write().await;
                if let Some(ref mut file) = *file_guard {
                    let _ =
                        file.write_all(log_line_str.as_ref().map(|l| l.as_bytes()).unwrap_or(&[]));
                }
            }

            drop(buffer);

            let file_size = fs::metadata(path).map(|m| m.len()).unwrap_or(0);
            if file_size >= self.max_file_size_bytes() {
                let _ = self.check_and_rotate().await;
            }
        }
    }

    pub async fn query_logs(&self, criteria: LogQuery) -> Result<Vec<LogEvent>, LogError> {
        let buffer = self.buffer.read().await;
        let events: Vec<LogEvent> = buffer
            .iter()
            .filter(|e| criteria.matches(e))
            .cloned()
            .collect();

        let mut events = events;
        if let Some(limit) = criteria.limit {
            events.truncate(limit);
        }

        Ok(events)
    }
}

#[async_trait]
impl AgentLogger for Logger {
    fn trace(&self, target: &str, message: &str, fields: LogFields) {
        let this = self.clone();
        let target = target.to_string();
        let message = message.to_string();
        let fields = fields.clone();
        tokio::spawn(async move {
            this.log_event_async(LogLevel::Trace, &target, &message, fields)
                .await;
        });
    }

    fn debug(&self, target: &str, message: &str, fields: LogFields) {
        let this = self.clone();
        let target = target.to_string();
        let message = message.to_string();
        let fields = fields.clone();
        tokio::spawn(async move {
            this.log_event_async(LogLevel::Debug, &target, &message, fields)
                .await;
        });
    }

    fn info(&self, target: &str, message: &str, fields: LogFields) {
        let this = self.clone();
        let target = target.to_string();
        let message = message.to_string();
        let fields = fields.clone();
        tokio::spawn(async move {
            this.log_event_async(LogLevel::Info, &target, &message, fields)
                .await;
        });
    }

    fn warn(&self, target: &str, message: &str, fields: LogFields) {
        let this = self.clone();
        let target = target.to_string();
        let message = message.to_string();
        let fields = fields.clone();
        tokio::spawn(async move {
            this.log_event_async(LogLevel::Warn, &target, &message, fields)
                .await;
        });
    }

    fn error(&self, target: &str, message: &str, fields: LogFields) {
        let this = self.clone();
        let target = target.to_string();
        let message = message.to_string();
        let fields = fields.clone();
        tokio::spawn(async move {
            this.log_event_async(LogLevel::Error, &target, &message, fields)
                .await;
        });
    }

    fn with_context(&self, context: LogFields) -> ChildLogger {
        ChildLogger::new(Arc::new(self.clone()), context)
    }

    async fn query(&self, criteria: LogQuery) -> Result<Vec<LogEvent>, LogError> {
        self.query_logs(criteria).await
    }
}

#[async_trait]
impl AgentLogger for ChildLogger {
    fn trace(&self, target: &str, message: &str, fields: LogFields) {
        let merged = self.merge_fields(fields);
        self.parent.trace(target, message, merged);
    }

    fn debug(&self, target: &str, message: &str, fields: LogFields) {
        let merged = self.merge_fields(fields);
        self.parent.debug(target, message, merged);
    }

    fn info(&self, target: &str, message: &str, fields: LogFields) {
        let merged = self.merge_fields(fields);
        self.parent.info(target, message, merged);
    }

    fn warn(&self, target: &str, message: &str, fields: LogFields) {
        let merged = self.merge_fields(fields);
        self.parent.warn(target, message, merged);
    }

    fn error(&self, target: &str, message: &str, fields: LogFields) {
        let merged = self.merge_fields(fields);
        self.parent.error(target, message, merged);
    }

    fn with_context(&self, context: LogFields) -> ChildLogger {
        let mut merged_context = self.context.clone();
        if let Some(session_id) = context.session_id.clone() {
            merged_context.session_id = Some(session_id);
        }
        if let Some(tool_name) = context.tool_name.clone() {
            merged_context.tool_name = Some(tool_name);
        }
        if context.latency_ms.is_some() {
            merged_context.latency_ms = context.latency_ms;
        }
        if let Some(model) = context.model.clone() {
            merged_context.model = Some(model);
        }
        if let Some(provider) = context.provider.clone() {
            merged_context.provider = Some(provider);
        }
        if context.token_count.is_some() {
            merged_context.token_count = context.token_count;
        }
        if let Some(error_code) = context.error_code.clone() {
            merged_context.error_code = Some(error_code);
        }
        if let Some(file_path) = context.file_path.clone() {
            merged_context.file_path = Some(file_path);
        }
        if context.line.is_some() {
            merged_context.line = context.line;
        }
        merged_context.extra.extend(context.extra);

        ChildLogger::new(self.parent.clone(), merged_context)
    }

    async fn query(&self, criteria: LogQuery) -> Result<Vec<LogEvent>, LogError> {
        self.parent.query(criteria).await
    }
}

fn glob_match_pattern(pattern: &str, target: &str) -> bool {
    if pattern.is_empty() {
        return target.is_empty();
    }

    if pattern == "*" {
        return true;
    }

    let pattern_chars: Vec<char> = pattern.chars().collect();
    let text_chars: Vec<char> = target.chars().collect();

    let mut p_idx = 0;
    let mut t_idx = 0;
    let mut star_idx: Option<usize> = None;
    let mut t_star_idx: Option<usize> = None;

    while t_idx < text_chars.len() {
        if p_idx < pattern_chars.len()
            && (pattern_chars[p_idx] == text_chars[t_idx] || pattern_chars[p_idx] == '*')
        {
            if pattern_chars[p_idx] == '*' {
                star_idx = Some(p_idx);
                t_star_idx = Some(t_idx);
                p_idx += 1;
            } else {
                p_idx += 1;
                t_idx += 1;
            }
        } else if let Some(si) = star_idx {
            p_idx = si + 1;
            t_idx = t_star_idx.unwrap() + 1;
            t_star_idx = Some(t_idx);
        } else {
            return false;
        }
    }

    while p_idx < pattern_chars.len() && pattern_chars[p_idx] == '*' {
        p_idx += 1;
    }

    p_idx == pattern_chars.len()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn test_glob_match_pattern() {
        assert!(glob_match_pattern("agent", "agent"));
        assert!(!glob_match_pattern("agent", "agent.extra"));
        assert!(glob_match_pattern("agent*", "agent.extra"));
        assert!(glob_match_pattern("llm.*", "llm.openai"));
        assert!(glob_match_pattern("tool.*.read", "tool.bash.read"));
        assert!(glob_match_pattern("*", "anything"));
    }

    #[tokio::test]
    async fn test_logger_creation() {
        let config = LoggingConfig::default();
        let _logger = Logger::new(config).unwrap();
    }

    #[tokio::test]
    async fn test_logger_should_log() {
        let config = LoggingConfig::default();
        let logger = Logger::new(config).unwrap();

        assert!(logger.should_log("agent", LogLevel::Info));
        assert!(!logger.should_log("agent", LogLevel::Trace));
    }

    #[tokio::test]
    async fn test_all_log_levels_emit_correctly() {
        let config = LoggingConfig {
            level: LogLevel::Trace,
            ..Default::default()
        };
        let logger = Logger::new(config).unwrap();

        logger.trace("test", "trace message", LogFields::default());
        tokio::task::yield_now().await;
        logger.debug("test", "debug message", LogFields::default());
        tokio::task::yield_now().await;
        logger.info("test", "info message", LogFields::default());
        tokio::task::yield_now().await;
        logger.warn("test", "warn message", LogFields::default());
        tokio::task::yield_now().await;
        logger.error("test", "error message", LogFields::default());
        tokio::task::yield_now().await;

        let all_events = logger.query_logs(LogQuery::new()).await.unwrap();
        assert_eq!(all_events.len(), 5);

        let levels: Vec<LogLevel> = all_events.iter().map(|e| e.level).collect();
        assert!(levels.contains(&LogLevel::Trace));
        assert!(levels.contains(&LogLevel::Debug));
        assert!(levels.contains(&LogLevel::Info));
        assert!(levels.contains(&LogLevel::Warn));
        assert!(levels.contains(&LogLevel::Error));
    }

    #[tokio::test]
    async fn test_child_logger_context_chaining() {
        let config = LoggingConfig::default();
        let logger = Logger::new(config).unwrap();

        let child1 = logger.with_context(LogFields::with_session_id("session_1"));
        child1.info("test", "from child1", LogFields::default());
        tokio::task::yield_now().await;

        let grandchild_context = LogFields::default().with_tool_name("tool1");
        let grandchild = child1.with_context(grandchild_context);
        grandchild.info("test", "from grandchild", LogFields::default());
        tokio::task::yield_now().await;

        let all_events = logger.query_logs(LogQuery::new()).await.unwrap();
        assert_eq!(all_events.len(), 2);

        let child1_event = &all_events[0];
        assert_eq!(
            child1_event.fields.session_id,
            Some("session_1".to_string())
        );

        let grandchild_event = &all_events[1];
        assert_eq!(
            grandchild_event.fields.session_id,
            Some("session_1".to_string())
        );
        assert_eq!(grandchild_event.fields.tool_name, Some("tool1".to_string()));
    }

    #[tokio::test]
    async fn test_with_context_creates_child_with_merged_fields() {
        let config = LoggingConfig::default();
        let logger = Logger::new(config).unwrap();

        let parent_fields = LogFields {
            session_id: Some("parent_session".to_string()),
            tool_name: Some("parent_tool".to_string()),
            latency_ms: Some(100),
            ..Default::default()
        };

        let child = logger.with_context(parent_fields);

        let caller_fields = LogFields {
            tool_name: Some("caller_tool".to_string()),
            latency_ms: Some(200),
            error_code: Some("ERR_TEST".to_string()),
            ..Default::default()
        };

        let merged = child.merge_fields(caller_fields);

        assert_eq!(merged.session_id, Some("parent_session".to_string()));
        assert_eq!(merged.tool_name, Some("caller_tool".to_string()));
        assert_eq!(merged.latency_ms, Some(200));
        assert_eq!(merged.error_code, Some("ERR_TEST".to_string()));
    }

    #[tokio::test]
    async fn test_query_returns_matching_events() {
        let config = LoggingConfig::default();
        let logger = Logger::new(config).unwrap();

        logger.info("agent", "message 1", LogFields::with_session_id("sess_1"));
        tokio::task::yield_now().await;
        logger.error("agent", "message 2", LogFields::with_session_id("sess_1"));
        tokio::task::yield_now().await;
        logger.info("agent", "message 3", LogFields::with_session_id("sess_2"));
        tokio::task::yield_now().await;

        let sess1_query = LogQuery::new().with_session_id("sess_1");
        let sess1_events = logger.query(sess1_query).await.unwrap();
        assert_eq!(sess1_events.len(), 2);

        let sess2_query = LogQuery::new().with_session_id("sess_2");
        let sess2_events = logger.query(sess2_query).await.unwrap();
        assert_eq!(sess2_events.len(), 1);

        let level_query = LogQuery::new().with_level(LogLevel::Error);
        let error_events = logger.query(level_query).await.unwrap();
        assert_eq!(error_events.len(), 1);
        assert_eq!(error_events[0].message, "message 2");
    }

    #[tokio::test]
    async fn test_trait_objects_work_across_implementations() {
        let config = LoggingConfig::default();
        let logger: Arc<dyn AgentLogger> = Arc::new(Logger::new(config).unwrap());

        logger.info("test", "direct call", LogFields::default());
        tokio::task::yield_now().await;

        let child = logger.with_context(LogFields::with_session_id("child_session"));
        let child: Arc<dyn AgentLogger> = Arc::new(child);
        child.info("test", "child call", LogFields::default());
        tokio::task::yield_now().await;

        let all_events = logger.query(LogQuery::new()).await.unwrap();
        assert_eq!(all_events.len(), 2);

        let child_only = child.query(LogQuery::new()).await.unwrap();
        assert_eq!(child_only.len(), 2);
    }

    #[tokio::test]
    async fn test_rotation_triggered_at_max_file_size() {
        let temp_dir = TempDir::new().unwrap();
        let log_path = temp_dir.path().join("opencode.log");

        let config = LoggingConfig {
            file_path: Some(log_path.clone()),
            max_file_size_mb: 1,
            max_rotated_files: 3,
            ..Default::default()
        };

        let logger = Logger::new(config).unwrap();

        let msg_len = 100usize;
        let msgs_to_fill = (1024 * 1024 * 2) / msg_len;
        let batch_size = 500;

        for batch_start in (0..msgs_to_fill).step_by(batch_size) {
            let batch_end = std::cmp::min(batch_start + batch_size, msgs_to_fill);
            let mut handles = vec![];

            for i in batch_start..batch_end {
                let logger_clone = logger.clone();
                handles.push(tokio::spawn(async move {
                    logger_clone.info("test", &format!("message {:05}", i), LogFields::default());
                }));
            }

            for handle in handles {
                let _ = handle.await;
            }
        }

        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
        tokio::task::yield_now().await;

        let rotated_exists = (1..=3).any(|n| {
            let rotated_path = temp_dir.path().join(format!("opencode.log.{}", n));
            rotated_path.exists()
        });
        assert!(
            rotated_exists,
            "At least one rotated file should exist when exceeding max size"
        );

        assert!(
            log_path.exists(),
            "New log file should be created after rotation"
        );
    }

    #[tokio::test]
    async fn test_rotated_file_numbering_increments_correctly() {
        let temp_dir = TempDir::new().unwrap();
        let log_path = temp_dir.path().join("opencode.log");

        let config = LoggingConfig {
            file_path: Some(log_path.clone()),
            max_file_size_mb: 1,
            max_rotated_files: 3,
            ..Default::default()
        };

        let logger = Logger::new(config).unwrap();

        let msg_len = 100usize;
        let msgs_to_fill = (1024 * 1024 * 4) / msg_len;
        let batch_size = 500;

        for batch_start in (0..msgs_to_fill).step_by(batch_size) {
            let batch_end = std::cmp::min(batch_start + batch_size, msgs_to_fill);
            let mut handles = vec![];

            for i in batch_start..batch_end {
                let logger_clone = logger.clone();
                handles.push(tokio::spawn(async move {
                    logger_clone.info("test", &format!("message {:05}", i), LogFields::default());
                }));
            }

            for handle in handles {
                let _ = handle.await;
            }
        }

        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
        tokio::task::yield_now().await;

        let file_1_exists = temp_dir.path().join("opencode.log.1").exists();
        let file_2_exists = temp_dir.path().join("opencode.log.2").exists();
        let _file_3_exists = temp_dir.path().join("opencode.log.3").exists();

        assert!(
            file_1_exists,
            "opencode.log.1 should exist after first rotation"
        );
        assert!(
            file_2_exists,
            "opencode.log.2 should exist after second rotation"
        );
    }

    #[tokio::test]
    async fn test_oldest_file_deleted_when_max_rotated_files_exceeded() {
        let temp_dir = TempDir::new().unwrap();
        let log_path = temp_dir.path().join("opencode.log");

        let config = LoggingConfig {
            file_path: Some(log_path.clone()),
            max_file_size_mb: 1,
            max_rotated_files: 3,
            ..Default::default()
        };

        let logger = Logger::new(config).unwrap();

        let msg_len = 100usize;
        let msgs_to_fill = (1024 * 1024 * 6) / msg_len;
        let batch_size = 500;

        for batch_start in (0..msgs_to_fill).step_by(batch_size) {
            let batch_end = std::cmp::min(batch_start + batch_size, msgs_to_fill);
            let mut handles = vec![];

            for i in batch_start..batch_end {
                let logger_clone = logger.clone();
                handles.push(tokio::spawn(async move {
                    logger_clone.info("test", &format!("message {:05}", i), LogFields::default());
                }));
            }

            for handle in handles {
                let _ = handle.await;
            }
        }

        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
        tokio::task::yield_now().await;

        assert!(
            temp_dir.path().join("opencode.log.1").exists(),
            "opencode.log.1 should exist"
        );
        assert!(
            temp_dir.path().join("opencode.log.2").exists(),
            "opencode.log.2 should exist"
        );
        assert!(
            temp_dir.path().join("opencode.log.3").exists(),
            "opencode.log.3 should exist"
        );
        assert!(
            !temp_dir.path().join("opencode.log.4").exists(),
            "opencode.log.4 should NOT exist (oldest deleted)"
        );
    }

    #[tokio::test]
    async fn test_new_log_file_created_after_rotation() {
        let temp_dir = TempDir::new().unwrap();
        let log_path = temp_dir.path().join("opencode.log");

        let config = LoggingConfig {
            file_path: Some(log_path.clone()),
            max_file_size_mb: 1,
            max_rotated_files: 3,
            ..Default::default()
        };

        let logger = Logger::new(config).unwrap();

        let msg_len = 100usize;
        let msgs_to_fill = (1024 * 1024 * 2) / msg_len;
        let batch_size = 500;

        for batch_start in (0..msgs_to_fill).step_by(batch_size) {
            let batch_end = std::cmp::min(batch_start + batch_size, msgs_to_fill);
            let mut handles = vec![];

            for i in batch_start..batch_end {
                let logger_clone = logger.clone();
                handles.push(tokio::spawn(async move {
                    logger_clone.info("test", &format!("message {:05}", i), LogFields::default());
                }));
            }

            for handle in handles {
                let _ = handle.await;
            }
        }

        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
        tokio::task::yield_now().await;

        assert!(
            log_path.exists(),
            "opencode.log should exist after rotation"
        );

        let metadata = fs::metadata(&log_path).unwrap();
        assert!(
            metadata.len() < 1024 * 1024,
            "New log file should be smaller than max size after rotation"
        );
    }
}
