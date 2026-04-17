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
        let sessions = self
            .sessions
            .lock()
            .unwrap_or_else(|poison| poison.into_inner());
        Ok(sessions.get(id).cloned())
    }

    async fn find_all(
        &self,
        limit: usize,
        offset: usize,
    ) -> Result<Vec<SessionInfo>, StorageError> {
        let sessions = self
            .sessions
            .lock()
            .unwrap_or_else(|poison| poison.into_inner());
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
        let mut sessions = self
            .sessions
            .lock()
            .unwrap_or_else(|poison| poison.into_inner());
        sessions.insert(session.id.to_string(), session.clone());
        Ok(())
    }

    async fn delete(&self, id: &str) -> Result<(), StorageError> {
        let mut sessions = self
            .sessions
            .lock()
            .unwrap_or_else(|poison| poison.into_inner());
        sessions.remove(id);
        Ok(())
    }

    async fn count(&self) -> Result<usize, StorageError> {
        let sessions = self
            .sessions
            .lock()
            .unwrap_or_else(|poison| poison.into_inner());
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
        let projects = self
            .projects
            .lock()
            .unwrap_or_else(|poison| poison.into_inner());
        Ok(projects.get(id).cloned())
    }

    async fn find_by_path(&self, path: &str) -> Result<Option<ProjectModel>, StorageError> {
        let projects = self
            .projects
            .lock()
            .unwrap_or_else(|poison| poison.into_inner());
        Ok(projects.values().find(|p| p.path == path).cloned())
    }

    async fn find_all(
        &self,
        limit: usize,
        offset: usize,
    ) -> Result<Vec<ProjectModel>, StorageError> {
        let projects = self
            .projects
            .lock()
            .unwrap_or_else(|poison| poison.into_inner());
        let mut projects: Vec<_> = projects.values().cloned().collect();
        projects.sort_by(|a, b| b.updated_at.cmp(&a.updated_at));
        Ok(projects.into_iter().skip(offset).take(limit).collect())
    }

    async fn save(&self, project: &ProjectModel) -> Result<(), StorageError> {
        let mut projects = self
            .projects
            .lock()
            .unwrap_or_else(|poison| poison.into_inner());
        projects.insert(project.id.clone(), project.clone());
        Ok(())
    }

    async fn delete(&self, id: &str) -> Result<(), StorageError> {
        let mut projects = self
            .projects
            .lock()
            .unwrap_or_else(|poison| poison.into_inner());
        projects.remove(id);
        Ok(())
    }

    async fn count(&self) -> Result<usize, StorageError> {
        let projects = self
            .projects
            .lock()
            .unwrap_or_else(|poison| poison.into_inner());
        Ok(projects.len())
    }
}

use crate::models::AccountModel;
use crate::repository::AccountRepository;

pub struct InMemoryAccountRepository {
    accounts: Arc<Mutex<HashMap<String, AccountModel>>>,
}

impl InMemoryAccountRepository {
    pub fn new() -> Self {
        Self {
            accounts: Arc::new(Mutex::new(HashMap::new())),
        }
    }
}

impl Default for InMemoryAccountRepository {
    fn default() -> Self {
        Self::new()
    }
}

impl sealed::Sealed for InMemoryAccountRepository {}

#[async_trait]
impl AccountRepository for InMemoryAccountRepository {
    async fn find_by_id(&self, id: &str) -> Result<Option<AccountModel>, StorageError> {
        let accounts = self
            .accounts
            .lock()
            .unwrap_or_else(|poison| poison.into_inner());
        Ok(accounts.get(id).cloned())
    }

    async fn find_by_username(&self, username: &str) -> Result<Option<AccountModel>, StorageError> {
        let accounts = self
            .accounts
            .lock()
            .unwrap_or_else(|poison| poison.into_inner());
        Ok(accounts.values().find(|a| a.username == username).cloned())
    }

    async fn find_by_email(&self, email: &str) -> Result<Option<AccountModel>, StorageError> {
        let accounts = self
            .accounts
            .lock()
            .unwrap_or_else(|poison| poison.into_inner());
        Ok(accounts
            .values()
            .find(|a| a.email.as_deref() == Some(email))
            .cloned())
    }

    async fn find_all(
        &self,
        limit: usize,
        offset: usize,
    ) -> Result<Vec<AccountModel>, StorageError> {
        let accounts = self
            .accounts
            .lock()
            .unwrap_or_else(|poison| poison.into_inner());
        let mut accounts: Vec<_> = accounts.values().cloned().collect();
        accounts.sort_by(|a, b| b.updated_at.cmp(&a.updated_at));
        Ok(accounts.into_iter().skip(offset).take(limit).collect())
    }

    async fn save(&self, account: &AccountModel) -> Result<(), StorageError> {
        let mut accounts = self
            .accounts
            .lock()
            .unwrap_or_else(|poison| poison.into_inner());
        accounts.insert(account.id.clone(), account.clone());
        Ok(())
    }

    async fn delete(&self, id: &str) -> Result<(), StorageError> {
        let mut accounts = self
            .accounts
            .lock()
            .unwrap_or_else(|poison| poison.into_inner());
        accounts.remove(id);
        Ok(())
    }

    async fn count(&self) -> Result<usize, StorageError> {
        let accounts = self
            .accounts
            .lock()
            .unwrap_or_else(|poison| poison.into_inner());
        Ok(accounts.len())
    }
}

use crate::models::PluginStateModel;
use crate::repository::PluginStateRepository;

pub struct InMemoryPluginStateRepository {
    states: Arc<Mutex<HashMap<String, PluginStateModel>>>,
}

impl InMemoryPluginStateRepository {
    pub fn new() -> Self {
        Self {
            states: Arc::new(Mutex::new(HashMap::new())),
        }
    }
}

impl Default for InMemoryPluginStateRepository {
    fn default() -> Self {
        Self::new()
    }
}

impl sealed::Sealed for InMemoryPluginStateRepository {}

#[async_trait]
impl PluginStateRepository for InMemoryPluginStateRepository {
    async fn find_by_id(&self, plugin_id: &str) -> Result<Option<PluginStateModel>, StorageError> {
        let states = self
            .states
            .lock()
            .unwrap_or_else(|poison| poison.into_inner());
        Ok(states.get(plugin_id).cloned())
    }

    async fn save(&self, state: &PluginStateModel) -> Result<(), StorageError> {
        let mut states = self
            .states
            .lock()
            .unwrap_or_else(|poison| poison.into_inner());
        states.insert(state.plugin_id.clone(), state.clone());
        Ok(())
    }

    async fn delete(&self, plugin_id: &str) -> Result<(), StorageError> {
        let mut states = self
            .states
            .lock()
            .unwrap_or_else(|poison| poison.into_inner());
        states.remove(plugin_id);
        Ok(())
    }
}

#[cfg(test)]
mod account_repository_tests {
    use super::*;
    use chrono::Utc;
    use uuid::Uuid;

    fn create_test_account(username: &str) -> AccountModel {
        AccountModel {
            id: Uuid::new_v4().to_string(),
            username: username.to_string(),
            email: Some(format!("{}@example.com", username)),
            password_hash: "hashed_password".to_string(),
            created_at: Utc::now(),
            updated_at: Utc::now(),
            last_login_at: None,
            is_active: true,
            data: None,
        }
    }

    #[tokio::test]
    async fn test_account_save_and_find_by_id() {
        let repo = InMemoryAccountRepository::new();
        let account = create_test_account("testuser");

        repo.save(&account).await.unwrap();
        let found = repo.find_by_id(&account.id).await.unwrap();

        assert!(found.is_some());
        assert_eq!(found.unwrap().username, "testuser");
    }

    #[tokio::test]
    async fn test_account_find_by_username() {
        let repo = InMemoryAccountRepository::new();
        let account = create_test_account("testuser");

        repo.save(&account).await.unwrap();
        let found = repo.find_by_username("testuser").await.unwrap();

        assert!(found.is_some());
        assert_eq!(found.unwrap().id, account.id);
    }

    #[tokio::test]
    async fn test_account_find_by_email() {
        let repo = InMemoryAccountRepository::new();
        let account = create_test_account("testuser");

        repo.save(&account).await.unwrap();
        let found = repo.find_by_email("testuser@example.com").await.unwrap();

        assert!(found.is_some());
        assert_eq!(found.unwrap().id, account.id);
    }

    #[tokio::test]
    async fn test_account_find_all_with_pagination() {
        let repo = InMemoryAccountRepository::new();

        for i in 0..5 {
            repo.save(&create_test_account(&format!("user{}", i)))
                .await
                .unwrap();
        }

        let all = repo.find_all(10, 0).await.unwrap();
        assert_eq!(all.len(), 5);

        let page1 = repo.find_all(2, 0).await.unwrap();
        assert_eq!(page1.len(), 2);

        let page2 = repo.find_all(2, 2).await.unwrap();
        assert_eq!(page2.len(), 2);
    }

    #[tokio::test]
    async fn test_account_delete() {
        let repo = InMemoryAccountRepository::new();
        let account = create_test_account("testuser");

        repo.save(&account).await.unwrap();
        assert!(repo.find_by_id(&account.id).await.unwrap().is_some());

        repo.delete(&account.id).await.unwrap();
        assert!(repo.find_by_id(&account.id).await.unwrap().is_none());
    }

    #[tokio::test]
    async fn test_account_count() {
        let repo = InMemoryAccountRepository::new();
        assert_eq!(repo.count().await.unwrap(), 0);

        repo.save(&create_test_account("user1")).await.unwrap();
        repo.save(&create_test_account("user2")).await.unwrap();
        assert_eq!(repo.count().await.unwrap(), 2);
    }

    #[tokio::test]
    async fn test_account_not_found() {
        let repo = InMemoryAccountRepository::new();

        assert!(repo.find_by_id("nonexistent").await.unwrap().is_none());
        assert!(repo
            .find_by_username("nonexistent")
            .await
            .unwrap()
            .is_none());
        assert!(repo
            .find_by_email("nonexistent@example.com")
            .await
            .unwrap()
            .is_none());
    }
}

#[cfg(test)]
mod plugin_state_repository_tests {
    use super::*;
    use chrono::Utc;

    fn create_test_plugin_state(plugin_id: &str) -> PluginStateModel {
        PluginStateModel {
            plugin_id: plugin_id.to_string(),
            state_data: r#"{"key": "value"}"#.to_string(),
            updated_at: Utc::now(),
        }
    }

    #[tokio::test]
    async fn test_plugin_state_save_and_find_by_id() {
        let repo = InMemoryPluginStateRepository::new();
        let state = create_test_plugin_state("test-plugin");

        repo.save(&state).await.unwrap();
        let found = repo.find_by_id("test-plugin").await.unwrap();

        assert!(found.is_some());
        assert_eq!(found.unwrap().state_data, r#"{"key": "value"}"#);
    }

    #[tokio::test]
    async fn test_plugin_state_update() {
        let repo = InMemoryPluginStateRepository::new();
        let state = create_test_plugin_state("test-plugin");
        repo.save(&state).await.unwrap();

        let updated = create_test_plugin_state("test-plugin");
        repo.save(&updated).await.unwrap();

        let found = repo.find_by_id("test-plugin").await.unwrap();
        assert!(found.is_some());
    }

    #[tokio::test]
    async fn test_plugin_state_delete() {
        let repo = InMemoryPluginStateRepository::new();
        let state = create_test_plugin_state("test-plugin");

        repo.save(&state).await.unwrap();
        assert!(repo.find_by_id("test-plugin").await.unwrap().is_some());

        repo.delete("test-plugin").await.unwrap();
        assert!(repo.find_by_id("test-plugin").await.unwrap().is_none());
    }

    #[tokio::test]
    async fn test_plugin_state_not_found() {
        let repo = InMemoryPluginStateRepository::new();
        assert!(repo.find_by_id("nonexistent").await.unwrap().is_none());
    }
}
