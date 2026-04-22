use crate::sealed;
use crate::tool::{Tool, ToolContext, ToolResult};
use async_trait::async_trait;
use chrono::{DateTime, NaiveDateTime, Utc};
use opencode_core::OpenCodeError;
use opencode_logging::{event::LogLevel, query::LogQuery, store::LogStore};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use std::sync::Mutex;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogQueryToolInput {
    pub session_id: Option<String>,
    pub level: Option<String>,
    pub target: Option<String>,
    pub since: Option<String>,
    pub until: Option<String>,
    pub limit: Option<usize>,
}

pub struct LogQueryTool {
    store: Arc<Mutex<Option<LogStore>>>,
}

impl LogQueryTool {
    pub fn new() -> Self {
        Self {
            store: Arc::new(Mutex::new(None)),
        }
    }

    pub fn with_store(store: LogStore) -> Self {
        Self {
            store: Arc::new(Mutex::new(Some(store))),
        }
    }

    fn parse_level(level_str: &str) -> Option<LogLevel> {
        match level_str.to_lowercase().as_str() {
            "trace" => Some(LogLevel::Trace),
            "debug" => Some(LogLevel::Debug),
            "info" => Some(LogLevel::Info),
            "warn" | "warning" => Some(LogLevel::Warn),
            "error" => Some(LogLevel::Error),
            _ => None,
        }
    }

    fn parse_datetime(datetime_str: &str) -> Option<DateTime<Utc>> {
        if let Ok(dt) = DateTime::parse_from_rfc3339(datetime_str) {
            return Some(dt.with_timezone(&Utc));
        }
        if let Ok(ndt) = NaiveDateTime::parse_from_str(datetime_str, "%Y-%m-%dT%H:%M:%S") {
            return Some(ndt.and_utc());
        }
        if let Ok(ndt) = NaiveDateTime::parse_from_str(datetime_str, "%Y-%m-%d %H:%M:%S") {
            return Some(ndt.and_utc());
        }
        None
    }
}

impl Default for LogQueryTool {
    fn default() -> Self {
        Self::new()
    }
}

impl Clone for LogQueryTool {
    fn clone(&self) -> Self {
        Self {
            store: Arc::clone(&self.store),
        }
    }
}

impl sealed::Sealed for LogQueryTool {}

#[async_trait]
impl Tool for LogQueryTool {
    fn name(&self) -> &str {
        "log_query"
    }

    fn description(&self) -> &str {
        "Query log events for agent self-diagnosis. Supports filtering by session_id, level, target, since, until, and limit."
    }

    fn clone_tool(&self) -> Box<dyn Tool> {
        Box::new(self.clone())
    }

    async fn execute(
        &self,
        args: serde_json::Value,
        _ctx: Option<ToolContext>,
    ) -> Result<ToolResult, OpenCodeError> {
        let input: LogQueryToolInput = serde_json::from_value(args)
            .map_err(|e| OpenCodeError::Parse(format!("Invalid arguments: {}", e)))?;

        let mut query = LogQuery::new();

        if let Some(session_id) = input.session_id {
            query = query.with_session_id(session_id);
        }

        if let Some(level_str) = input.level {
            if let Some(level) = Self::parse_level(&level_str) {
                query = query.with_level(level);
            } else {
                return Ok(ToolResult::err(format!("Invalid log level: {}", level_str)));
            }
        }

        if let Some(target) = input.target {
            query = query.with_target(target);
        }

        if let Some(since_str) = input.since {
            if let Some(since) = Self::parse_datetime(&since_str) {
                query = query.with_since(since);
            } else {
                return Ok(ToolResult::err(format!(
                    "Invalid since date format: {}. Use ISO 8601 format (e.g., 2024-01-01T00:00:00Z)",
                    since_str
                )));
            }
        }

        if let Some(until_str) = input.until {
            if let Some(until) = Self::parse_datetime(&until_str) {
                query = query.with_until(until);
            } else {
                return Ok(ToolResult::err(format!(
                    "Invalid until date format: {}. Use ISO 8601 format (e.g., 2024-01-01T00:00:00Z)",
                    until_str
                )));
            }
        }

        if let Some(limit) = input.limit {
            query = query.with_limit(limit);
        }

        let store_guard = self
            .store
            .lock()
            .map_err(|_| OpenCodeError::Tool("Failed to acquire lock on log store".to_string()))?;
        let store = store_guard
            .as_ref()
            .ok_or_else(|| OpenCodeError::Tool("Log store not initialized".to_string()))?;

        match store.query(&query) {
            Ok(events) => {
                let count = events.len();
                let result = serde_json::json!({
                    "events": events,
                    "count": count
                });
                Ok(ToolResult::ok(
                    serde_json::to_string_pretty(&result)
                        .unwrap_or_else(|_| r#"{"events":[],"count":0}"#.to_string()),
                ))
            }
            Err(e) => Ok(ToolResult::err(format!("Query failed: {}", e))),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use opencode_logging::event::LogEvent;
    use tempfile::TempDir;

    fn create_test_store() -> (TempDir, LogStore) {
        let temp_dir = TempDir::new().unwrap();
        let db_path = temp_dir.path().join("test.db");
        let store = LogStore::new(&db_path).unwrap();
        (temp_dir, store)
    }

    #[tokio::test]
    async fn test_log_query_tool_name_and_description() {
        let tool = LogQueryTool::new();
        assert_eq!(tool.name(), "log_query");
        assert!(tool.description().contains("Query log events"));
    }

    #[tokio::test]
    async fn test_log_query_tool_execute_with_empty_results() {
        let (_temp_dir, store) = create_test_store();
        let tool = LogQueryTool::with_store(store);

        let result = tool.execute(serde_json::json!({}), None).await.unwrap();

        assert!(result.success);
        let parsed: serde_json::Value = serde_json::from_str(&result.content).expect("Valid JSON");
        assert!(parsed["events"].is_array());
        assert_eq!(parsed["count"], 0);
    }

    #[tokio::test]
    async fn test_log_query_tool_execute_with_target_filter() {
        let (_temp_dir, store) = create_test_store();

        store
            .append(&LogEvent::new(
                1,
                LogLevel::Info,
                "llm.openai",
                "OpenAI response",
            ))
            .unwrap();
        store
            .append(&LogEvent::new(
                2,
                LogLevel::Info,
                "llm.anthropic",
                "Anthropic response",
            ))
            .unwrap();
        store
            .append(&LogEvent::new(3, LogLevel::Info, "tool.read", "Read file"))
            .unwrap();

        let tool = LogQueryTool::with_store(store);

        let result = tool
            .execute(
                serde_json::json!({
                    "target": "llm.*"
                }),
                None,
            )
            .await
            .unwrap();

        assert!(result.success);
        let parsed: serde_json::Value = serde_json::from_str(&result.content).expect("Valid JSON");
        assert_eq!(parsed["count"], 2);
    }

    #[tokio::test]
    async fn test_log_query_tool_execute_with_limit() {
        let (_temp_dir, store) = create_test_store();

        for i in 1..=10 {
            store
                .append(&LogEvent::new(
                    i,
                    LogLevel::Info,
                    "agent",
                    format!("Message {}", i),
                ))
                .unwrap();
        }

        let tool = LogQueryTool::with_store(store);

        let result = tool
            .execute(
                serde_json::json!({
                    "limit": 3
                }),
                None,
            )
            .await
            .unwrap();

        assert!(result.success);
        let parsed: serde_json::Value = serde_json::from_str(&result.content).expect("Valid JSON");
        assert_eq!(parsed["count"], 3);
    }

    #[tokio::test]
    async fn test_log_query_tool_iso_8601_parsing() {
        let (_temp_dir, store) = create_test_store();

        store
            .append(
                &LogEvent::new(1, LogLevel::Info, "agent", "Before cutoff")
                    .with_session_id("sess_time"),
            )
            .unwrap();

        let tool = LogQueryTool::with_store(store);

        let result = tool
            .execute(
                serde_json::json!({
                    "session_id": "sess_time",
                    "since": "2020-01-01T00:00:00Z"
                }),
                None,
            )
            .await
            .unwrap();

        assert!(result.success);
        let parsed: serde_json::Value = serde_json::from_str(&result.content).expect("Valid JSON");
        assert_eq!(parsed["count"], 1);
    }

    #[tokio::test]
    async fn test_log_query_tool_invalid_level() {
        let (_temp_dir, store) = create_test_store();
        let tool = LogQueryTool::with_store(store);

        let result = tool
            .execute(
                serde_json::json!({
                    "level": "invalid_level"
                }),
                None,
            )
            .await
            .unwrap();

        assert!(!result.success);
        assert!(result.error.unwrap().contains("Invalid log level"));
    }

    #[tokio::test]
    async fn test_log_query_tool_invalid_since_date() {
        let (_temp_dir, store) = create_test_store();
        let tool = LogQueryTool::with_store(store);

        let result = tool
            .execute(
                serde_json::json!({
                    "since": "not-a-date"
                }),
                None,
            )
            .await
            .unwrap();

        assert!(!result.success);
        assert!(result.error.unwrap().contains("Invalid since date format"));
    }

    #[tokio::test]
    async fn test_log_query_tool_invalid_until_date() {
        let (_temp_dir, store) = create_test_store();
        let tool = LogQueryTool::with_store(store);

        let result = tool
            .execute(
                serde_json::json!({
                    "until": "not-a-date"
                }),
                None,
            )
            .await
            .unwrap();

        assert!(!result.success);
        assert!(result.error.unwrap().contains("Invalid until date format"));
    }

    #[tokio::test]
    async fn test_log_query_tool_multiple_filters() {
        let (_temp_dir, store) = create_test_store();

        store
            .append(
                &LogEvent::new(1, LogLevel::Error, "tool.read", "Error in read")
                    .with_session_id("sess_multi"),
            )
            .unwrap();
        store
            .append(
                &LogEvent::new(2, LogLevel::Info, "tool.read", "Info in read")
                    .with_session_id("sess_multi"),
            )
            .unwrap();
        store
            .append(
                &LogEvent::new(3, LogLevel::Error, "tool.write", "Error in write")
                    .with_session_id("sess_multi"),
            )
            .unwrap();

        let tool = LogQueryTool::with_store(store);

        let result = tool
            .execute(
                serde_json::json!({
                    "session_id": "sess_multi",
                    "level": "error"
                }),
                None,
            )
            .await
            .unwrap();

        assert!(result.success);
        let parsed: serde_json::Value = serde_json::from_str(&result.content).expect("Valid JSON");
        assert_eq!(parsed["count"], 2);
    }

    #[tokio::test]
    async fn test_log_query_tool_returns_structured_json() {
        let (_temp_dir, store) = create_test_store();

        store
            .append(
                &LogEvent::new(1, LogLevel::Info, "test.target", "Test message")
                    .with_session_id("sess_struct"),
            )
            .unwrap();

        let tool = LogQueryTool::with_store(store);

        let result = tool
            .execute(serde_json::json!({"session_id": "sess_struct"}), None)
            .await
            .unwrap();

        assert!(result.success);

        let parsed: serde_json::Value = serde_json::from_str(&result.content).expect("Valid JSON");

        assert!(parsed.is_object());
        assert!(parsed["events"].is_array());
        assert!(parsed["count"].is_number());

        let events = parsed["events"].as_array().unwrap();
        assert_eq!(events.len(), 1);

        let event = &events[0];
        assert!(event["seq"].is_number());
        assert!(event["timestamp"].is_string());
        assert!(event["level"].is_string());
        assert!(event["target"].is_string());
        assert!(event["message"].is_string());
    }

    #[test]
    fn test_parse_level() {
        assert_eq!(LogQueryTool::parse_level("trace"), Some(LogLevel::Trace));
        assert_eq!(LogQueryTool::parse_level("debug"), Some(LogLevel::Debug));
        assert_eq!(LogQueryTool::parse_level("info"), Some(LogLevel::Info));
        assert_eq!(LogQueryTool::parse_level("warn"), Some(LogLevel::Warn));
        assert_eq!(LogQueryTool::parse_level("warning"), Some(LogLevel::Warn));
        assert_eq!(LogQueryTool::parse_level("error"), Some(LogLevel::Error));
        assert_eq!(LogQueryTool::parse_level("ERROR"), Some(LogLevel::Error));
        assert_eq!(LogQueryTool::parse_level("INFO"), Some(LogLevel::Info));
        assert_eq!(LogQueryTool::parse_level("invalid"), None);
    }

    #[test]
    fn test_parse_datetime() {
        assert!(LogQueryTool::parse_datetime("2024-01-01T00:00:00Z").is_some());
        assert!(LogQueryTool::parse_datetime("2024-12-31T23:59:59Z").is_some());
        assert!(LogQueryTool::parse_datetime("2024-01-01T00:00:00+00:00").is_some());
        assert!(LogQueryTool::parse_datetime("2024-01-01 00:00:00").is_some());
        assert!(LogQueryTool::parse_datetime("invalid").is_none());
    }
}
