# Gap Analysis Report: flag Module

**Module**: flag
**Source**: `opencode-rust/crates/core/src/flag.rs`
**PRD**: `docs/PRD/modules/flag.md`
**Analysis Date**: 2026-04-22
**Iteration**: 46

---

## Executive Summary

The `flag` module is **fully implemented** according to PRD specifications (462 lines, all core types, flags, and methods present). However, it suffers from a **critical integration gap** — the module is never instantiated or used anywhere in the codebase, making the entire system dead code. Additionally, required unit tests specified in the PRD are completely missing.

**Implementation Status**: 100% PRD compliance for standalone module, but 0% integration

---

## Gap Analysis

### 1. Functionality Completeness

| PRD Requirement | Status | Gap |
|-----------------|--------|-----|
| `Flag` struct | ✅ Implemented | None |
| `FlagManager` struct | ✅ Implemented | None |
| 28 Boolean flags | ✅ All registered | None |
| 11 String flags | ✅ All registered | None |
| 2 Number flags | ✅ All registered | None |
| Core methods (`get`, `get_string`, `get_number`, `set`, `is_enabled`) | ✅ Implemented | None |
| `load_from_env()` | ✅ Implemented | Never called |
| Convenience methods (9 methods) | ✅ All implemented | None |
| `truthy()` helper | ✅ Implemented | Not used by `load_from_env()` |

### 2. Interface Completeness

| Interface | Status | Gap |
|-----------|--------|-----|
| `FlagManager::new()` | ✅ Implemented | Never called |
| `FlagManager::get()` | ✅ Implemented | Never used |
| `FlagManager::get_string()` | ✅ Implemented | Never used |
| `FlagManager::get_number()` | ✅ Implemented | Never used |
| `FlagManager::set()` | ✅ Implemented | Never used |
| `FlagManager::is_enabled()` | ✅ Implemented | Never used |
| `FlagManager::load_from_env()` | ✅ Implemented | Never called |
| Convenience methods | ✅ Implemented | Never used |

### 3. Integration Status

| Usage Point | Status | Gap |
|-------------|--------|-----|
| `FlagManager` instantiation | ❌ Missing | Module never instantiated |
| `load_from_env()` call | ❌ Missing | Env vars never loaded |
| Usage in application | ❌ Missing | Entire module is dead code |
| Re-export in lib.rs | ✅ Present | Line 169: `pub(crate) use flag::FlagManager;` |

---

## Critical Issues (P0)

### P0-1: FlagManager Never Instantiated
- **Severity**: Critical (Blocking)
- **Module**: flag.rs, application initialization
- **Description**: `FlagManager` is defined and exported but never instantiated anywhere in the codebase. The entire flag system is effectively dead code.
- **Evidence**: `grep -r "FlagManager::new"` returns no results outside of `flag.rs` itself
- **Fix**: Add `FlagManager` instantiation at application startup, typically in the main entry point or config initialization.

### P0-2: load_from_env() Never Called
- **Severity**: Critical (Blocking)
- **Module**: flag.rs
- **Description**: Even if `FlagManager` were instantiated, `load_from_env()` is never called, meaning all flag values remain at their defaults and environment variables have no effect.
- **Evidence**: `grep -r "load_from_env"` shows only self-reference
- **Fix**: Call `flags.load_from_env()` immediately after `FlagManager::new()`

### P0-3: No Unit Tests
- **Severity**: Critical (Blocking)
- **Module**: flag.rs
- **Description**: The PRD specifies 10 unit tests that are completely absent:
  - `new_manager_has_all_boolean_flags`
  - `default_values_are_false_except_markdown`
  - `set_overrides_value`
  - `unknown_flag_returns_false`
  - `opencode_client_defaults_to_cli`
  - `bash_timeout_has_default`
  - `exa_enabled_when_experimental_is_true`
  - `plan_mode_enabled_when_experimental_is_true`
  - `string_flag_returns_none_when_not_set`
  - `number_flag_returns_none_when_not_set`
- **Fix**: Add `#[cfg(test)] mod tests` block with all 10 test cases per PRD specification

---

## High Priority Issues (P1)

### P1-1: Unregistered Flag Referenced in Code
- **Severity**: High
- **Module**: flag.rs:434
- **Description**: `opencode_enable_exa()` calls `truthy("OPENCODE_EXPERIMENTAL_EXA")` but `OPENCODE_EXPERIMENTAL_EXA` is never registered as a flag in `FlagManager::new()`
- **Current Code**:
  ```rust
  pub fn opencode_enable_exa(&self) -> bool {
      self.get("OPENCODE_ENABLE_EXA").unwrap_or(false)
          || self.opencode_experimental()
          || truthy("OPENCODE_EXPERIMENTAL_EXA")  // Unregistered!
  }
  ```
- **Fix**: Either register `OPENCODE_EXPERIMENTAL_EXA` in `new()` or remove the `truthy()` check

### P1-2: No Integration with Application Flow
- **Severity**: High
- **Module**: Application entry points
- **Description**: No part of the application checks feature flags before enabling/disabling functionality. All conditional logic based on flags is non-functional.
- **Fix**: Replace hardcoded feature checks with flag queries throughout the codebase

---

## Medium Priority Issues (P2)

### P2-1: truthy() Not Used by load_from_env()
- **Severity**: Medium
- **Module**: flag.rs
- **Description**: `load_from_env()` uses inline `to_lowercase() == "true" || val == "1"` instead of calling the `truthy()` helper function, creating code duplication and potential inconsistency.
- **Current**: Line 390: `flag.value = val == "1" || val.to_lowercase() == "true";`
- **Expected**: `flag.value = truthy(name);`
- **Fix**: Refactor `load_from_env()` to use `truthy(name)` for consistency

### P2-2: No Setter for String/Number Flags
- **Severity**: Medium
- **Module**: flag.rs
- **Description**: Only boolean flags have a `set()` method. String and number flags can only be set via `load_from_env()` or direct mutation.
- **Fix**: Add `set_string()` and `set_number()` methods for testing purposes

### P2-3: No Method to List All Flags
- **Severity**: Medium
- **Module**: flag.rs
- **Description**: No method to retrieve all registered flags (useful for debugging/admin)
- **Fix**: Add methods like `all_flags()`, `all_string_flags()`, `all_number_flags()`

---

## Technical Debt

| Item | Severity | Description |
|------|----------|-------------|
| `#[allow(dead_code)]` scattered | Low | 3 instances - indicates unused public API |
| No error handling in `load_from_env()` | Low | Silently ignores invalid number values |
| No logging of flag values at startup | Low | Debugging flags without visibility is difficult |
| No documentation comments on implementation | Low | Only `lib.rs` has doc comment re-export |

---

## Missing Tests

Per PRD specification, these tests must exist but are completely absent:

```
#[cfg(test)]
mod tests {
    // Boolean flag tests
    #[test] fn new_manager_has_all_boolean_flags()
    #[test] fn default_values_are_false_except_markdown()
    #[test] fn set_overrides_value()
    #[test] fn unknown_flag_returns_false()
    #[test] fn exa_enabled_when_experimental_is_true()
    #[test] fn plan_mode_enabled_when_experimental_is_true()

    // String/Number flag tests
    #[test] fn opencode_client_defaults_to_cli()
    #[test] fn bash_timeout_has_default()
    #[test] fn string_flag_returns_none_when_not_set()
    #[test] fn number_flag_returns_none_when_not_set()
}
```

**Current test status**: 0/10 tests implemented

---

## Implementation Progress Summary

| Category | Total | Completed | Missing | % Complete |
|----------|-------|-----------|---------|------------|
| Core Types | 2 | 2 | 0 | 100% |
| Boolean Flags | 28 | 28 | 0 | 100% |
| String Flags | 11 | 11 | 0 | 100% |
| Number Flags | 2 | 2 | 0 | 100% |
| Core Methods | 7 | 7 | 0 | 100% |
| Convenience Methods | 9 | 9 | 0 | 100% |
| Unit Tests | 10 | 0 | 10 | 0% |
| Integration | 1 | 0 | 1 | 0% |

**Overall Implementation**: ~85% for the module itself, but 0% for integration and testing

---

## Gap Summary Table

| Gap Item | Severity | Module | Fix Priority |
|----------|----------|--------|--------------|
| FlagManager never instantiated | P0 | Application init | Immediate |
| load_from_env() never called | P0 | Application init | Immediate |
| No unit tests | P0 | flag.rs tests | Immediate |
| Unregistered OPENCODE_EXPERIMENTAL_EXA | P1 | flag.rs | High |
| No application integration | P1 | Throughout app | High |
| truthy() not used in load_from_env() | P2 | flag.rs | Medium |
| No set_string/set_number methods | P2 | flag.rs | Medium |
| No method to list all flags | P2 | flag.rs | Medium |

---

## Recommended Actions

### Immediate (P0)
1. Add `FlagManager` instantiation and `load_from_env()` call at application startup
2. Implement all 10 unit tests as specified in PRD

### High Priority (P1)
3. Register `OPENCODE_EXPERIMENTAL_EXA` or remove reference from `opencode_enable_exa()`
4. Audit and replace hardcoded feature flags with `FlagManager` queries throughout codebase

### Medium Priority (P2)
5. Refactor `load_from_env()` to use `truthy()` helper
6. Add `set_string()` and `set_number()` methods for testing
7. Add `all_flags()` method for debugging/admin
