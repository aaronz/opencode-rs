# Gap Analysis Report: ratatui-testing

**Project:** ratatui-testing
**Date:** 2026-04-17
**Status:** Implementation Complete (PRD Conformance: ~95%)

---

## Executive Summary

The `ratatui-testing` crate implementation is **substantially complete** and aligns well with the PRD requirements. All five core modules (PtySimulator, BufferDiff, StateTester, TestDsl, CliTester) are fully implemented with functional APIs matching the PRD specifications. The main deviation is the addition of `DialogRenderTester` which exceeds the PRD scope.

**Overall Completion:** ~95%

---

## 1. Gap List

| Gap Item | Severity | Module | Description |修复建议 |
|----------|----------|--------|-------------|---------|
| Missing `similar-asserts` dev dependency | P2 | Cargo.toml | PRD specifies `similar-asserts = "1.5"` as dev-dependency but it's not in Cargo.toml | Add to dev-dependencies if snapshot comparison needs visual diff |
| `dialog_tester.rs` not in PRD file structure | P2 | Extra Module | Implements `DialogRenderTester` which is useful but not specified in PRD | Document as PRD extension or move to separate module |
| `ChildProcess` export not in PRD | P2 | cli.rs | `ChildProcess` struct exported from lib.rs but not documented in PRD CliTester section | Add to PRD or remove export if not needed |
| `wait_for_async` not explicitly in PRD | P2 | dsl.rs | PRD shows `wait_for(timeout, predicate)` but implementation has `wait_for_async` variant | Add `wait_for_async` to PRD if async testing is required |
| No snapshot file format versioning | P2 | snapshot.rs | Snapshots store Cell data but no version field for format migration | Add version field to SerializedBuffer struct |

---

## 2. P0/P1/P2 Issue Classification

### P0 - Blocking Issues
**None identified.** All core acceptance criteria are met.

### P1 - High Priority Issues
| Issue | Module | Description |
|-------|--------|-------------|
| Windows PTY returns generic errors | pty.rs:281-386 | Windows implementation returns errors but doesn't provide actionable guidance. However, this is documented as a known limitation in PRD. |

### P2 - Medium Priority Issues
| Issue | Module | Description |
|-------|--------|-------------|
| Missing `similar-asserts` dev-dep | Cargo.toml | Dev tooling not included |
| Extra module not in PRD | dialog_tester.rs | Useful extension but not documented |
| Missing async wait API in PRD | dsl.rs | `wait_for_async` exists but not in PRD |
| Snapshot versioning | snapshot.rs | No format version for migrations |

---

## 3. Technical Debt

| Item | Location | Description | Impact |
|------|----------|-------------|--------|
| `#[allow(clippy::unwrap_used)]` | lib.rs:1 | Global clippy allowance for tests | Low - test code only |
| Hardcoded timeout (30s) | cli.rs:86 | `run()` uses hardcoded 30s timeout | Medium - should be configurable |
| `poll_until` creates new runtime | dsl.rs:442-470 | Each `poll_until` call creates a new tokio runtime | Medium - resource inefficiency |
| `wait_for_async` spawns thread | dsl.rs:364-379 | Creates thread per call | Medium - thread overhead |
| `with_backend` is public but internal | dialog_tester.rs:11 | `with_backend` exposed publicly | Low - utility method |
| Windows PTY stub has extensive docs | pty.rs:244-280 | 40+ lines of docs for stub | Low - documentation overhead |
| Test cleanup via `ok()` | Multiple test files | `std::fs::remove_file(...).ok()` for cleanup | Low - best-effort cleanup |

---

## 4. Implementation Progress Summary

### Module-by-Module Assessment

#### PtySimulator ✅ Complete (100%)
- [x] Creates PTY master/slave pair on Unix
- [x] Writes strings to PTY slave
- [x] Reads output from PTY master with timeout
- [x] Resizes PTY window (cols/rows)
- [x] Injects KeyEvent via crossterm
- [x] Injects MouseEvent via crossterm
- [x] Unix implementation fully functional
- [x] Windows stub with descriptive errors

#### BufferDiff ✅ Complete (100%)
- [x] Compares two Buffers cell-by-cell
- [x] Reports exact x,y of differences
- [x] Supports ignoring foreground color
- [x] Supports ignoring background color
- [x] Supports ignoring attributes (bold, italic, etc.)
- [x] Provides human-readable diff output
- [x] String-based comparison (diff_str)
- [x] 40+ unit tests covering all features

#### StateTester ✅ Complete (100%)
- [x] Captures serializable state to JSON
- [x] Compares current state to captured snapshot
- [x] Reports mismatches with JSON diff
- [x] Named snapshots support
- [x] Terminal state capture (from Buffer)
- [x] Clear/remove snapshots API
- [x] 30+ unit tests

#### TestDsl ✅ Complete (100%)
- [x] Renders widget to Buffer
- [x] Composes PTY, BufferDiff, StateTester
- [x] Fluent API chains correctly
- [x] Wait-for predicate support
- [x] Async wait support
- [x] Snapshot save/load integration
- [x] 70+ unit tests

#### CliTester ✅ Complete (100%)
- [x] Spawns process with args
- [x] Captures stdout/stderr
- [x] Returns exit code
- [x] Cleans up temp directories
- [x] Working directory support
- [x] Environment variable support
- [x] Output assertion helpers
- [x] 20+ unit tests

#### Snapshot Management ✅ Complete (100%)
- [x] Save/load Buffer snapshots
- [x] Color/style serialization
- [x] Configurable directory via env var
- [x] Path sanitization
- [x] 6+ unit tests

#### DialogRenderTester ⚠️ Extra (Not in PRD)
- [x] Border detection
- [x] Content checking
- [x] Title detection
- [x] Line counting utilities
- [x] Useful for TUI dialog testing

---

## 5. Dependencies Analysis

### Required by PRD ✅ All Present
```toml
ratatui = "0.28"           ✅
crossterm = "0.28"         ✅
portable-pty = "0.8"       ✅
anyhow = "1.0"             ✅
thiserror = "2.0"          ✅
serde = "1.0"              ✅
serde_json = "1.0"         ✅
tempfile = "3.14"          ✅
tokio = "1.45"             ✅
```

### Not Present (Optional)
```toml
similar-asserts = "1.5"    ⚠️ Missing (dev-dep, optional)
```

---

## 6. File Structure Comparison

### Expected (PRD) vs Actual

| Expected | Actual | Status |
|----------|--------|--------|
| Cargo.toml | Cargo.toml | ✅ |
| src/lib.rs | src/lib.rs | ✅ |
| src/pty.rs | src/pty.rs | ✅ |
| src/diff.rs | src/diff.rs | ✅ |
| src/state.rs | src/state.rs | ✅ |
| src/dsl.rs | src/dsl.rs | ✅ |
| src/cli.rs | src/cli.rs | ✅ |
| src/snapshot.rs | src/snapshot.rs | ✅ |
| - | src/dialog_tester.rs | ⚠️ Extra |
| tests/pty_tests.rs | tests/pty_tests.rs | ✅ |
| tests/buffer_diff_tests.rs | tests/buffer_diff_tests.rs | ✅ |
| tests/state_tests.rs | tests/state_tests.rs | ✅ |
| tests/dsl_tests.rs | tests/dsl_tests.rs | ✅ |
| tests/integration_tests.rs | tests/integration_tests.rs | ✅ |
| - | tests/dialog_tests.rs | ⚠️ Extra |
| - | tests/snapshot_tests.rs | ⚠️ Extra |
| - | tests/dsl_integration_tests.rs | ⚠️ Extra |

---

## 7. Recommendations

### Immediate (P1)
None required - all blocking issues resolved.

### Short-term (P2)
1. Add `similar-asserts = "1.5"` to dev-dependencies if visual snapshot diffing is desired
2. Consider adding `version` field to snapshot format for forward compatibility
3. Document `wait_for_async` in PRD or add to acceptance criteria
4. Evaluate whether `DialogRenderTester` should be formally added to PRD

### Long-term (Future)
1. Implement Windows ConPTY support (acknowledged as difficult in PRD)
2. Add snapshot diffing with `similar-asserts` for better test output
3. Consider async/await first-class support in TestDsl

---

## 8. Conclusion

The `ratatui-testing` crate is **production-ready** with all PRD acceptance criteria met. The implementation exceeds the PRD scope in some areas (DialogRenderTester, additional async methods) and has minor gaps (missing dev-dependency). No blocking issues exist.

**Recommendation:** Approve for integration. Address P2 items in follow-up iteration if desired.