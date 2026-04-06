use crate::credential_store::{Credential, CredentialStore};
use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine as _};
use chrono::{DateTime, Duration, Utc};
use opencode_core::OpenCodeError;
use rand::{distributions::Alphanumeric, Rng};
use reqwest::Url;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::HashMap;
use std::fs;
use std::io::{Read, Write};
use std::net::TcpListener;
use std::path::{Path, PathBuf};
use std::sync::Mutex;
use std::thread;
use std::time::Duration as StdDuration;
use thiserror::Error;
use uuid::Uuid;

pub type AuthUrl = String;
pub type State = String;
pub type CodeVerifier = String;

const DEFAULT_EXPIRES_IN_SECS: u64 = 3600;
const OAUTH_SESSION_FILE_NAME: &str = "oauth-sessions.json";

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct OAuthToken {
    pub access_token: String,
    pub refresh_token: Option<String>,
    pub expires_in: u64,
    pub token_type: String,
    pub scope: Option<String>,
    pub received_at: DateTime<Utc>,
}

impl OAuthToken {
    pub fn is_expired(&self) -> bool {
        Utc::now() >= self.expires_at()
    }

    pub fn expires_at(&self) -> DateTime<Utc> {
        self.received_at + Duration::seconds(self.expires_in as i64)
    }

    fn from_response(response: OAuthTokenResponse) -> Self {
        Self {
            access_token: response.access_token,
            refresh_token: response.refresh_token,
            expires_in: response.expires_in.unwrap_or(DEFAULT_EXPIRES_IN_SECS),
            token_type: response.token_type,
            scope: response.scope,
            received_at: Utc::now(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct OAuthTokenResponse {
    access_token: String,
    refresh_token: Option<String>,
    expires_in: Option<u64>,
    token_type: String,
    scope: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeviceCodeResponse {
    pub device_code: String,
    pub user_code: String,
    pub verification_uri: String,
    pub verification_uri_complete: Option<String>,
    pub expires_in: u64,
    pub interval: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeviceCodeSession {
    pub provider: String,
    pub device_code: String,
    pub user_code: String,
    pub verification_uri: String,
    pub verification_uri_complete: Option<String>,
    pub expires_at: DateTime<Utc>,
    pub interval_secs: u64,
    pub created_at: DateTime<Utc>,
}

impl DeviceCodeSession {
    pub fn is_expired(&self) -> bool {
        Utc::now() >= self.expires_at
    }

    pub fn time_remaining(&self) -> Duration {
        let remaining = self.expires_at - Utc::now();
        if remaining.num_seconds() < 0 {
            Duration::zero()
        } else {
            remaining
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct OAuthSession {
    pub provider: String,
    pub state: String,
    pub code_verifier: String,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone)]
pub struct OAuthSessionManager {
    file_path: PathBuf,
}

impl OAuthSessionManager {
    pub fn new(data_dir: PathBuf) -> Self {
        Self {
            file_path: data_dir.join(OAUTH_SESSION_FILE_NAME),
        }
    }

    pub fn default_path() -> PathBuf {
        std::env::var("OPENCODE_DATA_DIR")
            .map(PathBuf::from)
            .unwrap_or_else(|_| {
                dirs::home_dir()
                    .map(|h| h.join(".config/opencode-rs"))
                    .unwrap_or_else(|| PathBuf::from(".opencode-rs"))
            })
    }

    pub fn from_default_location() -> Self {
        Self::new(Self::default_path())
    }

    pub fn file_path(&self) -> &Path {
        &self.file_path
    }

    pub fn load_active_sessions(&self) -> Result<HashMap<String, OAuthSession>, OAuthError> {
        if !self.file_path.exists() {
            return Ok(HashMap::new());
        }

        let content = fs::read_to_string(&self.file_path)?;
        let sessions = serde_json::from_str(&content)?;
        Ok(sessions)
    }

    pub fn save_active_sessions(
        &self,
        sessions: &HashMap<String, OAuthSession>,
    ) -> Result<(), OAuthError> {
        if let Some(parent) = self.file_path.parent() {
            fs::create_dir_all(parent)?;
        }

        let body = serde_json::to_string_pretty(sessions)?;
        fs::write(&self.file_path, body)?;

        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let _ = fs::set_permissions(&self.file_path, std::fs::Permissions::from_mode(0o600));
        }

        Ok(())
    }

    pub fn save_session(&self, session: OAuthSession) -> Result<(), OAuthError> {
        let mut sessions = self.load_active_sessions()?;
        sessions.insert(session.provider.clone(), session);
        self.save_active_sessions(&sessions)
    }

    pub fn load_session(&self, provider: &str) -> Result<Option<OAuthSession>, OAuthError> {
        let sessions = self.load_active_sessions()?;
        let session = sessions.get(provider).cloned();

        if let Some(ref s) = session {
            let age = Utc::now() - s.created_at;
            if age > Duration::hours(24) {
                self.clear_session(provider)?;
                return Ok(None);
            }
        }

        Ok(session)
    }

    pub fn clear_session(&self, provider: &str) -> Result<(), OAuthError> {
        let mut sessions = self.load_active_sessions()?;
        sessions.remove(provider);
        self.save_active_sessions(&sessions)
    }
}

pub struct OAuthFlow {
    client: reqwest::blocking::Client,
    pending_states: Mutex<HashMap<String, String>>,
    credential_store: CredentialStore,
    session_manager: OAuthSessionManager,
}

impl OAuthFlow {
    pub fn new() -> Self {
        Self {
            client: reqwest::blocking::Client::new(),
            pending_states: Mutex::new(HashMap::new()),
            credential_store: CredentialStore::new(),
            session_manager: OAuthSessionManager::from_default_location(),
        }
    }

    pub fn with_client_and_store(
        client: reqwest::blocking::Client,
        credential_store: CredentialStore,
        session_manager: OAuthSessionManager,
    ) -> Self {
        Self {
            client,
            pending_states: Mutex::new(HashMap::new()),
            credential_store,
            session_manager,
        }
    }

    pub fn generate_code_verifier() -> String {
        rand::thread_rng()
            .sample_iter(&Alphanumeric)
            .take(86)
            .map(char::from)
            .collect()
    }

    pub fn generate_code_challenge(verifier: &str) -> String {
        let mut hasher = Sha256::new();
        hasher.update(verifier.as_bytes());
        URL_SAFE_NO_PAD.encode(hasher.finalize())
    }

    pub fn start_login(
        &self,
        provider: &str,
        client_id: &str,
        redirect_uri: &str,
    ) -> Result<(AuthUrl, State, CodeVerifier), OAuthError> {
        let state = Uuid::new_v4().simple().to_string();
        let code_verifier = Self::generate_code_verifier();
        let code_challenge = Self::generate_code_challenge(&code_verifier);
        let authorize_url = Self::provider_authorize_url(provider)?;

        let auth_url = Url::parse_with_params(
            &authorize_url,
            &[
                ("response_type", "code"),
                ("client_id", client_id),
                ("redirect_uri", redirect_uri),
                ("state", state.as_str()),
                ("code_challenge", code_challenge.as_str()),
                ("code_challenge_method", "S256"),
            ],
        )?
        .to_string();

        self.pending_states
            .lock()
            .map_err(|_| OAuthError::StatePoisoned)?
            .insert(state.clone(), code_verifier.clone());

        self.session_manager.save_session(OAuthSession {
            provider: provider.to_string(),
            state: state.clone(),
            code_verifier: code_verifier.clone(),
            created_at: Utc::now(),
        })?;

        Ok((auth_url, state, code_verifier))
    }

    pub fn complete_login(
        &self,
        auth_code: &str,
        state: &str,
        code_verifier: &str,
        client_id: &str,
        client_secret: &str,
        token_url: &str,
    ) -> Result<OAuthToken, OAuthError> {
        let expected_verifier = self
            .pending_states
            .lock()
            .map_err(|_| OAuthError::StatePoisoned)?
            .remove(state)
            .ok_or(OAuthError::InvalidState)?;

        if expected_verifier != code_verifier {
            return Err(OAuthError::InvalidCodeVerifier);
        }

        let response = self
            .client
            .post(token_url)
            .form(&[
                ("grant_type", "authorization_code"),
                ("code", auth_code),
                ("client_id", client_id),
                ("client_secret", client_secret),
                ("code_verifier", code_verifier),
            ])
            .send()?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().unwrap_or_default();
            return Err(OAuthError::TokenExchangeFailed {
                status: status.as_u16(),
                body,
            });
        }

        let token_response: OAuthTokenResponse = response.json()?;
        Ok(OAuthToken::from_response(token_response))
    }

    pub fn refresh_token(
        &self,
        refresh_token: &str,
        client_id: &str,
        client_secret: &str,
        token_url: &str,
    ) -> Result<OAuthToken, OAuthError> {
        let response = self
            .client
            .post(token_url)
            .form(&[
                ("grant_type", "refresh_token"),
                ("refresh_token", refresh_token),
                ("client_id", client_id),
                ("client_secret", client_secret),
            ])
            .send()?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().unwrap_or_default();
            return Err(OAuthError::TokenRefreshFailed {
                status: status.as_u16(),
                body,
            });
        }

        let token_response: OAuthTokenResponse = response.json()?;
        Ok(OAuthToken::from_response(token_response))
    }

    pub fn store_token(&self, provider: &str, token: &OAuthToken) -> Result<(), OAuthError> {
        let mut metadata = HashMap::new();
        if let Some(refresh_token) = &token.refresh_token {
            metadata.insert("refresh_token".to_string(), refresh_token.clone());
        }
        metadata.insert("expires_in".to_string(), token.expires_in.to_string());
        metadata.insert("token_type".to_string(), token.token_type.clone());
        if let Some(scope) = &token.scope {
            metadata.insert("scope".to_string(), scope.clone());
        }
        metadata.insert("received_at".to_string(), token.received_at.to_rfc3339());

        self.credential_store.store(
            provider,
            &Credential {
                api_key: token.access_token.clone(),
                base_url: None,
                metadata,
            },
        )?;
        Ok(())
    }

    pub fn load_token(&self, provider: &str) -> Result<Option<OAuthToken>, OAuthError> {
        let credential = self.credential_store.load(provider)?;
        let Some(credential) = credential else {
            return Ok(None);
        };

        let received_at = credential
            .metadata
            .get("received_at")
            .and_then(|value| chrono::DateTime::parse_from_rfc3339(value).ok())
            .map(|dt| dt.with_timezone(&Utc))
            .unwrap_or_else(Utc::now);

        let expires_in = credential
            .metadata
            .get("expires_in")
            .and_then(|value| value.parse::<u64>().ok())
            .unwrap_or(DEFAULT_EXPIRES_IN_SECS);

        Ok(Some(OAuthToken {
            access_token: credential.api_key,
            refresh_token: credential.metadata.get("refresh_token").cloned(),
            expires_in,
            token_type: credential
                .metadata
                .get("token_type")
                .cloned()
                .unwrap_or_else(|| "Bearer".to_string()),
            scope: credential.metadata.get("scope").cloned(),
            received_at,
        }))
    }

    pub fn load_token_for_provider(
        &self,
        provider: &str,
    ) -> Result<Option<OAuthToken>, OAuthError> {
        self.load_token(provider)
    }

    pub fn ensure_fresh_token(
        &self,
        provider: &str,
        client_id: &str,
        client_secret: &str,
        token_url: &str,
    ) -> Result<Option<OAuthToken>, OAuthError> {
        let token = self.load_token(provider)?;
        let Some(token) = token else {
            return Ok(None);
        };

        if !token.is_expired() {
            return Ok(Some(token));
        }

        let refresh = token
            .refresh_token
            .as_deref()
            .ok_or(OAuthError::MissingRefreshToken)?;
        let refreshed = self.refresh_token(refresh, client_id, client_secret, token_url)?;
        self.store_token(provider, &refreshed)?;
        Ok(Some(refreshed))
    }

    pub fn start_device_code_flow(
        &self,
        provider: &str,
        client_id: &str,
        device_code_url: &str,
        scopes: Option<&str>,
    ) -> Result<DeviceCodeSession, OAuthError> {
        let client = reqwest::blocking::Client::new();
        let params = vec![
            ("client_id", client_id.to_string()),
            ("scope", scopes.unwrap_or("").to_string()),
        ];

        let response = client
            .post(device_code_url)
            .form(&params)
            .send()
            .map_err(|e| OAuthError::DeviceCodeInitFailed(e.to_string()))?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().unwrap_or_default();
            return Err(OAuthError::DeviceCodeInitFailed(format!(
                "{}: {}",
                status, body
            )));
        }

        let device_resp: DeviceCodeResponse = response
            .json()
            .map_err(|e| OAuthError::DeviceCodeParseFailed(e.to_string()))?;

        let session = DeviceCodeSession {
            provider: provider.to_string(),
            device_code: device_resp.device_code.clone(),
            user_code: device_resp.user_code.clone(),
            verification_uri: device_resp.verification_uri.clone(),
            verification_uri_complete: device_resp.verification_uri_complete.clone(),
            expires_at: Utc::now() + Duration::seconds(device_resp.expires_in as i64),
            interval_secs: device_resp.interval.unwrap_or(5),
            created_at: Utc::now(),
        };

        self.session_manager.save_session(OAuthSession {
            provider: provider.to_string(),
            state: format!("device:{}", device_resp.device_code),
            code_verifier: String::new(),
            created_at: Utc::now(),
        })?;

        Ok(session)
    }

    pub fn poll_device_code_authorization(
        &self,
        session: &DeviceCodeSession,
        client_id: &str,
        client_secret: &str,
        token_url: &str,
        on_pending: Option<&dyn Fn(&DeviceCodeSession)>,
    ) -> Result<OAuthToken, OAuthError> {
        let client = reqwest::blocking::Client::new();

        loop {
            if session.is_expired() {
                return Err(OAuthError::DeviceCodeExpired);
            }

            if let Some(callback) = on_pending {
                callback(session);
            }

            let response = client
                .post(token_url)
                .form(&[
                    ("grant_type", "urn:ietf:params:oauth:grant-type:device_code"),
                    ("device_code", session.device_code.as_str()),
                    ("client_id", client_id),
                    ("client_secret", client_secret),
                ])
                .send()
                .map_err(|e| OAuthError::DeviceCodePollFailed(e.to_string()))?;

            if response.status().is_success() {
                let token_response: OAuthTokenResponse = response
                    .json()
                    .map_err(|e| OAuthError::DeviceCodeParseFailed(e.to_string()))?;
                return Ok(OAuthToken::from_response(token_response));
            }

            let status = response.status().as_u16();
            let body = response.text().unwrap_or_default();

            if body.contains("authorization_pending") || status == 400 {
                thread::sleep(StdDuration::from_secs(session.interval_secs));
                continue;
            }

            if body.contains("slow_down") {
                thread::sleep(StdDuration::from_secs(session.interval_secs + 5));
                continue;
            }

            if body.contains("expired_token") {
                return Err(OAuthError::DeviceCodeExpired);
            }

            return Err(OAuthError::DeviceCodePollFailed(format!(
                "{}: {}",
                status, body
            )));
        }
    }

    fn provider_authorize_url(provider: &str) -> Result<String, OAuthError> {
        if provider.starts_with("http://") || provider.starts_with("https://") {
            return Ok(provider.to_string());
        }

        let url = match provider {
            "github" => "https://github.com/login/oauth/authorize",
            "openai" => "https://auth.openai.com/oauth/authorize",
            _ => {
                return Err(OAuthError::UnknownProvider(provider.to_string()));
            }
        };
        Ok(url.to_string())
    }

    pub fn start_browser_login(
        &self,
        provider: &str,
        client_id: &str,
        redirect_port: u16,
    ) -> Result<(State, CodeVerifier), OAuthError> {
        let redirect_uri = format!("http://127.0.0.1:{}/callback", redirect_port);
        let (auth_url, state, verifier) = self.start_login(provider, client_id, &redirect_uri)?;

        if let Err(e) = open_browser(&auth_url) {
            eprintln!("Failed to open browser automatically: {}", e);
            eprintln!("Please open the following URL manually:");
            eprintln!("{}", auth_url);
        }

        Ok((state, verifier))
    }

    pub fn run_callback_server_and_wait(
        &self,
        port: u16,
        timeout_secs: u64,
    ) -> Result<(String, String), OAuthError> {
        let listener = TcpListener::bind(format!("127.0.0.1:{}", port))
            .map_err(|e| OAuthError::CallbackServerFailed(e.to_string()))?;

        listener
            .set_nonblocking(true)
            .map_err(|e| OAuthError::CallbackServerFailed(e.to_string()))?;

        let deadline = std::time::Instant::now() + StdDuration::from_secs(timeout_secs);

        loop {
            if std::time::Instant::now() >= deadline {
                return Err(OAuthError::CallbackServerFailed(
                    "Timeout waiting for callback".to_string(),
                ));
            }

            match listener.accept() {
                Ok((mut stream, _)) => {
                    let mut buffer = [0u8; 4096];
                    let n = stream
                        .read(&mut buffer)
                        .map_err(|e| OAuthError::CallbackServerFailed(e.to_string()))?;

                    let request = String::from_utf8_lossy(&buffer[..n]);
                    let (code, state) = Self::parse_callback_request(&request)?;

                    let response = "HTTP/1.1 200 OK\r\nContent-Type: text/html\r\n\r\n\
                        <html><body><h1>Authentication successful!</h1>\
                        <p>You can close this window and return to the terminal.</p></body></html>";
                    let _ = stream.write_all(response.as_bytes());
                    let _ = stream.flush();

                    return Ok((code, state));
                }
                Err(ref e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                    thread::sleep(StdDuration::from_millis(100));
                    continue;
                }
                Err(e) => return Err(OAuthError::CallbackServerFailed(e.to_string())),
            }
        }
    }

    fn parse_callback_request(request: &str) -> Result<(String, String), OAuthError> {
        let first_line = request
            .lines()
            .next()
            .ok_or(OAuthError::InvalidCallbackRequest)?;
        let parts: Vec<&str> = first_line.split_whitespace().collect();
        if parts.len() < 2 {
            return Err(OAuthError::InvalidCallbackRequest);
        }

        let query = parts[1]
            .split('?')
            .nth(1)
            .ok_or(OAuthError::InvalidCallbackRequest)?;
        let mut code = None;
        let mut state = None;

        for param in query.split('&') {
            if let Some((key, value)) = param.split_once('=') {
                let decoded = value.replace('+', " ");
                match key {
                    "code" => code = Some(decoded),
                    "state" => state = Some(decoded),
                    "error" => return Err(OAuthError::AuthorizationDenied(decoded)),
                    _ => {}
                }
            }
        }

        match (code, state) {
            (Some(c), Some(s)) => Ok((c, s)),
            _ => Err(OAuthError::InvalidCallbackRequest),
        }
    }
}

fn open_browser(url: &str) -> Result<(), String> {
    let cmd = if cfg!(target_os = "macos") {
        "open"
    } else if cfg!(target_os = "linux") {
        "xdg-open"
    } else if cfg!(target_os = "windows") {
        "start"
    } else {
        return Err(format!("Unsupported platform for browser open: {}", url));
    };

    std::process::Command::new(cmd)
        .arg(url)
        .spawn()
        .map_err(|e| format!("Failed to open browser: {}", e))?;
    Ok(())
}

fn urlencoding_decode(value: &str) -> String {
    url::form_urlencoded::parse(value.as_bytes())
        .next()
        .map(|(_, v)| v.into_owned())
        .unwrap_or_else(|| value.replace('+', " "))
}

impl Default for OAuthFlow {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Error)]
pub enum OAuthError {
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),
    #[error("json error: {0}")]
    Json(#[from] serde_json::Error),
    #[error("network error: {0}")]
    Network(#[from] reqwest::Error),
    #[error("url parse error: {0}")]
    Url(#[from] url::ParseError),
    #[error("state lock poisoned")]
    StatePoisoned,
    #[error("invalid oauth state")]
    InvalidState,
    #[error("invalid oauth code verifier")]
    InvalidCodeVerifier,
    #[error("missing refresh token")]
    MissingRefreshToken,
    #[error("unknown oauth provider: {0}")]
    UnknownProvider(String),
    #[error("token exchange failed ({status}): {body}")]
    TokenExchangeFailed { status: u16, body: String },
    #[error("token refresh failed ({status}): {body}")]
    TokenRefreshFailed { status: u16, body: String },
    #[error("callback server failed: {0}")]
    CallbackServerFailed(String),
    #[error("invalid callback request")]
    InvalidCallbackRequest,
    #[error("authorization denied: {0}")]
    AuthorizationDenied(String),
    #[error("device code init failed: {0}")]
    DeviceCodeInitFailed(String),
    #[error("device code parse failed: {0}")]
    DeviceCodeParseFailed(String),
    #[error("device code expired")]
    DeviceCodeExpired,
    #[error("device code poll failed: {0}")]
    DeviceCodePollFailed(String),
}

impl From<OpenCodeError> for OAuthError {
    fn from(value: OpenCodeError) -> Self {
        OAuthError::Io(std::io::Error::other(value.to_string()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::{Read, Write};
    use std::net::TcpListener;
    use std::sync::mpsc;
    use std::thread;

    fn test_flow(tmp: &tempfile::TempDir) -> OAuthFlow {
        let store = CredentialStore::with_paths(
            tmp.path().join("credentials.enc.json"),
            tmp.path().join("credentials.key"),
        );
        let session_manager = OAuthSessionManager::new(tmp.path().to_path_buf());
        OAuthFlow::with_client_and_store(reqwest::blocking::Client::new(), store, session_manager)
    }

    fn spawn_mock_token_server(response_body: String) -> String {
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let addr = format!("http://{}", listener.local_addr().unwrap());
        let (tx, rx) = mpsc::channel::<()>();

        thread::spawn(move || {
            tx.send(()).unwrap();
            let (mut stream, _) = listener.accept().unwrap();
            let mut buffer = [0u8; 8192];
            let _ = stream.read(&mut buffer).unwrap();

            let response = format!(
                "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
                response_body.len(),
                response_body
            );
            stream.write_all(response.as_bytes()).unwrap();
        });

        let _ = rx.recv();
        addr
    }

    #[test]
    fn generates_pkce_code_challenge() {
        let verifier = "abc123";
        let challenge = OAuthFlow::generate_code_challenge(verifier);
        assert_eq!(challenge, "bKE9UspwyIPg8LsQHkJaiehiTeUdstI5JZOvaoQRgJA");
    }

    #[test]
    fn start_login_persists_session_and_state() {
        let tmp = tempfile::tempdir().unwrap();
        let flow = test_flow(&tmp);

        let (url, state, verifier) = flow
            .start_login("github", "client-1", "http://127.0.0.1/callback")
            .unwrap();
        assert!(url.contains("client_id=client-1"));
        assert!(url.contains("code_challenge_method=S256"));
        assert!(!state.is_empty());
        assert!(!verifier.is_empty());

        let session = flow
            .session_manager
            .load_session("github")
            .unwrap()
            .unwrap();
        assert_eq!(session.state, state);
        assert_eq!(session.code_verifier, verifier);
    }

    #[test]
    fn complete_login_exchanges_code_and_validates_state() {
        let tmp = tempfile::tempdir().unwrap();
        let flow = test_flow(&tmp);
        let (_url, state, verifier) = flow
            .start_login("github", "client-1", "http://127.0.0.1/callback")
            .unwrap();

        let endpoint = spawn_mock_token_server(
            serde_json::json!({
                "access_token": "access-1",
                "refresh_token": "refresh-1",
                "expires_in": 1200,
                "token_type": "Bearer",
                "scope": "repo"
            })
            .to_string(),
        );

        let token = flow
            .complete_login(
                "auth-code",
                &state,
                &verifier,
                "client-1",
                "secret-1",
                &endpoint,
            )
            .unwrap();

        assert_eq!(token.access_token, "access-1");
        assert_eq!(token.refresh_token.as_deref(), Some("refresh-1"));
        assert_eq!(token.token_type, "Bearer");
    }

    #[test]
    fn complete_login_fails_when_state_mismatches() {
        let tmp = tempfile::tempdir().unwrap();
        let flow = test_flow(&tmp);
        let (_url, _state, verifier) = flow
            .start_login("github", "client-1", "http://127.0.0.1/callback")
            .unwrap();

        let err = flow
            .complete_login(
                "auth-code",
                "unexpected-state",
                &verifier,
                "client-1",
                "secret-1",
                "http://127.0.0.1:65534/token",
            )
            .unwrap_err();

        assert!(matches!(err, OAuthError::InvalidState));
    }

    #[test]
    fn stores_loads_and_refreshes_token() {
        let tmp = tempfile::tempdir().unwrap();
        let flow = test_flow(&tmp);

        let token = OAuthToken {
            access_token: "expired-access".into(),
            refresh_token: Some("refresh-1".into()),
            expires_in: 1,
            token_type: "Bearer".into(),
            scope: Some("repo".into()),
            received_at: Utc::now() - Duration::seconds(10),
        };
        flow.store_token("github", &token).unwrap();

        let endpoint = spawn_mock_token_server(
            serde_json::json!({
                "access_token": "refreshed-access",
                "refresh_token": "refresh-2",
                "expires_in": 3600,
                "token_type": "Bearer",
                "scope": "repo"
            })
            .to_string(),
        );

        let refreshed = flow
            .ensure_fresh_token("github", "client-1", "secret-1", &endpoint)
            .unwrap()
            .unwrap();
        assert_eq!(refreshed.access_token, "refreshed-access");
        assert_eq!(refreshed.refresh_token.as_deref(), Some("refresh-2"));
        assert!(!refreshed.is_expired());
    }

    #[test]
    fn test_session_expired_after_24_hours() {
        let tmp = tempfile::tempdir().unwrap();
        let flow = test_flow(&tmp);

        let old_session = OAuthSession {
            provider: "github".to_string(),
            state: "old-state".to_string(),
            code_verifier: "old-verifier".to_string(),
            created_at: Utc::now() - Duration::hours(25),
        };

        flow.session_manager.save_session(old_session).unwrap();

        let session = flow.session_manager.load_session("github").unwrap();
        assert!(session.is_none());
    }

    #[test]
    fn test_parse_callback_request_valid() {
        let request = "GET /callback?code=abc123&state=xyz789 HTTP/1.1\r\nHost: 127.0.0.1\r\n\r\n";
        let (code, state) = OAuthFlow::parse_callback_request(request).unwrap();
        assert_eq!(code, "abc123");
        assert_eq!(state, "xyz789");
    }

    #[test]
    fn test_parse_callback_request_with_error() {
        let request =
            "GET /callback?error=access_denied&error_description=User+denied HTTP/1.1\r\n\r\n";
        let result = OAuthFlow::parse_callback_request(request);
        assert!(matches!(result, Err(OAuthError::AuthorizationDenied(_))));
    }

    #[test]
    fn test_parse_callback_request_invalid() {
        let request = "INVALID REQUEST";
        let result = OAuthFlow::parse_callback_request(request);
        assert!(matches!(result, Err(OAuthError::InvalidCallbackRequest)));
    }

    #[test]
    fn test_device_code_session_expiration() {
        let session = DeviceCodeSession {
            provider: "github".to_string(),
            device_code: "dev-code".to_string(),
            user_code: "USER-123".to_string(),
            verification_uri: "https://github.com/login/device".to_string(),
            verification_uri_complete: Some(
                "https://github.com/login/device?code=USER-123".to_string(),
            ),
            expires_at: Utc::now() - Duration::seconds(1),
            interval_secs: 5,
            created_at: Utc::now() - Duration::minutes(10),
        };

        assert!(session.is_expired());
        assert_eq!(session.time_remaining(), Duration::zero());
    }

    #[test]
    fn test_device_code_session_not_expired() {
        let session = DeviceCodeSession {
            provider: "github".to_string(),
            device_code: "dev-code".to_string(),
            user_code: "USER-123".to_string(),
            verification_uri: "https://github.com/login/device".to_string(),
            verification_uri_complete: None,
            expires_at: Utc::now() + Duration::minutes(5),
            interval_secs: 5,
            created_at: Utc::now(),
        };

        assert!(!session.is_expired());
        assert!(session.time_remaining().num_seconds() > 0);
    }
}
