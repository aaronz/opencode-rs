use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Snapshot {
    pub id: String,
    pub created_at: DateTime<Utc>,
    pub description: String,
    pub files: HashMap<String, String>,
}

pub struct SnapshotManager {
    snapshots: Vec<Snapshot>,
}

impl SnapshotManager {
    pub fn new() -> Self {
        Self {
            snapshots: Vec::new(),
        }
    }

    pub fn create(&mut self, description: String, files: HashMap<String, String>) -> Snapshot {
        let snapshot = Snapshot {
            id: uuid::Uuid::new_v4().to_string(),
            created_at: Utc::now(),
            description,
            files,
        };
        self.snapshots.push(snapshot.clone());
        snapshot
    }

    pub fn get(&self, id: &str) -> Option<&Snapshot> {
        self.snapshots.iter().find(|s| s.id == id)
    }

    pub fn list(&self) -> &[Snapshot] {
        &self.snapshots
    }

    pub fn revert(&self, id: &str) -> Option<&HashMap<String, String>> {
        self.get(id).map(|s| &s.files)
    }
}

impl Default for SnapshotManager {
    fn default() -> Self {
        Self::new()
    }
}
