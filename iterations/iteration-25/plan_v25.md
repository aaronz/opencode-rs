# Implementation Plan v25 - ratatui-testing

## Overview

This plan outlines the implementation roadmap for `ratatui-testing` v2.5 based on gap analysis from iteration-25.

## P0 - Critical (Must Fix)

### 1. Create `tests/dsl_tests.rs`
- **Status**: MISSING (required by PRD)
- **File**: `opencode-rust/ratatui-testing/tests/dsl_tests.rs`
- **Required Tests**:
  - TestDsl construction and defaults
  - `with_size()` dimensions
  - `init_terminal()` initialization
  - `with_pty()` command parameter
  - `with_buffer_diff()` initialization
  - `with_state_tester()` initialization
  - `render()` widget rendering
  - `render_with_state()` stateful rendering
  - Fluent API method chaining
  - `capture_buffer()` retrieval
  - `send_keys()` key injection
  - `wait_for()` timeout behavior
  - `wait_with_predicates()` predicate handling
  - `buffer_lines()` content extraction
  - Snapshot save/load

### 2. Add `capture_stdout()` and `capture_stderr()` to CliTester
- **Status**: MISSING (required by PRD)
- **File**: `src/cli.rs`
- **Implementation**:
  ```rust
  pub fn capture_stdout(mut self) -> Self { /* ... */ }
  pub fn capture_stderr(mut self) -> Self { /* ... */ }
  ```
- **Note**: Output is already captured in `run()` - these are fluent configuration methods

## P1 - High Priority

### 3. Add no-parameter `with_pty()` to TestDsl
- **Status**: Gap FR-DSL-GAP-001
- **File**: `src/dsl.rs`
- **Implementation**: Add `pub fn with_pty(self) -> Result<Self>` that uses default command `["bash", "-c", "echo ready"]`
- **Reference**: PtySimulator already has `new()` that does this

### 4. Fix CliTester `temp_dir` field usage
- **Status**: Gap FR-CLI-GAP-001
- **File**: `src/cli.rs`
- **Issue**: `temp_dir` field exists but `with_temp_dir()` sets `working_dir` instead
- **Fix**: `working_dir` should use `temp_dir.path()` when no explicit working_dir set

### 5. Add `StateTester::compare_state()` method
- **Status**: Missing convenience API
- **File**: `src/state.rs`
- **Implementation**: Add `pub fn compare_state<S>(&self, state: &S) -> Result<StateDiff>` that serializes and compares

### 6. Fix `diff_str()` to use IgnoreOptions
- **Status**: Gap FR-DIFF-GAP-001
- **File**: `src/diff.rs`
- **Issue**: `diff_str()` bypasses IgnoreOptions and directly calls `diff()`

## P2 - Medium Priority

### 7. Enhance DialogRenderTester
- **Status**: Basic functionality only
- **File**: `src/dialog_tester.rs`
- **Enhancements**:
  - `has_title(buffer, title)` check
  - `has_specific_content(buffer, content)` check
  - `count_lines_with_content(buffer)` already exists

### 8. Fix `wait_for` to use `predicates` field
- **Status**: Redundant code
- **File**: `src/dsl.rs`
- **Issue**: `predicates` field only used by `wait_with_predicates()`, not by `wait_for()`

### 9. Export `ChildProcess` from lib.rs
- **Status**: Type not exported
- **File**: `src/lib.rs`
- **Issue**: `ChildProcess` defined in cli.rs but not publicly accessible

### 10. Add platform detection for PTY
- **Status**: No Windows detection
- **File**: `src/pty.rs`
- **Enhancement**: Add `#[cfg(unix)]` and Windows fallback error message

## Implementation Order

1. **P0**: Create `tests/dsl_tests.rs` - Required test coverage
2. **P0**: Add `capture_stdout()`/`capture_stderr()` to CliTester - Required API
3. **P1**: Add no-param `with_pty()` to TestDsl - Required by spec
4. **P1**: Fix `temp_dir` field usage in CliTester
5. **P1**: Add `compare_state()` to StateTester
6. **P1**: Fix `diff_str()` IgnoreOptions
7. **P2**: Export `ChildProcess` from lib.rs
8. **P2**: Enhance DialogRenderTester
9. **P2**: Fix `wait_for` predicates
10. **P2**: Add Windows platform detection

## Dependencies

All dependencies already present in `Cargo.toml`:
- ratatui 0.28
- crossterm 0.28
- portable-pty 0.8
- tokio 1.45
- serde 1.0
- tempfile 3.14

## Verification

After each P0 task:
```bash
cd opencode-rust/ratatui-testing && cargo test
```

Final verification:
```bash
cargo test --all-features
cargo clippy --all-targets --all-features -- -D warnings
```