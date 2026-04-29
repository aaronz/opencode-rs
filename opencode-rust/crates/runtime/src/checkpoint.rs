use std::collections::HashMap;
use std::sync::Arc;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use tokio::sync::RwLock;
use uuid::Uuid;

use crate::errors::RuntimeFacadeError;
use crate::types::{RuntimeFacadeTaskId, RuntimeFacadeTaskStatus};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Checkpoint {
    pub id: Uuid,
    pub task_id: RuntimeFacadeTaskId,
    pub session_id: Uuid,
    pub turn_id: Uuid,
    pub task_status: RuntimeFacadeTaskStatus,
    pub task_description: String,
    pub current_step: String,
    pub data: serde_json::Value,
    pub created_at: DateTime<Utc>,
}

impl Checkpoint {
    pub fn new(
        task_id: RuntimeFacadeTaskId,
        session_id: Uuid,
        turn_id: Uuid,
        task_status: RuntimeFacadeTaskStatus,
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
        task_id: &RuntimeFacadeTaskId,
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
pub struct RuntimeFacadeCheckpointStore {
    checkpoints: Arc<RwLock<HashMap<Uuid, Checkpoint>>>,
    by_task: Arc<RwLock<HashMap<RuntimeFacadeTaskId, Uuid>>>,
    by_session: Arc<RwLock<HashMap<Uuid, Vec<Uuid>>>>,
}

impl RuntimeFacadeCheckpointStore {
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
        task_id: &RuntimeFacadeTaskId,
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

impl CheckpointStore for RuntimeFacadeCheckpointStore {
    async fn save_checkpoint(&self, checkpoint: &Checkpoint) -> Result<(), RuntimeFacadeError> {
        self.save(checkpoint).await
    }

    async fn load_latest(
        &self,
        task_id: &RuntimeFacadeTaskId,
    ) -> Result<Option<Checkpoint>, RuntimeFacadeError> {
        self.load_latest_for_task(task_id).await
    }

    async fn load_latest_for_session(
        &self,
        session_id: &Uuid,
    ) -> Result<Option<Checkpoint>, RuntimeFacadeError> {
        let by_session = self.by_session.read().await;
        let checkpoint_ids = by_session.get(session_id).cloned().unwrap_or_default();
        drop(by_session);

        let checkpoints = self.checkpoints.read().await;
        Ok(checkpoint_ids
            .iter()
            .rev()
            .find_map(|id| checkpoints.get(id).cloned()))
    }

    async fn delete_checkpoint(&self, id: &Uuid) -> Result<(), RuntimeFacadeError> {
        let removed = {
            let mut checkpoints = self.checkpoints.write().await;
            checkpoints.remove(id)
        };

        let Some(removed) = removed else {
            return Ok(());
        };

        {
            let mut by_task = self.by_task.write().await;
            if by_task.get(&removed.task_id) == Some(id) {
                by_task.remove(&removed.task_id);
            }
        }

        {
            let mut by_session = self.by_session.write().await;
            if let Some(ids) = by_session.get_mut(&removed.session_id) {
                ids.retain(|checkpoint_id| checkpoint_id != id);
                if ids.is_empty() {
                    by_session.remove(&removed.session_id);
                }
            }
        }

        Ok(())
    }

    async fn list_for_session(
        &self,
        session_id: &Uuid,
    ) -> Result<Vec<Checkpoint>, RuntimeFacadeError> {
        let by_session = self.by_session.read().await;
        let checkpoint_ids = by_session.get(session_id).cloned().unwrap_or_default();
        drop(by_session);

        let checkpoints = self.checkpoints.read().await;
        Ok(checkpoint_ids
            .iter()
            .filter_map(|id| checkpoints.get(id).cloned())
            .collect())
    }
}
