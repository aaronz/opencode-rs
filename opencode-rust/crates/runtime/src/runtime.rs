use std::sync::Arc;

use opencode_agent::AgentRuntime;
use opencode_core::events::DomainEvent;
use tokio::sync::RwLock;
use uuid::Uuid;

use crate::commands::{RuntimeFacadeCommand, TaskControlCommand};
use crate::errors::RuntimeFacadeError;
use crate::services::RuntimeFacadeServices;
use crate::task_store::RuntimeFacadeTaskStore;
use crate::types::{RuntimeFacadeResponse, RuntimeFacadeStatus, RuntimeFacadeTask, RuntimeFacadeTaskId, TaskKind};

pub struct RuntimeFacade {
    services: RuntimeFacadeServices,
    status: Arc<RwLock<RuntimeFacadeStatus>>,
}

impl RuntimeFacade {
    pub fn new(services: RuntimeFacadeServices) -> Self {
        Self {
            services,
            status: Arc::new(RwLock::new(RuntimeFacadeStatus::Idle)),
        }
    }

    pub fn handle(&self) -> RuntimeFacadeHandle {
        RuntimeFacadeHandle {
            services: self.services.clone(),
            status: Arc::clone(&self.status),
        }
    }

    pub async fn status(&self) -> RuntimeFacadeStatus {
        self.status.read().await.clone()
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
    status: Arc<RwLock<RuntimeFacadeStatus>>,
}

impl RuntimeFacadeHandle {
    pub async fn status(&self) -> RuntimeFacadeStatus {
        self.status.read().await.clone()
    }

    pub fn task_store(&self) -> Arc<RuntimeFacadeTaskStore> {
        Arc::clone(&self.services.task_store)
    }

    pub fn agent_runtime(&self) -> Arc<RwLock<AgentRuntime>> {
        Arc::clone(&self.services.agent_runtime)
    }

    pub fn clone_self(&self) -> Self {
        Self {
            services: self.services.clone(),
            status: Arc::clone(&self.status),
        }
    }

    pub async fn execute(
        &self,
        command: RuntimeFacadeCommand,
    ) -> Result<RuntimeFacadeResponse, RuntimeFacadeError> {
        // Delegate directly to this handle's Runtime by borrowing shared state
        // RuntimeFacade::execute just needs &self to access services and status
        RuntimeFacade::execute_standalone(&self.services, Arc::clone(&self.status), command).await
    }
}

impl RuntimeFacade {
    /// Execute a command using shared services without needing a full Runtime instance.
    async fn execute_standalone(
        services: &RuntimeFacadeServices,
        _status: Arc<RwLock<RuntimeFacadeStatus>>,
        command: RuntimeFacadeCommand,
    ) -> Result<RuntimeFacadeResponse, RuntimeFacadeError> {
        match command {
            RuntimeFacadeCommand::SubmitUserInput(cmd) => {
                let (session_id, turn_id) = if let Some(ref session_id) = cmd.session_id {
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
                    let session_uuid = session.id;

                    let task = RuntimeFacadeTask::new(
                        session_uuid,
                        turn_uuid.0,
                        TaskKind::Agent,
                        cmd.input,
                        None,
                    );

                    services
                        .storage
                        .save_session(&session)
                        .await
                        .map_err(|e| RuntimeFacadeError::Dependency(e.to_string()))?;

                    Self::record_completed_task(services, task).await?;

                    (session.id.to_string(), turn_uuid.0.to_string())
                } else {
                    let (session_id_str, turn_uuid, task) = services
                        .agent_runtime
                        .read()
                        .await
                        .with_session_mut(|session| {
                            let turn_id = session.start_turn(None);
                            let task = RuntimeFacadeTask::new(
                                session.id,
                                turn_id.0,
                                TaskKind::Agent,
                                cmd.input.clone(),
                                None,
                            );
                            (session.id.to_string(), turn_id, task)
                        })
                        .await;

                    Self::record_completed_task(services, task).await?;

                    (session_id_str, turn_uuid.0.to_string())
                };

                Ok(RuntimeFacadeResponse {
                    session_id: Some(session_id),
                    turn_id: Some(turn_id),
                    accepted: true,
                    message: "turn created".to_string(),
                })
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

                    services.event_bus.publish(
                        DomainEvent::TaskCancelled {
                            session_id: task.session_id.to_string(),
                            turn_id: task.turn_id.to_string(),
                            task_id: task.id.0.to_string(),
                        },
                    );

                    Ok(RuntimeFacadeResponse {
                        session_id: Some(task.session_id.to_string()),
                        turn_id: Some(task.turn_id.to_string()),
                        accepted: true,
                        message: format!("task {} cancellation requested", task_id),
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
                        })
                    } else {
                        Err(RuntimeFacadeError::Dependency(format!(
                            "approval request not found: {}",
                            cmd.request_id
                        )))
                    }
                } else {
                    if queue.reject(approval_id) {
                        tracing::info!(
                            request_id = %cmd.request_id,
                            "Permission rejected via RuntimeFacadeCommand"
                        );
                        Ok(RuntimeFacadeResponse {
                            session_id: None,
                            turn_id: None,
                            accepted: true,
                            message: format!("permission request {} rejected", cmd.request_id),
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

    async fn record_completed_task(
        services: &RuntimeFacadeServices,
        task: RuntimeFacadeTask,
    ) -> Result<(), RuntimeFacadeError> {
        let task_id = task.id;
        let session_id = task.session_id.to_string();
        let turn_id = task.turn_id.to_string();
        let task_kind = "agent".to_string();

        services.task_store.add_task(task).await;

        let started_task = services
            .task_store
            .update_task(task_id, RuntimeFacadeTask::mark_started)
            .await
            .ok_or_else(|| {
                RuntimeFacadeError::Dependency(format!(
                    "task disappeared before start: {}",
                    task_id.0
                ))
            })?;

        services.event_bus.publish(
            DomainEvent::TaskStarted {
                session_id: session_id.clone(),
                turn_id: turn_id.clone(),
                task_id: started_task.id.0.to_string(),
                task_kind,
            },
        );

        let completed_task = services
            .task_store
            .complete_task(task_id)
            .await
            .ok_or_else(|| {
                RuntimeFacadeError::Dependency(format!(
                    "task disappeared before completion: {}",
                    task_id.0
                ))
            })?;

        services.event_bus.publish(
            DomainEvent::TaskCompleted {
                session_id,
                turn_id,
                task_id: completed_task.id.0.to_string(),
            },
        );

        Ok(())
    }
}
