use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use tokio::sync::RwLock;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RollbackEntry {
    pub path: PathBuf,
    pub original_content: Option<String>,
    pub backup_content: Option<String>,
    pub operation: RollbackOperation,
    pub created_at: DateTime<Utc>,
    pub applied: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RollbackOperation {
    Create,
    Modify,
    Delete,
}

pub struct RollbackStore {
    entries: Arc<RwLock<HashMap<PathBuf, RollbackEntry>>>,
}

impl Default for RollbackStore {
    fn default() -> Self {
        Self::new()
    }
}

impl RollbackStore {
    pub fn new() -> Self {
        Self {
            entries: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub async fn record_create(&self, path: PathBuf) -> RollbackEntry {
        let entry = RollbackEntry {
            path: path.clone(),
            original_content: None,
            backup_content: None,
            operation: RollbackOperation::Create,
            created_at: Utc::now(),
            applied: false,
        };
        let mut entries = self.entries.write().await;
        entries.insert(path, entry.clone());
        entry
    }

    pub async fn record_modify(&self, path: PathBuf, original_content: String) -> RollbackEntry {
        let entry = RollbackEntry {
            path: path.clone(),
            original_content: Some(original_content),
            backup_content: None,
            operation: RollbackOperation::Modify,
            created_at: Utc::now(),
            applied: false,
        };
        let mut entries = self.entries.write().await;
        entries.insert(path, entry.clone());
        entry
    }

    pub async fn record_delete(&self, path: PathBuf, backup_content: String) -> RollbackEntry {
        let entry = RollbackEntry {
            path: path.clone(),
            original_content: None,
            backup_content: Some(backup_content),
            operation: RollbackOperation::Delete,
            created_at: Utc::now(),
            applied: false,
        };
        let mut entries = self.entries.write().await;
        entries.insert(path, entry.clone());
        entry
    }

    pub async fn mark_applied(&self, path: &PathBuf) {
        let mut entries = self.entries.write().await;
        if let Some(entry) = entries.get_mut(path) {
            entry.applied = true;
        }
    }

    pub async fn get(&self, path: &PathBuf) -> Option<RollbackEntry> {
        let entries = self.entries.read().await;
        entries.get(path).cloned()
    }

    pub async fn get_unapplied(&self) -> Vec<RollbackEntry> {
        let entries = self.entries.read().await;
        entries.values().filter(|e| !e.applied).cloned().collect()
    }

    pub async fn remove(&self, path: &PathBuf) -> Option<RollbackEntry> {
        let mut entries = self.entries.write().await;
        entries.remove(path)
    }

    pub async fn clear(&self) {
        let mut entries = self.entries.write().await;
        entries.clear();
    }

    pub async fn list_all(&self) -> Vec<RollbackEntry> {
        let entries = self.entries.read().await;
        entries.values().cloned().collect()
    }
}

pub struct RollbackManager<F: super::fs::FileSystem> {
    fs: Arc<F>,
    store: Arc<RollbackStore>,
}

impl<F: super::fs::FileSystem> RollbackManager<F> {
    pub fn new(fs: Arc<F>, store: Arc<RollbackStore>) -> Self {
        Self { fs, store }
    }

    pub async fn write_with_rollback(
        &self,
        path: &PathBuf,
        content: &str,
    ) -> Result<(), super::fs::FileSystemError> {
        let original_content = self.fs.read_file_if_exists(path).await.ok().flatten();

        if let Some(original) = original_content {
            self.store.record_modify(path.clone(), original).await;
        } else {
            self.store.record_create(path.clone()).await;
        }

        self.fs.write_file(path, content).await?;
        self.store.mark_applied(path).await;
        Ok(())
    }

    pub async fn delete_with_rollback(
        &self,
        path: &PathBuf,
    ) -> Result<(), super::fs::FileSystemError> {
        if let Ok(content) = self.fs.read_file(path).await {
            self.store.record_delete(path.clone(), content).await;
        }

        self.fs.delete_file(path).await?;
        self.store.mark_applied(path).await;
        Ok(())
    }

    pub async fn rollback(&self, path: &PathBuf) -> Result<(), super::fs::FileSystemError> {
        let entry = match self.store.get(path).await {
            Some(e) => e,
            None => return Ok(()),
        };

        match entry.operation {
            RollbackOperation::Create => {
                if self.fs.exists(path).await {
                    self.fs.delete_file(path).await?;
                }
            }
            RollbackOperation::Modify => {
                if let Some(original) = entry.original_content {
                    self.fs.write_file(path, &original).await?;
                }
            }
            RollbackOperation::Delete => {
                if let Some(backup) = entry.backup_content {
                    self.fs.write_file(path, &backup).await?;
                }
            }
        }

        self.store.remove(path).await;
        Ok(())
    }

    pub async fn rollback_all(&self) -> Result<(), super::fs::FileSystemError> {
        let unapplied = self.store.get_unapplied().await;
        for entry in unapplied {
            self.rollback(&entry.path).await?;
        }
        Ok(())
    }
}
