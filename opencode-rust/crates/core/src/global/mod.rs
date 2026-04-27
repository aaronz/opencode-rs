mod types;
pub use types::GlobalState;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::Config;
    use crate::session::Session;
    use std::sync::Arc;

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
