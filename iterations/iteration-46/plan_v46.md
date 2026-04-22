# Implementation Plan: flag Module (Iteration 46)

**Document Version**: 46
**Date**: 2026-04-22
**Status**: Active Implementation
**Based on**: Gap Analysis + Spec v46

---

## Phase 1: Unit Tests (P0)

| ID | Task | Status | Notes |
|----|------|--------|-------|
| 1.1 | Implement `new_manager_has_all_boolean_flags` test | ⬜ | Verify all 28 boolean flags registered |
| 1.2 | Implement `default_values_are_false_except_markdown` test | ⬜ | Only OPENCODE_EXPERIMENTAL_MARKDOWN defaults to true |
| 1.3 | Implement `set_overrides_value` test | ⬜ | Boolean set() works correctly |
| 1.4 | Implement `unknown_flag_returns_false` test | ⬜ | Fallback behavior for missing flags |
| 1.5 | Implement `exa_enabled_when_experimental_is_true` test | ⬜ | Convenience method works |
| 1.6 | Implement `plan_mode_enabled_when_experimental_is_true` test | ⬜ | Convenience method works |
| 1.7 | Implement `opencode_client_defaults_to_cli` test | ⬜ | String flag default value |
| 1.8 | Implement `bash_timeout_has_default` test | ⬜ | Number flag default (120000ms) |
| 1.9 | Implement `string_flag_returns_none_when_not_set` test | ⬜ | String flag behavior |
| 1.10 | Implement `number_flag_returns_none_when_not_set` test | ⬜ | Number flag behavior |

---

## Phase 2: Application Integration (P0)

| ID | Task | Status | Notes |
|----|------|--------|-------|
| 2.1 | Find application entry point(s) | ⬜ | Locate main.rs, lib.rs, config init |
| 2.2 | Instantiate FlagManager at startup | ⬜ | Add FlagManager::new() |
| 2.3 | Call flags.load_from_env() | ⬜ | Populate from environment |
| 2.4 | Store FlagManager globally | ⬜ | AppState, RuntimeContext, or similar |

---

## Phase 3: Bug Fixes (P1)

| ID | Task | Status | Notes |
|----|------|--------|-------|
| 3.1 | Register OPENCODE_EXPERIMENTAL_EXA flag | ⬜ | Or remove truthy() reference in opencode_enable_exa() |
| 3.2 | Audit hardcoded feature checks | ⬜ | Search for OPENCODE_* env var usage |
| 3.3 | Replace hardcoded checks with FlagManager | ⬜ | Use flag queries throughout app |

---

## Phase 4: API Enhancements (P2)

| ID | Task | Status | Notes |
|----|------|--------|-------|
| 4.1 | Refactor load_from_env() to use truthy() | ⬜ | Remove code duplication |
| 4.2 | Add set_string() method | ⬜ | For testing string flags |
| 4.3 | Add set_number() method | ⬜ | For testing number flags |
| 4.4 | Add all_flags() getter | ⬜ | For debugging/admin |
| 4.5 | Add all_string_flags() getter | ⬜ | For debugging/admin |
| 4.6 | Add all_number_flags() getter | ⬜ | For debugging/admin |

---

## Implementation Order

```
Phase 1 (Tests - P0):
1. Add #[cfg(test)] mod tests to flag.rs
2. Implement all 10 unit tests
3. Run cargo test -p opencode-core

Phase 2 (Integration - P0):
4. Find application entry point
5. Instantiate FlagManager in app init
6. Call load_from_env()
7. Store in global state

Phase 3 (Bug Fix - P1):
8. Fix OPENCODE_EXPERIMENTAL_EXA registration
9. Audit and replace hardcoded checks

Phase 4 (Polish - P2):
10. Refactor load_from_env() to use truthy()
11. Add set_string/set_number methods
12. Add all_flags getters
```

---

## Key Files

| File | Changes |
|------|---------|
| `opencode-rust/crates/core/src/flag.rs` | Add tests, fix unregistered flag, add setters |
| `opencode-rust/crates/core/src/lib.rs` | Export FlagManager if needed |
| Application entry points | Instantiate FlagManager and call load_from_env() |

---

## Dependencies

- serde with derive (already present)
- std::collections::HashMap (built-in)
- std::env (built-in)

---

## Verification

- [ ] All 10 unit tests pass
- [ ] FlagManager instantiated at startup
- [ ] load_from_env() called at startup
- [ ] OPENCODE_EXPERIMENTAL_EXA registered or removed
- [ ] All convenience methods work correctly