use std::sync::Arc;

use opencode_agent::AgentRuntime;
use opencode_core::bus::SharedEventBus;
use opencode_core::permission::PermissionManager;
use opencode_permission::PermissionScope;
use opencode_storage::StorageService;
use tokio::sync::RwLock;

use crate::checkpoint::RuntimeCheckpointStore;
use crate::permission::RuntimePermissionAdapter;
use crate::task_store::RuntimeTaskStore;
use crate::tool_router::RuntimeToolRouter;
use crate::trace_store::RuntimeTraceStore;

#[derive(Clone)]
pub struct RuntimeServices {
    pub event_bus: SharedEventBus,
    pub permission_manager: Arc<RwLock<PermissionManager>>,
    pub storage: Arc<StorageService>,
    pub agent_runtime: Arc<RwLock<AgentRuntime>>,
    pub task_store: Arc<RuntimeTaskStore>,
    pub tool_router: Arc<RuntimeToolRouter>,
    pub permission_adapter: RuntimePermissionAdapter,
    pub trace_store: Arc<RuntimeTraceStore>,
    pub checkpoint_store: Arc<RuntimeCheckpointStore>,
}

impl RuntimeServices {
    pub fn new(
        event_bus: SharedEventBus,
        permission_manager: Arc<RwLock<PermissionManager>>,
        storage: Arc<StorageService>,
        agent_runtime: Arc<RwLock<AgentRuntime>>,
        task_store: Arc<RuntimeTaskStore>,
        tool_router: Arc<RuntimeToolRouter>,
    ) -> Self {
        let permission_adapter = RuntimePermissionAdapter::new(
            Arc::clone(&permission_manager),
            Arc::new(RwLock::new(opencode_permission::ApprovalQueue::new(
                PermissionScope::default(),
            ))),
            None,
        );

        Self {
            event_bus,
            permission_manager,
            storage,
            agent_runtime,
            task_store,
            tool_router,
            permission_adapter,
            trace_store: Arc::new(RuntimeTraceStore::new()),
            checkpoint_store: Arc::new(RuntimeCheckpointStore::new()),
        }
    }
}
