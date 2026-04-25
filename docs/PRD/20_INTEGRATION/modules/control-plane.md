# control-plane.md — Control Plane / ACP Module

## Module Overview

- **Crate**: `opencode-control-plane`
- **Source**: `crates/control-plane/src/lib.rs`
- **Status**: Fully implemented — PRD reflects actual Rust API
- **Purpose**: Agent Communication Protocol (ACP) — secure peer-to-peer communication, handshake/negotiation, SSO/OIDC integration, connection pooling, workspace management, and event streaming between opencode instances.

---

## Crate Layout

```
crates/control-plane/src/
├── lib.rs              ← Public re-exports
├── [various modules for ACP]
```

**Key Cargo.toml dependencies**:
```toml
[dependencies]
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
thiserror = "2.0"
tokio = { version = "1.45", features = ["full"] }
reqwest = { version = "0.12" }

opencode-core = { path = "../core" }
```

**Public exports**:
```rust
pub use acp::{
    AcpAgentEvent, AcpEventStream, AcpEventType, SharedAcpStream,
    EventBus, AcpHandshake, AcpHandshakeConfig, AcpHandshakeConfirmation,
    AcpHandshakeManager, AcpHandshakeResponse, AcpOutgoingHandshake,
    HandshakeState, Jwk, JwkClaims, Jwks, JwksError, JwksValidator,
    SamlAssertion, SamlAuthnRequest, SamlAuthnRequestBuilder, SamlError,
    SamlResponse, OidcState, SsoConfig, SsoManager, SsoProvider,
    AcpConnectionManager, AcpConnectionState, AcpIncomingMessage,
    AcpOutgoingMessage, AcpTransportClient, SharedConnectionManager,
    WorkspaceManager,
};
```

---

## Core Types

### ACP Event Stream

```rust
pub type SharedAcpStream = Arc<AcpEventStream>;

pub struct AcpEventStream {
    tx: broadcast::Sender<AcpAgentEvent>,
    rx: broadcast::Receiver<AcpAgentEvent>,
}

pub enum AcpEventType {
    AgentEvent(AcpAgentEvent),
    Heartbeat,
    Disconnect,
    Reconnect,
}

pub struct AcpAgentEvent {
    pub event_type: String,
    pub session_id: Option<String>,
    pub payload: serde_json::Value,
    pub timestamp: DateTime<Utc>,
}
```

### ACP Handshake

```rust
pub struct AcpHandshake {
    pub config: AcpHandshakeConfig,
    pub state: HandshakeState,
}

#[derive(Debug, Clone)]
pub struct AcpHandshakeConfig {
    pub server_id: String,
    pub version: String,
    pub capabilities: Vec<String>,
    pub timeout: Duration,
}

pub enum HandshakeState {
    Idle,
    WaitingForResponse,
    Completed,
    Failed(String),
}

pub struct AcpHandshakeManager { ... }
impl AcpHandshakeManager {
    pub fn new(config: AcpHandshakeConfig) -> Self;
    pub async fn initiate(&self) -> Result<AcpHandshakeResponse, AcpError>;
    pub async fn accept(&self, incoming: AcpIncomingHandshake) -> Result<AcpHandshakeConfirmation, AcpError>;
}

pub struct AcpHandshakeResponse {
    pub accepted: bool,
    pub server_id: String,
    pub session_key: Option<String>,
    pub error: Option<String>,
}

pub struct AcpHandshakeConfirmation {
    pub session_id: String,
    pub capabilities: Vec<String>,
    pub expires_at: DateTime<Utc>,
}
```

### SSO / OIDC

```rust
pub struct SsoManager {
    providers: HashMap<String, SsoProvider>,
}

pub struct SsoProvider {
    pub name: String,
    pub issuer: String,
    pub client_id: String,
    pub discovery_url: Option<String>,
    pub jwks_uri: Option<String>,
}

impl SsoManager {
    pub fn new() -> Self;
    pub fn register_provider(&mut self, provider: SsoProvider) -> Result<(), SsoError>;
    pub async fn authenticate(&self, provider_name: &str, token: &str) -> Result<OidcState, SsoError>;
    pub fn get_jwks(&self, provider_name: &str) -> Result<Jwks, SsoError>;
}

pub struct OidcState {
    pub sub: String,
    pub email: Option<String>,
    pub name: Option<String>,
    pub access_token: String,
    pub expires_at: DateTime<Utc>,
}
```

### JWKS / JWT Validation

```rust
pub struct Jwk {
    pub kty: String,
    pub kid: Option<String>,
    pub alg: Option<String>,
    pub n: Option<String>,  // RSA modulus
    pub e: Option<String>,  // RSA exponent
    pub use_: Option<String>,
}

pub struct Jwks {
    pub keys: Vec<Jwk>,
}

pub struct JwkClaims {
    pub sub: String,
    pub iss: String,
    pub aud: String,
    pub exp: DateTime<Utc>,
    pub iat: DateTime<Utc>,
    pub email: Option<String>,
}

pub struct JwksValidator { ... }
impl JwksValidator {
    pub fn new(jwks: Jwks) -> Self;
    pub fn validate_token(&self, token: &str) -> Result<JwkClaims, JwksError>;
}

#[derive(Debug, thiserror::Error)]
pub enum JwksError {
    #[error("invalid token")]
    InvalidToken,
    #[error("token expired")]
    Expired,
    #[error("no matching key found")]
    NoKeyFound,
}
```

### SAML

```rust
pub struct SamlAuthnRequestBuilder { ... }
impl SamlAuthnRequestBuilder {
    pub fn new(issuer: &str, acs_url: &str) -> Self;
    pub fn with_name_id_policy(&mut self, policy: &str) -> &mut Self;
    pub fn build(&self) -> SamlAuthnRequest;
}

pub struct SamlAuthnRequest { ... }
impl SamlAuthnRequest {
    pub fn encoded(&self) -> String;
}

pub struct SamlResponse { ... }
impl SamlResponse {
    pub fn decode(encoded: &str) -> Result<Self, SamlError>;
    pub fn assertions(&self) -> Vec<SamlAssertion>;
}

pub struct SamlAssertion {
    pub subject: String,
    pub session_index: Option<String>,
    pub attributes: HashMap<String, String>,
}

#[derive(Debug, thiserror::Error)]
pub enum SamlError {
    #[error("invalid SAML response")]
    InvalidResponse,
    #[error("signature verification failed")]
    SignatureInvalid,
    #[error("SAML error: {0}")]
    Other(String),
}
```

### Connection Management

```rust
pub type SharedConnectionManager = Arc<AcpConnectionManager>;

pub struct AcpConnectionManager {
    connections: RwLock<HashMap<String, AcpConnectionState>>,
    transport: Arc<AcpTransportClient>,
}

pub struct AcpConnectionState {
    pub peer_id: String,
    pub established_at: DateTime<Utc>,
    pub last_heartbeat: DateTime<Utc>,
    pub status: ConnectionStatus,
}

pub enum ConnectionStatus {
    Connected,
    Disconnected,
    Reconnecting,
}

pub struct AcpTransportClient {
    client: reqwest::Client,
}

impl AcpTransportClient {
    pub fn new() -> Self;
    pub async fn send(&self, endpoint: &str, msg: AcpOutgoingMessage) -> Result<AcpIncomingMessage, AcpError>;
}

pub struct AcpOutgoingMessage {
    pub to: String,
    pub msg_type: String,
    pub payload: serde_json::Value,
}

pub struct AcpIncomingMessage {
    pub from: String,
    pub msg_type: String,
    pub payload: serde_json::Value,
}
```

### Workspace Manager

```rust
pub struct WorkspaceManager {
    workspaces: RwLock<HashMap<String, Workspace>>,
}

pub struct Workspace {
    pub id: String,
    pub name: String,
    pub owner: String,
    pub members: Vec<String>,
    pub created_at: DateTime<Utc>,
}

impl WorkspaceManager {
    pub fn new() -> Self;
    pub async fn create_workspace(&self, name: &str, owner: &str) -> Result<Workspace, AcpError>;
    pub async fn get_workspace(&self, id: &str) -> Option<Workspace>;
    pub async fn list_workspaces(&self, user: &str) -> Vec<Workspace>;
    pub async fn add_member(&self, workspace_id: &str, user: &str) -> Result<(), AcpError>;
}
```

---

## Inter-Crate Dependencies

| Dependant Crate | What it uses from `opencode-control-plane` |
|---|---|
| `opencode-server` | `SharedAcpStream`, `AcpConnectionManager` in `ServerState` |
| `opencode-core` | Event types for ACP event bus |

---

## Test Design

```rust
#[cfg(test)]
mod tests {
    #[test]
    fn test_jwks_validator_validates_token() { ... }

    #[test]
    fn test_saml_authn_request_encoding() {
        let req = SamlAuthnRequestBuilder::new("https://issuer.com", "https://acs.url")
            .build();
        let encoded = req.encoded();
        assert!(!encoded.is_empty());
    }

    #[tokio::test]
    async fn test_acp_event_stream_broadcast() {
        let stream = AcpEventStream::new(256);
        let event = AcpAgentEvent {
            event_type: "session.started".into(),
            session_id: Some("s1".into()),
            payload: serde_json::json!({}),
            timestamp: Utc::now(),
        };
        stream.broadcast(event).unwrap();
    }

    #[test]
    fn test_handshake_state_transitions() {
        let mut handshake = AcpHandshake::new(config);
        assert_eq!(handshake.state, HandshakeState::Idle);
        // ... transition tests
    }
}
```
