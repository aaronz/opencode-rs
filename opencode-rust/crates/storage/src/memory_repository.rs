use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use async_trait::async_trait;
use opencode_core::{Session, SessionInfo};

use crate::error::StorageError;
use crate::models::ProjectModel;
use crate::repository::{sealed, ProjectRepository, SessionRepository};

pub struct InMemorySessionRepository {
    sessions: Arc<Mutex<HashMap<String, Session>>>,
}

impl InMemorySessionRepository {
    pub fn new() -> Self {
        Self {
            sessions: Arc::new(Mutex::new(HashMap::new())),
        }
    }
}

impl Default for InMemorySessionRepository {
    fn default() -> Self {
        Self::new()
    }
}

impl sealed::Sealed for InMemorySessionRepository {}

#[async_trait]
impl SessionRepository for InMemorySessionRepository {
    async fn find_by_id(&self, id: &str) -> Result<Option<Session>, StorageError> {
        let sessions = self.sessions.lock().unwrap();
        Ok(sessions.get(id).cloned())
    }

    async fn find_all(
        &self,
        limit: usize,
        offset: usize,
    ) -> Result<Vec<SessionInfo>, StorageError> {
        let sessions = self.sessions.lock().unwrap();
        let mut sessions: Vec<_> = sessions.values().cloned().collect();
        sessions.sort_by(|a, b| b.updated_at.cmp(&a.updated_at));
        let sessions: Vec<_> = sessions.into_iter().skip(offset).take(limit).collect();
        Ok(sessions
            .into_iter()
            .map(|s| SessionInfo {
                id: s.id,
                created_at: s.created_at,
                updated_at: s.updated_at,
                message_count: s.messages.len(),
                preview: s
                    .messages
                    .last()
                    .map(|m| m.content.chars().take(50).collect())
                    .unwrap_or_default(),
            })
            .collect())
    }

    async fn save(&self, session: &Session) -> Result<(), StorageError> {
        let mut sessions = self.sessions.lock().unwrap();
        sessions.insert(session.id.to_string(), session.clone());
        Ok(())
    }

    async fn delete(&self, id: &str) -> Result<(), StorageError> {
        let mut sessions = self.sessions.lock().unwrap();
        sessions.remove(id);
        Ok(())
    }

    async fn count(&self) -> Result<usize, StorageError> {
        let sessions = self.sessions.lock().unwrap();
        Ok(sessions.len())
    }
}

pub struct InMemoryProjectRepository {
    projects: Arc<Mutex<HashMap<String, ProjectModel>>>,
}

impl InMemoryProjectRepository {
    pub fn new() -> Self {
        Self {
            projects: Arc::new(Mutex::new(HashMap::new())),
        }
    }
}

impl Default for InMemoryProjectRepository {
    fn default() -> Self {
        Self::new()
    }
}

impl sealed::Sealed for InMemoryProjectRepository {}

#[async_trait]
impl ProjectRepository for InMemoryProjectRepository {
    async fn find_by_id(&self, id: &str) -> Result<Option<ProjectModel>, StorageError> {
        let projects = self.projects.lock().unwrap();
        Ok(projects.get(id).cloned())
    }

    async fn find_by_path(&self, path: &str) -> Result<Option<ProjectModel>, StorageError> {
        let projects = self.projects.lock().unwrap();
        Ok(projects.values().find(|p| p.path == path).cloned())
    }

    async fn find_all(
        &self,
        limit: usize,
        offset: usize,
    ) -> Result<Vec<ProjectModel>, StorageError> {
        let projects = self.projects.lock().unwrap();
        let mut projects: Vec<_> = projects.values().cloned().collect();
        projects.sort_by(|a, b| b.updated_at.cmp(&a.updated_at));
        Ok(projects.into_iter().skip(offset).take(limit).collect())
    }

    async fn save(&self, project: &ProjectModel) -> Result<(), StorageError> {
        let mut projects = self.projects.lock().unwrap();
        projects.insert(project.id.clone(), project.clone());
        Ok(())
    }

    async fn delete(&self, id: &str) -> Result<(), StorageError> {
        let mut projects = self.projects.lock().unwrap();
        projects.remove(id);
        Ok(())
    }

    async fn count(&self) -> Result<usize, StorageError> {
        let projects = self.projects.lock().unwrap();
        Ok(projects.len())
    }
}
