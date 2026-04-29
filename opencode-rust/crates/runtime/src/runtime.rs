use std::sync::Arc;

use tokio::sync::RwLock;

use crate::commands::RuntimeCommand;
use crate::errors::RuntimeFacadeError;
use crate::services::RuntimeServices;
use crate::types::{RuntimeResponse, RuntimeStatus};

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

    pub async fn execute(
        &self,
        command: RuntimeCommand,
    ) -> Result<RuntimeResponse, RuntimeFacadeError> {
        match command {
            RuntimeCommand::SubmitUserInput(cmd) => {
                if let Some(session_id) = cmd.session_id {
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

                    let turn_id = session.start_turn(None).0.to_string();
                    let persisted_session_id = session.id.to_string();

                    self.services
                        .storage
                        .save_session(&session)
                        .await
                        .map_err(|e| RuntimeFacadeError::Dependency(e.to_string()))?;

                    Ok(RuntimeResponse {
                        session_id: Some(persisted_session_id),
                        turn_id: Some(turn_id),
                        accepted: true,
                        message: "turn created".to_string(),
                    })
                } else {
                    let (session_id, turn_id) = self
                        .services
                        .agent_runtime
                        .read()
                        .await
                        .with_session_mut(|session| {
                            let turn_id = session.start_turn(None);
                            (session.id.to_string(), turn_id.0.to_string())
                        })
                        .await;

                    Ok(RuntimeResponse {
                        session_id: Some(session_id),
                        turn_id: Some(turn_id),
                        accepted: true,
                        message: "turn created".to_string(),
                    })
                }
            }
            RuntimeCommand::TaskControl(_) => {
                Err(RuntimeFacadeError::NotImplemented("task control"))
            }
            RuntimeCommand::PermissionResponse(_) => {
                Err(RuntimeFacadeError::NotImplemented("permission response"))
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
