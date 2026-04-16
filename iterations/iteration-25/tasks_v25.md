# Task List v25 - ratatui-testing

## P0 - Critical Tasks

### Task 1: Create `tests/dsl_tests.rs`
- **Priority**: P0
- **Module**: tests
- **Files**: `opencode-rust/ratatui-testing/tests/dsl_tests.rs`
- **Description**: Create missing test file for TestDsl module
- **Requirements**:
  - [x] TestDsl construction (new, default)
  - [x] with_size() dimensions
  - [x] init_terminal() initialization
  - [x] with_pty() with command parameter
  - [x] with_buffer_diff() initialization
  - [x] with_state_tester() initialization
  - [x] render() widget rendering
  - [x] render_with_state() stateful rendering
  - [x] Fluent API method chaining (then, then_result)
  - [x] capture_buffer() retrieval
  - [x] send_keys() key injection
  - [x] wait_for() timeout behavior
  - [x] wait_with_predicates() predicate handling
  - [x] poll_until() polling behavior
  - [x] buffer_lines() content extraction
  - [x] save_snapshot() method
  - [x] load_snapshot() method
  - [x] load_snapshot_and_assert_eq() method
- **Verification**: `cargo test -p ratatui-testing dsl_tests` ✅ Done

### Task 2: Add capture_stdout() and capture_stderr() to CliTester
- **Priority**: P0
- **Module**: cli
- **Files**: `opencode-rust/ratatui-testing/src/cli.rs`
- **Description**: Add missing fluent methods for stdout/stderr capture configuration
- **Implementation**:
  - [ ] Add `capture_stdout(self) -> Self` method
  - [ ] Add `capture_stderr(self) -> Self` method
  - [ ] These configure output to be captured (already happens in run())
- **Verification**: `cargo test -p ratatui-testing cli`

## P1 - High Priority Tasks

### Task 3: Add no-parameter with_pty() to TestDsl
- **Priority**: P1
- **Module**: dsl
- **Files**: `opencode-rust/ratatui-testing/src/dsl.rs`
- **Description**: Add with_pty() without command parameter (gap FR-DSL-GAP-001)
- **Implementation**:
  - [ ] Add `pub fn with_pty(self) -> Result<Self>`
  - [ ] Use default command `["bash", "-c", "echo ready"]`
- **Verification**: `cargo test -p ratatui-testing dsl`
- **Reference**: PtySimulator::new() already does this

### Task 4: Fix CliTester temp_dir field usage
- **Priority**: P1
- **Module**: cli
- **Files**: `opencode-rust/ratatui-testing/src/cli.rs`
- **Description**: Fix gap FR-CLI-GAP-001 where temp_dir is never used
- **Implementation**:
  - [ ] When `with_temp_dir()` is called, store TempDir in `temp_dir` field
  - [ ] In `run()` and `spawn()`, use `temp_dir.path()` as working_dir if no explicit working_dir set
- **Verification**: `cargo test -p ratatui-testing cli`

### Task 5: Add StateTester::compare_state() method
- **Priority**: P1
- **Module**: state
- **Files**: `opencode-rust/ratatui-testing/src/state.rs`
- **Description**: Add convenience method to directly compare serializable state
- **Implementation**:
  - [ ] Add `pub fn compare_state<S>(&self, state: &S) -> Result<StateDiff>`
  - [ ] Serialize state to Value, then compare to default snapshot
- **Verification**: `cargo test -p ratatui-testing state`

### Task 6: Fix diff_str() to use IgnoreOptions
- **Priority**: P1
- **Module**: diff
- **Files**: `opencode-rust/ratatui-testing/src/diff.rs`
- **Description**: Fix gap FR-DIFF-GAP-001
- **Implementation**:
  - [ ] Review diff_str() implementation
  - [ ] Ensure IgnoreOptions are applied when comparing strings
- **Verification**: `cargo test -p ratatui-testing buffer_diff`

## P2 - Medium Priority Tasks

### Task 7: Export ChildProcess from lib.rs
- **Priority**: P2
- **Module**: lib
- **Files**: `opencode-rust/ratatui-testing/src/lib.rs`
- **Description**: Export ChildProcess type publicly (gap: N/A - not exported)
- **Implementation**:
  - [ ] Add `pub use cli::ChildProcess;` to lib.rs
- **Verification**: `cargo build -p ratatui-testing`

### Task 8: Enhance DialogRenderTester
- **Priority**: P2
- **Module**: dialog_tester
- **Files**: `opencode-rust/ratatui-testing/src/dialog_tester.rs`
- **Description**: Add more dialog verification methods
- **Implementation**:
  - [x] Add `has_title(buffer, title) -> bool`
  - [x] Add `has_specific_content(buffer, content) -> bool`
  - [x] Review existing methods for completeness
- **Verification**: `cargo test -p ratatui-testing dialog` ✅ Done

### Task 9: Fix wait_for to use predicates field
- **Priority**: P2
- **Module**: dsl
- **Files**: `opencode-rust/ratatui-testing/src/dsl.rs`
- **Description**: Refactor wait_for to optionally use predicates
- **Implementation**:
  - [x] Review wait_for implementation
  - [x] Consider integrating predicates field usage
  - [x] Document if predicates are intentionally separate
- **Verification**: `cargo test -p ratatui-testing dsl` ✅ Done

### Task 10: Add Windows platform detection for PTY
- **Priority**: P2
- **Module**: pty
- **Files**: `opencode-rust/ratatui-testing/src/pty.rs`
- **Description**: Add Windows detection and clear error message
- **Implementation**:
  - [x] Add `#[cfg(windows)]` check in PtySimulator::new()
  - [x] Return clear error on Windows: "PTY not supported on Windows"
  - [x] Use `#[cfg(unix)]` for Unix-specific code
- **Verification**: Build on different platforms ✅ Done

## Verification Commands

```bash
# Run all tests
cd opencode-rust/ratatui-testing && cargo test

# Run tests for specific module
cargo test -p ratatui-testing dsl_tests
cargo test -p ratatui-testing cli
cargo test -p ratatui-testing state
cargo test -p ratatui-testing buffer_diff

# Run clippy
cargo clippy --all-targets --all-features -- -D warnings

# Build
cargo build --all-features
```

## Status Summary

| Task | Priority | Status | Dependencies |
|------|----------|--------|-------------|
| 1. Create dsl_tests.rs | P0 | ✅ Done | - |
| 2. CliTester capture methods | P0 | TODO | - |
| 3. with_pty() no-param | P1 | TODO | Task 1 |
| 4. temp_dir field fix | P1 | TODO | - |
| 5. compare_state() | P1 | TODO | - |
| 6. diff_str() IgnoreOptions | P1 | TODO | - |
| 7. Export ChildProcess | P2 | TODO | - |
| 8. Enhance DialogRenderTester | P2 | ✅ Done | - |
| 9. wait_for predicates | P2 | TODO | - |
| 10. Windows platform detection | P2 | ✅ Done | - |