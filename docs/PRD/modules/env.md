# Module: env

## Overview

The `env` module in `opencode-core` (`crates/core/src/env.rs`) provides instance-isolated environment variable access via `EnvManager`. It initializes from the process environment and allows per-instance overrides without mutating `std::env`.

**Crate**: `opencode-core`  
**Source**: `crates/core/src/env.rs`  
**Visibility**: `pub(crate)` — internal to `opencode-core`  
**Status**: Fully implemented (141 lines)

---

## Crate Layout

```
crates/core/src/
└── env.rs
```

**No external re-export** — used internally in `opencode-core` only. If downstream crates need it, expose via `crate::env::EnvManager` or add a `pub use`.

---

## Core Types

### `EnvManager`

```rust
pub(crate) struct EnvManager {
    env: RwLock<HashMap<String, String>>,
}
```

Initialized by copying the current process environment (`std::env::vars()`). Subsequent `set`/`remove` operations affect only this instance.

---

## Key Implementations

```rust
impl EnvManager {
    /// Creates a new manager populated with current process env vars.
    pub fn new() -> Self {
        let mut env = HashMap::new();
        for (key, value) in std::env::vars() {
            env.insert(key, value);
        }
        Self { env: RwLock::new(env) }
    }

    /// Get a variable by key. Returns None if not present.
    pub fn get(&self, key: &str) -> Option<String> {
        self.env
            .read()
            .unwrap_or_else(|p| p.into_inner())
            .get(key)
            .cloned()
    }

    /// Get all variables as a cloned HashMap.
    pub fn all(&self) -> HashMap<String, String> {
        self.env
            .read()
            .unwrap_or_else(|p| p.into_inner())
            .clone()
    }

    /// Set (or overwrite) a variable.
    pub fn set(&self, key: String, value: String) {
        self.env
            .write()
            .unwrap_or_else(|p| p.into_inner())
            .insert(key, value);
    }

    /// Remove a variable (no-op if not present).
    pub fn remove(&self, key: &str) {
        self.env
            .write()
            .unwrap_or_else(|p| p.into_inner())
            .remove(key);
    }

    /// Borrow the env map for reading multiple values without cloning.
    pub fn env(&self) -> std::sync::RwLockReadGuard<'_, HashMap<String, String>> {
        self.env
            .read()
            .unwrap_or_else(|p| p.into_inner())
    }
}

impl Default for EnvManager {
    fn default() -> Self { Self::new() }
}
```

**Poison handling**: Both `read()` and `write()` use `.unwrap_or_else(|p| p.into_inner())` to recover from poisoned locks — the data is still valid even if a prior panicking thread held the lock.

---

## Concurrency Pattern

`std::sync::RwLock<HashMap<String, String>>` — multiple concurrent readers, exclusive writer:
- Reads: `read()` → shared access
- Writes: `write()` → exclusive access
- **Not async** — safe to use in sync contexts and in tokio `spawn_blocking`

---

## Usage Pattern

```rust
use opencode_core::env::EnvManager;

let env = EnvManager::new();

// Inherit process env
let path = env.get("PATH").unwrap_or_default();

// Override for this instance (doesn't affect std::env)
env.set("OPENCODE_MODEL".into(), "gpt-4o".into());

// Use in LLM provider selection
let model = env.get("OPENCODE_MODEL").unwrap_or_else(|| "claude-3-5-sonnet".into());

// Pass to subprocesses by extracting all vars
let vars = env.all(); // HashMap<String, String>
std::process::Command::new("bash")
    .envs(vars)
    .spawn()
    .unwrap();
```

---

## Relationship to `FlagManager`

`FlagManager` (`crates/core/src/flag.rs`) reads from `std::env::var()` directly, not from `EnvManager`. They are parallel mechanisms:
- `EnvManager` — instance-isolated snapshot of env
- `FlagManager` — typed feature flag accessor (reads live `std::env`)

In practice, `EnvManager` is used when spawning subprocesses with custom env, while `FlagManager` is used to check feature flags at startup.

---

## Test Design

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn initializes_from_process_env() {
        let env = EnvManager::new();
        // PATH or HOME must exist on any Unix system
        assert!(env.get("PATH").is_some() || env.get("HOME").is_some());
    }

    #[test]
    fn set_and_get_custom_var() {
        let env = EnvManager::new();
        env.set("MY_TEST_VAR".into(), "hello".into());
        assert_eq!(env.get("MY_TEST_VAR"), Some("hello".into()));
    }

    #[test]
    fn set_overwrites_existing() {
        let env = EnvManager::new();
        env.set("X".into(), "1".into());
        env.set("X".into(), "2".into());
        assert_eq!(env.get("X"), Some("2".into()));
    }

    #[test]
    fn remove_deletes_var() {
        let env = EnvManager::new();
        env.set("REMOVE_ME".into(), "yes".into());
        env.remove("REMOVE_ME");
        assert_eq!(env.get("REMOVE_ME"), None);
    }

    #[test]
    fn remove_nonexistent_is_noop() {
        let env = EnvManager::new();
        env.remove("DEFINITELY_NOT_SET_XYZ");
        // no panic
    }

    #[test]
    fn all_returns_snapshot() {
        let env = EnvManager::new();
        env.set("SNAP_TEST".into(), "value".into());
        let all = env.all();
        assert_eq!(all.get("SNAP_TEST"), Some(&"value".into()));
    }

    #[test]
    fn env_guard_is_nonempty() {
        let env = EnvManager::new();
        let guard = env.env();
        assert!(!guard.is_empty());
    }

    #[test]
    fn default_is_same_as_new() {
        let env = EnvManager::default();
        assert!(env.get("PATH").is_some() || env.get("HOME").is_some());
    }

    #[test]
    fn instance_isolation_does_not_affect_process_env() {
        let env = EnvManager::new();
        env.set("ISOLATION_TEST".into(), "isolated".into());
        // std::env should NOT be affected
        assert!(std::env::var("ISOLATION_TEST").is_err());
    }
}
```
