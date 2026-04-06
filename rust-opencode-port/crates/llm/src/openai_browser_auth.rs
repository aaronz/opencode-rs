use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine as _};
use opencode_core::{InstallationManager, OpenCodeError};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::fs;
use std::io::{Read, Write};
use std::net::TcpListener;
use std::path::{Path, PathBuf};
use uuid::Uuid;

const SESSION_FILE_NAME: &str = "openai-browser-auth.json";
const OPENAI_AUTH_ISSUER: &str = "https://auth.openai.com";
const OPENAI_CODEX_CLIENT_ID: &str = "app_EMoamEEZ73f0CkXaXp7hrann";

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct OpenAiBrowserSession {
    pub access_token: String,
    pub refresh_token: String,
    pub expires_at_epoch_ms: i64,
    pub account_id: Option<String>,
}

impl OpenAiBrowserSession {
    pub fn is_expired(&self) -> bool {
        chrono::Utc::now().timestamp_millis() >= self.expires_at_epoch_ms
    }
}

#[derive(Debug, Clone)]
pub struct OpenAiBrowserAuthStore {
    file_path: PathBuf,
}

impl OpenAiBrowserAuthStore {
    pub fn new(data_dir: PathBuf) -> Self {
        Self {
            file_path: data_dir.join(SESSION_FILE_NAME),
        }
    }

    pub fn default_path() -> PathBuf {
        if let Ok(data_dir) = std::env::var("OPENCODE_DATA_DIR") {
            return PathBuf::from(data_dir);
        }
        InstallationManager::new().info().data_path.clone()
    }

    pub fn from_default_location() -> Self {
        Self::new(Self::default_path())
    }

    pub fn file_path(&self) -> &Path {
        &self.file_path
    }

    pub fn load(&self) -> Result<Option<OpenAiBrowserSession>, OpenCodeError> {
        if !self.file_path.exists() {
            return Ok(None);
        }
        let content = fs::read_to_string(&self.file_path)?;
        let session = serde_json::from_str(&content)?;
        Ok(Some(session))
    }

    pub fn save(&self, session: &OpenAiBrowserSession) -> Result<(), OpenCodeError> {
        if let Some(parent) = self.file_path.parent() {
            fs::create_dir_all(parent)?;
        }
        let content = serde_json::to_string_pretty(session)?;
        fs::write(&self.file_path, content)?;
        Ok(())
    }

    pub fn clear(&self) -> Result<(), OpenCodeError> {
        if self.file_path.exists() {
            fs::remove_file(&self.file_path)?;
        }
        Ok(())
    }
}

#[derive(Debug, Clone)]
pub struct OpenAiBrowserAuthRequest {
    pub redirect_uri: String,
    pub state: String,
    pub code_verifier: String,
}

#[derive(Debug, Clone)]
pub struct OpenAiBrowserCallback {
    pub code: String,
    pub state: String,
}

pub struct LocalCallbackServer {
    listener: TcpListener,
    request: OpenAiBrowserAuthRequest,
}

impl LocalCallbackServer {
    pub fn request(&self) -> OpenAiBrowserAuthRequest {
        self.request.clone()
    }

    pub fn wait_for_callback(&self) -> Result<OpenAiBrowserCallback, OpenCodeError> {
        let (mut stream, _) = self.listener.accept()?;
        let mut buffer = [0_u8; 8192];
        let size = stream.read(&mut buffer)?;
        let request = String::from_utf8_lossy(&buffer[..size]);
        let path = request
            .lines()
            .next()
            .and_then(|line| line.split_whitespace().nth(1))
            .ok_or_else(|| OpenCodeError::Parse("Invalid OAuth callback request".to_string()))?;

        let url = reqwest::Url::parse(&format!(
            "http://127.0.0.1:{}{}",
            self.listener.local_addr()?.port(),
            path
        ))
        .map_err(|e| OpenCodeError::Parse(e.to_string()))?;

        let code = url
            .query_pairs()
            .find(|(key, _)| key == "code")
            .map(|(_, value)| value.to_string())
            .ok_or_else(|| OpenCodeError::Parse("Missing OAuth code in callback".to_string()))?;
        let state = url
            .query_pairs()
            .find(|(key, _)| key == "state")
            .map(|(_, value)| value.to_string())
            .ok_or_else(|| OpenCodeError::Parse("Missing OAuth state in callback".to_string()))?;

        let body = "<!doctype html><html><body><h1>Authorization successful</h1><p>You can close this window and return to opencode-rs.</p></body></html>";
        let response = format!(
            "HTTP/1.1 200 OK\r\nContent-Type: text/html; charset=utf-8\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
            body.len(), body
        );
        stream.write_all(response.as_bytes())?;

        Ok(OpenAiBrowserCallback { code, state })
    }
}

#[derive(Deserialize)]
struct TokenResponse {
    access_token: String,
    refresh_token: String,
    expires_in: Option<i64>,
    id_token: Option<String>,
}

pub struct OpenAiBrowserAuthService {
    client: reqwest::blocking::Client,
}

impl Default for OpenAiBrowserAuthService {
    fn default() -> Self {
        Self::new()
    }
}

impl OpenAiBrowserAuthService {
    pub fn new() -> Self {
        Self {
            client: reqwest::blocking::Client::new(),
        }
    }

    pub fn start_local_callback_listener(&self) -> Result<LocalCallbackServer, OpenCodeError> {
        let listener = TcpListener::bind("127.0.0.1:0")?;
        let port = listener.local_addr()?.port();
        Ok(LocalCallbackServer {
            listener,
            request: OpenAiBrowserAuthRequest {
                redirect_uri: format!("http://127.0.0.1:{}/auth/callback", port),
                state: Uuid::new_v4().simple().to_string(),
                code_verifier: generate_verifier(),
            },
        })
    }

    pub fn build_authorize_url(&self, request: &OpenAiBrowserAuthRequest) -> String {
        reqwest::Url::parse_with_params(
            &format!("{}/oauth/authorize", OPENAI_AUTH_ISSUER),
            &[
                ("response_type", "code"),
                ("client_id", OPENAI_CODEX_CLIENT_ID),
                ("redirect_uri", request.redirect_uri.as_str()),
                ("scope", "openid profile email offline_access"),
                (
                    "code_challenge",
                    generate_challenge(&request.code_verifier).as_str(),
                ),
                ("code_challenge_method", "S256"),
                ("id_token_add_organizations", "true"),
                ("codex_cli_simplified_flow", "true"),
                ("state", request.state.as_str()),
                ("originator", "opencode-rs"),
            ],
        )
        .expect("valid authorize url")
        .to_string()
    }

    pub fn exchange_code(
        &self,
        callback: OpenAiBrowserCallback,
        request: &OpenAiBrowserAuthRequest,
    ) -> Result<OpenAiBrowserSession, OpenCodeError> {
        if callback.state != request.state {
            return Err(OpenCodeError::Parse(
                "State mismatch in OpenAI OAuth callback".to_string(),
            ));
        }

        let token_response = self
            .client
            .post(format!("{}/oauth/token", OPENAI_AUTH_ISSUER))
            .header("Content-Type", "application/x-www-form-urlencoded")
            .form(&[
                ("grant_type", "authorization_code"),
                ("code", callback.code.as_str()),
                ("redirect_uri", request.redirect_uri.as_str()),
                ("client_id", OPENAI_CODEX_CLIENT_ID),
                ("code_verifier", request.code_verifier.as_str()),
            ])
            .send()
            .map_err(|e| {
                OpenCodeError::Network(format!("Failed to exchange OpenAI OAuth code: {}", e))
            })?;

        if !token_response.status().is_success() {
            let status = token_response.status();
            let body = token_response.text().unwrap_or_default();
            return Err(OpenCodeError::Network(format!(
                "OpenAI OAuth token exchange failed {}: {}",
                status, body
            )));
        }

        let tokens: TokenResponse = token_response.json().map_err(|e| {
            OpenCodeError::Network(format!(
                "Failed to decode OpenAI OAuth token response: {}",
                e
            ))
        })?;

        Ok(OpenAiBrowserSession {
            access_token: tokens.access_token,
            refresh_token: tokens.refresh_token,
            expires_at_epoch_ms: chrono::Utc::now().timestamp_millis()
                + tokens.expires_in.unwrap_or(3600) * 1000,
            account_id: tokens
                .id_token
                .as_deref()
                .and_then(extract_account_id_from_jwt),
        })
    }
}

fn generate_verifier() -> String {
    let raw = format!(
        "{}{}{}",
        Uuid::new_v4().simple(),
        Uuid::new_v4().simple(),
        Uuid::new_v4().simple()
    );
    raw.chars().take(86).collect()
}

fn generate_challenge(verifier: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(verifier.as_bytes());
    URL_SAFE_NO_PAD.encode(hasher.finalize())
}

pub fn extract_account_id_from_jwt(token: &str) -> Option<String> {
    let payload = token.split('.').nth(1)?;
    let decoded = URL_SAFE_NO_PAD.decode(payload).ok()?;
    let value: serde_json::Value = serde_json::from_slice(&decoded).ok()?;
    value
        .get("chatgpt_account_id")
        .and_then(|v| v.as_str())
        .map(|s| s.to_string())
        .or_else(|| {
            value
                .get("https://api.openai.com/auth")
                .and_then(|v| v.get("chatgpt_account_id"))
                .and_then(|v| v.as_str())
                .map(|s| s.to_string())
        })
        .or_else(|| {
            value
                .get("organizations")
                .and_then(|v| v.as_array())
                .and_then(|items| items.first())
                .and_then(|item| item.get("id"))
                .and_then(|v| v.as_str())
                .map(|s| s.to_string())
        })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn fake_jwt(payload: serde_json::Value) -> String {
        format!(
            "header.{}.sig",
            URL_SAFE_NO_PAD.encode(serde_json::to_vec(&payload).unwrap())
        )
    }

    #[test]
    fn authorize_url_contains_pkce_and_state() {
        let request = OpenAiBrowserAuthRequest {
            redirect_uri: "http://127.0.0.1:1455/auth/callback".into(),
            state: "state-123".into(),
            code_verifier: "verifier-123".into(),
        };

        let service = OpenAiBrowserAuthService::new();
        let url = service.build_authorize_url(&request);

        assert!(url.contains("client_id=app_EMoamEEZ73f0CkXaXp7hrann"));
        assert!(url.contains("code_challenge_method=S256"));
        assert!(url.contains("state=state-123"));
    }

    #[test]
    fn extracts_chatgpt_account_id_from_id_token() {
        let payload = serde_json::json!({
            "https://api.openai.com/auth": {
                "chatgpt_account_id": "acct_123"
            }
        });

        let token = fake_jwt(payload);
        assert_eq!(
            extract_account_id_from_jwt(&token).as_deref(),
            Some("acct_123")
        );
    }

    #[test]
    fn oauth_session_round_trips_to_disk() {
        let dir = tempfile::tempdir().unwrap();
        let store = OpenAiBrowserAuthStore::new(dir.path().to_path_buf());
        let session = OpenAiBrowserSession {
            access_token: "access".into(),
            refresh_token: "refresh".into(),
            expires_at_epoch_ms: 123456,
            account_id: Some("acct_123".into()),
        };

        store.save(&session).unwrap();
        let loaded = store.load().unwrap().unwrap();
        assert_eq!(loaded, session);
    }
}
