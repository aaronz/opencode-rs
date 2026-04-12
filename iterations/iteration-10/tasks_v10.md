# Task List v10

**Version:** 10.0
**Generated:** 2026-04-13
**Priority:** P0 > P1 > P2
**Total P0 Tasks:** 18 (all clippy errors)

---

## P0 Tasks (Must Fix Before Release)

### P0-9: Clippy Errors - 18 Total

#### ratatui-testing (1 error)

- [ ] **Task:** Add `impl Default for StateTester`
  - **File:** `ratatui-testing/src/state.rs`
  - **Line:** 6
  - **Error:** `new_without_default`
  - **Fix:** Implement `Default` trait for `StateTester`
  - **Verification:** `cargo clippy -p ratatui-testing -- -D warnings`

---

#### opencode-core (17 errors)

##### Config.rs - 4 errors

- [x] **Task:** Fix deprecated `AgentMode` enum usage ✅
  - **File:** `crates/core/src/config.rs`
  - **Line:** 436
  - **Error:** deprecated enum `AgentMode`
  - **Fix:** Removed deprecated mode field and AgentMode enum
  - **Verification:** `cargo clippy -p opencode-core -- -D warnings`

- [x] **Task:** Fix deprecated `AgentConfig::mode` field ✅
  - **File:** `crates/core/src/config.rs`
  - **Line:** 2771 (original) → struct definition area
  - **Error:** deprecated field `AgentConfig::mode`
  - **Fix:** Removed unnecessary `#[allow(deprecated)]` attribute from AgentConfig struct. The deprecated mode field was already removed in previous commits.
  - **Verification:** `cargo clippy -p opencode-core -- -D warnings`

- [x] **Task:** Rewrite block with `?` operator ✅
  - **File:** `crates/core/src/config.rs`
  - **Line:** 1594
  - **Error:** `question_mark` - block may be rewritten with `?`
  - **Fix:** Already fixed - code at line 1594 uses correct pattern
  - **Verification:** `cargo clippy -p opencode-core -- -D warnings`

- [ ] **Task:** Remove unnecessary borrow
  - **File:** `crates/core/src/config.rs`
  - **Line:** 2068
  - **Error:** `needless_borrows_for_generic_args`
  - **Fix:** Remove unnecessary borrow
  - **Verification:** `cargo clippy -p opencode-core -- -D warnings`

##### command.rs - 1 error

- [ ] **Task:** Fix deprecated `AgentConfig::mode` field
  - **File:** `crates/core/src/command.rs`
  - **Line:** 567
  - **Error:** deprecated field `AgentConfig::mode`
  - **Fix:** Remove usage or use `permission` field instead
  - **Verification:** `cargo clippy -p opencode-core -- -D warnings`

##### session_sharing.rs - 2 errors

- [ ] **Task:** Use `ok_or()` instead of closure
  - **File:** `crates/core/src/session_sharing.rs`
  - **Line:** 323
  - **Error:** `redundant_closure` - unnecessary closure for `Option::None`
  - **Fix:** Use `ok_or()` directly instead of closure
  - **Verification:** `cargo clippy -p opencode-core -- -D warnings`

- [ ] **Task:** Use `entry()` API properly
  - **File:** `crates/core/src/session_sharing.rs`
  - **Line:** 225
  - **Error:** `map_entry` - `contains_key` followed by `insert`
  - **Fix:** Use `entry()` API to avoid redundant lookup
  - **Verification:** `cargo clippy -p opencode-core -- -D warnings`

##### crash_recovery.rs - 1 error

- [ ] **Task:** Replace `and_then` with `map`
  - **File:** `crates/core/src/crash_recovery.rs`
  - **Line:** 241
  - **Error:** `and_then` - `Option.and_then(|x| Some(y))`
  - **Fix:** Replace with `map(|x| y)`
  - **Verification:** `cargo clippy -p opencode-core -- -D warnings`

##### skill.rs - 5+ errors

- [ ] **Task:** Factor complex type into type alias
  - **File:** `crates/core/src/skill.rs`
  - **Error:** `very_complex_type`
  - **Fix:** Factor parts into `type` definitions
  - **Verification:** `cargo clippy -p opencode-core -- -D warnings`

- [ ] **Task:** Change `&PathBuf` to `&Path` (5 occurrences)
  - **File:** `crates/core/src/skill.rs`
  - **Line:** 116
  - **Error:** `needless_borrow` - `&PathBuf` instead of `&Path`
  - **Fix:** Change `&PathBuf` parameter to `&Path`
  - **Verification:** `cargo clippy -p opencode-core -- -D warnings`

---

## P1 Tasks (Should Fix)

### P1-3: Deprecated Fields

- [ ] **Task:** Plan removal of deprecated `mode` field
  - **Module:** config
  - **Status:** In Progress
  - **Notes:** Full removal deferred to v4.0
  - **Related:** TD-003, TD-004, TD-005, TD-006

---

## P2 Tasks (Nice to Have - Deferred)

- [ ] P2-16: Remaining clippy warnings (not errors)
- [ ] P2-17: Per-crate test backlog

---

## Verification Commands

```bash
# Full clippy check (target)
cargo clippy --all -- -D warnings

# Per-crate verification
cargo clippy -p ratatui-testing -- -D warnings
cargo clippy -p opencode-core -- -D warnings
```

---

## Progress Summary

| Category | Total | Completed | Remaining |
|----------|-------|----------|-----------|
| P0 Tasks | 18 | 2 | 16 |
| P1 Tasks | 1 | 0 | 1 |
| P2 Tasks | 2 | 0 | 2 |
| **Total** | **21** | **2** | **19** |

---

*Task list generated: 2026-04-13*
