use std::collections::HashMap;
use std::sync::Arc;

use tokio::sync::RwLock;

use crate::types::{RuntimeTask, RuntimeTaskId, RuntimeTaskStatus};

#[derive(Debug, Clone)]
pub struct RuntimeTaskStore {
    active_tasks: Arc<RwLock<HashMap<RuntimeTaskId, RuntimeTask>>>,
}

impl RuntimeTaskStore {
    pub fn new() -> Self {
        Self {
            active_tasks: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub async fn add_task(&self, task: RuntimeTask) -> RuntimeTaskId {
        let id = task.id;
        self.active_tasks.write().await.insert(id, task);
        id
    }

    pub async fn get_task(&self, id: RuntimeTaskId) -> Option<RuntimeTask> {
        self.active_tasks.read().await.get(&id).cloned()
    }

    pub async fn remove_task(&self, id: RuntimeTaskId) -> Option<RuntimeTask> {
        self.active_tasks.write().await.remove(&id)
    }

    pub async fn update_task<F>(&self, id: RuntimeTaskId, f: F) -> Option<RuntimeTask>
    where
        F: FnOnce(&mut RuntimeTask),
    {
        let mut tasks = self.active_tasks.write().await;
        if let Some(task) = tasks.get_mut(&id) {
            f(task);
            Some(task.clone())
        } else {
            None
        }
    }

    pub async fn cancel_task(&self, id: RuntimeTaskId) -> Option<RuntimeTask> {
        let mut tasks = self.active_tasks.write().await;
        if let Some(task) = tasks.get_mut(&id) {
            if task.can_cancel() {
                task.request_cancellation();
                task.mark_cancelling();
                return Some(task.clone());
            }
        }
        None
    }

    pub async fn list_active_tasks(&self) -> Vec<RuntimeTask> {
        self.active_tasks
            .read()
            .await
            .values()
            .filter(|t| !t.is_terminal())
            .cloned()
            .collect()
    }

    pub async fn list_tasks_by_session(&self, session_id: uuid::Uuid) -> Vec<RuntimeTask> {
        self.active_tasks
            .read()
            .await
            .values()
            .filter(|t| t.session_id == session_id)
            .cloned()
            .collect()
    }

    pub async fn complete_task(&self, id: RuntimeTaskId) -> Option<RuntimeTask> {
        self.update_task(id, RuntimeTask::mark_completed).await
    }

    pub async fn fail_task(&self, id: RuntimeTaskId) -> Option<RuntimeTask> {
        self.update_task(id, RuntimeTask::mark_failed).await
    }

    pub async fn active_count(&self) -> usize {
        self.active_tasks
            .read()
            .await
            .values()
            .filter(|t| !t.is_terminal())
            .count()
    }

    pub async fn task_status(&self, id: RuntimeTaskId) -> Option<RuntimeTaskStatus> {
        self.active_tasks.read().await.get(&id).map(|t| t.status)
    }
}

impl Default for RuntimeTaskStore {
    fn default() -> Self {
        Self::new()
    }
}
