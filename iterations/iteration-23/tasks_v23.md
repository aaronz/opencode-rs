# Task List - Iteration 23

**Project:** OpenCode Rust Port  
**Iteration:** 23  
**Date:** 2026-04-14  
**Status:** In Progress - PRD 20 (ratatui-testing) Implementation

---

## P0 - Blocking Issues (Must Complete First)

### PtySimulator Implementation

- [ ] **FR-121:** PtySimulator creates PTY master/slave pair
  - File: `opencode-rust/ratatui-testing/src/pty.rs`
  - Add fields: `master: Option<Box<dyn MasterPty>>`, `child: Option<Box<dyn Child>>`, `writer`, `reader`
  - Implement `new(command: &[&str]) -> Result<Self>` creating PtyPair and spawning child

- [ ] **FR-122:** PtySimulator writes strings to PTY slave
  - File: `opencode-rust/ratatui-testing/src/pty.rs`
  - Implement `write_input(&mut self, input: &str) -> Result<()>`

- [ ] **FR-123:** PtySimulator reads output from PTY master with timeout
  - File: `opencode-rust/ratatui-testing/src/pty.rs`
  - Change signature to `read_output(&mut self, timeout: Duration) -> Result<String>`
  - Implement timeout-based reading

- [ ] **FR-124:** PtySimulator resizes PTY window (cols/rows)
  - File: `opencode-rust/ratatui-testing/src/pty.rs`
  - Implement `resize(&mut self, cols: u16, rows: u16) -> Result<()>`

- [ ] **FR-125:** PtySimulator injects KeyEvent via crossterm
  - File: `opencode-rust/ratatui-testing/src/pty.rs`
  - Implement `inject_key_event(&mut self, event: KeyEvent) -> Result<()>`
  - Use `crossterm::execute!` for event injection

- [ ] **FR-126:** PtySimulator injects MouseEvent via crossterm
  - File: `opencode-rust/ratatui-testing/src/pty.rs`
  - Implement `inject_mouse_event(&mut self, event: MouseEvent) -> Result<()>`
  - Use `crossterm::execute!` for event injection

---

### BufferDiff Implementation

- [ ] **FR-127:** BufferDiff compares two Buffers cell-by-cell
  - File: `opencode-rust/ratatui-testing/src/diff.rs`
  - Implement `diff(&self, expected: &str, actual: &str) -> DiffResult`
  - Parse strings to Buffer and compare cell-by-cell

- [ ] **FR-128:** BufferDiff reports exact x,y of differences
  - File: `opencode-rust/ratatui-testing/src/diff.rs`
  - Define `CellDiff { x: u16, y: u16, expected: Cell, actual: Cell }`
  - Populate differences vector with exact coordinates

- [ ] **FR-129:** BufferDiff supports ignoring foreground color
  - File: `opencode-rust/ratatui-testing/src/diff.rs`
  - Add `ignore_fg: bool` field to BufferDiff
  - Implement `ignore_fg(mut self, ignore: bool) -> Self` builder method

- [ ] **FR-130:** BufferDiff supports ignoring background color
  - File: `opencode-rust/ratatui-testing/src/diff.rs`
  - Add `ignore_bg: bool` field to BufferDiff
  - Implement `ignore_bg(mut self, ignore: bool) -> Self` builder method

- [ ] **FR-131:** BufferDiff supports ignoring attributes
  - File: `opencode-rust/ratatui-testing/src/diff.rs`
  - Add `ignore_attributes: bool` field to BufferDiff
  - Implement `ignore_attributes(mut self, ignore: bool) -> Self` builder method

- [ ] **FR-132:** BufferDiff provides human-readable diff output
  - File: `opencode-rust/ratatui-testing/src/diff.rs`
  - Define `DiffResult { passed: bool, expected: Buffer, actual: Buffer, differences: Vec<CellDiff> }`
  - Implement `Display` trait for DiffResult

---

### StateTester Implementation

- [ ] **FR-133:** StateTester captures serializable state to JSON
  - File: `opencode-rust/ratatui-testing/src/state.rs`
  - Add `snapshot: Option<Value>` and `captured: Vec<Value>` fields
  - Implement `capture<S>(&mut self, state: &S) -> Result<()>` where S: Serialize

- [ ] **FR-134:** StateTester compares current state to captured snapshot
  - File: `opencode-rust/ratatui-testing/src/state.rs`
  - Implement `assert_state<S>(&self, state: &S) -> Result<()>` comparing to snapshot

- [ ] **FR-135:** StateTester reports mismatches with JSON diff
  - File: `opencode-rust/ratatui-testing/src/state.rs`
  - Implement `assert_state_matches(&self, expected: &Value) -> Result<()>`

---

### TestDsl Implementation

- [ ] **FR-136:** TestDsl renders widget to Buffer
  - File: `opencode-rust/ratatui-testing/src/dsl.rs`
  - Implement `render(&self, widget: &impl Widget) -> Result<Buffer>`

- [ ] **FR-137:** TestDsl composes PTY, BufferDiff, StateTester
  - File: `opencode-rust/ratatui-testing/src/dsl.rs`
  - Add fields: `pty: Option<PtySimulator>`, `buffer_diff: BufferDiff`, `state_tester: StateTester`
  - Implement `with_pty(mut self, cmd: &[&str]) -> Result<Self>`
  - Implement `pty_mut(&mut self) -> Option<&mut PtySimulator>`

- [ ] **FR-138:** TestDsl provides fluent API with wait-for predicate
  - File: `opencode-rust/ratatui-testing/src/dsl.rs`
  - Implement `assert_buffer_eq(&self, expected: &Buffer, actual: &Buffer) -> Result<()>`
  - Implement `send_keys(&mut self, keys: &str) -> Result<&mut Self>`
  - Implement `wait_for<F>(&mut self, timeout: Duration, predicate: F) -> Result<&mut Self>`
  - Implement `capture_state<S>(&mut self, state: &S) -> &mut Self`
  - Implement `assert_state<S: serde::Serialize>(&self, state: &S) -> Result<()>`

---

### CliTester Implementation

- [ ] **FR-139:** CliTester spawns process, captures stdout/stderr, returns exit code
  - File: `opencode-rust/ratatui-testing/src/cli.rs`
  - Define `CliOutput { pub exit_code: i32, pub stdout: String, pub stderr: String }`
  - Implement `run(&self, args: &[&str]) -> Result<CliOutput>` using std::process::Command

- [ ] **FR-140:** CliTester cleans up temp directories
  - File: `opencode-rust/ratatui-testing/src/cli.rs`
  - Add `temp_dir: Option<tempfile::TempDir>` field
  - Implement `with_temp_dir(mut self) -> Result<Self>`

---

### Integration Tests

- [ ] Create `tests/pty_tests.rs` - PTY functionality tests
  - Test PTY creation with command
  - Test write_input and read_output
  - Test resize
  - Test inject_key_event
  - Test inject_mouse_event

- [ ] Create `tests/buffer_diff_tests.rs` - Buffer comparison tests
  - Test cell-by-cell comparison
  - Test ignore_fg option
  - Test ignore_bg option
  - Test ignore_attributes option
  - Test diff output formatting

- [ ] Create `tests/state_tests.rs` - State testing tests
  - Test capture serializable state
  - Test assert_state with matching state
  - Test assert_state with mismatched state (should fail)
  - Test assert_state_matches

- [ ] Create `tests/dsl_tests.rs` - Fluent API tests
  - Test new() creates empty components
  - Test with_pty() creates PTY
  - Test pty_mut() returns mutable reference
  - Test render() with simple widget
  - Test send_keys() chains correctly
  - Test wait_for() with predicate
  - Test capture_state() and assert_state()

- [ ] Create `tests/cli_tests.rs` - CLI testing tests
  - Test run() with simple command
  - Test stdout capture
  - Test stderr capture
  - Test exit code reporting
  - Test with_temp_dir() cleanup

- [ ] Create `tests/integration_tests.rs` - Full workflow tests
  - End-to-end TUI test scenario
  - PTY + BufferDiff composition
  - State capture across multiple operations

---

## P1 - High Priority Issues

### Phase 6 Release Qualification

- [ ] Begin end-to-end integration testing
- [ ] Set up performance benchmarking suite
- [ ] Prepare security audit checklist
- [ ] Validate observability (logging, metrics, tracing)

### Bedrock Test Fix

- [ ] Fix `test_bedrock_credential_resolution_bearer_token_priority`
  - File: Likely in `opencode-rust/crates/llm/`
  - Use `temp_env::var()` for environment variable isolation
  - Or run test in separate process

---

## P2 - Medium Priority Issues

### Code Cleanup

- [ ] Clean up TestHarness dead code
  - File: `opencode-rust/crates/cli/tests/common.rs`
  - Remove unused helper methods

### Clippy Fixes

- [ ] Run `cargo clippy --fix --allow-dirty` across all crates
- [ ] Address any warnings introduced

---

## Verification Tasks

- [ ] Run `cargo build -p ratatui-testing`
- [ ] Run `cargo test -p ratatui-testing`
- [ ] Run `cargo clippy -p ratatui-testing`
- [ ] Verify all FR items pass acceptance criteria

---

## Task Statistics

| Priority | Count | Completed |
|----------|-------|-----------|
| P0 | 20 | 0 |
| P1 | 4 | 0 |
| P2 | 3 | 0 |
| **Total** | **27** | **0** |

---

## Dependencies

- PtySimulator → portable-pty, crossterm
- BufferDiff → ratatui
- StateTester → serde, serde_json
- TestDsl → PtySimulator, BufferDiff, StateTester, ratatui
- CliTester → tempfile, std::process::Command
- Integration Tests → All modules above

---

*Task list generated: 2026-04-14*
*Iteration: 23*
*Priority: P0 tasks first, then P1, then P2*
