# Task List - Iteration 21

**Generated:** 2026-04-14
**Priority:** P0 > P1 > P2
**Based on:** Spec v21 (Iteration 21)

---

## P0 - Blocking Issues (PRD 20 Implementation)

### P0.1: PtySimulator Implementation

**Status:** TODO
**File:** `ratatui-testing/src/pty.rs`
**Priority:** CRITICAL

#### Tasks:
- [x] P0.1.1: Add `portable-pty` import and types
- [ ] P0.1.2: Implement `PtySimulator` struct with `master` and `child` fields
- [ ] P0.1.3: Implement `new(command: &[&str]) -> Result<Self>`
- [ ] P0.1.4: Implement `write_input(&mut self, data: &str) -> Result<()>`
- [ ] P0.1.5: Implement `read_output(&mut self, timeout: Duration) -> Result<String>`
- [ ] P0.1.6: Implement `resize(&mut self, cols: u16, rows: u16) -> Result<()>`
- [ ] P0.1.7: Implement `inject_key_event(&mut self, event: KeyEvent) -> Result<()>`
- [ ] P0.1.8: Implement `inject_mouse_event(&mut self, event: MouseEvent) -> Result<()>`
- [ ] P0.1.9: Ensure proper error handling with `thiserror`

---

### P0.2: BufferDiff Implementation

**Status:** TODO
**File:** `ratatui-testing/src/diff.rs`
**Priority:** CRITICAL

#### Tasks:
- [ ] P0.2.1: Define `CellDiff` struct with `x: u16`, `y: u16`, `expected: Cell`, `actual: Cell`
- [ ] P0.2.2: Define `DiffResult` struct with `cells: Vec<CellDiff>`, `total_diffs: usize`, `passed: bool`
- [ ] P0.2.3: Implement `BufferDiff` struct with ignore options
- [ ] P0.2.4: Implement `new() -> Self`
- [ ] P0.2.5: Implement `ignore_fg(mut self, ignore: bool) -> Self`
- [ ] P0.2.6: Implement `ignore_bg(mut self, ignore: bool) -> Self`
- [ ] P0.2.7: Implement `ignore_attributes(mut self, ignore: bool) -> Self`
- [ ] P0.2.8: Implement `diff(&self, expected: &Buffer, actual: &Buffer) -> DiffResult`
- [ ] P0.2.9: Implement human-readable diff output formatting

---

### P0.3: StateTester Implementation

**Status:** TODO
**File:** `ratatui-testing/src/state.rs`
**Priority:** CRITICAL

#### Tasks:
- [ ] P0.3.1: Define `StateTester` struct with `snapshot: Option<Value>` field
- [ ] P0.3.2: Implement `new() -> Self`
- [ ] P0.3.3: Implement `capture<S>(&mut self, state: &S) -> Result<()>` where S: Serialize
- [ ] P0.3.4: Implement `assert_state<S>(&self, current: &S) -> Result<()>` where S: Serialize
- [ ] P0.3.5: Implement `assert_state_matches(&self, expected: &Value) -> Result<()>`
- [ ] P0.3.6: Add proper error types with `thiserror`

---

### P0.4: TestDsl Implementation

**Status:** TODO
**File:** `ratatui-testing/src/dsl.rs`
**Priority:** CRITICAL

#### Tasks:
- [ ] P0.4.1: Define `TestDsl` struct composing `PtySimulator`, `BufferDiff`, `StateTester`
- [ ] P0.4.2: Implement `new() -> Self`
- [ ] P0.4.3: Implement `with_pty(command: &[&str]) -> Result<Self>`
- [ ] P0.4.4: Implement `render_widget<W: Widget>(&self, widget: &W) -> Result<Buffer>`
- [ ] P0.4.5: Implement `send_keys(&mut self, keys: &str) -> Result<&mut Self>`
- [ ] P0.4.6: Implement `wait_for<F>(&mut self, timeout: Duration, predicate: F) -> Result<&mut Self>`
- [ ] P0.4.7: Implement `capture_state<S>(&mut self, state: &S) -> &mut Self`
- [ ] P0.4.8: Implement `assert_state<S>(&self, state: &S) -> Result<()>`
- [ ] P0.4.9: Ensure fluent API chains correctly

---

### P0.5: CliTester Implementation

**Status:** TODO
**File:** `ratatui-testing/src/cli.rs`
**Priority:** CRITICAL

#### Tasks:
- [ ] P0.5.1: Define `CliTester` struct with `bin`, `args`, `env`, `temp_dir` fields
- [ ] P0.5.2: Define `CliOutput` struct with `exit_code`, `stdout`, `stderr`
- [ ] P0.5.3: Implement `new(bin: &str) -> Self`
- [ ] P0.5.4: Implement `args(mut self, args: &[&str]) -> Self`
- [ ] P0.5.5: Implement `env(mut self, key: &str, value: &str) -> Self`
- [ ] P0.5.6: Implement `with_temp_dir(mut self) -> Result<Self>`
- [ ] P0.5.7: Implement `run(&self) -> Result<CliOutput>`
- [ ] P0.5.8: Ensure stdout/stderr capture and exit code return

---

### P0.6: Integration Tests

**Status:** TODO
**File:** `ratatui-testing/tests/`
**Priority:** CRITICAL

#### Tasks:
- [ ] P0.6.1: Create `tests/pty_tests.rs` - PTY functionality tests
  - Test PTY creation with command
  - Test write_input and read_output
  - Test resize operation
  - Test key event injection
  - Test mouse event injection
- [ ] P0.6.2: Create `tests/buffer_diff_tests.rs` - Buffer comparison tests
  - Test identical buffers pass
  - Test different buffers show differences
  - Test ignore_fg option
  - Test ignore_bg option
  - Test ignore_attributes option
- [ ] P0.6.3: Create `tests/state_tests.rs` - State testing tests
  - Test state capture to JSON
  - Test state comparison with matching state
  - Test state comparison with different state
  - Test JSON mismatch reporting
- [ ] P0.6.4: Create `tests/dsl_tests.rs` - Fluent API tests
  - Test widget rendering to Buffer
  - Test key sending via PTY
  - Test wait_for predicate
  - Test state capture and assertion
  - Test fluent chaining
- [ ] P0.6.5: Create `tests/cli_tests.rs` - CLI testing tests
  - Test process spawning
  - Test stdout/stderr capture
  - Test exit code return
  - Test temp directory cleanup
- [ ] P0.6.6: Create `tests/integration_tests.rs` - Full workflow tests
  - Test complete TUI test scenario
  - Test end-to-end user interaction simulation

---

### P0.7: Cargo.toml Update

**Status:** TODO
**File:** `ratatui-testing/Cargo.toml`
**Priority:** CRITICAL

#### Tasks:
- [ ] P0.7.1: Add `serde_json = "1.0"` dependency
- [ ] P0.7.2: Add `assert_cmd = "2.0"` to dev-dependencies
- [ ] P0.7.3: Add `tempfile = "3.15"` to dev-dependencies
- [ ] P0.7.4: Verify `portable-pty = "0.8"` is present
- [ ] P0.7.5: Verify `crossterm` has `events` and `input` features

---

## P1 - High Priority Issues

### P1.1: Phase 6 Release Qualification

**Status:** TODO
**Phase:** Release Qualification
**Priority:** HIGH

#### Tasks:
- [ ] P1.1.1: Create Phase 6 planning document
- [ ] P1.1.2: Define end-to-end test scenarios
- [ ] P1.1.3: Set up performance benchmarking
- [ ] P1.1.4: Plan security audit checklist
- [ ] P1.1.5: Define observability validation criteria

---

### P1.2: Bedrock Test Fix

**Status:** TODO
**File:** Likely in `crates/llm/`
**Priority:** HIGH

#### Tasks:
- [ ] P1.2.1: Identify failing test `test_bedrock_credential_resolution_bearer_token_priority`
- [ ] P1.2.2: Use `temp_env::var()` for environment variable isolation
- [ ] P1.2.3: Verify fix with `cargo test --all-features`

---

## P2 - Medium Priority Issues

### P2.1: Trailing Whitespace Fix

**Status:** TODO
**File:** `crates/storage/src/service.rs`
**Priority:** MEDIUM

#### Tasks:
- [ ] P2.1.1: Run `cargo fmt --all` to fix trailing whitespace
- [ ] P2.1.2: Verify no whitespace issues remain

---

### P2.2: Clippy Warnings

**Status:** TODO
**Files:** Multiple crates
**Priority:** MEDIUM

#### Tasks:
- [ ] P2.2.1: Run `cargo clippy --all -- -D warnings`
- [ ] P2.2.2: Fix dead code warnings
- [ ] P2.2.3: Fix unused variable warnings
- [ ] P2.2.4: Fix unused import warnings
- [ ] P2.2.5: Verify all fixes with `cargo clippy --all -- -D warnings`

---

## Task Summary

| Priority | Count | Completed | Remaining |
|----------|-------|-----------|-----------|
| P0 | 40 | 0 | 40 |
| P1 | 6 | 0 | 6 |
| P2 | 7 | 0 | 7 |
| **Total** | **53** | **0** | **53** |

---

## Dependencies

- P0.1 (PtySimulator) must complete before P0.4 (TestDsl)
- P0.2 (BufferDiff) must complete before P0.4 (TestDsl)
- P0.3 (StateTester) must complete before P0.4 (TestDsl)
- P0.7 (Cargo.toml) must complete before all others
- P0.4 (TestDsl) should be completed before P0.6.4 (DSL tests)

---

## Verification Commands

```bash
# Build ratatui-testing
cargo build -p ratatui-testing

# Run ratatui-testing tests
cargo test -p ratatui-testing

# Format all code
cargo fmt --all

# Run clippy
cargo clippy --all -- -D warnings

# Run all tests
cargo test --all-features
```
