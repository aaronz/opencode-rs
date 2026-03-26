use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Flag {
    pub name: String,
    pub description: String,
    pub default: bool,
    pub value: bool,
}

pub struct FlagManager {
    flags: HashMap<String, Flag>,
}

impl FlagManager {
    pub fn new() -> Self {
        let mut flags = HashMap::new();

        flags.insert(
            "OPENCODE_EXPERIMENTAL".to_string(),
            Flag {
                name: "OPENCODE_EXPERIMENTAL".to_string(),
                description: "Enable experimental features".to_string(),
                default: false,
                value: false,
            },
        );

        flags.insert(
            "OPENCODE_DEBUG".to_string(),
            Flag {
                name: "OPENCODE_DEBUG".to_string(),
                description: "Enable debug mode".to_string(),
                default: false,
                value: false,
            },
        );

        Self { flags }
    }

    pub fn get(&self, name: &str) -> Option<bool> {
        self.flags.get(name).map(|f| f.value)
    }

    pub fn set(&mut self, name: &str, value: bool) {
        if let Some(flag) = self.flags.get_mut(name) {
            flag.value = value;
        }
    }

    pub fn is_enabled(&self, name: &str) -> bool {
        self.get(name).unwrap_or(false)
    }

    pub fn load_from_env(&mut self) {
        for (name, flag) in self.flags.iter_mut() {
            if let Ok(val) = std::env::var(name) {
                flag.value = val == "1" || val.to_lowercase() == "true";
            }
        }
    }
}

impl Default for FlagManager {
    fn default() -> Self {
        Self::new()
    }
}
