use crate::session::Session;
use crate::OpenCodeError;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Checkpoint {
    pub id: Uuid,
    pub session_id: Uuid,
    pub sequence_number: usize,
    pub created_at: DateTime<Utc>,
    pub description: String,
    pub checkpoint_path: PathBuf,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CheckpointMetadata {
    pub id: Uuid,
    pub session_id: Uuid,
    pub sequence_number: usize,
    pub created_at: DateTime<Utc>,
    pub description: String,
}

pub struct CheckpointManager {
    pub(crate) checkpoints_dir: PathBuf,
    pub(crate) max_checkpoints: usize,
}

impl Default for CheckpointManager {
    fn default() -> Self {
        Self::new()
    }
}

impl CheckpointManager {
    pub fn new() -> Self {
        let checkpoints_dir = Session::sessions_dir().join("checkpoints");
        Self {
            checkpoints_dir,
            max_checkpoints: 10,
        }
    }

    pub fn with_max_checkpoints(mut self, max: usize) -> Self {
        self.max_checkpoints = max;
        self
    }

    pub fn checkpoint_dir(&self, session_id: &Uuid) -> PathBuf {
        self.checkpoints_dir.join(session_id.to_string())
    }

    pub fn checkpoint_path(&self, session_id: &Uuid, seq: usize) -> PathBuf {
        self.checkpoint_dir(session_id)
            .join(format!("checkpoint_{:04}.json", seq))
    }

    pub fn create(
        &self,
        session: &Session,
        description: &str,
    ) -> Result<Checkpoint, OpenCodeError> {
        let session_dir = self.checkpoint_dir(&session.id);
        fs::create_dir_all(&session_dir)?;

        let existing_checkpoints = self.list(&session.id)?;
        let sequence_number = existing_checkpoints.len();

        let checkpoint_path = self.checkpoint_path(&session.id, sequence_number);
        session.save(&checkpoint_path)?;

        let checkpoint = Checkpoint {
            id: Uuid::new_v4(),
            session_id: session.id,
            sequence_number,
            created_at: Utc::now(),
            description: description.to_string(),
            checkpoint_path: checkpoint_path.clone(),
        };

        self.prune_old_checkpoints(&session.id)?;

        Ok(checkpoint)
    }

    pub fn load(
        &self,
        session_id: &Uuid,
        sequence_number: usize,
    ) -> Result<Session, OpenCodeError> {
        let path = self.checkpoint_path(session_id, sequence_number);
        Session::load(&path)
    }

    pub fn list(&self, session_id: &Uuid) -> Result<Vec<CheckpointMetadata>, OpenCodeError> {
        let dir = self.checkpoint_dir(session_id);
        if !dir.exists() {
            return Ok(Vec::new());
        }

        let mut checkpoints = Vec::new();

        for entry in fs::read_dir(&dir)? {
            let entry = entry?;
            let path = entry.path();
            if path.extension().is_some_and(|ext| ext == "json")
                && path
                    .file_stem()
                    .and_then(|s| s.to_str())
                    .is_some_and(|s| s.starts_with("checkpoint_"))
            {
                if let Ok(content) = fs::read_to_string(&path) {
                    if let Ok(session) = serde_json::from_str::<Session>(&content) {
                        let seq = path
                            .file_stem()
                            .and_then(|s| s.to_str())
                            .and_then(|s| s.strip_prefix("checkpoint_"))
                            .and_then(|s| s.strip_suffix(".json"))
                            .and_then(|s| s.parse::<usize>().ok())
                            .unwrap_or(0);

                        checkpoints.push(CheckpointMetadata {
                            id: Uuid::new_v4(),
                            session_id: *session_id,
                            sequence_number: seq,
                            created_at: session.updated_at,
                            description: format!("Checkpoint {}", seq),
                        });
                    }
                }
            }
        }

        checkpoints.sort_by(|a, b| a.sequence_number.cmp(&b.sequence_number));
        Ok(checkpoints)
    }

    pub fn get_latest(&self, session_id: &Uuid) -> Result<Option<Session>, OpenCodeError> {
        let checkpoints = self.list(session_id)?;
        if let Some(latest) = checkpoints.last() {
            Ok(Some(self.load(session_id, latest.sequence_number)?))
        } else {
            Ok(None)
        }
    }

    pub fn delete(&self, session_id: &Uuid, sequence_number: usize) -> Result<(), OpenCodeError> {
        let path = self.checkpoint_path(session_id, sequence_number);
        if path.exists() {
            fs::remove_file(path)?;
        }
        Ok(())
    }

    pub fn delete_all(&self, session_id: &Uuid) -> Result<(), OpenCodeError> {
        let dir = self.checkpoint_dir(session_id);
        if dir.exists() {
            fs::remove_dir_all(dir)?;
        }
        Ok(())
    }

    fn prune_old_checkpoints(&self, session_id: &Uuid) -> Result<(), OpenCodeError> {
        let checkpoints = self.list(session_id)?;
        if checkpoints.len() > self.max_checkpoints {
            let to_delete: Vec<_> = checkpoints
                .iter()
                .take(checkpoints.len() - self.max_checkpoints)
                .collect();

            for checkpoint in to_delete {
                self.delete(session_id, checkpoint.sequence_number)?;
            }
        }
        Ok(())
    }
}

pub fn create_checkpoint(
    session: &Session,
    description: &str,
) -> Result<Checkpoint, OpenCodeError> {
    let manager = CheckpointManager::new();
    manager.create(session, description)
}

pub fn restore_checkpoint(
    session_id: &Uuid,
    sequence_number: usize,
) -> Result<Session, OpenCodeError> {
    let manager = CheckpointManager::new();
    manager.load(session_id, sequence_number)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Message;
    use tempfile::TempDir;

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
}
