# server.md — HTTP Server Module

## Module Overview

- **Crate**: `opencode-server`
- **Source**: `crates/server/src/lib.rs`
- **Status**: Fully implemented — PRD reflects actual Rust API
- **Purpose**: Actix-web HTTP server with WebSocket session hub, ACP integration, REST API routes, mDNS discovery, CORS middleware, and shared server state across all request handlers.

---

## Crate Layout

```
crates/server/src/
├── lib.rs              ← ServerState, run_server, run_server_with_shutdown, health_check
├── mdns.rs             ← mDNS service discovery
├── middleware.rs       ← CORS, auth middleware
├── routes/            ← API route handlers
│   ├── mod.rs
│   ├── config_routes.rs
│   ├── ws.rs           ← SessionHub WebSocket
│   ├── share.rs        ← ShareServer
│   ├── acp_ws.rs       ← ACP WebSocket client registry
│   ├── web_ui.rs       ← Static file serving, index
│   ├── status.rs       ← /api/status endpoint
│   └── error.rs        ← JSON error responses
├── streaming/          ← WebSocket streaming
│   ├── mod.rs
│   └── conn_state.rs   ← ConnectionMonitor, ReconnectionStore
└── server_integration_tests.rs
```

**Key Cargo.toml dependencies**:
```toml
[dependencies]
actix-web = "4"
actix-web-actors = "4"
futures = "0.3"
tokio = { version = "1.45", features = ["full"] }
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
uuid = { version = "1.0", features = ["v4", "serde"] }
thiserror = "2.0"

opencode-core = { path = "../core" }
opencode-llm = { path = "../llm" }
opencode-storage = { path = "../storage" }
opencode-tools = { path = "../tools" }
opencode-permission = { path = "../permission" }
opencode-control-plane = { path = "../control-plane" }
```

---

## Core Types

### ServerState

```rust
#[derive(Clone)]
pub struct ServerState {
    pub storage: Arc<StorageService>,
    pub models: Arc<ModelRegistry>,
    pub config: Arc<RwLock<Config>>,
    pub event_bus: SharedEventBus,
    pub reconnection_store: ReconnectionStore,
    pub temp_db_dir: Option<PathBuf>,
    pub connection_monitor: Arc<ConnectionMonitor>,
    pub share_server: Arc<RwLock<ShareServer>>,
    pub acp_enabled: bool,
    pub acp_stream: SharedAcpStream,
    pub acp_client_registry: SharedAcpClientRegistry,
    pub tool_registry: Arc<ToolRegistry>,
    pub session_hub: Arc<SessionHub>,
    pub server_start_time: std::time::SystemTime,
    pub permission_manager: Arc<RwLock<opencode_core::permission::PermissionManager>>,
    pub approval_queue: Arc<RwLock<ApprovalQueue>>,
    pub audit_log: Option<Arc<AuditLog>>,
}
```

### SessionHub

```rust
// From routes/ws.rs
pub struct SessionHub {
    sessions: Arc<RwLock<HashMap<String, SessionHandle>>>,
    broadcast_tx: broadcast::Sender<HubMessage>,
}

impl SessionHub {
    pub fn new(capacity: usize) -> Self;
    pub async fn join(&self, session_id: &str, handle: SessionHandle) -> Result<(), HubError>;
    pub async fn leave(&self, session_id: &str);
    pub async fn broadcast(&self, msg: HubMessage) -> Result<(), HubError>;
    pub fn subscribe(&self) -> broadcast::Receiver<HubMessage>;
}

pub struct SessionHandle { ... }  // WebSocket connection handle
pub enum HubMessage { ... }
pub enum HubError { ... }
```

### ShareServer

```rust
// From routes/share.rs
pub struct ShareServer {
    config: ShareConfig,
    sessions: Arc<RwLock<HashMap<String, SharedSession>>>,
}

impl ShareServer {
    pub fn with_default_config() -> Self;
    pub async fn create_share_link(&self, session_id: Uuid) -> Result<String, ShareError>;
    pub async fn get_shared_session(&self, share_id: &str) -> Result<SharedSession, ShareError>;
    pub async fn expire_share(&self, share_id: &str) -> Result<(), ShareError>;
}
```

### ConnectionMonitor and ReconnectionStore

```rust
// From streaming/conn_state.rs
pub struct ConnectionMonitor {
    connections: Arc<AtomicU32>,
    last_disconnect: Arc<Mutex<Option<std::time::SystemTime>>>,
}

pub struct ReconnectionStore {
    store: Arc<RwLock<HashMap<String, ReconnectionInfo>>>,
}
```

---

## Server Functions

```rust
pub async fn health_check() -> impl Responder {
    HttpResponse::Ok().json(serde_json::json!({
        "status": "ok",
        "version": env!("CARGO_PKG_VERSION")
    }))
}

pub async fn run_server(state: Arc<ServerState>, host: &str, port: u16) -> std::io::Result<()> {
    // Actix-web server with all routes, CORS, auth middleware
    // Returns on server shutdown
}

pub async fn run_server_with_shutdown(
    state: Arc<ServerState>,
    host: &str,
    port: u16,
    shutdown_rx: oneshot::Receiver<()>,
) -> std::io::Result<()> {
    // Same as run_server but listens for shutdown signal
    // Cancels gracefully on shutdown_rx
}

fn validate_port(port: u16) -> std::io::Result<()> {
    // Port must be >= 1024
}
```

### Route Registration

```rust
App::new()
    .route("/", web::get().to(routes::web_ui::index))
    .route("/api/docs", web::get().to(routes::web_ui::api_docs))
    .route("/static/{filename:.*}", web::get().to(routes::web_ui::serve_static))
    .route("/health", web::get().to(health_check))
    .route("/api/status", web::get().to(routes::status::get_status))
    .service(
        web::scope("/api")
            .wrap_fn(|req, srv| {
                // API key authorization middleware
                // Returns 401 if invalid
                Either::Left(ready(...)) or Either::Right(srv.call(req))
            })
            .configure(routes::config_routes),
    )
```

---

## Inter-Crate Dependencies

| Dependant Crate | What it uses from `opencode-server` |
|---|---|
| `opencode-cli` | `run_server`, `run_server_with_shutdown` to start HTTP server |
| `opencode-core` | `ServerState` for global access |
| `opencode-tui` | Desktop mode uses embedded server |

**Dependencies of `opencode-server`**:
| Crate | Usage |
|---|---|
| `opencode-core` | `Config`, `ServerConfig`, `SharedEventBus` |
| `opencode-storage` | `StorageService` |
| `opencode-llm` | `ModelRegistry` |
| `opencode-tools` | `ToolRegistry` |
| `opencode-permission` | `ApprovalQueue`, `AuditLog`, `PermissionScope` |
| `opencode-control-plane` | `SharedAcpStream`, `SharedAcpClientRegistry` |

---

## Test Design

```rust
#[cfg(test)]
mod tests {
    #[test]
    fn validate_port_rejects_privileged_ports() {
        assert!(validate_port(80).is_err());
        assert!(validate_port(1023).is_err());
    }

    #[test]
    fn validate_port_accepts_non_privileged_ports() {
        assert!(validate_port(1024).is_ok());
        assert!(validate_port(65535).is_ok());
    }

    #[test]
    fn server_state_clone_preserves_fields() {
        // Full ServerState construction with all Arc/RwLock fields
        // Clones and verifies fields are preserved
    }

    #[test]
    fn server_state_all_fields_initialized() {
        // Verifies all 17 ServerState fields can be initialized
    }
}

#[cfg(test)]
mod server_integration_tests { ... }
```
