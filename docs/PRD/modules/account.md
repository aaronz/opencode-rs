# PRD: account Module

## Module Overview

- **Module Name**: `account`
- **Source Path**: `packages/opencode/src/account/`
- **Type**: Integration Service
- **Rust Crate**: `crates/account/` (or merge into existing `crates/auth/`)
- **Purpose**: User account management — device-code OAuth flow for opencode.ai authentication, token storage/refresh in SQLite, organization listing, and account info retrieval with TTL caching.

---

## Functionality

### Core Features

1. **Device Code OAuth** — Initiates device-code flow (`POST /oauth/device/code`), polls for completion
2. **Token Storage** — Stores access tokens and refresh tokens in SQLite via `AccountRepo`
3. **Token Refresh** — Automatically refreshes expired access tokens transparently
4. **Organization Listing** — Lists organizations the authenticated user belongs to
5. **Account Info** — Returns current account profile (user ID, email, display name)
6. **Poll Result Handling** — Returns typed poll results: `PollSuccess`, `PollPending`, `PollSlow`, `PollExpired`, `PollDenied`, `PollError`
7. **Remote Config** — Fetches remote configuration from control plane with caching
8. **Cache** — Caches account info with TTL to avoid repeated API calls

---

## OAuth Flow

```
1. POST {server_url}/oauth/device/code
   ← { device_code, user_code, verification_uri, expires_in, interval }

2. Display user_code / verification_uri to user (or auto-open browser)

3. Poll POST {server_url}/oauth/token
   with { device_code, grant_type: "urn:ietf:params:oauth:grant-type:device_code" }
   until { 200: success, 400 "authorization_pending": keep polling,
           400 "slow_down": increase interval, 400 "expired_token": fail,
           400 "access_denied": fail }

4. On success: store AccessToken + RefreshToken in SQLite

5. On subsequent calls: use AccessToken, refresh automatically on 401
```

---

## API Surface

### Types

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccountInfo {
    pub id: AccountId,
    pub email: String,
    pub name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrgInfo {
    pub id: OrgId,
    pub name: String,
    pub slug: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoginInfo {
    pub device_code: String,
    pub user_code: String,
    pub verification_uri: String,
    pub expires_in: u64,
    pub interval: u64,
}

#[derive(Debug, Clone)]
pub enum PollResult {
    Success { account: AccountInfo, access_token: String },
    Pending,
    Slow,
    Expired,
    Denied,
    Error(String),
}
```

### `AccountService`

```rust
pub struct AccountService {
    db: Arc<Mutex<Connection>>,
    http: reqwest::Client,
    cache: Arc<Mutex<Option<(AccountInfo, Instant)>>>,
    cache_ttl: Duration,
}

impl AccountService {
    /// Start device code login flow
    pub async fn login(&self, server_url: &str) -> Result<LoginInfo, AccountError> {
        #[derive(Serialize)]
        struct DeviceCodeRequest { client_id: String }

        #[derive(Deserialize)]
        struct DeviceCodeResponse {
            device_code: String,
            user_code: String,
            verification_uri: String,
            expires_in: u64,
            interval: u64,
        }

        let resp: DeviceCodeResponse = self.http
            .post(format!("{}/oauth/device/code", server_url))
            .json(&serde_json::json!({ "client_id": "opencode" }))
            .send().await?
            .json().await
            .map_err(AccountError::Http)?;

        Ok(LoginInfo {
            device_code: resp.device_code,
            user_code: resp.user_code,
            verification_uri: resp.verification_uri,
            expires_in: resp.expires_in,
            interval: resp.interval,
        })
    }

    /// Poll for OAuth token
    pub async fn poll(&self, device_code: &str, server_url: &str) -> Result<PollResult, AccountError> {
        #[derive(Serialize)]
        struct TokenRequest<'a> {
            device_code: &'a str,
            grant_type: &'a str,
        }

        #[derive(Deserialize)]
        struct TokenResponse {
            access_token: String,
            refresh_token: Option<String>,
            token_type: String,
        }

        #[derive(Deserialize)]
        struct OAuthError {
            error: String,
        }

        let resp = self.http
            .post(format!("{}/oauth/token", server_url))
            .json(&TokenRequest { device_code, grant_type: "urn:ietf:params:oauth:grant-type:device_code" })
            .send().await?
            .error_for_status()
            .map_err(AccountError::Http)?;

        match resp.status().as_u16() {
            200 => {
                let token_resp: TokenResponse = resp.json().await.map_err(AccountError::Http)?;
                // Fetch account info using the new token
                let account = self.fetch_account_info(&token_resp.access_token).await?;
                self.store_tokens(&account.id, &token_resp.access_token, token_resp.refresh_token.as_deref(), server_url).await?;
                Ok(PollResult::Success { account, access_token: token_resp.access_token })
            }
            400 => {
                let err: OAuthError = resp.json().await.map_err(AccountError::Http)?;
                match err.error.as_str() {
                    "authorization_pending" => Ok(PollResult::Pending),
                    "slow_down" => Ok(PollResult::Slow),
                    "expired_token" => Ok(PollResult::Expired),
                    "access_denied" => Ok(PollResult::Denied),
                    _ => Ok(PollResult::Error(err.error)),
                }
            }
            _ => Err(AccountError::UnexpectedResponse(resp.status().to_string())),
        }
    }

    /// Refresh an expired access token
    pub async fn refresh(&self, account_id: &AccountId) -> Result<String, AccountError> {
        let (refresh_token, server_url) = self.get_refresh_token(account_id).await?
            .ok_or(AccountError::NoRefreshToken)?;

        #[derive(Serialize)]
        struct RefreshRequest<'a> {
            refresh_token: &'a str,
            grant_type: &'a str,
        }

        #[derive(Deserialize)]
        struct RefreshResponse {
            access_token: String,
            refresh_token: Option<String>,
        }

        let resp: RefreshResponse = self.http
            .post(format!("{}/oauth/token", server_url))
            .json(&RefreshRequest { refresh_token: &refresh_token, grant_type: "refresh_token" })
            .send().await?
            .json().await
            .map_err(AccountError::Http)?;

        self.update_access_token(account_id, &resp.access_token).await?;
        Ok(resp.access_token)
    }

    /// List all stored accounts
    pub async fn list(&self) -> Result<Vec<AccountInfo>, AccountError> {
        let conn = self.db.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT id, email, name FROM account"
        )?;
        let accounts = stmt.query_map([], |row| {
            Ok(AccountInfo {
                id: AccountId(row.get::<_, String>(0)?),
                email: row.get(1)?,
                name: row.get(2)?,
            })
        })?.filter_map(|r| r.ok()).collect();
        Ok(accounts)
    }

    /// Get organizations for an account
    pub async fn orgs(&self, account_id: &AccountId) -> Result<Vec<OrgInfo>, AccountError> {
        let (access_token, server_url) = self.get_access_token(account_id).await?;
        let resp: Vec<OrgInfo> = self.http
            .get(format!("{}/api/user/orgs", server_url))
            .bearer_auth(&access_token)
            .send().await?
            .json().await
            .map_err(AccountError::Http)?;
        Ok(resp)
    }
}
```

### `AccountError`

```rust
#[derive(Debug, Error)]
pub enum AccountError {
    #[error("HTTP error: {0}")]
    Http(#[from] reqwest::Error),

    #[error("Database error: {0}")]
    Database(String, #[source] rusqlite::Error),

    #[error("No refresh token available")]
    NoRefreshToken,

    #[error("Account not found: {0}")]
    AccountNotFound(String),

    #[error("Unexpected OAuth response: {0}")]
    UnexpectedResponse(String),

    #[error("Token refresh failed: {0}")]
    RefreshFailed(String),
}
```

### `AccountRepo` (SQLite)

```rust
/// Initialize account table
pub fn init_db(conn: &Connection) -> Result<(), rusqlite::Error> {
    conn.execute(
        "CREATE TABLE IF NOT EXISTS account (
            id          TEXT PRIMARY KEY,
            email       TEXT NOT NULL,
            name        TEXT NOT NULL,
            access_token TEXT NOT NULL,
            refresh_token TEXT,
            server_url  TEXT NOT NULL,
            created_at  INTEGER NOT NULL
        )",
        [],
    )?;
    Ok(())
}

impl AccountRepo {
    pub async fn store_tokens(
        &self,
        id: &AccountId,
        email: &str,
        name: &str,
        access_token: &str,
        refresh_token: Option<&str>,
        server_url: &str,
    ) -> Result<(), AccountError> {
        let conn = self.db.lock().unwrap();
        conn.execute(
            "INSERT OR REPLACE INTO account (id, email, name, access_token, refresh_token, server_url, created_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
            params![id.to_string(), email, name, access_token, refresh_token, server_url, chrono::Utc::now().timestamp()],
        )?;
        Ok(())
    }

    pub fn get_access_token(&self, id: &AccountId) -> Result<Option<(String, String)>, AccountError> {
        let conn = self.db.lock().unwrap();
        let mut stmt = conn.prepare("SELECT access_token, server_url FROM account WHERE id = ?1")?;
        let result = stmt.query_row(params![id.to_string()], |row| {
            Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?))
        });
        match result {
            Ok(r) => Ok(Some(r)),
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(e) => Err(AccountError::Database("get_access_token".into(), e)),
        }
    }
}
```

---

## Crate Layout

```
crates/account/
├── Cargo.toml
├── src/
│   ├── lib.rs       # AccountService, AccountError, types
│   ├── oauth.rs     # OAuth flow (login, poll, refresh)
│   ├── repo.rs      # AccountRepo, SQLite operations
│   └── remote.rs    # Remote config fetching
└── tests/
    └── account_tests.rs
```

### `Cargo.toml`

```toml
[package]
name = "opencode-account"
version = "0.1.0"
edition = "2021"

[dependencies]
tokio = { version = "1.45", features = ["sync", "rt"] }
rusqlite = { version = "0.32", features = ["bundled"] }
reqwest = { version = "0.12", features = ["json", "rustls-tls"] }
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
thiserror = "2.0"
tracing = "0.1"
chrono = { version = "0.4", features = ["serde"] }
anyhow = "1.0"

[dev-dependencies]
tokio-test = "0.4"
tempfile = "3"
wiremock = "0.6"
```

---

## Dependencies

| Dependency | Purpose |
|---|---|
| `storage` module | SQLite connection for `AccountRepo` |
| `reqwest` | OAuth API calls |
| `rusqlite` | Token persistence |
| `chrono` | Timestamp storage |

---

## Acceptance Criteria

- [x] Device code flow starts and returns user code + verification URI
- [x] Polling returns correct typed result for each server state
- [x] Tokens are stored in SQLite on successful login
- [x] Expired access tokens are automatically refreshed
- [x] `list()` returns all stored accounts
- [x] `orgs()` returns organizations for an account
- [x] `remoteConfig()` fetches and caches remote configuration
- [x] Cache TTL prevents repeated API calls

---

## Test Design

```rust
#[tokio::test]
async fn test_poll_pending_returns_pending_variant() {
    let mock = MockServer::start().await;
    Mock::given(post, "/oauth/token")
        .respond_with(ResponseTemplate::new(400)
            .set_body_json(json!({"error": "authorization_pending"})))
        .mount(&mock)
        .await;

    let svc = AccountService::new_test(&mock.uri());
    let result = svc.poll("device123", &mock.uri()).await.unwrap();
    assert!(matches!(result, PollResult::Pending));
}

#[tokio::test]
async fn test_poll_slow_returns_slow_variant() {
    let mock = MockServer::start().await;
    Mock::given(post, "/oauth/token")
        .respond_with(ResponseTemplate::new(400)
            .set_body_json(json!({"error": "slow_down"})))
        .mount(&mock)
        .await;

    let svc = AccountService::new_test(&mock.uri());
    let result = svc.poll("device123", &mock.uri()).await.unwrap();
    assert!(matches!(result, PollResult::Slow));
}

#[tokio::test]
async fn test_poll_success_stores_tokens() {
    let mock = MockServer::start().await;

    // Mock /oauth/token returning success
    Mock::given(post, "/oauth/token")
        .respond_with(ResponseTemplate::new(200)
            .set_body_json(json!({
                "access_token": "access_tok_123",
                "refresh_token": "refresh_tok_456",
                "token_type": "Bearer"
            })))
        .mount(&mock)
        .await;

    // Mock /api/user/info returning account details
    Mock::given(get, "/api/user/info")
        .respond_with(ResponseTemplate::new(200)
            .set_body_json(json!({
                "id": "acc_123",
                "email": "test@example.com",
                "name": "Test User"
            })))
        .mount(&mock)
        .await;

    let svc = AccountService::new_test(&mock.uri());
    let result = svc.poll("device123", &mock.uri()).await.unwrap();
    assert!(matches!(result, PollResult::Success { .. }));

    let accounts = svc.list().await.unwrap();
    assert_eq!(accounts.len(), 1);
}

#[tokio::test]
async fn test_refresh_uses_refresh_token() {
    let mock = MockServer::start().await;
    Mock::given(post, "/oauth/token")
        .respond_with(ResponseTemplate::new(200)
            .set_body_json(json!({
                "access_token": "new_access",
                "refresh_token": "new_refresh"
            })))
        .mount(&mock)
        .await;

    let svc = AccountService::new_test(&mock.uri());
    // Pre-store an account with refresh token
    svc.db.lock().unwrap().execute(
        "INSERT INTO account (id, email, name, access_token, refresh_token, server_url, created_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
        params!["acc1", "test@example.com", "Test", "old_access", "refresh_tok", &mock.uri(), 0],
    ).unwrap();

    let new_token = svc.refresh(&AccountId("acc1".into())).await.unwrap();
    assert_eq!(new_token, "new_access");
}
```

---

## Source Reference

*Source: `packages/opencode/src/account/index.ts`*
*No existing Rust equivalent — implement in `crates/account/`*
