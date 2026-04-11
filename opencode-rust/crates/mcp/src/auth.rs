use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use thiserror::Error;

const MCP_AUTH_FILE_NAME: &str = "mcp-auth.json";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpOAuthConfig {
    pub client_id: String,
    pub client_secret: Option<String>,
    pub auth_url: String,
    pub token_url: String,
    pub scopes: Option<String>,
}

impl McpOAuthConfig {
    pub fn new(client_id: String, auth_url: String, token_url: String) -> Self {
        Self {
            client_id,
            client_secret: None,
            auth_url,
            token_url,
            scopes: None,
        }
    }

    pub fn with_client_secret(mut self, secret: String) -> Self {
        self.client_secret = Some(secret);
        self
    }

    pub fn with_scopes(mut self, scopes: String) -> Self {
        self.scopes = Some(scopes);
        self
    }
}

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

pub struct McpOAuthManager {
    config_store: McpAuthTokenStore,
    oauth_configs: HashMap<String, McpOAuthConfig>,
}

impl McpOAuthManager {
    pub fn new(data_dir: std::path::PathBuf) -> Self {
        let mut config_store = McpAuthTokenStore::new(data_dir);
        if let Err(e) = config_store.load() {
            tracing::warn!("Failed to load OAuth tokens: {}", e);
        }
        Self {
            config_store,
            oauth_configs: HashMap::new(),
        }
    }

    pub fn from_default_location() -> Self {
        Self::new(McpAuthTokenStore::default_path())
    }

    pub fn register_server_oauth(
        &mut self,
        server_name: &str,
        config: McpOAuthConfig,
    ) -> Result<(), McpAuthError> {
        self.oauth_configs.insert(server_name.to_string(), config);
        Ok(())
    }

    pub fn get_oauth_config(&self, server_name: &str) -> Option<&McpOAuthConfig> {
        self.oauth_configs.get(server_name)
    }

    pub fn get_token(&self, server_name: &str) -> Option<&McpOAuthToken> {
        self.config_store.get_token(server_name)
    }

    pub fn is_server_oauth_enabled(&self, server_name: &str) -> bool {
        self.oauth_configs.contains_key(server_name)
            && self.config_store.is_server_oauth_enabled(server_name)
    }

    pub fn store_token(&mut self, token: McpOAuthToken) -> Result<(), McpAuthError> {
        self.config_store.store_token(token)
    }

    pub fn remove_token(&mut self, server_name: &str) -> Result<(), McpAuthError> {
        self.oauth_configs.remove(server_name);
        self.config_store.remove_token(server_name)
    }

    pub fn get_or_refresh_token(
        &mut self,
        server_name: &str,
    ) -> Result<Option<String>, McpAuthError> {
        let config = self
            .oauth_configs
            .get(server_name)
            .ok_or_else(|| McpAuthError::OAuthNotConfigured(server_name.to_string()))?;

        let token = match self.config_store.get_token(server_name) {
            Some(t) if !t.is_expired() => t.clone(),
            Some(t) if t.expires_soon(300) => {
                if let Some(refresh_token) = &t.refresh_token {
                    let refreshed = self.refresh_token(server_name, refresh_token, config)?;
                    self.config_store.store_token(refreshed.clone())?;
                    refreshed
                } else {
                    t.clone()
                }
            }
            Some(_) | None => {
                return Ok(self
                    .config_store
                    .get_token(server_name)
                    .map(|t| t.access_token.clone()));
            }
        };

        Ok(Some(token.access_token.clone()))
    }

    fn refresh_token(
        &self,
        server_name: &str,
        refresh_token: &str,
        config: &McpOAuthConfig,
    ) -> Result<McpOAuthToken, McpAuthError> {
        let client = reqwest::blocking::Client::new();
        let params = vec![
            ("grant_type", "refresh_token"),
            ("refresh_token", refresh_token),
            ("client_id", config.client_id.as_str()),
        ];

        let mut request = client.post(&config.token_url).form(&params);
        if let Some(ref secret) = config.client_secret {
            request = request.form(&[("client_secret", secret.as_str())]);
        }

        let response = request.send().map_err(|e| McpAuthError::Network(e))?;

        let status = response.status();

        if !status.is_success() {
            let body = response.text().unwrap_or_default();
            return Err(McpAuthError::TokenRefreshFailed {
                status: status.as_u16(),
                body,
            });
        }

        #[derive(Deserialize)]
        struct TokenResponse {
            access_token: String,
            refresh_token: Option<String>,
            expires_in: Option<u64>,
            token_type: String,
        }

        let token_resp: TokenResponse = {
            let text = response.text().map_err(McpAuthError::Network)?;
            serde_json::from_str(&text).map_err(|e| McpAuthError::Json(e))?
        };

        Ok(McpOAuthToken {
            server_name: server_name.to_string(),
            access_token: token_resp.access_token,
            refresh_token: token_resp
                .refresh_token
                .or_else(|| Some(refresh_token.to_string())),
            token_type: token_resp.token_type,
            expires_in: token_resp.expires_in.unwrap_or(3600),
            scope: config.scopes.clone(),
            received_at: chrono::Utc::now(),
        })
    }

    pub fn list_registered_servers(&self) -> Vec<&str> {
        self.oauth_configs.keys().map(|s| s.as_str()).collect()
    }

    pub fn file_path(&self) -> &Path {
        self.config_store.file_path()
    }
}

#[derive(Debug, Error)]
pub enum McpAuthError {
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),
    #[error("json error: {0}")]
    Json(#[from] serde_json::Error),
    #[error("network error: {0}")]
    Network(#[from] reqwest::Error),
    #[error("oauth not configured for server: {0}")]
    OAuthNotConfigured(String),
    #[error("token refresh failed ({status}): {body}")]
    TokenRefreshFailed { status: u16, body: String },
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

    #[test]
    fn test_mcp_oauth_config_per_server() {
        let tmp = tempfile::tempdir().unwrap();
        let mut manager = McpOAuthManager::new(tmp.path().to_path_buf());

        let github_config = McpOAuthConfig::new(
            "github-client-id".to_string(),
            "https://github.com/login/oauth/authorize".to_string(),
            "https://github.com/login/oauth/access_token".to_string(),
        )
        .with_scopes("repo,gist".to_string());

        let jira_config = McpOAuthConfig::new(
            "jira-client-id".to_string(),
            "https://auth.atlassian.com/authorize".to_string(),
            "https://auth.atlassian.com/oauth/token".to_string(),
        )
        .with_scopes("read:jira-work".to_string());

        manager
            .register_server_oauth("github", github_config.clone())
            .unwrap();
        manager
            .register_server_oauth("jira", jira_config.clone())
            .unwrap();

        let retrieved_github = manager.get_oauth_config("github").unwrap();
        let retrieved_jira = manager.get_oauth_config("jira").unwrap();

        assert_eq!(retrieved_github.client_id, "github-client-id");
        assert_eq!(retrieved_github.scopes.as_deref(), Some("repo,gist"));
        assert_eq!(retrieved_jira.client_id, "jira-client-id");
        assert_eq!(retrieved_jira.scopes.as_deref(), Some("read:jira-work"));
    }

    #[test]
    fn test_mcp_oauth_different_servers_different_settings() {
        let tmp = tempfile::tempdir().unwrap();
        let mut manager = McpOAuthManager::new(tmp.path().to_path_buf());

        let config1 = McpOAuthConfig::new(
            "client1".to_string(),
            "https://auth.example.com/authorize".to_string(),
            "https://auth.example.com/token".to_string(),
        )
        .with_client_secret("secret1".to_string())
        .with_scopes("read write".to_string());

        let config2 = McpOAuthConfig::new(
            "client2".to_string(),
            "https://other.auth.example.com/authorize".to_string(),
            "https://other.auth.example.com/token".to_string(),
        )
        .with_scopes("admin".to_string());

        manager.register_server_oauth("server1", config1).unwrap();
        manager.register_server_oauth("server2", config2).unwrap();

        let s1 = manager.get_oauth_config("server1").unwrap();
        let s2 = manager.get_oauth_config("server2").unwrap();

        assert!(s1.client_secret.is_some());
        assert_eq!(s1.client_secret.as_deref(), Some("secret1"));
        assert!(s2.client_secret.is_none());

        assert_eq!(s1.scopes.as_deref(), Some("read write"));
        assert_eq!(s2.scopes.as_deref(), Some("admin"));

        assert_ne!(s1.auth_url, s2.auth_url);
    }

    #[test]
    fn test_mcp_oauth_token_storage_secure() {
        let tmp = tempfile::tempdir().unwrap();
        let mut store = McpAuthTokenStore::new(tmp.path().to_path_buf());
        store.load().unwrap();

        let token = McpOAuthToken {
            server_name: "secure-server".to_string(),
            access_token: "super_secret_token".to_string(),
            refresh_token: Some("refresh_token_123".to_string()),
            token_type: "Bearer".to_string(),
            expires_in: 3600,
            scope: Some("secure_scope".to_string()),
            received_at: chrono::Utc::now(),
        };

        store.store_token(token).unwrap();

        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let metadata = std::fs::metadata(store.file_path()).unwrap();
            let permissions = metadata.permissions();
            let mode = permissions.mode();
            assert_eq!(
                mode & 0o777,
                0o600,
                "Token file should have 0o600 permissions"
            );
        }

        let loaded = store.get_token("secure-server").unwrap();
        assert_eq!(loaded.access_token, "super_secret_token");
        assert_eq!(loaded.refresh_token.as_deref(), Some("refresh_token_123"));
    }

    #[test]
    fn test_mcp_oauth_manager_token_retrieval() {
        let tmp = tempfile::tempdir().unwrap();
        let mut manager = McpOAuthManager::new(tmp.path().to_path_buf());

        let config = McpOAuthConfig::new(
            "test-client".to_string(),
            "https://auth.example.com/authorize".to_string(),
            "https://auth.example.com/token".to_string(),
        );
        manager
            .register_server_oauth("test-server", config)
            .unwrap();

        let token = McpOAuthToken {
            server_name: "test-server".to_string(),
            access_token: "test_access_token".to_string(),
            refresh_token: Some("test_refresh_token".to_string()),
            token_type: "Bearer".to_string(),
            expires_in: 3600,
            scope: None,
            received_at: chrono::Utc::now(),
        };
        manager.store_token(token).unwrap();

        let retrieved_token = manager.get_token("test-server").unwrap();
        assert_eq!(retrieved_token.access_token, "test_access_token");
    }

    #[test]
    fn test_mcp_oauth_is_server_enabled() {
        let tmp = tempfile::tempdir().unwrap();
        let mut manager = McpOAuthManager::new(tmp.path().to_path_buf());

        assert!(!manager.is_server_oauth_enabled("github"));

        let config = McpOAuthConfig::new(
            "client".to_string(),
            "https://auth.example.com/authorize".to_string(),
            "https://auth.example.com/token".to_string(),
        );
        manager.register_server_oauth("github", config).unwrap();

        assert!(!manager.is_server_oauth_enabled("github"));

        let token = McpOAuthToken {
            server_name: "github".to_string(),
            access_token: "token".to_string(),
            refresh_token: None,
            token_type: "Bearer".to_string(),
            expires_in: 3600,
            scope: None,
            received_at: chrono::Utc::now(),
        };
        manager.store_token(token).unwrap();

        assert!(manager.is_server_oauth_enabled("github"));
    }
}
