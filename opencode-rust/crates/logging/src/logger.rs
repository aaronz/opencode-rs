//! Core Logger trait and implementation.

use std::fmt::Debug;
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
        }
    }
}

impl Logger {
    pub fn new(config: LoggingConfig) -> Result<Self, LogError> {
        let buffer = SessionLogBuffer::new(config.memory_buffer_size);

        if let Some(ref path) = config.file_path {
            if let Some(dir) = path.parent() {
                std::fs::create_dir_all(dir).ok();
            }
        }

        Ok(Self {
            config,
            buffer: Arc::new(RwLock::new(buffer)),
            next_seq: Arc::new(AtomicU64::new(1)),
        })
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

    async fn log_event_async(&self, level: LogLevel, target: &str, message: &str, fields: LogFields) {
        if !self.should_log(target, level) {
            return;
        }

        let seq = self.next_seq.fetch_add(1, Ordering::SeqCst);

        let event = LogEvent {
            seq,
            timestamp: chrono::Utc::now(),
            level,
            target: target.to_string(),
            message: message.to_string(),
            fields,
            span_id: None,
            parent_seq: None,
        };

        let mut buffer = self.buffer.write().await;
        buffer.push(event);
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
        tokio::task::spawn(async move {
            this.log_event_async(LogLevel::Trace, &target, &message, fields).await;
        });
    }

    fn debug(&self, target: &str, message: &str, fields: LogFields) {
        let this = self.clone();
        let target = target.to_string();
        let message = message.to_string();
        tokio::task::spawn(async move {
            this.log_event_async(LogLevel::Debug, &target, &message, fields).await;
        });
    }

    fn info(&self, target: &str, message: &str, fields: LogFields) {
        let this = self.clone();
        let target = target.to_string();
        let message = message.to_string();
        tokio::task::spawn(async move {
            this.log_event_async(LogLevel::Info, &target, &message, fields).await;
        });
    }

    fn warn(&self, target: &str, message: &str, fields: LogFields) {
        let this = self.clone();
        let target = target.to_string();
        let message = message.to_string();
        tokio::task::spawn(async move {
            this.log_event_async(LogLevel::Warn, &target, &message, fields).await;
        });
    }

    fn error(&self, target: &str, message: &str, fields: LogFields) {
        let this = self.clone();
        let target = target.to_string();
        let message = message.to_string();
        tokio::task::spawn(async move {
            this.log_event_async(LogLevel::Error, &target, &message, fields).await;
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
        if p_idx < pattern_chars.len() && (pattern_chars[p_idx] == text_chars[t_idx] || pattern_chars[p_idx] == '*') {
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
        let mut config = LoggingConfig::default();
        config.level = LogLevel::Trace;
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
        assert_eq!(child1_event.fields.session_id, Some("session_1".to_string()));

        let grandchild_event = &all_events[1];
        assert_eq!(grandchild_event.fields.session_id, Some("session_1".to_string()));
        assert_eq!(grandchild_event.fields.tool_name, Some("tool1".to_string()));
    }

    #[tokio::test]
    async fn test_with_context_creates_child_with_merged_fields() {
        let config = LoggingConfig::default();
        let logger = Logger::new(config).unwrap();

        let mut parent_fields = LogFields::default();
        parent_fields.session_id = Some("parent_session".to_string());
        parent_fields.tool_name = Some("parent_tool".to_string());
        parent_fields.latency_ms = Some(100);

        let child = logger.with_context(parent_fields);

        let mut caller_fields = LogFields::default();
        caller_fields.tool_name = Some("caller_tool".to_string());
        caller_fields.latency_ms = Some(200);
        caller_fields.error_code = Some("ERR_TEST".to_string());

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
}