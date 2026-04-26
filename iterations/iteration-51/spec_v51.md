# OpenCode RS Specification v51

**Document Version:** 51
**Generated:** 2026-04-26
**PRD Sources:** `cli.md`, `acp.md` (Agent Communication Protocol - Client Side)
**Status:** Updated — ACP Module Added with Gap Analysis Applied

---

## 1. Overview

### 1.1 Purpose

This document describes the implementation for OpenCode RS, covering both the CLI module and the ACP (Agent Communication Protocol) client module.

### 1.2 Module Information

| Module | Crate | Source | Status |
|--------|-------|--------|--------|
| CLI | `opencode-cli` | `crates/cli/src/` | Partially Implemented |
| ACP Client | `opencode-acp` | `crates/acp/` | Not Implemented - New |

---

## Part I: CLI Module Specification

---

## 2. CLI Crate Layout

```
crates/cli/src/
├── lib.rs              ← Re-exports, tests
├── cmd/                ← Command implementations
├── output/             ← NdjsonSerializer for streaming output
└── webview.rs          ← Webview integration
```

**Key Cargo.toml dependencies**:
```toml
[dependencies]
clap = { version = "4", features = ["derive"] }
tokio = { version = "1.45", features = ["full"] }
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"

opencode-core = { path = "../core" }
opencode-config = { path = "../config" }
opencode-agent = { path = "../agent" }
```

---

## 3. CLI Functional Requirements

### FR-001: NdjsonSerializer Implementation

**Priority:** P0 — Complete
**Status:** ✅ Implemented

The system MUST implement `NdjsonSerializer<W: Write>` with all methods for streaming output:

```rust
pub struct NdjsonSerializer<W: Write> {
    writer: W,
}

impl<W: Write> NdjsonSerializer<W> {
    pub fn new(writer: W) -> Self;
    pub fn write_message(&mut self, role: &str, content: &str) -> Result<(), io::Error>;
    pub fn write_start(&mut self, model: &str) -> Result<(), io::Error>;
    pub fn write_chunk(&mut self, content: &str) -> Result<(), io::Error>;
    pub fn write_done(&mut self) -> Result<(), io::Error>;
    pub fn write_error(&mut self, error: &str) -> Result<(), io::Error>;
    pub fn write_tool_call(&mut self, tool: &str, args: &str) -> Result<(), io::Error>;
    pub fn write_tool_result(&mut self, tool: &str, result: &str) -> Result<(), io::Error>;
    pub fn flush(&mut self) -> Result<(), io::Error>;
}
```

**Implementation:** `crates/cli/src/output/ndjson.rs`

---

### FR-002: Output Events (NDJSON format)

**Priority:** P0 — Complete
**Status:** ✅ Implemented

The system MUST output events in NDJSON format:

```json
{"event": "start", "model": "gpt-4o"}
{"event": "message", "role": "user", "content": "Hello"}
{"event": "chunk", "content": "Hello"}
{"event": "tool_call", "tool": "read", "args": {"path": "foo.txt"}}
{"event": "tool_result", "tool": "read", "result": "file contents"}
{"event": "error", "error": "something went wrong"}
{"event": "done"}
```

---

### FR-003: CLI Commands Enum

**Priority:** P0 — Complete
**Status:** ✅ Implemented

The system MUST implement comprehensive command parsing via clap:

```rust
pub enum CliCommand {
    Run { model: Option<String>, prompt: Option<String> },
    Session { subcommand: SessionSubcommand },
    List,
    Info,
    Config { subcommand: ConfigSubcommand },
    Agent { agent_type: String, prompt: String },
    Web { port: Option<u16> },
    Serve { port: Option<u16> },
    Desktop { port: Option<u16> },
}

pub enum SessionSubcommand {
    List,
    Show { id: String },
    Delete { id: String },
    Export { id: String, format: String },
}

pub enum ConfigSubcommand {
    Get { key: String },
    Set { key: String, value: String },
    Show,
}
```

**Implementation:** `crates/cli/src/main.rs:59-237`

---

### FR-004: Agent Run Command

**Priority:** P0 — Incomplete (Stub)
**Status:** 🚧 Gap: `cmd/agent.rs:48` — Only prints debug message

The system MUST implement actual agent execution logic with LLM provider:

```rust
pub fn run(agent_type: &str, prompt: &str, config: &Config) -> Result<ExitCode> {
    // TODO: Implement actual agent execution
    // 1. Create agent from AgentRegistry based on agent_type
    // 2. Initialize LLM provider from config
    // 3. Execute agent with prompt
    // 4. Stream output via NdjsonSerializer if --format is set
}
```

**Required Actions:**
- Query `AgentRegistry` for requested agent type
- Initialize LLM provider from `Config`
- Execute agent with provided prompt
- Handle streaming output based on `--format` flag

---

### FR-005: Config Set Command

**Priority:** P0 — Incomplete (Stub)
**Status:** 🚧 Gap: `cmd/config.rs:199-202` — Always exits with error "Invalid setting key"

The system MUST implement config key-value persistence via Config:

```rust
pub enum ConfigSubcommand {
    Set { key: String, value: String },  // Must persist to config file
}
```

**Required Actions:**
- Parse dot-notation key (e.g., `agent.model`)
- Validate value type matches expected type
- Persist to config file via Config system
- Reload config to apply changes

---

### FR-006: Run Command with LLM Streaming

**Priority:** P0 — Incomplete (Placeholder)
**Status:** 🚧 Gap: `cmd/run.rs:228-255` — Outputs mock JSON instead of actual LLM streaming

The system MUST integrate with actual LLM streaming output for `--format ndjson/json`:

```rust
pub struct RunCommand {
    pub model: Option<String>,
    pub prompt: Option<String>,
    pub format: Option<String>,  // "ndjson" or "json"
}

impl RunCommand {
    pub async fn execute(&self, config: &Config) -> Result<ExitCode> {
        // TODO: Implement actual LLM execution
        // 1. Load model from config or use default
        // 2. Initialize LLM provider
        // 3. Send prompt to LLM with streaming
        // 4. Write events to stdout via NdjsonSerializer
    }
}
```

**Required Actions:**
- Load default model from Config (not hardcoded "gpt-4o")
- Initialize appropriate LLM provider
- Stream responses via NdjsonSerializer
- Handle tool calls and results in output

---

### FR-007: Account Authentication Commands

**Priority:** P1 — Incomplete (Stub)
**Status:** 🚧 Gap: `cmd/account.rs` — Returns "not_implemented"

The system MUST implement login/logout/status with auth module:

```rust
pub enum AccountSubcommand {
    Login { provider: String },    // Browser-based OAuth
    Logout { provider: String },
    Status,
}
```

**Required Actions:**
- Integrate with `opencode-auth` crate
- Implement browser-based OAuth flow
- Store credentials securely
- Display authentication status

---

### FR-008: Attach Command

**Priority:** P1 — Incomplete (Stub)
**Status:** 🚧 Gap: `cmd/attach.rs:83-90` — Only prints URL/session_id

The system MUST implement actual session attachment to remote/local:

```rust
pub struct AttachCommand {
    pub session_id: Option<String>,
    pub url: Option<String>,
}

impl AttachCommand {
    pub async fn execute(&self, config: &Config) -> Result<ExitCode> {
        // TODO: Implement actual session attachment
        // 1. Connect to remote/local session via ACP protocol
        // 2. Transfer control to attached session
        // 3. Handle session state synchronization
    }
}
```

**Required Actions:**
- Connect via ACP protocol to remote session
- Handle local session attachment
- Manage session state transfer

---

### FR-009: MCP Add Subcommand

**Priority:** P1 — Incomplete (Missing)
**Status:** 🚧 Gap: `cmd/mcp.rs` — `Add` variant not in `McpAction` enum

The system MUST add `Add` variant to `McpAction` with server configuration:

```rust
pub enum McpAction {
    List,
    Add { name: String, command: String, args: Vec<String> },
    Remove { name: String },
}
```

**Required Actions:**
- Add `Add` variant to `McpAction` enum
- Validate MCP server command exists
- Persist MCP server configuration
- Initialize MCP client connection

---

### FR-010: Session Review and Diff Commands

**Priority:** P1 — Complete
**Status:** ✅ Implemented: `cmd/session.rs:991-1011`

The system MUST implement actual file review/diff functionality:

```rust
pub enum SessionSubcommand {
    // ... existing ...
    Review { session_id: String, file_path: String },
    Diff { session_id: String },
}
```

**Required Actions:**
- Load session messages and context
- Implement file diff using session state changes
- Show review of modified files
- Format output for terminal display

---

### FR-011: Agent List Command

**Priority:** P1 — Incomplete (Missing)
**Status:** 🚧 Gap: `cmd/agent.rs:48` — Not implemented

The system MUST query `AgentRegistry` and list available agents:

```rust
pub enum AgentSubcommand {
    List,  // Not implemented
    Run { agent_type: String, prompt: String },
}
```

**Required Actions:**
- Query AgentRegistry for all registered agents
- Display agent name, description, and capabilities
- Support `--verbose` flag for detailed info

---

### FR-012: Auth Login Command

**Priority:** P1 — Not Started
**Status:** ❌ Gap: No `auth login` implementation

The system MUST implement provider credentials login:

```bash
opencode auth login --provider openai
opencode auth login --provider anthropic
```

**Required Actions:**
- Support multiple providers (OpenAI, Anthropic, etc.)
- Browser-based OAuth flow for each provider
- Secure credential storage
- Display login status

---

### FR-013: Agent Create Command

**Priority:** P2 — Complete
**Status:** ✅ Implemented: `cmd/agent.rs:375-415`

The system MUST support creating custom agents:

```bash
opencode agent create --name my-agent --type custom
```

**Required Actions:**
- Create agent configuration ✅
- Register with AgentRegistry ✅
- Persist to config ✅

**Implementation Details:**
- `create_agent()` function in `cmd/agent.rs:375-415` handles agent creation
- Stores agent config in `config.agent.agents` HashMap
- Persists to config file via `save_config()`
- `AgentAction::Create` variant handles CLI input with name, agent_type, description, model options
- Unit tests verify agent config creation, storage, and retrieval

---

### FR-014: Session Fork Command

**Priority:** P1 — Incomplete (Partial)
**Status:** 🚧 Gap: `cmd/session.rs:673-698` — Only prints JSON output, no TUI integration

The system MUST implement session forking with proper TUI integration:

```rust
pub struct SessionForkCommand {
    pub session_id: String,
    pub new_session_id: Option<String>,
}
```

**Required Actions:**
- Create fork via SessionSharing
- Properly transfer session context
- Integrate with TUI for session management

---

### FR-015: GitHub Install Persistence

**Priority:** P1 — Incomplete
**Status:** 🚧 Gap: `cmd/github.rs:189-211` — Workflow not saved to workspace

The system MUST persist installed workflow to workspace config:

```rust
pub struct GithubInstallCommand {
    pub workflow_path: String,
    pub workspace: Option<String>,
}
```

**Required Actions:**
- Write workflow to `.opencode/` directory
- Persist to workspace config
- Verify installation on subsequent runs

---

### FR-016: Providers Login Multi-Provider Support

**Priority:** P1 — Incomplete (OpenAI only)
**Status:** 🚧 Gap: `cmd/providers.rs:142-145` — Limited to OpenAI

The system MUST extend to other providers (Anthropic, etc.):

```rust
pub enum ProviderLoginCommand {
    Login { provider: String },  // Currently only supports OpenAI
}
```

**Required Actions:**
- Extend browser auth to Anthropic
- Add Google AI Studio support
- Add Azure OpenAI support
- Add custom provider support

---

### FR-017: ACP Handshake Session Persistence

**Priority:** P1 — Incomplete
**Status:** 🚧 Gap: `cmd/acp.rs:250-292` — Session not stored for reconnection

The system MUST persist ACP session for reconnection:

```rust
pub struct AcpHandshakeCommand {
    pub client_id: String,
    pub capabilities: Vec<String>,
}
```

**Required Actions:**
- Store ACP session after successful handshake
- Implement session recovery on reconnect
- Handle session expiration

---

### FR-018: Config Migrate Command

**Priority:** P1 — Incomplete (Stub)
**Status:** 🚧 Gap: `cmd/config.rs:204-208` — Stub with TOML deprecation error

The system MUST implement TOML→JSONC migration or remove flag:

```rust
pub enum ConfigSubcommand {
    Migrate,  // Currently errors with "TOML config deprecated"
}
```

**Required Actions:**
- Either implement migration from TOML to JSONC format
- Or remove the `--migrate` flag with deprecation notice

---

### FR-019: Environment Variable Parsing

**Priority:** P2 — Incomplete
**Status:** 🚧 Gap: `main.rs` — Env vars not parsed before App init

The system MUST parse and apply environment variables before App init:

| Variable | Description | Status |
|----------|-------------|--------|
| `OPENCODE_AUTO_SHARE` | Auto-share sessions | 🚧 Missing |
| `OPENCODE_CONFIG` | Config file path | 🚧 Missing |
| `OPENCODE_CONFIG_DIR` | Config directory path | 🚧 Missing |
| `OPENCODE_DISABLE_AUTOUPDATE` | Disable auto-update check | 🚧 Missing |
| `OPENCODE_ENABLE_EXA` | Enable Exa web search | 🚧 Missing |
| `OPENCODE_SERVER_PASSWORD` | Server basic auth password | 🚧 Missing |

**Required Actions:**
- Parse env vars before Config loading
- Apply env var overrides to configuration
- Document env var precedence order

---

### FR-020: Model Visibility Config Integration

**Priority:** P2 — Incomplete (Flat file)
**Status:** 🚧 Gap: `cmd/models.rs:238-267` — Uses flat JSON file instead of Config

The system MUST use Config system for model visibility:

```rust
pub struct ModelsVisibilityCommand {
    pub model_id: String,
    pub visible: bool,
}
```

**Required Actions:**
- Move model visibility to Config system
- Update Config schema to support model visibility
- Deprecate flat JSON file

---

### FR-021: Default Model from Config

**Priority:** P2 — Incomplete (Hardcoded)
**Status:** 🚧 Gap: `cmd/run.rs:226` — Hardcoded "gpt-4o"

The system MUST load default model from Config or registry:

```rust
impl RunCommand {
    fn get_default_model(&self, config: &Config) -> String {
        config.get("agent.model").unwrap_or_else(|_| "gpt-4o".to_string())
    }
}
```

---

### FR-022: Session Undo/Redo

**Priority:** P0 — Complete
**Status:** ✅ Implemented

The system provides working history operations:

```rust
// cmd/session.rs:603-667
pub enum SessionSubcommand {
    Undo { session_id: String },
    Redo { session_id: String },
}
```

---

### FR-023: Session Export

**Priority:** P0 — Complete
**Status:** ✅ Implemented

```rust
// cmd/session.rs:743-779
pub enum SessionSubcommand {
    Export { id: String, format: String },
}
```

---

### FR-024: Serve Command

**Priority:** P0 — Complete
**Status:** ✅ Implemented

```rust
// cmd/serve.rs — Full server initialization
pub struct ServeCommand {
    pub port: Option<u16>,
}
```

---

### FR-025: Web Command

**Priority:** P0 — Complete
**Status:** ✅ Implemented

```rust
// cmd/web.rs — Full web interface
pub struct WebCommand {
    pub port: Option<u16>,
}
```

---

### FR-026: Desktop Command

**Priority:** P0 — Complete
**Status:** ✅ Implemented

```rust
// cmd/desktop.rs — Full desktop mode
pub struct DesktopCommand {
    pub port: Option<u16>,
}
```

---

### FR-027: ACP CLI Commands

**Priority:** P0 — Complete (Partial)
**Status:** 🚧 Gap: `cmd/acp.rs` — Missing `ack` command

```rust
// cmd/acp.rs — HTTP API calls
pub enum AcpSubcommand {
    Status,
    Connect { url: String },
    Handshake { client_id: String, capabilities: Vec<String> },
    Ack,  // ❌ MISSING
}
```

---

### FR-028: GitHub OAuth Integration

**Priority:** P0 — Complete
**Status:** ✅ Implemented

```rust
// cmd/github.rs — OAuth + GitHub API
pub enum GithubSubcommand {
    Login,
    RepoList,
    IssueList,
}
```

---

### FR-029: Providers List

**Priority:** P0 — Complete
**Status:** ✅ Implemented

```rust
// cmd/providers.rs — Comprehensive provider list
pub enum ProvidersSubcommand {
    List,
    Login { provider: String },
}
```

---

### FR-030: Models List/Switch

**Priority:** P0 — Complete
**Status:** ✅ Implemented

```rust
// cmd/models.rs — Full model registry access
pub enum ModelsSubcommand {
    List,
    Switch { model: String },
}
```

---

### FR-031: Session List/Show/Delete

**Priority:** P0 — Complete
**Status:** ✅ Implemented

```rust
// cmd/session.rs — Full CRUD operations
pub enum SessionSubcommand {
    List,
    Show { id: String },
    Delete { id: String },
}
```

---

## Part II: ACP Module Specification (Client-Side)

---

## 4. ACP Module Overview

### 4.1 Purpose

- **Module Name**: `acp` (Agent Communication Protocol)
- **Source Path**: `crates/acp/`
- **Type**: Integration
- **Rust Crate**: `opencode-acp`
- **Purpose**: Client-side Agent Communication Protocol — connects to remote ACP servers, performs handshake, exchanges messages, and reports status.

> **Note**: The `control-plane` crate covers the server-side ACP implementation. This `acp` crate covers the **client-side** ACP client that agents use to connect to other agents/servers.

---

## 5. ACP Crate Layout

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

---

## 6. ACP Functional Requirements

### FR-100: Create `crates/acp/` Crate Structure

**Priority:** P0 — Critical
**Status:** ❌ Not Implemented

The system MUST create the `crates/acp/` crate with proper structure:

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

### FR-101: AcpConnectionState Enum

**Priority:** P0 — Critical
**Status:** ❌ Not Implemented

The system MUST implement the connection state enum:

```rust
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AcpConnectionState {
    Disconnected,
    Handshaking,
    Connected,
    Failed(String),
}
```

---

### FR-102: AcpState Struct

**Priority:** P0 — Critical
**Status:** ❌ Not Implemented

The system MUST implement the state struct:

```rust
struct AcpState {
    connection_state: AcpConnectionState,
    client_id: String,
    server_id: Option<String>,
    session_token: Option<String>,
    capabilities: Vec<String>,
    server_url: Option<String>,
}
```

---

### FR-103: AcpClient Struct

**Priority:** P0 — Critical
**Status:** ❌ Not Implemented

The system MUST implement the ACP client:

```rust
pub struct AcpClient {
    http: reqwest::Client,
    state: Arc<Mutex<AcpState>>,
    bus: Arc<BusService>,
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

---

### FR-104: AcpError Enum

**Priority:** P0 — Critical
**Status:** ❌ Not Implemented

The system MUST implement comprehensive error handling:

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

### FR-105: ACP Protocol Types

**Priority:** P0 — Critical
**Status:** ❌ Not Implemented

The system MUST implement all protocol types:

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

---

### FR-106: ACP State Machine

**Priority:** P0 — Critical
**Status:** ❌ Not Implemented

The system MUST implement atomic, thread-safe state transitions:

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

**Required Actions:**
- State transitions must be atomic
- Use `Arc<Mutex<AcpState>>` for thread safety
- Publish bus events on state changes

---

### FR-107: Bus Event Publishing

**Priority:** P0 — Critical
**Status:** ❌ Not Implemented

The system MUST publish bus events on connection state changes:

```rust
// On successful connect:
bus.publish("acp.connected", { server_id, capabilities })

// On disconnect:
bus.publish("acp.disconnected", {})
```

---

### FR-108: ACP CLI Commands

**Priority:** P1 — Important
**Status:** ❌ Not Implemented

The system MUST implement all ACP CLI commands:

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

/// acp ack --handshake-id <id> --accepted <bool>
pub async fn cmd_ack(client: &AcpClient, handshake_id: &str, accepted: bool) -> Result<(), AcpError> {
    client.ack(handshake_id, accepted).await?;
    println!("Handshake acknowledgement sent");
    Ok(())
}
```

---

### FR-109: ACP Unit Tests

**Priority:** P0 — Critical
**Status:** ❌ Not Implemented

The system MUST implement comprehensive unit tests:

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

---

### FR-110: ACP Integration Tests

**Priority:** P2 — Nice to Have
**Status:** ❌ Not Implemented

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

## 7. ACP API Reference

### 7.1 Types

| Type | Description |
|------|-------------|
| `AcpConnectionState` | Connection state enum (Disconnected, Handshaking, Connected, Failed) |
| `AcpStatus` | Status struct with connected, client_id, capabilities, server_url |
| `HandshakeRequest` | Request for handshake with client_id, capabilities, version |
| `HandshakeResponse` | Response with server_id, accepted_capabilities, session_token |
| `ConnectRequest` | Request to connect with url and client_id |
| `AckRequest` | Request to acknowledge handshake with handshake_id and accepted |
| `AcpMessage` | Message struct with from, to, message_type, payload, timestamp |

### 7.2 ACP Command Reference

| Command | Description | HTTP Method |
|---------|-------------|-------------|
| `status` | Get ACP server status | GET |
| `handshake` | Perform capability exchange | POST |
| `connect` | Connect to agent at URL | POST |
| `ack` | Acknowledge handshake | POST |

---

## 8. ACP State Machine Diagram

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

### ACP Protocol Flow

#### Connect Flow

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

#### Message Flow

```
client.send_message(to, type, payload)
  → POST {server_url}/api/acp/message
    { to, from: client_id, message_type, payload, timestamp }
  ← { ok: true } or { error: "..." }
```

---

## 9. ACP Dependencies

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

## 10. CLI Usage Reference

```bash
# ACP protocol commands
opencode acp status
opencode acp connect --url <url>
opencode acp handshake --client-id <id> --capabilities chat,tasks
opencode acp ack --handshake-id <id> --accepted true
```

---

## 11. Implementation Status Summary

### CLI Module - Completed ✅

| Feature | FR Number |
|---------|-----------|
| NdjsonSerializer with all methods | FR-001 |
| Test suite for NdjsonSerializer | FR-001 |
| CLI argument parsing (clap) | FR-003 |
| `session list/show/delete` | FR-031 |
| `session export` | FR-023 |
| `session undo/redo` | FR-022 |
| `models list/switch` | FR-030 |
| `providers list` | FR-029 |
| `github login/repo-list/issue-list` | FR-028 |
| `serve` command | FR-024 |
| `web` command | FR-025 |
| `desktop` command | FR-026 |
| `acp status/connect/handshake` | FR-027 |
| `session review/diff` | FR-010 |

### CLI Module - In Progress 🚧

| Feature | FR Number | Gap Location |
|---------|-----------|--------------|
| `agent run` | FR-004 | `cmd/agent.rs:48` |
| `config set` | FR-005 | `cmd/config.rs:199-202` |
| `run --format ndjson/json` | FR-006 | `cmd/run.rs:228-255` |
| `account login/logout/status` | FR-007 | `cmd/account.rs` |
| `attach` | FR-008 | `cmd/attach.rs:83-90` |
| `mcp add` | FR-009 | `cmd/mcp.rs` |
| `agent list` | FR-011 | `cmd/agent.rs:48` |
| `session fork` | FR-014 | `cmd/session.rs:673-698` |
| `github install` persistence | FR-015 | `cmd/github.rs:189-211` |
| `providers login` multi-provider | FR-016 | `cmd/providers.rs:142-145` |
| `acp handshake` session storage | FR-017 | `cmd/acp.rs:250-292` |
| `config migrate` | FR-018 | `cmd/config.rs:204-208` |
| Environment variables | FR-019 | `main.rs` |
| Model visibility config | FR-020 | `cmd/models.rs:238-267` |
| Default model from config | FR-021 | `cmd/run.rs:226` |
| `acp ack` command | FR-027 | `cmd/acp.rs` |

### ACP Module - Not Started ❌

| Feature | FR Number |
|---------|-----------|
| Create `crates/acp/` crate structure | FR-100 |
| `AcpConnectionState` enum | FR-101 |
| `AcpState` struct | FR-102 |
| `AcpClient` struct with async methods | FR-103 |
| `AcpError` enum | FR-104 |
| ACP Protocol types | FR-105 |
| ACP State machine | FR-106 |
| Bus event publishing | FR-107 |
| ACP CLI commands (ack) | FR-108 |
| ACP unit tests | FR-109 |
| ACP integration tests | FR-110 |

---

## 12. Technical Debt

### CLI Module Technical Debt

| Item | Location | Description | Priority |
|------|----------|-------------|----------|
| Magic string "gpt-4o" | `cmd/run.rs:226` | Default model should come from Config | P2 |
| Magic string "cmd+k" | `cmd/config.rs:217-218` | Hardcoded keybinds in JSON output | P2 |
| No error propagation | `cmd/providers.rs:67-88` | `open_browser()` uses `unwrap()` | P1 |
| Duplicated `load_config()` | Multiple cmd files | Each command loads Config independently | P2 |
| `#[allow(dead_code)]` on modules | `cmd/mod.rs` | Some modules may be unused | P3 |
| Hardcoded API base URL | `cmd/github.rs:5-8` | Should be configurable | P2 |
| `SessionRecord` duplication | `cmd/session.rs:11-24` | Duplicates core Session types | P2 |
| `ModelRow` struct | `cmd/models.rs:44-52` | Could use existing model types | P3 |
| `ProviderRow` struct | `cmd/providers.rs:23-30` | Duplicates provider info | P3 |
| Async runtime creation | Multiple cmd files | Each command creates own Runtime | P2 |
| No shared error handling | Throughout | Commands exit with `std::process::exit(1)` | P2 |

### ACP Module Technical Debt

| Item | Description | Remediation |
|---|---|---|
| Protocol types in core | `crates/core/src/acp.rs` contains protocol types that should be in `crates/acp/` | Move to `crates/acp/src/protocol.rs` |
| Duplicate `AcpHandshakeResponse` | CLI defines its own `AcpHandshakeResponse` in `crates/cli/src/cmd/acp.rs` | Import from shared `crates/acp` or `crates/core` |
| Mixed sync/async | `AcpProtocol` is sync but ACP API is async | Rewrite as async `AcpClient` |
| Hardcoded URLs | URLs like `/api/acp/handshake` are hardcoded in CLI | Move to configurable base URL |
| No connection timeout | `connect()` has no timeout configuration | Add configurable timeout |
| No retry logic | Failed connections not retried | Implement retry with backoff |
| Missing `version` in `AcpStatus` | Status doesn't report protocol version | Add `version` field |

---

## 13. File Locations Reference

### CLI Files

| File | Purpose |
|---|---|
| `crates/cli/src/lib.rs` | CLI module entry point |
| `crates/cli/src/main.rs` | CLI argument parsing |
| `crates/cli/src/cmd/` | Command implementations |
| `crates/cli/src/output/ndjson.rs` | NDJSON serializer |

### ACP Files

| File | Purpose |
|---|---|
| `crates/acp/` | **MISSING** - Client crate not created |
| `crates/core/src/acp.rs` | Current partial ACP types (to be moved/consolidated) |
| `crates/control-plane/src/lib.rs` | Server-side ACP exports |
| `crates/control-plane/src/transport.rs` | Server-side transport layer |
| `crates/control-plane/src/handshake.rs` | Server-side handshake |
| `crates/control-plane/src/acp_stream.rs` | Event stream for ACP |
| `crates/cli/src/cmd/acp.rs` | Current CLI commands (incomplete) |
| `crates/config/src/lib.rs` | `AcpConfig` and `AcpSession` types |

---

## 14. Acceptance Criteria

### CLI Module

#### Must Have (P0)
- [ ] `agent run` executes actual agent with LLM provider
- [ ] `config set` persists key-value pairs to config file
- [ ] `run --format ndjson/json` streams actual LLM responses

#### Should Have (P1)
- [ ] `account login/logout/status` works with auth module
- [ ] `attach` connects to remote/local sessions
- [ ] `mcp add` adds MCP servers
- [ ] `session review/diff` shows file changes
- [ ] `agent list` displays available agents
- [ ] ACP handshake persists session

#### Could Have (P2)
- [ ] Environment variables are parsed before config load
- [ ] Model visibility uses Config system
- [ ] Default model loads from Config
- [ ] `providers login` supports multiple providers
- [ ] `github install` persists workflow to workspace

### ACP Module

#### Must Have (P0)
- [ ] `crates/acp/` crate created with proper structure
- [ ] `AcpClient` with http, state, bus fields
- [ ] `AcpConnectionState` enum with Disconnected/Handshaking/Connected/Failed
- [ ] `AcpState` struct with all required fields
- [ ] `AcpError` enum with all error variants
- [ ] `status()` returns correct `AcpStatus`
- [ ] `connect()` transitions state: Disconnected → Handshaking → Connected
- [ ] `handshake()` sends correct request and parses response
- [ ] `connect()` publishes `acp.connected` bus event on success
- [ ] `disconnect()` transitions to Disconnected and publishes `acp.disconnected`
- [ ] `send_message()` returns error when not connected
- [ ] State transitions are atomic and thread-safe
- [ ] ACP errors are descriptive and include server response when available

#### Should Have (P1)
- [ ] CLI commands `status`, `connect`, `ack` work correctly
- [ ] `ack` CLI command added to handle handshake acknowledgement

#### Could Have (P2)
- [ ] Integration tests for full connect → message → disconnect cycle
- [ ] Session sharing functionality

---

## 15. Cross-References

- [PRD](./cli.md) — CLI Module PRD
- [PRD](./acp.md) — ACP Module PRD
- [Gap Analysis](./gap-analysis.md) — ACP Module Gap Analysis

---

*Document generated from gap analysis. Implementation status tracked per FR number.*
*Iteration 51: ACP Module added with full specification based on PRD and gap analysis.*