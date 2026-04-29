use std::sync::Arc;

use tokio::sync::RwLock;
use uuid::Uuid;

use crate::commands::{RuntimeCommand, TaskControlCommand};
use crate::errors::RuntimeFacadeError;
use crate::events::RuntimeEvent;
use crate::services::RuntimeServices;
use crate::task_store::RuntimeTaskStore;
use crate::types::{RuntimeResponse, RuntimeStatus, RuntimeTask, RuntimeTaskId, TaskKind};

pub struct Runtime {
    services: RuntimeServices,
    status: Arc<RwLock<RuntimeStatus>>,
}

impl Runtime {
    pub fn new(services: RuntimeServices) -> Self {
        Self {
            services,
            status: Arc::new(RwLock::new(RuntimeStatus::Idle)),
        }
    }

    pub fn handle(&self) -> RuntimeHandle {
        RuntimeHandle {
            services: self.services.clone(),
            status: Arc::clone(&self.status),
        }
    }

    pub async fn status(&self) -> RuntimeStatus {
        self.status.read().await.clone()
    }

    pub fn task_store(&self) -> Arc<RuntimeTaskStore> {
        Arc::clone(&self.services.task_store)
    }

    async fn emit_task_event(&self, event: RuntimeEvent) {
        self.services.event_bus.publish(event.into());
    }

    pub async fn execute(
        &self,
        command: RuntimeCommand,
    ) -> Result<RuntimeResponse, RuntimeFacadeError> {
        match command {
            RuntimeCommand::SubmitUserInput(cmd) => {
                let (session_id, turn_id) = if let Some(session_id) = cmd.session_id {
                    let mut session = self
                        .services
                        .storage
                        .load_session(&session_id)
                        .await
                        .map_err(|e| RuntimeFacadeError::Dependency(e.to_string()))?
                        .ok_or_else(|| {
                            RuntimeFacadeError::Dependency(format!(
                                "session not found: {session_id}"
                            ))
                        })?;

                    let turn_uuid = session.start_turn(None);
                    let session_uuid = session.id;

                    let task = RuntimeTask::new(
                        session_uuid,
                        turn_uuid.0,
                        TaskKind::Agent,
                        cmd.input,
                        None,
                    );
                    let task_id_str = task.id.0.to_string();
                    self.services.task_store.add_task(task).await;

                    self.emit_task_event(RuntimeEvent::TaskStarted {
                        session_id: session_uuid.to_string(),
                        turn_id: turn_uuid.0.to_string(),
                        task_id: task_id_str,
                        task_kind: "agent".to_string(),
                    })
                    .await;

                    self.services
                        .storage
                        .save_session(&session)
                        .await
                        .map_err(|e| RuntimeFacadeError::Dependency(e.to_string()))?;

                    (session.id.to_string(), turn_uuid.0.to_string())
                } else {
                    let (session_id_str, turn_uuid, task) = self
                        .services
                        .agent_runtime
                        .read()
                        .await
                        .with_session_mut(|session| {
                            let turn_id = session.start_turn(None);
                            let task = RuntimeTask::new(
                                session.id,
                                turn_id.0,
                                TaskKind::Agent,
                                cmd.input.clone(),
                                None,
                            );
                            (session.id.to_string(), turn_id, task)
                        })
                        .await;

                    let task_id_str = task.id.0.to_string();
                    self.services.task_store.add_task(task).await;

                    self.emit_task_event(RuntimeEvent::TaskStarted {
                        session_id: session_id_str.clone(),
                        turn_id: turn_uuid.0.to_string(),
                        task_id: task_id_str,
                        task_kind: "agent".to_string(),
                    })
                    .await;

                    (session_id_str, turn_uuid.0.to_string())
                };

                Ok(RuntimeResponse {
                    session_id: Some(session_id),
                    turn_id: Some(turn_id),
                    accepted: true,
                    message: "turn created".to_string(),
                })
            }
            RuntimeCommand::TaskControl(cmd) => self.execute_task_control(cmd).await,
            RuntimeCommand::PermissionResponse(_) => {
                Err(RuntimeFacadeError::NotImplemented("permission response"))
            }
        }
    }

    async fn execute_task_control(
        &self,
        cmd: TaskControlCommand,
    ) -> Result<RuntimeResponse, RuntimeFacadeError> {
        match cmd {
            TaskControlCommand::Cancel { task_id } => {
                let task_uuid = Uuid::parse_str(&task_id).map_err(|_| {
                    RuntimeFacadeError::Dependency(format!("invalid task id: {task_id}"))
                })?;
                let rt_task_id = RuntimeTaskId(task_uuid);

                let task = self
                    .services
                    .task_store
                    .cancel_task(rt_task_id)
                    .await
                    .ok_or_else(|| {
                        RuntimeFacadeError::Dependency(format!("task not found: {task_id}"))
                    })?;

                self.emit_task_event(RuntimeEvent::TaskCancelled {
                    session_id: task.session_id.to_string(),
                    turn_id: task.turn_id.to_string(),
                    task_id: task.id.0.to_string(),
                })
                .await;

                Ok(RuntimeResponse {
                    session_id: Some(task.session_id.to_string()),
                    turn_id: Some(task.turn_id.to_string()),
                    accepted: true,
                    message: format!("task {} cancellation requested", task_id),
                })
            }
        }
    }
}

#[derive(Clone)]
pub struct RuntimeHandle {
    services: RuntimeServices,
    status: Arc<RwLock<RuntimeStatus>>,
}

impl RuntimeHandle {
    pub async fn status(&self) -> RuntimeStatus {
        self.status.read().await.clone()
    }

    pub fn task_store(&self) -> Arc<RuntimeTaskStore> {
        Arc::clone(&self.services.task_store)
    }

    pub async fn execute(
        &self,
        command: RuntimeCommand,
    ) -> Result<RuntimeResponse, RuntimeFacadeError> {
        let runtime = Runtime {
            services: self.services.clone(),
            status: Arc::clone(&self.status),
        };
        runtime.execute(command).await
    }
}
