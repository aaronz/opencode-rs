# Specification: ratatui-testing v0.1.0

**Project:** ratatui-testing
**Version:** 0.1.0
**Date:** 2026-04-17
**Status:** Implementation Complete (98%)

---

## Overview

`ratatui-testing` is a TUI testing framework for Rust applications built on `ratatui`. It provides infrastructure for automated testing of terminal user interfaces including PTY simulation, buffer diffing, event injection, and snapshot testing.

---

## Architecture

### Core Components

| Module | Status | Description |
|--------|--------|-------------|
| `PtySimulator` | ✅ Complete | PTY simulation for Unix platforms |
| `BufferDiff` | ✅ Complete | Cell-by-cell buffer comparison |
| `StateTester` | ✅ Complete | JSON-based state verification |
| `TestDsl` | ✅ Complete | Fluent test composition |
| `CliTester` | ✅ Complete | CLI process testing |
| `DialogRenderTester` | ✅ Complete | Dialog rendering helpers |
| `Snapshot` | ✅ Complete | Buffer snapshot persistence |

---

## Feature Requirements

### FR-PTY-001: PTY Simulation
**Module:** `pty.rs`

#### Description
Creates pseudo-terminals for injecting keyboard/mouse input and capturing terminal output.

#### Public API

```rust
pub struct PtySimulator { /* ... */ }

impl PtySimulator {
    /// Creates a new PTY with default bash command
    pub fn new() -> Result<Self>;

    /// Creates a new PTY with custom command
    pub fn new_with_command(command: &[&str]) -> Result<Self>;

    /// Write string input to PTY slave
    pub fn write_input(&mut self, input: &str) -> Result<()>;

    /// Read output from PTY master with timeout
    pub fn read_output(&mut self, timeout: Duration) -> Result<String>;

    /// Resize PTY window (cols/rows)
    pub fn resize(&mut self, cols: u16, rows: u16) -> Result<()>;

    /// Inject KeyEvent via crossterm encoding
    pub fn inject_key_event(&mut self, event: KeyEvent) -> Result<()>;

    /// Inject MouseEvent via crossterm encoding
    pub fn inject_mouse_event(&mut self, event: MouseEvent) -> Result<()>;

    /// Check if child process is still running
    pub fn is_child_running(&self) -> bool;
}
```

#### Cross-Platform Support
- **Unix:** Full implementation using `portable-pty`
- **Windows:** Stub implementation with descriptive error messages

#### Acceptance Criteria
- [x] Creates PTY master/slave pair on Unix
- [x] Writes strings to PTY slave
- [x] Reads output from PTY master with timeout
- [x] Resizes PTY window (cols/rows)
- [x] Injects KeyEvent via crossterm
- [x] Injects MouseEvent via crossterm
- [x] Cross-platform (Unix primary, Windows best-effort)

#### Known Issues
- **[P1]** `read_output()` breaks after first successful read instead of draining buffer (line 130 in `src/pty.rs`)

---

### FR-DIFF-001: Buffer Diffing
**Module:** `diff.rs`

#### Description
Compares ratatui `Buffer` output to detect rendering differences with configurable ignore options.

#### Public API

```rust
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct IgnoreOptions {
    pub ignore_foreground: bool,
    pub ignore_background: bool,
    pub ignore_attributes: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub struct CellDiff {
    pub x: u16,
    pub y: u16,
    pub expected: Cell,
    pub actual: Cell,
}

#[derive(Debug, Clone, PartialEq)]
pub struct DiffResult {
    pub passed: bool,
    pub expected: Buffer,
    pub actual: Buffer,
    pub differences: Vec<CellDiff>,
    pub total_diffs: usize,
}

pub struct BufferDiff {
    options: IgnoreOptions,
}

impl BufferDiff {
    pub fn new() -> Self;
    pub fn with_options(options: IgnoreOptions) -> Self;
    pub fn ignore_foreground(mut self) -> Self;
    pub fn ignore_background(mut self) -> Self;
    pub fn ignore_attributes(mut self) -> Self;
    pub fn diff(&self, expected: &Buffer, actual: &Buffer) -> DiffResult;
    pub fn diff_str(&self, expected: &str, actual: &str) -> DiffResult;
    pub fn diff_to_string(&self, expected: &Buffer, actual: &Buffer) -> String;
}
```

#### CellDiff Helper Methods
```rust
impl CellDiff {
    pub fn expected_symbol(&self) -> &str;
    pub fn actual_symbol(&self) -> &str;
    pub fn expected_foreground(&self) -> Color;
    pub fn actual_foreground(&self) -> Color;
    pub fn expected_background(&self) -> Color;
    pub fn actual_background(&self) -> Color;
    pub fn expected_modifier(&self) -> Modifier;
    pub fn actual_modifier(&self) -> Modifier;
    pub fn symbol(&self) -> (&str, &str);
    pub fn foreground(&self) -> (Color, Color);
    pub fn background(&self) -> (Color, Color);
    pub fn modifier(&self) -> (Modifier, Modifier);
}
```

#### Acceptance Criteria
- [x] Compares two Buffers cell-by-cell
- [x] Reports exact x,y of differences
- [x] Supports ignoring foreground color
- [x] Supports ignoring background color
- [x] Supports ignoring attributes (bold, italic, etc.)
- [x] Provides human-readable diff output (Display impl)

---

### FR-STATE-001: State Testing
**Module:** `state.rs`

#### Description
Verifies application state transitions based on events using JSON serialization.

#### Public API

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TerminalState {
    pub width: u16,
    pub height: u16,
    pub content: Vec<String>,
    pub cursor_x: Option<u16>,
    pub cursor_y: Option<u16>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StateSnapshot {
    pub json: Value,
    pub path: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StateDiffEntry {
    pub path: String,
    pub diff_type: DiffType,
    pub expected: Option<Value>,
    pub actual: Option<Value>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum DiffType {
    Added,
    Removed,
    Modified,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StateDiff {
    pub differences: Vec<StateDiffEntry>,
    pub total_diffs: usize,
}

pub struct StateTester {
    snapshots: HashMap<String, StateSnapshot>,
    default_path: String,
}

impl StateTester {
    pub fn new() -> Self;
    pub fn with_default_path(mut self, path: impl Into<String>) -> Self;
    pub fn capture_state<S>(&mut self, state: &S, name: Option<&str>) -> Result<StateSnapshot>
    where S: serde::Serialize;
    pub fn capture_terminal_state(&mut self, buffer: &Buffer, cursor_x: Option<u16>, cursor_y: Option<uu16>, name: Option<&str>) -> Result<StateSnapshot>;
    pub fn get_snapshot(&self, name: &str) -> Option<&StateSnapshot>;
    pub fn list_snapshots(&self) -> Vec<&str>;
    pub fn compare(&self, current: &Value, snapshot: &StateSnapshot) -> Result<StateDiff>;
    pub fn compare_by_name(&self, current: &Value, name: &str) -> Result<StateDiff>;
    pub fn assert_state<S>(&self, state: &S) -> Result<()>
    where S: serde::Serialize;
    pub fn assert_state_named<S>(&self, state: &S, name: &str) -> Result<()>
    where S: serde::Serialize;
    pub fn assert_state_matches(&self, expected: &Value, actual: &Value) -> Result<()>;
    pub fn remove_snapshot(&mut self, name: &str) -> Option<StateSnapshot>;
    pub fn clear_snapshots(&mut self);
}
```

#### Acceptance Criteria
- [x] Captures serializable state to JSON
- [x] Compares current state to captured snapshot
- [x] Reports mismatches with JSON diff
- [x] TerminalState for buffer capture
- [x] Multiple snapshot management

---

### FR-DSL-001: Test DSL
**Module:** `dsl.rs`

#### Description
Fluent interface for composing test scenarios combining PTY, BufferDiff, StateTester, and rendering.

#### Public API

```rust
pub struct TestDsl {
    width: u16,
    height: u16,
    terminal: Option<Terminal<TestBackend>>,
    pty: Option<PtySimulator>,
    buffer_diff: Option<BufferDiff>,
    state_tester: Option<StateTester>,
    last_render: Option<Buffer>,
    predicates: Vec<WaitPredicate>,
}

impl TestDsl {
    pub fn new() -> Self;
    pub fn with_size(mut self, width: u16, height: u16) -> Self;
    pub fn init_terminal(mut self) -> Self;
    pub fn with_pty(mut self) -> Result<Self>;
    pub fn with_pty_command(mut self, command: &[&str]) -> Result<Self>;
    pub fn with_buffer_diff(mut self) -> Self;
    pub fn with_state_tester(mut self) -> Self;
    pub fn render(mut self, widget: impl Widget + 'static) -> Self;
    pub fn render_with_state<S, W, F>(self, state: &S, widget_fn: F) -> Self;
    pub fn capture_buffer(&self) -> Option<Buffer>;
    pub fn get_terminal(&self) -> Option<&Terminal<TestBackend>>;
    pub fn get_terminal_mut(&mut self) -> Option<&mut Terminal<TestBackend>>;
    pub fn get_pty(&self) -> Option<&PtySimulator>;
    pub fn get_pty_mut(&mut self) -> Option<&mut PtySimulator>;
    pub fn get_buffer_diff(&self) -> Option<&BufferDiff>;
    pub fn get_buffer_diff_mut(&mut self) -> Option<&mut BufferDiff>;
    pub fn get_state_tester(&self) -> Option<&StateTester>;
    pub fn get_state_tester_mut(&mut self) -> Option<&mut StateTester>;

    // Assertion methods
    pub fn assert_no_diffs(&self, expected: &Buffer) -> Result<()>;
    pub fn assert_buffer_matches(&self, expected: &Buffer, options: IgnoreOptions) -> Result<()>;
    pub fn assert_buffer_eq(&self, expected: &Buffer, actual: &Buffer) -> Result<()>;
    pub fn assert_state<S>(&self, state: &S) -> Result<()>
    where S: serde::Serialize;

    // PTY operations
    pub fn write_to_pty(&mut self, input: &str) -> Result<()>;
    pub fn send_keys(&mut self, keys: &str) -> Result<&mut Self>;
    pub fn read_from_pty(&mut self, timeout: Duration) -> Result<String>;
    pub fn resize_pty(&mut self, cols: u16, rows: u16) -> Result<()>;
    pub fn is_pty_child_running(&self) -> bool;
    pub fn assert_pty_running(&self) -> Result<()>;
    pub fn assert_pty_stopped(&self) -> Result<()>;

    // State operations
    pub fn capture_state<S>(&mut self, state: &S, name: Option<&str>) -> Result<()>
    where S: serde::Serialize;
    pub fn snapshot_state(&mut self, name: &str) -> Result<()>;
    pub fn compare_to_snapshot(&self, name: &str) -> Result<()>;

    // Wait/polling operations
    pub fn wait_for<F>(mut self, timeout: Duration, predicate: F) -> Result<Self>
    where F: Fn() -> bool + Send + 'static;
    pub fn wait_for_async<F, Fut>(self, timeout: Duration, predicate: F) -> Result<Self>
    where F: Fn() -> Fut + Send + 'static, Fut: Future<Output = bool> + Send;
    pub fn wait_with_predicates(mut self, timeout: Duration) -> Result<Self>;
    pub fn poll_until<F>(self, timeout: Duration, condition: F) -> Result<Self>
    where F: FnMut() -> bool;
    pub fn poll_until_async<F, Fut>(self, timeout: Duration, condition: F) -> Result<Self>
    where F: FnMut() -> Fut + Send + 'static, Fut: Future<Output = bool> + Send + 'static;

    // Chaining operations
    pub fn then<F>(self, f: F) -> Self
    where F: FnOnce(Self) -> Self;
    pub fn then_result<F>(self, f: F) -> Result<Self>
    where F: FnOnce(Self) -> Result<Self>;
    pub fn add_predicate(mut self, predicate: WaitPredicate) -> Self;

    // Buffer inspection
    pub fn buffer_content_at(&self, x: u16, y: u16) -> Option<String>;
    pub fn buffer_line_at(&self, y: u16) -> Option<String>;
    pub fn buffer_lines(&self) -> Option<Vec<String>>;

    // Snapshot operations
    pub fn save_snapshot(&mut self, name: &str) -> Result<&mut Self>;
    pub fn load_snapshot(&self, name: &str) -> Result<Buffer>;
    pub fn load_snapshot_and_assert_eq(&self, name: &str) -> Result<()>;
}

pub struct WaitPredicate { /* ... */ }

impl WaitPredicate {
    pub fn new<F>(description: impl Into<String>, check_fn: F) -> Self
    where F: Fn() -> bool + Send + 'static;
    pub fn from_buffer_content<F>(description: impl Into<String>, check_fn: F) -> Self
    where F: Fn(Option<&Buffer>) -> bool + Send + 'static;
    pub fn check(&self) -> bool;
    pub fn description(&self) -> String;
}
```

#### Acceptance Criteria
- [x] Renders widget to Buffer
- [x] Composes PTY, BufferDiff, StateTester
- [x] Fluent API chains correctly
- [x] Wait-for predicate support
- [x] Async wait/polling variants
- [x] Multiple predicate support
- [x] Buffer content inspection helpers
- [x] Snapshot save/load integration

---

### FR-CLI-001: CLI Testing
**Module:** `cli.rs`

#### Description
Tests CLI entry points and argument parsing with process management.

#### Public API

```rust
pub struct CliTester {
    command: String,
    args: Vec<String>,
    env_vars: HashMap<String, String>,
    working_dir: Option<PathBuf>,
    temp_dir: Option<TempDir>,
    capture_stdout: bool,
    capture_stderr: bool,
}

impl CliTester {
    pub fn new(command: &str) -> Self;
    pub fn arg(mut self, arg: &str) -> Self;
    pub fn args(mut self, args: &[&str]) -> Self;
    pub fn env(mut self, key: &str, value: &str) -> Self;
    pub fn envs(mut self, vars: HashMap<&str, &str>) -> Self;
    pub fn working_dir(mut self, dir: PathBuf) -> Self;
    pub fn with_temp_dir(self) -> Result<(Self, PathBuf)>;
    pub fn capture_stdout(mut self) -> Self;
    pub fn capture_stderr(mut self) -> Self;
    pub async fn run(&self) -> Result<CliOutput>;
    pub async fn run_with_timeout(&self, timeout: Duration) -> Result<CliOutput>;
    pub async fn spawn(&self) -> Result<ChildProcess>;
}

pub struct ChildProcess {
    inner: tokio::process::Child,
}

impl ChildProcess {
    pub async fn wait(mut self) -> Result<CliOutput>;
    pub async fn kill(&mut self) -> Result<()>;
}

#[derive(Debug, Clone)]
pub struct CliOutput {
    pub stdout: String,
    pub stderr: String,
    pub exit_code: i32,
    pub success: bool,
}

impl CliOutput {
    pub fn assert_success(&self) -> Result<()>;
    pub fn assert_exit_code(&self, expected: i32) -> Result<()>;
    pub fn assert_stdout_contains(&self, expected: &str) -> Result<()>;
    pub fn assert_stderr_contains(&self, expected: &str) -> Result<()>;
}
```

#### Acceptance Criteria
- [x] Spawns process with args
- [x] Captures stdout/stderr
- [x] Returns exit code
- [x] Cleans up temp directories (via TempDir RAII)
- [x] Non-blocking spawn() for process management
- [x] Fluent assertion methods

---

### FR-DIALOG-001: Dialog Render Testing
**Module:** `dialog_tester.rs`

#### Description
Helper utilities for testing TUI dialog rendering.

#### Public API

```rust
pub struct DialogRenderTester;

impl DialogRenderTester {
    pub fn new() -> Self;
    pub fn with_backend(width: u16, height: u16) -> TestBackend;
    pub fn terminal(width: u16, height: u16) -> Terminal<TestBackend>;
    pub fn has_border(buffer: &Buffer) -> bool;
    pub fn has_content(buffer: &Buffer) -> bool;
    pub fn count_lines_with_content(buffer: &Buffer) -> usize;
    pub fn has_title(buffer: &Buffer, title: &str) -> bool;
    pub fn has_specific_content(buffer: &Buffer, content: &str) -> bool;
}
```

#### Acceptance Criteria
- [x] Creates TestBackend for rendering
- [x] Border detection helpers
- [x] Content presence checks
- [x] Title matching
- [x] Content search

---

### FR-SNAP-001: Snapshot Management
**Module:** `snapshot.rs`

#### Description
Persists ratatui Buffers to disk for regression testing.

#### Public API

```rust
/// Load a snapshot Buffer from disk
pub fn load_snapshot(name: &str) -> Result<Buffer>;

/// Save a Buffer snapshot to disk
pub fn save_snapshot(name: &str, buffer: &Buffer) -> Result<()>;
```

#### Implementation Details
- Snapshots stored as JSON in `snapshots/` directory (configurable via `RATATUI_TESTING_SNAPSHOT_DIR`)
- Serializes: area, cells (symbol, fg, bg, modifier_bits)
- Sanitizes snapshot names (replaces `/`, `\`, `..` with `_`)

#### Acceptance Criteria
- [x] Saves Buffer to JSON file
- [x] Loads Buffer from JSON file
- [x] Configurable snapshot directory via env var
- [x] Preserves cell styling (fg, bg, modifiers)

---

## Dependencies

```toml
[dependencies]
ratatui = "0.28"
crossterm = "0.28"
anyhow = "1.0"
thiserror = "2.0"
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
portable-pty = "0.8"
tokio = { version = "1.45", features = ["rt-multi-thread", "sync", "time", "macros", "process", "io-util"] }
tracing = "0.1"
tempfile = "3.14"

[dev-dependencies]
tokio = { version = "1.45", features = ["full"] }
```

---

## File Structure

```
ratatui-testing/
├── Cargo.toml
├── src/
│   ├── lib.rs              # Public exports
│   ├── pty.rs              # PtySimulator implementation
│   ├── diff.rs             # BufferDiff implementation
│   ├── state.rs            # StateTester implementation
│   ├── dsl.rs              # TestDsl implementation
│   ├── cli.rs              # CliTester implementation
│   ├── dialog_tester.rs    # DialogRenderTester implementation
│   └── snapshot.rs         # Snapshot persistence
└── tests/
    ├── pty_tests.rs
    ├── buffer_diff_tests.rs
    ├── state_tests.rs
    ├── dsl_tests.rs
    ├── dsl_integration_tests.rs
    ├── dialog_tests.rs
    ├── snapshot_tests.rs
    └── integration_tests.rs
```

---

## Gap Analysis Summary (2026-04-17)

### P0 Issues (Blocking)
**None identified.**

### P1 Issues (High Priority)

| ID | Issue | Location | Impact |
|----|-------|----------|--------|
| GAP-001 | PtySimulator `read_output()` breaks after first read, may miss buffered data | `src/pty.rs:130` | Tests may miss output arriving in multiple chunks |

**Code Issue:**
```rust
Ok(n) => {
    buffer.extend_from_slice(&temp_buf[..n]);
    break;  // <-- Premature break
}
```

### P2 Issues (Medium Priority)

| ID | Issue | Location | Impact |
|----|-------|----------|--------|
| GAP-002 | Missing `similar-asserts` dev-dependency | `Cargo.toml` | Dev workflow mismatch with PRD |
| GAP-003 | DialogTester not documented in PRD | `dialog_tester.rs` | Documentation gap |
| GAP-004 | Dead code warnings (`#[allow(dead_code)]`) | `dialog_tester.rs:92,100` | Code cleanliness |

---

## Technical Debt

| ID | Description | Estimated Effort |
|----|-------------|------------------|
| TD-001 | Fix PtySimulator read loop to drain buffer completely | 15 min |
| TD-002 | Add `similar-asserts = "1.5"` to dev-dependencies | 1 min |
| TD-003 | Review DialogTester - document or remove | 30 min |
| TD-004 | Remove or use `#[allow(dead_code)]` functions | 5 min |

---

## Test Coverage Summary

| Module | Unit Tests | Integration Tests |
|--------|------------|-------------------|
| PtySimulator | 21 | 3 |
| BufferDiff | 50+ | 4 |
| StateTester | 20+ | 4 |
| TestDsl | 50+ | 3 |
| CliTester | 22 | 5 |
| DialogRenderTester | 15 | - |
| Snapshot | 6 | - |

---

## Cross-References

| Document | Topic |
|----------|-------|
| [TUI System](./09-tui-system.md) | TUI layout, keybindings, views |
| [TUI Plugin API](./15-tui-plugin-api.md) | TUI plugin configuration |
| [Rust Test Implementation Roadmap](./17-rust-test-implementation-roadmap.md) | Overall testing strategy |
| [Crate-by-Crate Test Backlog](./18-crate-by-crate-test-backlog.md) | Testing tasks per crate |

---

## Acceptance Criteria Summary

### Overall Completion: 98%

| Category | Progress |
|----------|----------|
| Core Modules | 100% |
| API Surface | 100% |
| Test Coverage | 95% |
| Documentation | 85% |
| Cross-platform | 90% |

### Acceptance Criteria Status

| Module | Status |
|--------|--------|
| PtySimulator | ✅ Complete (1 P1 issue) |
| BufferDiff | ✅ Complete |
| StateTester | ✅ Complete |
| TestDsl | ✅ Complete |
| CliTester | ✅ Complete |
| DialogRenderTester | ✅ Complete |
| Snapshot | ✅ Complete |
| Integration | ✅ Complete |

---

*Document generated: 2026-04-17*
*Specification version: 31*
