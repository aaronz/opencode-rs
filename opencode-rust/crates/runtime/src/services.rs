use std::sync::Arc;

use opencode_agent::AgentRuntime;
use opencode_core::bus::SharedEventBus;
use opencode_core::permission::PermissionManager;
use opencode_permission::PermissionScope;
use opencode_storage::StorageService;
use tokio::sync::RwLock;

use crate::checkpoint::RuntimeFacadeCheckpointStore;
use crate::permission::RuntimeFacadePermissionAdapter;
use crate::task_store::RuntimeFacadeTaskStore;
use crate::tool_router::RuntimeFacadeToolRouter;
use crate::trace_store::RuntimeFacadeTraceStore;

#[derive(Clone)]
pub struct RuntimeFacadeServices {
    pub event_bus: SharedEventBus,
    pub permission_manager: Arc<RwLock<PermissionManager>>,
    pub storage: Arc<StorageService>,
    pub agent_runtime: Arc<RwLock<AgentRuntime>>,
    pub task_store: Arc<RuntimeFacadeTaskStore>,
    pub tool_router: Arc<RuntimeFacadeToolRouter>,
    pub permission_adapter: RuntimeFacadePermissionAdapter,
    pub trace_store: Arc<RuntimeFacadeTraceStore>,
    pub checkpoint_store: Arc<RuntimeFacadeCheckpointStore>,
}

impl RuntimeFacadeServices {
    pub fn agent_runtime(&self) -> Arc<RwLock<AgentRuntime>> {
        Arc::clone(&self.agent_runtime)
    }

    pub fn new(
        event_bus: SharedEventBus,
        permission_manager: Arc<RwLock<PermissionManager>>,
        storage: Arc<StorageService>,
        agent_runtime: Arc<RwLock<AgentRuntime>>,
        task_store: Arc<RuntimeFacadeTaskStore>,
        tool_router: Arc<RuntimeFacadeToolRouter>,
    ) -> Self {
        let permission_adapter = RuntimeFacadePermissionAdapter::new(
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
            trace_store: Arc::new(RuntimeFacadeTraceStore::new()),
            checkpoint_store: Arc::new(RuntimeFacadeCheckpointStore::new()),
        }
    }
}
