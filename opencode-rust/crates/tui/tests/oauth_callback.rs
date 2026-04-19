use opencode_llm::{
    GoogleOAuthCallback, GoogleOAuthRequest, GoogleOAuthService, GoogleOAuthSession,
    GoogleOAuthStore,
};
use std::io::{Read, Write};
use std::net::TcpListener;
use std::thread;

struct GoogleLocalCallbackServer {
    listener: TcpListener,
}

impl GoogleLocalCallbackServer {
    fn wait_for_callback(&self) -> Result<GoogleOAuthCallback, String> {
        let (mut stream, _) = self.listener.accept().map_err(|e| e.to_string())?;
        let mut buffer = [0_u8; 8192];
        let size = stream.read(&mut buffer).map_err(|e| e.to_string())?;
        let request = String::from_utf8_lossy(&buffer[..size]);
        let path = request
            .lines()
            .next()
            .and_then(|line| line.split_whitespace().nth(1))
            .ok_or_else(|| "Invalid OAuth callback request".to_string())?;

        let port = self
            .listener
            .local_addr()
            .map_err(|e| e.to_string())?
            .port();
        let url = reqwest::Url::parse(&format!("http://127.0.0.1:{}{}", port, path))
            .map_err(|e| e.to_string())?;

        let code = url
            .query_pairs()
            .find(|(key, _)| key == "code")
            .map(|(_, value)| value.to_string())
            .ok_or_else(|| "Missing OAuth code in callback".to_string())?;
        let state = url
            .query_pairs()
            .find(|(key, _)| key == "state")
            .map(|(_, value)| value.to_string())
            .ok_or_else(|| "Missing OAuth state in callback".to_string())?;

        let body = "<!doctype html><html><body><h1>Authorization successful</h1></body></html>";
        let response = format!(
            "HTTP/1.1 200 OK\r\nContent-Type: text/html; charset=utf-8\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
            body.len(),
            body
        );
        stream
            .write_all(response.as_bytes())
            .map_err(|e| e.to_string())?;

        Ok(GoogleOAuthCallback { code, state })
    }
}

fn run_callback_server_and_get_port() -> (GoogleLocalCallbackServer, GoogleOAuthRequest, u16) {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let port = listener.local_addr().unwrap().port();
    let request = GoogleOAuthRequest {
        redirect_uri: format!("http://127.0.0.1:{}/auth/callback", port),
        state: "test-state-12345".to_string(),
        code_verifier: "test-verifier-12345678901234567890123456789012345678901234567890123456"
            .to_string(),
    };
    let server = GoogleLocalCallbackServer { listener };
    (server, request, port)
}

fn send_mock_oauth_callback(port: u16, code: &str, state: &str) -> Result<(), String> {
    let url = format!(
        "http://127.0.0.1:{}/auth/callback?code={}&state={}",
        port, code, state
    );
    let response = reqwest::blocking::get(&url).map_err(|e| e.to_string())?;
    if response.status().as_u16() != 200 {
        return Err(format!(
            "Expected status 200, got {}",
            response.status().as_u16()
        ));
    }
    Ok(())
}

#[test]
fn test_oauth_callback_is_received_correctly() {
    let (server, request, port) = run_callback_server_and_get_port();

    let handle = thread::spawn(move || server.wait_for_callback());

    send_mock_oauth_callback(port, "test-auth-code-abc123", &request.state).unwrap();

    let result = handle.join().unwrap();
    assert!(result.is_ok());
    let callback = result.unwrap();
    assert_eq!(callback.code, "test-auth-code-abc123");
    assert_eq!(callback.state, request.state);
}

#[test]
fn test_oauth_callback_extracts_code_and_state() {
    let (server, request, port) = run_callback_server_and_get_port();
    let expected_code = "my-oauth-authorization-code";

    let handle = thread::spawn(move || server.wait_for_callback());

    send_mock_oauth_callback(port, expected_code, &request.state).unwrap();

    let result = handle.join().unwrap();
    assert!(result.is_ok());
    let callback = result.unwrap();
    assert_eq!(callback.code, expected_code);
    assert_eq!(callback.state, request.state);
}

#[test]
fn test_oauth_code_exchange_state_mismatch_error() {
    let service = GoogleOAuthService::new();
    let request = GoogleOAuthRequest {
        redirect_uri: "http://127.0.0.1:9999/auth/callback".to_string(),
        state: "exchange-test-state".to_string(),
        code_verifier: "test-verifier-12345678901234567890123456789012345678901234567890123456"
            .to_string(),
    };

    let callback = GoogleOAuthCallback {
        code: "auth-code-to-exchange".to_string(),
        state: "wrong-state".to_string(),
    };

    let result = service.exchange_code(callback, &request);
    assert!(result.is_err());
}

#[test]
fn test_oauth_callback_missing_code_returns_error() {
    let (server, request, port) = run_callback_server_and_get_port();

    let handle = thread::spawn(move || server.wait_for_callback());

    let url = format!(
        "http://127.0.0.1:{}/auth/callback?state={}",
        port, request.state
    );
    let _ = reqwest::blocking::get(&url);

    let result = handle.join().unwrap();
    assert!(result.is_err());
    let err_msg = result.unwrap_err();
    assert!(err_msg.contains("code") || err_msg.contains("Missing"));
}

#[test]
fn test_oauth_callback_missing_state_returns_error() {
    let (server, _request, port) = run_callback_server_and_get_port();

    let handle = thread::spawn(move || server.wait_for_callback());

    let url = format!("http://127.0.0.1:{}/auth/callback?code=test-code", port);
    let _ = reqwest::blocking::get(&url);

    let result = handle.join().unwrap();
    assert!(result.is_err());
}

#[test]
fn test_oauth_tokens_stored_in_credential_store() {
    let dir = tempfile::tempdir().unwrap();
    let store = GoogleOAuthStore::new(dir.path().to_path_buf());

    let session = GoogleOAuthSession {
        access_token: "ya29.test-access-token".to_string(),
        refresh_token: Some("1//test-refresh-token".to_string()),
        expires_at_epoch_ms: chrono::Utc::now().timestamp_millis() + 3600000,
        email: Some("testuser@gmail.com".to_string()),
    };

    assert!(store.save(&session).is_ok());

    let loaded = store.load().unwrap();
    assert!(loaded.is_some());
    let loaded_session = loaded.unwrap();
    assert_eq!(loaded_session.access_token, "ya29.test-access-token");
    assert_eq!(
        loaded_session.refresh_token,
        Some("1//test-refresh-token".to_string())
    );
    assert_eq!(loaded_session.email, Some("testuser@gmail.com".to_string()));
}

#[test]
fn test_oauth_store_clears_session() {
    let dir = tempfile::tempdir().unwrap();
    let store = GoogleOAuthStore::new(dir.path().to_path_buf());

    let session = GoogleOAuthSession {
        access_token: "temp-token".to_string(),
        refresh_token: None,
        expires_at_epoch_ms: chrono::Utc::now().timestamp_millis() + 3600000,
        email: None,
    };

    store.save(&session).unwrap();
    assert!(store.file_path().exists());

    store.clear().unwrap();
    assert!(!store.file_path().exists());
}

#[test]
fn test_oauth_session_is_expired() {
    let expired_session = GoogleOAuthSession {
        access_token: "expired-token".to_string(),
        refresh_token: Some("refresh".to_string()),
        expires_at_epoch_ms: chrono::Utc::now().timestamp_millis() - 1000,
        email: None,
    };
    assert!(expired_session.is_expired());

    let valid_session = GoogleOAuthSession {
        access_token: "valid-token".to_string(),
        refresh_token: Some("refresh".to_string()),
        expires_at_epoch_ms: chrono::Utc::now().timestamp_millis() + 3600000,
        email: None,
    };
    assert!(!valid_session.is_expired());
}

#[test]
fn test_oauth_callback_state_mismatch_returns_error() {
    let service = GoogleOAuthService::new();
    let request = GoogleOAuthRequest {
        redirect_uri: "http://127.0.0.1:8080/auth/callback".to_string(),
        state: "correct-state".to_string(),
        code_verifier: "verifier".to_string(),
    };

    let callback = GoogleOAuthCallback {
        code: "some-code".to_string(),
        state: "wrong-state".to_string(),
    };

    let result = service.exchange_code(callback, &request);
    assert!(result.is_err());
}

#[test]
fn test_oauth_invalid_callback_server_request() {
    let service = GoogleOAuthService::new();
    let server = service.start_local_callback_listener().unwrap();
    let req = server.request();

    assert_eq!(req.state.len(), 32);
    assert!(req.redirect_uri.starts_with("http://127.0.0.1:"));
    assert!(req.redirect_uri.ends_with("/auth/callback"));
}

#[test]
fn test_oauth_callback_server_sends_success_response() {
    let (server, request, port) = run_callback_server_and_get_port();

    let handle = thread::spawn(move || server.wait_for_callback());

    let url = format!(
        "http://127.0.0.1:{}/auth/callback?code=test-code&state={}",
        port, request.state
    );
    let response = reqwest::blocking::get(&url).unwrap();

    assert_eq!(response.status().as_u16(), 200);
    let body = response.text().unwrap();
    assert!(body.contains("Authorization successful"));

    let _ = handle.join();
}

#[test]
fn test_google_oauth_service_creates_callback_listener() {
    let service = GoogleOAuthService::new();
    let result = service.start_local_callback_listener();
    assert!(result.is_ok());

    let server = result.unwrap();
    let request = server.request();
    assert!(!request.state.is_empty());
    assert!(request.redirect_uri.contains("/auth/callback"));
}

#[test]
fn test_oauth_session_persists_across_store_restart() {
    let dir = tempfile::tempdir().unwrap();
    let store_path = dir.path().to_path_buf();

    let store1 = GoogleOAuthStore::new(store_path.clone());
    let session = GoogleOAuthSession {
        access_token: "persist-token".to_string(),
        refresh_token: Some("persist-refresh".to_string()),
        expires_at_epoch_ms: chrono::Utc::now().timestamp_millis() + 7200000,
        email: Some("persist@test.com".to_string()),
    };
    store1.save(&session).unwrap();
    drop(store1);

    let store2 = GoogleOAuthStore::new(store_path);
    let loaded = store2.load().unwrap().unwrap();
    assert_eq!(loaded.access_token, "persist-token");
    assert_eq!(loaded.email, Some("persist@test.com".to_string()));
}
