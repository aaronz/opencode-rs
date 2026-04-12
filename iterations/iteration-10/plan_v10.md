# Implementation Plan v10

**Version:** 10.0
**Generated:** 2026-04-13
**Based on:** Spec v10 and Gap Analysis (Iteration 10)
**Status:** Draft

---

## 1. Overview

This plan tracks the remaining implementation work for the OpenCode Rust port based on Iteration 10 analysis.

**Overall Completion:** ~90-92%
**Phase Status:** Phase 5-6 of 6 (Hardening, Release Qualification)
**Primary Blocker:** P0-9 (18 clippy errors)

---

## 2. Immediate Priorities

### P0 - Must Fix Before Release

| Priority | Issue | Module | Errors | Status |
|----------|-------|--------|--------|--------|
| **P0-9** | Clippy fails with `-D warnings` | core, ratatui-testing | 18 | ❌ OPEN |

### P0-9 Clippy Error Breakdown

#### ratatui-testing (1 error)
| Error | File | Line | Fix |
|-------|------|------|-----|
| `new_without_default` | state.rs | 6 | Add `impl Default for StateTester` |

#### opencode-core (17 errors)
| Error | File | Line | Fix |
|-------|------|------|-----|
| deprecated `AgentMode` | config.rs | 436 | Remove or use `permission` field |
| deprecated `AgentConfig::mode` | command.rs | 567 | Remove or use `permission` field |
| deprecated `AgentConfig::mode` | config.rs | 2771 | Remove or use `permission` field |
| `question_mark` | config.rs | 1594 | Rewrite block with `?` operator |
| `needless_borrows_for_generic_args` | config.rs | 2068 | Remove unnecessary borrow |
| `redundant_closure` | session_sharing.rs | 323 | Use `ok_or()` instead of closure |
| `map_entry` | session_sharing.rs | 225 | Use `entry()` API properly |
| `and_then` → `map` | crash_recovery.rs | 241 | Replace with `map` |
| `very_complex_type` | skill.rs | (complex) | Factor into type alias |
| `&PathBuf` → `&Path` | skill.rs | 116 | Change to `&Path` (5 occurrences) |

---

## 3. Implementation Phases

### Phase 5: Hardening (~85%)
- [x] P0-8: Clippy unreachable pattern (FIXED)
- [x] P0-new-2: Desktop WebView (DONE)
- [x] P0-new-3: ACP HTTP+SSE transport (DONE)
- [ ] **P0-9: Clippy errors (18 total)** - IN PROGRESS
- [x] P1-2: Circular variable expansion (DONE)
- [x] P1-5: Multiline input (DONE)
- [x] P1-7: TUI Plugin dialogs (DONE)
- [x] P1-8: TUI Plugin slots (DONE)
- [x] P1-9: Session sharing (DONE)
- [x] P1-10: Permission inheritance (DONE)
- [x] P1-11: Request validation (DONE)
- [ ] **P1-3: Deprecated fields (mode, tools, theme, keybinds)** - IN PROGRESS

### Phase 6: Release Qualification (~70%)
- [ ] Non-functional baselines verification
- [ ] Final integration testing
- [ ] Release documentation

---

## 4. P0-9 Fix Plan

### Step 1: ratatui-testing (1 error)
- [ ] Add `impl Default for StateTester` in `ratatui-testing/src/state.rs`

### Step 2: opencode-core config.rs (4 errors)
- [ ] Remove/fix deprecated `AgentMode` at line 436
- [ ] Remove/fix deprecated `AgentConfig::mode` at line 2771
- [ ] Fix `question_mark` at line 1594 - rewrite block with `?`
- [ ] Fix `needless_borrows_for_generic_args` at line 2068

### Step 3: opencode-core command.rs (1 error)
- [ ] Remove/fix deprecated `AgentConfig::mode` at line 567

### Step 4: opencode-core session_sharing.rs (2 errors)
- [ ] Fix `redundant_closure` at line 323 - use `ok_or()`
- [ ] Fix `map_entry` at line 225 - use `entry()` API

### Step 5: opencode-core crash_recovery.rs (1 error)
- [ ] Fix `and_then` → `map` at line 241

### Step 6: opencode-core skill.rs (5+ errors)
- [ ] Fix `very_complex_type` - factor into type alias
- [ ] Fix `&PathBuf` → `&Path` at line 116 (5 occurrences)

---

## 5. Verification

After each step, verify with:
```bash
cargo clippy --all -- -D warnings
```

Target: All clippy checks pass with `-D warnings` before release.

---

## 6. Progress Tracking

| Iteration | Date | P0-9 Status | Completion |
|-----------|------|-------------|------------|
| 9 | 2026-04-12 | 18 errors | ~90-92% |
| 10 | 2026-04-13 | 18 errors | ~90-92% |

---

*Plan generated: 2026-04-13*
