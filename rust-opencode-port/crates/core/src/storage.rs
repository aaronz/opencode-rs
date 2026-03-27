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

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_storage_save_load() {
        let tmp = TempDir::new().unwrap();
        let storage = Storage::new(tmp.path().to_path_buf());

        #[derive(Serialize, Deserialize, PartialEq, Debug)]
        struct Data {
            value: i32,
        }

        storage.save("test", &Data { value: 42 }).unwrap();
        let loaded: Data = storage.load("test").unwrap();

        assert_eq!(loaded.value, 42);
    }

    #[test]
    fn test_storage_exists() {
        let tmp = TempDir::new().unwrap();
        let storage = Storage::new(tmp.path().to_path_buf());

        assert!(!storage.exists("test"));
        storage.save("test", &"data").unwrap();
        assert!(storage.exists("test"));
    }

    #[test]
    fn test_storage_delete() {
        let tmp = TempDir::new().unwrap();
        let storage = Storage::new(tmp.path().to_path_buf());

        storage.save("test", &"data").unwrap();
        assert!(storage.exists("test"));

        storage.delete("test").unwrap();
        assert!(!storage.exists("test"));
    }

    #[test]
    fn test_storage_list_keys() {
        let tmp = TempDir::new().unwrap();
        let storage = Storage::new(tmp.path().to_path_buf());

        storage.save("key1", &"data1").unwrap();
        storage.save("key2", &"data2").unwrap();

        let keys = storage.list_keys().unwrap();
        assert!(keys.contains(&"key1".to_string()));
        assert!(keys.contains(&"key2".to_string()));
    }

    #[test]
    fn test_storage_load_not_found() {
        let tmp = TempDir::new().unwrap();
        let storage = Storage::new(tmp.path().to_path_buf());

        let result: Result<String, _> = storage.load("nonexistent");
        assert!(result.is_err());
    }
}
