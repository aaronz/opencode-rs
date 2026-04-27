use crate::OpenCodeError;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

pub struct Storage {
    base_path: PathBuf,
}

impl Storage {
    pub fn new(base_path: PathBuf) -> Self {
        std::fs::create_dir_all(&base_path).ok();
        Self { base_path }
    }

    pub fn save<T: Serialize>(&self, key: &str, value: &T) -> Result<(), OpenCodeError> {
        let path = self.base_path.join(format!("{}.json", key));
        let json = serde_json::to_string_pretty(value)?;
        std::fs::write(path, json)?;
        Ok(())
    }

    pub fn load<T: for<'de> Deserialize<'de>>(&self, key: &str) -> Result<T, OpenCodeError> {
        let path = self.base_path.join(format!("{}.json", key));
        if !path.exists() {
            return Err(OpenCodeError::Config(format!("Key not found: {}", key)));
        }
        let json = std::fs::read_to_string(path)?;
        Ok(serde_json::from_str(&json)?)
    }

    pub fn exists(&self, key: &str) -> bool {
        self.base_path.join(format!("{}.json", key)).exists()
    }

    pub fn delete(&self, key: &str) -> Result<(), OpenCodeError> {
        let path = self.base_path.join(format!("{}.json", key));
        if path.exists() {
            std::fs::remove_file(path)?;
        }
        Ok(())
    }

    pub fn list_keys(&self) -> Result<Vec<String>, OpenCodeError> {
        let mut keys = Vec::new();
        for entry in std::fs::read_dir(&self.base_path)? {
            let entry = entry?;
            if let Some(name) = entry.file_name().to_str() {
                if name.ends_with(".json") {
                    keys.push(name.trim_end_matches(".json").to_string());
                }
            }
        }
        Ok(keys)
    }
}