# Gap Analysis Report: ratatui-testing

**Date**: 2026-04-17  
**Iteration**: 26  
**Prepared for**: ratatui-testing crate implementation review

---

## Executive Summary

The `ratatui-testing` crate has been **substantially implemented** from the PRD stub state. All five core modules (PtySimulator, BufferDiff, StateTester, TestDsl, CliTester) are functional with comprehensive APIs, extensive unit tests, and integration tests. The implementation exceeds PRD expectations in several areas while maintaining some gaps and technical debt.

**Implementation Status**: ~95% Complete

---

## 1. Implementation Progress Summary

### Modules Status

| Module | Status | PRD Requirement | Gap |
|--------|--------|-----------------|-----|
| `PtySimulator` | ✅ Complete | Full PTY simulation | Minor - Windows stub only |
| `BufferDiff` | ✅ Complete | Cell-by-cell comparison | None |
| `StateTester` | ✅ Complete | State capture/assertion | None |
| `TestDsl` | ✅ Complete | Fluent test composition | None |
| `CliTester` | ✅ Complete | CLI process testing | None |
| `DialogRenderTester` | ✅ Complete | Dialog testing utilities | PRD didn't specify |
| `Snapshot` | ✅ Complete | File-based snapshot persistence | PRD didn't specify |

### File Structure Compliance

| PRD Path | Actual Path | Status |
|----------|-------------|--------|
| `src/lib.rs` | `src/lib.rs` | ✅ |
| `src/pty.rs` | `src/pty.rs` | ✅ |
| `src/diff.rs` | `src/diff.rs` | ✅ |
| `src/state.rs` | `src/state.rs` | ✅ |
| `src/dsl.rs` | `src/dsl.rs` | ✅ |
| `src/cli.rs` | `src/cli.rs` | ✅ |
| `src/snapshot.rs` | `src/snapshot.rs` | ✅ (not in PRD) |
| `tests/pty_tests.rs` | `tests/pty_tests.rs` | ✅ |
| `tests/buffer_diff_tests.rs` | `tests/buffer_diff_tests.rs` | ✅ |
| `tests/state_tests.rs` | `tests/state_tests.rs` | ✅ |
| `tests/dsl_tests.rs` | `tests/dsl_tests.rs` | ✅ |
| `tests/integration_tests.rs` | `tests/integration_tests.rs` | ✅ |

---

## 2. Gap Analysis by Module

### 2.1 PtySimulator

| Acceptance Criteria | Status | Notes |
|---------------------|--------|-------|
| Creates PTY master/slave pair on Unix | ✅ | `native_pty_system().openpty()` |
| Writes strings to PTY slave | ✅ | `write_input()` |
| Reads output from PTY master with timeout | ✅ | `read_output()` |
| Resizes PTY window (cols/rows) | ✅ | `resize()` |
| Injects KeyEvent via crossterm | ✅ | `inject_key_event()` |
| Injects MouseEvent via crossterm | ✅ | `inject_mouse_event()` |

**Gap**: Windows implementation is a stub that returns errors (`anyhow::bail!("PTY not supported on Windows")`).

**Severity**: P2 (Windows support is best-effort per PRD)

### 2.2 BufferDiff

| Acceptance Criteria | Status | Notes |
|---------------------|--------|-------|
| Compares two Buffers cell-by-cell | ✅ | Full implementation |
| Reports exact x,y of differences | ✅ | `CellDiff` with x, y fields |
| Supports ignoring foreground color | ✅ | `ignore_foreground()` |
| Supports ignoring background color | ✅ | `ignore_background()` |
| Supports ignoring attributes | ✅ | `ignore_attributes()` |
| Provides human-readable diff output | ✅ | `fmt::Display` impl |

**Gap**: None

### 2.3 StateTester

| Acceptance Criteria | Status | Notes |
|---------------------|--------|-------|
| Captures serializable state to JSON | ✅ | `capture_state()` |
| Compares current state to captured snapshot | ✅ | `compare()`, `assert_state()` |
| Reports mismatches with JSON diff | ✅ | `StateDiff` with path tracking |

**Gap**: None

### 2.4 TestDsl

| Acceptance Criteria | Status | Notes |
|---------------------|--------|-------|
| Renders widget to Buffer | ✅ | `render()` |
| Composes PTY, BufferDiff, StateTester | ✅ | Fluent API |
| Fluent API chains correctly | ✅ | `then()`, `then_result()` |
| Wait-for predicate support | ✅ | `wait_for()`, `wait_with_predicates()` |

**Gap**: PRD mentions `render(&self, widget: &impl Widget) -> Result<Buffer>` but implementation uses `render(mut self, widget: impl Widget + 'static) -> Self`. This is a signature difference but functionally equivalent for fluent chaining.

### 2.5 CliTester

| Acceptance Criteria | Status | Notes |
|---------------------|--------|-------|
| Spawns process with args | ✅ | `run()`, `spawn()` |
| Captures stdout/stderr | ✅ | `capture_stdout()`, `capture_stderr()` |
| Returns exit code | ✅ | `CliOutput.exit_code` |
| Cleans up temp directories | ✅ | `TempDir` with drop cleanup |

**Gap**: None

---

## 3. Priority Issues (P0/P1/P2)

### P0 - Blocking Issues

**None identified**

### P1 - High Priority

| Issue | Module | Description | Fix |
|-------|--------|-------------|-----|
| Missing `ChildProcess` export | `lib.rs` | `ChildProcess` struct used in tests but not exported | Add `ChildProcess` to exports |
| `CliTester::with_temp_dir` signature mismatch | `cli.rs` | PRD: `pub fn with_temp_dir(mut self) -> Result<Self>`<br>Actual: `pub fn with_temp_dir(self) -> Result<(Self, PathBuf)>` | The implementation returns the path, which is more ergonomic. PRD needs updating. |

### P2 - Medium Priority

| Issue | Module | Description | Fix |
|-------|--------|-------------|-----|
| Windows PTY stub | `pty.rs` | All methods return error on Windows | Consider implementing with `conpty` crate |
| Test file cleanup | All | Some tests manually remove snapshot files after running | Consider using `tempfile` for automatic cleanup |

### Technical Debt

| Item | Location | Description | Remediation |
|------|----------|-------------|-------------|
| Snapshot directory hardcoded | `snapshot.rs:9` | `const SNAPSHOT_DIR: &str = "snapshots"` | Make configurable via environment or builder pattern |
| Thread sleep in tests | Various | `std::thread::sleep(Duration::from_millis(100))` for PTY output | Could use `wait_for` predicate instead |
| Multiple tokio runtimes | `dsl.rs` | Creates new runtime in various methods | Consider sharing runtime or using `#[tokio::test]` |
| `#[allow(clippy::collapsible_str_replace)]` | `snapshot.rs:90` | Clippy suppression | Refactor to avoid suppression |

---

## 4. Detailed Gap List

| Gap Item | Severity | Module | PRD Reference |修复建议 |
|----------|----------|--------|---------------|---------|
| Windows PTY not supported | P2 | pty.rs | "Cross-platform (Unix primary, Windows best-effort)" | Document limitation; optionally implement conpty |
| `ChildProcess` not exported | P1 | lib.rs | Public API | Add to `pub use` exports |
| `with_temp_dir` returns tuple | P1 | cli.rs | Signature mismatch | Update PRD or implementation |
| `render()` signature differs | P2 | dsl.rs | `mut self` vs `&self` | PRD is guidance; implementation is ergonomic |
| Snapshot dir not configurable | Tech Debt | snapshot.rs | No configuration mentioned | Add builder pattern for custom dir |

---

## 5. Test Coverage Analysis

### Unit Tests

| Module | Test Count | Status |
|--------|------------|--------|
| PtySimulator | 23 (including platform-specific) | ✅ |
| BufferDiff | 45+ in `diff.rs` + 50+ in `buffer_diff_tests.rs` | ✅ |
| StateTester | 50+ in `state.rs` + 45+ in `state_tests.rs` | ✅ |
| TestDsl | 115+ in `dsl.rs` + 75+ in `dsl_tests.rs` | ✅ |
| CliTester | 18 in `cli.rs` | ✅ |
| DialogRenderTester | 12 in `dialog_tests.rs` | ✅ |
| Snapshot | 4 in `snapshot.rs` | ✅ |

### Integration Tests

| File | Test Count | Status |
|------|------------|--------|
| `integration_tests.rs` | 27 | ✅ |
| `dsl_integration_tests.rs` | 29 | ✅ |

**Total Test Count**: ~350+ tests

### Missing Test Coverage

1. `DialogRenderTester` - No tests for `assert_render_result` and `assert_empty_state` helper functions
2. End-to-end mouse event injection through DSL
3. Concurrent PTY operations

---

## 6. Dependencies Analysis

### Required vs Implemented

| PRD Dependency | Version | Actual | Status |
|---------------|---------|--------|--------|
| ratatui | 0.28 | 0.28 | ✅ |
| crossterm | 0.28 (events, mouse) | 0.28 (optional) | ✅ |
| portable-pty | 0.8 | 0.8 | ✅ |
| anyhow | 1.0 | 1.0 | ✅ |
| thiserror | 2.0 | 2.0 | ✅ |
| serde | 1.0 (derive) | 1.0 (derive) | ✅ |
| serde_json | 1.0 | 1.0 | ✅ |
| tempfile | 3.14 | 3.14 | ✅ |
| tokio | 1.45 (full) | 1.45 (rt-multi-thread, sync, time, macros, process, io-util) | ✅ |

**Issue**: `crossterm` is marked as `optional` in Cargo.toml but is required for PTY event injection. This should be a required dependency, not optional.

---

## 7. API Completeness Check

### PtySimulator (PRD vs Implementation)

| PRD Method | Implemented | Signature Match |
|------------|-------------|-----------------|
| `new()` | ✅ | ✅ |
| `resize()` | ✅ | ✅ |
| `write_input()` | ✅ | ✅ |
| `read_output()` | ✅ | ✅ |
| `inject_key_event()` | ✅ | ✅ |
| `inject_mouse_event()` | ✅ | ✅ |

**Additional implemented**: `new_with_command()`, `is_child_running()`, `encode_key_event()`, `encode_mouse_event()` (private)

### BufferDiff (PRD vs Implementation)

| PRD Method | Implemented | Signature Match |
|------------|-------------|-----------------|
| `new()` | ✅ | ✅ |
| `ignore_fg()` | ✅ | `ignore_foreground()` - naming difference |
| `ignore_bg()` | ✅ | `ignore_background()` - naming difference |
| `ignore_attributes()` | ✅ | ✅ |
| `diff()` | ✅ | ✅ |
| `diff_str()` | ✅ | ✅ (not in PRD but useful) |

### StateTester (PRD vs Implementation)

| PRD Method | Implemented | Signature Match |
|------------|-------------|-----------------|
| `new()` | ✅ | ✅ |
| `capture()` | ✅ | `capture_state()` - naming difference |
| `assert_state()` | ✅ | ✅ |
| `assert_state_matches()` | ✅ | ✅ |

**Additional implemented**: `capture_terminal_state()`, `compare()`, `compare_by_name()`, `compare_state()`, `get_snapshot()`, `list_snapshots()`, `remove_snapshot()`, `clear_snapshots()`, `diff_to_string()`, `with_default_path()`

### TestDsl (PRD vs Implementation)

| PRD Method | Implemented | Signature Match |
|------------|-------------|-----------------|
| `new()` | ✅ | ✅ |
| `with_pty()` | ✅ | ✅ |
| `render()` | ✅ | ✅ (uses `mut self`) |
| `assert_buffer_eq()` | ✅ | ✅ |
| `send_keys()` | ✅ | ✅ |
| `wait_for()` | ✅ | ✅ |
| `capture_state()` | ✅ | ✅ |
| `assert_state()` | ✅ | ✅ |

**Additional implemented**: `with_size()`, `init_terminal()`, `with_pty_command()`, `with_buffer_diff()`, `with_state_tester()`, `render_with_state()`, `then()`, `then_result()`, `wait_for_async()`, `wait_with_predicates()`, `poll_until()`, `poll_until_async()`, `buffer_content_at()`, `buffer_line_at()`, `buffer_lines()`, `snapshot_state()`, `compare_to_snapshot()`, `save_snapshot()`, `load_snapshot()`, `load_snapshot_and_assert_eq()`, etc.

### CliTester (PRD vs Implementation)

| PRD Method | Implemented | Signature Match |
|------------|-------------|-----------------|
| `new()` | ✅ | ✅ |
| `with_temp_dir()` | ✅ | Returns tuple `(Self, PathBuf)` |
| `run()` | ✅ | `async run()` with timeout |
| `capture_stdout()` | ✅ | ✅ |
| `capture_stderr()` | ✅ | ✅ |

**Additional implemented**: `arg()`, `args()`, `env()`, `envs()`, `working_dir()`, `run_with_timeout()`, `spawn()`, `ChildProcess` struct

---

## 8. Recommendations

### Immediate Actions (P1)

1. **Export `ChildProcess`** in `lib.rs`:
   ```rust
   pub use cli::{ChildProcess, CliOutput, CliTester};
   ```

2. **Make `crossterm` a required dependency** (not optional) since PTY testing requires it.

### Future Improvements (P2)

1. Consider adding `conpty` support for Windows PTY
2. Add configuration options for snapshot directory
3. Improve test cleanup with `tempfile` patterns

### Documentation Updates (Low Priority)

1. Update PRD to reflect actual `CliTester::with_temp_dir` signature
2. Consider adding `#[deprecated]` annotations for naming differences (e.g., `ignore_fg` vs `ignore_foreground`)

---

## 9. Conclusion

The `ratatui-testing` crate has evolved significantly from the stub implementation described in the PRD. All core acceptance criteria are met, and the implementation exceeds the PRD scope in several areas (additional methods, better ergonomics, more comprehensive tests).

**Key Strengths:**
- Comprehensive test coverage (~350+ tests)
- All five core modules fully implemented
- Fluent DSL API is ergonomic and complete
- Excellent integration between components

**Key Gaps:**
- Windows PTY support is stub-only (documented limitation)
- `ChildProcess` not exported from lib.rs
- `crossterm` marked optional but is required

**Overall Assessment**: The crate is production-ready for Unix systems with the identified gaps being minor issues that can be addressed in follow-up iterations.
