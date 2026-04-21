# Specification: Module `env` (Iteration 39)

**Crate**: `opencode-core`
**Source**: `opencode-rust/crates/core/src/env.rs`
**Visibility**: `pub(crate)` — internal to `opencode-core`
**Status**: ✅ Fully Implemented — PRD reflects actual Rust API
**Last Updated**: 2026-04-21
**Gap Score**: ~3% (Missing one test case)

---

## 1. Overview

The `env` module provides instance-isolated environment variable access via `EnvManager`. It initializes from the process environment and allows per-instance overrides without mutating `std::env`.

**Design Principles**:
- Instance isolation: changes to `EnvManager` do not affect `std::env`
- Thread-safe access via `RwLock`
- Poison-safe lock handling (recovers data from panicking threads)
- No async dependencies — safe for sync and `tokio::spawn_blocking` contexts

---

## 2. Module Structure

```
opencode-rust/crates/core/src/
└── env.rs              ← EnvManager struct, impl methods, tests
```

**No external re-export** — used internally in `opencode-core` only.

---

## 3. Type Definitions

### FR-390: EnvManager Struct

```rust
#[allow(dead_code)]
pub(crate) struct EnvManager {
    /// Per-instance environment variables
    env: RwLock<HashMap<String, String>>,
}
```

**Fields**:
| Field | Type | Description |
|-------|------|-------------|
| `env` | `RwLock<HashMap<String, String>>` | Thread-safe map of env vars |

**Visibility**: `pub(crate)` — accessible within `opencode-core`

---

## 4. Core Methods

### FR-391: EnvManager::new()

```rust
pub fn new() -> Self
```

**Description**: Creates a new manager populated with current process env vars.

**Behavior**:
1. Creates empty `HashMap`
2. Iterates over `std::env::vars()` and inserts all key-value pairs
3. Wraps in `RwLock` and returns

**Returns**: `Self` (fully initialized `EnvManager`)

---

### FR-392: EnvManager::get()

```rust
pub fn get(&self, key: &str) -> Option<String>
```

**Description**: Get an environment variable by key.

**Parameters**:
| Param | Type | Description |
|-------|------|-------------|
| `key` | `&str` | Variable name to look up |

**Returns**: `Option<String>` — `Some(value)` if found, `None` if not present

**Poison Handling**: Uses `.unwrap_or_else(|p| p.into_inner())` to recover from poisoned locks

---

### FR-393: EnvManager::all()

```rust
pub fn all(&self) -> HashMap<String, String>
```

**Description**: Get all environment variables as a cloned HashMap.

**Returns**: `HashMap<String, String>` — snapshot of all variables at call time

**Poison Handling**: Uses `.unwrap_or_else(|p| p.into_inner())` to recover from poisoned locks

---

### FR-394: EnvManager::set()

```rust
pub fn set(&self, key: String, value: String)
```

**Description**: Set (or overwrite) an environment variable.

**Parameters**:
| Param | Type | Description |
|-------|------|-------------|
| `key` | `String` | Variable name |
| `value` | `String` | Variable value |

**Behavior**:
- Acquires write lock
- Inserts or overwrites the key-value pair
- **Does NOT modify `std::env`**

**Poison Handling**: Uses `.unwrap_or_else(|p| p.into_inner())` to recover from poisoned locks

---

### FR-395: EnvManager::remove()

```rust
pub fn remove(&self, key: &str)
```

**Description**: Remove an environment variable. No-op if not present.

**Parameters**:
| Param | Type | Description |
|-------|------|-------------|
| `key` | `&str` | Variable name to remove |

**Behavior**:
- Acquires write lock
- Removes the key if present, does nothing if absent
- **Does NOT modify `std::env`**

**Poison Handling**: Uses `.unwrap_or_else(|p| p.into_inner())` to recover from poisoned locks

---

### FR-396: EnvManager::env()

```rust
pub fn env(&self) -> std::sync::RwLockReadGuard<'_, HashMap<String, String>>
```

**Description**: Borrow the env map for reading multiple values without cloning.

**Returns**: `RwLockReadGuard<HashMap<String, String>>` — guard for read access

**Use Case**: Batch reads when you need to check multiple variables efficiently

**Poison Handling**: Uses `.unwrap_or_else(|p| p.into_inner())` to recover from poisoned locks

---

### FR-397: EnvManager::default()

```rust
impl Default for EnvManager {
    fn default() -> Self {
        Self::new()
    }
}
```

**Description**: Default constructor delegates to `new()` — same initialization behavior

---

## 5. Concurrency Pattern

### FR-398: RwLock-based Thread Safety

```rust
std::sync::RwLock<HashMap<String, String>>
```

| Operation | Lock Type | Concurrency |
|-----------|----------|-------------|
| `get()` | `read()` | Multiple concurrent readers |
| `all()` | `read()` | Multiple concurrent readers |
| `env()` | `read()` | Multiple concurrent readers |
| `set()` | `write()` | Exclusive writer |
| `remove()` | `write()` | Exclusive writer |

**Not async** — safe to use in sync contexts and in `tokio::spawn_blocking`

---

## 6. Poison Handling

### FR-399: Poison Recovery

Both `read()` and `write()` use `.unwrap_or_else(|p| p.into_inner())`:

```rust
self.env
    .read()
    .unwrap_or_else(|poisoned| poisoned.into_inner())
```

**Rationale**: If a previous thread panicked while holding the lock, the data is still valid (HashMap is heap-allocated). This pattern recovers the data rather than propagating the panic.

---

## 7. Usage Pattern

### FR-400: Basic Usage

```rust
use opencode_core::env::EnvManager;

let env = EnvManager::new();

// Inherit process env
let path = env.get("PATH").unwrap_or_default();

// Override for this instance (doesn't affect std::env)
env.set("OPENCODE_MODEL".into(), "gpt-4o".into());

// Use in LLM provider selection
let model = env.get("OPENCODE_MODEL")
    .unwrap_or_else(|| "claude-3-5-sonnet".into());

// Pass to subprocesses by extracting all vars
let vars = env.all(); // HashMap<String, String>
std::process::Command::new("bash")
    .envs(vars)
    .spawn()
    .unwrap();
```

### FR-401: Instance Isolation

```rust
let env = EnvManager::new();

// This only affects the `env` instance
env.set("ISOLATION_TEST".into(), "isolated".into());

// std::env is NOT affected — this returns Err
assert!(std::env::var("ISOLATION_TEST").is_err());
```

---

## 8. Relationship to FlagManager

### FR-402: FlagManager Distinction

| Aspect | `EnvManager` | `FlagManager` |
|--------|--------------|---------------|
| Source | Instance snapshot | `std::env::var()` directly |
| Mutation | Per-instance overrides | N/A (read-only) |
| Use Case | Subprocess env propagation | Feature flag checking at startup |

**In practice**:
- `EnvManager` — used when spawning subprocesses with custom env
- `FlagManager` — used to check feature flags at startup

---

## 9. Test Specification

### FR-403: Test Requirements

| Test ID | Test Name | Status | Notes |
|---------|-----------|--------|-------|
| FR-403.1 | `initializes_from_process_env` | ✅ Implemented | PATH or HOME must exist |
| FR-403.2 | `set_and_get_custom_var` | ✅ Implemented | Set and retrieve custom var |
| FR-403.3 | `set_overwrites_existing` | ✅ Implemented | Overwrite triggers new value |
| FR-403.4 | `remove_deletes_var` | ✅ Implemented | Remove returns None |
| FR-403.5 | `remove_nonexistent_is_noop` | ✅ Implemented | No panic on missing key |
| FR-403.6 | `all_returns_snapshot` | ✅ Implemented | all() returns cloned map |
| FR-403.7 | `env_guard_is_nonempty` | ✅ Implemented | Guard provides access |
| FR-403.8 | `default_is_same_as_new` | ✅ Implemented | Default delegates to new |
| FR-403.9 | `instance_isolation_does_not_affect_process_env` | ❌ **Missing** | **P1 Gap** |

### FR-404: Missing Test Implementation

The following test is **required but missing**:

```rust
#[test]
fn test_env_instance_isolation() {
    let env = EnvManager::new();

    // Set a variable on the EnvManager instance
    env.set("ISOLATION_TEST".into(), "isolated".into());

    // Verify it's accessible via EnvManager
    assert_eq!(env.get("ISOLATION_TEST"), Some("isolated".into()));

    // Verify std::env is NOT affected
    assert!(std::env::var("ISOLATION_TEST").is_err());
}
```

**Priority**: P1 — blocks test coverage completeness

---

## 10. Technical Debt

### FR-405: Known Technical Debt

| Item | Type | Description | Remediation |
|------|------|-------------|-------------|
| `#[allow(dead_code)]` on `EnvManager` | Dead Code | Struct marked but never used outside module | Add actual use points or document why |
| `#[allow(dead_code)]` on `impl` block | Dead Code | Impl block marked but all methods used internally | Remove suppression, add use points |
| Missing instance isolation test | Test Gap | 88.9% coverage (8/9 tests) | Add `test_env_instance_isolation` |

---

## 11. Implementation Checklist

| Requirement | ID | Status | Notes |
|------------|----|--------|-------|
| EnvManager struct | FR-390 | ✅ Implemented | RwLock-wrapped HashMap |
| new() | FR-391 | ✅ Implemented | Copies from std::env |
| get() | FR-392 | ✅ Implemented | Returns Option<String> |
| all() | FR-393 | ✅ Implemented | Returns cloned HashMap |
| set() | FR-394 | ✅ Implemented | Instance-only mutation |
| remove() | FR-395 | ✅ Implemented | No-op if absent |
| env() | FR-396 | ✅ Implemented | Returns read guard |
| Default impl | FR-397 | ✅ Implemented | Delegates to new() |
| RwLock concurrency | FR-398 | ✅ Implemented | Read/write lock pattern |
| Poison handling | FR-399 | ✅ Implemented | unwrap_or_else pattern |
| Usage patterns | FR-400-401 | ✅ Implemented | Subprocess env, isolation |
| FlagManager distinction | FR-402 | ✅ Implemented | Parallel mechanisms |
| Tests | FR-403 | ⚠️ 8/9 | Missing FR-403.9 |
| Technical debt | FR-405 | ⚠️ Low | Dead code warnings, missing test |

---

## 12. Gap Summary

| Gap Item | Severity | Module | Remediation |
|---------|----------|--------|-------------|
| Missing `instance_isolation_does_not_affect_process_env` test | P1 | env::tests | Add test verifying std::env::var() returns Err after env.set() |

---

## 13. Conclusion

The `env` module is **well-implemented** and closely follows the PRD specification. All core functionality (7/7 features) and interfaces (6/6) are implemented correctly.

**Blocking issues**: None

**Minor gaps**:
- Test coverage 88.9% (8/9) — missing instance isolation test
- Two `#[allow(dead_code)]` attributes that should be addressed

**Overall assessment**: Module is **production-ready** with minor test gap. The missing test should be added to achieve 100% test coverage per PRD requirements.

---

*Specification generated by Sisyphus gap analysis pipeline*
*FR numbers: FR-390 to FR-405 (aligned to iteration 39)*
