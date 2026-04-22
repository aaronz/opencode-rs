# Logging System Task List v48

**Document Version:** 48.1
**Generated:** 2026-04-22
**Spec Source:** `iterations/iteration-48/spec_v48.md`
**Priority:** P0 items first, then P1, then P2

---

## P0 Tasks (Core Infrastructure - Must Complete First)

### Task 0.1: Create Logging Crate Structure
**FR:** N/A
**File:** `crates/logging/`
**Status:** Not started
**Priority:** P0

**Subtasks:**
- [ ] Create directory structure `crates/logging/src/` and `crates/logging/tests/`
- [ ] Create `crates/logging/Cargo.toml` with all dependencies
- [ ] Create `crates/logging/src/lib.rs` with module declarations
- [ ] Add crate to workspace `Cargo.toml`

**Verification:**
- `cargo build -p opencode-logging` succeeds
- No compiler warnings

---

### Task 0.2: Implement LogLevel Enum with TRACE
**FR:** FR-003
**File:** `crates/logging/src/event.rs`
**Status:** Not started
**Priority:** P0

**Subtasks:**
- [ ] Create `LogLevel` enum with variants: `Trace`, `Debug`, `Info`, `Warn`, `Error`
- [ ] Implement `From<LogLevel> for tracing::Level` conversion
- [ ] Implement `serde::Serialize` and `serde::Deserialize`
- [ ] Add unit tests for level ordering

**Verification:**
- `cargo test -p opencode-logging log_level` passes
- All level variants serialize/deserialize correctly

---

### Task 0.3: Implement LogFields Struct
**FR:** FR-002
**File:** `crates/logging/src/event.rs`
**Status:** ✅ Done
**Priority:** P0

**Subtasks:**
- [x] Create `LogFields` struct with all optional fields:
  - `session_id: Option<String>`
  - `tool_name: Option<String>`
  - `latency_ms: Option<u64>`
  - `model: Option<String>`
  - `provider: Option<String>`
  - `token_count: Option<u64>`
  - `error_code: Option<String>`
  - `file_path: Option<String>`
  - `line: Option<u32>`
  - `extra: HashMap<String, serde_json::Value>`
- [x] Implement `Default` for LogFields
- [x] Implement `serde::Serialize` and `serde::Deserialize`

**Verification:**
- `cargo test -p opencode-logging log_fields` passes ✅
- Default construction works ✅

---

### Task 0.4: Implement LogEvent Struct
**FR:** FR-001
**File:** `crates/logging/src/event.rs`
**Status:** Not started
**Priority:** P0

**Subtasks:**
- [ ] Create `LogEvent` struct with fields:
  - `seq: u64` - Unique sequence number
  - `timestamp: DateTime<Utc>` - High-precision timestamp
  - `level: LogLevel` - Log severity level
  - `target: String` - Target component
  - `message: String` - Human-readable message
  - `fields: LogFields` - Structured fields
  - `span_id: Option<String>` - Span context
  - `parent_seq: Option<u64>` - Parent log event ID
- [ ] Implement `serde::Serialize` and `serde::Deserialize`
- [ ] Add constructor `LogEvent::new(level, target, message, fields)`
- [ ] Add sequence number generation (atomic counter)

**Verification:**
- `cargo test -p opencode-logging log_event` passes
- LogEvent serializes/deserializes correctly
- Sequence numbers are unique and incrementing

---

### Task 0.5: Implement LogQuery Struct
**FR:** FR-005
**File:** `crates/logging/src/query.rs`
**Status:** Not started
**Priority:** P0

**Subtasks:**
- [ ] Create `LogQuery` struct with fields:
  - `session_id: Option<String>`
  - `level: Option<LogLevel>`
  - `target: Option<String>`
  - `since: Option<DateTime<Utc>>`
  - `until: Option<DateTime<Utc>>`
  - `limit: Option<usize>`
- [ ] Implement `LogQuery::matches(&self, event: &LogEvent) -> bool`
- [ ] Implement `Default` for LogQuery
- [ ] Add helper methods: `LogQuery::for_session()`, `LogQuery::for_level()`

**Verification:**
- `cargo test -p opencode-logging log_query` passes
- Query filtering works for all field combinations

---

### Task 0.6: Implement SessionLogBuffer (Ring Buffer)
**FR:** FR-006
**File:** `crates/logging/src/store.rs`
**Status:** ✅ Done
**Priority:** P0

**Subtasks:**
- [x] Create `SessionLogBuffer` struct with `capacity: usize`
- [x] Implement `push(&mut self, event: LogEvent)` - O(1) insert with oldest eviction
- [x] Implement `get_range(&self, from_seq: u64, to_seq: u64) -> Vec<&LogEvent>`
- [x] Implement `get_by_level(&self, level: LogLevel) -> Vec<&LogEvent>`
- [x] Implement `len()` and `is_empty()` methods
- [x] Add tests for ring buffer behavior (eviction, wraparound)

**Verification:**
- `cargo test -p opencode-logging session_log_buffer` passes ✅
- O(1) insertion confirmed ✅
- Oldest event evicted when capacity exceeded ✅

---

### Task 0.7: Implement LogStore (SQLite Persistence)
**FR:** FR-007
**File:** `crates/logging/src/store.rs`
**Status:** Not started
**Priority:** P0

**Subtasks:**
- [ ] Create `LogStore` struct with `conn: Connection`
- [ ] Implement `LogStore::new(path: &Path) -> Result<Self, LogError>`
- [ ] Implement database schema creation (logs table + indexes)
- [ ] Implement `append(&self, event: &LogEvent) -> Result<(), LogError>`
- [ ] Implement `query(&self, criteria: LogQuery) -> Result<Vec<LogEvent>, LogError>`
- [ ] Implement `recent(&self, session_id: &str, limit: usize) -> Result<Vec<LogEvent>, LogError>`
- [ ] Implement `prune(&self, older_than: DateTime<Utc>) -> Result<u64, LogError>`

**Verification:**
- `cargo test -p opencode-logging log_store` passes
- Insert and query operations work correctly
- Index performance verified

---

### Task 0.8: Implement AgentLogger Trait
**FR:** FR-004
**File:** `crates/logging/src/logger.rs`
**Status:** Not started
**Priority:** P0

**Subtasks:**
- [ ] Define `AgentLogger` trait with methods:
  - `fn trace(&self, target: &str, message: &str, fields: LogFields)`
  - `fn debug(&self, target: &str, message: &str, fields: LogFields)`
  - `fn info(&self, target: &str, message: &str, fields: LogFields)`
  - `fn warn(&self, target: &str, message: &str, fields: LogFields)`
  - `fn error(&self, target: &str, message: &str, fields: LogFields)`
  - `fn with_context(&self, context: LogFields) -> ChildLogger`
  - `async fn query(&self, criteria: LogQuery) -> Result<Vec<LogEvent>, LogError>`
- [ ] Create `ChildLogger` struct for context chaining
- [ ] Create `AgentLoggerImpl` struct with implementations

**Verification:**
- `cargo test -p opencode-logging agent_logger` passes
- All log levels work correctly
- Context chaining works properly

---

## P1 Tasks (Security, Macros, Extended Structures)

### Task 1.1: Implement LoggingConfig
**FR:** FR-015
**File:** `crates/logging/src/config.rs`
**Status:** Not started
**Priority:** P1

**Subtasks:**
- [ ] Create `LoggingConfig` struct:
  - `level: LogLevel`
  - `targets: HashMap<String, LogLevel>`
  - `file_path: Option<PathBuf>`
  - `max_file_size_mb: usize`
  - `max_rotated_files: usize`
  - `show_in_tui: bool`
  - `tui_position: TuiLogPosition`
  - `memory_buffer_size: usize`
  - `retention_days: u32`
- [ ] Create `TuiLogPosition` enum with `Bottom`, `Right`, `Overlay`
- [ ] Implement `Default` with sensible defaults
- [ ] Implement TOML deserialization

**Verification:**
- `cargo test -p opencode-logging logging_config` passes
- Default configuration works
- TOML parsing works

---

### Task 1.2: Implement SanitizedValue for Secret Redaction
**FR:** FR-011
**File:** `crates/logging/src/sanitizer.rs`
**Status:** Not started
**Priority:** P1

**Subtasks:**
- [ ] Create `SanitizedValue` enum:
  - `Safe(String)`
  - `Redacted(String)`
  - `Nested(HashMap<String, SanitizedValue>)`
- [ ] Create `Sanitizer` struct with redaction rules
- [ ] Implement `sanitize_value(&self, key: &str, value: &serde_json::Value) -> SanitizedValue`
- [ ] Implement redaction for patterns:
  - Exact: `api_key`, `password`, `token`, `secret`, `authorization`
  - Suffix: `*_key`, `*_token`, `*_secret`
- [ ] Implement `sanitize_parameters(&self, params: &serde_json::Value) -> SanitizedValue`

**Verification:**
- `cargo test -p opencode-logging sanitizer` passes
- All secret patterns redacted correctly
- Nested structures handled properly

---

### Task 1.3: Implement log_tool! Macro
**FR:** FR-012
**File:** `crates/logging/src/macros.rs`
**Status:** Not started
**Priority:** P1

**Subtasks:**
- [ ] Implement `log_tool!` macro with signature:
  ```rust
  log_tool!($logger:expr, $tool:expr, $status:expr, $($field:tt)*)
  ```
- [ ] Macro expands to `$logger.info(&format!("tool.{}", $tool), &format!("Tool {} completed", $status), log_fields!(...))`
- [ ] Support field syntax: `latency_ms = 45`, `session_id`, etc.
- [ ] Add usage documentation

**Verification:**
- `cargo test -p opencode-logging log_tool_macro` passes
- Macro expands correctly
- All field types work

---

### Task 1.4: Implement log_llm! Macro
**FR:** FR-013
**File:** `crates/logging/src/macros.rs`
**Status:** Not started
**Priority:** P1

**Subtasks:**
- [ ] Implement `log_llm!` macro with signature:
  ```rust
  log_llm!($logger:expr, $provider:expr, $model:expr, $tokens:expr, $latency:expr, $status:expr)
  ```
- [ ] Macro expands to call to $logger.info with target `llm.$provider`
- [ ] Sets provider, model, token_count, latency_ms fields

**Verification:**
- `cargo test -p opencode-logging log_llm_macro` passes
- Macro expands correctly
- Fields populated correctly

---

### Task 1.5: Implement log_fields! Macro
**FR:** FR-014
**File:** `crates/logging/src/macros.rs`
**Status:** Not started
**Priority:** P2

**Subtasks:**
- [ ] Implement `log_fields!` macro with two forms:
  - Field-only: `log_fields!(session_id, tool_name)`
  - Key-value: `log_fields!(latency_ms = 45, error_code = Some(...))`
- [ ] Support all LogFields optional fields
- [ ] Support expression values (not just literals)

**Verification:**
- `cargo test -p opencode-logging log_fields_macro` passes
- Both forms work correctly
- Expression evaluation works

---

### Task 1.6: Implement ReasoningLog and ToolConsideration
**FR:** FR-008
**File:** `crates/logging/src/event.rs`
**Status:** Not started
**Priority:** P1

**Subtasks:**
- [ ] Create `ToolConsideration` struct:
  - `tool_name: String`
  - `reason: String`
  - `selected: bool`
- [ ] Create `ReasoningLog` struct:
  - `step_id: String`
  - `session_id: String`
  - `timestamp: DateTime<Utc>`
  - `prompt: String`
  - `response: String`
  - `tools_considered: Vec<ToolConsideration>`
  - `decision: String`
  - `prompt_tokens: u64`
  - `completion_tokens: u64`
  - `latency_ms: u64`
- [ ] Implement `serde::Serialize` and `serde::Deserialize`

**Verification:**
- `cargo test -p opencode-logging reasoning_log` passes
- Serialization/deserialization works

---

### Task 1.7: Implement ToolExecutionLog
**FR:** FR-009
**File:** `crates/logging/src/event.rs`
**Status:** Not started
**Priority:** P1

**Subtasks:**
- [ ] Create `ToolExecutionLog` struct:
  - `execution_id: String`
  - `session_id: String`
  - `tool_name: String`
  - `timestamp: DateTime<Utc>`
  - `parameters: SanitizedValue`
  - `result: ToolResult` (from opencode-core)
  - `latency_ms: u64`
  - `error: Option<ErrorContext>`
- [ ] Implement `serde::Serialize` and `serde::Deserialize`

**Verification:**
- `cargo test -p opencode-logging tool_execution_log` passes
- Uses SanitizedValue for parameters

---

### Task 1.8: Implement ErrorContext, ErrorFrame, CauseInfo
**FR:** FR-010
**File:** `crates/logging/src/event.rs`
**Status:** Not started
**Priority:** P1

**Subtasks:**
- [ ] Create `ErrorFrame` struct:
  - `file: String`
  - `line: u32`
  - `function: String`
- [ ] Create `CauseInfo` struct:
  - `code: String`
  - `message: String`
- [ ] Create `ErrorContext` struct:
  - `code: String`
  - `message: String`
  - `stack: Vec<ErrorFrame>`
  - `cause_chain: Vec<CauseInfo>`
  - `context: HashMap<String, String>`
- [ ] Implement `serde::Serialize` and `serde::Deserialize`

**Verification:**
- `cargo test -p opencode-logging error_context` passes
- Stack trace captured correctly
- Cause chain captured correctly

---

### Task 1.9: Implement TUI Log Panel
**FR:** FR-016
**Files:** `crates/logging/src/tui/mod.rs`, `crates/logging/src/tui/log_panel.rs`, `crates/logging/src/tui/log_renderer.rs`
**Status:** Not started
**Priority:** P1

**Subtasks:**
- [ ] Create `tui/mod.rs` with module exports
- [ ] Create `LogPanel` widget implementing ratatui `Widget` trait
- [ ] Implement layout:
  ```
  ┌─────────────────────────────────────────────────────┐
  │ Session: sess_abc123  │  Logs (23)  │  [Clear] [Filter] │
  ├─────────────────────────────────────────────────────┤
  │ 10:30:45 INFO  agent    Session started              │
  └─────────────────────────────────────────────────────┘
  ```
- [ ] Implement Ctrl+L toggle visibility
- [ ] Implement arrow key scroll navigation
- [ ] Implement level filter buttons (INFO, DEBUG, WARN, ERROR)
- [ ] Implement component filter (agent, tool.*, llm.*)
- [ ] Implement text search within logs
- [ ] Implement expandable error details on click
- [ ] Implement auto-scroll toggle

**Verification:**
- `cargo test -p opencode-logging --features tui log_panel` passes
- All keybindings work
- Rendering correct

---

## P2 Tasks (Advanced Features)

### Task 2.1: Implement Per-Component Log Level Filtering
**FR:** FR-017
**File:** `crates/logging/src/logger.rs`
**Status:** Not started
**Priority:** P2

**Subtasks:**
- [ ] Implement target glob matching:
  - Exact match: `agent` matches `agent`
  - Wildcard prefix: `llm.*` matches `llm.openai`
  - Wildcard suffix: `*.read` matches `tool.read`
- [ ] Implement filter resolution order:
  1. Check exact target match
  2. Check glob pattern match
  3. Fall back to global level
- [ ] Add configuration for per-component levels

**Verification:**
- `cargo test -p opencode-logging per_component_filter` passes
- Glob patterns work correctly
- Priority resolution correct

---

### Task 2.2: Implement Size-Based Log Rotation
**FR:** FR-018
**File:** `crates/logging/src/logger.rs`
**Status:** Not started
**Priority:** P2

**Subtasks:**
- [ ] Implement file size check before write
- [ ] Implement rotation:
  - `opencode.log` → `opencode.log.1` → `opencode.log.2`
  - Increment all rotated file numbers
- [ ] Create new `opencode.log` after rotation
- [ ] Delete oldest when `max_rotated_files` exceeded
- [ ] Add configuration for `max_file_size_mb` and `max_rotated_files`

**Verification:**
- `cargo test -p opencode-logging log_rotation` passes
- Rotation triggered at correct size
- Oldest files deleted correctly

---

### Task 2.3: Implement Log Query Tool for Agent Self-Diagnosis
**FR:** FR-019
**File:** `crates/tools/src/log_query.rs`
**Status:** Not started
**Priority:** P2

**Subtasks:**
- [ ] Create `LogQueryTool` struct implementing `Tool` trait
- [ ] Define tool schema:
  - `session_id: Option<String>`
  - `level: Option<String>`
  - `target: Option<String>`
  - `since: Option<String>` (ISO 8601)
  - `until: Option<String>` (ISO 8601)
  - `limit: Option<usize>`
- [ ] Implement `execute(&self, params: Value, logger: &dyn AgentLogger) -> Result<Value, ToolError>`
- [ ] Return structured log events matching query criteria

**Verification:**
- `cargo test -p opencode-tools log_query_tool` passes
- Tool executes correctly
- Results formatted properly

---

## Testing Tasks

### Task T.1: Logger Tests
**File:** `crates/logging/tests/logger_tests.rs`
**Status:** Not started

**Subtasks:**
- [ ] Test LogEvent serialization
- [ ] Test level filtering
- [ ] Test query matching
- [ ] Test ChildLogger context chaining

---

### Task T.2: Sanitizer Tests
**File:** `crates/logging/tests/sanitizer_tests.rs`
**Status:** Not started

**Subtasks:**
- [ ] Test api_key redaction
- [ ] Test password redaction
- [ ] Test token redaction
- [ ] Test *_key suffix redaction
- [ ] Test *_token suffix redaction
- [ ] Test *_secret suffix redaction
- [ ] Test authorization redaction
- [ ] Test nested structures

---

### Task T.3: Query Tests
**File:** `crates/logging/tests/query_tests.rs`
**Status:** Not started

**Subtasks:**
- [ ] Test query by session_id
- [ ] Test query by level
- [ ] Test query by target
- [ ] Test query by time range
- [ ] Test combined queries
- [ ] Test limit parameter

---

### Task T.4: TUI Tests
**File:** `crates/logging/tests/tui_tests.rs`
**Status:** Not started

**Subtasks:**
- [ ] Test log panel rendering
- [ ] Test empty state rendering
- [ ] Test filter behavior
- [ ] Test scroll behavior

---

## Integration Tests

### Task I.1: Log Store Query by Session
**Status:** Not started
**Subtasks:**
- [ ] Insert multiple logs with different session_ids
- [ ] Query by session_id
- [ ] Verify only matching logs returned

---

### Task I.2: Log Rotation
**Status:** Not started
**Subtasks:**
- [ ] Configure small max_file_size_mb
- [ ] Write logs until rotation triggered
- [ ] Verify rotated files created
- [ ] Verify oldest deleted when max_rotated_files exceeded

---

### Task I.3: Reasoning Log Persistence
**Status:** Not started
**Subtasks:**
- [ ] Create ReasoningLog entry
- [ ] Store in LogStore
- [ ] Query and retrieve
- [ ] Verify fields preserved

---

### Task I.4: Tool Execution Sanitization
**Status:** Not started
**Subtasks:**
- [ ] Execute tool with parameters containing secrets
- [ ] Verify parameters sanitized in log
- [ ] Verify secret values redacted

---

## Completion Checklist

- [ ] All P0 tasks completed and verified
- [ ] All P1 tasks completed and verified
- [ ] All P2 tasks completed and verified
- [ ] All unit tests pass
- [ ] All integration tests pass
- [ ] Code formatted with `cargo fmt`
- [ ] No clippy warnings (`cargo clippy -D warnings`)
- [ ] Documentation updated

---

*Task list generated from spec_v48.md. Priority order: P0 → P1 → P2.*