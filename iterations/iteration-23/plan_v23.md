# Implementation Plan - Iteration 23

**Project:** OpenCode Rust Port  
**Iteration:** 23  
**Date:** 2026-04-14  
**Status:** In Progress - PRD 20 (ratatui-testing) Implementation  
**Overall Completion:** ~93-96%

---

## Executive Summary

Implementation is approximately 93-96% complete. All PRDs except PRD 20 are fully implemented. PRD 20 (ratatui-testing framework) requires full implementation with 15 P0 blocking issues.

**Critical Gap:** `ratatui-testing` framework entirely in stub form (PRD 20 not implemented)

---

## Phase Status

| Phase | Description | Status | Completion |
|-------|-------------|--------|------------|
| Phase 0 | Project Foundation | ✅ Done | 100% |
| Phase 1 | Authority (Core/Config/Storage/Server) | ✅ Done | ~100% |
| Phase 2 | Runtime Core (Agent/Tools/Plugin/TUI Plugin) | ✅ Done | ~100% |
| Phase 3 | Infrastructure (MCP/LSP/Providers/Formatters/Skills/TUI) | ✅ Done | ~100% |
| Phase 4 | Interface (Desktop/Web/GitHub-GitLab) | ✅ Done | ~98% |
| Phase 5 | Hardening (Compatibility/Convention) | ✅ Done | ~98% |
| Phase 6 | Release Qualification | ❌ Not Started | ~0% |
| **PRD 20** | **ratatui-testing Framework** | **❌ In Progress** | **~5%** |

---

## Priority Classification

### P0 - Blocking Issues (15 issues - ALL in ratatui-testing)

| # | Issue | Module | Impact |
|---|-------|--------|--------|
| 1 | PtySimulator PTY master/slave creation not implemented | pty.rs | Blocks PTY functionality |
| 2 | PtySimulator `resize()` method missing | pty.rs | Blocks window resize testing |
| 3 | PtySimulator `inject_key_event()` method missing | pty.rs | Blocks keyboard input testing |
| 4 | PtySimulator `inject_mouse_event()` method missing | pty.rs | Blocks mouse input testing |
| 5 | PtySimulator `read_output()` lacks timeout | pty.rs | Blocks output timing tests |
| 6 | BufferDiff `DiffResult` and `CellDiff` structs missing | diff.rs | Blocks diff result reporting |
| 7 | BufferDiff ignore options missing | diff.rs | Blocks flexible diff testing |
| 8 | StateTester `capture()` method missing | state.rs | Blocks state snapshot testing |
| 9 | StateTester `assert_state_matches()` missing | state.rs | Blocks snapshot comparison |
| 10 | TestDsl PTY composition missing | dsl.rs | Blocks fluent test API |
| 11 | TestDsl `send_keys()`, `wait_for()`, `assert_buffer_eq()` missing | dsl.rs | Blocks event testing |
| 12 | CliTester process spawning not implemented | cli.rs | Blocks CLI testing |
| 13 | CliTester `CliOutput` struct missing | cli.rs | Blocks output capture |
| 14 | CliTester temp directory cleanup not implemented | cli.rs | Blocks test isolation |
| 15 | ratatui-testing tests/ directory empty | tests/ | Blocks test coverage |

### P1 - High Priority Issues (2 issues)

| # | Issue | Module | Impact |
|---|-------|--------|--------|
| 16 | Phase 6 Release Qualification not systematically started | all | Cannot release |
| 17 | `test_bedrock_credential_resolution_bearer_token_priority` fails | llm | Test reliability |

### P2 - Medium Priority Issues (2 issues)

| # | Issue | Module | Impact |
|---|-------|--------|--------|
| 18 | TestHarness unused helper methods | cli/tests | Code cleanliness |
| 19 | Multiple clippy warnings | multiple | Code quality |

---

## PRD 20 Implementation Roadmap

### Module: PtySimulator (`ratatui-testing/src/pty.rs`)

**Current Status:** 24 lines stub - imports `portable_pty` but creates no PTY

**Target Implementation:**
```rust
pub struct PtySimulator {
    master: Option<Box<dyn MasterPty>>,
    child: Option<Box<dyn Child>>,
    writer: Option<Box<dyn Write + Send>>,
    reader: Option<Box<dyn BufRead>>,
}
```

**Required Methods:**
1. `pub fn new(command: &[&str]) -> Result<Self>` - Create PtyPair, spawn child process
2. `pub fn resize(&mut self, cols: u16, rows: u16) -> Result<()>` - Resize PTY window
3. `pub fn write_input(&mut self, input: &str) -> Result<()>` - Write to PTY slave
4. `pub fn read_output(&mut self, timeout: Duration) -> Result<String>` - Read with timeout
5. `pub fn inject_key_event(&mut self, event: KeyEvent) -> Result<()>` - Inject via crossterm
6. `pub fn inject_mouse_event(&mut self, event: MouseEvent) -> Result<()>` - Inject via crossterm

**Dependencies:** `portable-pty`, `crossterm` (events, mouse features)

---

### Module: BufferDiff (`ratatui-testing/src/diff.rs`)

**Current Status:** 19 lines stub - returns empty string

**Target Implementation:**
```rust
pub struct BufferDiff {
    ignore_fg: bool,
    ignore_bg: bool,
    ignore_attributes: bool,
}

pub struct DiffResult {
    pub passed: bool,
    pub expected: Buffer,
    pub actual: Buffer,
    pub differences: Vec<CellDiff>,
}

pub struct CellDiff {
    pub x: u16,
    pub y: u16,
    pub expected: Cell,
    pub actual: Cell,
}
```

**Required Methods:**
1. `pub fn new() -> Self` - Create new BufferDiff
2. `pub fn ignore_fg(mut self, ignore: bool) -> Self` - Builder method
3. `pub fn ignore_bg(mut self, ignore: bool) -> Self` - Builder method
4. `pub fn ignore_attributes(mut self, ignore: bool) -> Self` - Builder method
5. `pub fn diff(&self, expected: &str, actual: &str) -> DiffResult` - Compare buffers
6. `pub fn diff_str(&self, expected: &str, actual: &str) -> DiffResult` - Alias for diff

**Dependencies:** `ratatui` for `Buffer` and `Cell` types

---

### Module: StateTester (`ratatui-testing/src/state.rs`)

**Current Status:** 22 lines stub - no capture method

**Target Implementation:**
```rust
pub struct StateTester {
    snapshot: Option<Value>,
    captured: Vec<Value>,
}
```

**Required Methods:**
1. `pub fn new() -> Self` - Create new StateTester
2. `pub fn capture<S>(&mut self, state: &S) -> Result<()>` - Capture serializable state to JSON
3. `pub fn assert_state<S>(&self, state: &S) -> Result<()>` - Compare to captured snapshot
4. `pub fn assert_state_matches(&self, expected: &Value) -> Result<()>` - Compare to expected JSON

**Dependencies:** `serde`, `serde_json`

---

### Module: TestDsl (`ratatui-testing/src/dsl.rs`)

**Current Status:** 19 lines stub - no PTY composition

**Target Implementation:**
```rust
pub struct TestDsl {
    pty: Option<PtySimulator>,
    buffer_diff: BufferDiff,
    state_tester: StateTester,
}
```

**Required Methods:**
1. `pub fn new() -> Self` - Create new TestDsl
2. `pub fn with_pty(mut self, cmd: &[&str]) -> Result<Self>` - Create and configure PTY
3. `pub fn pty_mut(&mut self) -> Option<&mut PtySimulator>` - Mutable PTY access
4. `pub fn render(&self, widget: &impl Widget) -> Result<Buffer>` - Render widget to Buffer
5. `pub fn assert_buffer_eq(&self, expected: &Buffer, actual: &Buffer) -> Result<()>`
6. `pub fn send_keys(&mut self, keys: &str) -> Result<&mut Self>` - Send keys to PTY
7. `pub fn wait_for<F>(&mut self, timeout: Duration, predicate: F) -> Result<&mut Self>`
8. `pub fn capture_state<S>(&mut self, state: &S) -> &mut Self`
9. `pub fn assert_state<S: serde::Serialize>(&self, state: &S) -> Result<()>`

**Dependencies:** `ratatui` for `Widget` and `Buffer` traits

---

### Module: CliTester (`ratatui-testing/src/cli.rs`)

**Current Status:** 19 lines stub - returns empty string

**Target Implementation:**
```rust
pub struct CliTester {
    temp_dir: Option<tempfile::TempDir>,
}

pub struct CliOutput {
    pub exit_code: i32,
    pub stdout: String,
    pub stderr: String,
}
```

**Required Methods:**
1. `pub fn new() -> Self` - Create new CliTester
2. `pub fn with_temp_dir(mut self) -> Result<Self>` - Create temp directory
3. `pub fn run(&self, args: &[&str]) -> Result<CliOutput>` - Spawn process, capture output
4. `pub fn capture_stdout(&mut self) -> &mut Self` - Enable stdout capture
5. `pub fn capture_stderr(&mut self) -> &mut Self` - Enable stderr capture

**Dependencies:** `tempfile`, `std::process::Command` or `assert_cmd`

---

## Integration Tests Plan

Create the following test files in `ratatui-testing/tests/`:

| File | Purpose | Test Cases |
|------|---------|------------|
| `pty_tests.rs` | PTY functionality | read/write/resize/inject tests |
| `buffer_diff_tests.rs` | Buffer comparison | cell-by-cell diff, ignore options |
| `state_tests.rs` | State testing | capture, assert, mismatches |
| `dsl_tests.rs` | Fluent API | compose PTY, BufferDiff, StateTester |
| `cli_tests.rs` | CLI spawning | process args, stdout/stderr, exit code |
| `integration_tests.rs` | Full workflow | end-to-end test scenarios |

---

## File Structure

```
ratatui-testing/
├── Cargo.toml
├── src/
│   ├── lib.rs
│   ├── pty.rs          # PtySimulator implementation
│   ├── diff.rs         # BufferDiff implementation
│   ├── state.rs        # StateTester implementation
│   ├── dsl.rs          # TestDsl implementation
│   ├── cli.rs          # CliTester implementation
│   └── snapshot.rs    # Snapshot management (future)
└── tests/
    ├── pty_tests.rs
    ├── buffer_diff_tests.rs
    ├── state_tests.rs
    ├── dsl_tests.rs
    ├── cli_tests.rs
    └── integration_tests.rs
```

---

## Dependencies

```toml
[dependencies]
ratatui = "0.28"
crossterm = { version = "0.28", features = ["events", "mouse"] }
portable-pty = "0.8"
anyhow = "1.0"
thiserror = "2.0"
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
tempfile = "3.14"
tokio = { version = "1.45", features = ["full"] }

[dev-dependencies]
similar-asserts = "1.5"
```

---

## Implementation Sequence

### Phase 1: PtySimulator (P0)
1. Implement `new()` with portable-pty PtyPair creation
2. Add `resize()` using master.resize()
3. Implement `write_input()` using writer
4. Implement `read_output()` with Duration timeout
5. Implement `inject_key_event()` using crossterm execute!
6. Implement `inject_mouse_event()` using crossterm execute!

### Phase 2: BufferDiff (P0)
1. Define `DiffResult` and `CellDiff` structs
2. Add ignore options (ignore_fg, ignore_bg, ignore_attributes)
3. Implement `diff()` with cell-by-cell comparison
4. Implement `diff_str()` as alias
5. Add Display impl for human-readable diff output

### Phase 3: StateTester (P0)
1. Add snapshot and captured fields
2. Implement `capture()` with serde::Serialize
3. Implement `assert_state()` comparing to snapshot
4. Implement `assert_state_matches()` comparing JSON

### Phase 4: TestDsl (P0)
1. Add PtySimulator, BufferDiff, StateTester composition
2. Implement `with_pty()` and `pty_mut()`
3. Implement `render()` with ratatui Widget
4. Implement `assert_buffer_eq()`
5. Implement `send_keys()` using PTY
6. Implement `wait_for()` with predicate
7. Implement `capture_state()` and `assert_state()`

### Phase 5: CliTester (P0)
1. Define `CliOutput` struct
2. Add temp_dir field
3. Implement `with_temp_dir()`
4. Implement `run()` spawning process with Command
5. Implement stdout/stderr capture methods

### Phase 6: Integration Tests (P0)
1. Create `tests/pty_tests.rs`
2. Create `tests/buffer_diff_tests.rs`
3. Create `tests/state_tests.rs`
4. Create `tests/dsl_tests.rs`
5. Create `tests/cli_tests.rs`
6. Create `tests/integration_tests.rs`

### Phase 7: Phase 6 Start (P1)
1. Begin end-to-end integration tests
2. Performance benchmarking
3. Security audit preparation
4. Observability validation

---

## Risk Assessment

| Risk | Probability | Impact | Mitigation |
|------|-------------|--------|------------|
| portable-pty API changes | Low | Medium | Pin version, check compatibility |
| crossterm event format changes | Low | Medium | Pin version, use feature flags |
| Ratatui Buffer API instability | Medium | Medium | Use stable ratatui version |
| Test flakiness on CI | Medium | High | Use timeouts, retry logic |

---

## Success Criteria

1. All 6 ratatui-testing modules compile without errors
2. PtySimulator creates functional PTY master/slave pair
3. BufferDiff correctly identifies cell-by-cell differences
4. StateTester captures and compares JSON snapshots
5. TestDsl composes all components fluently
6. CliTester spawns processes and captures output
7. Integration tests cover all major functionality
8. `cargo test -p ratatui-testing` passes
9. `cargo clippy -p ratatui-testing` passes with no warnings

---

*Plan generated: 2026-04-14*
*Iteration: 23*
*Priority: Implement PRD 20 (ratatui-testing) per specifications*
