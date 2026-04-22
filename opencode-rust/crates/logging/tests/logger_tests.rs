use opencode_logging::config::LoggingConfig;
use opencode_logging::event::{LogEvent, LogFields, LogLevel};
use opencode_logging::log_llm;
use opencode_logging::log_tool;
use opencode_logging::logger::Logger;
use opencode_logging::query::LogQuery;
use opencode_logging::AgentLogger;

#[tokio::test]
async fn test_log_event_round_trip_serialization() {
    let event = LogEvent::new(1, LogLevel::Info, "test.target", "Test message")
        .with_session_id("sess_123")
        .with_span_id("trace_abc:span_42")
        .with_parent_seq(0)
        .with_tool_name("test_tool")
        .with_latency_ms(100);

    let json = serde_json::to_string(&event).unwrap();
    let deserialized: LogEvent = serde_json::from_str(&json).unwrap();

    assert_eq!(deserialized.seq, event.seq);
    assert_eq!(deserialized.level, event.level);
    assert_eq!(deserialized.target, event.target);
    assert_eq!(deserialized.message, event.message);
    assert_eq!(deserialized.fields.session_id, event.fields.session_id);
    assert_eq!(deserialized.span_id, event.span_id);
    assert_eq!(deserialized.parent_seq, event.parent_seq);
    assert_eq!(deserialized.fields.tool_name, event.fields.tool_name);
    assert_eq!(deserialized.fields.latency_ms, event.fields.latency_ms);
}

#[tokio::test]
async fn test_level_filtering_excludes_non_matching_levels() {
    let mut config = LoggingConfig::default();
    config.level = LogLevel::Debug;
    let logger = Logger::new(config).unwrap();

    logger.info("test", "info message", LogFields::default());
    logger.debug("test", "debug message", LogFields::default());
    logger.warn("test", "warn message", LogFields::default());
    logger.error("test", "error message", LogFields::default());
    tokio::task::yield_now().await;

    let all_events = logger.query_logs(LogQuery::new()).await.unwrap();
    assert_eq!(all_events.len(), 4);

    let error_query = LogQuery::new().with_level(LogLevel::Error);
    let error_events = logger.query_logs(error_query).await.unwrap();
    assert_eq!(error_events.len(), 1);
    assert_eq!(error_events[0].level, LogLevel::Error);

    let warn_query = LogQuery::new().with_level(LogLevel::Warn);
    let warn_events = logger.query_logs(warn_query).await.unwrap();
    assert_eq!(warn_events.len(), 1);
    assert_eq!(warn_events[0].level, LogLevel::Warn);
}

#[tokio::test]
async fn test_query_matching_all_field_combinations() {
    let event = LogEvent::new(1, LogLevel::Error, "agent.test", "Test message")
        .with_session_id("sess_abc")
        .with_tool_name("read")
        .with_latency_ms(42);

    let session_query = LogQuery::new().with_session_id("sess_abc");
    assert!(session_query.matches(&event));

    let session_mismatch = LogQuery::new().with_session_id("sess_xyz");
    assert!(!session_mismatch.matches(&event));

    let level_query = LogQuery::new().with_level(LogLevel::Error);
    assert!(level_query.matches(&event));

    let level_mismatch = LogQuery::new().with_level(LogLevel::Info);
    assert!(!level_mismatch.matches(&event));

    let target_query = LogQuery::new().with_target("agent.*");
    assert!(target_query.matches(&event));

    let target_mismatch = LogQuery::new().with_target("llm.*");
    assert!(!target_mismatch.matches(&event));

    let combined_query = LogQuery::new()
        .with_session_id("sess_abc")
        .with_level(LogLevel::Error)
        .with_target("agent.*");
    assert!(combined_query.matches(&event));

    let combined_mismatch = LogQuery::new()
        .with_session_id("sess_abc")
        .with_level(LogLevel::Error)
        .with_target("llm.*");
    assert!(!combined_mismatch.matches(&event));
}

#[tokio::test]
async fn test_child_logger_inherits_and_extends_parent_context() {
    let config = LoggingConfig::default();
    let logger = Logger::new(config).unwrap();

    let parent_context = LogFields::with_session_id("parent_session");
    let child_logger = logger.with_context(parent_context);

    child_logger.info("test", "child message", LogFields::default());
    tokio::task::yield_now().await;

    let events = logger.query_logs(LogQuery::new()).await.unwrap();
    assert_eq!(events.len(), 1);
    assert_eq!(
        events[0].fields.session_id,
        Some("parent_session".to_string())
    );

    let grandchild_context = LogFields::default().with_tool_name("grandchild_tool");
    let grandchild_logger = child_logger.with_context(grandchild_context);

    grandchild_logger.info("test", "grandchild message", LogFields::default());
    tokio::task::yield_now().await;

    let events = logger.query_logs(LogQuery::new()).await.unwrap();
    assert_eq!(events.len(), 2);

    let parent_event = &events[0];
    assert_eq!(
        parent_event.fields.session_id,
        Some("parent_session".to_string())
    );

    let grandchild_event = &events[1];
    assert_eq!(
        grandchild_event.fields.session_id,
        Some("parent_session".to_string())
    );
    assert_eq!(
        grandchild_event.fields.tool_name,
        Some("grandchild_tool".to_string())
    );
}

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

#[tokio::test]
async fn test_per_component_filter_exact_match_agent() {
    let mut config = LoggingConfig::default();
    config.level = LogLevel::Info;
    config.targets.insert("agent".to_string(), LogLevel::Debug);
    let logger = Logger::new(config).unwrap();

    assert!(logger.should_log("agent", LogLevel::Debug));
    assert!(logger.should_log("agent", LogLevel::Info));
    assert!(!logger.should_log("agent", LogLevel::Trace));
}

#[tokio::test]
async fn test_per_component_filter_prefix_wildcard() {
    let mut config = LoggingConfig::default();
    config.level = LogLevel::Info;
    config.targets.insert("llm.*".to_string(), LogLevel::Debug);
    let logger = Logger::new(config).unwrap();

    assert!(logger.should_log("llm.openai", LogLevel::Debug));
    assert!(logger.should_log("llm.openai", LogLevel::Info));
    assert!(!logger.should_log("llm.openai", LogLevel::Trace));
    assert!(logger.should_log("llm.anthropic", LogLevel::Debug));
    assert!(!logger.should_log("llm.anthropic", LogLevel::Trace));
}

#[tokio::test]
async fn test_per_component_filter_suffix_wildcard() {
    let mut config = LoggingConfig::default();
    config.level = LogLevel::Info;
    config.targets.insert("*.read".to_string(), LogLevel::Debug);
    let logger = Logger::new(config).unwrap();

    assert!(logger.should_log("tool.read", LogLevel::Debug));
    assert!(logger.should_log("tool.read", LogLevel::Info));
    assert!(!logger.should_log("tool.read", LogLevel::Trace));
    assert!(logger.should_log("file.read", LogLevel::Debug));
    assert!(!logger.should_log("file.read", LogLevel::Trace));
}

#[tokio::test]
async fn test_per_component_filter_priority_resolution_order() {
    let mut config = LoggingConfig::default();
    config.level = LogLevel::Info;
    config.targets.insert("agent".to_string(), LogLevel::Error);
    config.targets.insert("llm.*".to_string(), LogLevel::Debug);
    config.targets.insert("*.read".to_string(), LogLevel::Trace);
    let logger = Logger::new(config).unwrap();

    assert!(logger.should_log("agent", LogLevel::Error));
    assert!(!logger.should_log("agent", LogLevel::Warn));
    assert!(!logger.should_log("agent", LogLevel::Info));
    assert!(!logger.should_log("agent", LogLevel::Debug));
    assert!(!logger.should_log("agent", LogLevel::Trace));

    assert!(logger.should_log("llm.openai", LogLevel::Debug));
    assert!(logger.should_log("llm.openai", LogLevel::Info));
    assert!(!logger.should_log("llm.openai", LogLevel::Trace));

    assert!(logger.should_log("tool.read", LogLevel::Trace));
    assert!(logger.should_log("tool.read", LogLevel::Debug));
    assert!(logger.should_log("tool.read", LogLevel::Info));
}

#[tokio::test]
async fn test_per_component_filter_fallback_to_global() {
    let mut config = LoggingConfig::default();
    config.level = LogLevel::Warn;
    let logger = Logger::new(config).unwrap();

    assert!(logger.should_log("unknown.target", LogLevel::Warn));
    assert!(logger.should_log("unknown.target", LogLevel::Error));
    assert!(!logger.should_log("unknown.target", LogLevel::Info));
    assert!(!logger.should_log("unknown.target", LogLevel::Debug));
    assert!(!logger.should_log("unknown.target", LogLevel::Trace));
}
