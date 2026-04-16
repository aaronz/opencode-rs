# ratatui-testing Specification v2.5

## Overview

`ratatui-testing` is a TUI testing framework for Rust applications built on `ratatui`. It provides infrastructure for automated testing of terminal user interfaces including PTY simulation, buffer diffing, event injection, and snapshot testing.

---

## 1. Module Architecture

### 1.1 Core Modules

| Module | File | Description |
|--------|------|-------------|
| `PtySimulator` | `src/pty.rs` | PTY master/slave pair management, input injection |
| `BufferDiff` | `src/diff.rs` | Cell-by-cell buffer comparison with ignore options |
| `StateTester` | `src/state.rs` | Application state capture and comparison |
| `TestDsl` | `src/dsl.rs` | Fluent test composition API |
| `CliTester` | `src/cli.rs` | CLI process spawning and output capture |
| `Snapshot` | `src/snapshot.rs` | Buffer snapshot save/load functionality |
| `DialogRenderTester` | `src/dialog_tester.rs` | Dialog rendering verification helpers |

### 1.2 Public API (lib.rs)

```rust
pub use cli::{CliOutput, CliTester};
pub use dialog_tester::{assert_render_result, DialogRenderTester};
pub use diff::{BufferDiff, CellDiff, DiffResult, IgnoreOptions};
pub use dsl::{TestDsl, WaitPredicate};
pub use pty::PtySimulator;
pub use snapshot::{load_snapshot, save_snapshot};
pub use state::{DiffType, StateDiff, StateDiffEntry, StateSnapshot, StateTester, TerminalState};
```

---

## 2. Component Specifications

### 2.1 PtySimulator (FR-PTY-001)

**Purpose**: Pseudo-terminal simulation for injecting keyboard/mouse input and capturing output.

**Location**: `src/pty.rs`

**Public API**:

```rust
pub struct PtySimulator { /* ... */ }

impl PtySimulator {
    pub fn new() -> Result<Self>;  // FR-PTY-001
    pub fn new_with_command(command: &[&str]) -> Result<Self>;  // FR-PTY-002
    pub fn resize(&mut self, cols: u16, rows: u16) -> Result<()>;  // FR-PTY-003
    pub fn write_input(&mut self, input: &str) -> Result<()>;  // FR-PTY-004
    pub fn read_output(&mut self, timeout: Duration) -> Result<String>;  // FR-PTY-005
    pub fn inject_key_event(&mut self, event: KeyEvent) -> Result<()>;  // FR-PTY-006
    pub fn inject_mouse_event(&mut self, event: MouseEvent) -> Result<()>;  // FR-PTY-007
    pub fn is_child_running(&self) -> bool;  // FR-PTY-008
}
```

**FR-PTY-001**: `new()` creates PTY with default command `["bash", "-c", "echo ready"]`

**FR-PTY-002**: `new_with_command(command: &[&str])` creates PTY with specified command

**FR-PTY-003**: `resize(cols, rows)` resizes PTY window via `MasterPty::resize()`

**FR-PTY-004**: `write_input(input)` writes string to PTY slave

**FR-PTY-005**: `read_output(timeout)` reads from PTY master with timeout

**FR-PTY-006**: `inject_key_event(event)` injects crossterm KeyEvent via UTF-8 escape sequences

**FR-PTY-007**: `inject_mouse_event(event)` injects crossterm MouseEvent via SGR protocol

**FR-PTY-008**: `is_child_running()` checks if child process is still running

**Dependencies**: `portable-pty`, `crossterm`

---

### 2.2 BufferDiff (FR-DIFF-001)

**Purpose**: Compare ratatui Buffer output to detect rendering differences.

**Location**: `src/diff.rs`

**Public API**:

```rust
pub struct IgnoreOptions {
    pub ignore_foreground: bool,
    pub ignore_background: bool,
    pub ignore_attributes: bool,
}

impl IgnoreOptions {
    pub fn ignore_foreground(mut self) -> Self;
    pub fn ignore_background(mut self) -> Self;
    pub fn ignore_attributes(mut self) -> Self;
}

pub struct CellDiff {
    pub x: u16,
    pub y: u16,
    pub expected: Cell,
    pub actual: Cell,
}

pub struct DiffResult {
    pub passed: bool,
    pub expected: Buffer,
    pub actual: Buffer,
    pub differences: Vec<CellDiff>,
    pub total_diffs: usize,
}

pub struct BufferDiff { /* ... */ }

impl BufferDiff {
    pub fn new() -> Self;  // FR-DIFF-001
    pub fn with_options(options: IgnoreOptions) -> Self;  // FR-DIFF-002
    pub fn ignore_foreground(mut self) -> Self;  // FR-DIFF-003
    pub fn ignore_background(mut self) -> Self;  // FR-DIFF-004
    pub fn ignore_attributes(mut self) -> Self;  // FR-DIFF-005
    pub fn diff(&self, expected: &Buffer, actual: &Buffer) -> DiffResult;  // FR-DIFF-006
    pub fn diff_str(&self, expected: &str, actual: &str) -> DiffResult;  // FR-DIFF-007
    pub fn diff_to_string(&self, expected: &Buffer, actual: &Buffer) -> String;  // FR-DIFF-008
}
```

**FR-DIFF-001**: `new()` creates BufferDiff with default IgnoreOptions

**FR-DIFF-002**: `with_options(options)` creates BufferDiff with specified IgnoreOptions

**FR-DIFF-003**: `ignore_foreground()` fluent method sets ignore_foreground=true

**FR-DIFF-004**: `ignore_background()` fluent method sets ignore_background=true

**FR-DIFF-005**: `ignore_attributes()` fluent method sets ignore_attributes=true

**FR-DIFF-006**: `diff(expected, actual)` compares two Buffers cell-by-cell

**FR-DIFF-007**: `diff_str(expected, actual)` parses strings to Buffers then diffs (Note: does not apply IgnoreOptions - see gap FR-DIFF-GAP-001)

**FR-DIFF-008**: `diff_to_string()` returns diff result as formatted string

**Dependencies**: `ratatui`

---

### 2.3 StateTester (FR-STATE-001)

**Purpose**: Verify application state after events are injected.

**Location**: `src/state.rs`

**Public API**:

```rust
pub struct TerminalState {
    pub width: u16,
    pub height: u16,
    pub content: Vec<String>,
    pub cursor_x: Option<u16>,
    pub cursor_y: Option<u16>,
}

pub struct StateSnapshot {
    pub json: Value,
    pub path: Vec<String>,
}

pub struct StateDiffEntry {
    pub path: String,
    pub diff_type: DiffType,
    pub expected: Option<Value>,
    pub actual: Option<Value>,
}

pub enum DiffType { Added, Removed, Modified }

pub struct StateDiff {
    pub differences: Vec<StateDiffEntry>,
    pub total_diffs: usize,
}

pub struct StateTester { /* ... */ }

impl StateTester {
    pub fn new() -> Self;  // FR-STATE-001
    pub fn with_default_path(mut self, path: impl Into<String>) -> Self;  // FR-STATE-002
    pub fn capture_state<S>(&mut self, state: &S, name: Option<&str>) -> Result<StateSnapshot>;  // FR-STATE-003
    pub fn capture_terminal_state(&mut self, buffer: &Buffer, cursor_x: Option<u16>, cursor_y: Option<u16>, name: Option<&str>) -> Result<StateSnapshot>;  // FR-STATE-004
    pub fn get_snapshot(&self, name: &str) -> Option<&StateSnapshot>;  // FR-STATE-005
    pub fn list_snapshots(&self) -> Vec<&str>;  // FR-STATE-006
    pub fn compare(&self, current: &Value, snapshot: &StateSnapshot) -> Result<StateDiff>;  // FR-STATE-007
    pub fn compare_by_name(&self, current: &Value, name: &str) -> Result<StateDiff>;  // FR-STATE-008
    pub fn assert_state<S>(&self, state: &S) -> Result<()>;  // FR-STATE-009
    pub fn assert_state_named<S>(&self, state: &S, name: &str) -> Result<()>;  // FR-STATE-010
    pub fn assert_state_matches(&self, expected: &Value, actual: &Value) -> Result<()>;  // FR-STATE-011
    pub fn diff_to_string(&self, current: &Value, snapshot: &StateSnapshot) -> Result<String>;  // FR-STATE-012
    pub fn remove_snapshot(&mut self, name: &str) -> Option<StateSnapshot>;  // FR-STATE-013
    pub fn clear_snapshots(&mut self);  // FR-STATE-014
}
```

**FR-STATE-001**: `new()` creates empty StateTester

**FR-STATE-002**: `with_default_path()` sets default snapshot name

**FR-STATE-003**: `capture_state()` serializes and stores state snapshot

**FR-STATE-004**: `capture_terminal_state()` captures TerminalState from Buffer

**FR-STATE-005**: `get_snapshot()` retrieves named snapshot

**FR-STATE-006**: `list_snapshots()` lists all snapshot names

**FR-STATE-007**: `compare()` compares current state to snapshot, returns StateDiff

**FR-STATE-008**: `compare_by_name()` compares current to named snapshot

**FR-STATE-009**: `assert_state()` asserts current state matches default snapshot

**FR-STATE-010**: `assert_state_named()` asserts current state matches named snapshot

**FR-STATE-011**: `assert_state_matches()` directly compares two Value objects

**FR-STATE-012**: `diff_to_string()` returns diff as formatted string

**FR-STATE-013**: `remove_snapshot()` removes named snapshot

**FR-STATE-014**: `clear_snapshots()` removes all snapshots

**Dependencies**: `serde`, `serde_json`

---

### 2.4 TestDsl (FR-DSL-001)

**Purpose**: Fluent interface for composing test scenarios.

**Location**: `src/dsl.rs`

**Public API**:

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

pub struct WaitPredicate { /* ... */ }

impl TestDsl {
    pub fn new() -> Self;  // FR-DSL-001
    pub fn with_size(mut self, width: u16, height: u16) -> Self;  // FR-DSL-002
    pub fn init_terminal(mut self) -> Self;  // FR-DSL-003
    pub fn with_pty(mut self, command: &[&str]) -> Result<Self>;  // FR-DSL-004
    pub fn with_buffer_diff(mut self) -> Self;  // FR-DSL-005
    pub fn with_state_tester(mut self) -> Self;  // FR-DSL-006
    pub fn render(mut self, widget: impl Widget + 'static) -> Self;  // FR-DSL-007
    pub fn render_with_state<S, W, F>(self, state: &S, widget_fn: F) -> Self;  // FR-DSL-008
    pub fn capture_buffer(&self) -> Option<Buffer>;  // FR-DSL-009
    pub fn get_terminal(&self) -> Option<&Terminal<TestBackend>>;  // FR-DSL-010
    pub fn get_terminal_mut(&mut self) -> Option<&mut Terminal<TestBackend>>;  // FR-DSL-011
    pub fn get_pty(&self) -> Option<&PtySimulator>;  // FR-DSL-012
    pub fn get_pty_mut(&mut self) -> Option<&mut PtySimulator>;  // FR-DSL-013
    pub fn get_buffer_diff(&self) -> Option<&BufferDiff>;  // FR-DSL-014
    pub fn get_state_tester(&self) -> Option<&StateTester>;  // FR-DSL-015
    pub fn add_predicate(mut self, predicate: WaitPredicate) -> Self;  // FR-DSL-016
    pub fn assert_no_diffs(&self, expected: &Buffer) -> Result<()>;  // FR-DSL-017
    pub fn assert_buffer_eq(&self, expected: &Buffer, actual: &Buffer) -> Result<()>;  // FR-DSL-018
    pub fn capture_state<S>(&mut self, state: &S, name: Option<&str>) -> Result<()>;  // FR-DSL-019
    pub fn assert_state<S>(&self, state: &S) -> Result<()>;  // FR-DSL-020
    pub fn send_keys(&mut self, keys: &str) -> Result<&mut Self>;  // FR-DSL-021
    pub fn wait_for<F>(self, timeout: Duration, predicate: F) -> Result<Self>;  // FR-DSL-022
    pub fn wait_with_predicates(mut self, timeout: Duration) -> Result<Self>;  // FR-DSL-023
    pub fn poll_until<F>(self, timeout: Duration, condition: F) -> Result<Self>;  // FR-DSL-024
    pub fn buffer_lines(&self) -> Option<Vec<String>>;  // FR-DSL-025
    pub fn save_snapshot(&mut self, name: &str) -> Result<&mut Self>;  // FR-DSL-026
    pub fn load_snapshot(&self, name: &str) -> Result<Buffer>;  // FR-DSL-027
    pub fn load_snapshot_and_assert_eq(&self, name: &str) -> Result<()>;  // FR-DSL-028
}
```

**FR-DSL-001**: `new()` creates TestDsl with default 80x30 size

**FR-DSL-002**: `with_size()` sets terminal dimensions

**FR-DSL-003**: `init_terminal()` initializes TestBackend terminal

**FR-DSL-004**: `with_pty(command)` initializes PTY with command (Note: requires command parameter - see gap FR-DSL-GAP-001)

**FR-DSL-005**: `with_buffer_diff()` initializes BufferDiff

**FR-DSL-006**: `with_state_tester()` initializes StateTester

**FR-DSL-007**: `render()` renders widget to buffer

**FR-DSL-008**: `render_with_state()` renders widget with state callback

**FR-DSL-009**: `capture_buffer()` returns last rendered buffer

**FR-DSL-010/011**: `get_terminal()` / `get_terminal_mut()` accessors

**FR-DSL-012/013**: `get_pty()` / `get_pty_mut()` accessors

**FR-DSL-014**: `get_buffer_diff()` accessor

**FR-DSL-015**: `get_state_tester()` accessor

**FR-DSL-016**: `add_predicate()` adds WaitPredicate for wait_with_predicates

**FR-DSL-017**: `assert_no_diffs()` asserts no diffs against expected buffer

**FR-DSL-018**: `assert_buffer_eq()` asserts two buffers are equal

**FR-DSL-019**: `capture_state()` captures state to StateTester

**FR-DSL-020**: `assert_state()` asserts state matches

**FR-DSL-021**: `send_keys()` parses key sequence and injects via PTY

**FR-DSL-022**: `wait_for()` waits for predicate with timeout

**FR-DSL-023**: `wait_with_predicates()` waits for all predicates

**FR-DSL-024**: `poll_until()` polls condition until true or timeout

**FR-DSL-025**: `buffer_lines()` extracts lines from last render

**FR-DSL-026**: `save_snapshot()` saves buffer to snapshot file

**FR-DSL-027**: `load_snapshot()` loads buffer from snapshot file

**FR-DSL-028**: `load_snapshot_and_assert_eq()` loads and asserts equality

**Dependencies**: `ratatui`, `crossterm`, `tokio`

---

### 2.5 CliTester (FR-CLI-001)

**Purpose**: Test CLI entry points and argument parsing.

**Location**: `src/cli.rs`

**Public API**:

```rust
pub struct CliTester {
    command: String,
    args: Vec<String>,
    env_vars: HashMap<String, String>,
    working_dir: Option<PathBuf>,
    temp_dir: Option<TempDir>,  // Note: not used in working_dir - see gap FR-CLI-GAP-001
}

pub struct ChildProcess {
    inner: tokio::process::Child,
}

pub struct CliOutput {
    pub stdout: String,
    pub stderr: String,
    pub exit_code: i32,
    pub success: bool,
}

impl CliTester {
    pub fn new(command: &str) -> Self;  // FR-CLI-001
    pub fn arg(mut self, arg: &str) -> Self;  // FR-CLI-002
    pub fn args(mut self, args: &[&str]) -> Self;  // FR-CLI-003
    pub fn env(mut self, key: &str, value: &str) -> Self;  // FR-CLI-004
    pub fn envs(mut self, vars: HashMap<&str, &str>) -> Self;  // FR-CLI-005
    pub fn working_dir(mut self, dir: PathBuf) -> Self;  // FR-CLI-006
    pub fn with_temp_dir(self) -> Result<(Self, PathBuf)>;  // FR-CLI-007
    pub async fn run(&self) -> Result<CliOutput>;  // FR-CLI-008
    pub async fn run_with_timeout(&self, timeout: Duration) -> Result<CliOutput>;  // FR-CLI-009
    pub async fn spawn(&self) -> Result<ChildProcess>;  // FR-CLI-010
}

impl CliOutput {
    pub fn assert_success(&self) -> Result<()>;  // FR-CLI-011
    pub fn assert_exit_code(&self, expected: i32) -> Result<()>;  // FR-CLI-012
    pub fn assert_stdout_contains(&self, expected: &str) -> Result<()>;  // FR-CLI-013
    pub fn assert_stderr_contains(&self, expected: &str) -> Result<()>;  // FR-CLI-014
}
```

**FR-CLI-001**: `new(command)` creates CliTester for command

**FR-CLI-002**: `arg()` adds single argument

**FR-CLI-003**: `args()` adds multiple arguments

**FR-CLI-004**: `env()` sets environment variable

**FR-CLI-005**: `envs()` sets multiple environment variables

**FR-CLI-006**: `working_dir()` sets working directory

**FR-CLI-007**: `with_temp_dir()` creates temp directory that is cleaned up on drop

**FR-CLI-008**: `run()` executes with 30s timeout, returns CliOutput

**FR-CLI-009**: `run_with_timeout()` executes with custom timeout

**FR-CLI-010**: `spawn()` spawns process without waiting

**FR-CLI-011**: `assert_success()` asserts exit code 0

**FR-CLI-012**: `assert_exit_code()` asserts specific exit code

**FR-CLI-013**: `assert_stdout_contains()` asserts stdout contains string

**FR-CLI-014**: `assert_stderr_contains()` asserts stderr contains string

**Note**: Missing `capture_stdout()` and `capture_stderr()` fluent methods per PRD - see gap FR-CLI-GAP-002

**Dependencies**: `tokio`, `tempfile`

---

### 2.6 Snapshot Module (FR-SNAP-001)

**Purpose**: Buffer snapshot save/load for regression testing.

**Location**: `src/snapshot.rs`

**Public API**:

```rust
pub fn load_snapshot(name: &str) -> Result<Buffer>;  // FR-SNAP-001
pub fn save_snapshot(name: &str, buffer: &Buffer) -> Result<()>;  // FR-SNAP-002
```

**FR-SNAP-001**: `load_snapshot(name)` loads buffer from `snapshots/{name}.json`

**FR-SNAP-002**: `save_snapshot(name, buffer)` saves buffer to `snapshots/{name}.json`

**Dependencies**: `ratatui`, `serde`

---

### 2.7 DialogRenderTester (FR-DIALOG-001)

**Purpose**: Dialog rendering verification helpers.

**Location**: `src/dialog_tester.rs`

**Public API**:

```rust
pub struct DialogRenderTester;

impl DialogRenderTester {
    pub fn new() -> Self;  // FR-DIALOG-001
    pub fn with_backend(width: u16, height: u16) -> TestBackend;  // FR-DIALOG-002
    pub fn terminal(width: u16, height: u16) -> Terminal<TestBackend>;  // FR-DIALOG-003
    pub fn has_border(buffer: &Buffer) -> bool;  // FR-DIALOG-004
    pub fn has_content(buffer: &Buffer) -> bool;  // FR-DIALOG-005
    pub fn count_lines_with_content(buffer: &Buffer) -> usize;  // FR-DIALOG-006
}

pub fn assert_render_result(buffer: &Buffer);  // FR-DIALOG-007
pub fn assert_empty_state(buffer: &Buffer);  // FR-DIALOG-008
```

**FR-DIALOG-001**: `new()` creates DialogRenderTester

**FR-DIALOG-002**: `with_backend()` creates TestBackend with dimensions

**FR-DIALOG-003**: `terminal()` creates Terminal with TestBackend

**FR-DIALOG-004**: `has_border()` checks if buffer has border characters (─ or │)

**FR-DIALOG-005**: `has_content()` checks if buffer has non-space content

**FR-DIALOG-006**: `count_lines_with_content()` counts lines with non-space content

**FR-DIALOG-007**: `assert_render_result()` asserts border and content present

**FR-DIALOG-008**: `assert_empty_state()` asserts border present even when empty

**Dependencies**: `ratatui`

---

## 3. Known Gaps and Issues

### 3.1 P0 - Critical

| Gap ID | Description | Impact | Reference |
|--------|-------------|--------|-----------|
| FR-CLI-GAP-002 | `CliTester` missing `capture_stdout()` and `capture_stderr()` fluent methods | Cannot configure stdout/stderr capture per PRD | CliTester |
| N/A | `tests/dsl_tests.rs` file is missing | Test coverage incomplete per PRD | tests/ |

### 3.2 P1 - High Priority

| Gap ID | Description | Impact | Reference |
|--------|-------------|--------|-----------|
| FR-DSL-GAP-001 | `TestDsl::with_pty()` requires command parameter | PRD shows `with_pty(self)` without params | TestDsl:66 |
| FR-DIFF-GAP-001 | `diff_str()` does not use IgnoreOptions | String comparison ignores builder options | BufferDiff:259 |
| FR-CLI-GAP-001 | `temp_dir` field not used in `working_dir` | Temp dir is never set as working directory | CliTester:16 |

### 3.3 P2 - Medium Priority

| Gap ID | Description | Impact | Reference |
|--------|-------------|--------|-----------|
| FR-DIALOG-GAP-001 | DialogRenderTester functionality is basic | Limited dialog verification methods | DialogRenderTester |
| N/A | `wait_for` methods don't use `predicates` field | Code redundancy | TestDsl:270-307 |
| N/A | No Windows platform detection for PTY | May fail on Windows without clear error | PtySimulator |
| N/A | `ChildProcess` not exported from lib.rs | Type not publicly accessible | lib.rs |
| N/A | `SNAPSHOT_DIR` is hardcoded to "snapshots" | Inflexible snapshot directory | snapshot.rs:9 |

---

## 4. File Structure

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
│   ├── snapshot.rs         # Snapshot management
│   └── dialog_tester.rs    # DialogRenderTester
├── tests/
│   ├── pty_tests.rs        # PtySimulator tests (11 tests)
│   ├── buffer_diff_tests.rs # BufferDiff tests (35+ tests)
│   ├── state_tests.rs      # StateTester tests (33 tests)
│   ├── dsl_tests.rs       # MISSING - required by PRD
│   ├── dsl_integration_tests.rs # DSL integration tests
│   └── integration_tests.rs # Overall integration tests
└── snapshots/              # Snapshot files directory
```

---

## 5. Dependencies

```toml
[dependencies]
ratatui = "0.28"
crossterm = { version = "0.28", optional = true }
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

[features]
default = ["crossterm"]
```

---

## 6. Acceptance Criteria Status

### PtySimulator
- [x] Creates PTY master/slave pair on Unix (FR-PTY-001, FR-PTY-002)
- [x] Writes strings to PTY slave (FR-PTY-004)
- [x] Reads output from PTY master with timeout (FR-PTY-005)
- [x] Resizes PTY window (FR-PTY-003)
- [x] Injects KeyEvent via crossterm (FR-PTY-006)
- [x] Injects MouseEvent via crossterm (FR-PTY-007)
- [ ] Cross-platform detection (P2 gap)

### BufferDiff
- [x] Compares two Buffers cell-by-cell (FR-DIFF-006)
- [x] Reports exact x,y of differences (FR-DIFF-006)
- [x] Supports ignoring foreground color (FR-DIFF-003)
- [x] Supports ignoring background color (FR-DIFF-004)
- [x] Supports ignoring attributes (FR-DIFF-005)
- [x] Provides human-readable diff output (FR-DIFF-008)
- [ ] `diff_str` supports IgnoreOptions (FR-DIFF-GAP-001)

### StateTester
- [x] Captures serializable state to JSON (FR-STATE-003)
- [x] Compares current state to captured snapshot (FR-STATE-007)
- [x] Reports mismatches with JSON diff (FR-STATE-012)

### TestDsl
- [x] Renders widget to Buffer (FR-DSL-007)
- [x] Composes PTY, BufferDiff, StateTester (FR-DSL-004, FR-DSL-005, FR-DSL-006)
- [x] Fluent API chains correctly (FR-DSL-001 to FR-DSL-028)
- [ ] `with_pty()` no-parameter version (FR-DSL-GAP-001)

### CliTester
- [x] Spawns process with args (FR-CLI-008)
- [x] Captures stdout/stderr (FR-CLI-008)
- [x] Returns exit code (FR-CLI-008)
- [x] Cleans up temp directories (FR-CLI-007)
- [ ] `capture_stdout()` and `capture_stderr()` methods (FR-CLI-GAP-002)

### CliTester temp_dir Fix Needed
- [ ] `temp_dir` field should be used in `working_dir` when no explicit working_dir set (FR-CLI-GAP-001)

### Integration
- [ ] All modules compile together
- [ ] Integration tests pass
- [ ] Works with `cargo test`
- [ ] Cross-platform (Unix primary, Windows best-effort)

---

## 7. Test Coverage

| Test File | Module | Test Count | Status |
|-----------|--------|------------|--------|
| `tests/pty_tests.rs` | PtySimulator | 11 | Complete |
| `tests/buffer_diff_tests.rs` | BufferDiff | 35+ | Complete |
| `tests/state_tests.rs` | StateTester | 33 | Complete |
| `tests/dsl_tests.rs` | TestDsl | 0 | **MISSING** |
| `tests/dsl_integration_tests.rs` | TestDsl | Multiple | Complete |
| `tests/integration_tests.rs` | All | 28 | Complete |

---

## 8. Revision History

| Version | Date | Changes |
|---------|------|---------|
| v2.5 | 2026-04-16 | Added gap analysis from iteration-25, added FR-XXX IDs |
| v2.0 | 2026-04-14 | Restructured with acceptance criteria |
| v1.0 | 2026-04-10 | Initial specification |

---

## 9. Cross-References

| Document | Topic |
|----------|-------|
| [TUI System](./09-tui-system.md) | TUI layout, keybindings, views |
| [TUI Plugin API](./15-tui-plugin-api.md) | TUI plugin configuration |
| [Rust Test Implementation Roadmap](./17-rust-test-implementation-roadmap.md) | Overall testing strategy |
| [Crate-by-Crate Test Backlog](./18-crate-by-crate-test-backlog.md) | Testing tasks per crate |
