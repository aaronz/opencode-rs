use opencode_logging::config::LoggingConfig;
use opencode_logging::event::{LogEvent, LogLevel};
use opencode_logging::log_llm;
use opencode_logging::log_tool;
use opencode_logging::logger::Logger;
use opencode_logging::query::LogQuery;
use opencode_logging::AgentLogger;

#[tokio::test]
async fn test_log_tool_macro_expands_to_info_call() {
    let config = LoggingConfig::default();
    let logger = Logger::new(config).unwrap();

    log_tool!(logger, "read", "success", latency_ms = 45);
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
        latency_ms = 100,
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

    log_tool!(logger, "bash", "success", latency_ms = 50);
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

#[tokio::test]
async fn test_log_llm_macro_expands_to_info_call() {
    let config = LoggingConfig::default();
    let logger = Logger::new(config).unwrap();

    log_llm!(logger, "openai", "gpt-4", 1800, 250, "success");
    tokio::task::yield_now().await;

    let events = logger.query_logs(LogQuery::new()).await.unwrap();
    assert_eq!(events.len(), 1);

    let event = &events[0];
    assert_eq!(event.level, LogLevel::Info);
}

#[tokio::test]
async fn test_log_llm_macro_target_format() {
    let config = LoggingConfig::default();
    let logger = Logger::new(config).unwrap();

    log_llm!(logger, "openai", "gpt-4", 1800, 250, "success");
    tokio::task::yield_now().await;

    let events = logger.query_logs(LogQuery::new()).await.unwrap();
    assert_eq!(events.len(), 1);

    let event = &events[0];
    assert_eq!(event.target, "llm.openai");
}

#[tokio::test]
async fn test_log_llm_macro_all_fields_populated() {
    let config = LoggingConfig::default();
    let logger = Logger::new(config).unwrap();

    log_llm!(logger, "anthropic", "claude-3", 2500, 300, "success");
    tokio::task::yield_now().await;

    let events = logger.query_logs(LogQuery::new()).await.unwrap();
    assert_eq!(events.len(), 1);

    let event = &events[0];
    assert_eq!(event.fields.provider, Some("anthropic".to_string()));
    assert_eq!(event.fields.model, Some("claude-3".to_string()));
    assert_eq!(event.fields.token_count, Some(2500));
    assert_eq!(event.fields.latency_ms, Some(300));
}

#[tokio::test]
async fn test_log_llm_macro_message_format() {
    let config = LoggingConfig::default();
    let logger = Logger::new(config).unwrap();

    log_llm!(logger, "openai", "gpt-4", 1800, 250, "success");
    tokio::task::yield_now().await;

    let events = logger.query_logs(LogQuery::new()).await.unwrap();
    assert_eq!(events.len(), 1);

    let event = &events[0];
    assert_eq!(event.message, "LLM request completed: success");
}

#[tokio::test]
async fn test_log_llm_macro_with_various_providers() {
    let config = LoggingConfig::default();
    let logger = Logger::new(config).unwrap();

    log_llm!(logger, "openai", "gpt-4", 1000, 100, "success");
    log_llm!(logger, "anthropic", "claude-3-sonnet", 2000, 200, "partial");
    log_llm!(logger, "ollama", "llama3", 500, 50, "error");
    tokio::task::yield_now().await;

    let events = logger.query_logs(LogQuery::new()).await.unwrap();
    assert_eq!(events.len(), 3);

    let targets: Vec<&str> = events.iter().map(|e| e.target.as_str()).collect();
    assert!(targets.contains(&"llm.openai"));
    assert!(targets.contains(&"llm.anthropic"));
    assert!(targets.contains(&"llm.ollama"));
}
