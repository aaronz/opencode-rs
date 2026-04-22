# Specification: flag Module (Iteration 46)

**Document Version**: 46
**Date**: 2026-04-22
**Status**: Updated from Gap Analysis
**Source PRD**: `docs/PRD/modules/flag.md`
**Implementation**: `opencode-rust/crates/core/src/flag.rs`

---

## Module Overview

- **Module Name**: `flag`
- **Source Path**: `opencode-rust/crates/core/src/flag.rs`
- **Type**: Configuration/Feature Flags
- **Purpose**: Manages feature flags, runtime configuration switches, and optional string/number parameters. All flags are read from environment variables.
- **Visibility**: `pub(crate)` — internal to `opencode-core`
- **Status**: Module implemented (462 lines), integration and tests missing

---

## Implementation Status Summary

| Component | Status | Gap |
|-----------|--------|-----|
| `Flag` struct | ✅ Implemented | None |
| `FlagManager` struct | ✅ Implemented | None |
| 28 Boolean flags | ✅ All registered | None |
| 11 String flags | ✅ All registered | None |
| 2 Number flags | ✅ All registered | None |
| Core methods (`get`, `get_string`, `get_number`, `set`, `is_enabled`) | ✅ Implemented | None |
| `load_from_env()` | ✅ Implemented | Never called outside module |
| Convenience methods (9 methods) | ✅ All implemented | None |
| Unit tests | ❌ Missing | All 10 tests absent |
| Module instantiation | ❌ Missing | Never instantiated in app |
| `load_from_env()` call | ❌ Missing | Never called in app flow |

**Overall Progress**: ~85% module implementation, 0% integration, 0% tests

---

## Feature Requirements

### FR-001: FlagManager Instantiation and Initialization

**Priority**: P0
**Module**: `flag.rs` / Application entry points
**Status**: ❌ Not Implemented

#### Description
`FlagManager` must be instantiated at application startup and `load_from_env()` must be called to populate flag values from environment variables.

#### Requirements
- [ ] Instantiate `FlagManager` at application startup (e.g., in main entry or config init)
- [ ] Call `flags.load_from_env()` immediately after instantiation
- [ ] Store `FlagManager` instance in a globally accessible location (e.g., `AppState`, `RuntimeContext`)
- [ ] All subsequent feature checks must use the initialized `FlagManager`

#### Implementation Location
```
crates/core/src/lib.rs (re-export already exists at line 169)
Application initialization in crates/cli/src/ or crates/tui/src/
```

---

### FR-002: Unit Tests for Boolean Flags

**Priority**: P0
**Module**: `flag.rs`
**Status**: ❌ Not Implemented

#### Description
Implement unit tests as specified in PRD for boolean flag functionality.

#### Tests Required
```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_manager_has_all_boolean_flags() {
        let fm = FlagManager::new();
        assert!(fm.get("OPENCODE_EXPERIMENTAL").is_some());
        assert!(fm.get("OPENCODE_DEBUG").is_some());
        // ... all 28 flags
    }

    #[test]
    fn default_values_are_false_except_markdown() {
        let fm = FlagManager::new();
        assert!(!fm.is_enabled("OPENCODE_EXPERIMENTAL"));
        assert!(fm.is_enabled("OPENCODE_EXPERIMENTAL_MARKDOWN")); // default true
    }

    #[test]
    fn set_overrides_value() {
        let mut fm = FlagManager::new();
        assert!(!fm.is_enabled("OPENCODE_DEBUG"));
        fm.set("OPENCODE_DEBUG", true);
        assert!(fm.is_enabled("OPENCODE_DEBUG"));
    }

    #[test]
    fn unknown_flag_returns_false() {
        let fm = FlagManager::new();
        assert!(!fm.is_enabled("NONEXISTENT_FLAG"));
    }

    #[test]
    fn exa_enabled_when_experimental_is_true() {
        let mut fm = FlagManager::new();
        fm.set("OPENCODE_EXPERIMENTAL", true);
        assert!(fm.opencode_enable_exa());
    }

    #[test]
    fn plan_mode_enabled_when_experimental_is_true() {
        let mut fm = FlagManager::new();
        fm.set("OPENCODE_EXPERIMENTAL", true);
        assert!(fm.opencode_experimental_plan_mode());
    }
}
```

---

### FR-003: Unit Tests for String and Number Flags

**Priority**: P0
**Module**: `flag.rs`
**Status**: ❌ Not Implemented

#### Tests Required
```rust
#[test]
fn opencode_client_defaults_to_cli() {
    let fm = FlagManager::new();
    assert_eq!(fm.opencode_client(), "cli");
}

#[test]
fn bash_timeout_has_default() {
    let fm = FlagManager::new();
    // Default is 120000 ms
    assert_eq!(fm.opencode_experimental_bash_timeout_ms(), Some(120_000));
}

#[test]
fn string_flag_returns_none_when_not_set() {
    let fm = FlagManager::new();
    assert!(fm.get_string("OPENCODE_CONFIG").is_none());
}

#[test]
fn number_flag_returns_none_when_not_set() {
    let fm = FlagManager::new();
    // Without env var, number flags are None
    assert!(fm.get_number("OPENCODE_EXPERIMENTAL_OUTPUT_TOKEN_MAX").is_none());
}
```

---

### FR-004: Register OPENCODE_EXPERIMENTAL_EXA Flag

**Priority**: P1
**Module**: `flag.rs`
**Status**: ⚠️ Partial (referenced but unregistered)

#### Description
The `opencode_enable_exa()` method references `OPENCODE_EXPERIMENTAL_EXA` but this flag is not registered in `FlagManager::new()`.

#### Current Code
```rust
pub fn opencode_enable_exa(&self) -> bool {
    self.get("OPENCODE_ENABLE_EXA").unwrap_or(false)
        || self.opencode_experimental()
        || truthy("OPENCODE_EXPERIMENTAL_EXA")  // Unregistered!
}
```

#### Requirements
- [ ] Register `OPENCODE_EXPERIMENTAL_EXA` in `FlagManager::new()` with default `false`
- [ ] Or remove the `truthy("OPENCODE_EXPERIMENTAL_EXA")` check if not needed

---

### FR-005: Refactor load_from_env() to Use truthy() Helper

**Priority**: P2
**Module**: `flag.rs`
**Status**: ⚠️ Code duplication

#### Description
`load_from_env()` duplicates the truthy check logic instead of using the existing `truthy()` helper function.

#### Current Code (Line 390)
```rust
flag.value = val == "1" || val.to_lowercase() == "true";
```

#### Requirements
- [ ] Refactor to use `truthy(name)` for consistency
- [ ] Both approaches are functionally equivalent

---

### FR-006: Add Setter Methods for String and Number Flags

**Priority**: P2
**Module**: `flag.rs`
**Status**: ❌ Not Implemented

#### Description
Only boolean flags have a `set()` method. String and number flags can only be set via `load_from_env()`.

#### API
```rust
pub fn set_string(&mut self, name: &str, value: Option<String>) {
    if let Some(v) = self.string_flags.get_mut(name) {
        *v = value;
    }
}

pub fn set_number(&mut self, name: &str, value: Option<u64>) {
    if let Some(v) = self.number_flags.get_mut(name) {
        *v = value;
    }
}
```

#### Requirements
- [ ] Add `set_string()` method for testing
- [ ] Add `set_number()` method for testing

---

### FR-007: Add Method to List All Flags

**Priority**: P2
**Module**: `flag.rs`
**Status**: ❌ Not Implemented

#### Description
No method to retrieve all registered flags (useful for debugging/admin).

#### API
```rust
pub fn all_flags(&self) -> &HashMap<String, Flag>
pub fn all_string_flags(&self) -> &HashMap<String, Option<String>>
pub fn all_number_flags(&self) -> &HashMap<String, Option<u64>>
```

#### Requirements
- [ ] Add getter methods for all flag collections
- [ ] Useful for debugging and admin interfaces

---

### FR-008: Application Integration - Replace Hardcoded Feature Checks

**Priority**: P1
**Module**: Throughout application
**Status**: ❌ Not Implemented

#### Description
No part of the application currently checks feature flags. All conditional logic based on flags is non-functional.

#### Requirements
- [ ] Audit codebase for hardcoded feature checks
- [ ] Replace with `FlagManager` queries
- [ ] Ensure all feature toggles use the flag system

---

## Core Types

### `Flag`

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
#[allow(dead_code)]
pub(crate) struct Flag {
    pub name: String,
    pub description: String,
    pub default: bool,
    pub value: bool,
}
```

---

### `FlagManager`

```rust
#[allow(dead_code)]
pub(crate) struct FlagManager {
    flags: HashMap<String, Flag>,
    string_flags: HashMap<String, Option<String>>,
    number_flags: HashMap<String, Option<u64>>,
}
```

---

## All Registered Flags

### Boolean Flags (`OPENCODE_*`)

| Env Var | Default | Description |
|---|---|---|
| `OPENCODE_EXPERIMENTAL` | false | Master experimental gate |
| `OPENCODE_DEBUG` | false | Debug mode |
| `OPENCODE_AUTO_SHARE` | false | Auto-share sessions |
| `OPENCODE_DISABLE_AUTOUPDATE` | false | Disable auto-update |
| `OPENCODE_ALWAYS_NOTIFY_UPDATE` | false | Always notify about updates |
| `OPENCODE_DISABLE_PRUNE` | false | Disable pruning |
| `OPENCODE_DISABLE_TERMINAL_TITLE` | false | Disable terminal title |
| `OPENCODE_DISABLE_DEFAULT_PLUGINS` | false | Disable default plugins |
| `OPENCODE_DISABLE_LSP_DOWNLOAD` | false | Disable LSP server download |
| `OPENCODE_ENABLE_EXPERIMENTAL_MODELS` | false | Enable experimental models |
| `OPENCODE_DISABLE_AUTOCOMPACT` | false | Disable auto-compaction |
| `OPENCODE_DISABLE_MODELS_FETCH` | false | Disable model catalog fetch |
| `OPENCODE_DISABLE_CLAUDE_CODE` | false | Disable Claude Code features |
| `OPENCODE_ENABLE_QUESTION_TOOL` | false | Enable question tool |
| `OPENCODE_EXPERIMENTAL_FILEWATCHER` | false | Enable file watcher |
| `OPENCODE_EXPERIMENTAL_DISABLE_FILEWATCHER` | false | Disable file watcher |
| `OPENCODE_EXPERIMENTAL_ICON_DISCOVERY` | false | Enable icon discovery |
| `OPENCODE_EXPERIMENTAL_DISABLE_COPY_ON_SELECT` | `cfg!(windows)` | Disable copy-on-select |
| `OPENCODE_ENABLE_EXA` | false | Enable Exa web search |
| `OPENCODE_EXPERIMENTAL_OXFMT` | false | Enable oxfmt formatter |
| `OPENCODE_EXPERIMENTAL_LSP_TY` | false | Enable LSP ty |
| `OPENCODE_EXPERIMENTAL_LSP_TOOL` | false | Enable LSP tool |
| `OPENCODE_DISABLE_FILETIME_CHECK` | false | Disable file time check |
| `OPENCODE_EXPERIMENTAL_PLAN_MODE` | false | Enable plan mode |
| `OPENCODE_EXPERIMENTAL_WORKSPACES` | false | Enable workspaces |
| `OPENCODE_EXPERIMENTAL_MARKDOWN` | **true** | Enable markdown rendering |
| `OPENCODE_EXPERIMENTAL_VARIANT_REASONING` | false | Enable variant/reasoning budget |
| `OPENCODE_DISABLE_CHANNEL_DB` | false | Disable channel DB |
| `OPENCODE_SKIP_MIGRATIONS` | false | Skip DB migrations |
| `OPENCODE_STRICT_CONFIG_DEPS` | false | Strict config dependency checking |

---

### String Flags

| Env Var | Purpose |
|---|---|
| `OPENCODE_GIT_BASH_PATH` | Override git-bash executable path |
| `OPENCODE_CONFIG` | Path to config file override |
| `OPENCODE_CONFIG_CONTENT` | Inline config JSON/TOML |
| `OPENCODE_PERMISSION` | Permission scope override |
| `OPENCODE_FAKE_VCS` | Fake VCS backend for testing |
| `OPENCODE_CLIENT` | Client type identifier (default: "cli") |
| `OPENCODE_SERVER_PASSWORD` | Server auth password |
| `OPENCODE_SERVER_USERNAME` | Server auth username |
| `OPENCODE_MODELS_URL` | Override models catalog URL |
| `OPENCODE_MODELS_PATH` | Local models catalog path |
| `OPENCODE_DB` | Override SQLite DB path |

---

### Number Flags

| Env Var | Purpose |
|---|---|
| `OPENCODE_EXPERIMENTAL_BASH_DEFAULT_TIMEOUT_MS` | Bash tool timeout (default: 120000) |
| `OPENCODE_EXPERIMENTAL_OUTPUT_TOKEN_MAX` | Max output tokens |

---

## Key Implementations

### Core Methods

```rust
impl FlagManager {
    pub fn new() -> Self { /* registers all flags */ }

    pub fn get(&self, name: &str) -> Option<bool>
    pub fn get_string(&self, name: &str) -> Option<String>
    pub fn get_number(&self, name: &str) -> Option<u64>
    pub fn set(&mut self, name: &str, value: bool)
    pub fn is_enabled(&self, name: &str) -> bool
    pub fn load_from_env(&mut self)
}
```

### Convenience Methods

```rust
impl FlagManager {
    pub fn opencode_auto_share(&self) -> bool
    pub fn opencode_client(&self) -> String  // default: "cli"
    pub fn opencode_enable_question_tool(&self) -> bool
    pub fn opencode_experimental(&self) -> bool

    // Exa enabled if OPENCODE_ENABLE_EXA OR OPENCODE_EXPERIMENTAL
    pub fn opencode_enable_exa(&self) -> bool

    // Plan mode if OPENCODE_EXPERIMENTAL OR OPENCODE_EXPERIMENTAL_PLAN_MODE
    pub fn opencode_experimental_plan_mode(&self) -> bool

    // LSP tool if OPENCODE_EXPERIMENTAL OR OPENCODE_EXPERIMENTAL_LSP_TOOL
    pub fn opencode_experimental_lsp_tool(&self) -> bool

    // Variant reasoning if OPENCODE_EXPERIMENTAL OR OPENCODE_EXPERIMENTAL_VARIANT_REASONING
    pub fn opencode_experimental_variant_reasoning(&self) -> bool

    // Bash timeout: OPENCODE_EXPERIMENTAL_BASH_DEFAULT_TIMEOUT_MS or default 120000ms
    pub fn opencode_experimental_bash_timeout_ms(&self) -> Option<u64>
}
```

---

## Usage Pattern

```rust
use opencode_core::flag::FlagManager;

let mut flags = FlagManager::new();
flags.load_from_env();  // Call once at startup

// Check features
if flags.opencode_experimental() {
    println!("Running in experimental mode");
}

if flags.is_enabled("OPENCODE_DISABLE_AUTOCOMPACT") {
    // skip compaction
}

// Get string config
let db_path = flags.get_string("OPENCODE_DB");
let client_type = flags.opencode_client(); // "cli", "tui", "desktop"

// Get bash timeout
let timeout = flags.opencode_experimental_bash_timeout_ms().unwrap_or(120_000);
```

---

## Gap Summary

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

---

## Test Design

| Test | Feature | FR |
|------|---------|-----|
| `new_manager_has_all_boolean_flags` | All 28 flags registered | FR-002 |
| `default_values_are_false_except_markdown` | Default values | FR-002 |
| `set_overrides_value` | Boolean setter | FR-002 |
| `unknown_flag_returns_false` | Fallback behavior | FR-002 |
| `exa_enabled_when_experimental_is_true` | Convenience method | FR-002 |
| `plan_mode_enabled_when_experimental_is_true` | Convenience method | FR-002 |
| `opencode_client_defaults_to_cli` | String flag default | FR-003 |
| `bash_timeout_has_default` | Number flag default | FR-003 |
| `string_flag_returns_none_when_not_set` | String flag behavior | FR-003 |
| `number_flag_returns_none_when_not_set` | Number flag behavior | FR-003 |

---

## Dependencies

| Dependency | Purpose | Status |
|------------|---------|--------|
| `serde` with derive | Serialization | ✅ Exists |
| `std::collections::HashMap` | Flag storage | ✅ Built-in |
| `std::env` | Environment variable access | ✅ Built-in |

---

## Files to Modify

| File | Changes |
|------|---------|
| `opencode-rust/crates/core/src/flag.rs` | Add tests, fix unregistered flag, add setters |
| `opencode-rust/crates/core/src/lib.rs` | Export FlagManager publicly if needed |
| Application entry points | Instantiate FlagManager and call load_from_env() |

---

## Verification Checklist

- [ ] `FlagManager::new()` creates all 28 boolean flags
- [ ] `load_from_env()` populates flag values from environment
- [ ] All convenience methods work correctly
- [ ] `opencode_client()` defaults to "cli"
- [ ] `opencode_experimental_bash_timeout_ms()` defaults to 120000
- [ ] All 10 unit tests pass
- [ ] `FlagManager` is instantiated at application startup
- [ ] `load_from_env()` is called at startup
- [ ] `OPENCODE_EXPERIMENTAL_EXA` is registered or removed

---

*Document generated: 2026-04-22*
*Based on: PRD (docs/PRD/modules/flag.md) + Gap Analysis (iteration-46)*
