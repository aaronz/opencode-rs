# Task List v11

**Generated:** 2026-04-13  
**Based on:** Iteration 11 gap analysis  
**Total Tasks:** 10  
**Completed:** 2  
**In Progress:** 1  
**Pending:** 7  

---

## Priority: P1 (Must Fix Before Release)

### Task T-001: Fix Flaky Test `test_theme_config_resolve_path_tilde_expansion`

| Field | Value |
|-------|-------|
| **Priority** | P1 - HIGH |
| **Status** | ✅ Done |
| **Module** | core/config |
| **Issue** | `dirs::home_dir()` doesn't respect `HOME` env var on macOS |
| **Test** | `test_theme_config_resolve_path_tilde_expansion` |
| **Fix Required** | Use `dirs_next::home_dir()` or mock properly |

**Action Items:**
- [x] Find the test file containing `test_theme_config_resolve_path_tilde_expansion`
- [x] Identify where `dirs::home_dir()` is called
- [x] Replace with `dirs_next::home_dir()` OR add proper mocking for HOME env var
- [x] Verify test passes on macOS
- [x] Ensure test still passes on Linux

**Estimated Effort:** 1-2 hours

**Changes Made:**
- Added `dirs-next = "2.0"` to workspace dependencies in `Cargo.toml`
- Added `dirs-next = { workspace = true }` to core crate dependencies
- Replaced `dirs::home_dir()` with `dirs_next::home_dir()` in `ThemeConfig::resolve_path()` method in `crates/core/src/config.rs:1048-1050`

---

### Task T-002: Plan Deprecated Fields Removal (P1-3)

| Field | Value |
|-------|-------|
| **Priority** | P1 - MEDIUM |
| **Status** | 🚧 In Progress |
| **Module** | config |
| **Issue** | Deprecated fields still present: `mode`, `tools`, `theme`, `keybinds` |
| **Target** | v4.0 removal |

**Action Items:**
- [ ] Document current usage of deprecated fields in code
- [ ] Add deprecation warnings for all deprecated fields (if not already)
- [ ] Create migration guide for v4.0
- [ ] Plan field removal timeline for v4.0

**Affected Files:**
- `crates/config/src/config.rs`
- `crates/core/src/config.rs`
- `crates/cli/src/command.rs`

**Estimated Effort:** 2-3 hours planning

---

## Priority: P2 (Nice to Have - Deferred)

### Task T-003: Resolve Remaining Clippy Warnings (P2-16)

| Field | Value |
|-------|-------|
| **Priority** | P2 - LOW |
| **Status** | ✅ Done |
| **Module** | various |
| **Issue** | Some clippy warnings remain (not errors) |
| **Note** | Does not block release |

**Action Items:**
- [x] Run `cargo clippy --all -- -W clippy::all` to identify warnings
- [x] Categorize warnings by severity
- [x] Fix warnings in non-deferred scope if time permits

**Verification:**
```
cargo clippy --all -- -W clippy::all  # Zero warnings
cargo clippy --all -- -D warnings     # Passes
```

**Estimated Effort:** 4-8 hours (if deferred, skip for now)

---

### Task T-004: Complete Per-Crate Test Backlog (P2-17)

| Field | Value |
|-------|-------|
| **Priority** | P2 - LOW |
| **Status** | Deferred |
| **Module** | tests |
| **Issue** | Per-crate test coverage incomplete |
| **Reference** | FR-026, FR-027 |

**Action Items:**
- [ ] Review current test coverage per crate
- [ ] Identify missing test cases from backlog
- [ ] Add tests for edge cases
- [ ] Ensure mock helpers are properly used

**Estimated Effort:** Ongoing

---

## Priority: P2 (Technical Debt - Deferred)

### Task T-005: Remove Magic Numbers in Compaction (TD-007)

| Field | Value |
|-------|-------|
| **Priority** | P2 - LOW |
| **Status** | Deferred |
| **Module** | core |
| **Issue** | Magic numbers in compaction code |
| **Fix** | Make configurable |

**Action Items:**
- [ ] Find magic numbers in compaction logic
- [ ] Extract to configuration constants
- [ ] Add to config schema if appropriate

**Estimated Effort:** 2-3 hours

---

### Task T-006: Evaluate Custom JSONC Parser (TD-008)

| Field | Value |
|-------|-------|
| **Priority** | P2 - MEDIUM |
| **Status** | Deferred |
| **Module** | config |
| **Issue** | Custom JSONC parser in use |
| **Consideration** | Use existing crate (e.g., `json_comments`) |

**Action Items:**
- [ ] Research existing JSONC parsing crates
- [ ] Evaluate performance impact of switching
- [ ] Plan migration if beneficial

**Estimated Effort:** 4-6 hours research + migration

---

### Task T-007: Deprecated Fields - `tools` (TD-004)

| Field | Value |
|-------|-------|
| **Priority** | P2 - MEDIUM |
| **Status** | Deferred |
| **Module** | config |
| **Issue** | `tools` field deprecated |
| **Fix** | Remove after migration |

**Action Items:**
- [ ] Ensure all uses migrated to `permission` field
- [ ] Add deprecation warning if not present
- [ ] Schedule removal for v4.0

**Estimated Effort:** 1-2 hours

---

### Task T-008: Deprecated Fields - `theme` (TD-005)

| Field | Value |
|-------|-------|
| **Priority** | P2 - LOW |
| **Status** | ✅ Done |
| **Module** | config |
| **Issue** | `theme` field moved to tui.json |
| **Fix** | Remove from config |

**Action Items:**
- [x] Ensure theme config fully migrated to tui.json
- [x] Remove `theme` field from opencode.json schema
- [x] Add deprecation warning if still present

**Estimated Effort:** 1-2 hours

---

### Task T-009: Deprecated Fields - `keybinds` (TD-006)

| Field | Value |
|-------|-------|
| **Priority** | P2 - LOW |
| **Status** | ✅ Done |
| **Module** | config |
| **Issue** | `keybinds` field moved to tui.json |
| **Fix** | Remove from config |

**Action Items:**
- [x] Ensure keybinds config fully migrated to tui.json
- [x] Remove `keybinds` field from opencode.json schema
- [x] Add deprecation warning if still present

**Estimated Effort:** 1-2 hours

---

## Priority: Maintenance (Ongoing)

### Task T-010: Phase 6 - Release Qualification Completion

| Field | Value |
|-------|-------|
| **Priority** | P1 |
| **Status** | 🚧 In Progress |
| **Phase** | Phase 6 of 6 |
| **Coverage** | ~80% |

**Action Items:**
- [x] All P0 blockers resolved
- [x] Clippy passes with `-D warnings`
- [x] All major PRD features implemented
- [x] Fix flaky test (T-001)
- [ ] Complete non-functional test baselines
- [ ] Final release sign-off

---

## Task Summary

| ID | Task | Priority | Status | Estimated Effort |
|----|------|----------|--------|------------------|
| T-001 | Fix flaky test `test_theme_config_resolve_path_tilde_expansion` | **P1** | ✅ Done | 1-2h |
| T-002 | Plan deprecated fields removal | **P1** | 🚧 In Progress | 2-3h |
| T-003 | Resolve remaining clippy warnings | P2 | ✅ Done | 4-8h |
| T-004 | Complete per-crate test backlog | P2 | Deferred | Ongoing |
| T-005 | Remove magic numbers in compaction | P2 | Deferred | 2-3h |
| T-006 | Evaluate custom JSONC parser | P2 | Deferred | 4-6h |
| T-007 | Remove deprecated `tools` field | P2 | Deferred | 1-2h |
| T-008 | Remove deprecated `theme` field | P2 | Deferred | 1-2h |
| T-009 | Remove deprecated `keybinds` field | P2 | ✅ Done | 1-2h |
| T-010 | Complete Phase 6 release qualification | P1 | 🚧 In Progress | - |

---

## Done Issues (Iteration 11)

| ID | Issue | Fixed In |
|----|-------|----------|
| P0-9 | Clippy fails with `-D warnings` (18 errors) | **Iteration 11** |
| P1-F1 | `test_theme_config_resolve_path_tilde_expansion` fails on macOS | **Iteration 11** |

---

*Task list generated: 2026-04-13*
*Next step: Complete Phase 6 release qualification (T-010)*