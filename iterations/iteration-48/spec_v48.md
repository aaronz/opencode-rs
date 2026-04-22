# Logging System Specification v1.0

**Document Version:** 48
**Generated:** 2026-04-22
**PRD Source:** `docs/PRD/system/logging-system.md`
**Status:** Implementation Required

---

## 1. Overview

### 1.1 Purpose

This document defines the comprehensive logging system for the OpenCode coding agent to enable self-troubleshooting and debugging. The system captures full execution context including agent reasoning, tool execution, LLM interactions, and errors with sufficient detail for AI-powered diagnosis.

### 1.2 Scope

**In Scope:**
- Structured log events for all agent operations
- Log levels: TRACE, DEBUG, INFO, WARN, ERROR with per-component filtering
- Log storage and retrieval with session-based queries
- TUI log viewer for real-time debugging
- Error context capture with stack traces and causal chains
- LLM interaction logging (prompts, responses, tokens, latency)
- Tool execution logging with input/output sanitization

**Out of Scope:**
- Log aggregation across multiple OpenCode instances
- Cloud-based log management
- Log-based alerting/monitoring

---

## 2. Module Structure

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
│   │   └── log_renderer.rs        # Formatted log display
│   └── macros.rs                 # log_tool!, log_llm! convenience macros
└── tests/
    ├── logger_tests.rs
    ├── sanitizer_tests.rs
    ├── query_tests.rs
    └── tui_tests.rs
```

---

## 3. Functional Requirements

### FR-001: Log Event Structure

The system MUST implement `LogEvent` with the following fields:

| Field | Type | Description |
|-------|------|-------------|
| `seq` | `u64` | Unique sequence number for ordering |
| `timestamp` | `DateTime<Utc>` | High-precision timestamp |
| `level` | `LogLevel` | Log severity level |
| `target` | `String` | Target component (e.g., "agent", "tool.read", "llm.openai") |
| `message` | `String` | Human-readable message |
| `fields` | `LogFields` | Structured fields for querying |
| `span_id` | `Option<String>` | Span context for trace correlation |
| `parent_seq` | `Option<u64>` | Parent log event ID for causality chains |

**Implementation:** `crates/logging/src/event.rs`

---

### FR-002: Log Fields Structure

The system MUST implement `LogFields` with the following optional fields:

| Field | Type | Description |
|-------|------|-------------|
| `session_id` | `Option<String>` | Session identifier |
| `tool_name` | `Option<String>` | Tool name if applicable |
| `latency_ms` | `Option<u64>` | Operation latency in milliseconds |
| `model` | `Option<String>` | LLM model name |
| `provider` | `Option<String>` | LLM provider name |
| `token_count` | `Option<u64>` | Token count for LLM operations |
| `error_code` | `Option<String>` | Error code for errors |
| `file_path` | `Option<String>` | File path for file operations |
| `line` | `Option<u32>` | Line number for location context |
| `extra` | `HashMap<String, serde_json::Value>` | Additional flattened fields |

**Implementation:** `crates/logging/src/event.rs`

---

### FR-003: Log Level Definition

The system MUST implement `LogLevel` enum with five levels:

| Level | Usage | Default Filter |
|-------|-------|----------------|
| `TRACE` | Very detailed execution traces (LLM raw prompts, tool parameters) | Off |
| `DEBUG` | Detailed debugging information (tool results, intermediate states) | Off |
| `INFO` | General operational events (session start, tool completion, LLM responses) | On |
| `WARN` | Potential issues that don't stop execution (rate limits, retries) | On |
| `ERROR` | Failures that need attention (tool failures, API errors) | On |

**Implementation:** `crates/logging/src/event.rs`

---

### FR-004: AgentLogger Trait

The system MUST implement `AgentLogger` trait with the following interface:

```rust
pub trait AgentLogger: Send + Sync {
    fn trace(&self, target: &str, message: &str, fields: LogFields);
    fn debug(&self, target: &str, message: &str, fields: LogFields);
    fn info(&self, target: &str, message: &str, fields: LogFields);
    fn warn(&self, target: &str, message: &str, fields: LogFields);
    fn error(&self, target: &str, message: &str, fields: LogFields);
    fn with_context(&self, context: LogFields) -> ChildLogger;
    async fn query(&self, criteria: LogQuery) -> Result<Vec<LogEvent>, LogError>;
}
```

**Implementation:** `crates/logging/src/logger.rs`

---

### FR-005: Log Query Structure

The system MUST implement `LogQuery` for filtering logs:

| Field | Type | Description |
|-------|------|-------------|
| `session_id` | `Option<String>` | Filter by session |
| `level` | `Option<LogLevel>` | Filter by log level |
| `target` | `Option<String>` | Filter by target component |
| `since` | `Option<DateTime<Utc>>` | Filter logs after this time |
| `until` | `Option<DateTime<Utc>>` | Filter logs before this time |
| `limit` | `Option<usize>` | Maximum results to return |

**Implementation:** `crates/logging/src/query.rs`

---

### FR-006: Session Log Buffer (Ring Buffer)

The system MUST implement `SessionLogBuffer` for in-memory log storage:

| Method | Description |
|--------|-------------|
| `push(&mut self, event: LogEvent)` | Add log event, evicting oldest if at capacity |
| `get_range(&self, from_seq: u64, to_seq: u64) -> Vec<&LogEvent>` | Get logs within sequence range |
| `get_by_level(&self, level: LogLevel) -> Vec<&LogEvent>` | Get logs by level |

**Configuration:**
- `capacity: usize` - Maximum number of events to store
- Uses ring buffer algorithm for O(1) insert with oldest eviction

**Implementation:** `crates/logging/src/store.rs`

---

### FR-007: Persistent Log Store

The system MUST implement `LogStore` for SQLite-backed persistent storage:

| Method | Description |
|--------|-------------|
| `append(&self, event: &LogEvent) -> Result<(), LogError>` | Store a log event |
| `query(&self, criteria: LogQuery) -> Result<Vec<LogEvent>, LogError>` | Query logs with filters |
| `recent(&self, session_id: &str, limit: usize) -> Result<Vec<LogEvent>, LogError>` | Get recent logs for TUI |
| `prune(&self, older_than: DateTime<Utc>) -> Result<u64, LogError>` | Remove old logs based on retention |

**Database Schema:**
```sql
CREATE TABLE logs (
    seq INTEGER PRIMARY KEY,
    timestamp TEXT NOT NULL,
    level TEXT NOT NULL,
    target TEXT NOT NULL,
    message TEXT NOT NULL,
    fields TEXT NOT NULL,
    span_id TEXT,
    parent_seq INTEGER
);

CREATE INDEX idx_logs_session ON logs(session_id);
CREATE INDEX idx_logs_level ON logs(level);
CREATE INDEX idx_logs_timestamp ON logs(timestamp);
```

**Implementation:** `crates/logging/src/store.rs`

---

### FR-008: Reasoning Log Structure

The system MUST implement `ReasoningLog` for agent decision-making:

| Field | Type | Description |
|-------|------|-------------|
| `step_id` | `String` | Unique step identifier |
| `session_id` | `String` | Session identifier |
| `timestamp` | `DateTime<Utc>` | Timestamp |
| `prompt` | `String` | Prompt sent to LLM |
| `response` | `String` | Response received (sanitized) |
| `tools_considered` | `Vec<ToolConsideration>` | Tools considered and reasoning |
| `decision` | `String` | Final decision and reasoning |
| `prompt_tokens` | `u64` | Tokens used for prompt |
| `completion_tokens` | `u64` | Tokens used for completion |
| `latency_ms` | `u64` | Total latency |

```rust
pub struct ToolConsideration {
    pub tool_name: String,
    pub reason: String,
    pub selected: bool,
}
```

**Implementation:** `crates/logging/src/event.rs`

---

### FR-009: Tool Execution Log Structure

The system MUST implement `ToolExecutionLog`:

| Field | Type | Description |
|-------|------|-------------|
| `execution_id` | `String` | Unique execution identifier |
| `session_id` | `String` | Session identifier |
| `tool_name` | `String` | Tool name |
| `timestamp` | `DateTime<Utc>` | Execution timestamp |
| `parameters` | `SanitizedValue` | Tool parameters (sanitized) |
| `result` | `ToolResult` | Execution result |
| `latency_ms` | `u64` | Execution latency |
| `error` | `Option<ErrorContext>` | Error if any |

**Implementation:** `crates/logging/src/event.rs`

---

### FR-010: Error Context Structure

The system MUST implement `ErrorContext`:

| Field | Type | Description |
|-------|------|-------------|
| `code` | `String` | Error code for programmatic detection |
| `message` | `String` | Human-readable message |
| `stack` | `Vec<ErrorFrame>` | Full stack trace or error chain |
| `cause_chain` | `Vec<CauseInfo>` | Causality chain (original → wrapping) |
| `context` | `HashMap<String, String>` | Additional context |

```rust
pub struct ErrorFrame {
    pub file: String,
    pub line: u32,
    pub function: String,
}

pub struct CauseInfo {
    pub code: String,
    pub message: String,
}
```

**Implementation:** `crates/logging/src/event.rs`

---

### FR-011: Sanitized Value for Secret Redaction

The system MUST implement `SanitizedValue` for secret redaction:

```rust
pub enum SanitizedValue {
    Safe(String),
    Redacted(String),
    Nested(HashMap<String, SanitizedValue>),
}
```

**Redaction Rules:**
| Field Pattern | Action |
|--------------|--------|
| `api_key` | Redact |
| `password` | Redact |
| `token` | Redact |
| `secret` | Redact |
| `authorization` | Redact |
| `.*_key` | Redact |
| `.*_token` | Redact |
| `.*_secret` | Redact |

**Implementation:** `crates/logging/src/sanitizer.rs`

---

### FR-012: Log Tool Macro

The system MUST implement `log_tool!` convenience macro:

```rust
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
```

**Usage Example:**
```rust
log_tool!(logger, "read", "success", latency_ms = 45);
```

**Implementation:** `crates/logging/src/macros.rs`

---

### FR-013: Log LLM Macro

The system MUST implement `log_llm!` convenience macro:

```rust
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

**Usage Example:**
```rust
log_llm!(logger, "openai", "gpt-4", 1800, 250, "success");
```

**Implementation:** `crates/logging/src/macros.rs`

---

### FR-014: Log Fields Macro

The system MUST implement `log_fields!` convenience macro:

```rust
#[macro_export]
macro_rules! log_fields {
    ($($field:ident),*) => { ... };
    ($($field:ident = $value:expr),*) => { ... };
}
```

**Usage Example:**
```rust
log_fields!(session_id, tool_name = "read");
log_fields!(latency_ms = 45, error_code = Some("ERR_NOT_FOUND".to_string()));
```

**Implementation:** `crates/logging/src/macros.rs`

---

### FR-015: Logging Configuration

The system MUST implement `LoggingConfig`:

```rust
pub struct LoggingConfig {
    pub level: LogLevel,
    pub targets: HashMap<String, LogLevel>,
    pub file_path: Option<PathBuf>,
    pub max_file_size_mb: usize,
    pub max_rotated_files: usize,
    pub show_in_tui: bool,
    pub tui_position: TuiLogPosition,
    pub memory_buffer_size: usize,
    pub retention_days: u32,
}

pub enum TuiLogPosition {
    Bottom,
    Right,
    Overlay,
}
```

**Default Configuration (TOML):**
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

**Implementation:** `crates/logging/src/config.rs`

---

### FR-016: TUI Log Panel

The system MUST implement a TUI log panel with the following features:

**Layout:**
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

**Features:**
| Feature | Description |
|---------|-------------|
| Toggle visibility | `Ctrl+L` key binding |
| Scroll navigation | Arrow keys for up/down |
| Level filter | Filter by INFO, DEBUG, WARN, ERROR |
| Component filter | Filter by agent, tool.*, llm.* |
| Search | Text search within logs |
| Error details | Click on error for expandable stack trace |
| Auto-scroll | Toggle for new event auto-scroll |

**Implementation:** `crates/logging/src/tui/log_panel.rs`, `crates/logging/src/tui/log_renderer.rs`

---

### FR-017: Per-Component Log Level Filtering

The system MUST support per-component log level filtering with glob patterns:

**Pattern Examples:**
| Pattern | Matches |
|---------|---------|
| `agent` | Exactly "agent" |
| `llm.*` | "llm.openai", "llm.anthropic" |
| `tool.read` | Exactly "tool.read" |
| `tool.*` | "tool.read", "tool.write", "tool.bash" |

**Filter Resolution:**
1. Check for exact target match
2. Check for glob pattern match (`*` matches any string)
3. Fall back to global level if no match

**Implementation:** `crates/logging/src/logger.rs`

---

### FR-018: Size-Based Log Rotation

The system MUST implement size-based log rotation:

| Parameter | Description |
|-----------|-------------|
| `max_file_size_mb` | Max file size before rotation |
| `max_rotated_files` | Number of rotated files to keep |

**Rotation Behavior:**
1. When log file exceeds `max_file_size_mb`, rotate
2. Rename current log to `opencode.log.N` (N = 1, 2, ...)
3. Create new `opencode.log`
4. Delete oldest when `max_rotated_files` exceeded

**Implementation:** `crates/logging/src/logger.rs`

---

### FR-019: Self-Troubleshooting Log Query Tool

The system MUST implement a log query tool for agent self-diagnosis:

**Tool Interface:**
```rust
pub struct LogQueryTool {
    pub session_id: Option<String>,
    pub level: Option<String>,
    pub target: Option<String>,
    pub since: Option<String>,
    pub until: Option<String>,
    pub limit: Option<usize>,
}
```

**Tool Result:**
Returns structured log events matching the query criteria.

**Implementation:** `crates/tools/src/log_query.rs`

---

## 4. Data Structures Summary

### 4.1 Core Structures

```rust
// event.rs
pub struct LogEvent {
    pub seq: u64,
    pub timestamp: DateTime<Utc>,
    pub level: LogLevel,
    pub target: String,
    pub message: String,
    pub fields: LogFields,
    pub span_id: Option<String>,
    pub parent_seq: Option<u64>,
}

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
    pub extra: HashMap<String, serde_json::Value>,
}

pub enum LogLevel {
    Trace,
    Debug,
    Info,
    Warn,
    Error,
}

pub struct ReasoningLog {
    pub step_id: String,
    pub session_id: String,
    pub timestamp: DateTime<Utc>,
    pub prompt: String,
    pub response: String,
    pub tools_considered: Vec<ToolConsideration>,
    pub decision: String,
    pub prompt_tokens: u64,
    pub completion_tokens: u64,
    pub latency_ms: u64,
}

pub struct ToolExecutionLog {
    pub execution_id: String,
    pub session_id: String,
    pub tool_name: String,
    pub timestamp: DateTime<Utc>,
    pub parameters: SanitizedValue,
    pub result: ToolResult,
    pub latency_ms: u64,
    pub error: Option<ErrorContext>,
}

pub struct ErrorContext {
    pub code: String,
    pub message: String,
    pub stack: Vec<ErrorFrame>,
    pub cause_chain: Vec<CauseInfo>,
    pub context: HashMap<String, String>,
}

pub struct ErrorFrame {
    pub file: String,
    pub line: u32,
    pub function: String,
}

pub enum SanitizedValue {
    Safe(String),
    Redacted(String),
    Nested(HashMap<String, SanitizedValue>),
}
```

### 4.2 Query Structure

```rust
// query.rs
pub struct LogQuery {
    pub session_id: Option<String>,
    pub level: Option<LogLevel>,
    pub target: Option<String>,
    pub since: Option<DateTime<Utc>>,
    pub until: Option<DateTime<Utc>>,
    pub limit: Option<usize>,
}
```

### 4.3 Configuration Structure

```rust
// config.rs
pub struct LoggingConfig {
    pub level: LogLevel,
    pub targets: HashMap<String, LogLevel>,
    pub file_path: Option<PathBuf>,
    pub max_file_size_mb: usize,
    pub max_rotated_files: usize,
    pub show_in_tui: bool,
    pub tui_position: TuiLogPosition,
    pub memory_buffer_size: usize,
    pub retention_days: u32,
}

pub enum TuiLogPosition {
    Bottom,
    Right,
    Overlay,
}
```

---

## 5. Dependencies

```toml
# crates/logging/Cargo.toml
[package]
name = "opencode-logging"
version = "0.1.0"
edition = "2021"

[dependencies]
tracing = "0.1"
tracing-appender = "0.2"
tracing-subscriber = { version = "0.3", features = ["json", "env-filter"] }
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
chrono = { version = "0.4", features = ["serde"] }
tokio = { version = "1.45", features = ["sync", "rt"] }
rusqlite = { version = "0.32", features = ["bundled"] }
thiserror = "2.0"
anyhow = "1.0"
ratatui = { version = "0.28", optional = true }
uuid = { version = "1.0", features = ["v4", "serde"] }

[features]
default = []
tui = ["ratatui"]
```

---

## 6. Implementation Order

1. **Phase 1: Core Infrastructure**
   - FR-001: LogEvent structure
   - FR-002: LogFields structure
   - FR-003: LogLevel with TRACE

2. **Phase 2: Query & Storage**
   - FR-005: LogQuery structure
   - FR-006: SessionLogBuffer (ring buffer)
   - FR-007: LogStore (persistent SQLite)

3. **Phase 3: Logger Implementation**
   - FR-004: AgentLogger trait
   - FR-015: LoggingConfig
   - FR-017: Per-component log level filtering
   - FR-018: Size-based log rotation

4. **Phase 4: Security & Macros**
   - FR-011: SanitizedValue for secret redaction
   - FR-012: log_tool! macro
   - FR-013: log_llm! macro
   - FR-014: log_fields! macro

5. **Phase 5: Extended Structures**
   - FR-008: ReasoningLog
   - FR-009: ToolExecutionLog
   - FR-010: ErrorContext

6. **Phase 6: TUI Integration**
   - FR-016: TUI Log Panel

7. **Phase 7: Self-Troubleshooting**
   - FR-019: Log query tool

---

## 7. Test Requirements

### Unit Tests

| Test File | Coverage |
|-----------|----------|
| `logger_tests.rs` | LogEvent serialization, level filtering, query matching |
| `sanitizer_tests.rs` | Secret redaction for api_key, password, token fields |
| `query_tests.rs` | LogQuery filtering by session, level, target, time range |
| `tui_tests.rs` | Log panel rendering, filter behavior, empty state |

### Integration Tests

| Test | Description |
|------|-------------|
| `test_log_store_query_by_session` | Insert logs and query by session_id |
| `test_log_rotation` | Write enough to trigger rotation, verify files |
| `test_reasoning_log_persistence` | Store and retrieve reasoning logs |
| `test_tool_execution_sanitization` | Verify sensitive params are redacted |

---

## 8. Acceptance Criteria

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

## 9. Cross-References

- [01-core-architecture.md](./01-core-architecture.md) — Session and entity model
- [02-agent-system.md](./02-agent-system.md) — Agent execution model
- [03-tools-system.md](./03-tools-system.md) — Tool execution lifecycle
- [06-configuration-system.md](./06-configuration-system.md) — Config schema
- [09-tui-system.md](./09-tui-system.md) — TUI panel integration

---

## 10. Gap Resolution Status

| Gap Item | Severity | FR Number | Status |
|----------|----------|-----------|--------|
| No `crates/logging/` crate | P0 | - | Not started |
| No `LogEvent` struct | P0 | FR-001 | Not started |
| No `LogFields` struct | P0 | FR-002 | Not started |
| No `LogLevel::Trace` | P0 | FR-003 | Not started |
| No `AgentLogger` trait | P0 | FR-004 | Not started |
| No `LogQuery` struct | P0 | FR-005 | Not started |
| No `SessionLogBuffer` (ring buffer) | P0 | FR-006 | Not started |
| No `LogStore` (persistent) | P0 | FR-007 | Not started |
| No `ReasoningLog` struct | P1 | FR-008 | Not started |
| No `ToolExecutionLog` struct | P1 | FR-009 | Not started |
| No `ErrorContext` struct | P1 | FR-010 | Not started |
| No `SanitizedValue` enum | P1 | FR-011 | Not started |
| No `log_tool!` macro | P1 | FR-012 | Not started |
| No `log_llm!` macro | P1 | FR-013 | Not started |
| No `LoggingConfig` struct | P1 | FR-015 | Not started |
| No TUI Log Panel | P1 | FR-016 | Not started |
| No `log_fields!` macro | P2 | FR-014 | Not started |
| No per-component log level filtering | P2 | FR-017 | Not started |
| No log file rotation by size | P2 | FR-018 | Not started |
| No self-troubleshooting tool | P2 | FR-019 | Not started |

---

*Document generated from gap analysis. Implementation status tracked per FR number.*