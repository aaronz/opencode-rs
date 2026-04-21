//! Environment variable management module
//! Provides instance-isolated environment variable access

use std::collections::HashMap;
use std::sync::RwLock;

/// Manages environment variables with instance isolation.
///
/// # Purpose
/// `EnvManager` is designed for subprocess environment propagation. When spawning
/// subprocesses with custom environment variables (e.g., LLM provider config),
/// `EnvManager::all()` can be used with `std::process::Command::envs()` to pass
/// instance-isolated env vars without affecting `std::env`.
///
/// # Current Status
/// Currently only used in tests within this module. The `#[allow(dead_code)]`
/// suppression is justified because:
/// - The struct is `pub(crate)` and ready for integration
/// - Planned integration points: LLM provider selection, subprocess spawning
/// - All methods are exercised via unit tests (9/9 coverage)
#[allow(dead_code)]
pub(crate) struct EnvManager {
    /// Per-instance environment variables
    env: RwLock<HashMap<String, String>>,
}

#[allow(dead_code)]
impl EnvManager {
    pub fn new() -> Self {
        // Initialize with a copy of the current process environment
        let mut env = HashMap::new();
        for (key, value) in std::env::vars() {
            env.insert(key, value);
        }
        Self {
            env: RwLock::new(env),
        }
    }

    /// Get an environment variable
    pub fn get(&self, key: &str) -> Option<String> {
        self.env
            .read()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .get(key)
            .cloned()
    }

    /// Get all environment variables
    pub fn all(&self) -> HashMap<String, String> {
        self.env
            .read()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .clone()
    }

    /// Set an environment variable
    pub fn set(&self, key: String, value: String) {
        self.env
            .write()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .insert(key, value);
    }

    /// Remove an environment variable
    pub fn remove(&self, key: &str) {
        self.env
            .write()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .remove(key);
    }

    /// Get a reference to the environment (for reading multiple values)
    pub fn env(&self) -> std::sync::RwLockReadGuard<'_, HashMap<String, String>> {
        self.env
            .read()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
    }
}

impl Default for EnvManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_env_get_set() {
        let env = EnvManager::new();

        // Should have some system env vars
        assert!(env.get("PATH").is_some() || env.get("HOME").is_some());

        // Set and get custom var
        env.set("TEST_VAR".to_string(), "test_value".to_string());
        assert_eq!(env.get("TEST_VAR"), Some("test_value".to_string()));

        // Remove and verify
        env.remove("TEST_VAR");
        assert_eq!(env.get("TEST_VAR"), None);
    }

    #[test]
    fn test_env_all() {
        let env = EnvManager::new();
        let all = env.all();

        assert!(!all.is_empty());
        assert!(all.contains_key("PATH") || all.contains_key("HOME"));
    }

    #[test]
    fn test_env_env_guard() {
        let env = EnvManager::new();

        let guard = env.env();
        assert!(!guard.is_empty());
    }

    #[test]
    fn test_env_manager_new() {
        let env = EnvManager::new();
        assert!(env.get("PATH").is_some() || env.get("HOME").is_some());
    }

    #[test]
    fn test_env_manager_default() {
        let env = EnvManager::default();
        assert!(env.get("PATH").is_some() || env.get("HOME").is_some());
    }

    #[test]
    fn test_env_set_overwrites() {
        let env = EnvManager::new();

        env.set("TEST_VAR".to_string(), "value1".to_string());
        assert_eq!(env.get("TEST_VAR"), Some("value1".to_string()));

        env.set("TEST_VAR".to_string(), "value2".to_string());
        assert_eq!(env.get("TEST_VAR"), Some("value2".to_string()));
    }

    #[test]
    fn test_env_remove_nonexistent() {
        let env = EnvManager::new();
        // Should not panic
        env.remove("NONEXISTENT_VAR_12345");
        assert_eq!(env.get("NONEXISTENT_VAR_12345"), None);
    }

    /// Test that EnvManager::set() does NOT affect std::env (instance isolation)
    #[test]
    fn test_env_instance_isolation() {
        let env = EnvManager::new();

        env.set(
            "ISOLATION_TEST_VAR".to_string(),
            "isolated_value".to_string(),
        );

        assert_eq!(
            env.get("ISOLATION_TEST_VAR"),
            Some("isolated_value".to_string())
        );
        assert!(std::env::var("ISOLATION_TEST_VAR").is_err());

        env.remove("ISOLATION_TEST_VAR");
    }

    #[test]
    fn test_env_empty_instance_returns_none() {
        let env = EnvManager::new();
        assert_eq!(env.get("__DOES_NOT_EXIST__"), None);
        assert_eq!(env.get("__ANOTHER_MISSING__"), None);
    }
}
