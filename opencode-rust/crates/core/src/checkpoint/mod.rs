mod manager;
mod types;

pub use manager::{create_checkpoint, restore_checkpoint, CheckpointManager};
pub use types::{Checkpoint, CheckpointMetadata};

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Message, Session};
    use tempfile::TempDir;
    use uuid::Uuid;

    #[test]
    fn test_checkpoint_create_and_load() {
        let tmp = TempDir::new().unwrap();

        let mut session = Session::new();
        session.add_message(Message::user("Test message".to_string()));

        let manager = CheckpointManager {
            checkpoints_dir: tmp.path().to_path_buf(),
            max_checkpoints: 5,
        };

        let checkpoint = manager.create(&session, "Test checkpoint").unwrap();

        assert_eq!(checkpoint.session_id, session.id);
        assert_eq!(checkpoint.sequence_number, 0);

        let loaded = manager.load(&session.id, 0).unwrap();
        assert_eq!(loaded.id, session.id);
        assert_eq!(loaded.messages.len(), 1);
    }

    #[test]
    fn test_checkpoint_list() {
        let tmp = TempDir::new().unwrap();

        let session = Session::new();

        let manager = CheckpointManager {
            checkpoints_dir: tmp.path().to_path_buf(),
            max_checkpoints: 5,
        };

        manager.create(&session, "First").unwrap();
        manager.create(&session, "Second").unwrap();

        let list = manager.list(&session.id).unwrap();
        assert_eq!(list.len(), 2);
    }

    #[test]
    fn test_checkpoint_pruning() {
        let tmp = TempDir::new().unwrap();

        let session = Session::new();

        let manager = CheckpointManager {
            checkpoints_dir: tmp.path().to_path_buf(),
            max_checkpoints: 2,
        };

        manager.create(&session, "1").unwrap();
        manager.create(&session, "2").unwrap();
        manager.create(&session, "3").unwrap();

        let list = manager.list(&session.id).unwrap();
        assert_eq!(list.len(), 2);
    }

    #[test]
    fn test_checkpoint_get_latest() {
        let tmp = TempDir::new().unwrap();

        let session = Session::new();

        let manager = CheckpointManager {
            checkpoints_dir: tmp.path().to_path_buf(),
            max_checkpoints: 5,
        };

        manager.create(&session, "First").unwrap();
        let list = manager.list(&session.id).unwrap();
        assert_eq!(list.len(), 1);
    }

    #[test]
    fn test_checkpoint_get_latest_none() {
        let tmp = TempDir::new().unwrap();

        let manager = CheckpointManager {
            checkpoints_dir: tmp.path().to_path_buf(),
            max_checkpoints: 5,
        };

        let session_id = Uuid::new_v4();
        let latest = manager.get_latest(&session_id).unwrap();
        assert!(latest.is_none());
    }

    #[test]
    fn test_checkpoint_delete() {
        let tmp = TempDir::new().unwrap();

        let session = Session::new();

        let manager = CheckpointManager {
            checkpoints_dir: tmp.path().to_path_buf(),
            max_checkpoints: 5,
        };

        manager.create(&session, "First").unwrap();
        manager.create(&session, "Second").unwrap();

        let list_before = manager.list(&session.id).unwrap();
        assert_eq!(list_before.len(), 2);

        manager.delete(&session.id, 0).unwrap();

        let list_after = manager.list(&session.id).unwrap();
        assert_eq!(list_after.len(), 1);
    }

    #[test]
    fn test_checkpoint_delete_nonexistent() {
        let tmp = TempDir::new().unwrap();

        let manager = CheckpointManager {
            checkpoints_dir: tmp.path().to_path_buf(),
            max_checkpoints: 5,
        };

        let session = Session::new();
        manager.create(&session, "First").unwrap();

        let result = manager.delete(&session.id, 999);
        assert!(result.is_ok());
    }

    #[test]
    fn test_checkpoint_delete_all() {
        let tmp = TempDir::new().unwrap();

        let session = Session::new();

        let manager = CheckpointManager {
            checkpoints_dir: tmp.path().to_path_buf(),
            max_checkpoints: 5,
        };

        manager.create(&session, "First").unwrap();
        manager.create(&session, "Second").unwrap();

        let list_before = manager.list(&session.id).unwrap();
        assert_eq!(list_before.len(), 2);

        manager.delete_all(&session.id).unwrap();

        let list_after = manager.list(&session.id).unwrap();
        assert!(list_after.is_empty());
    }

    #[test]
    fn test_checkpoint_delete_all_nonexistent() {
        let tmp = TempDir::new().unwrap();

        let manager = CheckpointManager {
            checkpoints_dir: tmp.path().to_path_buf(),
            max_checkpoints: 5,
        };

        let session_id = Uuid::new_v4();
        let result = manager.delete_all(&session_id);
        assert!(result.is_ok());
    }

    #[test]
    fn test_checkpoint_manager_new() {
        let manager = CheckpointManager::new();
        assert_eq!(manager.max_checkpoints, 10);
    }

    #[test]
    fn test_checkpoint_manager_with_max_checkpoints() {
        let manager = CheckpointManager::new().with_max_checkpoints(20);
        assert_eq!(manager.max_checkpoints, 20);
    }

    #[test]
    fn test_checkpoint_manager_with_checkpoints_dir() {
        let tmp = TempDir::new().unwrap();
        let manager = CheckpointManager::new().with_checkpoints_dir(tmp.path().to_path_buf());
        assert_eq!(manager.checkpoints_dir, tmp.path());
    }

    #[test]
    fn test_checkpoint_dir() {
        let tmp = TempDir::new().unwrap();
        let manager = CheckpointManager {
            checkpoints_dir: tmp.path().to_path_buf(),
            max_checkpoints: 5,
        };

        let session_id = Uuid::new_v4();
        let dir = manager.checkpoint_dir(&session_id);
        assert_eq!(dir, tmp.path().join(session_id.to_string()));
    }

    #[test]
    fn test_checkpoint_path() {
        let tmp = TempDir::new().unwrap();
        let manager = CheckpointManager {
            checkpoints_dir: tmp.path().to_path_buf(),
            max_checkpoints: 5,
        };

        let session_id = Uuid::new_v4();
        let path = manager.checkpoint_path(&session_id, 5);
        assert!(path.to_str().unwrap().contains("checkpoint_0005.json"));
    }

    #[test]
    fn test_checkpoint_manager_default() {
        let manager = CheckpointManager::default();
        assert_eq!(manager.max_checkpoints, 10);
    }

    #[test]
    fn test_create_checkpoint_function() {
        let mut session = Session::new();
        session.add_message(Message::user("Test".to_string()));

        let checkpoint = create_checkpoint(&session, "Test").unwrap();
        assert_eq!(checkpoint.session_id, session.id);
    }
}
