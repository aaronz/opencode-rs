use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use thiserror::Error;

const MCP_AUTH_FILE_NAME: &str = "mcp-auth.json";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpOAuthToken {
    pub server_name: String,
    pub access_token: String,
    pub refresh_token: Option<String>,
    pub token_type: String,
    pub expires_in: u64,
    pub scope: Option<String>,
    pub received_at: chrono::DateTime<chrono::Utc>,
}

impl McpOAuthToken {
    pub fn is_expired(&self) -> bool {
        let now = chrono::Utc::now();
        now >= self.received_at + chrono::Duration::seconds(self.expires_in as i64)
    }

    pub fn expires_soon(&self, threshold_secs: i64) -> bool {
        let now = chrono::Utc::now();
        let threshold = chrono::Duration::seconds(threshold_secs);
        now + threshold >= self.received_at + chrono::Duration::seconds(self.expires_in as i64)
    }
}

pub struct McpAuthTokenStore {
    file_path: PathBuf,
    tokens: HashMap<String, McpOAuthToken>,
}

impl McpAuthTokenStore {
    pub fn new(data_dir: PathBuf) -> Self {
        Self {
            file_path: data_dir.join(MCP_AUTH_FILE_NAME),
            tokens: HashMap::new(),
        }
    }

    pub fn default_path() -> PathBuf {
        std::env::var("OPENCODE_DATA_DIR")
            .map(PathBuf::from)
            .unwrap_or_else(|_| {
                dirs::home_dir()
                    .map(|h| h.join(".local/share/opencode"))
                    .unwrap_or_else(|| PathBuf::from(".opencode"))
            })
    }

    pub fn from_default_location() -> Self {
        Self::new(Self::default_path())
    }

    pub fn load(&mut self) -> Result<(), McpAuthError> {
        if !self.file_path.exists() {
            self.tokens.clear();
            return Ok(());
        }

        let content = fs::read_to_string(&self.file_path)?;
        self.tokens = serde_json::from_str(&content)?;
        Ok(())
    }

    pub fn save(&self) -> Result<(), McpAuthError> {
        if let Some(parent) = self.file_path.parent() {
            fs::create_dir_all(parent)?;
        }

        let json = serde_json::to_string_pretty(&self.tokens)?;
        fs::write(&self.file_path, json)?;

        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let _ = fs::set_permissions(&self.file_path, std::fs::Permissions::from_mode(0o600));
        }

        Ok(())
    }

    pub fn store_token(&mut self, token: McpOAuthToken) -> Result<(), McpAuthError> {
        self.tokens.insert(token.server_name.clone(), token);
        self.save()
    }

    pub fn get_token(&self, server_name: &str) -> Option<&McpOAuthToken> {
        self.tokens.get(server_name)
    }

    pub fn remove_token(&mut self, server_name: &str) -> Result<(), McpAuthError> {
        self.tokens.remove(server_name);
        self.save()
    }

    pub fn list_tokens(&self) -> Vec<&McpOAuthToken> {
        self.tokens.values().collect()
    }

    pub fn is_server_oauth_enabled(&self, server_name: &str) -> bool {
        self.tokens.contains_key(server_name)
    }

    pub fn cleanup_expired(&mut self) -> Result<Vec<String>, McpAuthError> {
        let expired: Vec<String> = self
            .tokens
            .iter()
            .filter(|(_, token)| token.is_expired())
            .map(|(name, _)| name.clone())
            .collect();

        for name in &expired {
            self.tokens.remove(name);
        }

        if !expired.is_empty() {
            self.save()?;
        }

        Ok(expired)
    }

    pub fn file_path(&self) -> &Path {
        &self.file_path
    }
}

#[derive(Debug, Error)]
pub enum McpAuthError {
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),
    #[error("json error: {0}")]
    Json(#[from] serde_json::Error),
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_store(tmp: &tempfile::TempDir) -> McpAuthTokenStore {
        let mut store = McpAuthTokenStore::new(tmp.path().to_path_buf());
        store.load().unwrap();
        store
    }

    #[test]
    fn test_store_and_retrieve_token() {
        let tmp = tempfile::tempdir().unwrap();
        let mut store = test_store(&tmp);

        let token = McpOAuthToken {
            server_name: "github".to_string(),
            access_token: "gho_abc123".to_string(),
            refresh_token: Some("ghr_refresh".to_string()),
            token_type: "Bearer".to_string(),
            expires_in: 3600,
            scope: Some("repo".to_string()),
            received_at: chrono::Utc::now(),
        };

        store.store_token(token).unwrap();
        let retrieved = store.get_token("github").unwrap();
        assert_eq!(retrieved.access_token, "gho_abc123");
        assert_eq!(retrieved.server_name, "github");
    }

    #[test]
    fn test_token_expiration() {
        let token = McpOAuthToken {
            server_name: "test".to_string(),
            access_token: "token".to_string(),
            refresh_token: None,
            token_type: "Bearer".to_string(),
            expires_in: 1,
            scope: None,
            received_at: chrono::Utc::now() - chrono::Duration::seconds(10),
        };

        assert!(token.is_expired());
        assert!(token.expires_soon(3600));
    }

    #[test]
    fn test_token_not_expired() {
        let token = McpOAuthToken {
            server_name: "test".to_string(),
            access_token: "token".to_string(),
            refresh_token: None,
            token_type: "Bearer".to_string(),
            expires_in: 3600,
            scope: None,
            received_at: chrono::Utc::now(),
        };

        assert!(!token.is_expired());
        assert!(!token.expires_soon(60));
    }

    #[test]
    fn test_cleanup_expired_tokens() {
        let tmp = tempfile::tempdir().unwrap();
        let mut store = test_store(&tmp);

        let expired_token = McpOAuthToken {
            server_name: "expired".to_string(),
            access_token: "old".to_string(),
            refresh_token: None,
            token_type: "Bearer".to_string(),
            expires_in: 1,
            scope: None,
            received_at: chrono::Utc::now() - chrono::Duration::seconds(10),
        };

        let valid_token = McpOAuthToken {
            server_name: "valid".to_string(),
            access_token: "new".to_string(),
            refresh_token: None,
            token_type: "Bearer".to_string(),
            expires_in: 3600,
            scope: None,
            received_at: chrono::Utc::now(),
        };

        store.store_token(expired_token).unwrap();
        store.store_token(valid_token).unwrap();

        let cleaned = store.cleanup_expired().unwrap();
        assert_eq!(cleaned.len(), 1);
        assert_eq!(cleaned[0], "expired");

        assert!(store.get_token("expired").is_none());
        assert!(store.get_token("valid").is_some());
    }

    #[test]
    fn test_remove_token() {
        let tmp = tempfile::tempdir().unwrap();
        let mut store = test_store(&tmp);

        let token = McpOAuthToken {
            server_name: "github".to_string(),
            access_token: "token".to_string(),
            refresh_token: None,
            token_type: "Bearer".to_string(),
            expires_in: 3600,
            scope: None,
            received_at: chrono::Utc::now(),
        };

        store.store_token(token).unwrap();
        assert!(store.get_token("github").is_some());

        store.remove_token("github").unwrap();
        assert!(store.get_token("github").is_none());
    }

    #[test]
    fn test_file_is_separate_from_auth_json() {
        let tmp = tempfile::tempdir().unwrap();
        let store = McpAuthTokenStore::new(tmp.path().to_path_buf());
        assert_eq!(store.file_path().file_name().unwrap(), "mcp-auth.json");
    }
}
