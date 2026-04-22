//! Log query filtering and criteria.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use super::event::{LogEvent, LogLevel};

/// Query criteria for filtering logs
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct LogQuery {
    /// Filter by session
    pub session_id: Option<String>,
    /// Filter by log level
    pub level: Option<LogLevel>,
    /// Filter by target component
    pub target: Option<String>,
    /// Filter logs after this time
    pub since: Option<DateTime<Utc>>,
    /// Filter logs before this time
    pub until: Option<DateTime<Utc>>,
    /// Maximum results to return
    pub limit: Option<usize>,
}

impl LogQuery {
    /// Create a new LogQuery with no filters (matches everything)
    pub fn new() -> Self {
        Self::default()
    }

    /// Filter by session_id
    pub fn with_session_id(mut self, session_id: impl Into<String>) -> Self {
        self.session_id = Some(session_id.into());
        self
    }

    /// Filter by level
    pub fn with_level(mut self, level: LogLevel) -> Self {
        self.level = Some(level);
        self
    }

    /// Filter by target
    pub fn with_target(mut self, target: impl Into<String>) -> Self {
        self.target = Some(target.into());
        self
    }

    /// Filter logs after this time
    pub fn with_since(mut self, since: DateTime<Utc>) -> Self {
        self.since = Some(since);
        self
    }

    /// Filter logs before this time
    pub fn with_until(mut self, until: DateTime<Utc>) -> Self {
        self.until = Some(until);
        self
    }

    /// Set maximum results
    pub fn with_limit(mut self, limit: usize) -> Self {
        self.limit = Some(limit);
        self
    }

    /// Create a query that filters by a specific session
    pub fn for_session(session_id: impl Into<String>) -> Self {
        Self {
            session_id: Some(session_id.into()),
            ..Default::default()
        }
    }

    /// Create a query that filters by a specific log level
    pub fn for_level(level: LogLevel) -> Self {
        Self {
            level: Some(level),
            ..Default::default()
        }
    }

    /// Check if an event matches this query
    pub fn matches(&self, event: &LogEvent) -> bool {
        if let Some(ref session_id) = self.session_id {
            if event.fields.session_id.as_ref() != Some(session_id) {
                return false;
            }
        }

        if let Some(ref level) = self.level {
            if event.level != *level {
                return false;
            }
        }

        if let Some(ref target) = self.target {
            if !glob_match(target, &event.target) {
                return false;
            }
        }

        if let Some(ref since) = self.since {
            if event.timestamp < *since {
                return false;
            }
        }

        if let Some(ref until) = self.until {
            if event.timestamp > *until {
                return false;
            }
        }

        true
    }
}

/// Simple glob pattern matching (supports * wildcards)
fn glob_match(pattern: &str, text: &str) -> bool {
    if pattern.is_empty() {
        return text.is_empty();
    }

    if pattern == "*" {
        return true;
    }

    let pattern_chars: Vec<char> = pattern.chars().collect();
    let text_chars: Vec<char> = text.chars().collect();

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

    #[test]
    fn test_query_default() {
        let query = LogQuery::new();
        let event = LogEvent::new(1, LogLevel::Info, "test", "message");
        assert!(query.matches(&event));
    }

    #[test]
    fn test_query_by_session() {
        let query = LogQuery::new().with_session_id("sess_123");
        let event = LogEvent::new(1, LogLevel::Info, "test", "message").with_session_id("sess_123");
        assert!(query.matches(&event));

        let event2 =
            LogEvent::new(2, LogLevel::Info, "test", "message").with_session_id("sess_456");
        assert!(!query.matches(&event2));
    }

    #[test]
    fn test_query_by_level() {
        let query = LogQuery::new().with_level(LogLevel::Error);
        let error_event = LogEvent::new(1, LogLevel::Error, "test", "error");
        let info_event = LogEvent::new(2, LogLevel::Info, "test", "info");

        assert!(query.matches(&error_event));
        assert!(!query.matches(&info_event));
    }

    #[test]
    fn test_query_by_target() {
        let query = LogQuery::new().with_target("llm.*");
        let openai_event = LogEvent::new(1, LogLevel::Info, "llm.openai", "response");
        let anthropic_event = LogEvent::new(2, LogLevel::Info, "llm.anthropic", "response");
        let tool_event = LogEvent::new(3, LogLevel::Info, "tool.read", "read file");

        assert!(query.matches(&openai_event));
        assert!(query.matches(&anthropic_event));
        assert!(!query.matches(&tool_event));
    }

    #[test]
    fn test_glob_matching() {
        assert!(glob_match("agent", "agent"));
        assert!(!glob_match("agent", "agent.extra"));
        assert!(glob_match("agent*", "agent.extra"));
        assert!(glob_match("llm.*", "llm.openai"));
        assert!(glob_match("llm.*", "llm.anthropic"));
        assert!(!glob_match("llm.*", "llm"));
        assert!(glob_match("*", "anything"));
        assert!(glob_match("tool.*.read", "tool.bash.read"));
    }

    #[test]
    fn test_query_time_range() {
        let now = chrono::Utc::now();
        let query = LogQuery::new()
            .with_since(now - chrono::Duration::hours(1))
            .with_until(now + chrono::Duration::hours(1));

        let event = LogEvent::new(1, LogLevel::Info, "test", "message");
        assert!(query.matches(&event));
    }

    #[test]
    fn test_query_limit() {
        let query = LogQuery::new().with_limit(10);
        assert_eq!(query.limit, Some(10));
    }
}
