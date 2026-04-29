use std::sync::Arc;

use opencode_core::Session;
use opencode_storage::StorageService;

use crate::errors::RuntimeFacadeError;

#[derive(Clone)]
pub struct RuntimeSessionStore {
    storage: Arc<StorageService>,
}

impl RuntimeSessionStore {
    pub fn new(storage: Arc<StorageService>) -> Self {
        Self { storage }
    }

    pub async fn save_session(&self, session: &Session) -> Result<(), RuntimeFacadeError> {
        self.storage
            .save_session(session)
            .await
            .map_err(|e| RuntimeFacadeError::Dependency(e.to_string()))
    }

    pub async fn load_session(
        &self,
        session_id: &str,
    ) -> Result<Option<Session>, RuntimeFacadeError> {
        self.storage
            .load_session(session_id)
            .await
            .map_err(|e| RuntimeFacadeError::Dependency(e.to_string()))
    }
}
