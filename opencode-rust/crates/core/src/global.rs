//! Global state container for CLI/TUI runtime.
//!
//! This module provides [`GlobalState`], a lightweight dependency injection container
//! for the TUI and CLI runtime paths. It owns the application [`Config`], an
//! [`EventBus`] for pub/sub communication, and an optional active [`Session`].
//!
//! **Note**: The server path uses `ServerState` in `opencode-server` instead,
//! which holds `Arc<StorageService>`, `Arc<ModelRegistry>`, etc.
//!
//! # Example
//!
//! ```rust
//! use opencode_core::{Config, GlobalState};
//!
//! // At CLI/TUI startup:
//! let config = Config::default();
//! let global = GlobalState::new(config);
//!
//! // Access event bus:
//! let bus = std::sync::Arc::clone(&global.event_bus);
//!
//! // Access config:
//! let model = &global.config.model;
//!
//! // Set active session:
//! let mut state = global;
//! state.current_session = Some(opencode_core::Session::new());
//! ```
//!
//! # Extension Pattern
//!
//! When adding new global runtime state, extend [`GlobalState`] following this pattern:
//!
//! ```rust,ignore
//! pub struct GlobalState {
//!     pub config: Config,
//!     pub event_bus: Arc<EventBus>,
//!     pub current_session: Option<Session>,
//!     // Future additions:
//!     pub tool_registry: Option<Arc<opencode_tools::ToolRegistry>>,
//!     pub plugin_manager: Option<Arc<opencode_lsp::LspManager>>,
//! }
//! ```

use crate::bus::EventBus;
use crate::config::Config;
use crate::flag::FlagManager;
use crate::session::Session;
use std::sync::Arc;

/// Global state container for CLI/TUI runtime.
///
/// Owns [`Config`], [`EventBus`], [`FlagManager`], and optional active [`Session`].
pub struct GlobalState {
    pub config: Config,
    pub event_bus: Arc<EventBus>,
    pub flag_manager: FlagManager,
    pub current_session: Option<Session>,
}

impl GlobalState {
    pub fn new(config: Config) -> Self {
        let mut flag_manager = FlagManager::new();
        flag_manager.load_from_env();
        Self {
            config,
            event_bus: Arc::new(EventBus::new()),
            flag_manager,
            current_session: None,
        }
    }

    pub fn subscriber_count(&self) -> usize {
        self.event_bus.subscriber_count()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_global_state_has_no_session() {
        let state = GlobalState::new(Config::default());
        assert!(state.current_session.is_none());
    }

    #[test]
    fn event_bus_is_initialized() {
        let state = GlobalState::new(Config::default());
        assert_eq!(state.event_bus.subscriber_count(), 0);
    }

    #[test]
    fn subscriber_count_returns_correct_count() {
        let state = GlobalState::new(Config::default());
        assert_eq!(state.subscriber_count(), 0);
    }

    #[test]
    fn config_is_accessible() {
        let config = Config::default();
        let state = GlobalState::new(config);
        let _ = &state.config;
    }

    #[test]
    fn can_set_current_session() {
        let mut state = GlobalState::new(Config::default());
        state.current_session = Some(Session::new());
        assert!(state.current_session.is_some());
    }

    #[test]
    fn event_bus_is_arc_clonable() {
        let state = GlobalState::new(Config::default());
        let bus_clone = Arc::clone(&state.event_bus);
        let mut rx = bus_clone.subscribe();
        state
            .event_bus
            .publish(crate::bus::InternalEvent::ConfigUpdated);
        assert!(rx.try_recv().is_ok());
    }

    #[test]
    fn flag_manager_is_initialized() {
        let state = GlobalState::new(Config::default());
        assert!(state.flag_manager.get("OPENCODE_EXPERIMENTAL").is_some());
    }
}
