//! Filesystem module for OpenCode
//!
//! Provides async filesystem operations including atomic writes,
//! JSON read/write, and directory creation.

use crate::error::NamedError;
use std::path::Path;

/// Read file contents to string asynchronously.
pub async fn read_to_string(path: &Path) -> Result<String, std::io::Error> {
    tokio::fs::read_to_string(path).await
}

/// Write string to file asynchronously, creating parent dirs if needed.
pub async fn write(path: &Path, contents: &str) -> Result<(), std::io::Error> {
    if let Some(parent) = path.parent() {
        tokio::fs::create_dir_all(parent).await?;
    }
    tokio::fs::write(path, contents).await
}

/// Atomically write to file using temp file + rename pattern.
pub async fn atomic_write(path: &Path, contents: &str) -> Result<(), std::io::Error> {
    let tmp_path = path.with_extension("tmp");

    if let Some(parent) = path.parent() {
        tokio::fs::create_dir_all(parent).await?;
    }
    tokio::fs::write(&tmp_path, contents).await?;

    tokio::fs::rename(&tmp_path, path).await
}

/// Ensure directory exists, creating it and parent directories if needed.
pub async fn ensure_dir(path: &Path) -> Result<(), std::io::Error> {
    tokio::fs::create_dir_all(path).await
}

/// Read and deserialize JSON file asynchronously.
pub async fn read_json<T: serde::de::DeserializeOwned>(path: &Path) -> Result<T, NamedError> {
    let contents = tokio::fs::read_to_string(path)
        .await
        .map_err(|e| NamedError::new("IOError", e.to_string()))?;

    serde_json::from_str(&contents).map_err(|e| NamedError::new("JsonError", e.to_string()))
}

/// Serialize and write JSON file with pretty printing asynchronously.
pub async fn write_json<T: serde::Serialize>(path: &Path, value: &T) -> Result<(), NamedError> {
    let json = serde_json::to_string_pretty(value)
        .map_err(|e| NamedError::new("JsonError", e.to_string()))?;
    atomic_write(path, &json)
        .await
        .map_err(|e| NamedError::new("IOError", e.to_string()))
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[tokio::test]
    async fn test_atomic_write() {
        let temp = TempDir::new().unwrap();
        let path = temp.path().join("test.txt");

        atomic_write(&path, "hello world").await.unwrap();
        let contents = tokio::fs::read_to_string(&path).await.unwrap();
        assert_eq!(contents, "hello world");
    }

    #[tokio::test]
    async fn test_read_json() {
        let temp = TempDir::new().unwrap();
        let path = temp.path().join("config.json");

        let data = serde_json::json!({"name": "test", "version": 1});
        tokio::fs::write(&path, data.to_string()).await.unwrap();

        let decoded: serde_json::Value = read_json(&path).await.unwrap();
        assert_eq!(decoded["name"], "test");
    }

    #[tokio::test]
    async fn test_write_json() {
        let temp = TempDir::new().unwrap();
        let path = temp.path().join("config.json");

        let data = serde_json::json!({"name": "test", "version": 1});
        write_json(&path, &data).await.unwrap();

        let contents = tokio::fs::read_to_string(&path).await.unwrap();
        assert!(contents.contains('\n'));
        let decoded: serde_json::Value = serde_json::from_str(&contents).unwrap();
        assert_eq!(decoded["name"], "test");
    }

    #[tokio::test]
    async fn test_ensure_dir() {
        let temp = TempDir::new().unwrap();
        let path = temp.path().join("nested").join("deep").join("dir");

        ensure_dir(&path).await.unwrap();
        assert!(tokio::fs::metadata(&path).await.is_ok());
    }
}
