# Implementation Plan: ratatui-testing v0.1.0

**Project:** ratatui-testing
**Version:** 0.1.0
**Date:** 2026-04-17
**Iteration:** 31
**Status:** Implementation Complete (98%)

---

## 1. Overview

This plan addresses remaining gaps identified in the ratatui-testing specification. The implementation is 98% complete with only minor issues remaining.

## 2. Priority Classification

### P0 - Blocking Issues
**None identified.** All core acceptance criteria are met.

### P1 - High Priority (Address First)

| ID | Issue | Module | Impact |
|----|-------|--------|--------|
| GAP-001 | PtySimulator `read_output()` breaks after first read, missing buffered data | pty.rs:130 | Tests may miss output arriving in multiple chunks |

### P2 - Medium Priority (Address After P1)

| ID | Issue | Module | Impact |
|----|-------|--------|--------|
| GAP-002 | Missing `similar-asserts` dev-dependency | Cargo.toml | Dev workflow mismatch with PRD |
| GAP-003 | DialogTester module not documented in PRD | dialog_tester.rs | Documentation gap |
| GAP-004 | Dead code warnings in dialog_tester.rs | dialog_tester.rs:92,100 | Code cleanliness |
| GAP-005 | snapshot.rs module not in PRD | snapshot.rs | Documentation gap |

## 3. Implementation Tasks

### P1 Tasks (High Priority)

#### TASK-P1-001: Fix PtySimulator read_output buffer drain issue
**Issue:** GAP-001
**Location:** `src/pty.rs:122-141`
**Estimated Time:** 15 minutes

**Problem:**
The `read_output()` function breaks after the first successful read at line 130:
```rust
Ok(n) => {
    buffer.extend_from_slice(&temp_buf[..n]);
    break;  // <-- Premature break
}
```

**Fix Required:**
Remove the premature `break` statement to allow the loop to continue draining the buffer until timeout is reached or reader returns 0 (EOF).

**Acceptance Criteria:**
- [ ] `read_output()` continues reading until timeout
- [ ] All buffered data is captured, not just the first chunk
- [ ] Tests that rely on multi-chunk output pass correctly
- [ ] Existing tests continue to pass

### P2 Tasks (Medium Priority)

#### TASK-P2-001: Add missing dev-dependency
**Issue:** GAP-002
**Location:** `Cargo.toml`
**Estimated Time:** 1 minute

**Fix Required:**
Add `similar-asserts = "1.5"` to `[dev-dependencies]` section.

**Acceptance Criteria:**
- [ ] Cargo.toml includes `similar-asserts = "1.5"` in dev-dependencies
- [ ] `cargo check` passes
- [ ] Dev workflow matches PRD specification

#### TASK-P2-002: Document DialogTester in PRD
**Issue:** GAP-003
**Location:** `dialog_tester.rs`, `spec_v31.md`
**Estimated Time:** 30 minutes

**Fix Required:**
Update PRD specification to include DialogRenderTester module documentation.

**Acceptance Criteria:**
- [ ] PRD includes DialogRenderTester API documentation
- [ ] FR-DIALOG-001 section added or updated
- [ ] File structure in PRD matches actual implementation

#### TASK-P2-003: Remove or use dead code
**Issue:** GAP-004
**Location:** `dialog_tester.rs:92,100`
**Estimated Time:** 5 minutes

**Fix Required:**
Either:
1. Remove `assert_render_result` and `assert_empty_state` functions if not needed, OR
2. Use these functions in tests to eliminate warnings

**Acceptance Criteria:**
- [ ] No `#[allow(dead_code)]` decorators remain
- [ ] No compiler warnings about dead code
- [ ] Code cleanliness maintained

#### TASK-P2-004: Document snapshot.rs in PRD
**Issue:** GAP-005
**Location:** `snapshot.rs`, `spec_v31.md`
**Estimated Time:** 10 minutes

**Fix Required:**
Update PRD specification to include snapshot module.

**Acceptance Criteria:**
- [ ] FR-SNAP-001 section added to PRD
- [ ] Snapshot API documented
- [ ] File structure updated

## 4. Technical Debt

| ID | Description | Estimated Effort | Priority |
|----|-------------|------------------|----------|
| TD-001 | Fix PtySimulator read loop to drain buffer completely | 15 min | P1 |
| TD-002 | Add `similar-asserts = "1.5"` to dev-dependencies | 1 min | P2 |
| TD-003 | Review DialogTester - document or remove | 30 min | P2 |
| TD-004 | Remove or use `#[allow(dead_code)]` functions | 5 min | P2 |
| TD-005 | Update PRD to include snapshot.rs | 10 min | P2 |

## 5. Implementation Order

1. **TASK-P1-001** - Fix PtySimulator read_output (P1 - blocking issue)
2. **TASK-P2-001** - Add similar-asserts dev-dependency (P2 - quick fix)
3. **TASK-P2-002** - Document DialogTester in PRD (P2 - documentation)
4. **TASK-P2-003** - Remove dead code warnings (P2 - cleanup)
5. **TASK-P2-004** - Document snapshot.rs in PRD (P2 - documentation)

## 6. Verification

After completing all tasks:
- [ ] Run `cargo build --all-features` - must succeed
- [ ] Run `cargo test` - all tests must pass
- [ ] Run `cargo clippy --all-targets --all-features -- -D warnings` - no warnings
- [ ] Run `cargo fmt --all -- --check` - formatting correct

## 7. Dependencies

No new dependencies required (except already-listed similar-asserts).

## 8. Risks

| Risk | Mitigation |
|------|------------|
| Fixing read loop breaks existing tests | Run full test suite after change |
| Removing dead code breaks downstream | Verify no external usage |

---

*Plan version: 31.1*
*Generated: 2026-04-17*