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
