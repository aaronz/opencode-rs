use crate::bus::{EventBus, InternalEvent, SharedEventBus};
use crate::session::{Session, SessionInfo};
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::{Arc, RwLock};
use uuid::Uuid;

#[derive(Debug, Clone)]
pub enum SharingError {
    SessionNotFound(String),
    LockError(String),
    StorageError(String),
    ConcurrentAccess {
        session_id: String,
        expected_version: u64,
        actual_version: u64,
    },
}

impl std::fmt::Display for SharingError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SharingError::SessionNotFound(id) => write!(f, "Session not found: {}", id),
            SharingError::LockError(msg) => write!(f, "Lock error: {}", msg),
            SharingError::StorageError(msg) => write!(f, "Storage error: {}", msg),
            SharingError::ConcurrentAccess {
                session_id,
                expected_version,
                actual_version,
            } => write!(
                f,
                "Concurrent access conflict for session {}: expected version {}, actual version {}",
                session_id, expected_version, actual_version
            ),
        }
    }
}

impl std::error::Error for SharingError {}

#[derive(Debug, Clone)]
struct SessionEntry {
    session: Session,
    version: u64,
    #[allow(dead_code)]
    path: PathBuf,
}

pub struct SessionSharing {
    sessions: Arc<RwLock<HashMap<String, SessionEntry>>>,
    event_bus: SharedEventBus,
    base_path: PathBuf,
}

impl SessionSharing {
    pub fn new(base_path: PathBuf, event_bus: SharedEventBus) -> Self {
        std::fs::create_dir_all(&base_path).ok();
        Self {
            sessions: Arc::new(RwLock::new(HashMap::new())),
            event_bus,
            base_path,
        }
    }

    pub fn with_default_path() -> Self {
        let base_path = Session::sessions_dir();
        let event_bus = Arc::new(EventBus::new());
        Self::new(base_path, event_bus)
    }

    pub fn create_session(&self, name: Option<String>) -> Result<Session, SharingError> {
        let mut session = Session::new();
        let msg_content = name.unwrap_or_else(|| String::new());
        session.add_message(crate::message::Message::user(msg_content));

        let path = self.session_path(&session.id);
        session
            .save(&path)
            .map_err(|e| SharingError::StorageError(e.to_string()))?;

        let entry = SessionEntry {
            session: session.clone(),
            version: 1,
            path,
        };

        {
            let mut sessions = self
                .sessions
                .write()
                .map_err(|e| SharingError::LockError(e.to_string()))?;
            sessions.insert(session.id.to_string(), entry);
        }

        self.event_bus
            .publish(InternalEvent::SessionStarted(session.id.to_string()));

        Ok(session)
    }

    pub fn get_session(&self, id: &Uuid) -> Result<Session, SharingError> {
        let id_str = id.to_string();

        {
            let sessions = self
                .sessions
                .read()
                .map_err(|e| SharingError::LockError(e.to_string()))?;
            if let Some(entry) = sessions.get(&id_str) {
                return Ok(entry.session.clone());
            }
        }

        let path = self.session_path(id);
        if !path.exists() {
            return Err(SharingError::SessionNotFound(id_str));
        }

        let session =
            Session::load(&path).map_err(|e| SharingError::StorageError(e.to_string()))?;

        let entry = SessionEntry {
            session: session.clone(),
            version: 1,
            path,
        };

        {
            let mut sessions = self
                .sessions
                .write()
                .map_err(|e| SharingError::LockError(e.to_string()))?;
            sessions.insert(id_str, entry);
        }

        Ok(session)
    }

    pub fn save_session(&self, session: &Session) -> Result<(), SharingError> {
        let id_str = session.id.to_string();
        let path = self.session_path(&session.id);

        session
            .save(&path)
            .map_err(|e| SharingError::StorageError(e.to_string()))?;

        {
            let mut sessions = self
                .sessions
                .write()
                .map_err(|e| SharingError::LockError(e.to_string()))?;
            if let Some(entry) = sessions.get_mut(&id_str) {
                entry.session = session.clone();
                entry.version += 1;
            } else {
                sessions.insert(
                    id_str.clone(),
                    SessionEntry {
                        session: session.clone(),
                        version: 1,
                        path,
                    },
                );
            }
        }

        self.event_bus.publish(InternalEvent::MessageUpdated {
            session_id: id_str,
            message_id: session.id.to_string(),
        });

        Ok(())
    }

    pub fn delete_session(&self, id: &Uuid) -> Result<(), SharingError> {
        let id_str = id.to_string();

        let path = self.session_path(id);
        if path.exists() {
            std::fs::remove_file(&path).map_err(|e| SharingError::StorageError(e.to_string()))?;
        }

        {
            let mut sessions = self
                .sessions
                .write()
                .map_err(|e| SharingError::LockError(e.to_string()))?;
            sessions.remove(&id_str);
        }

        self.event_bus.publish(InternalEvent::SessionEnded(id_str));
        Ok(())
    }

    pub fn list_sessions(&self) -> Result<Vec<SessionInfo>, SharingError> {
        let cached_ids: std::collections::HashSet<String> = {
            let sessions = self
                .sessions
                .read()
                .map_err(|e| SharingError::LockError(e.to_string()))?;
            sessions.keys().cloned().collect()
        };

        let mut sessions_to_load: Vec<(String, PathBuf)> = Vec::new();
        if let Ok(entries) = std::fs::read_dir(&self.base_path) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.extension().and_then(|s| s.to_str()) == Some("json") {
                    if let Some(filename) = path.file_stem().and_then(|s| s.to_str()) {
                        if !cached_ids.contains(filename) {
                            sessions_to_load.push((filename.to_string(), path));
                        }
                    }
                }
            }
        }

        if !sessions_to_load.is_empty() {
            let mut sessions = self
                .sessions
                .write()
                .map_err(|e| SharingError::LockError(e.to_string()))?;

            for (id, path) in sessions_to_load {
                if !sessions.contains_key(&id) {
                    if let Ok(session) = Session::load(&path) {
                        sessions.insert(
                            id,
                            SessionEntry {
                                session,
                                version: 1,
                                path,
                            },
                        );
                    }
                }
            }
        }

        let sessions = self
            .sessions
            .read()
            .map_err(|e| SharingError::LockError(e.to_string()))?;

        let mut infos: Vec<SessionInfo> = sessions
            .values()
            .map(|entry| SessionInfo {
                id: entry.session.id,
                created_at: entry.session.created_at,
                updated_at: entry.session.updated_at,
                message_count: entry.session.messages.len(),
                preview: entry
                    .session
                    .messages
                    .last()
                    .map(|m| m.content.chars().take(50).collect())
                    .unwrap_or_default(),
            })
            .collect();

        infos.sort_by(|a, b| b.updated_at.cmp(&a.updated_at));
        Ok(infos)
    }

    pub fn fork_session(
        &self,
        parent_id: &Uuid,
        new_session_id: Uuid,
    ) -> Result<Session, SharingError> {
        let parent = self.get_session(parent_id)?;
        let child = parent.fork(new_session_id);

        let path = self.session_path(&child.id);
        child
            .save(&path)
            .map_err(|e| SharingError::StorageError(e.to_string()))?;

        let entry = SessionEntry {
            session: child.clone(),
            version: 1,
            path,
        };

        {
            let mut sessions = self
                .sessions
                .write()
                .map_err(|e| SharingError::LockError(e.to_string()))?;
            sessions.insert(child.id.to_string(), entry);
        }

        self.event_bus.publish(InternalEvent::SessionForked {
            original_id: parent_id.to_string(),
            new_id: child.id.to_string(),
            fork_point: parent.messages.len(),
        });

        Ok(child)
    }

    pub fn add_message(
        &self,
        session_id: &Uuid,
        message: crate::message::Message,
    ) -> Result<Session, SharingError> {
        let mut session = self.get_session(session_id)?;
        session.add_message(message);
        self.save_session(&session)?;
        Ok(session)
    }

    pub fn session_path(&self, id: &Uuid) -> PathBuf {
        self.base_path.join(format!("{}.json", id))
    }

    pub fn get_version(&self, id: &Uuid) -> Result<u64, SharingError> {
        let id_str = id.to_string();
        let sessions = self
            .sessions
            .read()
            .map_err(|e| SharingError::LockError(e.to_string()))?;

        sessions
            .get(&id_str)
            .map(|entry| entry.version)
            .ok_or_else(|| SharingError::SessionNotFound(id_str))
    }

    pub fn reload_session(&self, id: &Uuid) -> Result<Session, SharingError> {
        let id_str = id.to_string();
        let path = self.session_path(id);

        if !path.exists() {
            return Err(SharingError::SessionNotFound(id_str));
        }

        let session =
            Session::load(&path).map_err(|e| SharingError::StorageError(e.to_string()))?;

        {
            let mut sessions = self
                .sessions
                .write()
                .map_err(|e| SharingError::LockError(e.to_string()))?;
            if let Some(entry) = sessions.get_mut(&id_str) {
                entry.session = session.clone();
                entry.version += 1;
            } else {
                sessions.insert(
                    id_str,
                    SessionEntry {
                        session: session.clone(),
                        version: 1,
                        path,
                    },
                );
            }
        }

        Ok(session)
    }

    pub fn exists(&self, id: &Uuid) -> bool {
        let id_str = id.to_string();
        if let Ok(sessions) = self.sessions.read() {
            if sessions.contains_key(&id_str) {
                return true;
            }
        }
        self.session_path(id).exists()
    }

    pub fn clear_cache(&self) -> Result<(), SharingError> {
        let mut sessions = self
            .sessions
            .write()
            .map_err(|e| SharingError::LockError(e.to_string()))?;
        sessions.clear();
        Ok(())
    }

    pub fn event_bus(&self) -> SharedEventBus {
        self.event_bus.clone()
    }
}

impl Default for SessionSharing {
    fn default() -> Self {
        Self::with_default_path()
    }
}

impl Clone for SessionSharing {
    fn clone(&self) -> Self {
        Self {
            sessions: Arc::clone(&self.sessions),
            event_bus: Arc::clone(&self.event_bus),
            base_path: self.base_path.clone(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::message::Message;
    use tempfile::TempDir;

    fn create_test_sharing() -> (SessionSharing, TempDir) {
        let temp = TempDir::new().unwrap();
        let path = temp.path().to_path_buf();
        let event_bus = Arc::new(EventBus::new());
        let sharing = SessionSharing::new(path, event_bus);
        (sharing, temp)
    }

    #[test]
    fn test_session_sharing_new() {
        let (sharing, _temp) = create_test_sharing();
        let sessions = sharing.list_sessions().unwrap();
        assert!(sessions.is_empty());
    }

    #[test]
    fn test_session_sharing_create_session() {
        let (sharing, _temp) = create_test_sharing();
        let session = sharing
            .create_session(Some("Test session".to_string()))
            .unwrap();

        assert!(!session.id.is_nil());
        assert_eq!(session.messages.len(), 1);
        assert!(sharing.exists(&session.id));
    }

    #[test]
    fn test_session_sharing_get_session() {
        let (sharing, _temp) = create_test_sharing();
        let created = sharing.create_session(None).unwrap();

        let retrieved = sharing.get_session(&created.id).unwrap();
        assert_eq!(retrieved.id, created.id);
        assert_eq!(retrieved.messages.len(), created.messages.len());
    }

    #[test]
    fn test_session_sharing_save_and_get() {
        let (sharing, _temp) = create_test_sharing();
        let mut session = sharing.create_session(None).unwrap();

        session.add_message(Message::user("Hello".to_string()));
        session.add_message(Message::assistant("Hi there".to_string()));

        sharing.save_session(&session).unwrap();

        let retrieved = sharing.get_session(&session.id).unwrap();
        assert_eq!(retrieved.messages.len(), 3);
        assert_eq!(retrieved.messages[1].content, "Hello");
        assert_eq!(retrieved.messages[2].content, "Hi there");
    }

    #[test]
    fn test_session_sharing_delete_session() {
        let (sharing, _temp) = create_test_sharing();
        let session = sharing.create_session(None).unwrap();
        let id = session.id;

        assert!(sharing.exists(&id));
        sharing.delete_session(&id).unwrap();
        assert!(!sharing.exists(&id));
    }

    #[test]
    fn test_session_sharing_list_sessions() {
        let (sharing, _temp) = create_test_sharing();

        sharing.create_session(None).unwrap();
        sharing.create_session(None).unwrap();
        sharing.create_session(None).unwrap();

        let sessions = sharing.list_sessions().unwrap();
        assert_eq!(sessions.len(), 3);
    }

    #[test]
    fn test_session_sharing_fork_session() {
        let (sharing, _temp) = create_test_sharing();
        let mut parent = sharing.create_session(None).unwrap();
        parent.add_message(Message::user("Parent message".to_string()));
        sharing.save_session(&parent).unwrap();

        let child_id = Uuid::new_v4();
        let child = sharing.fork_session(&parent.id, child_id).unwrap();

        assert_ne!(child.id, parent.id);
        assert_eq!(
            child.parent_session_id.as_deref(),
            Some(parent.id.to_string().as_str())
        );
        assert_eq!(child.messages.len(), 2);
        assert!(sharing.exists(&child.id));
    }

    #[test]
    fn test_session_sharing_add_message() {
        let (sharing, _temp) = create_test_sharing();
        let session = sharing.create_session(None).unwrap();

        let updated = sharing
            .add_message(&session.id, Message::user("New message".to_string()))
            .unwrap();

        assert_eq!(updated.messages.len(), 2);
        assert_eq!(updated.messages[1].content, "New message");
    }

    #[test]
    fn test_session_sharing_version_increment() {
        let (sharing, _temp) = create_test_sharing();
        let session = sharing.create_session(None).unwrap();

        let v1 = sharing.get_version(&session.id).unwrap();
        assert_eq!(v1, 1);

        let mut s = sharing.get_session(&session.id).unwrap();
        s.add_message(Message::user("test".to_string()));
        sharing.save_session(&s).unwrap();

        let v2 = sharing.get_version(&session.id).unwrap();
        assert_eq!(v2, 2);
    }

    #[test]
    fn test_session_sharing_concurrent_access() {
        let (sharing, _temp) = create_test_sharing();
        let session = sharing.create_session(None).unwrap();
        let id = session.id;

        let v1 = sharing.get_version(&id).unwrap();

        let s1 = sharing.get_session(&id).unwrap();
        let s2 = sharing.get_session(&id).unwrap();

        let mut s1_modified = s1.clone();
        s1_modified.add_message(Message::user("From s1".to_string()));

        let mut s2_modified = s2.clone();
        s2_modified.add_message(Message::user("From s2".to_string()));

        sharing.save_session(&s1_modified).unwrap();

        let v2 = sharing.get_version(&id).unwrap();
        assert_eq!(v2, v1 + 1);

        sharing.save_session(&s2_modified).unwrap();

        let v3 = sharing.get_version(&id).unwrap();
        assert_eq!(v3, v2 + 1);
    }

    #[test]
    fn test_session_sharing_reload() {
        let (sharing, _temp) = create_test_sharing();
        let mut session = sharing.create_session(None).unwrap();

        session.add_message(Message::user("Data".to_string()));
        sharing.save_session(&session).unwrap();

        sharing.clear_cache().unwrap();

        let reloaded = sharing.reload_session(&session.id).unwrap();
        assert_eq!(reloaded.id, session.id);
        assert_eq!(reloaded.messages.len(), 2);
    }

    #[test]
    fn test_session_sharing_session_not_found() {
        let (sharing, _temp) = create_test_sharing();
        let fake_id = Uuid::new_v4();

        let result = sharing.get_session(&fake_id);
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            SharingError::SessionNotFound(_)
        ));
    }

    #[test]
    fn test_session_sharing_clone_independence() {
        let (sharing1, _temp) = create_test_sharing();
        let session = sharing1.create_session(None).unwrap();

        let sharing2 = sharing1.clone();

        let retrieved = sharing2.get_session(&session.id).unwrap();
        assert_eq!(retrieved.id, session.id);
    }
}
