# Implementation Plan v8

**Version:** 8.0
**Generated:** 2026-04-12
**Based on:** Spec v8 (Iteration 8 gap analysis)
**Status:** Draft

---

## 1. Executive Summary

**Overall Completion Estimate:** ~85-90%

**Phase Status:** Phase 4-6 of 6 (Interface Implementations, Hardening, Release Qualification)

### Critical Blockers

| Priority | Issue | Module | Status |
|----------|-------|--------|--------|
| **P0** | Clippy unreachable pattern | permission | ❌ Must Fix |
| **P0** | Desktop WebView integration | cli | ❌ Stub Only |

### Release Gate Status

| Gate | Criteria | Status |
|------|----------|--------|
| Phase 0 | Workspace builds, tests run, clippy clean | ❌ Clippy fails |
| Phase 1-3 | Authority/Runtime/Subsystem tests | ✅ Complete |
| Phase 4 | Interface smoke workflows | 🚧 Desktop WebView blocks |
| Phase 5a | Compatibility suite | ✅ Complete |
| Phase 5b | Conventions suite | ✅ Complete (23 tests) |
| Phase 6 | Non-functional baselines | 🚧 Partial |

---

## 2. P0 Blockers (Must Fix Before Release)

### P0-8: Clippy Unreachable Pattern [CRITICAL]

**File:** `crates/permission/src/models.rs:28`

**Issue:** The `intersect()` function has an unreachable pattern in the match expression that fails clippy with `-D warnings`.

```
error: unreachable pattern
  --> crates/permission/src/models.rs:28:51
   |
28 |             (AgentPermissionScope::ReadOnly, _) | (_, AgentPermissionScope::ReadOnly) => {
   |                                                   ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ no value can reach this
```

**Root Cause:** The pattern `(AgentPermissionScope::ReadOnly, _)` already matches all cases where the first element is `ReadOnly`, making the second pattern `(_, AgentPermissionScope::ReadOnly)` unreachable when the first element is also `ReadOnly`.

**Fix Required:** Correct the pattern matching logic to handle all cases properly without overlap.

**Impact:** Blocks `cargo clippy --all -- -D warnings` from passing.

---

### P0-new-2: Desktop WebView Integration

**File:** `crates/cli/src/desktop.rs`

**Issue:** WebView integration is stub-only. The code uses `wry` for WebView but only spawns a browser when the `desktop` feature is off. When `desktop` feature is enabled, `spawn_webview_thread` creates a WebView but doesn't properly integrate with the app lifecycle.

**Required:** Actual WebView component per PRD 13 that shares state with TUI/server.

**Impact:** Phase 4 (Interface Implementations) blocked.

---

## 3. P1 Issues (Should Fix Before Release)

### P1-2: Circular Variable Expansion Detection

**Module:** config

**Issue:** Variable expansion circular references are not fully handled.

**Status:** Deferred

**Fix Required:** Add detection algorithm for circular references in config variable expansion (`{env:VAR}` and `{file:PATH}`).

---

### P1-3: Deprecated Fields Removal

**Module:** config

**Issue:** Deprecated fields `mode`, `tools`, `theme`, `keybinds` remain in codebase.

**Status:** 🚧 In Progress

**Fields:**
| Field | Severity | Remediation | Status |
|-------|----------|-------------|--------|
| `mode` | Medium | Remove in major version | In Progress |
| `tools` | Medium | Remove after migration | Deferred |
| `theme` | Low | Moved to tui.json | Deferred |
| `keybinds` | Low | Moved to tui.json | Deferred |

---

### P1-9: Session Sharing Between Interfaces

**Module:** cli

**Issue:** Cross-interface session synchronization is partial.

**Status:** Deferred

**Required:** Complete session sharing between TUI, web, and ACP interfaces.

---

## 4. CLI Test Failures

### test_prompt_history_persistence

**File:** `crates/cli/tests/e2e_prompt_history.rs`

**Issue:** Assertion failed in history persistence test.

---

### test_prompt_history_navigation

**File:** `crates/cli/tests/e2e_prompt_history.rs`

**Issue:** `history.len() >= 3` assertion failed.

---

## 5. P2 Issues (Nice to Have)

| ID | Issue | Module | Status | Notes |
|----|-------|--------|--------|-------|
| P2-1 | Project VCS worktree root distinction | core | Deferred | Add `worktree_root` field if distinct |
| P2-2 | Workspace path validation | core | Deferred | Ensure paths resolve within project |
| P2-3 | Compaction shareability verification | storage | ✅ Done | |
| P2-4 | Deterministic collision resolution | tools | ✅ Done | |
| P2-5 | Result caching invalidation | tools | ✅ Done | |
| P2-6 | Per-server OAuth token storage | mcp | ✅ Done | |
| P2-7 | Context cost warnings | mcp | ✅ Done | Implemented in context_cost.rs |
| P2-8 | Experimental LSP tool testing | lsp | Deferred | |
| P2-9 | API error shape consistency | server | Deferred | |
| P2-10 | Plugin cleanup/unload | plugin | ✅ Done | |
| P2-11 | Shell prefix (`!`) handler | tui | ✅ Done | Implemented via InputParser and ShellHandler |
| P2-12 | Home view completion | tui | Deferred | Recent sessions, quick actions partial |

---

## 6. Phase Progress

| Phase | Description | Status | Coverage |
|-------|-------------|--------|----------|
| Phase 0 | Project Foundation | ✅ Complete | 100% |
| Phase 1 | Authority Implementation | ✅ Complete | ~98% |
| Phase 2 | Runtime Core | ✅ Complete | ~98% |
| Phase 3 | Infrastructure Subsystems | ✅ Complete | ~95% |
| Phase 4 | Interface Implementations | 🚧 In Progress | ~75% |
| Phase 5 | Hardening | 🚧 In Progress | ~80% |
| Phase 6 | Release Qualification | 🚧 Partial | ~60% |

---

## 7. Technical Debt

| ID | Item | Module | Severity | Status |
|----|------|--------|----------|--------|
| TD-001 | Clippy unreachable pattern | permission | CRITICAL | P0-8 |
| TD-002 | Desktop WebView stub | cli | P0 | P0-new-2 |
| TD-003 | Deprecated `mode` field | config | Medium | In Progress |
| TD-004 | Deprecated `tools` field | config | Medium | Deferred |
| TD-005 | Deprecated `theme` field | config | Low | Deferred |
| TD-006 | Deprecated `keybinds` field | config | Low | Deferred |
| TD-007 | Magic numbers in compaction | core | Low | Deferred |
| TD-008 | Custom JSONC parser | config | Medium | Deferred |
| TD-009 | `#[serde(other)]` in Part | core | Low | Deferred |
| TD-010 | Unused `SecretStorage` methods | core | Low | Deferred |
| TD-011 | Unused imports in core | core | Low | Deferred |
| TD-012 | Unused variable `e` in lsp_tool | tools | Low | Deferred |
| TD-013 | Unused `save_session_records` | cli | Low | Deferred |
| TD-014 | `open_browser` function unused | cli | Low | Deferred |
| TD-015 | `format_time_elapsed` function unused | tui | Low | Deferred |

---

## 8. Iteration History

| Iteration | Date | Completion | Key Changes |
|-----------|------|------------|-------------|
| 1 | 2026-04-09 | ~20% | Initial gap analysis |
| 4 | 2026-04-10 | ~35-40% | Major P0 progress |
| 5 | 2026-04-11 | ~70-75% | Desktop/ACP gaps identified |
| 6 | 2026-04-12 | ~80-85% | ACP done, dialogs/slots done, 1 P0 remains |
| 7 | 2026-04-12 | ~80-85% | P1-5 multiline done, P2-6, P2-7, P2-10 done |
| 8 | 2026-04-12 | ~85-90% | P0-8 clippy failure identified, 2 P0 blockers remain |

---

## 9. Immediate Actions

### Must Fix (P0 - Before Any Release)

1. **Fix P0-8: Clippy unreachable pattern**
   - File: `crates/permission/src/models.rs:28`
   - Fix: Correct the `intersect()` function pattern matching

2. **Fix P0-new-2: Desktop WebView integration**
   - File: `crates/cli/src/desktop.rs`
   - Implement actual WebView component per PRD 13

### Should Fix (P1 - Before Release)

3. **Fix CLI e2e test failures**
   - File: `crates/cli/tests/e2e_prompt_history.rs`
   - Fix: `test_prompt_history_persistence` and `test_prompt_history_navigation`

4. **Continue P1-3: Deprecated fields removal**
   - Complete `mode` field removal
   - Plan `tools`, `theme`, `keybinds` removal for v4.0

### Can Defer (P2 - Post Release)

5. P2-1, P2-2: Core architecture improvements
6. P2-8: Experimental LSP tool testing
7. P2-9: API error shape consistency
8. P2-12: Home view completion

---

*Plan generated: 2026-04-12*
*Iteration: 8*
