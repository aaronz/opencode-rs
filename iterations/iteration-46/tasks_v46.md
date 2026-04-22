# Task Checklist: flag Module (Iteration 46)

**Document Version**: 46
**Date**: 2026-04-22
**Status**: Active Implementation

---

## P0 Tasks (Blocking)

### FR-002: Unit Tests for Boolean Flags
- [x] `new_manager_has_all_boolean_flags`: Verify all 28 boolean flags registered
  - Check OPENCODE_EXPERIMENTAL, OPENCODE_DEBUG, OPENCODE_AUTO_SHARE, etc.
- [x] `default_values_are_false_except_markdown`: Only OPENCODE_EXPERIMENTAL_MARKDOWN defaults to true
- [x] `set_overrides_value`: Boolean set() correctly overrides default
- [x] `unknown_flag_returns_false`: Missing flags return false from is_enabled()
- [x] `exa_enabled_when_experimental_is_true`: opencode_enable_exa() works when experimental=true
- [x] `plan_mode_enabled_when_experimental_is_true`: opencode_experimental_plan_mode() works

### FR-003: Unit Tests for String and Number Flags
- [ ] `opencode_client_defaults_to_cli`: opencode_client() returns "cli" by default
- [ ] `bash_timeout_has_default`: opencode_experimental_bash_timeout_ms() returns Some(120_000)
- [ ] `string_flag_returns_none_when_not_set`: get_string() returns None when env not set
- [ ] `number_flag_returns_none_when_not_set`: get_number() returns None when env not set

### FR-001: ✅ Done
- [ ] Find application entry point (main.rs, lib.rs, or config init)
- [ ] Instantiate FlagManager at application startup
- [ ] Call flags.load_from_env() immediately after instantiation
- [ ] Store FlagManager in globally accessible location (AppState, RuntimeContext)

---

## P1 Tasks (High Priority)

### FR-004: Register OPENCODE_EXPERIMENTAL_EXA Flag
- [ ] Register OPENCODE_EXPERIMENTAL_EXA in FlagManager::new() with default false
- [ ] OR remove the truthy("OPENCODE_EXPERIMENTAL_EXA") check if not needed
- [ ] Verify opencode_enable_exa() method works correctly

### FR-008: Application Integration - Replace Hardcoded Feature Checks
- [ ] Audit codebase for hardcoded OPENCODE_* env var checks (grep)
- [ ] Replace hardcoded feature checks with FlagManager queries
- [ ] Ensure all feature toggles use the flag system

---

## P2 Tasks (Medium Priority)

### FR-005: Refactor load_from_env() to Use truthy() Helper
- [ ] Current: `flag.value = val == "1" || val.to_lowercase() == "true";`
- [ ] Change to: `flag.value = truthy(name);`
- [ ] Remove duplicate truthy check code

### FR-006: Add Setter Methods for String and Number Flags
- [ ] Add `set_string(&mut self, name: &str, value: Option<String>)` method
- [ ] Add `set_number(&mut self, name: &str, value: Option<u64>)` method
- [ ] Methods should only update if flag exists (silent no-op otherwise)

### FR-007: Add Method to List All Flags
- [ ] Add `all_flags(&self) -> &HashMap<String, Flag>` getter
- [ ] Add `all_string_flags(&self) -> &HashMap<String, Option<String>>` getter
- [ ] Add `all_number_flags(&self) -> &HashMap<String, Option<u64>>` getter
- [ ] Useful for debugging and admin interfaces

---

## Unit Tests Required

| Test | Feature | Status |
|------|---------|--------|
| new_manager_has_all_boolean_flags | All 28 flags registered | ✅ |
| default_values_are_false_except_markdown | Default values | ✅ |
| set_overrides_value | Boolean setter | ✅ |
| unknown_flag_returns_false | Fallback behavior | ✅ |
| exa_enabled_when_experimental_is_true | Convenience method | ✅ |
| plan_mode_enabled_when_experimental_is_true | Convenience method | ✅ |
| opencode_client_defaults_to_cli | String flag default | ⬜ |
| bash_timeout_has_default | Number flag default | ⬜ |
| string_flag_returns_none_when_not_set | String flag behavior | ⬜ |
| number_flag_returns_none_when_not_set | Number flag behavior | ⬜ |

---

## Acceptance Criteria

| ID | Criteria | Priority | Status |
|----|----------|----------|--------|
| AC-001 | FlagManager::new() creates all 28 boolean flags | P0 | ⬜ |
| AC-002 | load_from_env() populates flag values from environment | P0 | ⬜ |
| AC-003 | All convenience methods work correctly | P0 | ⬜ |
| AC-004 | opencode_client() defaults to "cli" | P0 | ⬜ |
| AC-005 | opencode_experimental_bash_timeout_ms() defaults to 120000 | P0 | ⬜ |
| AC-006 | All 10 unit tests pass | P0 | ⬜ |
| AC-007 | FlagManager is instantiated at application startup | P0 | ⬜ |
| AC-008 | load_from_env() is called at startup | P0 | ⬜ |
| AC-009 | OPENCODE_EXPERIMENTAL_EXA is registered or removed | P1 | ⬜ |
| AC-010 | load_from_env() uses truthy() helper | P2 | ⬜ |
| AC-011 | set_string() and set_number() methods exist | P2 | ⬜ |
| AC-012 | all_flags() getters exist for debugging | P2 | ⬜ |

---

## Verification Checklist

- [ ] All 28 boolean flags registered in FlagManager::new()
- [ ] All 11 string flags registered
- [ ] All 2 number flags registered
- [ ] FlagManager instantiated at application startup
- [ ] load_from_env() called at startup
- [ ] All 10 unit tests pass
- [ ] OPENCODE_EXPERIMENTAL_EXA registered or removed
- [ ] opencode_enable_exa() works correctly
- [ ] load_from_env() uses truthy() helper
- [ ] set_string() and set_number() methods added
- [ ] all_flags() getters added for debugging