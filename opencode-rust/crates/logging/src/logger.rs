//! Core Logger trait and implementation.

use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;

use tokio::sync::RwLock;

use crate::config::LoggingConfig;
use crate::error::LogError;
use crate::event::{LogEvent, LogFields, LogLevel};
use crate::query::LogQuery;
use crate::store::SessionLogBuffer;

pub trait AgentLogger: Send + Sync {
    fn trace(&self, target: &str, message: &str, fields: LogFields);
    fn debug(&self, target: &str, message: &str, fields: LogFields);
    fn info(&self, target: &str, message: &str, fields: LogFields);
    fn warn(&self, target: &str, message: &str, fields: LogFields);
    fn error(&self, target: &str, message: &str, fields: LogFields);
}

pub struct Logger {
    config: LoggingConfig,
    buffer: Arc<RwLock<SessionLogBuffer>>,
    next_seq: AtomicU64,
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
            next_seq: AtomicU64::new(1),
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

    pub async fn query(&self, criteria: LogQuery) -> Result<Vec<LogEvent>, LogError> {
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

impl AgentLogger for Logger {
    fn trace(&self, target: &str, message: &str, fields: LogFields) {
        let rt = tokio::runtime::Handle::current();
        rt.block_on(self.log_event_async(LogLevel::Trace, target, message, fields));
    }

    fn debug(&self, target: &str, message: &str, fields: LogFields) {
        let rt = tokio::runtime::Handle::current();
        rt.block_on(self.log_event_async(LogLevel::Debug, target, message, fields));
    }

    fn info(&self, target: &str, message: &str, fields: LogFields) {
        let rt = tokio::runtime::Handle::current();
        rt.block_on(self.log_event_async(LogLevel::Info, target, message, fields));
    }

    fn warn(&self, target: &str, message: &str, fields: LogFields) {
        let rt = tokio::runtime::Handle::current();
        rt.block_on(self.log_event_async(LogLevel::Warn, target, message, fields));
    }

    fn error(&self, target: &str, message: &str, fields: LogFields) {
        let rt = tokio::runtime::Handle::current();
        rt.block_on(self.log_event_async(LogLevel::Error, target, message, fields));
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
}