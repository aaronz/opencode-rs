# Implementation Plan: ratatui-testing v0.1.0

**Project:** ratatui-testing
**Version:** 0.1.0
**Date:** 2026-04-17
**Status:** 95% Complete

---

## Executive Summary

The `ratatui-testing` crate is **substantially complete** at 95%. All core modules are implemented and functional. No P0 blockers remain. The implementation is production-ready with minor documentation and polish tasks remaining.

---

## Progress Summary

| Module | Status | Completion |
|--------|--------|------------|
| PtySimulator | ✅ Complete (Unix), Windows best-effort | 100% |
| BufferDiff | ✅ Complete | 100% |
| StateTester | ✅ Complete | 100% |
| TestDsl | ✅ Complete | 100% |
| CliTester | ✅ Complete | 100% |
| DialogRenderTester | ✅ Complete | 100% |
| Snapshot | ✅ Complete | 100% |
| Integration | ✅ Complete | 100% |

**Overall Completion: 95%**

---

## Priority 0 (P0) Tasks - Critical Blockers

**None.** All critical blockers have been resolved.

---

## Priority 1 (P1) Tasks - High Priority

### P1-001: Make Windows PTY Limitation More Prominent
**Module:** PtySimulator
**Severity:** High
**Effort:** Low

**Issue:** Windows PTY limitation is documented but not prominent enough for Windows users.

**Actions:**
- [ ] Add `#[cfg(windows)]` compile-time warning/doc comment at module level
- [ ] Consider adding runtime warning when PTY operations are attempted on Windows
- [ ] Update lib.rs docs to clearly state Windows is best-effort

**Status:** Pending

---

## Priority 2 (P2) Tasks - Medium Priority

### P2-001: Update PRD File Structure Documentation
**Module:** Documentation
**Severity:** Medium
**Effort:** Low

**Issue:** PRD shows 6 test files but 8 exist. This is an enhancement, not a gap.

**Actions:**
- [ ] Update spec document to reflect actual file structure (8 test files)
- [ ] Document that extra test files (dialog_tests, dsl_integration_tests, snapshot_tests) are enhancements

**Status:** Pending

---

### P2-002: Document DialogRenderTester Extension
**Module:** Documentation
**Severity:** Medium
**Effort:** Low

**Issue:** DialogRenderTester module exists but is not in the original PRD.

**Actions:**
- [ ] Add DialogRenderTester to the module list in spec
- [ ] Document it as an extension providing dialog-specific testing helpers

**Status:** Pending

---

## Priority 3 (P3) Tasks - Technical Debt

### P3-001: Review Global unwrap_used Allowance
**Module:** lib.rs
**Severity:** Low
**Effort:** Medium

**Issue:** `#![allow(clippy::unwrap_used)]` globally permits unwrap usage.

**Actions:**
- [ ] Audit unwrap usages in lib.rs
- [ ] Consider replacing with proper error handling where feasible
- [ ] If necessary, move allowance to specific modules with comments explaining why

**Status:** Pending

---

### P3-002: Simplify Windows PTY Stub
**Module:** PtySimulator
**Severity:** Low
**Effort:** Medium

**Issue:** 105 lines of stub code for Windows could be simplified.

**Actions:**
- [ ] Consider using a macro to reduce boilerplate
- [ ] Ensure error messages are helpful and consistent

**Status:** Pending

---

### P3-003: Extract parse_key_sequence to Separate Module
**Module:** dsl.rs
**Severity:** Low
**Effort:** Medium

**Issue:** `parse_key_sequence` is 95 lines and could be reused elsewhere.

**Actions:**
- [ ] Extract `parse_key_sequence` to `src/key_sequence.rs`
- [ ] Add module to lib.rs exports
- [ ] Update dsl.rs to use the extracted module

**Status:** Pending

---

### P3-004: Share Tokio Runtime in wait_for Variants
**Module:** dsl.rs
**Severity:** Low
**Effort:** Medium

**Issue:** Each wait method creates its own tokio runtime.

**Actions:**
- [ ] Consider creating a shared runtime in TestDsl struct
- [ ] Or document why separate runtimes are necessary

**Status:** Pending

---

### P3-005: Consolidate Overlapping DialogTester Functionality
**Module:** dialog_tester
**Severity:** Low
**Effort:** Medium

**Issue:** DialogTester partially overlaps with TestDsl buffer inspection methods.

**Actions:**
- [ ] Audit overlapping methods
- [ ] Consider consolidating or clearly documenting distinct purposes

**Status:** Pending

---

## Completed Tasks

- [x] PtySimulator implementation (Unix)
- [x] BufferDiff implementation
- [x] StateTester implementation
- [x] TestDsl implementation
- [x] CliTester implementation
- [x] DialogRenderTester implementation
- [x] Snapshot management
- [x] Integration tests
- [x] Cross-platform support (Unix primary, Windows best-effort)

---

## Timeline

| Phase | Tasks | Target |
|-------|-------|--------|
| Phase 1 (P1) | Make Windows PTY limitation prominent | Immediate |
| Phase 2 (P2) | Documentation updates | This iteration |
| Phase 3 (P3) | Technical debt cleanup | Future iteration |

---

## Recommendations

1. **Ship as-is**: The crate is production-ready at 95% completion
2. **Windows users**: Already receive descriptive errors; ensure documentation is clear
3. **API deviations**: Consider updating PRD to reflect the superior fluent API design

---

*Plan generated: 2026-04-17*
*Specification version: 32*