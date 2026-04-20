# Module: bus

## Overview

The `bus` module lives in `opencode-core` (`crates/core/src/bus.rs`). It implements a process-wide broadcast event bus using `tokio::sync::broadcast`. Every subsystem (agent, server, TUI, storage, MCP) subscribes to `InternalEvent` variants to react to state changes without direct coupling.

**Crate**: `opencode-core`  
**Source**: `crates/core/src/bus.rs`  
**Status**: Fully implemented (272 lines)

---

## Crate Layout

```
crates/core/src/
└── bus.rs
```

**`crates/core/src/lib.rs`** exports:
```rust
pub mod bus;
pub use bus::{EventBus, InternalEvent, SharedEventBus};
```

**Key deps** (`crates/core/Cargo.toml`):
```toml
tokio = { workspace = true, features = ["sync"] }
serde = { workspace = true, features = ["derive"] }
```

---

## Core Types

### `InternalEvent`

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum InternalEvent {
    // Session lifecycle
    SessionStarted(String),
    SessionEnded(String),
    SessionForked { original_id: String, new_id: String, fork_point: usize },
    SessionShared { session_id: String, share_url: String },
    // Messages
    MessageAdded { session_id: String, message_id: String },
    MessageUpdated { session_id: String, message_id: String },
    // Tool calls
    ToolCallStarted { session_id: String, tool_name: String, call_id: String },
    ToolCallEnded { session_id: String, call_id: String, success: bool },
    ToolCallOutput { session_id: String, call_id: String, output: String },
    // Agent
    AgentStatusChanged { session_id: String, status: String },
    AgentStarted { session_id: String, agent: String },
    AgentStopped { session_id: String, agent: String },
    // Provider / model
    ProviderChanged { provider: String, model: String },
    ModelChanged { model: String },
    // Compaction
    CompactionTriggered { session_id: String, pruned_count: usize },
    CompactionCompleted { session_id: String, summary_inserted: bool },
    // System
    ConfigUpdated,
    AuthChanged { user_id: Option<String> },
    PermissionGranted { user_id: String, permission: String },
    PermissionDenied { user_id: String, permission: String },
    FileWatchEvent { path: String, kind: String },
    LspDiagnosticsUpdated { path: String, count: usize },
    ShellEnvChanged { key: String, value: String },
    UiToastShow { message: String, level: String },
    // Permission flow
    PermissionAsked { session_id: String, request_id: String, permission: String },
    PermissionReplied { session_id: String, request_id: String, granted: bool },
    // Plugins / MCP
    PluginLoaded { name: String },
    PluginUnloaded { name: String },
    McpServerConnected { name: String },
    McpServerDisconnected { name: String },
    AcpEventReceived { agent_id: String, event_type: String },
    // Server
    ServerStarted { port: u16 },
    ServerStopped,
    Error { source: String, message: String },
}

impl InternalEvent {
    /// Returns session_id for session-scoped events; None for global.
    pub fn session_id(&self) -> Option<&str> { ... }
}
```

### `EventBus` and `SharedEventBus`

```rust
pub struct EventBus {
    tx: broadcast::Sender<InternalEvent>,
}

/// Type alias used everywhere — always wrap in Arc.
pub type SharedEventBus = Arc<EventBus>;
```

Channel capacity: **256** (hardcoded). Lagged receivers get `RecvError::Lagged(n)`.

---

## Key Implementations

```rust
impl EventBus {
    pub fn new() -> Self {
        let (tx, _) = broadcast::channel(256);
        Self { tx }
    }

    /// Subscribe — late subscribers miss past events.
    pub fn subscribe(&self) -> broadcast::Receiver<InternalEvent> {
        self.tx.subscribe()
    }

    /// Publish — fire-and-forget; Ok(n) = number of receivers, ignored.
    pub fn publish(&self, event: InternalEvent) {
        let _ = self.tx.send(event);
    }

    pub fn subscriber_count(&self) -> usize {
        self.tx.receiver_count()
    }
}

impl Default for EventBus {
    fn default() -> Self { Self::new() }
}
```

---

## Inter-crate Dependencies

| Consumer | How it uses the bus |
|---|---|
| `opencode-server` | `SharedEventBus` in `ServerState`; publishes tool/agent events via SSE |
| `opencode-agent` | Subscribes to `PermissionReplied`, publishes `AgentStarted/Stopped` |
| `opencode-tui` | Subscribes to all events to update UI state in render loop |
| `opencode-mcp` | Publishes `McpServerConnected/Disconnected` |
| `opencode-storage` | May listen to `SessionStarted` to pre-create DB rows |

**Pattern** — always pass `SharedEventBus` by `Arc::clone`:
```rust
// In app bootstrap:
let bus: SharedEventBus = Arc::new(EventBus::new());

// Pass to subsystems:
let server_bus = Arc::clone(&bus);
let agent_bus = Arc::clone(&bus);
```

---

## Concurrency Pattern

`tokio::sync::broadcast` — MPMC:
- Publisher holds `Sender<T>` (Clone + Send + Sync)
- Each subscriber gets its own `Receiver<T>` via `.subscribe()`
- Always handle `RecvError::Lagged` in subscriber loops:

```rust
let mut rx = bus.subscribe();
tokio::spawn(async move {
    loop {
        match rx.recv().await {
            Ok(event) => handle(event).await,
            Err(broadcast::error::RecvError::Lagged(n)) => {
                tracing::warn!("bus: lagged {} events", n);
            }
            Err(broadcast::error::RecvError::Closed) => break,
        }
    }
});
```

---

## Test Design

All tests live in `crates/core/src/bus.rs` under `#[cfg(test)]`:

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_bus_has_no_subscribers() {
        assert_eq!(EventBus::new().subscriber_count(), 0);
    }

    #[test]
    fn publish_and_receive_sync() {
        let bus = EventBus::new();
        let mut rx = bus.subscribe();
        bus.publish(InternalEvent::SessionStarted("s1".into()));
        match rx.try_recv().unwrap() {
            InternalEvent::SessionStarted(id) => assert_eq!(id, "s1"),
            _ => panic!("wrong variant"),
        }
    }

    #[tokio::test]
    async fn publish_and_receive_async() {
        let bus = EventBus::new();
        let mut rx = bus.subscribe();
        bus.publish(InternalEvent::ServerStarted { port: 9000 });
        match rx.recv().await.unwrap() {
            InternalEvent::ServerStarted { port } => assert_eq!(port, 9000),
            _ => panic!(),
        }
    }

    #[test]
    fn late_subscriber_misses_early_events() {
        let bus = EventBus::new();
        bus.publish(InternalEvent::SessionEnded("s1".into()));
        let mut rx = bus.subscribe();
        bus.publish(InternalEvent::SessionEnded("s2".into()));
        match rx.try_recv().unwrap() {
            InternalEvent::SessionEnded(id) => assert_eq!(id, "s2"),
            _ => panic!(),
        }
    }

    #[test]
    fn session_id_returns_none_for_global_events() {
        assert!(InternalEvent::ConfigUpdated.session_id().is_none());
        assert!(InternalEvent::ServerStarted { port: 80 }.session_id().is_none());
    }

    #[test]
    fn session_id_returns_some_for_session_events() {
        let e = InternalEvent::SessionStarted("abc".into());
        assert_eq!(e.session_id(), Some("abc"));
    }

    #[test]
    fn multiple_subscribers_each_receive() {
        let bus = EventBus::new();
        let mut rx1 = bus.subscribe();
        let mut rx2 = bus.subscribe();
        bus.publish(InternalEvent::ConfigUpdated);
        assert!(rx1.try_recv().is_ok());
        assert!(rx2.try_recv().is_ok());
    }
}
```

---

## Adding New Events

1. Add a new variant to `InternalEvent` with named fields
2. Handle in the `session_id()` match arm if session-scoped
3. Update subscribers in `opencode-server` SSE handler and TUI render loop
4. Add a test in the `tests` module

```rust
// Example: adding a new event
NewToolRegistered { tool_name: String, source: String },

// In session_id():
Self::NewToolRegistered { .. } => None,
```
