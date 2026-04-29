use std::collections::HashMap;
use std::sync::Arc;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use tokio::sync::RwLock;
use uuid::Uuid;

use crate::errors::RuntimeFacadeError;
use crate::types::{RuntimeTaskId, RuntimeTaskStatus};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Checkpoint {
    pub id: Uuid,
    pub task_id: RuntimeTaskId,
    pub session_id: Uuid,
    pub turn_id: Uuid,
    pub task_status: RuntimeTaskStatus,
    pub task_description: String,
    pub current_step: String,
    pub data: serde_json::Value,
    pub created_at: DateTime<Utc>,
}

impl Checkpoint {
    pub fn new(
        task_id: RuntimeTaskId,
        session_id: Uuid,
        turn_id: Uuid,
        task_status: RuntimeTaskStatus,
        task_description: String,
        current_step: String,
        data: serde_json::Value,
    ) -> Self {
        Self {
            id: Uuid::new_v4(),
            task_id,
            session_id,
            turn_id,
            task_status,
            task_description,
            current_step,
            data,
            created_at: Utc::now(),
        }
    }
}

#[expect(async_fn_in_trait)]
pub trait CheckpointStore: Send + Sync {
    async fn save_checkpoint(&self, checkpoint: &Checkpoint) -> Result<(), RuntimeFacadeError>;
    async fn load_latest(
        &self,
        task_id: &RuntimeTaskId,
    ) -> Result<Option<Checkpoint>, RuntimeFacadeError>;
    async fn load_latest_for_session(
        &self,
        session_id: &Uuid,
    ) -> Result<Option<Checkpoint>, RuntimeFacadeError>;
    async fn delete_checkpoint(&self, id: &Uuid) -> Result<(), RuntimeFacadeError>;
    async fn list_for_session(
        &self,
        session_id: &Uuid,
    ) -> Result<Vec<Checkpoint>, RuntimeFacadeError>;
}

#[derive(Default, Clone)]
pub struct RuntimeCheckpointStore {
    checkpoints: Arc<RwLock<HashMap<Uuid, Checkpoint>>>,
    by_task: Arc<RwLock<HashMap<RuntimeTaskId, Uuid>>>,
    by_session: Arc<RwLock<HashMap<Uuid, Vec<Uuid>>>>,
}

impl RuntimeCheckpointStore {
    pub fn new() -> Self {
        Self::default()
    }

    pub async fn save(&self, checkpoint: &Checkpoint) -> Result<(), RuntimeFacadeError> {
        {
            let mut checkpoints = self.checkpoints.write().await;
            checkpoints.insert(checkpoint.id, checkpoint.clone());
        }
        {
            let mut by_task = self.by_task.write().await;
            by_task.insert(checkpoint.task_id, checkpoint.id);
        }
        {
            let mut by_session = self.by_session.write().await;
            by_session
                .entry(checkpoint.session_id)
                .or_default()
                .push(checkpoint.id);
        }
        Ok(())
    }

    pub async fn load_latest_for_task(
        &self,
        task_id: &RuntimeTaskId,
    ) -> Result<Option<Checkpoint>, RuntimeFacadeError> {
        let by_task = self.by_task.read().await;
        let checkpoint_id = by_task.get(task_id).copied();
        drop(by_task);

        match checkpoint_id {
            Some(id) => {
                let checkpoints = self.checkpoints.read().await;
                Ok(checkpoints.get(&id).cloned())
            }
            None => Ok(None),
        }
    }
}

impl CheckpointStore for RuntimeCheckpointStore {
    async fn save_checkpoint(&self, checkpoint: &Checkpoint) -> Result<(), RuntimeFacadeError> {
        self.save(checkpoint).await
    }

    async fn load_latest(
        &self,
        task_id: &RuntimeTaskId,
    ) -> Result<Option<Checkpoint>, RuntimeFacadeError> {
        self.load_latest_for_task(task_id).await
    }

    async fn load_latest_for_session(
        &self,
        _session_id: &Uuid,
    ) -> Result<Option<Checkpoint>, RuntimeFacadeError> {
        Ok(None)
    }

    async fn delete_checkpoint(&self, _id: &Uuid) -> Result<(), RuntimeFacadeError> {
        Ok(())
    }

    async fn list_for_session(
        &self,
        _session_id: &Uuid,
    ) -> Result<Vec<Checkpoint>, RuntimeFacadeError> {
        Ok(vec![])
    }
}
