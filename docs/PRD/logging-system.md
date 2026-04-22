# PRD: Comprehensive Logging System for Coding Agent Self-Troubleshooting

## Overview

This document defines a comprehensive logging system that captures the full execution context of the coding agent to enable self-troubleshooting and debugging. The system must provide visibility into agent reasoning, tool execution, LLM interactions, and errors with sufficient detail for an AI agent to diagnose issues autonomously.

---

## Scope

### In Scope

- **Structured log events** for all agent operations (tool calls, LLM requests, reasoning steps)
- **Log levels**: TRACE, DEBUG, INFO, WARN, ERROR with per-component filtering
- **Log storage and retrieval** with session-based queries
- **TUI log viewer** for real-time debugging
- **Error context capture** with stack traces and causal chains
- **LLM interaction logging** (prompts, responses, tokens, latency)
- **Tool execution logging** with input/output sanitized for security

### Out of Scope

- Log aggregation across multiple OpenCode instances (single instance only)
- Cloud-based log management
- Log-based alerting/monitoring (local only)

---

## Architecture

### Log Event Structure

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogEvent {
    /// Unique sequence number for ordering
    pub seq: u64,
    /// Timestamp with high precision
    pub timestamp: DateTime<Utc>,
    /// Log level
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

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct LogFields {
    pub session_id: Option<String>,
    pub tool_name: Option<String>,
    pub latency_ms: Option<u64>,
    pub model: Option<String>,
    pub provider: Option<String>,
    pub token_count: Option<u64>,
    pub error_code: Option<String>,
    pub file_path: Option<String>,
    pub line: Option<u32>,
}
```

### Log Levels

| Level | Usage |
|-------|-------|
| TRACE | Very detailed execution traces (LLM raw prompts, tool parameters) |
| DEBUG | Detailed debugging information (tool results, intermediate states) |
| INFO | General operational events (session start, tool completion, LLM responses) |
| WARN | Potential issues that don't stop execution (rate limits, retries) |
| ERROR | Failures that need attention (tool failures, API errors) |

---

## Functionality

### 1. Structured Logging API

#### Core Logger Trait

```rust
/// Main logging interface for the agent
pub trait AgentLogger: Send + Sync {
    /// Log a trace event
    fn trace(&self, target: &str, message: &str, fields: LogFields);

    /// Log a debug event
    fn debug(&self, target: &str, message: &str, fields: LogFields);

    /// Log an info event
    fn info(&self, target: &str, message: &str, fields: LogFields);

    /// Log a warning event
    fn warn(&self, target: &str, message: &str, fields: LogFields);

    /// Log an error event
    fn error(&self, target: &str, message: &str, fields: LogFields);

    /// Create a child logger with additional context
    fn with_context(&self, context: LogFields) -> ChildLogger;

    /// Query logs by criteria
    async fn query(&self, criteria: LogQuery) -> Result<Vec<LogEvent>, LogError>;
}

pub struct LogQuery {
    pub session_id: Option<String>,
    pub level: Option<LogLevel>,
    pub target: Option<String>,
    pub since: Option<DateTime<Utc>>,
    pub until: Option<DateTime<Utc>>,
    pub limit: Option<usize>,
}
```

#### Convenience Macros

```rust
/// Log tool execution with automatic context
#[macro_export]
macro_rules! log_tool {
    ($logger:expr, $tool:expr, $status:expr, $($field:tt)*) => {
        $logger.info(
            &format!("tool.{}", $tool),
            &format!("Tool {} completed", $status),
            log_fields!(session_id, tool_name = $tool, $($field)*)
        )
    };
}

/// Log LLM interaction
#[macro_export]
macro_rules! log_llm {
    ($logger:expr, $provider:expr, $model:expr, $tokens:expr, $latency:expr, $status:expr) => {
        $logger.info(
            &format!("llm.{}", $provider),
            &format!("LLM request completed: {}", $status),
            LogFields {
                provider: Some($provider.to_string()),
                model: Some($model.to_string()),
                token_count: Some($tokens),
                latency_ms: Some($latency),
                ..Default::default()
            }
        )
    };
}
```

### 2. Agent Reasoning Log

Log agent decision-making process for self-troubleshooting:

```rust
/// Agent reasoning step for debugging
pub struct ReasoningLog {
    pub step_id: String,
    pub session_id: String,
    pub timestamp: DateTime<Utc>,
    /// The prompt sent to the LLM
    pub prompt: String,
    /// The response received (sanitized)
    pub response: String,
    /// Tools considered and why
    pub tools_considered: Vec<ToolConsideration>,
    /// Final decision and reasoning
    pub decision: String,
    /// Tokens used for this step
    pub prompt_tokens: u64,
    pub completion_tokens: u64,
    pub latency_ms: u64,
}

pub struct ToolConsideration {
    pub tool_name: String,
    pub reason: String,
    pub selected: bool,
}
```

### 3. Tool Execution Logging

Each tool execution logs:

```rust
pub struct ToolExecutionLog {
    pub execution_id: String,
    pub session_id: String,
    pub tool_name: String,
    pub timestamp: DateTime<Utc>,
    /// Tool parameters (sanitized for secrets)
    pub parameters: SanitizedValue,
    /// Execution result
    pub result: ToolResult,
    /// Latency in milliseconds
    pub latency_ms: u64,
    /// Error if any
    pub error: Option<ErrorContext>,
}

/// SanitizedValue redacts sensitive data (API keys, passwords, etc.)
pub enum SanitizedValue {
    Safe(String),
    Redacted(String),  // Replaced with "[REDACTED]"
    Nested(HashMap<String, SanitizedValue>),
}
```

### 4. Error Context Capture

```rust
pub struct ErrorContext {
    /// Error code for programmatic detection
    pub code: String,
    /// Human-readable message
    pub message: String,
    /// Full stack trace or error chain
    pub stack: Vec<ErrorFrame>,
    /// Causality chain (original error → wrapping errors)
    pub cause_chain: Vec<CauseInfo>,
    /// Additional context (file, line, session_id)
    pub context: HashMap<String, String>,
}

pub struct ErrorFrame {
    pub file: String,
    pub line: u32,
    pub function: String,
}
```

### 5. Session Log Store

```rust
/// In-memory ring buffer for recent logs
pub struct SessionLogBuffer {
    capacity: usize,
    buffer: Vec<LogEvent>,
    head: usize,
}

impl SessionLogBuffer {
    /// Add log event, evicting oldest if at capacity
    pub fn push(&mut self, event: LogEvent);

    /// Get logs within a sequence range
    pub fn get_range(&self, from_seq: u64, to_seq: u64) -> Vec<&LogEvent>;

    /// Get logs by level
    pub fn get_by_level(&self, level: LogLevel) -> Vec<&LogEvent>;
}

/// Persistent log storage
pub struct LogStore {
    db: Database,
    log_path: PathBuf,
}

impl LogStore {
    /// Store a log event
    pub async fn append(&self, event: &LogEvent) -> Result<(), LogError>;

    /// Query logs with filters
    pub async fn query(&self, criteria: LogQuery) -> Result<Vec<LogEvent>, LogError>;

    /// Get recent logs for a session (for TUI display)
    pub async fn recent(&self, session_id: &str, limit: usize) -> Result<Vec<LogEvent>, LogError>;

    /// Prune old logs based on retention policy
    pub async fn prune(&self, older_than: DateTime<Utc>) -> Result<u64, LogError>;
}
```

### 6. TUI Log Viewer

The TUI displays a collapsible log panel:

```
┌─────────────────────────────────────────────────────┐
│ Session: sess_abc123  │  Logs (23)  │  [Clear] [Filter] │
├─────────────────────────────────────────────────────┤
│ 10:30:45 INFO  agent    Session started              │
│ 10:30:46 DEBUG tool.read  Read 142 lines from foo.rs │
│ 10:30:47 INFO  llm.openai  Tokens: 1,234 + 567       │
│ 10:30:52 WARN  tool.bash  Exit code 1 from command   │
│ 10:30:53 ERROR agent    Tool execution failed        │
└─────────────────────────────────────────────────────┘
```

**Log Panel Features:**
- Toggle visibility with `Ctrl+L`
- Scroll through logs with arrow keys
- Filter by level (INFO, DEBUG, WARN, ERROR)
- Filter by component (agent, tool.*, llm.*)
- Search within logs
- Click on error to see full details + stack trace
- Auto-scroll for new events (toggleable)

### 7. Configuration

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoggingConfig {
    /// Global minimum log level
    pub level: LogLevel,
    /// Per-component log levels
    pub targets: HashMap<String, LogLevel>,
    /// Log file path (null = no file logging)
    pub file_path: Option<PathBuf>,
    /// Maximum log file size before rotation (MB)
    pub max_file_size_mb: usize,
    /// Number of rotated log files to keep
    pub max_rotated_files: usize,
    /// Enable TUI log panel
    pub show_in_tui: bool,
    /// TUI log panel position
    pub tui_position: TuiLogPosition,
    /// Maximum logs to keep in memory per session
    pub memory_buffer_size: usize,
    /// Log retention period
    pub retention_days: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TuiLogPosition {
    Bottom,   // Below message area
    Right,    // Side panel
    Overlay,  // Floating overlay
}
```

**Default Configuration:**
```toml
[logging]
level = "info"
file_path = "~/.config/opencode/logs/opencode.log"
max_file_size_mb = 10
max_rotated_files = 5
show_in_tui = true
tui_position = "bottom"
memory_buffer_size = 1000
retention_days = 7

[logging.targets]
"agent" = "debug"
"llm.*" = "info"
"tool.*" = "debug"
"error" = "info"
```

---

## Module Structure

```
crates/logging/
├── Cargo.toml
├── src/
│   ├── lib.rs                    # Public exports
│   ├── logger.rs                 # Core Logger trait and implementation
│   ├── event.rs                  # LogEvent, LogFields, LogLevel
│   ├── store.rs                  # SessionLogBuffer and LogStore
│   ├── query.rs                  # LogQuery and filtering
│   ├── sanitizer.rs              # Sensitive data redaction
│   ├── tui/
│   │   ├── mod.rs
│   │   ├── log_panel.rs          # TUI log panel widget
│   │   └── log_renderer.rs       # Formatted log display
│   └── macros.rs                 # log_tool!, log_llm! convenience macros
└── tests/
    ├── logger_tests.rs
    ├── sanitizer_tests.rs
    ├── query_tests.rs
    └── tui_tests.rs
```

---

## Dependencies

```toml
[package]
name = "opencode-logging"
version = "0.1.0"
edition = "2021"

[dependencies]
# Tracing infrastructure
tracing = "0.1"
tracing-appender = "0.2"
tracing-subscriber = { version = "0.3", features = ["json", "env-filter"] }

# Serialization
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"

# Time handling
chrono = { version = "0.4", features = ["serde"] }

# Async runtime
tokio = { version = "1.45", features = ["sync", "rt"] }

# Database for persistent logs
rusqlite = { version = "0.32", features = ["bundled"] }

# Error handling
thiserror = "2.0"
anyhow = "1.0"

# UI (optional, only for TUI integration)
ratatui = { version = "0.28", optional = true }

# UUID for log correlation
uuid = { version = "1.0", features = ["v4", "serde"] }

[features]
default = []
tui = ["ratatui"]
```

---

## Acceptance Criteria

### Core Logging

- [ ] `LogEvent` captures seq, timestamp, level, target, message, fields, span_id, parent_seq
- [ ] Logger writes to both file and memory buffer
- [ ] Log file rotates when max size is reached
- [ ] Per-component log level filtering works
- [ ] Logs are queryable by session_id, level, target, time range

### Agent Reasoning Logs

- [ ] Every LLM interaction logs prompt tokens, completion tokens, latency, model
- [ ] Agent reasoning steps are logged with tool considerations and decisions
- [ ] Reasoning logs are stored persistently and queryable

### Tool Execution Logs

- [ ] Every tool call logs tool name, parameters (sanitized), latency, result/error
- [ ] Secret redaction works for: API keys, passwords, tokens, file paths with secrets
- [ ] Tool errors include full context (file, line, session_id)

### Error Handling

- [ ] Errors capture code, message, stack trace, and cause chain
- [ ] Errors are logged with ERROR level and queryable
- [ ] Error logs include recovery suggestions when available

### TUI Integration

- [ ] Log panel toggles with Ctrl+L
- [ ] Logs display with timestamp, level, target, message columns
- [ ] Filtering by level and component works
- [ ] Click on error shows full details in expandable view
- [ ] Auto-scroll toggle available

### Self-Troubleshooting Support

- [ ] Agent can query its own logs via tool call
- [ ] Log queries support time range, session, level, component filters
- [ ] Full error context available for diagnosis
- [ ] Actionable suggestions available from error codes

---

## Data Structures

### LogEvent (Full)

```rust
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogEvent {
    pub seq: u64,
    pub timestamp: DateTime<Utc>,
    pub level: LogLevel,
    pub target: String,
    pub message: String,
    pub fields: LogFields,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub span_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub parent_seq: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct LogFields {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub session_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub latency_ms: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub model: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub provider: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub token_count: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error_code: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub file_path: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub line: Option<u32>,
    #[serde(flatten)]
    pub extra: HashMap<String, serde_json::Value>,
}
```

### LogLevel

```rust
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum LogLevel {
    Trace,
    Debug,
    Info,
    Warn,
    Error,
}

impl LogLevel {
    pub fn as_str(&self) -> &'static str {
        match self {
            LogLevel::Trace => "TRACE",
            LogLevel::Debug => "DEBUG",
            LogLevel::Info => "INFO",
            LogLevel::Warn => "WARN",
            LogLevel::Error => "ERROR",
        }
    }
}
```

---

## Test Design

### Unit Tests

```rust
#[test]
fn test_log_event_serialization() {
    let event = LogEvent {
        seq: 1,
        timestamp: Utc::now(),
        level: LogLevel::Info,
        target: "agent".to_string(),
        message: "Session started".to_string(),
        fields: LogFields {
            session_id: Some("sess_123".to_string()),
            ..Default::default()
        },
        span_id: None,
        parent_seq: None,
    };
    let json = serde_json::to_string(&event).unwrap();
    assert!(json.contains("sess_123"));
}

#[test]
fn test_sanitizer_redacts_secrets() {
    let input = serde_json::json!({
        "api_key": "sk-secret123",
        "password": "my_password",
        "query": "normal query"
    });
    let sanitized = sanitize_json(&input);
    assert_eq!(sanitized["api_key"], serde_json::json!("[REDACTED]"));
    assert_eq!(sanitized["password"], serde_json::json!("[REDACTED]"));
    assert_eq!(sanitized["query"], serde_json::json!("normal query"));
}

#[test]
fn test_log_query_filter_by_level() {
    let query = LogQuery {
        level: Some(LogLevel::Error),
        ..Default::default()
    };
    assert!(query.matches_level(&LogLevel::Error));
    assert!(!query.matches_level(&LogLevel::Info));
}
```

### Integration Tests

```rust
#[tokio::test]
async fn test_log_store_query_by_session() {
    let store = LogStore::new_temp().await.unwrap();
    let session_id = "sess_abc";

    // Insert logs
    for i in 0..5 {
        let event = LogEvent {
            seq: i,
            timestamp: Utc::now(),
            level: LogLevel::Info,
            target: "agent".to_string(),
            message: format!("Log {}", i),
            fields: LogFields {
                session_id: Some(session_id.to_string()),
                ..Default::default()
            },
            span_id: None,
            parent_seq: None,
        };
        store.append(&event).await.unwrap();
    }

    // Query by session
    let results = store.query(LogQuery {
        session_id: Some(session_id.to_string()),
        ..Default::default()
    }).await.unwrap();

    assert_eq!(results.len(), 5);
}

#[tokio::test]
async fn test_log_rotation() {
    let temp_dir = tempfile::tempdir().unwrap();
    let logger = FileLogger::new(
        temp_dir.path().join("test.log"),
        100,  // 100 bytes max
        3,    // 3 files max
    );

    // Write enough to trigger rotation
    for i in 0..20 {
        logger.info("target", &format!("Log entry {}", i), LogFields::default());
    }

    // Verify rotation occurred
    let files = std::fs::read_dir(temp_dir.path()).unwrap().count();
    assert!(files > 1);
}
```

---

## Implementation Notes

### Performance Considerations

- Use lock-free ring buffer for in-memory logs (per-session)
- Async write to file (non-blocking)
- Batch writes for persistent storage
- Lazy serialization for unused fields

### Security Considerations

- Never log raw API keys or tokens
- Sanitize all tool parameters before logging
- Redact file paths with secrets
- Limit log retention for sensitive data

### Compatibility

- Preserve existing `tracing` integration in `crates/util/logging.rs`
- Extend observability module with new logging features
- Maintain backward compatibility for existing log queries

---

## Cross-References

- [01-core-architecture.md](./01-core-architecture.md) — Session and entity model
- [02-agent-system.md](./02-agent-system.md) — Agent execution model
- [03-tools-system.md](./03-tools-system.md) — Tool execution lifecycle
- [06-configuration-system.md](./06-configuration-system.md) — Config schema
- [09-tui-system.md](./09-tui-system.md) — TUI panel integration