//! Environment variable management module
//! Provides instance-isolated environment variable access

use std::collections::HashMap;
use std::sync::RwLock;

/// Manages environment variables with instance isolation
pub struct EnvManager {
    /// Per-instance environment variables
    env: RwLock<HashMap<String, String>>,
}

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
}
