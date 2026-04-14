# Implementation Plan - Iteration 20

**Version:** 2.0  
**Generated:** 2026-04-14  
**Status:** Implementation ~93-96% Complete  
**Iteration Focus:** PRD 20 - ratatui-testing Framework

---

## 1. Executive Summary

The OpenCode Rust implementation is approximately **93-96% complete**. The primary focus for iteration-20 is implementing the **ratatui-testing framework (PRD 20)**, which is currently entirely in stub form. This framework is critical for enabling Phase 6 (Release Qualification) testing.

### Key Metrics

| Metric | Value |
|--------|-------|
| Total Feature Requirements | 140 |
| Completed | 120 (85.7%) |
| In Progress | 20 (14.3%) - PRD 20 |
| Phase 6 Status | Not Started (0%) |

---

## 2. Implementation Status by Phase

| Phase | Description | Status | Completion |
|-------|-------------|--------|------------|
| Phase 0 | Project Foundation | ✅ Done | 100% |
| Phase 1 | Authority (Core/Config/Storage/Server) | ✅ Done | ~100% |
| Phase 2 | Runtime Core (Agent/Tools/Plugin/TUI Plugin) | ✅ Done | ~100% |
| Phase 3 | Infrastructure (MCP/LSP/Providers/Formatters/Skills/TUI) | ✅ Done | ~100% |
| Phase 4 | Interface (Desktop/Web/GitHub-GitLab) | ✅ Done | ~98% |
| Phase 5 | Hardening (Compatibility/Convention) | ✅ Done | ~98% |
| Phase 6 | Release Qualification | ❌ Not Started | ~0% |
| **PRD 20** | **ratatui-testing Framework** | **🔄 In Progress** | **~5%** |

---

## 3. PRD 20 Implementation Plan (Primary Focus)

### 3.1 Overview

The `ratatui-testing` crate provides a testing framework for TUI applications built with `ratatui`. All modules are currently stubs that return no-op values.

**Location:** `opencode-rust/ratatui-testing/`

### 3.2 Component Implementation Order

```
┌─────────────────────────────────────────────────────────────┐
│                    PRD 20 Dependencies                       │
├─────────────────────────────────────────────────────────────┤
│  1. PtySimulator (FR-122 to FR-125)                         │
│     └── Required by: TestDsl                                 │
│  2. BufferDiff (FR-126 to FR-129)                           │
│     └── Required by: TestDsl                                 │
│  3. StateTester (FR-130 to FR-132)                          │
│     └── Required by: TestDsl                                 │
│  4. TestDsl (FR-133 to FR-135)                              │
│     └── Composites: PtySimulator, BufferDiff, StateTester   │
│  5. CliTester (FR-136 to FR-138)                            │
│     └── Independent component                               │
│  6. Integration Tests (FR-139 to FR-141)                    │
└─────────────────────────────────────────────────────────────┘
```

### 3.3 PtySimulator Implementation (FR-122 to FR-125)

**File:** `src/pty.rs`

**Required Dependencies:**
- `portable-pty` for cross-platform PTY support
- `crossterm` for key/mouse event injection

**Methods to Implement:**

| Method | FR | Description |
|--------|-----|-------------|
| `new(command: &[&str])` | FR-122 | Create PTY master/slave pair |
| `write_input(data: &str)` | FR-123 | Write strings to PTY slave |
| `read_output()` | FR-123 | Read from PTY master with timeout |
| `resize(cols: u16, rows: u16)` | FR-124 | Resize PTY window dimensions |
| `inject_key_event(key: KeyEvent)` | FR-125 | Inject keyboard events |
| `inject_mouse_event(event: MouseEvent)` | FR-125 | Inject mouse events |

**Acceptance Criteria:**
- [ ] Creates PTY master/slave pair on Unix
- [ ] Writes strings to PTY slave
- [ ] Reads output from PTY master with configurable timeout
- [ ] Resizes PTY window (cols/rows)
- [ ] Injects KeyEvent via crossterm
- [ ] Injects MouseEvent via crossterm

### 3.4 BufferDiff Implementation (FR-126 to FR-129)

**File:** `src/diff.rs`

**Required Dependencies:**
- `ratatui` for Buffer/Cell types

**Structs to Implement:**

```rust
pub struct BufferDiff {
    ignore_colors: bool,
    ignore_attributes: bool,
}

#[derive(Debug)]
pub struct CellDiff {
    pub x: u16,
    pub y: u16,
    pub expected: Cell,
    pub actual: Cell,
}

#[derive(Debug)]
pub struct DiffResult {
    pub cells: Vec<CellDiff>,
    pub total_diffs: usize,
}
```

**Methods to Implement:**

| Method | FR | Description |
|--------|-----|-------------|
| `new()` | FR-126 | Create with options |
| `compare(expected, actual)` | FR-126 | Return DiffResult |
| `to_string()` | FR-129 | Human-readable diff |

**Acceptance Criteria:**
- [ ] Compares two Buffers cell-by-cell
- [ ] Reports exact x,y coordinates of differences
- [ ] Supports ignoring foreground/background/attributes
- [ ] Provides human-readable diff output

### 3.5 StateTester Implementation (FR-130 to FR-132)

**File:** `src/state.rs`

**Required Dependencies:**
- `serde_json` for JSON serialization

**Methods to Implement:**

| Method | FR | Description |
|--------|-----|-------------|
| `capture(state)` | FR-130 | Serialize state to JSON |
| `assert_state(current)` | FR-131 | Compare with snapshot |
| `assert_state_matches(expected)` | FR-131 | Compare with expected JSON |

**Acceptance Criteria:**
- [ ] Captures serializable state to JSON
- [ ] Compares current state to captured snapshot
- [ ] Reports mismatches with JSON diff format

### 3.6 TestDsl Implementation (FR-133 to FR-135)

**File:** `src/dsl.rs`

**Composition:**
- PtySimulator for input simulation
- BufferDiff for output verification
- StateTester for state validation

**Methods to Implement:**

| Method | FR | Description |
|--------|-----|-------------|
| `render_widget(widget)` | FR-133 | Render to Buffer |
| `send_keys(keys)` | FR-133 | Simulate keyboard input |
| `wait_for(predicate, timeout)` | FR-135 | Wait for condition |
| `assert_buffer_eq(expected)` | FR-133 | Compare buffers |

**Acceptance Criteria:**
- [ ] Renders widget to Buffer
- [ ] Composes PTY, BufferDiff, StateTester
- [ ] Fluent API chains correctly
- [ ] Wait-for predicate support

### 3.7 CliTester Implementation (FR-136 to FR-138)

**File:** `src/cli.rs`

**Required Dependencies:**
- `assert_cmd` for process spawning
- `tempfile` for temp directories

**Methods to Implement:**

| Method | FR | Description |
|--------|-----|-------------|
| `new(bin: &str)` | FR-136 | Create with binary path |
| `args(args: &[&str])` | FR-136 | Add command arguments |
| `env(key, value)` | FR-136 | Add environment variables |
| `temp_dir()` | FR-136 | Create temp directory |
| `run()` | FR-137 | Execute and return Output |

**Acceptance Criteria:**
- [ ] Spawns process with args
- [ ] Captures stdout/stderr
- [ ] Returns exit code
- [ ] Cleans up temp directories

### 3.8 Integration Tests (FR-139 to FR-141)

**Directory:** `tests/`

**Files to Create:**
- `tests/pty_tests.rs`
- `tests/buffer_diff_tests.rs`
- `tests/state_tester_tests.rs`
- `tests/test_dsl_tests.rs`
- `tests/cli_tester_tests.rs`

**Acceptance Criteria:**
- [ ] All modules compile together
- [ ] Integration tests pass
- [ ] Works with `cargo test -p ratatui-testing`
- [ ] Cross-platform (Unix primary, Windows best-effort)

---

## 4. Phase 6 Release Qualification Plan

Phase 6 cannot begin until PRD 20 is substantially complete.

### 4.1 Prerequisites

- [ ] PtySimulator implementation complete
- [ ] BufferDiff implementation complete
- [ ] StateTester implementation complete
- [ ] TestDsl implementation complete
- [ ] CliTester implementation complete
- [ ] Integration tests passing

### 4.2 Phase 6 Tasks (Not Started)

| Task | Description | Status |
|------|-------------|--------|
| E2E Integration Tests | End-to-end integration tests | ❌ Pending |
| Performance Benchmarks | Benchmark critical paths | ❌ Pending |
| Security Audit | Review auth, permissions, secrets | ❌ Pending |
| Observability Validation | Logging, tracing, metrics | ❌ Pending |

---

## 5. P2 Issues (Short Term)

| Issue | Status | Fix |
|-------|--------|-----|
| Trailing whitespace in `storage/src/service.rs` | ❌ Not Fixed | Run `cargo fmt --all` |
| Bedrock test environment pollution | ❌ Not Fixed | Use `temp_env` pattern |
| Clippy warnings | ⚠️ Minor | Run `cargo fix --tests --all` |

---

## 6. Implementation Checklist

### PRD 20 Components

- [ ] **PtySimulator** - `src/pty.rs`
  - [ ] Add `portable-pty` dependency
  - [ ] Implement PTY creation with command
  - [ ] Implement `write_input()`
  - [ ] Implement `read_output()` with timeout
  - [ ] Implement `resize()`
  - [ ] Implement `inject_key_event()`
  - [ ] Implement `inject_mouse_event()`

- [ ] **BufferDiff** - `src/diff.rs`
  - [ ] Implement `CellDiff` and `DiffResult` structs
  - [ ] Implement cell-by-cell comparison
  - [ ] Implement color ignore option
  - [ ] Implement attribute ignore option
  - [ ] Implement human-readable diff output

- [ ] **StateTester** - `src/state.rs`
  - [ ] Implement `capture()` method
  - [ ] Implement `assert_state()` method
  - [ ] Implement `assert_state_matches()` method
  - [ ] Add JSON diff reporting

- [ ] **TestDsl** - `src/dsl.rs`
  - [ ] Compose PtySimulator
  - [ ] Compose BufferDiff
  - [ ] Compose StateTester
  - [ ] Implement `render_widget()`
  - [ ] Implement `send_keys()`
  - [ ] Implement `wait_for()`
  - [ ] Implement `assert_buffer_eq()`
  - [ ] Implement fluent API chaining

- [ ] **CliTester** - `src/cli.rs`
  - [ ] Add `assert_cmd` dependency
  - [ ] Add `tempfile` dependency
  - [ ] Implement builder pattern
  - [ ] Implement `run()` with output capture

- [ ] **Integration Tests** - `tests/`
  - [ ] Create `tests/pty_tests.rs`
  - [ ] Create `tests/buffer_diff_tests.rs`
  - [ ] Create `tests/state_tester_tests.rs`
  - [ ] Create `tests/test_dsl_tests.rs`
  - [ ] Create `tests/cli_tester_tests.rs`
  - [ ] Verify all tests pass

### P2 Fixes

- [ ] Run `cargo fmt --all`
- [ ] Fix Bedrock test environment pollution
- [ ] Clean up clippy warnings

---

## 7. Success Criteria

| Criterion | Target | Current |
|-----------|--------|---------|
| Implementation Completion | 100% | ~96% |
| PRD 20 Completion | 100% | ~5% |
| Integration Tests Passing | 100% | 0% (no tests) |
| Clippy Warnings | 0 | Multiple |

---

## 8. Timeline

**Iteration 20 Focus:** Implement PRD 20 (ratatui-testing framework)

**Week 1:**
- PtySimulator implementation
- BufferDiff implementation

**Week 2:**
- StateTester implementation
- TestDsl implementation

**Week 3:**
- CliTester implementation
- Integration tests

**Week 4:**
- Integration testing and fixes
- Begin Phase 6 preparation

---

*Document Version: 2.0*  
*Iteration: 20*  
*Last Updated: 2026-04-14*
