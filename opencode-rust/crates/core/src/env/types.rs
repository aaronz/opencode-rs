use std::collections::HashMap;
use std::sync::RwLock;

#[allow(dead_code)]
pub(crate) struct EnvManager {
    env: RwLock<HashMap<String, String>>,
}

#[allow(dead_code)]
impl EnvManager {
    pub fn new() -> Self {
        let mut env = HashMap::new();
        for (key, value) in std::env::vars() {
            env.insert(key, value);
        }
        Self {
            env: RwLock::new(env),
        }
    }

    pub fn get(&self, key: &str) -> Option<String> {
        self.env
            .read()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .get(key)
            .cloned()
    }

    pub fn all(&self) -> HashMap<String, String> {
        self.env
            .read()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .clone()
    }

    pub fn set(&self, key: String, value: String) {
        self.env
            .write()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .insert(key, value);
    }

    pub fn remove(&self, key: &str) {
        self.env
            .write()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .remove(key);
    }

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