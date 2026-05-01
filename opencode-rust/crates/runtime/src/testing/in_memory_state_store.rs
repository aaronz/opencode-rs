use std::collections::HashMap;
use std::sync::{Arc, RwLock};

use async_trait::async_trait;
use opencode_core::Session;
use uuid::Uuid;

use crate::errors::RuntimeFacadeError;
use crate::persistence::StateStore;

pub struct InMemoryStateStore {
    sessions: Arc<RwLock<HashMap<Uuid, Session>>>,
}

impl Default for InMemoryStateStore {
    fn default() -> Self {
        Self::new()
    }
}

impl InMemoryStateStore {
    pub fn new() -> Self {
        Self {
            sessions: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub fn with_session(self: Arc<Self>, session: Session) -> Arc<Self> {
        self.sessions.write().unwrap().insert(session.id, session);
        self
    }

    pub fn session_count(&self) -> usize {
        self.sessions.read().unwrap().len()
    }

    pub fn has_session(&self, session_id: &Uuid) -> bool {
        self.sessions.read().unwrap().contains_key(session_id)
    }

    pub fn get_session(&self, session_id: &Uuid) -> Option<Session> {
        self.sessions.read().unwrap().get(session_id).cloned()
    }
}

#[async_trait]
impl StateStore for InMemoryStateStore {
    async fn save_session(&self, session: &Session) -> Result<(), RuntimeFacadeError> {
        self.sessions
            .write()
            .unwrap()
            .insert(session.id, session.clone());
        Ok(())
    }

    async fn load_session(&self, session_id: &str) -> Result<Option<Session>, RuntimeFacadeError> {
        let uuid = Uuid::parse_str(session_id).map_err(|_| {
            RuntimeFacadeError::InvalidConfiguration(format!(
                "Invalid session ID format: {}",
                session_id
            ))
        })?;
        Ok(self.sessions.read().unwrap().get(&uuid).cloned())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use opencode_core::Message;

    fn create_test_session() -> Session {
        let mut session = Session::new();
        session.messages.push(Message::user("test"));
        session
    }

    #[tokio::test]
    async fn test_in_memory_store_saves_and_loads_session() {
        let store = Arc::new(InMemoryStateStore::new());
        let session = create_test_session();
        let session_id = session.id.to_string();

        store.save_session(&session).await.unwrap();

        let loaded = store.load_session(&session_id).await.unwrap();
        assert!(loaded.is_some());
        assert_eq!(loaded.unwrap().id, session.id);
    }

    #[tokio::test]
    async fn test_in_memory_store_returns_none_for_missing_session() {
        let store = Arc::new(InMemoryStateStore::new());

        let loaded = store
            .load_session("00000000-0000-0000-0000-000000000000")
            .await
            .unwrap();
        assert!(loaded.is_none());
    }

    #[tokio::test]
    async fn test_in_memory_store_overwrites_existing_session() {
        let store = Arc::new(InMemoryStateStore::new());
        let mut session1 = create_test_session();
        let session_id = session1.id.to_string();

        let mut session2 = create_test_session();
        session2.id = session1.id;
        session2.messages.push(Message::user("updated"));

        store.save_session(&session1).await.unwrap();
        store.save_session(&session2).await.unwrap();

        let loaded = store.load_session(&session_id).await.unwrap();
        assert!(loaded.is_some());
        assert_eq!(loaded.unwrap().messages.len(), 2);
    }

    #[tokio::test]
    async fn test_in_memory_store_with_session_constructor() {
        let session = create_test_session();
        let session_id = session.id;
        let store = Arc::new(InMemoryStateStore::new()).with_session(session);

        let loaded = store.load_session(&session_id.to_string()).await.unwrap();
        assert!(loaded.is_some());
        assert_eq!(loaded.unwrap().id, session_id);
    }

    #[tokio::test]
    async fn test_has_session() {
        let session = create_test_session();
        let session_id = session.id;
        let store = Arc::new(InMemoryStateStore::new()).with_session(session);

        assert!(store.has_session(&session_id));
        assert!(!store.has_session(&Uuid::nil()));
    }

    #[tokio::test]
    async fn test_session_count() {
        let store = Arc::new(InMemoryStateStore::new());
        assert_eq!(store.session_count(), 0);

        let session1 = create_test_session();
        let session2 = create_test_session();

        store.save_session(&session1).await.unwrap();
        assert_eq!(store.session_count(), 1);

        store.save_session(&session2).await.unwrap();
        assert_eq!(store.session_count(), 2);
    }

    #[tokio::test]
    async fn test_invalid_session_id_format() {
        let store = Arc::new(InMemoryStateStore::new());

        let result = store.load_session("not-a-uuid").await;
        assert!(result.is_err());
    }
}
