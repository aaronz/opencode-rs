use std::sync::Arc;
use crate::config::Config;
use crate::session::Session;
use crate::bus::EventBus;

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
