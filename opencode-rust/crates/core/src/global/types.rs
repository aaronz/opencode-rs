use crate::bus::EventBus;
use crate::config::Config;
use crate::flag::types::FlagManager;
use crate::session::Session;
use std::sync::Arc;

#[allow(private_interfaces)]
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
