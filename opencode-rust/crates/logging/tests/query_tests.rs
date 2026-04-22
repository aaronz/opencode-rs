use opencode_logging::event::{LogEvent, LogLevel};
use opencode_logging::query::LogQuery;

#[test]
fn test_query_default() {
    let query = LogQuery::new();
    let event = LogEvent::new(1, LogLevel::Info, "test", "message");
    assert!(query.matches(&event));
}

#[test]
fn test_query_by_session() {
    let query = LogQuery::new().with_session_id("sess_123");
    let event = LogEvent::new(1, LogLevel::Info, "test", "message")
        .with_session_id("sess_123");
    assert!(query.matches(&event));

    let event2 = LogEvent::new(2, LogLevel::Info, "test", "message")
        .with_session_id("sess_456");
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

#[test]
fn test_query_combined_filters() {
    let query = LogQuery::new()
        .with_session_id("sess_123")
        .with_level(LogLevel::Error)
        .with_target("tool.*");

    let matching = LogEvent::new(1, LogLevel::Error, "tool.read", "failed")
        .with_session_id("sess_123");
    assert!(query.matches(&matching));

    let wrong_session = LogEvent::new(2, LogLevel::Error, "tool.read", "failed")
        .with_session_id("sess_456");
    assert!(!query.matches(&wrong_session));

    let wrong_level = LogEvent::new(3, LogLevel::Info, "tool.read", "ok")
        .with_session_id("sess_123");
    assert!(!query.matches(&wrong_level));

    let wrong_target = LogEvent::new(4, LogLevel::Error, "llm.openai", "error")
        .with_session_id("sess_123");
    assert!(!query.matches(&wrong_target));
}