# Specification Document - Iteration 24

**Project:** ratatui-testing
**Iteration:** 24
**Date:** 2026-04-16
**Phase:** ratatui-testing Implementation
**PRD Reference:** /Users/openclaw/Documents/github/opencode-rs/PRD.md

---

## Table of Contents

1. [Executive Summary](#1-executive-summary)
2. [Gap Analysis Summary](#2-gap-analysis-summary)
3. [Feature Requirements (FR)](#3-feature-requirements-fr)
4. [P0 - Blocking Issues](#4-p0---blocking-issues)
5. [P1 - High Priority Issues](#5-p1---high-priority-issues)
6. [P2 - Medium Priority Issues](#6-p2---medium-priority-issues)
7. [Technical Debt](#7-technical-debt)
8. [Implementation Roadmap](#8-implementation-roadmap)
9. [Acceptance Criteria](#9-acceptance-criteria)
10. [Cross-References](#10-cross-references)

---

## 1. Executive Summary

### Overall Status

| Category | Compliance | Gap Severity |
|----------|------------|--------------|
| PtySimulator | ✅ Complete | P2 |
| BufferDiff | ⚠️ Partial | P1 |
| StateTester | ✅ Complete | - |
| TestDsl | ⚠️ Partial | P1 |
| CliTester | ✅ Complete | - |

### Implementation Progress

| Module | Status | PRD Compliance |
|--------|--------|----------------|
| PtySimulator | ✅ Complete | 95% |
| BufferDiff | ⚠️ Partial | 70% |
| StateTester | ✅ Complete | 100% |
| TestDsl | ✅ Complete | 90% |
| CliTester | ✅ Complete | 100% |

### Key Metrics

| Metric | Current State | Target |
|--------|---------------|--------|
| Implementation Completion | ~85% | 100% |
| API Compliance | ~75% | 100% |
| Test Coverage | ~60% | 80%+ |

---

## 2. Gap Analysis Summary

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

### Missing Components

| Item | Severity | Description |
|------|----------|-------------|
| `DiffResult.passed` field | P1 | PRD defines `passed: bool` but impl has `total_diffs: usize` |
| `DiffResult.expected/actual` | P1 | PRD includes full Buffer references in DiffResult |
| `CellDiff` struct | P1 | PRD defines `expected: Cell, actual: Cell` but impl uses individual fields |
| `diff_str()` method | P2 | Impl has `diff_to_string()` not `diff_str(expected: &str, actual: &str)` |
| `assert_buffer_eq()` | P1 | PRD shows method not implemented |
| `send_keys()` | P2 | PRD shows method not implemented |
| Test files | P2 | Missing `buffer_diff_tests.rs`, `state_tests.rs`, `integration_tests.rs` |
| `snapshot.rs` module | P2 | Referenced in PRD file structure but not created |

---

## 3. Feature Requirements (FR)

| FR-ID | Feature | Priority | Status |
|-------|---------|----------|--------|
| FR-101 | BufferDiff API Alignment - DiffResult | P1 | Not Compliant |
| FR-102 | BufferDiff API Alignment - CellDiff | P1 | Not Compliant |
| FR-103 | BufferDiff diff_str Method | P2 | Not Implemented |
| FR-104 | TestDsl assert_buffer_eq Method | P1 | Implemented |
| FR-105 | TestDsl send_keys Method | P2 | Not Implemented |
| FR-106 | PtySimulator new() Signature Alignment | P2 | Not Compliant |
| FR-107 | Missing Test Files | P2 | Not Complete |
| FR-108 | Snapshot Module | P2 | Not Created |

---

### FR-101: BufferDiff API Alignment - DiffResult

**Priority:** P1 (High)
**Status:** Not Compliant

#### PRD Specification

```rust
pub struct DiffResult {
    pub passed: bool,
    pub expected: Buffer,
    pub actual: Buffer,
    pub differences: Vec<CellDiff>,
}
```

#### Current Implementation

```rust
pub struct DiffResult {
    pub differences: Vec<CellDiff>,
    pub total_diffs: usize,
}
```

#### Required Changes

1. Add `passed: bool` field computed from `total_diffs == 0`
2. Add `expected: Buffer` field storing the expected buffer reference
3. Add `actual: Buffer` field storing the actual buffer reference

#### Acceptance Criteria

- [ ] `DiffResult.passed` returns `true` when `total_diffs == 0`
- [ ] `DiffResult.expected` contains reference to expected Buffer
- [ ] `DiffResult.actual` contains reference to actual Buffer

---

### FR-102: BufferDiff API Alignment - CellDiff

**Priority:** P1 (High)
**Status:** Not Compliant

#### PRD Specification

```rust
pub struct CellDiff {
    pub x: u16,
    pub y: u16,
    pub expected: Cell,
    pub actual: Cell,
}
```

#### Current Implementation

```rust
pub struct CellDiff {
    pub x: u16,
    pub y: u16,
    pub expected_symbol: String,
    pub actual_symbol: String,
    pub expected_foreground: Option<Color>,
    pub actual_foreground: Option<Color>,
    pub expected_background: Option<Color>,
    pub actual_background: Option<Color>,
    pub expected_modifier: Option<Modifier>,
    pub actual_modifier: Option<Modifier>,
}
```

#### Required Changes

1. Refactor to use `ratatui::buffer::Cell` type for `expected` and `actual` fields
2. Keep `x` and `y` for position
3. Maintain backward compatibility via builder methods or separate struct

#### Acceptance Criteria

- [ ] `CellDiff` uses `ratatui::buffer::Cell` type for expected/actual
- [ ] Position (x, y) is still accessible
- [ ] Human-readable diff output is maintained

---

### FR-103: BufferDiff diff_str Method

**Priority:** P2 (Medium)
**Status:** Not Implemented

#### PRD Specification

```rust
pub fn diff_str(&self, expected: &str, actual: &str) -> DiffResult;
```

#### Current Implementation

Has `diff_to_string(&self, expected: &Buffer, actual: &Buffer) -> Result<String>` but not `diff_str(expected: &str, actual: &str) -> DiffResult`.

#### Required Changes

1. Implement `diff_str(expected: &str, actual: &str) -> DiffResult`
2. Parse string input into Buffer format
3. Return DiffResult directly (not wrapped in Result)

#### Acceptance Criteria

- [ ] `diff_str` accepts string inputs
- [ ] Returns `DiffResult` with `passed: true` for identical content
- [ ] Parses multi-line strings correctly

---

### FR-104: TestDsl assert_buffer_eq Method

**Priority:** P1 (High)
**Status:** Implemented

#### PRD Specification

```rust
pub fn assert_buffer_eq(&self, expected: &Buffer, actual: &Buffer) -> Result<()>;
```

#### Current Implementation

Has `assert_no_diffs(&self, expected: &Buffer)` and `assert_buffer_matches(&self, expected: &Buffer, options: IgnoreOptions)`.

#### Required Changes

1. Add `assert_buffer_eq(expected: &Buffer, actual: &Buffer) -> Result<()>`
2. Method should use internal BufferDiff to compare
3. Return error with diff details on failure

#### Acceptance Criteria

- [x] Method accepts two Buffer parameters
- [x] Returns `Ok(())` when buffers are identical
- [x] Returns error with diff details when different
- [x] Fluent API chainable

---

### FR-105: TestDsl send_keys Method

**Priority:** P2 (Medium)
**Status:** Not Implemented

#### PRD Specification

```rust
pub fn send_keys(&mut self, keys: &str) -> Result<&mut Self>;
```

#### Current Implementation

Has `write_to_pty(&mut self, input: &str) -> Result<()>` but not `send_keys`.

#### Required Changes

1. Add `send_keys(keys: &str) -> Result<&mut Self>`
2. Takes string input (e.g., "hello\n", "ctrl-c")
3. Returns `&mut Self` for chaining
4. Internally uses PTY for injection

#### Acceptance Criteria

- [ ] Method accepts string keys input
- [ ] Returns `&mut Self` for fluent chaining
- [ ] Keys are injected via PTY
- [ ] Common key sequences supported (enter, escape, etc.)

---

### FR-106: PtySimulator new() Signature Alignment

**Priority:** P2 (Medium)
**Status:** Not Compliant

#### PRD Specification

```rust
pub fn new() -> Result<Self>;
```

#### Current Implementation

```rust
pub fn new(command: &[&str]) -> Result<Self>;
```

#### Required Changes

1. Create a parameterless `new()` as alias or add `new_default()` 
2. Or update PRD to reflect actual API requiring command

#### Acceptance Criteria

- [ ] Either `new()` works without arguments OR
- [ ] PRD is updated to reflect `new(command: &[&str])` signature

---

### FR-107: Missing Test Files

**Priority:** P2 (Medium)
**Status:** Not Complete

#### Required Test Files

| File | Description | Lines |
|------|-------------|-------|
| `tests/buffer_diff_tests.rs` | BufferDiff specific tests | ~200 |
| `tests/state_tests.rs` | StateTester specific tests | ~200 |
| `tests/integration_tests.rs` | Cross-module integration tests | ~300 |

#### Acceptance Criteria

- [ ] `tests/buffer_diff_tests.rs` created with comprehensive tests
- [ ] `tests/state_tests.rs` created with comprehensive tests
- [ ] `tests/integration_tests.rs` created with cross-module tests

---

### FR-108: Snapshot Module

**Priority:** P2 (Medium)
**Status:** Not Created

#### PRD Specification

```text
ratatui-testing/
├── src/
│   ├── snapshot.rs     # Snapshot management
```

#### Required Functionality

1. Load snapshots from disk
2. Save snapshots to disk
3. Compare current state to stored snapshots
4. Snapshot naming and versioning

#### Acceptance Criteria

- [ ] `src/snapshot.rs` module created
- [ ] `load_snapshot(name: &str) -> Result<Buffer>`
- [ ] `save_snapshot(name: &str, buffer: &Buffer) -> Result<()>`
- [ ] Snapshots stored in organized directory structure

---

## 4. P0 - Blocking Issues

None - All core functionality works.

---

## 5. P1 - High Priority Issues

### P1-001: BufferDiff DiffResult Struct Mismatch

**Issue:** Missing `passed`, `expected`, `actual` fields in DiffResult

**Risk:** API incompatibility with PRD specification

**Fix:**
1. Add `passed: bool` computed field
2. Add `expected: Buffer` and `actual: Buffer` fields
3. Update `diff()` method to populate these fields

### P1-002: CellDiff Struct Mismatch

**Issue:** CellDiff should use `ratatui::buffer::Cell` type

**Risk:** API incompatibility with PRD specification

**Fix:**
1. Refactor CellDiff to use `expected: Cell` and `actual: Cell`
2. Maintain detailed field access via Cell properties
3. Update Display impl for human-readable output

### P1-003: TestDsl Missing assert_buffer_eq

**Issue:** Key fluent API method not implemented

**Risk:** Missing high-level assertion method

**Fix:**
1. Add `assert_buffer_eq(&self, expected: &Buffer, actual: &Buffer) -> Result<()>`
2. Use internal BufferDiff for comparison
3. Return detailed error on failure

---

## 6. P2 - Medium Priority Issues

| Issue | Description | Fix Effort | FR Reference |
|-------|-------------|------------|--------------|
| diff_str method | Missing string-based diff method | Low | FR-103 |
| Test files | Missing 3 test files | Medium | FR-107 |
| PtySimulator new() | API signature differs from PRD | Low | FR-106 |
| Snapshot module | Not created | Medium | FR-108 |
| send_keys method | Missing high-level key injection | Low | FR-105 |

---

## 7. Technical Debt

### Debt Summary

| Debt Item | Severity | Est. Effort | Dependencies | FR Reference |
|-----------|----------|-------------|--------------|--------------|
| BufferDiff API alignment | P1 | Medium | None | FR-101, FR-102 |
| Missing test files | P2 | Medium | None | FR-107 |
| Snapshot module | P2 | Medium | None | FR-108 |
| PtySimulator new() signature | P2 | Low | PRD decision | FR-106 |

---

## 8. Implementation Roadmap

### Phase 1: API Alignment (Current Sprint)

| Task | Deliverable | FR Reference |
|------|-------------|--------------|
| Fix DiffResult struct | Added passed, expected, actual | FR-101 |
| Fix CellDiff struct | Use ratatui Cell type | FR-102 |
| Add assert_buffer_eq | Method implemented | FR-104 |

### Phase 2: Missing Features (Next Sprint)

| Task | Deliverable | FR Reference |
|------|-------------|--------------|
| Implement diff_str | String-based diff | FR-103 |
| Add send_keys method | Key injection method | FR-105 |
| PtySimulator new() | Signature alignment | FR-106 |

### Phase 3: Test Coverage

| Task | Deliverable | FR Reference |
|------|-------------|--------------|
| buffer_diff_tests.rs | Comprehensive tests | FR-107 |
| state_tests.rs | Comprehensive tests | FR-107 |
| integration_tests.rs | Cross-module tests | FR-107 |

### Phase 4: Snapshot Module

| Task | Deliverable | FR Reference |
|------|-------------|--------------|
| Create snapshot.rs | Module with load/save | FR-108 |
| Integration with TestDsl | Snapshot support in DSL | FR-108 |

---

## 9. Acceptance Criteria

### Must Pass (Release Blocker)

| Criteria | Verification | FR Reference |
|----------|--------------|--------------|
| BufferDiff API matches PRD | Code review | FR-101, FR-102 |
| TestDsl has assert_buffer_eq | Code review | FR-104 |
| `cargo test` passes | Exit code 0 | All |

### Should Pass (Quality Gate)

| Criteria | Verification | FR Reference |
|----------|--------------|--------------|
| diff_str method implemented | Unit tests | FR-103 |
| send_keys method implemented | Unit tests | FR-105 |
| All test files exist | File check | FR-107 |
| Snapshot module exists | File check | FR-108 |

### Nice to Have (Polish)

| Criteria | Verification | FR Reference |
|----------|--------------|--------------|
| PtySimulator new() alignment | API consistency | FR-106 |
| 80%+ test coverage | Coverage report | FR-107 |

---

## 10. Cross-References

### PRD Cross-References

| Document | Topic |
|----------|-------|
| [TUI System](./09-tui-system.md) | TUI layout, keybindings, views |
| [TUI Plugin API](./15-tui-plugin-api.md) | TUI plugin configuration |
| [Rust Test Implementation Roadmap](./17-rust-test-implementation-roadmap.md) | Overall testing strategy |
| [Crate-by-Crate Test Backlog](./18-crate-by-crate-test-backlog.md) | Testing tasks per crate |

---

*Document generated from PRD: ratatui-testing and Gap Analysis Report*
*Iteration 24 - ratatui-testing Implementation*
