# Specification: Module `global` (Iteration 37)

**Crate**: `opencode-core`
**Source**: `crates/core/src/global.rs`
**Status**: Active Development
**Last Updated**: 2026-04-21

---

## 1. Overview

The `global` module provides the top-level `GlobalState` struct that owns the `Config`, `EventBus`, and active `Session`. It serves as the central dependency injection container for CLI/TUI runtime paths.

**Design Intent**: `GlobalState` is a lightweight DI root for the TUI and CLI paths (where there is a single active session at a time). The server path uses `ServerState` in `opencode-server` instead, which holds `Arc<StorageService>`, `Arc<ModelRegistry>`, etc.

---

## 2. Type Definitions

### FR-370: GlobalState Structure

```rust
/// Global state container for CLI/TUI runtime.
/// Owns Config, EventBus, and optional active Session.
pub struct GlobalState {
    /// Application configuration
    pub config: Config,
    /// Event bus for pub/sub communication
    pub event_bus: Arc<EventBus>,
    /// Currently active session (None if no session is active)
    pub current_session: Option<Session>,
}
```

### FR-371: Constructor

```rust
impl GlobalState {
    /// Creates a new GlobalState with the given configuration.
    /// EventBus is initialized with 0 subscribers.
    /// current_session is set to None.
    pub fn new(config: Config) -> Self {
        Self {
            config,
            event_bus: Arc::new(EventBus::new()),
            current_session: None,
        }
    }
}
```

### FR-372: Subscriber Count Accessor

```rust
impl GlobalState {
    /// Returns the number of subscribers currently registered with the event bus.
    pub fn subscriber_count(&self) -> usize {
        self.event_bus.subscriber_count()
    }
}
```

---

## 3. Extension Pattern (Future)

When adding new global runtime state, extend `GlobalState` following this pattern:

```rust
pub struct GlobalState {
    pub config: Config,
    pub event_bus: Arc<EventBus>,
    pub current_session: Option<Session>,
    // Future additions:
    pub tool_registry: Option<Arc<opencode_tools::ToolRegistry>>,
    pub plugin_manager: Option<Arc<opencode_lsp::LspManager>>,
}
```

---

## 4. Usage Pattern

```rust
use opencode_core::{Config, GlobalState};

// At CLI/TUI startup:
let config = Config::load().unwrap_or_default();
let global = GlobalState::new(config);

// Access event bus:
let bus = Arc::clone(&global.event_bus);

// Access config:
let model = &global.config.model;

// Set active session:
let mut state = global;
state.current_session = Some(Session::new());
```

---

## 5. Public API Export

**FR-373**: `GlobalState` must be publicly exported from `opencode-core` to allow external crates to use it.

```rust
// crates/core/src/lib.rs
pub use global::GlobalState;  // NOT pub(crate)
```

---

## 6. Test Specification

**FR-374**: `GlobalState` must include a comprehensive test module.

### FR-374.1: new_global_state_has_no_session

```rust
#[test]
fn new_global_state_has_no_session() {
    let state = GlobalState::new(Config::default());
    assert!(state.current_session.is_none());
}
```

### FR-374.2: event_bus_is_initialized

```rust
#[test]
fn event_bus_is_initialized() {
    let state = GlobalState::new(Config::default());
    // Bus should have 0 subscribers initially
    assert_eq!(state.event_bus.subscriber_count(), 0);
}
```

### FR-374.3: config_is_accessible

```rust
#[test]
fn config_is_accessible() {
    let mut config = Config::default();
    let state = GlobalState::new(config);
    let _ = &state.config;
}
```

### FR-374.4: can_set_current_session

```rust
#[test]
fn can_set_current_session() {
    let mut state = GlobalState::new(Config::default());
    state.current_session = Some(Session::new());
    assert!(state.current_session.is_some());
}
```

### FR-374.5: event_bus_is_arc_clonable

```rust
#[test]
fn event_bus_is_arc_clonable() {
    let state = GlobalState::new(Config::default());
    let bus_clone = Arc::clone(&state.event_bus);
    // Both point to same bus
    let mut rx = bus_clone.subscribe();
    state.event_bus.publish(crate::bus::InternalEvent::ConfigUpdated);
    assert!(rx.try_recv().is_ok());
}
```

---

## 7. Relationship to ServerState

| Aspect | GlobalState | ServerState |
|--------|-------------|-------------|
| Use Case | CLI/TUI (single session) | HTTP server (multi-session) |
| Location | `opencode-core` | `opencode-server` |
| Session | `Option<Session>` | `SessionHub` |
| Storage | None | `Arc<StorageService>` |
| Models | None | `Arc<ModelRegistry>` |

---

## 8. Implementation Checklist

| Requirement | ID | Status | Notes |
|-------------|----|--------|-------|
| Core structure | FR-370 | ✅ Implemented | 20 lines, minimal implementation |
| Constructor | FR-371 | ✅ Implemented | `new(config: Config) -> Self` |
| Subscriber count method | FR-372 | ⬜ Missing | Needs to be added |
| Public export | FR-373 | ⬜ Missing | Currently `pub(crate)`, needs `pub` |
| Test module | FR-374 | ⬜ Missing | 6 tests specified, none implemented |
| Doc comments | - | ⬜ Missing | Module and struct need documentation |

---

## 9. Gap Analysis Summary

| Gap | Severity | Fix Required |
|-----|----------|--------------|
| Missing test module | P0 | Add `#[cfg(test)] mod tests` with 6 tests |
| GlobalState not public | P0 | Change `pub(crate) use` to `pub use` in lib.rs |
| Missing subscriber_count() | P1 | Add `pub fn subscriber_count(&self) -> usize` |
| Session::new() validation | ✅ Verified | Exists at session.rs:137 |
| Extension pattern fields | P2 | Add when needed (tool_registry, etc.) |
| Documentation | P2 | Add doc comments |

---

## 10. Acceptance Criteria

- [ ] `GlobalState` is publicly exported from `opencode-core`
- [ ] `GlobalState::new(config)` creates instance with `current_session = None`
- [ ] `GlobalState::subscriber_count()` returns event bus subscriber count
- [ ] All 6 test cases pass (FR-374.1 through FR-374.5)
- [ ] Module has doc comments explaining purpose and usage
- [ ] `EventBus::subscriber_count()` is accessible via `GlobalState`

---

*Document Version: 37*
*Next Action: Implement missing items per gap analysis*
