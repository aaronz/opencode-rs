use std::sync::Arc;

use opencode_agent::{Agent, AgentRuntime, AgentType, BuildAgent};
use opencode_core::bus::SharedEventBus;
use opencode_core::hook::{HookEngine, HookEvent, HookPoint};
use opencode_core::permission::PermissionManager;
use opencode_core::Session;
use opencode_llm::Provider;
use opencode_permission::PermissionScope;
use opencode_storage::StorageService;
use opencode_tools::ToolRegistry;
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
    pub hook_engine: Arc<HookEngine>,
    pub agent_type: AgentType,
    pub agent: Arc<Box<dyn Agent>>,
    pub provider: Arc<RwLock<Option<Arc<dyn Provider + Send + Sync>>>>,
    pub tools: Option<Arc<ToolRegistry>>,
}

impl RuntimeFacadeServices {
    pub fn agent_runtime(&self) -> Arc<RwLock<AgentRuntime>> {
        Arc::clone(&self.agent_runtime)
    }

    pub fn create_agent_runtime(
        session: Session,
        agent_type: AgentType,
        event_bus: SharedEventBus,
    ) -> Arc<RwLock<AgentRuntime>> {
        let runtime = AgentRuntime::new(session, agent_type).with_event_bus(event_bus);
        Arc::new(RwLock::new(runtime))
    }

    fn create_agent(agent_type: AgentType) -> Arc<Box<dyn Agent>> {
        let agent: Box<dyn Agent> = match agent_type {
            AgentType::Build => Box::new(BuildAgent::new()),
            AgentType::Explore => Box::new(opencode_agent::ExploreAgent::new()),
            AgentType::Debug => Box::new(opencode_agent::DebugAgent::new()),
            AgentType::Plan => Box::new(opencode_agent::PlanAgent::new()),
            AgentType::Refactor => Box::new(opencode_agent::RefactorAgent::new()),
            AgentType::Review => Box::new(opencode_agent::ReviewAgent::new()),
            AgentType::General => Box::new(opencode_agent::GeneralAgent::new()),
            _ => Box::new(BuildAgent::new()),
        };
        Arc::new(agent)
    }

    #[allow(clippy::too_many_arguments)]
    pub fn new(
        event_bus: SharedEventBus,
        permission_manager: Arc<RwLock<PermissionManager>>,
        storage: Arc<StorageService>,
        agent_runtime: Arc<RwLock<AgentRuntime>>,
        task_store: Arc<RuntimeFacadeTaskStore>,
        tool_router: Arc<RuntimeFacadeToolRouter>,
        agent_type: AgentType,
        provider: Option<Arc<dyn Provider + Send + Sync>>,
        tools: Option<Arc<ToolRegistry>>,
    ) -> Self {
        Self::with_audit_log(
            event_bus,
            permission_manager,
            storage,
            agent_runtime,
            task_store,
            tool_router,
            agent_type,
            provider,
            tools,
            None,
        )
    }

    #[allow(clippy::too_many_arguments)]
    pub fn with_audit_log(
        event_bus: SharedEventBus,
        permission_manager: Arc<RwLock<PermissionManager>>,
        storage: Arc<StorageService>,
        agent_runtime: Arc<RwLock<AgentRuntime>>,
        task_store: Arc<RuntimeFacadeTaskStore>,
        tool_router: Arc<RuntimeFacadeToolRouter>,
        agent_type: AgentType,
        provider: Option<Arc<dyn Provider + Send + Sync>>,
        tools: Option<Arc<ToolRegistry>>,
        audit_log_path: Option<std::path::PathBuf>,
    ) -> Self {
        let approval_queue = Arc::new(RwLock::new(opencode_permission::ApprovalQueue::new(
            PermissionScope::from_env(),
        )));

        let audit_log = audit_log_path.and_then(|path| {
            opencode_permission::AuditLog::new(path)
                .inspect_err(|e| tracing::warn!(error = %e, "Failed to create audit log"))
                .ok()
                .map(Arc::new)
        });

        let permission_adapter = RuntimeFacadePermissionAdapter::new(
            Arc::clone(&permission_manager),
            approval_queue,
            audit_log,
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
            hook_engine: Arc::new(HookEngine::new()),
            agent_type,
            agent: Self::create_agent(agent_type),
            provider: Arc::new(RwLock::new(provider)),
            tools,
        }
    }

    pub async fn set_provider(&self, provider: Arc<dyn Provider + Send + Sync>) {
        *self.provider.write().await = Some(provider);
    }

    pub async fn trigger_hook(&self, point: HookPoint, event: HookEvent) {
        self.hook_engine.trigger(point, event).await;
    }
}
