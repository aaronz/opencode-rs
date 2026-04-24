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
}

impl Default for SecretStorage {
    fn default() -> Self {
        Self::new()
    }
}

pub fn resolve_keychain_secret(secret_name: &str) -> Option<String> {
    let secrets_path = if let Ok(data_dir) = std::env::var("OPENCODE_DATA_DIR") {
        std::path::PathBuf::from(data_dir).join(SECRET_FILE_NAME)
    } else {
        SecretStorage::new().secrets_path
    };
    let storage = SecretStorage::with_path(secrets_path);
    storage.get_secret(secret_name).ok()
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_secret_not_found() {
        let tmp = TempDir::new().unwrap();
        let secrets_path = tmp.path().join("secrets.json");
        let storage = SecretStorage::with_path(secrets_path);

        let result = storage.get_secret("nonexistent-secret");
        assert!(result.is_err());
    }

    #[test]
    fn test_get_secret_success() {
        let tmp = TempDir::new().unwrap();
        let secrets_path = tmp.path().join("secrets.json");
        let secrets_content = r#"{"my-secret": "secret-value"}"#;
        std::fs::write(&secrets_path, secrets_content).unwrap();
        let storage = SecretStorage::with_path(secrets_path);

        let result = storage.get_secret("my-secret");
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "secret-value");
    }

    #[test]
    fn test_get_secret_invalid_json() {
        let tmp = TempDir::new().unwrap();
        let secrets_path = tmp.path().join("secrets.json");
        std::fs::write(&secrets_path, "not valid json").unwrap();
        let storage = SecretStorage::with_path(secrets_path);

        let result = storage.get_secret("my-secret");
        assert!(result.is_err());
    }

    #[test]
    fn test_get_secret_not_string_value() {
        let tmp = TempDir::new().unwrap();
        let secrets_path = tmp.path().join("secrets.json");
        let secrets_content = r#"{"my-secret": 12345}"#;
        std::fs::write(&secrets_path, secrets_content).unwrap();
        let storage = SecretStorage::with_path(secrets_path);

        let result = storage.get_secret("my-secret");
        assert!(result.is_err());
    }

    #[test]
    fn test_resolve_keychain_secret() {
        let tmp = TempDir::new().unwrap();
        let secrets_path = tmp.path().join("secrets.json");
        let secrets_content = r#"{"test-key": "test-value"}"#;
        std::fs::write(&secrets_path, secrets_content).unwrap();

        let storage = SecretStorage::with_path(secrets_path);
        let result = storage.get_secret("test-key");
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "test-value");
    }

    #[test]
    fn test_resolve_keychain_secret_not_found() {
        let tmp = TempDir::new().unwrap();
        std::env::set_var("OPENCODE_DATA_DIR", tmp.path().to_str().unwrap());
        let secrets_path = tmp.path().join("secrets.json");
        std::fs::write(&secrets_path, "{}").unwrap();

        let result = resolve_keychain_secret("nonexistent");
        std::env::remove_var("OPENCODE_DATA_DIR");
        assert!(result.is_none());
    }

    #[test]
    fn test_secret_storage_error_display() {
        let err = SecretStorageError::NotFound("test-secret".to_string());
        assert!(err.to_string().contains("test-secret"));
    }

    #[test]
    fn test_secret_storage_new() {
        let storage = SecretStorage::new();
        assert!(storage
            .secrets_path
            .to_string_lossy()
            .contains("secrets.json"));
    }

    #[test]
    fn test_default_secrets_path_with_data_dir() {
        let tmp = TempDir::new().unwrap();
        std::env::set_var("OPENCODE_DATA_DIR", tmp.path().to_str().unwrap());
        let path = SecretStorage::default_secrets_path();
        std::env::remove_var("OPENCODE_DATA_DIR");
        assert!(path.to_string_lossy().contains("secrets.json"));
    }

    #[test]
    fn test_default_secrets_path_fallback() {
        std::env::remove_var("OPENCODE_DATA_DIR");
        std::env::remove_var("HOME");
        let path = SecretStorage::default_secrets_path();
        assert!(path.to_string_lossy().contains("secrets.json"));
    }
}
