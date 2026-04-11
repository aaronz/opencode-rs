use std::path::PathBuf;
use thiserror::Error;

const SECRET_FILE_NAME: &str = "secrets.json";

#[derive(Debug, Error)]
pub enum SecretStorageError {
    #[error("secret not found: {0}")]
    NotFound(String),
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),
    #[error("serialization error: {0}")]
    Serialization(#[from] serde_json::Error),
}

pub struct SecretStorage {
    secrets_path: PathBuf,
}

impl SecretStorage {
    pub fn new() -> Self {
        let secrets_path = Self::default_secrets_path();
        Self { secrets_path }
    }

    fn default_secrets_path() -> PathBuf {
        if let Ok(data_dir) = std::env::var("OPENCODE_DATA_DIR") {
            PathBuf::from(data_dir).join(SECRET_FILE_NAME)
        } else if let Some(home) = dirs::home_dir() {
            home.join(".local/share/opencode").join(SECRET_FILE_NAME)
        } else {
            PathBuf::from(".opencode").join(SECRET_FILE_NAME)
        }
    }

    pub fn with_path(path: PathBuf) -> Self {
        Self { secrets_path: path }
    }

    pub fn get_secret(&self, secret_name: &str) -> Result<String, SecretStorageError> {
        if !self.secrets_path.exists() {
            return Err(SecretStorageError::NotFound(secret_name.to_string()));
        }

        let content = std::fs::read_to_string(&self.secrets_path)?;
        let secrets: serde_json::Value = serde_json::from_str(&content)?;

        secrets
            .get(secret_name)
            .and_then(|v| v.as_str())
            .map(|s| s.to_string())
            .ok_or_else(|| SecretStorageError::NotFound(secret_name.to_string()))
    }

    pub fn set_secret(
        &self,
        secret_name: &str,
        secret_value: &str,
    ) -> Result<(), SecretStorageError> {
        let mut secrets = if self.secrets_path.exists() {
            let content = std::fs::read_to_string(&self.secrets_path)?;
            serde_json::from_str(&content).unwrap_or_else(|_| serde_json::json!({}))
        } else {
            serde_json::json!({})
        };

        secrets[secret_name] = serde_json::json!(secret_value);

        if let Some(parent) = self.secrets_path.parent() {
            std::fs::create_dir_all(parent)?;
        }

        let json = serde_json::to_string_pretty(&secrets)?;
        std::fs::write(&self.secrets_path, json)?;

        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            std::fs::set_permissions(&self.secrets_path, std::fs::Permissions::from_mode(0o600))?;
        }

        Ok(())
    }

    pub fn delete_secret(&self, secret_name: &str) -> Result<(), SecretStorageError> {
        if !self.secrets_path.exists() {
            return Ok(());
        }

        let content = std::fs::read_to_string(&self.secrets_path)?;
        let mut secrets: serde_json::Value = serde_json::from_str(&content)?;

        secrets.as_object_mut().map(|obj| obj.remove(secret_name));

        let json = serde_json::to_string_pretty(&secrets)?;
        std::fs::write(&self.secrets_path, json)?;

        Ok(())
    }

    pub fn list_secrets(&self) -> Vec<String> {
        if !self.secrets_path.exists() {
            return Vec::new();
        }

        if let Ok(content) = std::fs::read_to_string(&self.secrets_path) {
            if let Ok(json) = serde_json::from_str::<serde_json::Value>(&content) {
                if let Some(obj) = json.as_object() {
                    return obj.keys().cloned().collect();
                }
            }
        }
        Vec::new()
    }

    pub fn secrets_path(&self) -> &PathBuf {
        &self.secrets_path
    }

    pub fn has_secret(&self, secret_name: &str) -> bool {
        self.get_secret(secret_name).is_ok()
    }
}

impl Default for SecretStorage {
    fn default() -> Self {
        Self::new()
    }
}

pub fn resolve_keychain_secret(secret_name: &str) -> Option<String> {
    let storage = SecretStorage::new();
    storage.get_secret(secret_name).ok()
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    fn test_storage(tmp: &TempDir) -> SecretStorage {
        SecretStorage::with_path(tmp.path().join("secrets.json"))
    }

    #[test]
    fn test_secret_storage_creation() {
        let storage = SecretStorage::new();
        assert!(storage.secrets_path.to_string_lossy().contains("opencode"));
    }

    #[test]
    fn test_set_and_get_secret() {
        let tmp = TempDir::new().unwrap();
        let storage = test_storage(&tmp);

        storage.set_secret("test-secret", "test-value").unwrap();

        let retrieved = storage.get_secret("test-secret").unwrap();
        assert_eq!(retrieved, "test-value");
    }

    #[test]
    fn test_delete_secret() {
        let tmp = TempDir::new().unwrap();
        let storage = test_storage(&tmp);

        storage.set_secret("delete-test", "value").unwrap();
        storage.delete_secret("delete-test").unwrap();

        let result = storage.get_secret("delete-test");
        assert!(result.is_err());
    }

    #[test]
    fn test_secret_not_found() {
        let tmp = TempDir::new().unwrap();
        let storage = test_storage(&tmp);

        let result = storage.get_secret("nonexistent-secret");
        assert!(result.is_err());
    }

    #[test]
    fn test_list_secrets() {
        let tmp = TempDir::new().unwrap();
        let storage = test_storage(&tmp);

        storage.set_secret("secret1", "value1").unwrap();
        storage.set_secret("secret2", "value2").unwrap();

        let secrets = storage.list_secrets();
        assert_eq!(secrets.len(), 2);
        assert!(secrets.contains(&"secret1".to_string()));
        assert!(secrets.contains(&"secret2".to_string()));
    }

    #[test]
    fn test_has_secret() {
        let tmp = TempDir::new().unwrap();
        let storage = test_storage(&tmp);

        assert!(!storage.has_secret("test"));

        storage.set_secret("test", "value").unwrap();

        assert!(storage.has_secret("test"));
    }

    #[test]
    fn test_secrets_file_permissions() {
        let tmp = TempDir::new().unwrap();
        let storage = test_storage(&tmp);

        storage.set_secret("test-secret", "test-value").unwrap();

        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let metadata = std::fs::metadata(storage.secrets_path()).unwrap();
            let permissions = metadata.permissions();
            let mode = permissions.mode();
            assert_eq!(
                mode & 0o777,
                0o600,
                "Secrets file should have 0o600 permissions"
            );
        }
    }
}
