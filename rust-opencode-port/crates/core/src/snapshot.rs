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

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    #[test]
    fn test_snapshot_manager_new() {
        let sm = SnapshotManager::new();
        assert!(sm.list().is_empty());
    }

    #[test]
    fn test_snapshot_manager_create() {
        let mut sm = SnapshotManager::new();
        let mut files = HashMap::new();
        files.insert("test.txt".to_string(), "content".to_string());

        let snapshot = sm.create("Test snapshot".to_string(), files);

        assert!(!snapshot.id.is_empty());
        assert_eq!(snapshot.description, "Test snapshot");
    }

    #[test]
    fn test_snapshot_manager_get() {
        let mut sm = SnapshotManager::new();
        let mut files = HashMap::new();
        files.insert("test.txt".to_string(), "content".to_string());

        let snapshot = sm.create("Test".to_string(), files);
        let id = snapshot.id.clone();

        assert!(sm.get(&id).is_some());
    }

    #[test]
    fn test_snapshot_manager_list() {
        let mut sm = SnapshotManager::new();
        let files: HashMap<String, String> = HashMap::new();
        sm.create("snapshot1".to_string(), files.clone());
        sm.create("snapshot2".to_string(), files);

        assert_eq!(sm.list().len(), 2);
    }

    #[test]
    fn test_snapshot_manager_revert() {
        let mut sm = SnapshotManager::new();
        let mut files = HashMap::new();
        files.insert("file.txt".to_string(), "content".to_string());

        let snapshot = sm.create("Test".to_string(), files);

        let reverted = sm.revert(&snapshot.id);
        assert!(reverted.is_some());
    }
}
