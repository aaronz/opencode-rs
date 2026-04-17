use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine as _};
use opencode_core::{InstallationManager, OpenCodeError};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::fs;
use std::io::{Read, Write};
use std::net::TcpListener;
use std::path::PathBuf;
use uuid::Uuid;

const GOOGLE_OAUTH_SESSION_FILE: &str = "google-oauth-session.json";
const GOOGLE_TOKEN_ENDPOINT: &str = "https://oauth2.googleapis.com/token";
const GOOGLE_AUTH_ENDPOINT: &str = "https://accounts.google.com/o/oauth2/v2/auth";
const GOOGLE_CLIENT_ID: &str = "YOUR_GOOGLE_CLIENT_ID";
const GOOGLE_SCOPES: &str = "https://www.googleapis.com/auth/generative-language.retriever";

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct GoogleOAuthSession {
    pub access_token: String,
    pub refresh_token: Option<String>,
    pub expires_at_epoch_ms: i64,
    pub email: Option<String>,
}

impl GoogleOAuthSession {
    pub fn is_expired(&self) -> bool {
        chrono::Utc::now().timestamp_millis() >= self.expires_at_epoch_ms
    }
}

#[derive(Debug, Clone)]
pub struct GoogleOAuthStore {
    file_path: PathBuf,
}

impl GoogleOAuthStore {
    pub fn new(data_dir: PathBuf) -> Self {
        Self {
            file_path: data_dir.join(GOOGLE_OAUTH_SESSION_FILE),
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

    pub fn file_path(&self) -> &PathBuf {
        &self.file_path
    }

    pub fn load(&self) -> Result<Option<GoogleOAuthSession>, OpenCodeError> {
        if !self.file_path.exists() {
            return Ok(None);
        }
        let content = fs::read_to_string(&self.file_path)?;
        let session = serde_json::from_str(&content)?;
        Ok(Some(session))
    }

    pub fn save(&self, session: &GoogleOAuthSession) -> Result<(), OpenCodeError> {
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
pub struct GoogleOAuthRequest {
    pub redirect_uri: String,
    pub state: String,
    pub code_verifier: String,
}

#[derive(Debug, Clone)]
pub struct GoogleOAuthCallback {
    pub code: String,
    pub state: String,
}

pub struct GoogleLocalCallbackServer {
    listener: TcpListener,
    request: GoogleOAuthRequest,
}

impl GoogleLocalCallbackServer {
    pub fn request(&self) -> GoogleOAuthRequest {
        self.request.clone()
    }

    pub fn wait_for_callback(&self) -> Result<GoogleOAuthCallback, OpenCodeError> {
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

        Ok(GoogleOAuthCallback { code, state })
    }
}

#[derive(Deserialize)]
#[allow(dead_code)]
struct GoogleTokenResponse {
    access_token: String,
    refresh_token: Option<String>,
    expires_in: Option<i64>,
    #[allow(dead_code)]
    token_type: Option<String>,
}

pub struct GoogleOAuthService {
    client: reqwest::blocking::Client,
}

impl Default for GoogleOAuthService {
    fn default() -> Self {
        Self::new()
    }
}

impl GoogleOAuthService {
    pub fn new() -> Self {
        Self {
            client: reqwest::blocking::Client::new(),
        }
    }

    pub fn start_local_callback_listener(
        &self,
    ) -> Result<GoogleLocalCallbackServer, OpenCodeError> {
        let listener = TcpListener::bind("127.0.0.1:0")?;
        let port = listener.local_addr()?.port();
        Ok(GoogleLocalCallbackServer {
            listener,
            request: GoogleOAuthRequest {
                redirect_uri: format!("http://127.0.0.1:{}/auth/callback", port),
                state: Uuid::new_v4().simple().to_string(),
                code_verifier: generate_verifier(),
            },
        })
    }

    pub fn build_authorize_url(&self, request: &GoogleOAuthRequest) -> String {
        #[expect(clippy::expect_used)]
        let url = reqwest::Url::parse_with_params(
            GOOGLE_AUTH_ENDPOINT,
            &[
                ("response_type", "code"),
                ("client_id", GOOGLE_CLIENT_ID),
                ("redirect_uri", request.redirect_uri.as_str()),
                ("scope", GOOGLE_SCOPES),
                (
                    "code_challenge",
                    generate_challenge(&request.code_verifier).as_str(),
                ),
                ("code_challenge_method", "S256"),
                ("state", request.state.as_str()),
                ("access_type", "offline"),
                ("prompt", "consent"),
            ],
        )
        .expect("valid authorize url");
        url.to_string()
    }

    pub fn exchange_code(
        &self,
        callback: GoogleOAuthCallback,
        request: &GoogleOAuthRequest,
    ) -> Result<GoogleOAuthSession, OpenCodeError> {
        if callback.state != request.state {
            return Err(OpenCodeError::Parse(
                "State mismatch in Google OAuth callback".to_string(),
            ));
        }

        let token_response = self
            .client
            .post(GOOGLE_TOKEN_ENDPOINT)
            .header("Content-Type", "application/x-www-form-urlencoded")
            .form(&[
                ("grant_type", "authorization_code"),
                ("code", callback.code.as_str()),
                ("redirect_uri", request.redirect_uri.as_str()),
                ("client_id", GOOGLE_CLIENT_ID),
                ("code_verifier", request.code_verifier.as_str()),
            ])
            .send()
            .map_err(|e| {
                OpenCodeError::Network(format!("Failed to exchange Google OAuth code: {}", e))
            })?;

        if !token_response.status().is_success() {
            let status = token_response.status();
            let body = token_response.text().unwrap_or_default();
            return Err(OpenCodeError::Network(format!(
                "Google OAuth token exchange failed {}: {}",
                status, body
            )));
        }

        let tokens: GoogleTokenResponse = token_response.json().map_err(|e| {
            OpenCodeError::Network(format!(
                "Failed to decode Google OAuth token response: {}",
                e
            ))
        })?;

        Ok(GoogleOAuthSession {
            access_token: tokens.access_token,
            refresh_token: tokens.refresh_token,
            expires_at_epoch_ms: chrono::Utc::now().timestamp_millis()
                + tokens.expires_in.unwrap_or(3600) * 1000,
            email: None,
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_google_oauth_session_is_expired() {
        let session = GoogleOAuthSession {
            access_token: "access".into(),
            refresh_token: Some("refresh".into()),
            expires_at_epoch_ms: chrono::Utc::now().timestamp_millis() - 1000,
            email: None,
        };
        assert!(session.is_expired());
    }

    #[test]
    fn test_google_oauth_session_is_not_expired() {
        let session = GoogleOAuthSession {
            access_token: "access".into(),
            refresh_token: Some("refresh".into()),
            expires_at_epoch_ms: chrono::Utc::now().timestamp_millis() + 100000,
            email: None,
        };
        assert!(!session.is_expired());
    }

    #[test]
    fn test_google_oauth_store_load_no_file() {
        let dir = tempfile::tempdir().unwrap();
        let store = GoogleOAuthStore::new(dir.path().join("nonexistent.json"));
        let result = store.load();
        assert!(result.is_ok());
        assert!(result.unwrap().is_none());
    }

    #[test]
    fn test_google_oauth_store_round_trips_to_disk() {
        let dir = tempfile::tempdir().unwrap();
        let store = GoogleOAuthStore::new(dir.path().to_path_buf());
        let session = GoogleOAuthSession {
            access_token: "access".into(),
            refresh_token: Some("refresh".into()),
            expires_at_epoch_ms: 123456,
            email: Some("test@gmail.com".into()),
        };

        store.save(&session).unwrap();
        let loaded = store.load().unwrap().unwrap();
        assert_eq!(loaded, session);
    }

    #[test]
    fn test_google_oauth_store_clear() {
        let dir = tempfile::tempdir().unwrap();
        let store = GoogleOAuthStore::new(dir.path().to_path_buf());
        let session = GoogleOAuthSession {
            access_token: "access".into(),
            refresh_token: Some("refresh".into()),
            expires_at_epoch_ms: 123456,
            email: None,
        };
        store.save(&session).unwrap();
        store.clear().unwrap();
        assert!(!store.file_path().exists());
    }

    #[test]
    fn test_google_oauth_service_exchange_code_state_mismatch() {
        let service = GoogleOAuthService::new();
        let callback = GoogleOAuthCallback {
            code: "code".to_string(),
            state: "wrong-state".to_string(),
        };
        let request = GoogleOAuthRequest {
            redirect_uri: "http://127.0.0.1:8080/auth/callback".to_string(),
            state: "correct-state".to_string(),
            code_verifier: "verifier".to_string(),
        };
        let result = service.exchange_code(callback, &request);
        assert!(result.is_err());
    }

    #[test]
    fn test_authorize_url_contains_pkce_and_state() {
        let request = GoogleOAuthRequest {
            redirect_uri: "http://127.0.0.1:1455/auth/callback".into(),
            state: "state-123".into(),
            code_verifier: "verifier-123".into(),
        };

        let service = GoogleOAuthService::new();
        let url = service.build_authorize_url(&request);

        assert!(url.contains("client_id="));
        assert!(url.contains("code_challenge_method=S256"));
        assert!(url.contains("state=state-123"));
        assert!(url.contains("scope="));
    }

    #[test]
    fn test_generate_verifier_length() {
        let verifier = generate_verifier();
        assert_eq!(verifier.len(), 86);
    }

    #[test]
    fn test_generate_challenge_length() {
        let challenge = generate_challenge("test-verifier");
        assert!(!challenge.is_empty());
    }

    #[test]
    fn test_google_local_callback_server_request() {
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let port = listener.local_addr().unwrap().port();

        let server = GoogleLocalCallbackServer {
            listener,
            request: GoogleOAuthRequest {
                redirect_uri: format!("http://127.0.0.1:{}/auth/callback", port),
                state: "test-state".to_string(),
                code_verifier: "test-verifier".to_string(),
            },
        };

        std::thread::spawn(move || {
            std::thread::sleep(std::time::Duration::from_millis(100));
        });

        drop(server);
    }
}
