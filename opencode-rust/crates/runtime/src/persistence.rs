use std::sync::Arc;

use async_trait::async_trait;
use opencode_core::Session;

use crate::errors::RuntimeFacadeError;

#[async_trait]
pub trait StateStore: Send + Sync {
    async fn save_session(&self, session: &Session) -> Result<(), RuntimeFacadeError>;
    async fn load_session(&self, session_id: &str) -> Result<Option<Session>, RuntimeFacadeError>;
}

#[async_trait]
impl StateStore for opencode_storage::StorageService {
    async fn save_session(&self, session: &Session) -> Result<(), RuntimeFacadeError> {
        opencode_storage::StorageService::save_session(self, session)
            .await
            .map_err(|e| RuntimeFacadeError::Dependency(e.to_string()))
    }

    async fn load_session(&self, session_id: &str) -> Result<Option<Session>, RuntimeFacadeError> {
        opencode_storage::StorageService::load_session(self, session_id)
            .await
            .map_err(|e| RuntimeFacadeError::Dependency(e.to_string()))
    }
}

#[derive(Clone)]
pub struct RuntimeFacadeSessionStore {
    store: Arc<dyn StateStore>,
}

impl RuntimeFacadeSessionStore {
    pub fn new(store: Arc<dyn StateStore>) -> Self {
        Self { store }
    }

    pub async fn save_session(&self, session: &Session) -> Result<(), RuntimeFacadeError> {
        self.store.save_session(session).await
    }

    pub async fn load_session(
        &self,
        session_id: &str,
    ) -> Result<Option<Session>, RuntimeFacadeError> {
        self.store.load_session(session_id).await
    }
}
