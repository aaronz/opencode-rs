# Implementation Plan v9

**Version:** 9.0  
**Generated:** 2026-04-12  
**Based on:** Spec v9 and Gap Analysis Iteration 9  
**Status:** Draft

---

## 1. Overview

This document tracks the implementation plan for the OpenCode Rust port based on Iteration 9 gap analysis.

**Overall Completion Estimate: ~90-92%**  
**Phase Status:** Phase 5-6 of 6 (Hardening, Release Qualification)

---

## 2. Phase Status Summary

| Phase | Description | Status | Coverage |
|-------|-------------|--------|----------|
| Phase 0 | Project Foundation | ✅ Complete | 100% |
| Phase 1 | Authority Implementation | ✅ Complete | ~99% |
| Phase 2 | Runtime Core | ✅ Complete | ~99% |
| Phase 3 | Infrastructure Subsystems | ✅ Complete | ~98% |
| Phase 4 | Interface Implementations | ✅ Complete | ~90% |
| Phase 5 | Hardening | 🚧 In Progress | ~85% |
| Phase 6 | Release Qualification | 🚧 Partial | ~70% |

---

## 3. Release Gates Status

| Gate | Criteria | Status | Notes |
|------|----------|--------|-------|
| Phase 0 | Workspace builds, tests run, clippy clean | ❌ | Clippy fails (P0-9) |
| Phase 1 | Authority tests green (01, 06, 07) | ✅ | All 4 suites pass |
| Phase 2 | Runtime tests green (02, 03, 08, 15) | ✅ | All 5 suites pass |
| Phase 3 | Subsystem tests green (04, 05, 10, 11, 12) | ✅ | All 4 suites pass |
| Phase 4 | Interface smoke workflows pass (13, 14) | ✅ | Desktop WebView done |
| Phase 5a | Compatibility suite green | ✅ | All 3 suites pass |
| Phase 5b | Conventions suite green | ✅ | All 23 tests pass |
| Phase 6 | Non-functional baselines recorded | 🚧 | Partial - needs verification |

---

## 4. P0 Blockers (Must Fix Before Release)

### P0-9: Clippy Fails with `-D warnings` (18 errors)

**Status:** ❌ NEW - Must fix

**ratatui-testing (1 error):**
| Error | File | Fix |
|-------|------|-----|
| `new_without_default` for `StateTester` | ratatui-testing/src/state.rs:6 | Add `impl Default for StateTester` |

**opencode-core (17 errors):**
| Error | File | Line | Fix |
|-------|------|------|-----|
| deprecated `AgentMode` enum | config.rs | 436 | Remove or use non-deprecated variant |
| deprecated `AgentMode` enum | config.rs | (another) | Remove or use non-deprecated variant |
| deprecated `AgentConfig::mode` field | command.rs | 567 | Remove or use `permission` field |
| deprecated `AgentConfig::mode` field | config.rs | 2771 | Remove or use `permission` field |
| `question_mark` | config.rs | 1594 | Rewrite with `?` operator |
| `needless_borrows_for_generic_args` | config.rs | 2068 | Remove unnecessary borrow |
| `redundant_closure` | session_sharing.rs | 323 | Use `ok_or` instead |
| `map_entry` | session_sharing.rs | 225 | Use entry API |
| `and_then` → `map` | crash_recovery.rs | 241 | Change to `map` |
| `very_complex_type` | skill.rs | - | Factor into type definitions |
| `&PathBuf` → `&Path` | skill.rs | 116 | Fix 5 occurrences |

**Blocking:** Yes - Cannot release with `cargo clippy --all -- -D warnings` failing

---

## 5. P1 Issues (Important, In Progress/Deferred)

### P1-3: Deprecated Fields Removal
**Status:** 🚧 In Progress

| Field | Severity | Remediation | Target |
|-------|----------|-------------|--------|
| `mode` | Medium | Remove in v4.0 | v4.0 |
| `tools` | Medium | Remove after migration | Deferred |
| `theme` | Low | Moved to tui.json | Deferred |
| `keybinds` | Low | Moved to tui.json | Deferred |

**Action:** Add deprecation warnings for v4.0 removal

### P1-10: Variant/Reasoning Budget
**Status:** Deferred (marked experimental)

**Action:** Mark as experimental in documentation

---

## 6. P2 Issues (Nice to Have, Deferred)

| ID | Issue | Module | Status | Notes |
|----|-------|--------|--------|-------|
| P2-16 | Remaining clippy warnings | various | Deferred | Warnings only, not errors |
| P2-17 | Per-crate test backlog | tests | Deferred | Ongoing work |

---

## 7. Technical Debt

| ID | Item | Module | Severity | Remediation | Status |
|----|------|--------|----------|-------------|--------|
| TD-001 | ~~Clippy unreachable pattern~~ | ~~permission~~ | ~~CRITICAL~~ | ~~Fixed~~ | ✅ RESOLVED |
| TD-002 | ~~Desktop WebView stub~~ | ~~cli~~ | ~~P0~~ | ~~Implemented~~ | ✅ RESOLVED |
| TD-003 | Deprecated `mode` field | config | Medium | Remove in v4.0 | 🚧 In Progress |
| TD-004 | Deprecated `tools` field | config | Medium | Remove after migration | Deferred |
| TD-005 | Deprecated `theme` field | config | Low | Moved to tui.json | Deferred |
| TD-006 | Deprecated `keybinds` field | config | Low | Moved to tui.json | Deferred |
| TD-007 | Magic numbers in compaction | core | Low | Make configurable | Deferred |
| TD-008 | Custom JSONC parser | config | Medium | Consider existing crate | Deferred |
| TD-016 | Clippy errors (18) | core, ratatui-testing | HIGH | Fix all errors | P0-9 |

---

## 8. Iteration Progress

| Iteration | Date | Completion | Key Changes |
|-----------|------|------------|-------------|
| 1 | 2026-04-09 | ~20% | Initial gap analysis |
| 4 | 2026-04-10 | ~35-40% | Major P0 progress |
| 5 | 2026-04-11 | ~70-75% | Desktop/ACP gaps identified |
| 6 | 2026-04-12 | ~80-85% | ACP done, dialogs/slots done |
| 7 | 2026-04-12 | ~80-85% | Multiline done, P2-6/7/10/15 done |
| 8 | 2026-04-12 | ~85-90% | P0-8 clippy identified, 2 P0 blockers |
| 9 | 2026-04-12 | ~90-92% | P0-8, P0-new-2, P1-2, P1-9, P2-1/2/9/12/13/14/15 all fixed. 1 P0 remains (clippy) |

---

## 9. Immediate Actions

### Must Fix (Before Release) - P0

1. **Fix P0-9: Clippy errors (18 total)**
   - [ ] ratatui-testing: Add `impl Default for StateTester` in `ratatui-testing/src/state.rs`
   - [ ] core/config.rs: Fix deprecated `AgentMode` usage (2 occurrences)
   - [ ] core/command.rs: Fix deprecated `AgentConfig::mode` usage
   - [ ] core/config.rs: Fix `question_mark` at line 1594
   - [ ] core/config.rs: Fix `needless_borrows_for_generic_args` at line 2068
   - [ ] core/session_sharing.rs: Fix `redundant_closure` at line 323
   - [ ] core/session_sharing.rs: Fix `map_entry` at line 225
   - [ ] core/crash_recovery.rs: Fix `and_then` → `map` at line 241
   - [ ] core/skill.rs: Fix `very_complex_type`
   - [ ] core/skill.rs: Fix `&PathBuf` → `&Path` (5 occurrences at line 116)

### Should Fix (Before Release) - P1

2. **Plan P1-3: Deprecated fields removal**
   - [ ] Add deprecation warnings for `mode` field
   - [ ] Document removal plan for v4.0

3. **Plan P1-10: Variant/reasoning budget**
   - [ ] Mark as experimental in documentation

---

## 10. Next Steps

1. Fix all 18 clippy errors to pass `cargo clippy --all -- -D warnings`
2. Verify all tests pass after clippy fixes
3. Complete Phase 6 release qualification
4. Plan deprecated field removal for v4.0

---

*Plan generated: 2026-04-12*
*Iteration: 9*
