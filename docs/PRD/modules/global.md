# Module: global

## Overview

The `global` module in `opencode-core` (`crates/core/src/global.rs`) provides the top-level `GlobalState` struct that owns the `Config`, `EventBus`, and active `Session`. It is the central dependency injection container for the core runtime.

**Crate**: `opencode-core`  
**Source**: `crates/core/src/global.rs`  
**Status**: Minimal implementation (20 lines) — extension point for future global state

---

## Crate Layout

```
crates/core/src/
└── global.rs
```

**`crates/core/src/lib.rs`** exports:
```rust
pub mod global;
pub use global::GlobalState;
```

---

## Core Type

```rust
use crate::bus::EventBus;
use crate::config::Config;
use crate::session::Session;
use std::sync::Arc;

pub struct GlobalState {
    pub config: Config,
    pub event_bus: Arc<EventBus>,
    pub current_session: Option<Session>,
}

impl GlobalState {
    pub fn new(config: Config) -> Self {
        Self {
            config,
            event_bus: Arc::new(EventBus::new()),
            current_session: None,
        }
    }
}
```

---

## Design Intent

`GlobalState` is a lightweight DI root for the TUI and CLI paths (where there is a single active session at a time). The server path uses `ServerState` in `opencode-server` instead, which holds `Arc<StorageService>`, `Arc<ModelRegistry>`, etc.

For multi-session use (HTTP server), prefer `ServerState`. For single-session CLI/TUI, use `GlobalState`.

---

## Extension Pattern

When adding new global runtime state, extend `GlobalState`:

```rust
pub struct GlobalState {
    pub config: Config,
    pub event_bus: Arc<EventBus>,
    pub current_session: Option<Session>,
    // Future additions:
    pub tool_registry: Option<Arc<opencode_tools::ToolRegistry>>,
    pub plugin_manager: Option<Arc<tokio::sync::RwLock<opencode_plugin::PluginManager>>>,
    pub lsp_manager: Option<Arc<opencode_lsp::LspManager>>,
}
```

---

## Usage Pattern

```rust
use opencode_core::{Config, GlobalState};

// At CLI/TUI startup:
let config = Config::load().unwrap_or_default();
let global = GlobalState::new(config);

// Access:
let bus = Arc::clone(&global.event_bus);
let model = &global.config.model;

// Set active session:
let mut state = global;
state.current_session = Some(Session::new());
```

---

## Relationship to `ServerState`

`ServerState` (in `opencode-server/src/lib.rs`) is the heavier equivalent for HTTP server mode:

```rust
pub struct ServerState {
    pub storage: Arc<StorageService>,
    pub models: Arc<ModelRegistry>,
    pub config: Arc<RwLock<Config>>,
    pub event_bus: SharedEventBus,
    pub tool_registry: Arc<ToolRegistry>,
    pub session_hub: Arc<SessionHub>,
    pub permission_manager: Arc<RwLock<PermissionManager>>,
    pub approval_queue: Arc<RwLock<ApprovalQueue>>,
    pub audit_log: Option<Arc<AuditLog>>,
    // ... more fields
}
```

`GlobalState` is a simpler, non-`Arc`-wrapped alternative for single-process CLI use.

---

## Test Design

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::Config;

    #[test]
    fn new_global_state_has_no_session() {
        let state = GlobalState::new(Config::default());
        assert!(state.current_session.is_none());
    }

    #[test]
    fn event_bus_is_initialized() {
        let state = GlobalState::new(Config::default());
        // Bus should have 0 subscribers initially
        assert_eq!(state.event_bus.subscriber_count(), 0);
    }

    #[test]
    fn config_is_accessible() {
        let mut config = Config::default();
        // Config fields accessible
        let state = GlobalState::new(config);
        // state.config is accessible
        let _ = &state.config;
    }

    #[test]
    fn can_set_current_session() {
        use crate::session::Session;
        let mut state = GlobalState::new(Config::default());
        state.current_session = Some(Session::new());
        assert!(state.current_session.is_some());
    }

    #[test]
    fn event_bus_is_arc_clonable() {
        let state = GlobalState::new(Config::default());
        let bus_clone = Arc::clone(&state.event_bus);
        // Both point to same bus
        let mut rx = bus_clone.subscribe();
        state.event_bus.publish(crate::bus::InternalEvent::ConfigUpdated);
        assert!(rx.try_recv().is_ok());
    }
}
```
