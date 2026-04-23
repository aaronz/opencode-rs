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
    let event = LogEvent::new(1, LogLevel::Info, "test", "message").with_session_id("sess_123");
    assert!(query.matches(&event));

    let event2 = LogEvent::new(2, LogLevel::Info, "test", "message").with_session_id("sess_456");
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

    let matching =
        LogEvent::new(1, LogLevel::Error, "tool.read", "failed").with_session_id("sess_123");
    assert!(query.matches(&matching));

    let wrong_session =
        LogEvent::new(2, LogLevel::Error, "tool.read", "failed").with_session_id("sess_456");
    assert!(!query.matches(&wrong_session));

    let wrong_level =
        LogEvent::new(3, LogLevel::Info, "tool.read", "ok").with_session_id("sess_123");
    assert!(!query.matches(&wrong_level));

    let wrong_target =
        LogEvent::new(4, LogLevel::Error, "llm.openai", "error").with_session_id("sess_123");
    assert!(!query.matches(&wrong_target));
}

#[test]
fn test_matches_returns_true_when_all_criteria_match() {
    let query = LogQuery::new()
        .with_session_id("sess_abc")
        .with_level(LogLevel::Info)
        .with_target("agent");

    let event = LogEvent::new(1, LogLevel::Info, "agent", "message").with_session_id("sess_abc");

    assert!(query.matches(&event));
}

#[test]
fn test_matches_returns_false_when_any_criterion_fails() {
    let query = LogQuery::new()
        .with_session_id("sess_abc")
        .with_level(LogLevel::Error);

    let wrong_session =
        LogEvent::new(1, LogLevel::Error, "agent", "message").with_session_id("sess_xyz");
    assert!(!query.matches(&wrong_session));

    let wrong_level =
        LogEvent::new(2, LogLevel::Info, "agent", "message").with_session_id("sess_abc");
    assert!(!query.matches(&wrong_level));

    let both_wrong =
        LogEvent::new(3, LogLevel::Warn, "agent", "message").with_session_id("sess_xyz");
    assert!(!query.matches(&both_wrong));
}

#[test]
fn test_for_session_creates_query_for_specific_session() {
    let query = LogQuery::for_session("sess_xyz");

    assert_eq!(query.session_id, Some("sess_xyz".to_string()));

    let matching_event =
        LogEvent::new(1, LogLevel::Info, "test", "msg").with_session_id("sess_xyz");
    assert!(query.matches(&matching_event));

    let non_matching_event =
        LogEvent::new(2, LogLevel::Info, "test", "msg").with_session_id("sess_abc");
    assert!(!query.matches(&non_matching_event));
}

#[test]
fn test_for_level_creates_query_for_specific_level() {
    let query = LogQuery::for_level(LogLevel::Debug);

    assert_eq!(query.level, Some(LogLevel::Debug));

    let debug_event = LogEvent::new(1, LogLevel::Debug, "test", "msg");
    assert!(query.matches(&debug_event));

    let info_event = LogEvent::new(2, LogLevel::Info, "test", "msg");
    assert!(!query.matches(&info_event));
}

#[test]
fn test_empty_query_matches_all_events() {
    let query = LogQuery::new();

    let any_event = LogEvent::new(1, LogLevel::Trace, "any.target", "any message");
    assert!(query.matches(&any_event));

    let another_event = LogEvent::new(2, LogLevel::Error, "other.target", "other message")
        .with_session_id("sess_123");
    assert!(query.matches(&another_event));
}

#[test]
fn test_query_by_session_id_returns_only_that_session_logs() {
    let query = LogQuery::new().with_session_id("sess_abc");

    let matching_event =
        LogEvent::new(1, LogLevel::Info, "test", "msg").with_session_id("sess_abc");
    let non_matching_event =
        LogEvent::new(2, LogLevel::Info, "test", "msg").with_session_id("sess_xyz");

    assert!(query.matches(&matching_event));
    assert!(!query.matches(&non_matching_event));
}

#[test]
fn test_query_by_level_returns_only_matching_severity() {
    let query = LogQuery::new().with_level(LogLevel::Warn);

    let warn_event = LogEvent::new(1, LogLevel::Warn, "test", "warning");
    let info_event = LogEvent::new(2, LogLevel::Info, "test", "info");
    let error_event = LogEvent::new(3, LogLevel::Error, "test", "error");

    assert!(query.matches(&warn_event));
    assert!(!query.matches(&info_event));
    assert!(!query.matches(&error_event));
}

#[test]
fn test_query_by_target_returns_matching_components() {
    let query = LogQuery::new().with_target("tool.*");

    let read_event = LogEvent::new(1, LogLevel::Info, "tool.read", "read");
    let write_event = LogEvent::new(2, LogLevel::Info, "tool.write", "write");
    let agent_event = LogEvent::new(3, LogLevel::Info, "agent", "agent");

    assert!(query.matches(&read_event));
    assert!(query.matches(&write_event));
    assert!(!query.matches(&agent_event));
}

#[test]
fn test_query_by_time_range_filters_correctly() {
    let now = chrono::Utc::now();
    let query = LogQuery::new()
        .with_since(now - chrono::Duration::hours(1))
        .with_until(now + chrono::Duration::hours(1));

    let event = LogEvent::new(1, LogLevel::Info, "test", "message");
    assert!(query.matches(&event));

    let old_event = LogEvent::new(2, LogLevel::Info, "test", "old")
        .with_timestamp(now - chrono::Duration::hours(2));
    assert!(!query.matches(&old_event));

    let future_event = LogEvent::new(3, LogLevel::Info, "test", "future")
        .with_timestamp(now + chrono::Duration::hours(2));
    assert!(!query.matches(&future_event));
}

#[test]
fn test_query_combined_queries_match_all_criteria() {
    let query = LogQuery::new()
        .with_session_id("sess_123")
        .with_level(LogLevel::Error)
        .with_target("tool.*");

    let fully_matching =
        LogEvent::new(1, LogLevel::Error, "tool.read", "error").with_session_id("sess_123");
    assert!(query.matches(&fully_matching));

    let wrong_session =
        LogEvent::new(2, LogLevel::Error, "tool.read", "error").with_session_id("sess_other");
    assert!(!query.matches(&wrong_session));

    let wrong_level =
        LogEvent::new(3, LogLevel::Info, "tool.read", "ok").with_session_id("sess_123");
    assert!(!query.matches(&wrong_level));

    let wrong_target =
        LogEvent::new(4, LogLevel::Error, "llm.openai", "error").with_session_id("sess_123");
    assert!(!query.matches(&wrong_target));
}

#[test]
fn test_query_limit_parameter_caps_result_count() {
    let query = LogQuery::new().with_limit(10);
    assert_eq!(query.limit, Some(10));
}

#[test]
fn test_query_since_equals_until() {
    let now = chrono::Utc::now();
    let query = LogQuery::new().with_since(now).with_until(now);

    let event_at_exact_time =
        LogEvent::new(1, LogLevel::Info, "test", "message").with_timestamp(now);
    assert!(query.matches(&event_at_exact_time));

    let event_before = LogEvent::new(2, LogLevel::Info, "test", "before")
        .with_timestamp(now - chrono::Duration::seconds(1));
    assert!(!query.matches(&event_before));

    let event_after = LogEvent::new(3, LogLevel::Info, "test", "after")
        .with_timestamp(now + chrono::Duration::seconds(1));
    assert!(!query.matches(&event_after));
}

#[test]
fn test_query_limit_zero() {
    let query = LogQuery::new().with_limit(0);
    assert_eq!(query.limit, Some(0));
}

#[test]
fn test_query_limit_greater_than_results() {
    let query = LogQuery::new().with_limit(100);
    assert_eq!(query.limit, Some(100));
}
