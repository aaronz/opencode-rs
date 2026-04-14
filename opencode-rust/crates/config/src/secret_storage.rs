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

    #[cfg(test)]
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
    let storage = SecretStorage::new();
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
}
