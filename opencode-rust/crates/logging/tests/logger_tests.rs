use opencode_logging::config::LoggingConfig;
use opencode_logging::event::{LogEvent, LogLevel};
use opencode_logging::log_tool;
use opencode_logging::logger::Logger;
use opencode_logging::query::LogQuery;
use opencode_logging::AgentLogger;

#[tokio::test]
async fn test_log_tool_macro_expands_to_info_call() {
    let config = LoggingConfig::default();
    let logger = Logger::new(config).unwrap();

    log_tool!(logger, "read", "success", latency_ms = 45i64);
    tokio::task::yield_now().await;

    let events = logger.query_logs(LogQuery::new()).await.unwrap();
    assert_eq!(events.len(), 1);

    let event = &events[0];
    assert_eq!(event.target, "tool.read");
    assert_eq!(event.message, "Tool success completed");
    assert_eq!(event.fields.latency_ms, Some(45));
}

#[tokio::test]
async fn test_log_tool_macro_field_syntax() {
    let config = LoggingConfig::default();
    let logger = Logger::new(config).unwrap();

    log_tool!(
        logger,
        "write",
        "failed",
        latency_ms = 100i64,
        error_code = "ERR_WRITE"
    );
    tokio::task::yield_now().await;

    let events = logger.query_logs(LogQuery::new()).await.unwrap();
    assert_eq!(events.len(), 1);

    let event = &events[0];
    assert_eq!(event.target, "tool.write");
    assert_eq!(event.message, "Tool failed completed");
    assert_eq!(event.fields.latency_ms, Some(100));
    assert_eq!(event.fields.error_code, Some("ERR_WRITE".to_string()));
}

#[tokio::test]
async fn test_log_tool_macro_target_format() {
    let config = LoggingConfig::default();
    let logger = Logger::new(config).unwrap();

    log_tool!(logger, "bash", "success", latency_ms = 50i64);
    tokio::task::yield_now().await;

    let events = logger.query_logs(LogQuery::new()).await.unwrap();
    assert_eq!(events.len(), 1);

    let event = &events[0];
    assert_eq!(event.target, "tool.bash");
}

#[tokio::test]
async fn test_log_tool_macro_message_format() {
    let config = LoggingConfig::default();
    let logger = Logger::new(config).unwrap();

    log_tool!(logger, "read", "completed",);
    tokio::task::yield_now().await;

    let events = logger.query_logs(LogQuery::new()).await.unwrap();
    assert_eq!(events.len(), 1);

    let event = &events[0];
    assert_eq!(event.message, "Tool completed completed");
}

#[tokio::test]
async fn test_log_event_creation() {
    let event = LogEvent::new(1, LogLevel::Info, "test.target", "Test message");
    assert_eq!(event.seq, 1);
    assert_eq!(event.level, LogLevel::Info);
    assert_eq!(event.target, "test.target");
    assert_eq!(event.message, "Test message");
}

#[tokio::test]
async fn test_log_event_builder() {
    let event = LogEvent::new(1, LogLevel::Info, "tool.read", "File read")
        .with_session_id("sess_123")
        .with_tool_name("read")
        .with_latency_ms(42);

    assert_eq!(event.fields.session_id, Some("sess_123".to_string()));
    assert_eq!(event.fields.tool_name, Some("read".to_string()));
    assert_eq!(event.fields.latency_ms, Some(42));
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
async fn test_log_query_matching() {
    let query = LogQuery::new()
        .with_session_id("sess_123")
        .with_level(LogLevel::Error);

    let matching_event =
        LogEvent::new(1, LogLevel::Error, "test", "error").with_session_id("sess_123");

    let non_matching_event =
        LogEvent::new(2, LogLevel::Info, "test", "info").with_session_id("sess_123");

    assert!(query.matches(&matching_event));
    assert!(!query.matches(&non_matching_event));
}
