# PRD: acp Module

## Module Overview

- **Module Name**: `acp` (Agent Communication Protocol)
- **Source Path**: `packages/opencode/src/acp/`
- **Type**: Integration
- **Rust Crate**: `crates/acp/` (note: `crates/control-plane/` is already implemented — this is the client-side ACP protocol)
- **Purpose**: Client-side Agent Communication Protocol — connects to remote ACP servers, performs handshake, exchanges messages, and reports status.

> **Note**: The `control-plane.md` PRD covers the server-side ACP implementation. This `acp.md` covers the **client-side** ACP client that agents use to connect to other agents/servers.

---

## Functionality

### Core Features

1. **ACP Client Lifecycle** — Connect, handshake, exchange messages, disconnect
2. **Status Reporting** — `acp status` command shows connected/disconnected state
3. **Handshake Protocol** — Exchange client ID and capabilities with server
4. **Message Passing** — Send/receive typed messages to/from connected agents
5. **Session Sharing** — Share local session with remote agent via ACP
6. **CLI Commands** — `acp status`, `acp handshake`, `acp connect`, `acp ack`

### ACP Command Reference

| Command | Description | HTTP Method |
|---------|-------------|-------------|
| `status` | Get ACP server status | GET |
| `handshake` | Perform capability exchange | POST |
| `connect` | Connect to agent at URL | POST |
| `ack` | Acknowledge handshake | POST |

---

## API Surface

### Types

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AcpStatus {
    pub connected: bool,
    pub client_id: Option<String>,
    pub capabilities: Vec<String>,
    pub server_url: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HandshakeRequest {
    pub client_id: String,
    pub capabilities: Vec<String>,
    pub version: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HandshakeResponse {
    pub server_id: String,
    pub accepted_capabilities: Vec<String>,
    pub session_token: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConnectRequest {
    pub url: String,
    pub client_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AckRequest {
    pub handshake_id: String,
    pub accepted: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AcpMessage {
    pub from: String,
    pub to: String,
    pub message_type: String,
    pub payload: serde_json::Value,
    pub timestamp: i64,
}
```

### `AcpClient`

```rust
pub struct AcpClient {
    http: reqwest::Client,
    state: Arc<Mutex<AcpState>>,
    bus: Arc<BusService>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AcpConnectionState {
    Disconnected,
    Handshaking,
    Connected,
    Failed(String),
}

struct AcpState {
    connection_state: AcpConnectionState,
    client_id: String,
    server_id: Option<String>,
    session_token: Option<String>,
    capabilities: Vec<String>,
    server_url: Option<String>,
}

impl AcpClient {
    /// Create a new ACP client
    pub fn new(http: reqwest::Client, bus: Arc<BusService>) -> Self

    /// Get current status
    pub async fn status(&self) -> Result<AcpStatus, AcpError>

    /// Perform handshake with server
    pub async fn handshake(
        &self,
        server_url: &str,
        client_id: String,
        capabilities: Vec<String>,
    ) -> Result<HandshakeResponse, AcpError>

    /// Connect to a remote ACP server
    pub async fn connect(
        &self,
        server_url: &str,
        client_id: Option<String>,
    ) -> Result<(), AcpError>

    /// Acknowledge handshake
    pub async fn ack(
        &self,
        handshake_id: &str,
        accepted: bool,
    ) -> Result<(), AcpError>

    /// Send a message to the connected agent
    pub async fn send_message(
        &self,
        to: &str,
        message_type: &str,
        payload: serde_json::Value,
    ) -> Result<(), AcpError>

    /// Disconnect from server
    pub async fn disconnect(&self) -> Result<(), AcpError>

    /// Returns current connection state
    pub fn connection_state(&self) -> AcpConnectionState
}
```

### `AcpError`

```rust
#[derive(Debug, Error)]
pub enum AcpError {
    #[error("Not connected")]
    NotConnected,

    #[error("Handshake failed: {0}")]
    HandshakeFailed(String),

    #[error("Connection failed: {0}")]
    ConnectionFailed(String),

    #[error("Server returned error: {0}")]
    ServerError(String),

    #[error("Invalid response: {0}")]
    InvalidResponse(String),

    #[error("HTTP error: {0}")]
    Http(#[from] reqwest::Error),

    #[error("State error: {0}")]
    State(String),
}
```

---

## State Machine

```
Disconnected
    │
    │ connect()
    ▼
Handshaking ──(handshake success)──► Connected
    │                                    │
    │                                    │ disconnect()
    │                                    ▼
    └──(handshake failure)──► Failed
```

---

## ACP Protocol Flow

### Connect Flow

```
1. client.connect(url)
   → AcpConnectionState::Handshaking
   → POST {url}/api/acp/handshake
   ← { server_id, accepted_capabilities, session_token }
   → AcpConnectionState::Connected
   → bus.publish("acp.connected", { server_id, capabilities })

2. On disconnect:
   → AcpConnectionState::Disconnected
   → bus.publish("acp.disconnected", {})
```

### Message Flow

```
client.send_message(to, type, payload)
  → POST {server_url}/api/acp/message
    { to, from: client_id, message_type, payload, timestamp }
  ← { ok: true } or { error: "..." }
```

---

## Crate Layout

```
crates/acp/
├── Cargo.toml       # reqwest = { version = "0.12", features = ["json", "rustls-tls"] }
├── src/
│   ├── lib.rs       # AcpClient, AcpError, types
│   ├── client.rs    # Client implementation
│   ├── protocol.rs  # Protocol types and serialization
│   └── cli.rs       # CLI command handlers
└── tests/
    └── acp_tests.rs
```

### `Cargo.toml`

```toml
[package]
name = "opencode-acp"
version = "0.1.0"
edition = "2021"

[dependencies]
tokio = { version = "1.45", features = ["full"] }
reqwest = { version = "0.12", features = ["json", "rustls-tls"] }
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
thiserror = "2.0"
tracing = "0.1"
chrono = { version = "0.4", features = ["serde"] }
uuid = { version = "1.0", features = ["v4", "serde"] }

[dev-dependencies]
wiremock = "0.6"
tokio-test = "0.4"
```

---

## CLI Commands

```rust
/// acp status — shows current connection state
pub async fn cmd_status(client: &AcpClient) -> Result<(), AcpError> {
    let status = client.status().await?;
    println!("ACP Status: {}", if status.connected { "connected" } else { "disconnected" });
    if let Some(id) = &status.client_id {
        println!("Client ID: {}", id);
    }
    if !status.capabilities.is_empty() {
        println!("Capabilities: {}", status.capabilities.join(", "));
    }
    Ok(())
}

/// acp connect --url <url> --client-id <id>
pub async fn cmd_connect(client: &AcpClient, url: &str, client_id: Option<&str>) -> Result<(), AcpError> {
    let cid = client_id.unwrap_or_else(|| &client.state().client_id);
    client.connect(url, cid.to_string()).await?;
    println!("Connected to {}", url);
    Ok(())
}
```

---

## Dependencies

| Dependency | Purpose |
|---|---|
| `reqwest` | HTTP client for ACP API calls |
| `tokio` | Async runtime |
| `serde` / `serde_json` | JSON serialization |
| `thiserror` | Error enum |
| `tracing` | Structured logging |
| `chrono` | Timestamp for messages |
| `uuid` | Message ID generation |
| `bus` module | Event publishing (`acp.connected`, `acp.disconnected`) |

---

## Acceptance Criteria

- [ ] `status()` returns correct `AcpStatus` with connected state, client_id, capabilities
- [ ] `connect()` transitions state: `Disconnected → Handshaking → Connected`
- [ ] `handshake()` sends correct `HandshakeRequest` and parses `HandshakeResponse`
- [ ] `connect()` publishes `acp.connected` bus event on success
- [ ] `disconnect()` transitions to `Disconnected` and publishes `acp.disconnected`
- [ ] `send_message()` returns error when not connected
- [ ] State transitions are atomic and thread-safe
- [ ] ACP errors are descriptive and include server response when available
- [ ] CLI commands `status`, `connect`, `ack` work correctly

---

## Test Design

### Unit Tests

```rust
#[tokio::test]
async fn test_status_returns_disconnected_initially() {
    let mock = MockServer::start().await;
    let client = AcpClient::new_test(&mock.uri());
    let status = client.status().await.unwrap();
    assert!(!status.connected);
}

#[tokio::test]
async fn test_connect_transitions_state() {
    let mock = MockServer::start().await;
    Mock::given(post, "/api/acp/handshake")
        .respond_with(json_response(200, &HandshakeResponse {
            server_id: "srv1".into(),
            accepted_capabilities: vec!["chat".into()],
            session_token: Some("tok1".into()),
        }))
        .expect(1)
        .mount(&mock)
        .await;

    let client = AcpClient::new_test(&mock.uri());
    assert_eq!(client.connection_state(), AcpConnectionState::Disconnected);

    client.connect(&mock.uri(), "my-client".into()).await.unwrap();
    assert_eq!(client.connection_state(), AcpConnectionState::Connected);
}

#[tokio::test]
async fn test_send_message_returns_error_when_not_connected() {
    let client = AcpClient::new_test("http://localhost:1");
    let result = client.send_message("srv", "chat", json!({"text": "hi"})).await;
    assert!(matches!(result, Err(AcpError::NotConnected)));
}

#[tokio::test]
async fn test_disconnect_transitions_to_disconnected() {
    let mock = MockServer::start().await;
    Mock::given(post, "/api/acp/handshake")
        .respond_with(json_response(200, &HandshakeResponse {
            server_id: "srv1".into(),
            accepted_capabilities: vec![],
            session_token: None,
        }))
        .mount(&mock)
        .await;

    let client = AcpClient::new_test(&mock.uri());
    client.connect(&mock.uri(), "my-client".into()).await.unwrap();
    client.disconnect().await.unwrap();
    assert_eq!(client.connection_state(), AcpConnectionState::Disconnected);
}
```

### Integration Tests

```rust
#[tokio::test]
async fn test_full_connect_message_disconnect_cycle() {
    // Spin up mock ACP server
    // 1. GET /api/acp/status → { connected: false }
    // 2. POST /api/acp/handshake → { server_id, accepted_capabilities }
    // 3. POST /api/acp/message → { ok: true }
    // 4. Verify bus events fired
}
```

---

## Source Reference

*Source: `packages/opencode/src/acp/index.ts`*
*No existing Rust equivalent in `crates/control-plane/` — implement in `crates/acp/` as separate client crate*
