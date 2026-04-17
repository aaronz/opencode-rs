# Task List: ratatui-testing v0.1.0

**Project:** ratatui-testing
**Version:** 0.1.0
**Date:** 2026-04-17
**Status:** 95% Complete

---

## P0 - Critical Blockers

**None.** All critical blockers have been resolved.

---

## P1 - High Priority Tasks

### P1-001: Make Windows PTY Limitation More Prominent
**Module:** PtySimulator
**Priority:** P1
**Status:** Pending

**Issue:** Windows PTY limitation is documented but not prominent enough for Windows users.

**Acceptance Criteria:**
- [ ] `#[cfg(windows)]` compile-time warning at module level
- [ ] Runtime warning when PTY operations attempted on Windows
- [ ] Clear documentation in lib.rs about Windows best-effort status

**Subtasks:**
- [ ] Add prominent doc comment with `#[cfg(windows)]` at PtySimulator module entry
- [ ] Add runtime check in `new()` and `new_with_command()` that emits warning on Windows
- [ ] Update lib.rs module docs to clarify Windows support status

**Estimated Effort:** 2-3 hours

---

## P2 - Medium Priority Tasks

### P2-001: Update PRD File Structure Documentation
**Module:** Documentation
**Priority:** P2
**Status:** Pending

**Issue:** PRD shows 6 test files but 8 exist (dialog_tests, dsl_integration_tests, snapshot_tests are enhancements).

**Acceptance Criteria:**
- [ ] spec_v32.md reflects actual 8-file structure
- [ ] Extra test files documented as enhancements

**Subtasks:**
- [ ] Update File Structure section in spec to list all 8 test files
- [ ] Add note explaining extra files are enhancements beyond PRD

**Estimated Effort:** 1 hour

---

### P2-002: Document DialogRenderTester Extension
**Module:** Documentation
**Priority:** P2
**Status:** Pending

**Issue:** DialogRenderTester module exists but is not in the original PRD.

**Acceptance Criteria:**
- [ ] DialogRenderTester added to module list in spec
- [ ] Documented as extension providing dialog-specific testing helpers

**Subtasks:**
- [ ] Add FR-DIALOG-001 section to spec with DialogRenderTester details
- [ ] Update Module Status table to include DialogRenderTester

**Estimated Effort:** 1 hour

---

## P3 - Technical Debt Tasks

### P3-001: Review Global unwrap_used Allowance
**Module:** lib.rs
**Priority:** P3
**Status:** Pending

**Issue:** `#![allow(clippy::unwrap_used)]` globally permits unwrap usage throughout the crate.

**Acceptance Criteria:**
- [ ] Audit all unwrap usages
- [ ] Replace with proper error handling where feasible
- [ ] If allowance needed, move to specific modules with explanatory comments

**Subtasks:**
- [ ] Run `cargo clippy --all-targets -- -W clippy::unwrap_used` to find all usages
- [ ] For each unwrap, determine if it can be replaced with proper error handling
- [ ] Update lib.rs to remove global allowance if possible

**Estimated Effort:** 4-6 hours

---

### P3-002: Simplify Windows PTY Stub
**Module:** PtySimulator
**Priority:** P3
**Status:** Pending

**Issue:** 105 lines of stub code for Windows could be simplified with macros.

**Acceptance Criteria:**
- [ ] Reduce boilerplate in Windows stub implementation
- [ ] Ensure consistent, helpful error messages

**Subtasks:**
- [ ] Review current Windows stub implementation
- [ ] Create macro or helper to reduce repetition
- [ ] Verify error messages are descriptive

**Estimated Effort:** 2-3 hours

---

### P3-003: Extract parse_key_sequence to Separate Module
**Module:** dsl.rs
**Priority:** P3
**Status:** Pending

**Issue:** `parse_key_sequence` is 95 lines and could be reused elsewhere.

**Acceptance Criteria:**
- [ ] Function extracted to `src/key_sequence.rs`
- [ ] Module exported from lib.rs
- [ ] dsl.rs updated to use extracted module

**Subtasks:**
- [ ] Create `src/key_sequence.rs` with parse_key_sequence function
- [ ] Add `pub mod key_sequence;` to lib.rs
- [ ] Update dsl.rs to use `use crate::key_sequence::parse_key_sequence;`

**Estimated Effort:** 2-3 hours

---

### P3-004: Share Tokio Runtime in wait_for Variants
**Module:** dsl.rs
**Priority:** P3
**Status:** Pending

**Issue:** Each wait method creates its own tokio runtime, which is inefficient.

**Acceptance Criteria:**
- [ ] Investigate why separate runtimes are used
- [ ] Either share a runtime or document why separate ones are necessary

**Subtasks:**
- [ ] Audit wait_for, wait_for_async, poll_until, poll_until_async methods
- [ ] Determine if shared runtime is feasible
- [ ] Document decision in code comments

**Estimated Effort:** 3-4 hours

---

### P3-005: Consolidate Overlapping DialogTester Functionality
**Module:** dialog_tester
**Priority:** P3
**Status:** Pending

**Issue:** DialogTester partially overlaps with TestDsl buffer inspection methods.

**Acceptance Criteria:**
- [ ] Identify overlapping methods
- [ ] Either consolidate or document distinct purposes clearly

**Subtasks:**
- [ ] Compare DialogRenderTester and TestDsl buffer methods
- [ ] Document the distinction or consolidate duplicates

**Estimated Effort:** 2-3 hours

---

## Task Summary

| ID | Task | Priority | Status | Effort |
|----|------|----------|--------|--------|
| P1-001 | Make Windows PTY Limitation Prominent | P1 | Pending | 2-3h |
| P2-001 | Update PRD File Structure Docs | P2 | Pending | 1h |
| P2-002 | Document DialogRenderTester Extension | P2 | Pending | 1h |
| P3-001 | Review Global unwrap_used Allowance | P3 | Pending | 4-6h |
| P3-002 | Simplify Windows PTY Stub | P3 | Pending | 2-3h |
| P3-003 | Extract parse_key_sequence Module | P3 | Pending | 2-3h |
| P3-004 | Share Tokio Runtime in wait_for | P3 | Pending | 3-4h |
| P3-005 | Consolidate DialogTester Overlap | P3 | Pending | 2-3h |

**Total Estimated Effort:** 18-24 hours

---

## Completed Tasks (from previous iterations)

- [x] PtySimulator implementation (Unix)
- [x] BufferDiff implementation
- [x] StateTester implementation
- [x] TestDsl implementation
- [x] CliTester implementation
- [x] DialogRenderTester implementation
- [x] Snapshot management
- [x] All module unit tests
- [x] Integration tests

---

## Next Steps

1. **Immediate**: Complete P1-001 to improve Windows user experience
2. **This iteration**: Complete P2-001 and P2-002 for documentation accuracy
3. **Future**: Address P3 technical debt items based on priority and available time

---

*Task list generated: 2026-04-17*
*Specification version: 32*