use std::sync::Arc;

use opencode_agent::AgentRuntime;
use opencode_core::bus::SharedEventBus;
use opencode_core::permission::PermissionManager;
use opencode_storage::StorageService;
use tokio::sync::RwLock;

#[derive(Clone)]
pub struct RuntimeServices {
    pub event_bus: SharedEventBus,
    pub permission_manager: Arc<RwLock<PermissionManager>>,
    pub storage: Arc<StorageService>,
    pub agent_runtime: Arc<RwLock<AgentRuntime>>,
}

impl RuntimeServices {
    pub fn new(
        event_bus: SharedEventBus,
        permission_manager: Arc<RwLock<PermissionManager>>,
        storage: Arc<StorageService>,
        agent_runtime: Arc<RwLock<AgentRuntime>>,
    ) -> Self {
        Self {
            event_bus,
            permission_manager,
            storage,
            agent_runtime,
        }
    }
}
