# Logging System Implementation Plan v48

**Document Version:** 48.1
**Generated:** 2026-04-22
**Spec Source:** `iterations/iteration-48/spec_v48.md`
**Status:** Ready for Implementation

---

## 1. Implementation Strategy

### 1.1 Phase-Based Approach

Following the PRD's implementation order (Section 6), we proceed through 7 phases with P0 items first:

```
Phase 1: Core Infrastructure (P0 blockers)
Phase 2: Query & Storage (P0 blockers)
Phase 3: Logger Implementation (P0 blockers)
Phase 4: Security & Macros (P1 items)
Phase 5: Extended Structures (P1 items)
Phase 6: TUI Integration (P1 items)
Phase 7: Self-Troubleshooting (P2 items)
```

### 1.2 Crate Creation Strategy

The logging crate will be created as a standalone crate at `crates/logging/` with proper workspace integration.

**Key Dependencies:**
- `tracing` + `tracing-appender` + `tracing-subscriber` for structured logging
- `rusqlite` (bundled) for SQLite persistence
- `chrono` with `serde` for timestamps
- `serde` + `serde_json` for serialization
- `ratatui` (optional, gated by `tui` feature) for TUI components

---

## 2. Module Implementation Details

### 2.1 Phase 1: Core Infrastructure

#### FR-001 + FR-002 + FR-003: `event.rs`

**File:** `crates/logging/src/event.rs`

**Structures to implement:**

```rust
// LogLevel - with TRACE variant
pub enum LogLevel {
    Trace,
    Debug,
    Info,
    Warn,
    Error,
}

// LogFields - all optional fields + extra HashMap
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

// LogEvent - full structure
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

**Also in event.rs (Phase 5):**
- `ReasoningLog` + `ToolConsideration`
- `ToolExecutionLog` (references `SanitizedValue`)
- `ErrorContext` + `ErrorFrame` + `CauseInfo`

**Implementation Notes:**
- `LogLevel` needs `From<LogLevel> for tracing::Level` conversion
- `LogFields` needs `Default` implementation
- `LogEvent` needs `serde::Serialize` + `serde::Deserialize`

---

### 2.2 Phase 2: Query & Storage

#### FR-005: `query.rs`

**File:** `crates/logging/src/query.rs`

**LogQuery structure:**
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

**Filtering logic:** Implement `matches(&self, event: &LogEvent) -> bool`

---

#### FR-006: `store.rs` - SessionLogBuffer

**Ring Buffer Implementation:**
- O(1) insertion with oldest eviction
- `capacity: usize` configuration
- `push(event)` - adds event, evicts oldest if at capacity
- `get_range(from_seq, to_seq)` - range query
- `get_by_level(level)` - level filter

---

#### FR-007: `store.rs` - LogStore

**SQLite-backed persistent storage:**

```rust
pub struct LogStore {
    conn: Connection,
}

impl LogStore {
    pub fn append(&self, event: &LogEvent) -> Result<(), LogError>
    pub fn query(&self, criteria: LogQuery) -> Result<Vec<LogEvent>, LogError>
    pub fn recent(&self, session_id: &str, limit: usize) -> Result<Vec<LogEvent>, LogError>
    pub fn prune(&self, older_than: DateTime<Utc>) -> Result<u64, LogError>
}
```

**Schema:**
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

---

### 2.3 Phase 3: Logger Implementation

#### FR-004: `logger.rs` - AgentLogger Trait

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

pub struct ChildLogger { /* ... */ }
```

#### FR-015: `config.rs` - LoggingConfig

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

#### FR-017: Per-Component Log Level Filtering

**Glob pattern matching:**
- `agent` → exact match
- `llm.*` → prefix match with wildcard
- `tool.*` → prefix match with wildcard

**Filter resolution order:**
1. Exact target match
2. Glob pattern match (`*` matches any string)
3. Fall back to global level

#### FR-018: Size-Based Log Rotation

**Rotation parameters:**
- `max_file_size_mb` - rotate when exceeded
- `max_rotated_files` - keep N rotated files

**Rotation behavior:**
1. Check file size before write
2. If exceeded, rotate: `opencode.log` → `opencode.log.1` → `opencode.log.2`
3. Create new `opencode.log`
4. Delete oldest when `max_rotated_files` exceeded

---

### 2.4 Phase 4: Security & Macros

#### FR-011: `sanitizer.rs`

```rust
pub enum SanitizedValue {
    Safe(String),
    Redacted(String),
    Nested(HashMap<String, SanitizedValue>),
}
```

**Redaction rules (case-insensitive):**
| Pattern | Action |
|---------|--------|
| `api_key` | Redact |
| `password` | Redact |
| `token` | Redact |
| `secret` | Redact |
| `authorization` | Redact |
| `.*_key` suffix | Redact |
| `.*_token` suffix | Redact |
| `.*_secret` suffix | Redact |

#### FR-012: `log_tool!` macro

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

#### FR-013: `log_llm!` macro

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

#### FR-014: `log_fields!` macro

```rust
#[macro_export]
macro_rules! log_fields {
    // Field-only form: log_fields!(session_id, tool_name)
    ($($field:ident),*) => { ... };

    // Key-value form: log_fields!(latency_ms = 45, error_code = Some(...))
    ($($field:ident = $value:expr),*) => { ... };
}
```

---

### 2.5 Phase 5: Extended Structures (in `event.rs`)

#### FR-008: ReasoningLog

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

pub struct ToolConsideration {
    pub tool_name: String,
    pub reason: String,
    pub selected: bool,
}
```

#### FR-009: ToolExecutionLog

```rust
pub struct ToolExecutionLog {
    pub execution_id: String,
    pub session_id: String,
    pub tool_name: String,
    pub timestamp: DateTime<Utc>,
    pub parameters: SanitizedValue,
    pub result: ToolResult,  // ToolResult from opencode-core
    pub latency_ms: u64,
    pub error: Option<ErrorContext>,
}
```

#### FR-010: ErrorContext

```rust
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

pub struct CauseInfo {
    pub code: String,
    pub message: String,
}
```

---

### 2.6 Phase 6: TUI Integration

#### FR-016: TUI Log Panel

**Files:**
- `crates/logging/src/tui/mod.rs`
- `crates/logging/src/tui/log_panel.rs`
- `crates/logging/src/tui/log_renderer.rs`

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

**Layout:**
```
┌─────────────────────────────────────────────────────┐
│ Session: sess_abc123  │  Logs (23)  │  [Clear] [Filter] │
├─────────────────────────────────────────────────────┤
│ 10:30:45 INFO  agent    Session started              │
│ 10:30:46 DEBUG tool.read  Read 142 lines from foo.rs │
└─────────────────────────────────────────────────────┘
```

---

### 2.7 Phase 7: Self-Troubleshooting

#### FR-019: Log Query Tool

**File:** `crates/tools/src/log_query.rs`

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

**Implementation:** Implements the `Tool` trait from opencode-core.

---

## 3. File Structure

```
crates/logging/
├── Cargo.toml
├── src/
│   ├── lib.rs                    # Public exports
│   ├── logger.rs                 # Core Logger trait and implementation
│   ├── event.rs                  # LogEvent, LogFields, LogLevel, ReasoningLog, etc.
│   ├── store.rs                  # SessionLogBuffer and LogStore
│   ├── query.rs                  # LogQuery and filtering
│   ├── sanitizer.rs              # Sensitive data redaction
│   ├── config.rs                # LoggingConfig and TuiLogPosition
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

## 4. Implementation Order

| Phase | FR | Task | Priority |
|-------|-----|------|----------|
| 1 | - | Create `crates/logging/` crate with Cargo.toml | P0 |
| 1 | FR-003 | Implement `LogLevel` enum with TRACE | P0 |
| 1 | FR-002 | Implement `LogFields` struct | P0 |
| 1 | FR-001 | Implement `LogEvent` struct | P0 |
| 2 | FR-005 | Implement `LogQuery` struct | P0 |
| 2 | FR-006 | Implement `SessionLogBuffer` (ring buffer) | P0 |
| 2 | FR-007 | Implement `LogStore` (SQLite) | P0 |
| 3 | FR-004 | Implement `AgentLogger` trait | P0 |
| 3 | FR-015 | Implement `LoggingConfig` | P1 |
| 3 | FR-017 | Implement per-component log level filtering | P2 |
| 3 | FR-018 | Implement size-based log rotation | P2 |
| 4 | FR-011 | Implement `SanitizedValue` and sanitizer | P1 |
| 4 | FR-012 | Implement `log_tool!` macro | P1 |
| 4 | FR-013 | Implement `log_llm!` macro | P1 |
| 4 | FR-014 | Implement `log_fields!` macro | P2 |
| 5 | FR-008 | Implement `ReasoningLog` and `ToolConsideration` | P1 |
| 5 | FR-009 | Implement `ToolExecutionLog` | P1 |
| 5 | FR-010 | Implement `ErrorContext`, `ErrorFrame`, `CauseInfo` | P1 |
| 6 | FR-016 | Implement TUI Log Panel | P1 |
| 7 | FR-019 | Implement Log Query Tool | P2 |

---

## 5. Dependencies

```toml
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

## 6. Integration Points

### 6.1 Workspace Integration

Add to `Cargo.toml` at workspace root:
```toml
[workspace.dependencies]
opencode-logging = { path = "crates/logging" }
```

### 6.2 Existing Code Reuse

- `crates/util/src/logging.rs` - Extend existing `Logger` + `LogLevel` (without Trace)
- `crates/core/src/observability.rs` - Leverage `TokenUsage`, `SessionTrace` patterns
- `tracing` integration - Extend existing subscriber setup

### 6.3 Tool Integration (FR-019)

The `LogQueryTool` in `crates/tools/src/log_query.rs` will:
1. Implement the `Tool` trait from opencode-core
2. Use `AgentLogger::query()` to retrieve logs
3. Return structured log events matching query criteria

---

## 7. Testing Strategy

### 7.1 Unit Tests

| File | Coverage |
|------|----------|
| `logger_tests.rs` | LogEvent serialization, level filtering, query matching |
| `sanitizer_tests.rs` | Secret redaction for api_key, password, token fields |
| `query_tests.rs` | LogQuery filtering by session, level, target, time range |
| `tui_tests.rs` | Log panel rendering, filter behavior, empty state |

### 7.2 Integration Tests

| Test | Description |
|------|-------------|
| `test_log_store_query_by_session` | Insert logs and query by session_id |
| `test_log_rotation` | Write enough to trigger rotation, verify files |
| `test_reasoning_log_persistence` | Store and retrieve reasoning logs |
| `test_tool_execution_sanitization` | Verify sensitive params are redacted |

---

## 8. Acceptance Criteria Mapping

| Acceptance Criteria | FRs | Status |
|---------------------|-----|--------|
| LogEvent captures seq, timestamp, level, target, message, fields, span_id, parent_seq | FR-001 | Not started |
| Logger writes to both file and memory buffer | FR-006, FR-007 | Not started |
| Log file rotates when max size is reached | FR-018 | Not started |
| Per-component log level filtering works | FR-017 | Not started |
| Logs are queryable by session_id, level, target, time range | FR-005, FR-007 | Not started |
| Every LLM interaction logs prompt tokens, completion tokens, latency, model | FR-013 | Not started |
| Agent reasoning steps are logged with tool considerations and decisions | FR-008 | Not started |
| Reasoning logs are stored persistently and queryable | FR-007, FR-008 | Not started |
| Every tool call logs tool name, parameters (sanitized), latency, result/error | FR-009, FR-012 | Not started |
| Secret redaction works for: API keys, passwords, tokens, file paths with secrets | FR-011 | Not started |
| Tool errors include full context (file, line, session_id) | FR-010 | Not started |
| Errors capture code, message, stack trace, and cause chain | FR-010 | Not started |
| Errors are logged with ERROR level and queryable | FR-004, FR-005 | Not started |
| Error logs include recovery suggestions when available | FR-010 | Not started |
| Log panel toggles with Ctrl+L | FR-016 | Not started |
| Logs display with timestamp, level, target, message columns | FR-016 | Not started |
| Filtering by level and component works | FR-016 | Not started |
| Click on error shows full details in expandable view | FR-016 | Not started |
| Auto-scroll toggle available | FR-016 | Not started |
| Agent can query its own logs via tool call | FR-019 | Not started |
| Log queries support time range, session, level, component filters | FR-005, FR-019 | Not started |
| Full error context available for diagnosis | FR-010 | Not started |
| Actionable suggestions available from error codes | FR-010 | Not started |

---

## 9. Risk Mitigation

| Risk | Mitigation |
|------|------------|
| Ring buffer complexity | Use Vec with index wraparound for O(1) operations |
| SQLite schema migrations | Userusqlite migrations module for version tracking |
| TUI performance | Lazy rendering, virtual scrolling for large logs |
| Macro hygiene | Use `macro_rules!` with clear token matching |
| Secret redaction edge cases | Comprehensive regex patterns + test coverage |

---

## 10. Milestones

| Milestone | Description | Target |
|-----------|-------------|--------|
| M1 | Phase 1 complete: event.rs compiles with LogEvent, LogFields, LogLevel | Day 1 |
| M2 | Phase 2 complete: store.rs with ring buffer and SQLite store | Day 2 |
| M3 | Phase 3 complete: logger.rs with AgentLogger trait and config | Day 3 |
| M4 | Phase 4 complete: sanitizer and macros working | Day 4 |
| M5 | Phase 5 complete: Extended structures (ReasoningLog, etc.) | Day 5 |
| M6 | Phase 6 complete: TUI log panel functional | Day 6 |
| M7 | Phase 7 complete: Log query tool integrated | Day 7 |

---

*Plan generated from spec_v48.md. Implementation status tracked per task list.*