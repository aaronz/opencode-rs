# Task List - Iteration 24
**Project:** ratatui-testing  
**Date:** 2026-04-16  
**Total Tasks:** 11

---

## P1 - High Priority Tasks (API Alignment)

### Task 1.1: Fix DiffResult struct
**FR:** FR-101  
**Status:** ✅ Done  
**Priority:** P1  
**Effort:** Medium

**Description:**
- Add `passed: bool` field computed from `total_diffs == 0`
- Add `expected: Buffer` field storing expected buffer reference
- Add `actual: Buffer` field storing actual buffer reference
- Update `diff()` method to populate these fields
- Update `Display` impl

**Acceptance Criteria:**
- [ ] `DiffResult.passed` returns true when `total_diffs == 0`
- [ ] `DiffResult.expected` contains reference to expected Buffer
- [ ] `DiffResult.actual` contains reference to actual Buffer
- [ ] Existing tests still pass

---

### Task 1.2: Fix CellDiff struct
**FR:** FR-102  
**Status:** ✅ Done  
**Priority:** P1  
**Effort:** Medium

**Description:**
- Refactor to use `expected: Cell` and `actual: Cell` instead of individual fields
- Keep `x` and `y` for position
- Maintain backward compatibility via helper methods
- Update Display impl for human-readable output

**Acceptance Criteria:**
- [ ] `CellDiff` uses `ratatui::buffer::Cell` type for expected/actual
- [ ] Position (x, y) is still accessible
- [ ] Human-readable diff output is maintained
- [ ] Existing tests still pass

---

### Task 1.3: Add assert_buffer_eq to TestDsl
**FR:** FR-104  
**Status:** ✅ Done  
**Priority:** P1  
**Effort:** Low

**Description:**
- Add `assert_buffer_eq(&self, expected: &Buffer, actual: &Buffer) -> Result<()>`
- Use internal BufferDiff for comparison
- Return detailed error with diff info on failure

**Acceptance Criteria:**
- [x] Method accepts two Buffer parameters
- [x] Returns `Ok(())` when buffers are identical
- [x] Returns error with diff details when different
- [x] Existing tests still pass

---

## P2 - Medium Priority Tasks

### Task 2.1: Implement diff_str method
**FR:** FR-103  
**Status:** TODO  
**Priority:** P2  
**Effort:** Low

**Description:**
- Add `diff_str(expected: &str, actual: &str) -> DiffResult`
- Parse string input into Buffer format
- Return DiffResult directly (not wrapped in Result)

**Acceptance Criteria:**
- [ ] `diff_str` accepts string inputs
- [ ] Returns `DiffResult` with `passed: true` for identical content
- [ ] Parses multi-line strings correctly
- [ ] Unit tests pass

---

### Task 2.2: Add send_keys method
**FR:** FR-105  
**Status:** ✅ Done  
**Priority:** P2  
**Effort:** Low

**Description:**
- Add `send_keys(&mut self, keys: &str) -> Result<&mut Self>`
- Parse key sequences (enter, escape, ctrl-x, etc.)
- Return `&mut Self` for fluent chaining
- Internally uses PTY for injection

**Acceptance Criteria:**
- [x] Method accepts string keys input
- [x] Returns `&mut Self` for fluent chaining
- [x] Keys are injected via PTY
- [x] Common key sequences supported (enter, escape, etc.)
- [x] Unit tests pass

---

### Task 2.3: PtySimulator new() alignment
**FR:** FR-106  
**Status:** TODO  
**Priority:** P2  
**Effort:** Low

**Description:**
- Add `new()` as alias that creates a default PTY
- Uses `new(&["bash", "-c", "echo ready"])` or similar
- Or update PRD to reflect actual `new(command: &[&str])` signature

**Acceptance Criteria:**
- [ ] Either `new()` works without arguments OR
- [ ] PRD is updated to reflect `new(command: &[&str])` signature

---

### Task 2.4: Create tests/buffer_diff_tests.rs
**FR:** FR-107  
**Status:** TODO  
**Priority:** P2  
**Effort:** Medium

**Description:**
- Comprehensive BufferDiff tests
- CellDiff tests with Cell type
- diff_str tests
- ~200 lines

**Acceptance Criteria:**
- [ ] File created at `tests/buffer_diff_tests.rs`
- [ ] Comprehensive test coverage
- [ ] All tests pass

---

### Task 2.5: Create tests/state_tests.rs
**FR:** FR-107  
**Status:** TODO  
**Priority:** P2  
**Effort:** Medium

**Description:**
- StateTester specific tests
- Snapshot capture/compare tests
- ~200 lines

**Acceptance Criteria:**
- [ ] File created at `tests/state_tests.rs`
- [ ] Comprehensive test coverage
- [ ] All tests pass

---

### Task 2.6: Create tests/integration_tests.rs
**FR:** FR-107  
**Status:** TODO  
**Priority:** P2  
**Effort:** Medium

**Description:**
- Cross-module integration tests
- TestDsl composition tests
- End-to-end workflow tests
- ~300 lines

**Acceptance Criteria:**
- [ ] File created at `tests/integration_tests.rs`
- [ ] Cross-module tests pass
- [ ] All tests pass

---

### Task 2.7: Create snapshot.rs module
**FR:** FR-108  
**Status:** TODO  
**Priority:** P2  
**Effort:** Medium

**Description:**
- Create `src/snapshot.rs` module
- `load_snapshot(name: &str) -> Result<Buffer>`
- `save_snapshot(name: &str, buffer: &Buffer) -> Result<()>`
- Organized directory structure for snapshots

**Acceptance Criteria:**
- [ ] `src/snapshot.rs` module created
- [ ] `load_snapshot` function works
- [ ] `save_snapshot` function works
- [ ] Unit tests pass

---

### Task 2.8: Integrate snapshot with TestDsl
**FR:** FR-108  
**Status:** TODO  
**Priority:** P2  
**Effort:** Medium

**Description:**
- Add `save_snapshot()` method to TestDsl
- Add `load_snapshot()` method to TestDsl
- Support snapshot naming and versioning

**Acceptance Criteria:**
- [ ] `TestDsl::save_snapshot()` works
- [ ] `TestDsl::load_snapshot()` works
- [ ] Fluent API chaining works
- [ ] Integration tests pass

---

## Summary

| Priority | Tasks | Status |
|----------|-------|--------|
| P1 | 3 | TODO |
| P2 | 8 | TODO |
| **Total** | **11** | - |

---

## Phase Breakdown

| Phase | Tasks | Description |
|-------|-------|-------------|
| Phase 1 | 1.1, 1.2, 1.3 | API Alignment |
| Phase 2 | 2.1, 2.2, 2.3 | Missing Features |
| Phase 3 | 2.4, 2.5, 2.6 | Test Coverage |
| Phase 4 | 2.7, 2.8 | Snapshot Module |
