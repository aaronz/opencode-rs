use std::sync::Arc;

use async_trait::async_trait;
use opencode_agent::{Agent, AgentRuntime};
use opencode_core::events::DomainEvent;
use opencode_llm::Provider;
use tokio::sync::RwLock;
use uuid::Uuid;

use crate::commands::{ContextCommand, RuntimeFacadeCommand, TaskControlCommand};
use crate::context_view::RuntimeFacadeContextSummary;
use crate::errors::RuntimeFacadeError;
use crate::handle::RuntimeHandle;
use crate::services::RuntimeFacadeServices;
use crate::task_store::RuntimeFacadeTaskStore;
use crate::types::{
    RuntimeFacadeResponse, RuntimeFacadeTask, RuntimeFacadeTaskId, RuntimeStatus, TaskKind,
};

pub struct RuntimeFacade {
    services: RuntimeFacadeServices,
    status: Arc<RwLock<RuntimeStatus>>,
}

impl RuntimeFacade {
    pub fn new(services: RuntimeFacadeServices) -> Self {
        Self {
            services,
            status: Arc::new(RwLock::new(RuntimeStatus::Idle)),
        }
    }

    pub fn handle(&self) -> RuntimeFacadeHandle {
        RuntimeFacadeHandle {
            services: self.services.clone(),
            status: Arc::clone(&self.status),
        }
    }

    pub async fn set_provider(&self, provider: Arc<dyn Provider + Send + Sync>) {
        self.services.set_provider(provider).await;
    }

    pub async fn status(&self) -> RuntimeStatus {
        *self.status.read().await
    }

    pub fn task_store(&self) -> Arc<RuntimeFacadeTaskStore> {
        Arc::clone(&self.services.task_store)
    }

    pub async fn execute(
        &self,
        command: RuntimeFacadeCommand,
    ) -> Result<RuntimeFacadeResponse, RuntimeFacadeError> {
        RuntimeFacade::execute_standalone(&self.services, Arc::clone(&self.status), command).await
    }
}

#[derive(Clone)]
pub struct RuntimeFacadeHandle {
    services: RuntimeFacadeServices,
    status: Arc<RwLock<RuntimeStatus>>,
}

impl RuntimeFacadeHandle {
    pub async fn status(&self) -> RuntimeStatus {
        *self.status.read().await
    }

    pub fn task_store(&self) -> Arc<RuntimeFacadeTaskStore> {
        Arc::clone(&self.services.task_store)
    }

    pub fn agent_runtime(&self) -> Arc<RwLock<AgentRuntime>> {
        Arc::clone(&self.services.agent_runtime)
    }

    pub async fn set_provider(&self, provider: Arc<dyn Provider + Send + Sync>) {
        self.services.set_provider(provider).await;
    }

    pub fn clone_self(&self) -> Self {
        Self {
            services: self.services.clone(),
            status: Arc::clone(&self.status),
        }
    }

    pub async fn execute_with_status_tracking(
        &self,
        command: RuntimeFacadeCommand,
    ) -> Result<RuntimeFacadeResponse, RuntimeFacadeError> {
        let from_status = *self.status.read().await;
        let from_status_str = format!("{:?}", from_status);

        let result = RuntimeFacade::execute_standalone(&self.services, Arc::clone(&self.status), command).await;

        let to_status = *self.status.read().await;
        let to_status_str = format!("{:?}", to_status);

        if from_status != to_status {
            self.services.event_bus.publish(DomainEvent::RuntimeStatusChanged {
                session_id: None,
                from_status: from_status_str,
                to_status: to_status_str,
            });
        }

        result
    }
}

#[async_trait]
impl RuntimeHandle for RuntimeFacadeHandle {
    async fn execute(
        &self,
        command: RuntimeFacadeCommand,
    ) -> Result<RuntimeFacadeResponse, RuntimeFacadeError> {
        RuntimeFacade::execute_standalone(&self.services, Arc::clone(&self.status), command).await
    }

    async fn status(&self) -> RuntimeStatus {
        *self.status.read().await
    }

    async fn set_provider(&self, provider: Arc<dyn Provider + Send + Sync>) {
        self.services.set_provider(provider).await;
    }

    fn subscribe(&self) -> tokio::sync::broadcast::Receiver<DomainEvent> {
        self.services.event_bus.subscribe()
    }
}

impl RuntimeFacade {
    /// Execute a command using shared services without needing a full Runtime instance.
    async fn execute_standalone(
        services: &RuntimeFacadeServices,
        _status: Arc<RwLock<RuntimeStatus>>,
        command: RuntimeFacadeCommand,
    ) -> Result<RuntimeFacadeResponse, RuntimeFacadeError> {
        match command {
            RuntimeFacadeCommand::SubmitUserInput(cmd) => {
                let (session, turn_uuid, _task_id, session_id_str) =
                    if let Some(ref session_id) = cmd.session_id {
                        let mut session = services
                            .storage
                            .load_session(session_id)
                            .await
                            .map_err(|e| RuntimeFacadeError::Dependency(e.to_string()))?
                            .ok_or_else(|| {
                                RuntimeFacadeError::Dependency(format!(
                                    "session not found: {session_id}"
                                ))
                            })?;

                        let turn_uuid = session.start_turn(None);
                        let task_id = RuntimeFacadeTaskId(Uuid::new_v4());

                        services
                            .storage
                            .save_session(&session)
                            .await
                            .map_err(|e| RuntimeFacadeError::Dependency(e.to_string()))?;

                        (session, turn_uuid.0, task_id, session_id.clone())
                    } else {
                        let (session, turn_uuid, task_id) = services
                            .agent_runtime
                            .read()
                            .await
                            .with_session_mut(|session| {
                                let turn_id = session.start_turn(None);
                                let task_id = RuntimeFacadeTaskId(Uuid::new_v4());
                                (session.clone(), turn_id, task_id)
                            })
                            .await;

                        (session, turn_uuid.0, task_id, task_id.0.to_string())
                    };

                let turn_id_str = turn_uuid.to_string();
                let session_id_for_task =
                    Uuid::parse_str(&session_id_str).unwrap_or(Uuid::new_v4());

                let task = RuntimeFacadeTask::new(
                    session_id_for_task,
                    turn_uuid,
                    TaskKind::Agent,
                    cmd.input,
                    None,
                );

                let task_id_val = task.id;

                services.task_store.add_task(task.clone()).await;

                let started_task = services
                    .task_store
                    .update_task(task_id_val, RuntimeFacadeTask::mark_started)
                    .await
                    .ok_or_else(|| {
                        RuntimeFacadeError::Dependency(format!(
                            "task disappeared before start: {}",
                            task_id_val.0
                        ))
                    })?;

                services.event_bus.publish(DomainEvent::TaskStarted {
                    session_id: session_id_str.clone(),
                    turn_id: turn_id_str.clone(),
                    task_id: started_task.id.0.to_string(),
                    task_kind: "agent".to_string(),
                });

                let runtime = AgentRuntime::new(session, services.agent_type)
                    .with_event_bus(services.event_bus.clone());
                let provider = services.provider.read().await;
                let provider = provider.as_ref().map(|p| p.as_ref());
                let tools = services.tools.as_ref().map(|t| t.as_ref());
                let agent_ref = services.agent.as_ref().as_ref();

                let result = if let (Some(p), Some(t)) = (provider, tools) {
                    runtime.run_loop_streaming(agent_ref, p, t, None).await
                } else {
                    tracing::error!("Provider or tools not available");
                    Err(opencode_agent::RuntimeError::ToolExecutionFailed {
                        tool: "agent".to_string(),
                        reason: "provider or tools not available".to_string(),
                    })
                };

                match result {
                    Ok(_response) => {
                        tracing::info!(
                            session_id = %session_id_str,
                            "Agent loop completed successfully"
                        );

                        if let Some(completed) = services
                            .task_store
                            .update_task(task_id_val, RuntimeFacadeTask::mark_completed)
                            .await
                        {
                            services.event_bus.publish(DomainEvent::TaskCompleted {
                                session_id: session_id_str.clone(),
                                turn_id: turn_id_str.clone(),
                                task_id: completed.id.0.to_string(),
                            });
                        }
                    }
                    Err(e) => {
                        tracing::error!(
                            session_id = %session_id_str,
                            error = %e,
                            "Agent loop failed"
                        );

                        if let Some(failed) = services
                            .task_store
                            .update_task(task_id_val, RuntimeFacadeTask::mark_completed)
                            .await
                        {
                            services.event_bus.publish(DomainEvent::TaskCompleted {
                                session_id: session_id_str.clone(),
                                turn_id: turn_id_str.clone(),
                                task_id: failed.id.0.to_string(),
                            });
                        }
                    }
                }

                Ok(RuntimeFacadeResponse {
                    session_id: Some(session_id_str),
                    turn_id: Some(turn_id_str),
                    accepted: true,
                    message: "turn created".to_string(),
                    session: None,
                })
            }
            RuntimeFacadeCommand::RunAgent(cmd) => {
                let agent: Box<dyn Agent> = match cmd.agent_type {
                    opencode_agent::AgentType::Build => Box::new(opencode_agent::BuildAgent::new()),
                    opencode_agent::AgentType::Explore => {
                        Box::new(opencode_agent::ExploreAgent::new())
                    }
                    opencode_agent::AgentType::Debug => Box::new(opencode_agent::DebugAgent::new()),
                    opencode_agent::AgentType::Plan => Box::new(opencode_agent::PlanAgent::new()),
                    opencode_agent::AgentType::Refactor => {
                        Box::new(opencode_agent::RefactorAgent::new())
                    }
                    opencode_agent::AgentType::Review => {
                        Box::new(opencode_agent::ReviewAgent::new())
                    }
                    opencode_agent::AgentType::General => {
                        Box::new(opencode_agent::GeneralAgent::new())
                    }
                    _ => Box::new(opencode_agent::BuildAgent::new()),
                };

                let runtime = AgentRuntime::new(cmd.session, cmd.agent_type)
                    .with_event_bus(services.event_bus.clone());
                let provider = services.provider.read().await;
                let provider = provider.as_ref().map(|p| p.as_ref());
                let tools = services.tools.as_ref().map(|t| t.as_ref());

                if let (Some(p), Some(t)) = (provider, tools) {
                    let final_session = runtime.run_loop_streaming(&*agent, p, t, None).await;
                    match final_session {
                        Ok(_) => {
                            let updated_session = runtime.session().await;
                            Ok(RuntimeFacadeResponse {
                                session_id: None,
                                turn_id: None,
                                accepted: true,
                                message: "agent completed".to_string(),
                                session: Some(updated_session),
                            })
                        }
                        Err(e) => Err(RuntimeFacadeError::Dependency(e.to_string())),
                    }
                } else {
                    Err(RuntimeFacadeError::Dependency(
                        "provider or tools not available".to_string(),
                    ))
                }
            }
            RuntimeFacadeCommand::ExecuteShell(cmd) => {
                let args = serde_json::json!({
                    "command": cmd.command,
                    "timeout": cmd.timeout_secs,
                    "workdir": cmd.workdir,
                });

                match services.tool_router.execute_with_validation("bash", args, None).await {
                    Ok(result) => Ok(RuntimeFacadeResponse {
                        session_id: None,
                        turn_id: None,
                        accepted: true,
                        message: result.content,
                        session: None,
                    }),
                    Err(e) => Err(RuntimeFacadeError::Dependency(e.to_string())),
                }
            }
            RuntimeFacadeCommand::Context(cmd) => {
                let session_id = match &cmd {
                    ContextCommand::Inspect { session_id } => {
                        session_id.clone().unwrap_or_default()
                    }
                    ContextCommand::Explain { session_id } => {
                        session_id.clone().unwrap_or_default()
                    }
                    ContextCommand::Dump { session_id, .. } => {
                        session_id.clone().unwrap_or_default()
                    }
                    ContextCommand::Why { session_id, .. } => {
                        session_id.clone().unwrap_or_default()
                    }
                };

                let session = if session_id.is_empty() {
                    services
                        .storage
                        .load_session(&session_id)
                        .await
                        .map_err(|e| RuntimeFacadeError::Dependency(e.to_string()))?
                        .ok_or_else(|| {
                            RuntimeFacadeError::Dependency("session not found".to_string())
                        })?
                } else {
                    services.agent_runtime.read().await.session().await
                };

                let context = session.build_context();
                let summary = RuntimeFacadeContextSummary::from_context(&context);

                match cmd {
                    ContextCommand::Inspect { .. } => {
                        Ok(RuntimeFacadeResponse {
                            session_id: Some(session.id.to_string()),
                            turn_id: None,
                            accepted: true,
                            message: format!(
                                "Context Summary:\n  Total tokens: {}/{} ({:.1}%)\n  Layers: {}\n  Files: {}\n  Tools: {}\n  Session messages: {}\n  Prompt messages: {}",
                                summary.total_tokens,
                                summary.max_tokens,
                                summary.usage_pct * 100.0,
                                summary.layer_count,
                                summary.file_count,
                                summary.tool_count,
                                summary.session_count,
                                summary.prompt_message_count
                            ),
                            session: None,
                        })
                    }
                    ContextCommand::Explain { .. } => {
                        Ok(RuntimeFacadeResponse {
                            session_id: Some(session.id.to_string()),
                            turn_id: None,
                            accepted: true,
                            message: "Context ranking explanation:\n  - Recency (40%): newer messages have higher priority\n  - Relevance (30%): messages relevant to current task\n  - Importance (30%): importance score based on role/content".to_string(),
                            session: None,
                        })
                    }
                    ContextCommand::Dump { turn_id, .. } => {
                        Ok(RuntimeFacadeResponse {
                            session_id: Some(session.id.to_string()),
                            turn_id: Some(turn_id.clone()),
                            accepted: true,
                            message: format!(
                                "Context for turn {}:\n  Files: {:?}\n  Tools: {:?}\n  Sessions: {:?}",
                                turn_id, context.file_context, context.tool_context, context.session_context
                            ),
                            session: None,
                        })
                    }
                    ContextCommand::Why { file, .. } => {
                        let in_context = context.file_context.iter().any(|f| f.contains(file.as_str()));
                        Ok(RuntimeFacadeResponse {
                            session_id: Some(session.id.to_string()),
                            turn_id: None,
                            accepted: true,
                            message: if in_context {
                                format!("File '{}' is in context (referenced in conversation or opened)", file)
                            } else {
                                format!("File '{}' is not in context", file)
                            },
                            session: None,
                        })
                    }
                }
            }
            RuntimeFacadeCommand::TaskControl(cmd) => match cmd {
                TaskControlCommand::Cancel { task_id } => {
                    let task_uuid = Uuid::parse_str(&task_id).map_err(|_| {
                        RuntimeFacadeError::Dependency(format!("invalid task id: {task_id}"))
                    })?;
                    let rt_task_id = RuntimeFacadeTaskId(task_uuid);

                    let _task = services
                        .task_store
                        .cancel_task(rt_task_id)
                        .await
                        .ok_or_else(|| {
                            RuntimeFacadeError::Dependency(format!("task not found: {task_id}"))
                        })?;

                    let task = services
                        .task_store
                        .update_task(rt_task_id, RuntimeFacadeTask::mark_cancelled)
                        .await
                        .ok_or_else(|| {
                            RuntimeFacadeError::Dependency(format!(
                                "task not found after cancellation: {task_id}"
                            ))
                        })?;

                    services.event_bus.publish(DomainEvent::TaskCancelled {
                        session_id: task.session_id.to_string(),
                        turn_id: task.turn_id.to_string(),
                        task_id: task.id.0.to_string(),
                    });

                    Ok(RuntimeFacadeResponse {
                        session_id: Some(task.session_id.to_string()),
                        turn_id: Some(task.turn_id.to_string()),
                        accepted: true,
                        message: format!("task {} cancellation requested", task_id),
                        session: None,
                    })
                }
            },
            RuntimeFacadeCommand::PermissionResponse(cmd) => {
                let approval_id = Uuid::parse_str(&cmd.request_id).map_err(|_| {
                    RuntimeFacadeError::Dependency(format!(
                        "invalid approval request id: {}",
                        cmd.request_id
                    ))
                })?;

                let approval_queue = services.permission_adapter.approval_queue();
                let mut queue = approval_queue.write().await;

                if cmd.granted {
                    if let Some(approved) = queue.approve(approval_id) {
                        tracing::info!(
                            session_id = %approved.session_id,
                            tool_name = %approved.tool_name,
                            "Permission approved via RuntimeFacadeCommand"
                        );
                        Ok(RuntimeFacadeResponse {
                            session_id: Some(approved.session_id.to_string()),
                            turn_id: None,
                            accepted: true,
                            message: format!("permission for '{}' approved", approved.tool_name),
                            session: None,
                        })
                    } else {
                        Err(RuntimeFacadeError::Dependency(format!(
                            "approval request not found: {}",
                            cmd.request_id
                        )))
                    }
                } else if queue.reject(approval_id) {
                    tracing::info!(
                        request_id = %cmd.request_id,
                        "Permission rejected via RuntimeFacadeCommand"
                    );
                    Ok(RuntimeFacadeResponse {
                        session_id: None,
                        turn_id: None,
                        accepted: true,
                        message: format!("permission request {} rejected", cmd.request_id),
                        session: None,
                    })
                } else {
                    Err(RuntimeFacadeError::Dependency(format!(
                        "approval request not found: {}",
                        cmd.request_id
                    )))
                }
            }
        }
    }
}
