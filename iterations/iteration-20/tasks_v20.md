# Task List - Iteration 20

**Version:** 2.0  
**Generated:** 2026-04-14  
**Focus:** PRD 20 - ratatui-testing Framework Implementation

---

## P0 Priority Tasks (Blocking - PRD 20)

### 1. PtySimulator Implementation

**Status:** ❌ NOT STARTED  
**FR:** FR-122, FR-123, FR-124, FR-125  
**File:** `opencode-rust/ratatui-testing/src/pty.rs`

#### Tasks

- [x] **T-001:** Add `portable-pty` dependency to `Cargo.toml`
- [ ] **T-002:** Add `portable-pty` to features in `Cargo.toml`
- [ ] **T-003:** Implement `new(command: &[&str])` - Create PTY master/slave pair
- [ ] **T-004:** Implement `write_input(data: &str)` - Write to PTY slave
- [ ] **T-005:** Implement `read_output()` - Read from PTY master with timeout
- [ ] **T-006:** Implement `resize(cols: u16, rows: u16)` - Resize PTY window
- [ ] **T-007:** Implement `inject_key_event(key: KeyEvent)` - Inject keyboard events
- [ ] **T-008:** Implement `inject_mouse_event(event: MouseEvent)` - Inject mouse events
- [ ] **T-009:** Add unit tests for PtySimulator

#### Acceptance Criteria

- [ ] Creates PTY master/slave pair on Unix
- [ ] Writes strings to PTY slave
- [ ] Reads output from PTY master with configurable timeout
- [ ] Resizes PTY window (cols/rows)
- [ ] Injects KeyEvent via crossterm
- [ ] Injects MouseEvent via crossterm

---

### 2. BufferDiff Implementation

**Status:** ❌ NOT STARTED  
**FR:** FR-126, FR-127, FR-128, FR-129  
**File:** `opencode-rust/ratatui-testing/src/diff.rs`

#### Tasks

- [ ] **T-010:** Define `CellDiff` struct with x, y, expected, actual fields
- [ ] **T-011:** Define `DiffResult` struct with cells vector and total_diffs
- [ ] **T-012:** Add `ignore_colors` field to `BufferDiff`
- [ ] **T-013:** Add `ignore_attributes` field to `BufferDiff`
- [ ] **T-014:** Implement `new()` constructor with options
- [ ] **T-015:** Implement `compare(expected, actual)` - cell-by-cell comparison
- [ ] **T-016:** Implement `to_string()` - human-readable diff format
- [ ] **T-017:** Add unit tests for BufferDiff

#### Acceptance Criteria

- [ ] Compares two Buffers cell-by-cell
- [ ] Reports exact x,y coordinates of differences
- [ ] Supports ignoring foreground/background/attributes
- [ ] Provides human-readable diff output

---

### 3. StateTester Implementation

**Status:** ❌ NOT STARTED  
**FR:** FR-130, FR-131, FR-132  
**File:** `opencode-rust/ratatui-testing/src/state.rs`

#### Tasks

- [ ] **T-020:** Add `serde_json` dependency to `Cargo.toml`
- [ ] **T-021:** Implement `capture(state)` - serialize state to JSON snapshot
- [ ] **T-022:** Implement `assert_state(current)` - compare current with snapshot
- [ ] **T-023:** Implement `assert_state_matches(expected)` - compare with expected JSON
- [ ] **T-024:** Implement JSON diff reporting for mismatches
- [ ] **T-025:** Add unit tests for StateTester

#### Acceptance Criteria

- [ ] Captures serializable state to JSON
- [ ] Compares current state to captured snapshot
- [ ] Reports mismatches with JSON diff format showing expected vs actual values

---

### 4. TestDsl Implementation

**Status:** ❌ NOT STARTED  
**FR:** FR-133, FR-134, FR-135  
**File:** `opencode-rust/ratatui-testing/src/dsl.rs`

#### Tasks

- [ ] **T-030:** Add PtySimulator, BufferDiff, StateTester as fields
- [ ] **T-031:** Implement `new()` - initialize all components
- [ ] **T-032:** Implement `render_widget(widget)` - render to Buffer
- [ ] **T-033:** Implement `send_keys(keys)` - simulate keyboard input
- [ ] **T-034:** Implement `wait_for(predicate, timeout)` - wait for condition
- [ ] **T-035:** Implement `assert_buffer_eq(expected)` - compare buffers
- [ ] **T-036:** Verify fluent API composition works correctly
- [ ] **T-037:** Add unit tests for TestDsl

#### Acceptance Criteria

- [ ] Renders widget to Buffer
- [ ] Composes PtySimulator, BufferDiff, StateTester
- [ ] Fluent API chains correctly (e.g., `.send_keys("hello").wait_for(...).assert_buffer_eq(...)`)
- [ ] Wait-for predicate support with timeout

---

### 5. CliTester Implementation

**Status:** ❌ NOT STARTED  
**FR:** FR-136, FR-137, FR-138  
**File:** `opencode-rust/ratatui-testing/src/cli.rs`

#### Tasks

- [ ] **T-040:** Add `assert_cmd` dependency to `Cargo.toml`
- [ ] **T-041:** Add `tempfile` dependency to `Cargo.toml`
- [ ] **T-042:** Add `tempfile` to dev-dependencies in `Cargo.toml`
- [ ] **T-043:** Implement `new(bin: &str)` - create with binary path
- [ ] **T-044:** Implement builder pattern with `args(args: &[&str])`
- [ ] **T-045:** Implement builder pattern with `env(key, value)`
- [ ] **T-046:** Implement `temp_dir()` - create temp directory
- [ ] **T-047:** Implement `run()` - execute process, capture output, return exit code
- [ ] **T-048:** Ensure automatic cleanup of temp directories
- [ ] **T-049:** Add unit tests for CliTester

#### Acceptance Criteria

- [ ] Spawns process with args
- [ ] Captures stdout and stderr
- [ ] Returns exit code
- [ ] Cleans up temp directories automatically

---

### 6. Integration Tests

**Status:** ❌ NOT STARTED  
**FR:** FR-139, FR-140, FR-141  
**Directory:** `opencode-rust/ratatui-testing/tests/`

#### Tasks

- [ ] **T-050:** Create `tests/pty_tests.rs`
  - [ ] Test PTY creation
  - [ ] Test write/read operations
  - [ ] Test resize
  - [ ] Test key event injection
  - [ ] Test mouse event injection

- [ ] **T-051:** Create `tests/buffer_diff_tests.rs`
  - [ ] Test identical buffers
  - [ ] Test different buffers
  - [ ] Test ignore colors option
  - [ ] Test ignore attributes option
  - [ ] Test human-readable output

- [ ] **T-052:** Create `tests/state_tester_tests.rs`
  - [ ] Test state capture
  - [ ] Test state comparison
  - [ ] Test mismatch reporting

- [ ] **T-053:** Create `tests/test_dsl_tests.rs`
  - [ ] Test widget rendering
  - [ ] Test key sending
  - [ ] Test wait_for predicate
  - [ ] Test buffer assertion
  - [ ] Test fluent chaining

- [ ] **T-054:** Create `tests/cli_tester_tests.rs`
  - [ ] Test process spawning
  - [ ] Test output capture
  - [ ] Test exit code
  - [ ] Test temp directory cleanup

- [ ] **T-055:** Verify `cargo test -p ratatui-testing` passes
- [ ] **T-056:** Verify `cargo test -p ratatui-testing --all-features` passes

#### Acceptance Criteria

- [ ] All modules compile together without errors
- [ ] Integration tests pass with `cargo test -p ratatui-testing`
- [ ] Works with `cargo test --all-features -p ratatui-testing`
- [ ] Cross-platform (Unix primary, Windows best-effort)

---

## P1 Priority Tasks (High Priority)

### 7. Phase 6 Release Qualification Preparation

**Status:** ❌ NOT STARTED  
**Prerequisites:** PRD 20 complete

#### Tasks

- [ ] **T-060:** End-to-end integration tests
- [ ] **T-061:** Performance benchmarking
- [ ] **T-062:** Security audit
- [ ] **T-063:** Observability validation

---

## P2 Priority Tasks (Short Term)

### 8. Minor Fixes

**Status:** ❌ NOT STARTED

#### Tasks

- [ ] **T-070:** Run `cargo fmt --all` to fix trailing whitespace
- [ ] **T-071:** Fix Bedrock test environment pollution (`test_bedrock_credential_resolution_bearer_token_priority`)
- [ ] **T-072:** Run `cargo fix --tests --all` to fix clippy warnings

---

## Task Summary

| Priority | Tasks | Completed | Total |
|----------|-------|-----------|-------|
| P0 | T-001 to T-056 | 0 | 56 |
| P1 | T-060 to T-063 | 0 | 4 |
| P2 | T-070 to T-072 | 0 | 3 |
| **Total** | | **0** | **63** |

---

## Dependency Graph

```
T-001 (Add portable-pty dep)
    ↓
T-003 to T-009 (PtySimulator impl)
    ↓
T-037 (TestDsl tests)
    ↓
T-055, T-056 (Integration tests pass)

T-010 to T-017 (BufferDiff impl)
    ↓
T-037 (TestDsl tests)

T-020 (Add serde_json dep)
    ↓
T-021 to T-025 (StateTester impl)
    ↓
T-037 (TestDsl tests)

T-040, T-041 (Add assert_cmd, tempfile deps)
    ↓
T-043 to T-049 (CliTester impl)
    ↓
T-054 (CliTester tests)
```

---

## File Changes Required

### New Files

- `opencode-rust/ratatui-testing/tests/pty_tests.rs`
- `opencode-rust/ratatui-testing/tests/buffer_diff_tests.rs`
- `opencode-rust/ratatui-testing/tests/state_tester_tests.rs`
- `opencode-rust/ratatui-testing/tests/test_dsl_tests.rs`
- `opencode-rust/ratatui-testing/tests/cli_tester_tests.rs`

### Modified Files

- `opencode-rust/ratatui-testing/Cargo.toml` (add dependencies)
- `opencode-rust/ratatui-testing/src/pty.rs` (implement stubs)
- `opencode-rust/ratatui-testing/src/diff.rs` (implement stubs)
- `opencode-rust/ratatui-testing/src/state.rs` (implement stubs)
- `opencode-rust/ratatui-testing/src/dsl.rs` (implement stubs)
- `opencode-rust/ratatui-testing/src/cli.rs` (implement stubs)

---

*Document Version: 2.0*  
*Iteration: 20*  
*Last Updated: 2026-04-14*
