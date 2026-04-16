# Implementation Plan v26: ratatui-testing

**Date**: 2026-04-17  
**Iteration**: 26  
**Status**: ~95% Complete

---

## 1. Executive Summary

The `ratatui-testing` crate is substantially implemented with all five core modules functional. Remaining work focuses on closing P1 gaps identified in the gap analysis.

---

## 2. Priority Classification

### P0 - Blocking Issues
**None identified** - No blocking issues prevent continued development.

### P1 - High Priority (Must Fix)

| Gap ID | Issue | Module | Impact |
|--------|-------|--------|--------|
| FR-DIFF-GAP-001 | `diff_str()` does not apply IgnoreOptions | `diff.rs` | String comparison ignores builder options |
| FR-SNAP-GAP-001 | Snapshot directory hardcoded to "snapshots" | `snapshot.rs` | Inflexible snapshot directory |
| DEP-001 | `crossterm` marked optional but is required | `Cargo.toml` | PTY testing requires crossterm |

### P2 - Medium Priority (Should Fix)

| Gap ID | Issue | Module | Impact |
|--------|-------|--------|--------|
| FR-PTY-GAP-001 | Windows PTY stub returns errors | `pty.rs` | Windows PTY testing not supported |
| FR-DIALOG-GAP-001 | DialogRenderTester tests incomplete | `dialog_tests.rs` | Limited dialog verification coverage |

---

## 3. Implementation Status

### Completed Modules

| Module | Status | Notes |
|--------|--------|-------|
| PtySimulator | ✅ Complete | All FR-PTY-* acceptance criteria met |
| BufferDiff | ✅ Complete | All FR-DIFF-* acceptance criteria met, 1 gap |
| StateTester | ✅ Complete | All FR-STATE-* acceptance criteria met |
| TestDsl | ✅ Complete | All FR-DSL-* acceptance criteria met |
| CliTester | ✅ Complete | All FR-CLI-* acceptance criteria met |
| DialogRenderTester | ✅ Complete | Additional module not in original PRD |
| Snapshot | ✅ Complete | Additional module not in original PRD |

### Resolved Issues (This Iteration)

| Gap ID | Issue | Resolution |
|--------|-------|------------|
| FR-LIB-GAP-001 | `ChildProcess` not exported | Fixed - now exported in lib.rs |
| FR-CLI-GAP-001 | `temp_dir` not used in working_dir | Fixed - now used when no explicit working_dir |
| FR-CLI-GAP-002 | Missing `capture_stdout()`/`capture_stderr()` | Fixed - methods now implemented |
| FR-DSL-GAP-001 | `with_pty()` signature mismatch | Resolved - signature correct as `with_pty(command)` |
| FR-DSL-GAP-002 | `dsl_tests.rs` missing | Fixed - file now exists with 1144 lines |

---

## 4. Required Work

### 4.1 Fix: `diff_str()` IgnoreOptions (FR-DIFF-GAP-001)

**Location**: `src/diff.rs`

**Problem**: The `diff_str()` method parses strings to Buffers but does not apply the IgnoreOptions configured on the builder.

**Required Change**:
```rust
// Current (ignores IgnoreOptions):
pub fn diff_str(&self, expected: &str, actual: &str) -> DiffResult {
    // Parse strings to buffers, then call diff() which does use IgnoreOptions
}

// Need to ensure diff_str applies the same IgnoreOptions logic
```

**Fix Approach**: Modify `diff_str()` to apply ignore options at the Cell comparison level, not just Buffer level.

---

### 4.2 Fix: Snapshot Directory Configuration (FR-SNAP-GAP-001)

**Location**: `src/snapshot.rs:9`

**Problem**: `const SNAPSHOT_DIR: &str = "snapshots"` is hardcoded and not configurable.

**Fix Approach**: Add builder pattern or environment variable override:
```rust
// Option A: Environment variable
const SNAPSHOT_DIR: &str = env::var("RATATUI_TESTING_SNAPSHOT_DIR")
    .unwrap_or_else(|_| "snapshots".to_string());

// Option B: Builder pattern
pub struct SnapshotManager {
    dir: PathBuf,
}
```

---

### 4.3 Fix: Make `crossterm` Required Dependency

**Location**: `Cargo.toml`

**Problem**: `crossterm` is marked `optional` in Cargo.toml but is required for PTY event injection.

**Fix**:
```toml
[dependencies]
crossterm = "0.28"  # Required for PTY event injection
```

Remove `optional = true` from crossterm entry.

---

## 5. File Structure Compliance

| PRD Path | Actual Path | Status |
|----------|-------------|--------|
| `src/lib.rs` | `src/lib.rs` | ✅ |
| `src/pty.rs` | `src/pty.rs` | ✅ |
| `src/diff.rs` | `src/diff.rs` | ✅ |
| `src/state.rs` | `src/state.rs` | ✅ |
| `src/dsl.rs` | `src/dsl.rs` | ✅ |
| `src/cli.rs` | `src/cli.rs` | ✅ |
| `src/snapshot.rs` | `src/snapshot.rs` | ✅ (additional) |
| `tests/pty_tests.rs` | `tests/pty_tests.rs` | ✅ |
| `tests/buffer_diff_tests.rs` | `tests/buffer_diff_tests.rs` | ✅ |
| `tests/state_tests.rs` | `tests/state_tests.rs` | ✅ |
| `tests/dsl_tests.rs` | `tests/dsl_tests.rs` | ✅ |
| `tests/integration_tests.rs` | `tests/integration_tests.rs` | ✅ |

---

## 6. Dependencies Compliance

| PRD Dependency | Required | Status |
|----------------|----------|--------|
| ratatui | 0.28 | ✅ |
| crossterm | 0.28 | ✅ (needs to be required, not optional) |
| portable-pty | 0.8 | ✅ |
| anyhow | 1.0 | ✅ |
| thiserror | 2.0 | ✅ |
| serde | 1.0 | ✅ |
| serde_json | 1.0 | ✅ |
| tempfile | 3.14 | ✅ |
| tokio | 1.45 | ✅ |

---

## 7. Acceptance Criteria Status

### All Core Acceptance Criteria Met

- ✅ PtySimulator: FR-PTY-001 through FR-PTY-008
- ✅ BufferDiff: FR-DIFF-001 through FR-DIFF-008 (except FR-DIFF-GAP-001)
- ✅ StateTester: FR-STATE-001 through FR-STATE-014
- ✅ TestDsl: FR-DSL-001 through FR-DSL-028
- ✅ CliTester: FR-CLI-001 through FR-CLI-018

---

## 8. Test Coverage Status

| Test File | Lines | Status |
|-----------|-------|--------|
| `tests/pty_tests.rs` | 363 | ✅ Complete |
| `tests/buffer_diff_tests.rs` | 513 | ✅ Complete |
| `tests/state_tests.rs` | 592 | ✅ Complete |
| `tests/dsl_tests.rs` | 1144 | ✅ Complete |
| `tests/dsl_integration_tests.rs` | 650 | ✅ Complete |
| `tests/dialog_tests.rs` | 199 | ⚠️ Partial |
| `tests/integration_tests.rs` | 647 | ✅ Complete |

**Total Test Code**: ~4108 lines  
**Estimated Coverage**: 95%+

---

## 9. Timeline

| Phase | Tasks | Est. Time |
|-------|-------|-----------|
| P1 Fixes | FR-DIFF-GAP-001, FR-SNAP-GAP-001, DEP-001 | 2-3 hours |
| P2 Improvements | Windows PTY docs, dialog tests | 1-2 hours |
| Final Review | All tests pass, clippy clean | 30 min |

---

## 10. Next Steps

1. **Immediate**: Fix the 3 P1 issues identified
2. **Verify**: Run `cargo test --all-features` to ensure all tests pass
3. **Validate**: Run `cargo clippy` for lint issues
4. **Document**: Update any remaining gaps in spec

---

*End of Plan v26*
