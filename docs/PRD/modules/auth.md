# auth.md — Authentication Module

## Module Overview

- **Crate**: `opencode-auth`
- **Source**: `crates/auth/src/lib.rs`
- **Status**: Fully implemented — PRD reflects actual Rust API
- **Purpose**: Authentication and credential management with support for API keys, OAuth (Google, Copilot), JWT tokens, and credential stores.

---

## Crate Layout

```
crates/auth/src/
├── lib.rs              ← Public re-exports
├── credential_ref.rs
├── credential_store.rs
├── jwt.rs              ← JWT validation
├── manager.rs          ← AuthManager
├── oauth.rs            ← OAuth flows
└── password.rs         ← Password hashing/comparison
```

**Key Cargo.toml dependencies**:
```toml
[dependencies]
async-trait = "0.1"
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
thiserror = "2.0"
tokio = { version = "1.45", features = ["full"] }

opencode-core = { path = "../core" }
```

**Public exports from lib.rs**:
```rust
pub use credential_ref::CredentialRef;
pub use credential_store::{CredentialStore, InMemoryCredentialStore};
pub use jwt::{JwtClaims, JwtValidator};
pub use manager::{AuthManager, AuthStrategy, ProviderAuthConfig};
pub use oauth::{
    AuthApplicator, OAuthSession, OAuthSessionManager, OAuthTokenResponse,
};
```

---

## Core Types

### Credential

```rust
pub struct Credential {
    pub provider: String,
    pub key: String,  // API key or token
    pub refresh_token: Option<String>,
    pub expires_at: Option<DateTime<Utc>>,
}
```

### CredentialStore Trait

```rust
pub trait CredentialStore: Send + Sync {
    fn get(&self, provider: &str) -> Option<Credential>;
    fn set(&self, provider: &str, credential: Credential);
    fn remove(&self, provider: &str);
    fn list_providers(&self) -> Vec<String>;
}

// Implementations:
pub struct InMemoryCredentialStore {
    credentials: RwLock<HashMap<String, Credential>>,
}
```

### AuthManager

```rust
pub struct AuthManager {
    stores: HashMap<String, Arc<dyn CredentialStore>>,
    jwt_validators: HashMap<String, Arc<JwtValidator>>,
}

impl AuthManager {
    pub fn new() -> Self;
    pub fn add_store(&mut self, provider: String, store: Arc<dyn CredentialStore>);
    pub fn add_jwt_validator(&mut self, provider: String, validator: Arc<JwtValidator>);

    pub fn get_credential(&self, provider: &str) -> Option<Credential>;
    pub fn set_credential(&self, provider: &str, credential: Credential);
    pub fn remove_credential(&self, provider: &str);

    pub fn validate_token(&self, provider: &str, token: &str) -> Result<JwtClaims, AuthError>;
    pub fn refresh_token(&self, provider: &str, refresh_token: &str) -> Result<OAuthTokenResponse, AuthError>;
}

pub enum AuthStrategy {
    ApiKey(String),
    OAuth(OAuthSession),
    Bearer(String),
    Jwt(JwtClaims),
}

pub struct ProviderAuthConfig {
    pub provider: String,
    pub strategy: AuthStrategy,
}
```

### JWT

```rust
pub struct JwtClaims {
    pub sub: String,
    pub iss: Option<String>,
    pub aud: Option<String>,
    pub exp: DateTime<Utc>,
    pub iat: DateTime<Utc>,
    pub email: Option<String>,
    pub name: Option<String>,
}

pub struct JwtValidator {
    key: Vec<u8>,
    algorithm: JwtAlgorithm,
}

pub enum JwtAlgorithm {
    Hs256,
    Rs256,
}
```

### OAuth

```rust
pub struct OAuthSession {
    pub provider: String,
    pub access_token: String,
    pub refresh_token: Option<String>,
    pub expires_at: DateTime<Utc>,
    pub scopes: Vec<String>,
}

pub struct OAuthSessionManager {
    sessions: Arc<RwLock<HashMap<String, OAuthSession>>>,
}

impl OAuthSessionManager {
    pub fn new() -> Self;
    pub fn create_session(&self, session: OAuthSession) -> Result<String, AuthError>;  // returns session_id
    pub fn get_session(&self, session_id: &str) -> Option<OAuthSession>;
    pub fn remove_session(&self, session_id: &str);
}

pub struct OAuthTokenResponse {
    pub access_token: String,
    pub refresh_token: Option<String>,
    pub expires_in: u64,
    pub token_type: String,
}

pub struct AuthApplicator {
    // Applies auth headers to HTTP requests
}
```

### AuthError

```rust
#[derive(Debug, thiserror::Error)]
pub enum AuthError {
    #[error("credential not found: {0}")]
    CredentialNotFound(String),
    #[error("invalid token: {0}")]
    InvalidToken(String),
    #[error("token expired")]
    TokenExpired,
    #[error("oauth error: {0}")]
    OAuthError(String),
    #[error("keychain error: {0}")]
    Keychain(String),
}
```

---

## Inter-Crate Dependencies

| Dependant Crate | What it uses from `opencode-auth` |
|---|---|
| `opencode-server` | `AuthManager` for request authentication |
| `opencode-llm` | `AuthManager` to attach credentials to LLM requests |
| `opencode-control-plane` | JWT validation for ACP connections |

---

## Test Design

```rust
#[cfg(test)]
mod tests {
    #[test]
    fn test_credential_store_get_set() {
        let store = InMemoryCredentialStore::new();
        let cred = Credential { provider: "openai".into(), key: "sk-123".into(), refresh_token: None, expires_at: None };
        store.set("openai", cred);
        assert!(store.get("openai").is_some());
    }

    #[test]
    fn test_credential_store_remove() {
        let store = InMemoryCredentialStore::new();
        store.set("openai", Credential { provider: "openai".into(), key: "sk-123".into(), refresh_token: None, expires_at: None });
        store.remove("openai");
        assert!(store.get("openai").is_none());
    }

    #[test]
    fn test_oauth_session_manager() {
        let manager = OAuthSessionManager::new();
        let session = OAuthSession {
            provider: "google".into(),
            access_token: "access".into(),
            refresh_token: Some("refresh".into()),
            expires_at: Utc::now() + Duration::hours(1),
            scopes: vec!["email".into()],
        };
        let id = manager.create_session(session).unwrap();
        assert!(manager.get_session(&id).is_some());
    }

    #[test]
    fn test_auth_error_display() {
        let err = AuthError::InvalidToken("malformed JWT".into());
        assert!(err.to_string().contains("malformed JWT"));
    }
}
```
