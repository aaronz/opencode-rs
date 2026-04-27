mod types;

pub use types::{SyncManager, SyncState, SyncStatus};

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

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
