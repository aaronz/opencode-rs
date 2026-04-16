# Implementation Plan - Iteration 24
**Project:** ratatui-testing  
**Iteration:** 24  
**Date:** 2026-04-16  
**Status:** Updated based on gap analysis

---

## 1. Priority Classification

### P0 - Blocking (Must Fix)
None - All core functionality works.

### P1 - High Priority (API Alignment)

| FR | Task | Description | Effort |
|----|------|-------------|--------|
| FR-101 | Fix DiffResult struct | Add `passed`, `expected`, `actual` fields | Medium |
| FR-102 | Fix CellDiff struct | Use `ratatui::buffer::Cell` type for expected/actual | Medium |
| FR-104 | Add assert_buffer_eq | Implement missing DSL method | Low |

### P2 - Medium Priority

| FR | Task | Description | Effort |
|----|------|-------------|--------|
| FR-103 | Implement diff_str | String-based diff method | Low |
| FR-105 | Add send_keys | High-level key injection | Low |
| FR-106 | PtySimulator new() alignment | API signature differs from PRD | Low |
| FR-107 | Missing test files | Create 3 test files | Medium |
| FR-108 | Snapshot module | Create snapshot.rs | Medium |

---

## 2. Implementation Roadmap

### Phase 1: API Alignment (P1 Tasks)
**Objective:** Fix BufferDiff API to match PRD specification

#### Task 1.1: Fix DiffResult struct
- Add `passed: bool` field (computed from `total_diffs == 0`)
- Add `expected: Buffer` field
- Add `actual: Buffer` field
- Update `diff()` method to populate these fields
- Update `Display` impl to handle new fields

#### Task 1.2: Fix CellDiff struct
- Refactor to use `expected: Cell` and `actual: Cell` instead of individual fields
- Maintain backward compatibility via helper methods
- Update `Display` impl for human-readable output

#### Task 1.3: Add assert_buffer_eq to TestDsl
- Implement `assert_buffer_eq(&self, expected: &Buffer, actual: &Buffer) -> Result<()>`
- Use internal BufferDiff for comparison
- Return detailed error with diff info on failure

### Phase 2: Missing Features (P2 Tasks)
**Objective:** Complete implementation per PRD

#### Task 2.1: Implement diff_str method
- Add `diff_str(expected: &str, actual: &str) -> DiffResult`
- Parse string input into Buffer format
- Return DiffResult directly (not wrapped in Result)

#### Task 2.2: Add send_keys method
- Add `send_keys(&mut self, keys: &str) -> Result<&mut Self>`
- Parse key sequences (enter, escape, ctrl-x, etc.)
- Return `&mut Self` for fluent chaining

#### Task 2.3: PtySimulator new() alignment
- Add `new()` as alias for `new(&["bash", "-c", "echo ready"])`
- Or document that PRD should be updated to reflect actual API

### Phase 3: Test Coverage (P2 Tasks)
**Objective:** Complete integration test suite

#### Task 3.1: Create tests/buffer_diff_tests.rs
- Comprehensive BufferDiff tests
- CellDiff tests with Cell type
- diff_str tests
- ~200 lines

#### Task 3.2: Create tests/state_tests.rs
- StateTester specific tests
- Snapshot capture/compare tests
- ~200 lines

#### Task 3.3: Create tests/integration_tests.rs
- Cross-module integration tests
- TestDsl composition tests
- End-to-end workflow tests
- ~300 lines

### Phase 4: Snapshot Module (P2 Tasks)
**Objective:** Create snapshot.rs per PRD file structure

#### Task 4.1: Create src/snapshot.rs
- `load_snapshot(name: &str) -> Result<Buffer>`
- `save_snapshot(name: &str, buffer: &Buffer) -> Result<()>`
- Organized directory structure for snapshots

#### Task 4.2: Integrate with TestDsl
- Add `save_snapshot()` and `load_snapshot()` methods to TestDsl
- Support snapshot naming and versioning

---

## 3. File Changes Summary

### Modified Files
| File | Changes |
|------|---------|
| `src/diff.rs` | FR-101, FR-102, FR-103 |
| `src/dsl.rs` | FR-104, FR-105 |
| `src/pty.rs` | FR-106 |
| `src/lib.rs` | Export new types |

### New Files
| File | Purpose |
|------|---------|
| `src/snapshot.rs` | FR-108 |
| `tests/buffer_diff_tests.rs` | FR-107 |
| `tests/state_tests.rs` | FR-107 |
| `tests/integration_tests.rs` | FR-107 |

---

## 4. Dependencies

- No external dependencies required
- All changes are internal API alignment
- Snapshot module uses std::fs for file operations

---

## 5. Verification

### Must Pass (Release Blocker)
- [ ] `cargo build` succeeds
- [ ] `cargo test` passes
- [ ] BufferDiff API matches PRD
- [ ] TestDsl has assert_buffer_eq

### Should Pass (Quality Gate)
- [ ] diff_str method implemented
- [ ] send_keys method implemented
- [ ] All test files exist
- [ ] Snapshot module exists
