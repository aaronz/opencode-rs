use async_trait::async_trait;
use opencode_core::{Session, SessionInfo};

use crate::error::StorageError;

pub mod sealed {
    pub trait Sealed {}
}

#[async_trait]
pub trait SessionRepository: Send + Sync + sealed::Sealed {
    async fn find_by_id(&self, id: &str) -> Result<Option<Session>, StorageError>;
    async fn find_all(&self, limit: usize, offset: usize)
        -> Result<Vec<SessionInfo>, StorageError>;
    async fn save(&self, session: &Session) -> Result<(), StorageError>;
    async fn delete(&self, id: &str) -> Result<(), StorageError>;
    async fn list_by_project(
        &self,
        _project_path: &str,
        _limit: usize,
        _offset: usize,
    ) -> Result<Vec<SessionInfo>, StorageError> {
        let _ = (_project_path, _limit, _offset);
        Ok(Vec::new())
    }
    async fn count(&self) -> Result<usize, StorageError>;
}

use crate::models::ProjectModel;

#[async_trait]
pub trait ProjectRepository: Send + Sync + sealed::Sealed {
    async fn find_by_id(&self, id: &str) -> Result<Option<ProjectModel>, StorageError>;
    async fn find_by_path(&self, path: &str) -> Result<Option<ProjectModel>, StorageError>;
    async fn find_all(
        &self,
        limit: usize,
        offset: usize,
    ) -> Result<Vec<ProjectModel>, StorageError>;
    async fn save(&self, project: &ProjectModel) -> Result<(), StorageError>;
    async fn delete(&self, id: &str) -> Result<(), StorageError>;
    async fn count(&self) -> Result<usize, StorageError>;
}
