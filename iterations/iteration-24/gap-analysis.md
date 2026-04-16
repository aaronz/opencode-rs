# Gap Analysis Report: ratatui-testing

**Project:** ratatui-testing  
**Date:** 2026-04-16  
**PRD Reference:** /Users/openclaw/Documents/github/opencode-rs/PRD.md

---

## 1. Implementation Progress Summary

| Module | Status | PRD Compliance |
|--------|--------|----------------|
| PtySimulator | ✅ Complete | 95% |
| BufferDiff | ⚠️ Partial | 70% |
| StateTester | ✅ Complete | 100% |
| TestDsl | ✅ Complete | 90% |
| CliTester | ✅ Complete | 100% |
| DialogRenderTester | 🔧 Extra | N/A |

### Files Created
```
ratatui-testing/
├── Cargo.toml              ✅
├── src/
│   ├── lib.rs              ✅
│   ├── pty.rs              ✅ (279 lines)
│   ├── diff.rs             ✅ (522 lines)
│   ├── state.rs            ✅ (792 lines)
│   ├── dsl.rs              ✅ (1028 lines)
│   ├── cli.rs              ✅ (396 lines)
│   └── dialog_tester.rs    🔧 (57 lines, extra)
└── tests/
    ├── pty_tests.rs        ✅ (186 lines)
    └── dsl_integration_tests.rs  ✅ (499 lines)
```

### Missing Test Files (per PRD)
- ❌ `tests/buffer_diff_tests.rs`
- ❌ `tests/state_tests.rs`
- ❌ `tests/integration_tests.rs`

---

## 2. Gap Analysis by Module

### 2.1 PtySimulator

| Gap | Severity | Description | Fix Suggestion |
|-----|----------|-------------|----------------|
| API Signature Mismatch | P1 | PRD shows `new()` but impl requires `new(command: &[&str])` | Update PRD to reflect actual API or create overloaded constructor |
| Missing pixel_width/pixel_height resize | P2 | `resize()` uses 0 for pixel dimensions | Allow configuring pixel dimensions in resize |
| No Window Title Support | P2 | Missing `set_window_title()` method | Add method to set PTY window title |

**Acceptance Criteria Status:**
- ✅ Creates PTY master/slave pair on Unix
- ✅ Writes strings to PTY slave
- ✅ Reads output from PTY master with timeout
- ✅ Resizes PTY window (cols/rows)
- ✅ Injects KeyEvent via crossterm
- ✅ Injects MouseEvent via crossterm

---

### 2.2 BufferDiff

| Gap | Severity | Description | Fix Suggestion |
|-----|----------|-------------|----------------|
| DiffResult missing `passed` field | P1 | PRD defines `passed: bool` but impl has `total_diffs: usize` | Add `passed` field computed from `total_diffs == 0` |
| DiffResult missing `expected`/`actual` Buffer | P1 | PRD includes full Buffer references in DiffResult | Add `expected: Buffer` and `actual: Buffer` fields |
| CellDiff uses simplified struct | P1 | PRD defines `expected: Cell, actual: Cell` but impl uses individual fields | Refactor to use ratatui Cell type |
| Missing `diff_str()` method | P2 | Impl has `diff_to_string()` not `diff_str(expected: &str, actual: &str)` | Implement string-based diff that parses to Buffer |
| `diff_to_string` signature mismatch | P2 | Impl method differs from PRD specification | Consider alias or deprecate in favor of PRD signature |

**Acceptance Criteria Status:**
- ✅ Compares two Buffers cell-by-cell
- ✅ Reports exact x,y of differences
- ✅ Supports ignoring foreground color
- ✅ Supports ignoring background color
- ✅ Supports ignoring attributes (bold, italic, etc.)
- ✅ Provides human-readable diff output

---

### 2.3 StateTester

| Gap | Severity | Description | Fix Suggestion |
|-----|----------|-------------|----------------|
| None | - | Fully compliant with PRD | N/A |

**Acceptance Criteria Status:**
- ✅ Captures serializable state to JSON
- ✅ Compares current state to captured snapshot
- ✅ Reports mismatches with JSON diff

---

### 2.4 TestDsl

| Gap | Severity | Description | Fix Suggestion |
|-----|----------|-------------|----------------|
| Missing `render()` widget trait bound | P1 | Impl `render()` takes `impl Widget + 'static` but PRD shows generic | Verify widget trait compatibility |
| Missing `assert_buffer_eq()` method | P1 | PRD shows `assert_buffer_eq(&self, expected: &Buffer, actual: &Buffer)` | Add method that wraps diff functionality |
| Missing `send_keys()` method | P2 | PRD shows `send_keys(&mut self, keys: &str),` not implemented | Add `send_keys()` that injects key sequence |

**Acceptance Criteria Status:**
- ✅ Renders widget to Buffer
- ✅ Composes PTY, BufferDiff, StateTester
- ✅ Fluent API chains correctly
- ⚠️ Wait-for predicate support (partially implemented via `wait_for`, `wait_with_predicates`)

---

### 2.5 CliTester

| Gap | Severity | Description | Fix Suggestion |
|-----|----------|-------------|----------------|
| None | - | Fully compliant with PRD | N/A |

**Acceptance Criteria Status:**
- ✅ Spawns process with args
- ✅ Captures stdout/stderr
- ✅ Returns exit code
- ✅ Cleans up temp directories

---

## 3. P0/P1/P2 Issue Classification

### P0 - Blockers (Must Fix)
None - All core functionality works

### P1 - High Priority
1. **BufferDiff DiffResult Struct Mismatch** - Missing `passed`, `expected`, `actual` fields
2. **CellDiff Struct Mismatch** - Should use `ratatui::buffer::Cell` type
3. **TestDsl Missing `assert_buffer_eq()`** - Key method from fluent API
4. **TestDsl Missing `send_keys()`** - High-level key injection method

### P2 - Medium Priority
1. **Missing `diff_str()` method** - String-based buffer diff utility
2. **Missing test files** - `buffer_diff_tests.rs`, `state_tests.rs`, `integration_tests.rs`
3. **PtySimulator `new()` signature** - API differs from PRD
4. **Missing `snapshot.rs` module** - Referenced in PRD file structure

---

## 4. Technical Debt清单

| Item | Type | Description | Effort |
|------|------|-------------|--------|
| BufferDiff API alignment | Debt | Refactor DiffResult to match PRD | Medium |
| Missing integration tests | Debt | Add 3 missing test files | Medium |
| DialogRenderTester module | Extra | Not in PRD, but functional | Low |
| PtySimulator constructor | Debt | Add parameterless `new()` as alias | Low |

---

## 5. Missing Components

### 5.1 Missing Module
- `snapshot.rs` - Mentioned in PRD file structure but not created

### 5.2 Missing Test Files
- `tests/buffer_diff_tests.rs` - BufferDiff specific tests
- `tests/state_tests.rs` - StateTester specific tests
- `tests/integration_tests.rs` - Cross-module integration tests

### 5.3 Missing Features (from integration examples in PRD)
- `TestDsl::render(&self, widget: &impl Widget)` - Takes reference, not owned
- `TestDsl::assert_buffer_eq()` - Direct buffer equality assertion
- `TestDsl::send_keys()` - Convenience method for key injection
- `BufferDiff::diff_str()` - String-to-buffer diff

---

## 6. Recommendations

### Immediate (Current Sprint)
1. Add `passed`, `expected`, `actual` fields to `DiffResult`
2. Add `assert_buffer_eq()` method to `TestDsl`
3. Add `send_keys()` method to `TestDsl`

### Short-term (Next Sprint)
1. Create missing test files
2. Implement `diff_str()` method
3. Align `PtySimulator::new()` API with PRD

### Medium-term
1. Consider adding `snapshot.rs` module for snapshot management
2. Add pixel_width/pixel_height configuration to PTY resize
3. Add window title support to PtySimulator

---

## 7. Overall Assessment

**Implementation Completion:** ~85%  
**API Compliance:** ~75%  
**Test Coverage:** ~60% (missing 3 test files)

The ratatui-testing crate is **functionally complete** for core use cases but has **API alignment gaps** with the PRD. The implementation is production-quality with proper error handling and unit tests, but integration test coverage is incomplete.

**Strengths:**
- PtySimulator fully implemented with cross-platform support
- StateTester fully compliant
- CliTester fully compliant
- Comprehensive unit tests in each module
- Proper error handling with anyhow/thiserror

**Weaknesses:**
- BufferDiff API diverges from PRD
- Missing high-level DSL convenience methods
- Incomplete integration test suite
