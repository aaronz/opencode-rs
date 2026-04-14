# Implementation Plan - Iteration 21

**Generated:** 2026-04-14
**Based on:** Spec v21 (Iteration 21)
**Status:** Planning

---

## 1. Priority Summary

### P0 - Blocking (PRD 20 Implementation)

| Module | Status | Impact |
|--------|--------|--------|
| PtySimulator | STUB | Blocks TUI testing |
| BufferDiff | STUB | Blocks buffer comparison |
| StateTester | STUB | Blocks state testing |
| TestDsl | STUB | Blocks fluent test API |
| CliTester | STUB | Blocks CLI testing |
| Integration Tests | EMPTY | No test coverage |

### P1 - High Priority

| Item | Status | Impact |
|------|--------|--------|
| Phase 6 Release Qualification | Not Started | Cannot release |
| Bedrock test fix | Failing | Test reliability |

### P2 - Medium Priority

| Item | Status | Impact |
|------|--------|--------|
| Trailing whitespace | storage/src/service.rs | Cleanliness |
| Clippy warnings | Multiple crates | Code quality |

---

## 2. Implementation Strategy

### 2.1 PtySimulator Implementation

**File:** `ratatui-testing/src/pty.rs`

**Required Methods:**
- `new(command: &[&str]) -> Result<Self>` - Create PTY with command
- `write_input(&mut self, data: &str) -> Result<()>` - Write to PTY slave
- `read_output(&mut self, timeout: Duration) -> Result<String>` - Read from PTY master
- `resize(&mut self, cols: u16, rows: u16) -> Result<()>` - Resize window
- `inject_key_event(&mut self, event: KeyEvent) -> Result<()>` - Inject KeyEvent
- `inject_mouse_event(&mut self, event: MouseEvent) -> Result<()>` - Inject MouseEvent

**Dependencies:** `portable-pty`, `crossterm`

**Key Implementation Notes:**
- Use `portable-pty` for cross-platform PTY support
- Use `crossterm` for key/mouse event generation
- PTY master reads output with configurable timeout
- Child process spawned with specified command

### 2.2 BufferDiff Implementation

**File:** `ratatui-testing/src/diff.rs`

**Required Structs:**
- `CellDiff { x: u16, y: u16, expected: Cell, actual: Cell }`
- `DiffResult { cells: Vec<CellDiff>, total_diffs: usize, passed: bool }`

**Required Methods:**
- `new() -> Self` - Create with defaults
- `ignore_fg(mut self, ignore: bool) -> Self` - Ignore foreground color
- `ignore_bg(mut self, ignore: bool) -> Self` - Ignore background color
- `ignore_attributes(mut self, ignore: bool) -> Self` - Ignore style attributes
- `diff(&self, expected: &Buffer, actual: &Buffer) -> DiffResult` - Compare buffers

**Dependencies:** `ratatui` for Buffer/Cell types

**Key Implementation Notes:**
- Cell-by-cell comparison of ratatui::Buffer
- Support ignoring colors and attributes independently
- Return human-readable diff output

### 2.3 StateTester Implementation

**File:** `ratatui-testing/src/state.rs`

**Required Struct:**
- `StateTester { snapshot: Option<Value> }`

**Required Methods:**
- `new() -> Self` - Create with empty snapshot
- `capture<S>(&mut self, state: &S) -> Result<()>` - Serialize state to JSON
- `assert_state<S>(&self, current: &S) -> Result<()>` - Compare with snapshot
- `assert_state_matches(&self, expected: &Value) -> Result<()>` - Compare with expected JSON

**Dependencies:** `serde_json` for JSON serialization

### 2.4 TestDsl Implementation

**File:** `ratatui-testing/src/dsl.rs`

**Required Struct:**
- `TestDsl { pty: Option<PtySimulator>, diff: BufferDiff, state_tester: StateTester }`

**Required Methods:**
- `new() -> Self` - Create with default components
- `with_pty(mut self) -> Result<Self>` - Add PTY for interactive testing
- `render_widget<W: Widget>(&self, widget: &W) -> Result<Buffer>` - Render to Buffer
- `send_keys(&mut self, keys: &str) -> Result<&mut Self>` - Simulate keyboard input
- `wait_for<F>(&mut self, timeout: Duration, predicate: F) -> Result<&mut Self>` - Wait for condition
- `capture_state<S>(&mut self, state: &S) -> &mut Self` - Capture state snapshot
- `assert_state<S>(&self, state: &S) -> Result<()>` - Assert state matches

**Dependencies:** Combines PtySimulator, BufferDiff, StateTester

**Key Implementation Notes:**
- Fluent API composition of all components
- Integrates with ratatui::Widget for rendering
- Wait-for predicate with timeout support

### 2.5 CliTester Implementation

**File:** `ratatui-testing/src/cli.rs`

**Required Structs:**
- `CliTester { bin: String, args: Vec<String>, env: HashMap<String, String>, temp_dir: Option<TempDir> }`
- `CliOutput { exit_code: i32, stdout: String, stderr: String }`

**Required Methods:**
- `new(bin: &str) -> Self` - Create with binary path
- `args(mut self, args: &[&str]) -> Self` - Add command arguments
- `env(mut self, key: &str, value: &str) -> Self` - Add environment variables
- `with_temp_dir(mut self) -> Result<Self>` - Add temp directory
- `run(&self) -> Result<CliOutput>` - Execute and return output

**Dependencies:** `assert_cmd`, `tempfile`

### 2.6 Integration Tests

**Required Files:**
- `tests/pty_tests.rs` - PTY read/write/resize/inject tests
- `tests/buffer_diff_tests.rs` - Buffer comparison tests
- `tests/state_tests.rs` - State capture/assert tests
- `tests/dsl_tests.rs` - Fluent API tests
- `tests/cli_tests.rs` - CLI spawning tests
- `tests/integration_tests.rs` - Full workflow tests

---

## 3. Implementation Order

1. **PtySimulator** - Foundation for TTY interaction
2. **BufferDiff** - Buffer comparison utilities
3. **StateTester** - State validation
4. **TestDsl** - Fluent API composition
5. **CliTester** - CLI process testing
6. **Integration Tests** - Validate all components

---

## 4. Phase 6 Preparation

After PRD 20 implementation completes:

1. End-to-end integration tests
2. Performance benchmarking
3. Security audit
4. Observability validation

---

## 5. File Locations

| File | Current Lines | Target Lines |
|------|---------------|--------------|
| `ratatui-testing/src/pty.rs` | 19 | ~100 |
| `ratatui-testing/src/diff.rs` | 17 | ~120 |
| `ratatui-testing/src/state.rs` | 20 | ~80 |
| `ratatui-testing/src/dsl.rs` | 17 | ~100 |
| `ratatui-testing/src/cli.rs` | 17 | ~90 |
| `ratatui-testing/tests/*.rs` | 0 | ~500+ total |

---

## 6. Dependencies Update

Add to `ratatui-testing/Cargo.toml`:
```toml
[dependencies]
portable-pty = "0.8"
crossterm = { version = "0.28", features = ["events", "input"] }
anyhow = "1.0"
thiserror = "2.0"
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
ratatui = "0.28"

[dev-dependencies]
tokio = { version = "1.45", features = ["full"] }
assert_cmd = "2.0"
tempfile = "3.15"
```

---

## 7. Success Criteria

- [ ] All 6 ratatui-testing modules fully implemented
- [ ] All modules compile without errors
- [ ] Integration tests created and passing
- [ ] `cargo test -p ratatui-testing` passes
- [ ] Phase 6 planning initiated
