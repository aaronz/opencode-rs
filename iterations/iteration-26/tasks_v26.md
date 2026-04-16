# Task List v26: ratatui-testing

**Date**: 2026-04-17  
**Iteration**: 26  
**Total Tasks**: 7  
**P0 Tasks**: 0  
**P1 Tasks**: 3  
**P2 Tasks**: 2  
**Completed/Resolved**: 2

---

## P0 - Blocking Issues
**None** - No blocking issues identified.

---

## P1 - High Priority Tasks

### Task 1: Fix `diff_str()` IgnoreOptions Bug
**Gap ID**: FR-DIFF-GAP-001  
**Priority**: P1  
**Module**: `src/diff.rs`

**Description**: The `diff_str()` method parses strings to Buffers but does not apply IgnoreOptions configured on the builder. String comparison ignores builder options.

**Status**: ✅ Done

**Acceptance Criteria**:
- [x] `diff_str()` applies `ignore_foreground` when set
- [x] `diff_str()` applies `ignore_background` when set  
- [x] `diff_str()` applies `ignore_attributes` when set
- [x] Test added to verify IgnoreOptions are applied in `diff_str()`

**Files Modified**:
- `src/diff.rs`

**Verification**:
```bash
cargo test -p ratatui-testing diff_str  # All 12 tests pass
```

---

### Task 2: Make Snapshot Directory Configurable
**Gap ID**: FR-SNAP-GAP-001  
**Priority**: P1  
**Module**: `src/snapshot.rs`

**Description**: `SNAPSHOT_DIR` is hardcoded to `"snapshots"` and is not configurable.

**Acceptance Criteria**:
- [ ] Snapshot directory can be configured via environment variable `RATATUI_TESTING_SNAPSHOT_DIR`
- [ ] Default remains `"snapshots"` when env var not set
- [ ] Existing `load_snapshot()` and `save_snapshot()` work without changes
- [ ] Test added to verify configuration works

**Files to Modify**:
- `src/snapshot.rs`

**Verification**:
```bash
cargo test -p ratatui-testing snapshot
```

---

### Task 3: Make `crossterm` a Required Dependency
**Gap ID**: DEP-001  
**Priority**: P1  
**Module**: `Cargo.toml`

**Description**: `crossterm` is marked `optional = true` in Cargo.toml but is required for PTY event injection functionality.

**Acceptance Criteria**:
- [ ] `crossterm` removed from optional features in Cargo.toml
- [ ] `crossterm` remains as a required dependency with version "0.28"
- [ ] All tests pass without changes to source code
- [ ] Documentation updated to reflect crossterm as required

**Files to Modify**:
- `Cargo.toml`

**Verification**:
```bash
cargo build -p ratatui-testing --all-features
cargo test -p ratatui-testing
```

---

## P2 - Medium Priority Tasks

### Task 4: Document Windows PTY Limitation
**Gap ID**: FR-PTY-GAP-001  
**Priority**: P2  
**Module**: `src/pty.rs`

**Description**: Windows PTY implementation is a stub that returns errors. This is a documented limitation per PRD, but should be clearly documented in code.

**Acceptance Criteria**:
- [ ] `#[cfg(windows)]` implementation has clear doc comment explaining limitation
- [ ] Error message is helpful and points to documentation
- [ ] PRD reference included in comments

**Files to Modify**:
- `src/pty.rs`

---

### Task 5: Complete DialogRenderTester Tests
**Gap ID**: FR-DIALOG-GAP-001  
**Priority**: P2  
**Module**: `tests/dialog_tests.rs`

**Description**: DialogRenderTester tests are incomplete. Missing tests for `assert_render_result()` and `assert_empty_state()` helper functions.

**Acceptance Criteria**:
- [ ] Test for `assert_render_result()` helper function
- [ ] Test for `assert_empty_state()` helper function
- [ ] Tests verify border and content presence as specified in FR-DIALOG-007 and FR-DIALOG-008

**Files to Modify**:
- `tests/dialog_tests.rs`

**Verification**:
```bash
cargo test -p ratatui-testing dialog
```

---

## Resolved Tasks

### Task 6: Export `ChildProcess` in lib.rs
**Gap ID**: FR-LIB-GAP-001  
**Status**: ✅ RESOLVED  
**Notes**: `ChildProcess` is now exported in `lib.rs` per spec.

---

### Task 7: Fix `with_temp_dir` and `capture_stdout/capture_stderr`
**Gap ID**: FR-CLI-GAP-001, FR-CLI-GAP-002  
**Status**: ✅ RESOLVED  
**Notes**: 
- `with_temp_dir()` returns `(Self, PathBuf)` tuple as per implementation
- `capture_stdout()` and `capture_stderr()` methods are now implemented

---

## Task Summary

| # | Task | Priority | Status | Est. Time |
|---|------|----------|--------|-----------|
| 1 | Fix `diff_str()` IgnoreOptions bug | P1 | ✅ Done | 1-2 hrs |
| 2 | Make snapshot dir configurable | P1 | Open | 30 min |
| 3 | Make crossterm required dependency | P1 | Open | 15 min |
| 4 | Document Windows PTY limitation | P2 | Open | 15 min |
| 5 | Complete DialogRenderTester tests | P2 | Open | 1 hr |
| 6 | Export ChildProcess | - | ✅ Done | - |
| 7 | Fix temp_dir/capture methods | - | ✅ Done | - |

---

## Verification Commands

```bash
# Run all tests
cargo test -p ratatui-testing --all-features

# Run clippy
cargo clippy -p ratatui-testing --all-targets -- -D warnings

# Run doc tests
cargo test -p ratatui-testing --doc

# Format check
cargo fmt --all -- --check
```

---

*End of Task List v26*
