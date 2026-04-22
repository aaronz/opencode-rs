# Logging System Gap Analysis Report

**Generated:** 2026-04-22
**PRD Source:** `docs/PRD/system/logging-system.md`
**Implementation Source:** `opencode-rust/crates/util/src/logging.rs`, `opencode-rust/crates/core/src/observability.rs`

---

## 1. Gap Summary

| Gap Item | Severity | Module | 修复建议 |
|----------|----------|--------|---------|
| No `crates/logging/` crate | P0 | Architecture | Create dedicated logging crate |
| No `LogEvent` struct | P0 | event.rs | Implement LogEvent with seq, timestamp, level, target, message, fields, span_id, parent_seq |
| No `LogFields` struct | P0 | event.rs | Implement LogFields with all optional fields + extra HashMap |
| No `LogLevel::Trace` | P0 | event.rs | Add Trace variant to LogLevel enum |
| No `AgentLogger` trait | P0 | logger.rs | Implement trait with trace/debug/info/warn/error + with_context + query |
| No `LogQuery` struct | P0 | query.rs | Implement query filtering by session_id, level, target, time range |
| No `SessionLogBuffer` (ring buffer) | P0 | store.rs | Implement in-memory ring buffer for recent logs |
| No `LogStore` (persistent) | P0 | store.rs | Implement SQLite-backed persistent log storage |
| No `ReasoningLog` struct | P1 | logging | Implement agent reasoning log with prompt, response, tools_considered, decision |
| No `ToolExecutionLog` struct | P1 | logging | Implement tool execution log with sanitized parameters |
| No `ErrorContext` struct | P1 | logging | Implement error context with code, message, stack, cause_chain |
| No `SanitizedValue` enum | P1 | sanitizer.rs | Implement secret redaction for API keys, passwords, tokens |
| No `log_tool!` macro | P1 | macros.rs | Implement convenience macro for tool execution logging |
| No `log_llm!` macro | P1 | macros.rs | Implement convenience macro for LLM interaction logging |
| No `LoggingConfig` struct | P1 | config | Implement configuration with targets, file_path, tui_position, etc. |
| No TUI Log Panel | P1 | tui | Implement log panel widget with Ctrl+L toggle |
| No convenience `log_fields!` macro | P2 | macros.rs | Implement macro for constructing LogFields |
| No per-component log level filtering | P2 | logging | Implement target-based filtering (agent, llm.*, tool.*) |
| No log file rotation by size | P2 | logging | Implement size-based rotation in addition to daily |
| No self-troubleshooting tool | P2 | tools | Implement log query tool for agent to query own logs |

---

## 2. P0 Blockers (Must Fix)

### 2.1 No Dedicated `crates/logging/` Crate

**Current State:** Logging is scattered in `crates/util/src/logging.rs` with only basic `Logger` struct.

**PRD Requirement:** "Module Structure: `crates/logging/` with lib.rs, logger.rs, event.rs, store.rs, query.rs, sanitizer.rs, tui/, macros.rs"

**Gap:** No `crates/logging/` directory. The logging module is not isolated as its own crate.

**Fix:** Create `crates/logging/` with:
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

### 2.2 No `LogEvent` Structure

**Current State:** No `LogEvent` struct exists.

**PRD Requirement:**
```rust
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
```

**Gap:** No LogEvent implementation.

**Fix:** Implement `LogEvent` in `event.rs` with all required fields and proper serialization.

### 2.3 No `LogFields` Structure

**Current State:** No `LogFields` struct exists.

**PRD Requirement:**
```rust
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
```

**Gap:** No LogFields implementation.

**Fix:** Implement `LogFields` in `event.rs` with all optional fields plus flattened extra HashMap.

### 2.4 Missing `LogLevel::Trace`

**Current State:** `crates/util/src/logging.rs` has `LogLevel` with Debug, Info, Warn, Error only.

**PRD Requirement:** TRACE level for "Very detailed execution traces (LLM raw prompts, tool parameters)"

**Gap:** No Trace variant.

**Fix:** Add `Trace` variant to `LogLevel` enum and update `From<LogLevel> for Level` implementation.

### 2.5 No `AgentLogger` Trait

**Current State:** Basic `Logger` struct exists but no `AgentLogger` trait.

**PRD Requirement:**
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

**Gap:** No AgentLogger trait.

**Fix:** Implement `AgentLogger` trait in `logger.rs` with ChildLogger for context.

### 2.6 No `LogQuery` Structure

**Current State:** No LogQuery exists.

**PRD Requirement:**
```rust
pub struct LogQuery {
    pub session_id: Option<String>,
    pub level: Option<LogLevel>,
    pub target: Option<String>,
    pub since: Option<DateTime<Utc>>,
    pub until: Option<DateTime<Utc>>,
    pub limit: Option<usize>,
}
```

**Gap:** No LogQuery struct.

**Fix:** Implement `LogQuery` in `query.rs` with filtering logic.

### 2.7 No `SessionLogBuffer` (Ring Buffer)

**Current State:** No in-memory log buffer exists.

**PRD Requirement:** "In-memory ring buffer for recent logs"

**Gap:** No SessionLogBuffer implementation.

**Fix:** Implement `SessionLogBuffer` in `store.rs` with push, get_range, get_by_level methods.

### 2.8 No `LogStore` (Persistent Storage)

**Current State:** Basic file logging via tracing but no structured queryable storage.

**PRD Requirement:** "Persistent log storage" with append, query, recent, prune methods

**Gap:** No SQLite-backed LogStore.

**Fix:** Implement `LogStore` in `store.rs` using rusqlite for persistent storage.

---

## 3. P1 Issues (Should Fix)

### 3.1 No `ReasoningLog` Structure

**Current State:** Basic token tracking in `ObservabilityTracker` but no reasoning log.

**PRD Requirement:**
```rust
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
```

**Gap:** No ReasoningLog implementation.

**Fix:** Implement ReasoningLog for agent decision-making debugging.

### 3.2 No `ToolExecutionLog` Structure

**Current State:** Basic tool call tracking but no structured execution log.

**PRD Requirement:**
```rust
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
```

**Gap:** No ToolExecutionLog implementation.

**Fix:** Implement ToolExecutionLog with SanitizedValue for parameter redaction.

### 3.3 No `ErrorContext` Structure

**Current State:** Basic error tracking but no structured ErrorContext.

**PRD Requirement:**
```rust
pub struct ErrorContext {
    pub code: String,
    pub message: String,
    pub stack: Vec<ErrorFrame>,
    pub cause_chain: Vec<CauseInfo>,
    pub context: HashMap<String, String>,
}
```

**Gap:** No ErrorContext implementation.

**Fix:** Implement ErrorContext with stack trace and cause chain support.

### 3.4 No `SanitizedValue` Enum

**Current State:** No secret redaction for tool parameters.

**PRD Requirement:**
```rust
pub enum SanitizedValue {
    Safe(String),
    Redacted(String),
    Nested(HashMap<String, SanitizedValue>),
}
```

**Gap:** No SanitizedValue implementation.

**Fix:** Implement sanitizer.rs with redaction for api_key, password, token, secret fields.

### 3.5 No `log_tool!` Macro

**Current State:** No convenience macro for tool execution logging.

**PRD Requirement:**
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

**Gap:** No log_tool! macro.

**Fix:** Implement macro in macros.rs.

### 3.6 No `log_llm!` Macro

**Current State:** No convenience macro for LLM interaction logging.

**PRD Requirement:**
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

**Gap:** No log_llm! macro.

**Fix:** Implement macro in macros.rs.

### 3.7 No `LoggingConfig` Structure

**Current State:** Basic Logger config in util but no LoggingConfig.

**PRD Requirement:**
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

**Gap:** No LoggingConfig implementation.

**Fix:** Implement LoggingConfig with target-based filtering and TUI settings.

### 3.8 No TUI Log Panel

**Current State:** No log panel in TUI.

**PRD Requirement:**
```
┌─────────────────────────────────────────────────────┐
│ Session: sess_abc123  │  Logs (23)  │  [Clear] [Filter] │
├─────────────────────────────────────────────────────┤
│ 10:30:45 INFO  agent    Session started              │
│ 10:30:46 DEBUG tool.read  Read 142 lines from foo.rs │
│ 10:30:47 INFO  llm.openai  Tokens: 1,234 + 567       │
└─────────────────────────────────────────────────────┘
```

**Features:**
- Toggle visibility with `Ctrl+L`
- Scroll through logs with arrow keys
- Filter by level and component
- Click on error for details
- Auto-scroll toggle

**Gap:** No LogPanel widget, no log_renderer.rs, no tui/mod.rs.

**Fix:** Create tui/ directory with log_panel.rs and log_renderer.rs.

---

## 4. P2 Issues (Nice to Have)

### 4.1 No `log_fields!` Macro

**Current State:** No macro for constructing LogFields.

**PRD Requirement:** `log_fields!(session_id, tool_name = $tool, $($field)*)` syntax.

**Gap:** No log_fields! macro.

**Fix:** Implement log_fields! macro in macros.rs.

### 4.2 No Per-Component Log Level Filtering

**Current State:** Global log level only.

**PRD Requirement:** Per-component levels like `"agent" = "debug"`, `"llm.*" = "info"`, `"tool.*" = "debug"`.

**Gap:** No target-based filtering.

**Fix:** Implement target glob pattern matching in logger.

### 4.3 No Size-Based Log Rotation

**Current State:** Daily rotation only via tracing-appender.

**PRD Requirement:** Rotate when max_file_size_mb is reached.

**Gap:** Size-based rotation not implemented.

**Fix:** Implement size-based rotation in logger.

### 4.4 No Self-Troubleshooting Log Query Tool

**Current State:** Agent cannot query its own logs.

**PRD Requirement:** "Agent can query its own logs via tool call"

**Gap:** No log query tool.

**Fix:** Implement log query tool for agent self-diagnosis.

---

## 5. Technical Debt

| Item | Description |
|------|-------------|
| Basic Logger | Current `crates/util/src/logging.rs` Logger only supports file output, not memory buffer |
| Rotation Config | `Rotation` struct exists but not fully utilized |
| ObservabilityTracker | `crates/core/src/observability.rs` has basic token tracking but not integrated with logging system |
| No Test Infrastructure | No tests for logging module (sanitizer_tests.rs, query_tests.rs, tui_tests.rs don't exist) |
| Tracing Integration | Existing tracing setup needs to be extended, not replaced |

---

## 6. Implementation Progress Summary

### Completed (0%)

The logging system per PRD has **NOT been implemented**. The current state is:

| Component | Status |
|-----------|--------|
| LogEvent | ❌ Not implemented |
| LogFields | ❌ Not implemented |
| LogLevel (with TRACE) | ❌ Not implemented |
| AgentLogger trait | ❌ Not implemented |
| LogQuery | ❌ Not implemented |
| SessionLogBuffer | ❌ Not implemented |
| LogStore | ❌ Not implemented |
| ReasoningLog | ❌ Not implemented |
| ToolExecutionLog | ❌ Not implemented |
| ErrorContext | ❌ Not implemented |
| SanitizedValue | ❌ Not implemented |
| log_tool! macro | ❌ Not implemented |
| log_llm! macro | ❌ Not implemented |
| LoggingConfig | ❌ Not implemented |
| TUI Log Panel | ❌ Not implemented |
| crates/logging/ crate | ❌ Not created |

### Existing Code That Can Be Reused

1. **`crates/util/src/logging.rs`**: Contains basic `LogLevel` (without Trace), `Logger`, `Rotation` - can be extended
2. **`crates/core/src/observability.rs`**: Contains `TokenUsage`, `SessionTrace`, `ObservabilityTracker` - partial functionality
3. **`tracing` integration**: Existing tracing subscriber setup can be leveraged

---

## 7. Recommended Implementation Order

1. **Create `crates/logging/` crate** with basic structure
2. **Implement `event.rs`** - LogEvent, LogFields, LogLevel with TRACE
3. **Implement `query.rs`** - LogQuery with filtering
4. **Implement `store.rs`** - SessionLogBuffer (ring buffer) + LogStore (SQLite)
5. **Implement `logger.rs`** - AgentLogger trait + implementation
6. **Implement `sanitizer.rs`** - SanitizedValue for secret redaction
7. **Implement `macros.rs`** - log_tool!, log_llm!, log_fields!
8. **Implement TUI components** - log_panel.rs, log_renderer.rs
9. **Add tests** - logger_tests.rs, sanitizer_tests.rs, query_tests.rs, tui_tests.rs

---

## 8. Dependencies Required

```toml
# In crates/logging/Cargo.toml
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