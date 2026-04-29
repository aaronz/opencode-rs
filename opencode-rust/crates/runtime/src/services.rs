use std::sync::Arc;

use opencode_agent::AgentRuntime;
use opencode_core::bus::SharedEventBus;
use opencode_core::permission::PermissionManager;
use opencode_storage::StorageService;
use tokio::sync::RwLock;

use crate::task_store::RuntimeTaskStore;

#[derive(Clone)]
pub struct RuntimeServices {
    pub event_bus: SharedEventBus,
    pub permission_manager: Arc<RwLock<PermissionManager>>,
    pub storage: Arc<StorageService>,
    pub agent_runtime: Arc<RwLock<AgentRuntime>>,
    pub task_store: Arc<RuntimeTaskStore>,
}

impl RuntimeServices {
    pub fn new(
        event_bus: SharedEventBus,
        permission_manager: Arc<RwLock<PermissionManager>>,
        storage: Arc<StorageService>,
        agent_runtime: Arc<RwLock<AgentRuntime>>,
        task_store: Arc<RuntimeTaskStore>,
    ) -> Self {
        Self {
            event_bus,
            permission_manager,
            storage,
            agent_runtime,
            task_store,
        }
    }
}
