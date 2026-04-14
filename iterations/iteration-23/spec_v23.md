# Specification Document - Iteration 23

**Project:** OpenCode Rust Port
**Iteration:** 23
**Date:** 2026-04-14
**Status:** In Progress - PRD 20 (ratatui-testing) Implementation

---

## Overview

This document updates the specification for the OpenCode Rust implementation based on gap analysis performed in iteration 23. The primary focus remains on implementing PRD 20: the `ratatui-testing` framework for TUI testing.

**Overall Implementation Status:** ~93-96% complete

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

## 1. PRD 20: ratatui-testing Framework Specification

### 1.1 Overview

`ratatui-testing` is a TUI testing framework for Rust applications built on `ratatui`. It provides infrastructure for automated testing of terminal user interfaces including PTY simulation, buffer diffing, event injection, and snapshot testing.

### 1.2 Module Specifications

#### 1.2.1 PtySimulator Module

**File:** `ratatui-testing/src/pty.rs`

**Purpose:** Wrapper around POSIX PTY (or cross-platform `portable-pty` crate) for PTY simulation.

**Struct Definition:**
```rust
pub struct PtySimulator {
    master: Option<Box<dyn MasterPty>>,
    child: Option<Box<dyn Child>>,
    writer: Option<Box<dyn Write + Send>>,
    reader: Option<Box<dyn BufRead>>,
}
```

**Required Methods:**

| Method | Signature | Description |
|--------|-----------|-------------|
| `new` | `pub fn new(command: &[&str]) -> Result<Self>` | Creates PTY master/slave pair, spawns child process with command |
| `resize` | `pub fn resize(&mut self, cols: u16, rows: u16) -> Result<()>` | Resizes PTY window |
| `write_input` | `pub fn write_input(&mut self, input: &str) -> Result<()>` | Writes string to PTY slave |
| `read_output` | `pub fn read_output(&mut self, timeout: Duration) -> Result<String>` | Reads output from PTY master with timeout |
| `inject_key_event` | `pub fn inject_key_event(&mut self, event: KeyEvent) -> Result<()>` | Injects KeyEvent via crossterm |
| `inject_mouse_event` | `pub fn inject_mouse_event(&mut self, event: MouseEvent) -> Result<()>` | Injects MouseEvent via crossterm |

**Dependencies:**
- `portable-pty` for cross-platform PTY
- `crossterm` for event types (`KeyEvent`, `MouseEvent`)

**Acceptance Criteria:**
- [ ] Creates PTY master/slave pair on Unix
- [ ] Writes strings to PTY slave
- [ ] Reads output from PTY master with timeout
- [ ] Resizes PTY window (cols/rows)
- [ ] Injects KeyEvent via crossterm
- [ ] Injects MouseEvent via crossterm

---

#### 1.2.2 BufferDiff Module

**File:** `ratatui-testing/src/diff.rs`

**Purpose:** Compares ratatui buffer output to detect rendering differences.

**Struct Definitions:**
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

| Method | Signature | Description |
|--------|-----------|-------------|
| `new` | `pub fn new() -> Self` | Creates new BufferDiff |
| `ignore_fg` | `pub fn ignore_fg(mut self, ignore: bool) -> Self` | Builder method for foreground ignore |
| `ignore_bg` | `pub fn ignore_bg(mut self, ignore: bool) -> Self` | Builder method for background ignore |
| `ignore_attributes` | `pub fn ignore_attributes(mut self, ignore: bool) -> Self` | Builder method for attributes ignore |
| `diff` | `pub fn diff(&self, expected: &str, actual: &str) -> DiffResult` | Compares two buffer strings |
| `diff_str` | `pub fn diff_str(&self, expected: &str, actual: &str) -> DiffResult` | Alias for diff |

**Dependencies:**
- `ratatui` for `Buffer` and `Cell` types

**Acceptance Criteria:**
- [ ] Compares two Buffers cell-by-cell
- [ ] Reports exact x,y of differences
- [ ] Supports ignoring foreground color
- [ ] Supports ignoring background color
- [ ] Supports ignoring attributes (bold, italic, etc.)
- [ ] Provides human-readable diff output

---

#### 1.2.3 StateTester Module

**File:** `ratatui-testing/src/state.rs`

**Purpose:** Verifies application state after events are injected.

**Struct Definition:**
```rust
pub struct StateTester {
    snapshot: Option<Value>,
    captured: Vec<Value>,
}
```

**Required Methods:**

| Method | Signature | Description |
|--------|-----------|-------------|
| `new` | `pub fn new() -> Self` | Creates new StateTester |
| `capture` | `pub fn capture<S>(&mut self, state: &S) -> Result<()>` | Captures serializable state to JSON |
| `assert_state` | `pub fn assert_state<S>(&self, state: &S) -> Result<()>` | Compares current state to captured snapshot |
| `assert_state_matches` | `pub fn assert_state_matches(&self, expected: &Value) -> Result<()>` | Compares current state to expected JSON |

**Dependencies:**
- `serde` / `serde_json` for serialization

**Acceptance Criteria:**
- [ ] Captures serializable state to JSON
- [ ] Compares current state to captured snapshot
- [ ] Reports mismatches with JSON diff

---

#### 1.2.4 TestDsl Module

**File:** `ratatui-testing/src/dsl.rs`

**Purpose:** Fluent interface for composing test scenarios.

**Struct Definition:**
```rust
pub struct TestDsl {
    pty: Option<PtySimulator>,
    buffer_diff: BufferDiff,
    state_tester: StateTester,
}
```

**Required Methods:**

| Method | Signature | Description |
|--------|-----------|-------------|
| `new` | `pub fn new() -> Self` | Creates new TestDsl |
| `with_pty` | `pub fn with_pty(mut self, cmd: &[&str]) -> Result<Self>` | Creates and configures PTY |
| `pty_mut` | `pub fn pty_mut(&mut self) -> Option<&mut PtySimulator>` | Mutable access to PTY |
| `render` | `pub fn render(&self, widget: &impl Widget) -> Result<Buffer>` | Renders widget to Buffer |
| `assert_buffer_eq` | `pub fn assert_buffer_eq(&self, expected: &Buffer, actual: &Buffer) -> Result<()>` | Asserts buffer equality |
| `send_keys` | `pub fn send_keys(&mut self, keys: &str) -> Result<&mut Self>` | Sends keys to PTY |
| `wait_for` | `pub fn wait_for<F>(&mut self, timeout: Duration, predicate: F) -> Result<&mut Self>` | Waits for predicate |
| `capture_state` | `pub fn capture_state<S>(&mut self, state: &S) -> &mut Self` | Captures application state |
| `assert_state` | `pub fn assert_state<S: serde::Serialize>(&self, state: &S) -> Result<()>` | Asserts state matches |

**Dependencies:**
- `ratatui` for `Widget` and `Buffer` traits
- `PTYSimulator`, `BufferDiff`, `StateTester`

**Acceptance Criteria:**
- [ ] Renders widget to Buffer
- [ ] Composes PTY, BufferDiff, StateTester
- [ ] Fluent API chains correctly
- [ ] Wait-for predicate support

---

#### 1.2.5 CliTester Module

**File:** `ratatui-testing/src/cli.rs`

**Purpose:** Tests CLI entry points and argument parsing.

**Struct Definitions:**
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

| Method | Signature | Description |
|--------|-----------|-------------|
| `new` | `pub fn new() -> Self` | Creates new CliTester |
| `with_temp_dir` | `pub fn with_temp_dir(mut self) -> Result<Self>` | Creates temp directory for test isolation |
| `run` | `pub fn run(&self, args: &[&str]) -> Result<CliOutput>` | Spawns process with args, captures output |
| `capture_stdout` | `pub fn capture_stdout(&mut self) -> &mut Self` | Enables stdout capture |
| `capture_stderr` | `pub fn capture_stderr(&mut self) -> &mut Self` | Enables stderr capture |

**Dependencies:**
- `tempfile` for temp directory management
- `assert_cmd` or `std::process::Command` for process spawning

**Acceptance Criteria:**
- [ ] Spawns process with args
- [ ] Captures stdout/stderr
- [ ] Returns exit code
- [ ] Cleans up temp directories

---

### 1.3 File Structure

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
│   └── snapshot.rs     # Snapshot management
└── tests/
    ├── pty_tests.rs
    ├── buffer_diff_tests.rs
    ├── state_tests.rs
    ├── dsl_tests.rs
    └── integration_tests.rs
```

### 1.4 Dependencies

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

## 2. Feature Requirements

### 2.1 PRD 20 Feature Requirements (FR-121 to FR-140)

| FR | Feature | Module | Priority | Status |
|----|---------|--------|----------|--------|
| FR-121 | PtySimulator creates PTY master/slave pair | pty.rs | P0 | ❌ NOT STARTED |
| FR-122 | PtySimulator writes strings to PTY slave | pty.rs | P0 | ❌ NOT STARTED |
| FR-123 | PtySimulator reads output from PTY master with timeout | pty.rs | P0 | ❌ NOT STARTED |
| FR-124 | PtySimulator resizes PTY window (cols/rows) | pty.rs | P0 | ❌ NOT STARTED |
| FR-125 | PtySimulator injects KeyEvent via crossterm | pty.rs | P0 | ❌ NOT STARTED |
| FR-126 | PtySimulator injects MouseEvent via crossterm | pty.rs | P0 | ❌ NOT STARTED |
| FR-127 | BufferDiff compares two Buffers cell-by-cell | diff.rs | P0 | ❌ NOT STARTED |
| FR-128 | BufferDiff reports exact x,y of differences | diff.rs | P0 | ❌ NOT STARTED |
| FR-129 | BufferDiff supports ignoring foreground color | diff.rs | P0 | ❌ NOT STARTED |
| FR-130 | BufferDiff supports ignoring background color | diff.rs | P0 | ❌ NOT STARTED |
| FR-131 | BufferDiff supports ignoring attributes | diff.rs | P0 | ❌ NOT STARTED |
| FR-132 | BufferDiff provides human-readable diff output | diff.rs | P0 | ❌ NOT STARTED |
| FR-133 | StateTester captures serializable state to JSON | state.rs | P0 | ❌ NOT STARTED |
| FR-134 | StateTester compares current state to captured snapshot | state.rs | P0 | ❌ NOT STARTED |
| FR-135 | StateTester reports mismatches with JSON diff | state.rs | P0 | ❌ NOT STARTED |
| FR-136 | TestDsl renders widget to Buffer | dsl.rs | P0 | ❌ NOT STARTED |
| FR-137 | TestDsl composes PTY, BufferDiff, StateTester | dsl.rs | P0 | ❌ NOT STARTED |
| FR-138 | TestDsl provides fluent API with wait-for predicate | dsl.rs | P0 | ❌ NOT STARTED |
| FR-139 | CliTester spawns process, captures stdout/stderr, returns exit code | cli.rs | P0 | ❌ NOT STARTED |
| FR-140 | CliTester cleans up temp directories | cli.rs | P0 | ❌ NOT STARTED |

---

## 3. Gap Analysis Summary

### 3.1 Current Stub Status

| Module | Current Lines | Gap | Missing |
|--------|---------------|-----|---------|
| pty.rs | 24 lines | Full implementation needed | PTY creation, resize, inject_key_event, inject_mouse_event |
| diff.rs | 19 lines | DiffResult, CellDiff structs missing | Cell-by-cell comparison, ignore options |
| state.rs | 22 lines | capture() method missing | Snapshot storage, assert_state_matches |
| dsl.rs | 19 lines | PTY composition missing | Fluent API, wait_for, send_keys |
| cli.rs | 19 lines | Process spawning missing | CliOutput struct, temp dir cleanup |
| tests/ | Empty | No test files | 5+ test files needed |

### 3.2 P0 Blocking Issues

| Issue | Module | Impact |
|-------|--------|--------|
| PtySimulator PTY master/slave creation not implemented | pty.rs | Blocks PTY functionality |
| PtySimulator resize() method missing | pty.rs | Blocks window resize testing |
| PtySimulator inject_key_event() method missing | pty.rs | Blocks keyboard input testing |
| PtySimulator inject_mouse_event() method missing | pty.rs | Blocks mouse input testing |
| PtySimulator read_output() lacks timeout | pty.rs | Blocks output timing tests |
| BufferDiff DiffResult and CellDiff structs missing | diff.rs | Blocks diff result reporting |
| BufferDiff ignore options missing | diff.rs | Blocks flexible diff testing |
| StateTester capture() method missing | state.rs | Blocks state snapshot testing |
| StateTester assert_state_matches() missing | state.rs | Blocks snapshot comparison |
| TestDsl PTY composition missing | dsl.rs | Blocks fluent test API |
| TestDsl send_keys(), wait_for(), assert_buffer_eq() missing | dsl.rs | Blocks event testing |
| CliTester process spawning not implemented | cli.rs | Blocks CLI testing |
| CliTester CliOutput struct missing | cli.rs | Blocks output capture |
| CliTester temp directory cleanup not implemented | cli.rs | Blocks test isolation |
| ratatui-testing tests/ directory empty | tests/ | Blocks test coverage |

---

## 4. Implementation Recommendations

### 4.1 Immediate Actions (P0)

1. **Implement PtySimulator** (`ratatui-testing/src/pty.rs`)
   - Add `master: Option<Box<dyn MasterPty>>` field
   - Add `child: Option<Box<dyn Child>>` field
   - Add `reader` and `writer` fields for I/O
   - Implement `new(command: &[&str]) -> Result<Self>` to create `PtyPair`, spawn child
   - Implement `resize(&mut self, cols: u16, rows: u16) -> Result<()>` using `master.resize()`
   - Implement `write_input(&mut self, input: &str) -> Result<()>` using `writer`
   - Implement `read_output(&mut self, timeout: Duration) -> Result<String>` with timeout
   - Implement `inject_key_event(&mut self, event: KeyEvent) -> Result<()>` using `crossterm::execute!`
   - Implement `inject_mouse_event(&mut self, event: MouseEvent) -> Result<()>` using `crossterm::execute!`

2. **Implement BufferDiff** (`ratatui-testing/src/diff.rs`)
   - Add fields: `ignore_fg: bool`, `ignore_bg: bool`, `ignore_attributes: bool`
   - Add builder methods: `ignore_fg(mut self, ignore: bool) -> Self`, etc.
   - Define `DiffResult` struct with `passed: bool`, `expected: Buffer`, `actual: Buffer`, `differences: Vec<CellDiff>`
   - Define `CellDiff` struct with `x: u16`, `y: u16`, `expected: Cell`, `actual: Cell`
   - Implement `diff(&self, expected: &str, actual: &str) -> DiffResult` parsing to Buffer
   - Implement `diff_str(&self, expected: &str, actual: &str) -> DiffResult`
   - Implement human-readable diff output in `Display` impl for `DiffResult`

3. **Implement StateTester** (`ratatui-testing/src/state.rs`)
   - Add `snapshot: Option<serde_json::Value>` field
   - Add `captured: Vec<serde_json::Value>` field for history
   - Implement `capture<S>(&mut self, state: &S) -> Result<()>` where S: Serialize
   - Implement `assert_state<S>(&self, state: &S) -> Result<()>` comparing to snapshot
   - Implement `assert_state_matches(&self, expected: &Value) -> Result<()>` comparing JSON

4. **Implement TestDsl** (`ratatui-testing/src/dsl.rs`)
   - Add fields: `pty: Option<PtySimulator>`, `buffer_diff: BufferDiff`, `state_tester: StateTester`
   - Implement `new() -> Self` initializing empty components
   - Implement `with_pty(mut self, cmd: &[&str]) -> Result<Self>` to create PTY
   - Implement `pty_mut(&mut self) -> Option<&mut PtySimulator>`
   - Implement `render(&self, widget: &impl Widget) -> Result<Buffer>` using `ratatui` rendering
   - Implement `assert_buffer_eq(&self, expected: &Buffer, actual: &Buffer) -> Result<()>`
   - Implement `send_keys(&mut self, keys: &str) -> Result<&mut Self>` using PTY
   - Implement `wait_for<F>(&mut self, timeout: Duration, predicate: F) -> Result<&mut Self>`
   - Implement `capture_state<S>(&mut self, state: &S) -> &mut Self`
   - Implement `assert_state<S: serde::Serialize>(&self, state: &S) -> Result<()>`

5. **Implement CliTester** (`ratatui-testing/src/cli.rs`)
   - Define `CliOutput` struct with `exit_code: i32`, `stdout: String`, `stderr: String`
   - Add `temp_dir: Option<tempfile::TempDir>` field
   - Implement `new() -> Self`
   - Implement `with_temp_dir(mut self) -> Result<Self>` creating temp directory
   - Implement `run(&self, args: &[&str]) -> Result<CliOutput>` spawning process

6. **Add Integration Tests**
   - Create `tests/pty_tests.rs` - PTY read/write/resize/inject tests
   - Create `tests/buffer_diff_tests.rs` - Buffer comparison tests
   - Create `tests/state_tests.rs` - State capture/assert tests
   - Create `tests/dsl_tests.rs` - Fluent API tests
   - Create `tests/cli_tests.rs` - CLI spawning tests
   - Create `tests/integration_tests.rs` - Full workflow tests

### 4.2 Medium-term Actions (P1)

7. **Begin Phase 6 Release Qualification**
   - End-to-end integration tests
   - Performance benchmarking
   - Security audit
   - Observability validation

8. **Fix Bedrock Test Environment Pollution**
   - Use `temp_env::var()` for environment variable isolation
   - Or run this test in a separate process

### 4.3 Short-term Actions (P2)

9. **Clean up TestHarness dead code** in `crates/cli/tests/common.rs`

10. **Run `cargo clippy --fix --allow-dirty`** to fix clippy warnings

---

## 5. Cross-References

| Document | Topic |
|----------|-------|
| [TUI System](./09-tui-system.md) | TUI layout, keybindings, views |
| [TUI Plugin API](./15-tui-plugin-api.md) | TUI plugin configuration |
| [Rust Test Implementation Roadmap](./17-rust-test-implementation-roadmap.md) | Overall testing strategy |
| [Crate-by-Crate Test Backlog](./18-crate-by-crate-test-backlog.md) | Testing tasks per crate |

---

## 6. Iteration History

| Iteration | Date | Focus | Completion |
|-----------|------|-------|------------|
| 1-19 | 2025-2026 | Initial implementation phases | ~90% |
| 20 | 2026-04-13 | Integration hardening | ~92% |
| 21 | 2026-04-14 | Convention tests, route tests | ~93% |
| 22 | 2026-04-14 | Gap analysis | ~93-96% |
| 23 | 2026-04-14 | PRD 20 spec update | In Progress |

---

*Document generated: 2026-04-14*
*Iteration: 23*
*Phase: Phase 5-6 (Hardening and Release Qualification)*
*Priority: Implement PRD 20 (ratatui-testing) per specifications*
