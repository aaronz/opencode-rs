# PRD: account Module

## Module Overview

- **Module Name**: account
- **Source Path**: `packages/opencode/src/account/`
- **Type**: Integration Service
- **Purpose**: User account management — device-code OAuth flow for opencode.ai authentication, token storage/refresh, organization listing, and account info retrieval.

---

## Functionality

### Core Features

1. **Device Code OAuth** — Initiates device-code flow (`/oauth/device/code`), polls for completion
2. **Token Storage** — Stores access tokens and refresh tokens in SQLite via `AccountRepo`
3. **Token Refresh** — Transparently refreshes expired access tokens using refresh tokens
4. **Organization Listing** — Lists organizations the authenticated user belongs to
5. **Account Info** — Returns current account profile (user ID, email, display name)
6. **Poll Result Handling** — Returns typed poll results: `PollSuccess`, `PollPending`, `PollSlow`, `PollExpired`, `PollDenied`, `PollError`
7. **Remote Config** — Fetches remote configuration from control plane
8. **Cache** — Caches account info with TTL to avoid repeated API calls

### OAuth Flow

```
1. POST /oauth/device/code → DeviceCode + UserCode + verification_uri
2. Display UserCode to user (or open browser to verification_uri)
3. Poll /oauth/token with DeviceCode until success or expiry
4. Store AccessToken + RefreshToken in SQLite
5. On subsequent calls: use AccessToken, refresh if 401
```

---

## API Surface

### Types

```typescript
interface Info {
  id: AccountID
  email: string
  name: string
}

interface Org {
  id: OrgID
  name: string
  slug: string
}

interface Login {
  deviceCode: DeviceCode
  userCode: UserCode
  verificationUri: string
  expiresIn: number
  interval: number
}

type PollResult =
  | PollSuccess   // { account: Info; accessToken: string }
  | PollPending
  | PollSlow      // slow_down: increase polling interval
  | PollExpired   // device code expired
  | PollDenied    // user denied
  | PollError     // unexpected error
```

### Account Service

```typescript
interface AccountService {
  login(serverUrl: string): Effect<Login>
  poll(deviceCode: DeviceCode, serverUrl: string): Effect<PollResult>
  refresh(accountID: AccountID): Effect<string>   // returns new access token
  list(): Effect<Info[]>
  orgs(accountID: AccountID): Effect<Org[]>
  remoteConfig(accountID: AccountID): Effect<Record<string, Json>>
}
```

---

## Data Structures

### Database Table (`account.sql.ts`)

```sql
CREATE TABLE account (
  id          TEXT PRIMARY KEY,
  email       TEXT NOT NULL,
  name        TEXT NOT NULL,
  access_token TEXT NOT NULL,
  refresh_token TEXT,
  server_url  TEXT NOT NULL,
  created_at  INTEGER NOT NULL
);
```

---

## Dependencies

| Dependency | Purpose |
|---|---|
| `storage` module | SQLite for token persistence (`AccountRepo`) |
| `effect/http` | OAuth API calls |
| `effect/cache` | Account info TTL caching |

---

## Acceptance Criteria

- [ ] Device code flow starts and returns user code + verification URI
- [ ] Polling returns correct typed result for each server state
- [ ] Tokens are stored in SQLite on successful login
- [ ] Expired access tokens are automatically refreshed
- [ ] `list()` returns all stored accounts
- [ ] `orgs()` returns organizations for an account
- [ ] `remoteConfig()` fetches and caches remote configuration

---

## Rust Implementation Guidance

### Crate: `crates/auth/` (merge with existing auth crate)

### Key Crates

```toml
reqwest = { features = ["json"] }
rusqlite = { features = ["bundled"] }
tokio = { features = ["full"] }
serde = { features = ["derive"] }
```

### Architecture

```rust
pub struct AccountService {
    db: Arc<Mutex<Connection>>,
    http: reqwest::Client,
}

impl AccountService {
    pub async fn login(&self, server_url: &str) -> Result<LoginInfo> {
        let resp: DeviceCodeResponse = self.http
            .post(format!("{}/oauth/device/code", server_url))
            .json(&json!({"client_id": "opencode"}))
            .send().await?
            .json().await?;
        Ok(LoginInfo { device_code: resp.device_code, user_code: resp.user_code, .. })
    }

    pub async fn poll(&self, device_code: &str, server_url: &str) -> Result<PollResult> {
        let resp = self.http
            .post(format!("{}/oauth/token", server_url))
            .json(&json!({ "device_code": device_code, "grant_type": "urn:ietf:params:oauth:grant-type:device_code" }))
            .send().await?;
        match resp.status().as_u16() {
            200 => Ok(PollResult::Success(resp.json().await?)),
            400 => {
                let err: ErrorResponse = resp.json().await?;
                match err.error.as_str() {
                    "authorization_pending" => Ok(PollResult::Pending),
                    "slow_down" => Ok(PollResult::Slow),
                    "expired_token" => Ok(PollResult::Expired),
                    "access_denied" => Ok(PollResult::Denied),
                    _ => Ok(PollResult::Error(err.error)),
                }
            }
            _ => Err(AccountError::Transport),
        }
    }
}
```

---

## Test Design

```rust
#[tokio::test]
async fn test_poll_pending_returns_pending_variant() {
    let mock = MockServer::start().await;
    Mock::given(method("POST")).and(path("/oauth/token"))
        .respond_with(ResponseTemplate::new(400)
            .set_body_json(json!({"error": "authorization_pending"})))
        .mount(&mock).await;
    let svc = AccountService::new_test(&mock.uri());
    let result = svc.poll("device123", &mock.uri()).await.unwrap();
    assert!(matches!(result, PollResult::Pending));
}

#[tokio::test]
async fn test_successful_login_stores_token() {
    let svc = AccountService::new_test_with_db();
    // simulate successful poll
    svc.store_tokens("acc1", "access_tok", Some("refresh_tok"), "https://api.example.com").await.unwrap();
    let accounts = svc.list().await.unwrap();
    assert_eq!(accounts.len(), 1);
}
```

### Integration Tests (from TS patterns)

- `account/repo.test.ts`: CRUD operations on account SQLite table
- `account/service.test.ts`: Full OAuth device code flow with mock server
