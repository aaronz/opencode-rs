//! Log event structures for the OpenCode logging system.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::atomic::{AtomicU64, Ordering};
use tracing::Level as TracingLevel;

/// Global sequence number generator for LogEvent instances.
/// Sequences start at 1 and increment monotonically.
static SEQ_GENERATOR: AtomicU64 = AtomicU64::new(1);

/// Generate the next unique sequence number.
/// Sequences start at 1 and are guaranteed to be unique and incrementing.
pub fn next_seq() -> u64 {
    SEQ_GENERATOR.fetch_add(1, Ordering::Relaxed)
}

/// Log severity level
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum LogLevel {
    #[default]
    Trace,
    Debug,
    Info,
    Warn,
    Error,
}

impl LogLevel {
    /// Returns the ordinal value of the log level (0-based, matching severity)
    /// Trace=0, Debug=1, Info=2, Warn=3, Error=4
    pub fn ordinal(&self) -> u8 {
        match self {
            LogLevel::Trace => 0,
            LogLevel::Debug => 1,
            LogLevel::Info => 2,
            LogLevel::Warn => 3,
            LogLevel::Error => 4,
        }
    }
}

impl From<LogLevel> for TracingLevel {
    fn from(level: LogLevel) -> Self {
        match level {
            LogLevel::Trace => TracingLevel::TRACE,
            LogLevel::Debug => TracingLevel::DEBUG,
            LogLevel::Info => TracingLevel::INFO,
            LogLevel::Warn => TracingLevel::WARN,
            LogLevel::Error => TracingLevel::ERROR,
        }
    }
}

impl std::fmt::Display for LogLevel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            LogLevel::Trace => write!(f, "TRACE"),
            LogLevel::Debug => write!(f, "DEBUG"),
            LogLevel::Info => write!(f, "INFO"),
            LogLevel::Warn => write!(f, "WARN"),
            LogLevel::Error => write!(f, "ERROR"),
        }
    }
}

/// Structured fields for log events
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct LogFields {
    /// Session identifier
    pub session_id: Option<String>,
    /// Tool name if applicable
    pub tool_name: Option<String>,
    /// Operation latency in milliseconds
    pub latency_ms: Option<i64>,
    /// LLM model name
    pub model: Option<String>,
    /// LLM provider name
    pub provider: Option<String>,
    /// Token count for LLM operations
    pub token_count: Option<i64>,
    /// Error code for errors
    pub error_code: Option<String>,
    /// File path for file operations
    pub file_path: Option<String>,
    /// Line number for location context
    pub line: Option<u32>,
    /// Additional flattened fields
    #[serde(default)]
    pub extra: HashMap<String, serde_json::Value>,
}

impl LogFields {
    /// Create a new LogFields with only session_id set
    pub fn with_session_id(session_id: impl Into<String>) -> Self {
        Self {
            session_id: Some(session_id.into()),
            ..Default::default()
        }
    }

    /// Add a tool_name to the fields
    pub fn with_tool_name(mut self, tool_name: impl Into<String>) -> Self {
        self.tool_name = Some(tool_name.into());
        self
    }

    /// Add latency_ms to the fields
    pub fn with_latency_ms(mut self, latency_ms: i64) -> Self {
        self.latency_ms = Some(latency_ms);
        self
    }

    /// Add an extra field
    pub fn with_extra(mut self, key: impl Into<String>, value: serde_json::Value) -> Self {
        self.extra.insert(key.into(), value);
        self
    }
}

/// A single log event
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogEvent {
    /// Unique sequence number for ordering
    pub seq: u64,
    /// High-precision timestamp
    pub timestamp: DateTime<Utc>,
    /// Log severity level
    pub level: LogLevel,
    /// Target component (e.g., "agent", "tool.read", "llm.openai")
    pub target: String,
    /// Human-readable message
    pub message: String,
    /// Structured fields for querying
    pub fields: LogFields,
    /// Span context for trace correlation
    pub span_id: Option<String>,
    /// Parent log event ID for causality chains
    pub parent_seq: Option<u64>,
}

impl LogEvent {
    /// Create a new LogEvent
    pub fn new(
        seq: u64,
        level: LogLevel,
        target: impl Into<String>,
        message: impl Into<String>,
    ) -> Self {
        Self {
            seq,
            timestamp: Utc::now(),
            level,
            target: target.into(),
            message: message.into(),
            fields: LogFields::default(),
            span_id: None,
            parent_seq: None,
        }
    }

    /// Set the session_id for this event
    pub fn with_session_id(mut self, session_id: impl Into<String>) -> Self {
        self.fields.session_id = Some(session_id.into());
        self
    }

    /// Set the span_id for this event
    pub fn with_span_id(mut self, span_id: impl Into<String>) -> Self {
        self.span_id = Some(span_id.into());
        self
    }

    /// Set the parent_seq for this event
    pub fn with_parent_seq(mut self, parent_seq: u64) -> Self {
        self.parent_seq = Some(parent_seq);
        self
    }

    /// Set the tool_name for this event
    pub fn with_tool_name(mut self, tool_name: impl Into<String>) -> Self {
        self.fields.tool_name = Some(tool_name.into());
        self
    }

    /// Set the latency_ms for this event
    pub fn with_latency_ms(mut self, latency_ms: i64) -> Self {
        self.fields.latency_ms = Some(latency_ms);
        self
    }

    /// Add an extra field to this event
    pub fn with_extra(mut self, key: impl Into<String>, value: serde_json::Value) -> Self {
        self.fields.extra.insert(key.into(), value);
        self
    }
}

/// Tool consideration during reasoning
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolConsideration {
    /// Tool name
    pub tool_name: String,
    /// Reason for considering this tool
    pub reason: String,
    /// Whether this tool was selected
    pub selected: bool,
}

/// Reasoning log for agent decision-making
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReasoningLog {
    /// Unique step identifier
    pub step_id: String,
    /// Session identifier
    pub session_id: String,
    /// Timestamp
    pub timestamp: DateTime<Utc>,
    /// Prompt sent to LLM
    pub prompt: String,
    /// Response received (sanitized)
    pub response: String,
    /// Tools considered and reasoning
    pub tools_considered: Vec<ToolConsideration>,
    /// Final decision and reasoning
    pub decision: String,
    /// Tokens used for prompt
    pub prompt_tokens: u64,
    /// Tokens used for completion
    pub completion_tokens: u64,
    /// Total latency
    pub latency_ms: u64,
}

/// Tool result wrapper
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolResult {
    /// Whether the tool succeeded
    pub success: bool,
    /// Result message or error
    pub message: String,
    /// Output data if any
    pub output: Option<serde_json::Value>,
}

/// Error frame in a stack trace
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorFrame {
    /// File name
    pub file: String,
    /// Line number
    pub line: u32,
    /// Function name
    pub function: String,
}

/// Cause information in error chain
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CauseInfo {
    /// Error code
    pub code: String,
    /// Error message
    pub message: String,
}

/// Error context for error logging
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorContext {
    /// Error code for programmatic detection
    pub code: String,
    /// Human-readable message
    pub message: String,
    /// Full stack trace or error chain
    pub stack: Vec<ErrorFrame>,
    /// Causality chain (original → wrapping)
    pub cause_chain: Vec<CauseInfo>,
    /// Additional context
    #[serde(default)]
    pub context: HashMap<String, String>,
}

impl ErrorContext {
    /// Create a new ErrorContext with just code and message
    pub fn new(code: impl Into<String>, message: impl Into<String>) -> Self {
        Self {
            code: code.into(),
            message: message.into(),
            stack: Vec::new(),
            cause_chain: Vec::new(),
            context: HashMap::new(),
        }
    }

    /// Add a stack frame
    pub fn with_stack_frame(
        mut self,
        file: impl Into<String>,
        line: u32,
        function: impl Into<String>,
    ) -> Self {
        self.stack.push(ErrorFrame {
            file: file.into(),
            line,
            function: function.into(),
        });
        self
    }

    /// Add a cause to the chain
    pub fn with_cause(mut self, code: impl Into<String>, message: impl Into<String>) -> Self {
        self.cause_chain.push(CauseInfo {
            code: code.into(),
            message: message.into(),
        });
        self
    }

    /// Add context
    pub fn with_context(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.context.insert(key.into(), value.into());
        self
    }
}

/// Tool execution log
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolExecutionLog {
    /// Unique execution identifier
    pub execution_id: String,
    /// Session identifier
    pub session_id: String,
    /// Tool name
    pub tool_name: String,
    /// Execution timestamp
    pub timestamp: DateTime<Utc>,
    /// Tool parameters (sanitized)
    pub parameters: SanitizedValue,
    /// Execution result
    pub result: ToolResult,
    /// Execution latency
    pub latency_ms: u64,
    /// Error if any
    pub error: Option<ErrorContext>,
}

/// Sanitized value for secret redaction
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SanitizedValue {
    /// Safe string value
    Safe(String),
    /// Redacted value
    Redacted(String),
    /// Nested map with sanitized values
    Nested(HashMap<String, SanitizedValue>),
}

impl SanitizedValue {
    /// Create a safe value
    pub fn safe(value: impl Into<String>) -> Self {
        SanitizedValue::Safe(value.into())
    }

    /// Create a redacted value
    pub fn redacted(reason: impl Into<String>) -> Self {
        SanitizedValue::Redacted(reason.into())
    }

    /// Create a nested value
    pub fn nested(values: HashMap<String, SanitizedValue>) -> Self {
        SanitizedValue::Nested(values)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_log_level_ordinal() {
        assert_eq!(LogLevel::Trace.ordinal(), 0);
        assert_eq!(LogLevel::Debug.ordinal(), 1);
        assert_eq!(LogLevel::Info.ordinal(), 2);
        assert_eq!(LogLevel::Warn.ordinal(), 3);
        assert_eq!(LogLevel::Error.ordinal(), 4);
    }

    #[test]
    fn test_log_level_ordering() {
        assert!(LogLevel::Trace < LogLevel::Debug);
        assert!(LogLevel::Debug < LogLevel::Info);
        assert!(LogLevel::Info < LogLevel::Warn);
        assert!(LogLevel::Warn < LogLevel::Error);
    }

    #[test]
    fn test_log_level_to_tracing_level() {
        assert_eq!(tracing::Level::from(LogLevel::Trace), tracing::Level::TRACE);
        assert_eq!(tracing::Level::from(LogLevel::Debug), tracing::Level::DEBUG);
        assert_eq!(tracing::Level::from(LogLevel::Info), tracing::Level::INFO);
        assert_eq!(tracing::Level::from(LogLevel::Warn), tracing::Level::WARN);
        assert_eq!(tracing::Level::from(LogLevel::Error), tracing::Level::ERROR);
    }

    #[test]
    fn test_log_level_serialize_lowercase() {
        assert_eq!(
            serde_json::to_string(&LogLevel::Trace).unwrap(),
            "\"trace\""
        );
        assert_eq!(
            serde_json::to_string(&LogLevel::Debug).unwrap(),
            "\"debug\""
        );
        assert_eq!(serde_json::to_string(&LogLevel::Info).unwrap(), "\"info\"");
        assert_eq!(serde_json::to_string(&LogLevel::Warn).unwrap(), "\"warn\"");
        assert_eq!(
            serde_json::to_string(&LogLevel::Error).unwrap(),
            "\"error\""
        );
    }

    #[test]
    fn test_log_level_deserialize() {
        assert_eq!(
            serde_json::from_str::<LogLevel>("\"trace\"").unwrap(),
            LogLevel::Trace
        );
        assert_eq!(
            serde_json::from_str::<LogLevel>("\"debug\"").unwrap(),
            LogLevel::Debug
        );
        assert_eq!(
            serde_json::from_str::<LogLevel>("\"info\"").unwrap(),
            LogLevel::Info
        );
        assert_eq!(
            serde_json::from_str::<LogLevel>("\"warn\"").unwrap(),
            LogLevel::Warn
        );
        assert_eq!(
            serde_json::from_str::<LogLevel>("\"error\"").unwrap(),
            LogLevel::Error
        );
    }

    #[test]
    fn test_log_level_display() {
        assert_eq!(LogLevel::Trace.to_string(), "TRACE");
        assert_eq!(LogLevel::Debug.to_string(), "DEBUG");
        assert_eq!(LogLevel::Info.to_string(), "INFO");
        assert_eq!(LogLevel::Warn.to_string(), "WARN");
        assert_eq!(LogLevel::Error.to_string(), "ERROR");
    }

    #[test]
    fn test_log_event_creation() {
        let event = LogEvent::new(1, LogLevel::Info, "test.target", "Test message");
        assert_eq!(event.seq, 1);
        assert_eq!(event.level, LogLevel::Info);
        assert_eq!(event.target, "test.target");
        assert_eq!(event.message, "Test message");
    }

    #[test]
    fn test_log_event_builder() {
        let event = LogEvent::new(1, LogLevel::Info, "tool.read", "File read")
            .with_session_id("sess_123")
            .with_tool_name("read")
            .with_latency_ms(42);

        assert_eq!(event.fields.session_id, Some("sess_123".to_string()));
        assert_eq!(event.fields.tool_name, Some("read".to_string()));
        assert_eq!(event.fields.latency_ms, Some(42));
    }

    #[test]
    fn test_error_context_builder() {
        let error = ErrorContext::new("ERR_NOT_FOUND", "File not found")
            .with_stack_frame("main.rs", 42, "main")
            .with_cause("ERR_IO", "Failed to open file")
            .with_context("file_path", "/tmp/test.txt");

        assert_eq!(error.code, "ERR_NOT_FOUND");
        assert_eq!(error.message, "File not found");
        assert_eq!(error.stack.len(), 1);
        assert_eq!(error.stack[0].line, 42);
        assert_eq!(error.cause_chain.len(), 1);
        assert_eq!(
            error.context.get("file_path"),
            Some(&"/tmp/test.txt".to_string())
        );
    }

    #[test]
    fn test_sanitized_value() {
        let safe = SanitizedValue::safe("normal_value");
        let redacted = SanitizedValue::redacted("API key");
        let nested = SanitizedValue::nested(HashMap::from([(
            "password".to_string(),
            SanitizedValue::redacted("hidden"),
        )]));

        assert!(matches!(safe, SanitizedValue::Safe(_)));
        assert!(matches!(redacted, SanitizedValue::Redacted(_)));
        assert!(matches!(nested, SanitizedValue::Nested(_)));
    }

    #[test]
    fn test_log_fields_default_creates_empty_struct() {
        let fields = LogFields::default();

        // All Option fields should be None
        assert_eq!(fields.session_id, None);
        assert_eq!(fields.tool_name, None);
        assert_eq!(fields.latency_ms, None);
        assert_eq!(fields.model, None);
        assert_eq!(fields.provider, None);
        assert_eq!(fields.token_count, None);
        assert_eq!(fields.error_code, None);
        assert_eq!(fields.file_path, None);
        assert_eq!(fields.line, None);
        // extra should be an empty HashMap
        assert!(fields.extra.is_empty());
    }

    #[test]
    fn test_log_fields_serialization_includes_all_fields() {
        let mut fields = LogFields::default();
        fields.session_id = Some("sess_abc123".to_string());
        fields.tool_name = Some("read".to_string());
        fields.latency_ms = Some(42);
        fields.model = Some("gpt-4".to_string());
        fields.provider = Some("openai".to_string());
        fields.token_count = Some(1500);
        fields.error_code = Some("ERR_NOT_FOUND".to_string());
        fields.file_path = Some("/path/to/file.rs".to_string());
        fields.line = Some(100);
        fields
            .extra
            .insert("custom_key".to_string(), serde_json::json!("custom_value"));

        let json = serde_json::to_string(&fields).unwrap();

        // Verify all fields are present in serialized JSON
        assert!(json.contains("\"session_id\":\"sess_abc123\""));
        assert!(json.contains("\"tool_name\":\"read\""));
        assert!(json.contains("\"latency_ms\":42"));
        assert!(json.contains("\"model\":\"gpt-4\""));
        assert!(json.contains("\"provider\":\"openai\""));
        assert!(json.contains("\"token_count\":1500"));
        assert!(json.contains("\"error_code\":\"ERR_NOT_FOUND\""));
        assert!(json.contains("\"file_path\":\"/path/to/file.rs\""));
        assert!(json.contains("\"line\":100"));
        assert!(json.contains("\"custom_key\""));
        assert!(json.contains("\"custom_value\""));
    }

    #[test]
    fn test_log_fields_deserialization_populates_correct_fields() {
        let json = r#"{
            "session_id": "sess_xyz789",
            "tool_name": "write",
            "latency_ms": 100,
            "model": "claude-3",
            "provider": "anthropic",
            "token_count": 2000,
            "error_code": "ERR_PERMISSION",
            "file_path": "/src/main.rs",
            "line": 42,
            "extra": {"nested": {"key": "value"}, "number": 123}
        }"#;

        let fields: LogFields = serde_json::from_str(json).unwrap();

        assert_eq!(fields.session_id, Some("sess_xyz789".to_string()));
        assert_eq!(fields.tool_name, Some("write".to_string()));
        assert_eq!(fields.latency_ms, Some(100));
        assert_eq!(fields.model, Some("claude-3".to_string()));
        assert_eq!(fields.provider, Some("anthropic".to_string()));
        assert_eq!(fields.token_count, Some(2000));
        assert_eq!(fields.error_code, Some("ERR_PERMISSION".to_string()));
        assert_eq!(fields.file_path, Some("/src/main.rs".to_string()));
        assert_eq!(fields.line, Some(42));
        assert_eq!(
            fields
                .extra
                .get("nested")
                .unwrap()
                .as_object()
                .unwrap()
                .get("key")
                .unwrap()
                .as_str()
                .unwrap(),
            "value"
        );
        assert_eq!(fields.extra.get("number").unwrap().as_i64().unwrap(), 123);
    }

    #[test]
    fn test_log_fields_extra_hashmap_stores_arbitrary_json() {
        let mut fields = LogFields::default();

        // Store various JSON value types in extra
        fields
            .extra
            .insert("string_val".to_string(), serde_json::json!("hello"));
        fields
            .extra
            .insert("number_val".to_string(), serde_json::json!(42));
        fields
            .extra
            .insert("float_val".to_string(), serde_json::json!(3.14));
        fields
            .extra
            .insert("bool_val".to_string(), serde_json::json!(true));
        fields
            .extra
            .insert("null_val".to_string(), serde_json::json!(null));
        fields
            .extra
            .insert("array_val".to_string(), serde_json::json!([1, 2, 3]));
        fields.extra.insert(
            "object_val".to_string(),
            serde_json::json!({"key": "value"}),
        );

        assert_eq!(fields.extra.len(), 7);
        assert_eq!(
            fields.extra.get("string_val").unwrap().as_str().unwrap(),
            "hello"
        );
        assert_eq!(
            fields.extra.get("number_val").unwrap().as_i64().unwrap(),
            42
        );
        assert_eq!(
            fields.extra.get("float_val").unwrap().as_f64().unwrap(),
            3.14
        );
        assert_eq!(
            fields.extra.get("bool_val").unwrap().as_bool().unwrap(),
            true
        );
        assert!(fields.extra.get("null_val").unwrap().is_null());
        assert_eq!(
            fields
                .extra
                .get("array_val")
                .unwrap()
                .as_array()
                .unwrap()
                .len(),
            3
        );
        assert_eq!(
            fields
                .extra
                .get("object_val")
                .unwrap()
                .as_object()
                .unwrap()
                .get("key")
                .unwrap()
                .as_str()
                .unwrap(),
            "value"
        );

        // Verify serialization round-trip
        let json = serde_json::to_string(&fields).unwrap();
        let deserialized: LogFields = serde_json::from_str(&json).unwrap();
        assert_eq!(
            deserialized
                .extra
                .get("string_val")
                .unwrap()
                .as_str()
                .unwrap(),
            "hello"
        );
    }

    #[test]
    fn test_sequence_numbers_unique_and_incrementing() {
        let seq1 = next_seq();
        let seq2 = next_seq();
        let seq3 = next_seq();

        assert!(seq2 > seq1);
        assert!(seq3 > seq2);
        assert_eq!(seq2 - seq1, 1);
        assert_eq!(seq3 - seq2, 1);
    }

    #[test]
    fn test_timestamp_is_high_precision_utc() {
        let before = Utc::now();
        let event = LogEvent::new(1, LogLevel::Info, "test", "message");
        let after = Utc::now();

        assert!(event.timestamp >= before);
        assert!(event.timestamp <= after);
        assert_eq!(event.timestamp.timezone(), chrono::Utc);
    }

    #[test]
    fn test_log_event_new_constructor() {
        let event = LogEvent::new(42, LogLevel::Error, "my.target", "Error occurred");

        assert_eq!(event.seq, 42);
        assert_eq!(event.level, LogLevel::Error);
        assert_eq!(event.target, "my.target");
        assert_eq!(event.message, "Error occurred");
        assert!(event.span_id.is_none());
        assert!(event.parent_seq.is_none());
        assert_eq!(event.fields.session_id, None);
    }

    #[test]
    fn test_log_event_serialization_includes_all_fields() {
        let event = LogEvent::new(1, LogLevel::Info, "test", "message")
            .with_session_id("sess_123")
            .with_span_id("trace_abc:span_42")
            .with_parent_seq(0)
            .with_tool_name("test_tool")
            .with_latency_ms(100);

        let json = serde_json::to_string(&event).unwrap();

        assert!(json.contains(r#""seq":1"#));
        assert!(json.contains(r#""level":"info""#));
        assert!(json.contains(r#""target":"test""#));
        assert!(json.contains(r#""message":"message""#));
        assert!(json.contains(r#""session_id":"sess_123""#));
        assert!(json.contains(r#""span_id":"trace_abc:span_42""#));
        assert!(json.contains(r#""parent_seq":0"#));
        assert!(json.contains(r#""tool_name":"test_tool""#));
        assert!(json.contains(r#""latency_ms":100"#));
    }

    #[test]
    fn test_parent_seq_links_events_in_chains() {
        let event1 = LogEvent::new(1, LogLevel::Info, "test", "first");
        let event2 = LogEvent::new(2, LogLevel::Info, "test", "second").with_parent_seq(event1.seq);
        let event3 = LogEvent::new(3, LogLevel::Info, "test", "third").with_parent_seq(event2.seq);

        assert!(event1.parent_seq.is_none());
        assert_eq!(event2.parent_seq, Some(1));
        assert_eq!(event3.parent_seq, Some(2));
        assert_eq!(event3.parent_seq, Some(event2.seq));
    }

    #[test]
    fn test_span_id_format_trace_id_span_id() {
        let event =
            LogEvent::new(1, LogLevel::Debug, "test", "debug").with_span_id("abcd1234:span5678");

        assert_eq!(event.span_id, Some("abcd1234:span5678".to_string()));
        let json = serde_json::to_string(&event).unwrap();
        assert!(json.contains(r#""span_id":"abcd1234:span5678""#));
    }

    #[test]
    fn test_tool_consideration_serialization() {
        let tc = ToolConsideration {
            tool_name: "read".to_string(),
            reason: "Most appropriate for file operations".to_string(),
            selected: true,
        };

        let json = serde_json::to_string(&tc).unwrap();
        assert!(json.contains("\"tool_name\":\"read\""));
        assert!(json.contains("\"reason\":\"Most appropriate for file operations\""));
        assert!(json.contains("\"selected\":true"));

        let deserialized: ToolConsideration = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.tool_name, "read");
        assert_eq!(deserialized.reason, "Most appropriate for file operations");
        assert!(deserialized.selected);
    }

    #[test]
    fn test_reasoning_log_with_tools_considered_serializes() {
        let reasoning = ReasoningLog {
            step_id: "step_001".to_string(),
            session_id: "sess_abc123".to_string(),
            timestamp: Utc::now(),
            prompt: "What file should I read?".to_string(),
            response: "Read the config file".to_string(),
            tools_considered: vec![
                ToolConsideration {
                    tool_name: "read".to_string(),
                    reason: "Appropriate for reading files".to_string(),
                    selected: true,
                },
                ToolConsideration {
                    tool_name: "grep".to_string(),
                    reason: "Good for searching content".to_string(),
                    selected: false,
                },
            ],
            decision: "Selected read tool".to_string(),
            prompt_tokens: 1500,
            completion_tokens: 100,
            latency_ms: 250,
        };

        let json = serde_json::to_string(&reasoning).unwrap();
        assert!(json.contains("\"step_id\":\"step_001\""));
        assert!(json.contains("\"session_id\":\"sess_abc123\""));
        assert!(json.contains("\"prompt_tokens\":1500"));
        assert!(json.contains("\"completion_tokens\":100"));
        assert!(json.contains("\"latency_ms\":250"));
        assert!(json.contains("\"tools_considered\""));
        assert!(json.contains("\"decision\":\"Selected read tool\""));
        assert!(json.contains("\"tool_name\":\"read\""));
        assert!(json.contains("\"tool_name\":\"grep\""));
    }

    #[test]
    fn test_reasoning_log_deserialization_preserves_all_fields() {
        let json = r#"{
            "step_id": "step_002",
            "session_id": "sess_xyz789",
            "timestamp": "2026-04-22T10:30:00Z",
            "prompt": "Analyze the code",
            "response": "Found 5 issues",
            "tools_considered": [
                {"tool_name": "grep", "reason": "Search for patterns", "selected": true}
            ],
            "decision": "Using grep for analysis",
            "prompt_tokens": 2000,
            "completion_tokens": 150,
            "latency_ms": 300
        }"#;

        let reasoning: ReasoningLog = serde_json::from_str(json).unwrap();
        assert_eq!(reasoning.step_id, "step_002");
        assert_eq!(reasoning.session_id, "sess_xyz789");
        assert_eq!(reasoning.prompt, "Analyze the code");
        assert_eq!(reasoning.response, "Found 5 issues");
        assert_eq!(reasoning.tools_considered.len(), 1);
        assert_eq!(reasoning.tools_considered[0].tool_name, "grep");
        assert!(reasoning.tools_considered[0].selected);
        assert_eq!(reasoning.decision, "Using grep for analysis");
        assert_eq!(reasoning.prompt_tokens, 2000);
        assert_eq!(reasoning.completion_tokens, 150);
        assert_eq!(reasoning.latency_ms, 300);
    }

    #[test]
    fn test_reasoning_log_token_counts_and_latency_captured() {
        let reasoning = ReasoningLog {
            step_id: "step_003".to_string(),
            session_id: "sess_token_test".to_string(),
            timestamp: Utc::now(),
            prompt: "Test prompt".to_string(),
            response: "Test response".to_string(),
            tools_considered: vec![],
            decision: "No tools needed".to_string(),
            prompt_tokens: 12345,
            completion_tokens: 67890,
            latency_ms: 999,
        };

        let json = serde_json::to_string(&reasoning).unwrap();
        let deserialized: ReasoningLog = serde_json::from_str(&json).unwrap();

        assert_eq!(deserialized.prompt_tokens, 12345);
        assert_eq!(deserialized.completion_tokens, 67890);
        assert_eq!(deserialized.latency_ms, 999);
    }
}
