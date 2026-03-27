use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyncState {
    pub file: PathBuf,
    pub local_hash: String,
    pub remote_hash: String,
    pub status: SyncStatus,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SyncStatus {
    Synced,
    LocalChanged,
    RemoteChanged,
    Conflict,
}

pub struct SyncManager {
    states: HashMap<PathBuf, SyncState>,
}

impl SyncManager {
    pub fn new() -> Self {
        Self {
            states: HashMap::new(),
        }
    }

    pub fn track(&mut self, file: PathBuf, local_hash: String, remote_hash: String) {
        let status = if local_hash == remote_hash {
            SyncStatus::Synced
        } else {
            SyncStatus::LocalChanged
        };

        self.states.insert(
            file.clone(),
            SyncState {
                file,
                local_hash,
                remote_hash,
                status,
            },
        );
    }

    pub fn get_state(&self, file: &PathBuf) -> Option<&SyncState> {
        self.states.get(file)
    }

    pub fn needs_sync(&self, file: &PathBuf) -> bool {
        self.states
            .get(file)
            .map(|s| !matches!(s.status, SyncStatus::Synced))
            .unwrap_or(false)
    }
}

impl Default for SyncManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sync_manager_new() {
        let sm = SyncManager::new();
        assert!(sm.get_state(&PathBuf::from("/test")).is_none());
    }

    #[test]
    fn test_sync_manager_track_synced() {
        let mut sm = SyncManager::new();
        sm.track(
            PathBuf::from("/file"),
            "abc123".to_string(),
            "abc123".to_string(),
        );

        let state = sm.get_state(&PathBuf::from("/file")).unwrap();
        assert!(matches!(state.status, SyncStatus::Synced));
    }

    #[test]
    fn test_sync_manager_track_local_changed() {
        let mut sm = SyncManager::new();
        sm.track(
            PathBuf::from("/file"),
            "local".to_string(),
            "remote".to_string(),
        );

        let state = sm.get_state(&PathBuf::from("/file")).unwrap();
        assert!(matches!(state.status, SyncStatus::LocalChanged));
    }

    #[test]
    fn test_sync_manager_needs_sync() {
        let mut sm = SyncManager::new();
        sm.track(
            PathBuf::from("/file"),
            "local".to_string(),
            "remote".to_string(),
        );

        assert!(sm.needs_sync(&PathBuf::from("/file")));
        assert!(!sm.needs_sync(&PathBuf::from("/nonexistent")));
    }
}
