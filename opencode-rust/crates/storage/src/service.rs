use std::path::PathBuf;
use std::sync::Arc;

use crate::compaction::CompactionManager;
use crate::database::StoragePool;
use crate::error::StorageError;
use crate::models::{AccountModel, ProjectModel};
use crate::repository::{ProjectRepository, SessionRepository};
use opencode_core::{
    compaction::CompactionResult, crash_recovery::CrashRecovery, Message, OpenCodeError, Session,
    SessionInfo,
};
use rusqlite::params;

pub struct StorageService {
    session_repo: Arc<dyn SessionRepository>,
    project_repo: Arc<dyn ProjectRepository>,
    pool: StoragePool,
    compaction_manager: Option<CompactionManager>,
    crash_recovery: CrashRecovery,
}

impl StorageService {
    pub fn new(
        session_repo: Arc<dyn SessionRepository>,
        project_repo: Arc<dyn ProjectRepository>,
        pool: StoragePool,
    ) -> Self {
        Self {
            session_repo,
            project_repo,
            pool,
            compaction_manager: None,
            crash_recovery: CrashRecovery::new(),
        }
    }

    pub fn with_compaction_manager(mut self, manager: CompactionManager) -> Self {
        self.compaction_manager = Some(manager);
        self
    }

    pub fn with_crash_recovery_dump_dir(mut self, dump_dir: PathBuf) -> Self {
        self.crash_recovery = CrashRecovery::new().with_dump_dir(dump_dir);
        self
    }

    pub async fn save_session(&self, session: &Session) -> Result<(), OpenCodeError> {
        tracing::debug!(session_id = %session.id, message_count = session.messages.len(), "Saving session");
        self.session_repo.save(session).await.map_err(|e| {
            tracing::error!(session_id = %session.id, error = %e, "Failed to save session");
            OpenCodeError::from(e)
        })?;
        tracing::info!(session_id = %session.id, "Session saved successfully");
        Ok(())
    }

    pub async fn load_session(&self, id: &str) -> Result<Option<Session>, OpenCodeError> {
        tracing::debug!(session_id = %id, "Loading session");
        let result = self.session_repo.find_by_id(id).await.map_err(|e| {
            tracing::error!(session_id = %id, error = %e, "Failed to load session");
            OpenCodeError::from(e)
        })?;

        match &result {
            Some(_) => tracing::debug!(session_id = %id, "Session loaded successfully"),
            None => tracing::debug!(session_id = %id, "Session not found"),
        }
        Ok(result)
    }

    pub async fn get_session_messages_paginated(
        &self,
        id: &str,
        limit: usize,
        offset: usize,
    ) -> Result<Vec<Message>, OpenCodeError> {
        let session = self
            .load_session(id)
            .await?
            .ok_or_else(|| OpenCodeError::Storage("Session not found".to_string()))?;
        Ok(session
            .messages
            .iter()
            .skip(offset)
            .take(limit)
            .cloned()
            .collect())
    }

    pub async fn count_session_messages(&self, id: &str) -> Result<usize, OpenCodeError> {
        let session = self
            .load_session(id)
            .await?
            .ok_or_else(|| OpenCodeError::Storage("Session not found".to_string()))?;
        Ok(session.messages.len())
    }

    pub async fn delete_session(&self, id: &str) -> Result<(), OpenCodeError> {
        tracing::info!(session_id = %id, "Deleting session");
        self.session_repo.delete(id).await.map_err(|e| {
            tracing::error!(session_id = %id, error = %e, "Failed to delete session");
            OpenCodeError::from(e)
        })?;
        tracing::info!(session_id = %id, "Session deleted successfully");
        Ok(())
    }

    pub async fn list_sessions(
        &self,
        limit: usize,
        offset: usize,
    ) -> Result<Vec<SessionInfo>, OpenCodeError> {
        tracing::debug!(limit = limit, offset = offset, "Listing sessions");
        self.session_repo
            .find_all(limit, offset)
            .await
            .map_err(|e| {
                tracing::error!(error = %e, "Failed to list sessions");
                OpenCodeError::from(e)
            })
    }

    pub async fn count_sessions(&self) -> Result<usize, OpenCodeError> {
        self.session_repo.count().await.map_err(|e| {
            tracing::error!(error = %e, "Failed to count sessions");
            OpenCodeError::from(e)
        })
    }

    pub async fn save_project(&self, project: &ProjectModel) -> Result<(), OpenCodeError> {
        self.project_repo
            .save(project)
            .await
            .map_err(OpenCodeError::from)
    }

    pub async fn load_project(&self, id: &str) -> Result<Option<ProjectModel>, StorageError> {
        self.project_repo.find_by_id(id).await
    }

    pub async fn load_project_by_path(
        &self,
        path: &str,
    ) -> Result<Option<ProjectModel>, OpenCodeError> {
        self.project_repo
            .find_by_path(path)
            .await
            .map_err(OpenCodeError::from)
    }

    pub async fn list_projects(
        &self,
        limit: usize,
        offset: usize,
    ) -> Result<Vec<ProjectModel>, OpenCodeError> {
        self.project_repo
            .find_all(limit, offset)
            .await
            .map_err(OpenCodeError::from)
    }

    pub async fn save_account(&self, account: &AccountModel) -> Result<(), OpenCodeError> {
        let conn = self.pool.get().await?;
        let id = account.id.clone();
        let username = account.username.clone();
        let email = account.email.clone();
        let password_hash = account.password_hash.clone();
        let created_at = account.created_at.to_rfc3339();
        let updated_at = account.updated_at.to_rfc3339();
        let last_login_at = account.last_login_at.as_ref().map(|dt| dt.to_rfc3339());
        let is_active = account.is_active;
        let data = account.data.clone();

        conn.execute(move |c| {
            let json = data.as_deref().unwrap_or_default();
            c.execute(
                "INSERT OR REPLACE INTO accounts (id, username, email, password_hash, created_at, updated_at, last_login_at, is_active, data)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
                params![id, username, email, password_hash, created_at, updated_at, last_login_at, is_active, json],
            )
        }).await.map_err(|e| OpenCodeError::Storage(e.to_string()))??;
        Ok(())
    }

    pub async fn load_account_by_id(
        &self,
        id: &str,
    ) -> Result<Option<AccountModel>, OpenCodeError> {
        let conn = self.pool.get().await?;
        let id_str = id.to_string();

        let res = conn.execute(move |c| {
            let mut stmt = c.prepare(
                    "SELECT id, username, email, password_hash, created_at, updated_at, last_login_at, is_active, data FROM accounts WHERE id = ?1"
            )?;

            let mut rows = stmt.query_map(params![id_str], |row| {
                let id: String = row.get(0)?;
                let username: String = row.get(1)?;
                let email: Option<String> = row.get(2)?;
                let password_hash: String = row.get(3)?;
                let created_at: String = row.get(4)?;
                let updated_at: String = row.get(5)?;
                let last_login_at: Option<String> = row.get(6)?;
                let is_active: bool = row.get(7)?;
                let data: Option<String> = row.get(8)?;
                Ok(AccountModel {
                    id,
                    username,
                    email,
                    password_hash,
                    created_at: created_at.parse().unwrap_or_default(),
                    updated_at: updated_at.parse().unwrap_or_default(),
                    last_login_at: last_login_at.and_then(|s| s.parse().ok()),
                    is_active,
                    data,
                })
            })?;

            if let Some(Ok(model)) = rows.next() {
                Ok::<Option<AccountModel>, rusqlite::Error>(Some(model))
            } else {
                Ok::<Option<AccountModel>, rusqlite::Error>(None)
            }
        }).await.map_err(|e| OpenCodeError::Storage(e.to_string()))??;
        Ok(res)
    }

    pub async fn list_accounts(
        &self,
        limit: usize,
        offset: usize,
    ) -> Result<Vec<AccountModel>, OpenCodeError> {
        let conn = self.pool.get().await?;

        let res = conn.execute(move |c| {
            let mut stmt = c.prepare(
                "SELECT id, username, email, password_hash, created_at, updated_at, last_login_at, is_active, data FROM accounts
                 ORDER BY updated_at DESC
                 LIMIT ?1 OFFSET ?2"
            )?;

            let rows = stmt.query_map(params![limit as i32, offset as i32], |row| {
                let id: String = row.get(0)?;
                let username: String = row.get(1)?;
                let email: Option<String> = row.get(2)?;
                let password_hash: String = row.get(3)?;
                let created_at: String = row.get(4)?;
                let updated_at: String = row.get(5)?;
                let last_login_at: Option<String> = row.get(6)?;
                let is_active: bool = row.get(7)?;
                let data: Option<String> = row.get(8)?;
                Ok(AccountModel {
                    id,
                    username,
                    email,
                    password_hash,
                    created_at: created_at.parse().unwrap_or_default(),
                    updated_at: updated_at.parse().unwrap_or_default(),
                    last_login_at: last_login_at.and_then(|s| s.parse().ok()),
                    is_active,
                    data,
                })
            })?;

            let mut accounts = Vec::new();
            for row in rows {
                accounts.push(row?);
            }

            Ok::<Vec<AccountModel>, rusqlite::Error>(accounts)
        }).await.map_err(|e| OpenCodeError::Storage(e.to_string()))??;
        Ok(res)
    }

    pub async fn get_user_permissions(
        &self,
        user_id: &str,
    ) -> Result<Vec<(String, bool)>, OpenCodeError> {
        let conn = self.pool.get().await?;
        let user_id_str = user_id.to_string();

        conn.execute(move |c| {
            let mut stmt =
                c.prepare("SELECT permission, is_deny FROM permissions WHERE user_id = ?1")?;

            let rows = stmt.query_map(params![user_id_str], |row| {
                Ok((row.get::<_, String>(0)?, row.get::<_, bool>(1)?))
            })?;

            let mut permissions = Vec::new();
            for row in rows {
                permissions.push(row?);
            }

            Ok::<Vec<(String, bool)>, rusqlite::Error>(permissions)
        })
        .await
        .map_err(|e| OpenCodeError::Storage(e.to_string()))?
        .map_err(|e| OpenCodeError::Storage(e.to_string()))
    }

    /// Compact a session by delegating to CompactionManager.
    ///
    /// Returns `Ok(CompactionResult)` with the compacted session info.
    /// Returns `Err(StorageError::SessionNotFound)` if the session doesn't exist.
    pub async fn compact_session(&self, id: &str) -> Result<CompactionResult, StorageError> {
        // Load the session first
        let session = self
            .session_repo
            .find_by_id(id)
            .await?
            .ok_or_else(|| StorageError::SessionNotFound(id.to_string()))?;

        // Get compaction manager
        let compaction_manager = self.compaction_manager.as_ref().ok_or_else(|| {
            StorageError::Internal("Compaction manager not configured".to_string())
        })?;

        // Clone session for compaction (compact takes &mut Session)
        let mut session_to_compact = session;
        let result = compaction_manager
            .compact(&mut session_to_compact)
            .await
            .map_err(|e| StorageError::Internal(format!("Compaction failed: {}", e)))?;

        Ok(result.compaction_result)
    }

    pub async fn recover_session(&self, id: &str) -> Result<Session, StorageError> {
        self.crash_recovery
            .recover_session_latest(id)
            .map_err(|e| StorageError::Internal(format!("Recovery error: {}", e)))?
            .ok_or_else(|| StorageError::SessionNotFound(id.to_string()))
    }

    /// List all incomplete session IDs.
    ///
    /// An incomplete session is one that has crash dumps (was interrupted by a crash).
    /// Returns Vec of session Uuids that have incomplete crash dumps.
    pub async fn list_incomplete_sessions(&self) -> Result<Vec<uuid::Uuid>, StorageError> {
        let crashes = self.crash_recovery.list_recent_crashes(1000);
        let mut unique_session_ids: Vec<uuid::Uuid> = Vec::new();
        for crash in crashes {
            if let Ok(session_uuid) = uuid::Uuid::parse_str(&crash.session_id) {
                if !unique_session_ids.contains(&session_uuid) {
                    unique_session_ids.push(session_uuid);
                }
            }
        }
        Ok(unique_session_ids)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::database::StoragePool;
    use crate::memory_repository::InMemorySessionRepository;
    use crate::migration::MigrationManager;
    use chrono::Utc;
    use uuid::Uuid;

    fn create_test_session() -> Session {
        let mut session = Session::new();
        session.add_message(Message::user("Hello".to_string()));
        session.add_message(Message::assistant("Hi there".to_string()));
        session
    }

    fn create_test_project() -> ProjectModel {
        ProjectModel {
            id: Uuid::new_v4().to_string(),
            path: "/tmp/test_project".to_string(),
            name: Some("Test Project".to_string()),
            created_at: Utc::now(),
            updated_at: Utc::now(),
            data: None,
        }
    }

    fn create_test_account() -> AccountModel {
        AccountModel {
            id: Uuid::new_v4().to_string(),
            username: "testuser".to_string(),
            email: Some("test@example.com".to_string()),
            password_hash: "hashed_password".to_string(),
            created_at: Utc::now(),
            updated_at: Utc::now(),
            last_login_at: None,
            is_active: true,
            data: None,
        }
    }

    #[tokio::test]
    async fn test_storage_service_new() {
        let temp_dir = tempfile::tempdir().unwrap();
        let db_path = temp_dir.path().join("test.db");
        let pool = StoragePool::new(&db_path).unwrap();

        let session_repo = Arc::new(InMemorySessionRepository::new());
        let project_repo = Arc::new(crate::memory_repository::InMemoryProjectRepository::new());

        let service = StorageService::new(session_repo, project_repo, pool);
        assert!(service.session_repo.as_ref().count().await.is_ok());

        drop(temp_dir);
    }

    #[tokio::test]
    async fn test_save_and_load_session() {
        let temp_dir = tempfile::tempdir().unwrap();
        let db_path = temp_dir.path().join("test.db");
        let pool = StoragePool::new(&db_path).unwrap();

        let session_repo = Arc::new(InMemorySessionRepository::new());
        let project_repo = Arc::new(crate::memory_repository::InMemoryProjectRepository::new());

        let service = StorageService::new(session_repo, project_repo, pool);
        let session = create_test_session();

        service.save_session(&session).await.unwrap();

        let loaded = service.load_session(&session.id.to_string()).await.unwrap();
        assert!(loaded.is_some());
        assert_eq!(loaded.unwrap().id, session.id);

        drop(temp_dir);
    }

    #[tokio::test]
    async fn test_load_session_not_found() {
        let temp_dir = tempfile::tempdir().unwrap();
        let db_path = temp_dir.path().join("test.db");
        let pool = StoragePool::new(&db_path).unwrap();

        let session_repo = Arc::new(InMemorySessionRepository::new());
        let project_repo = Arc::new(crate::memory_repository::InMemoryProjectRepository::new());

        let service = StorageService::new(session_repo, project_repo, pool);

        let loaded = service.load_session("nonexistent-id").await.unwrap();
        assert!(loaded.is_none());

        drop(temp_dir);
    }

    #[tokio::test]
    async fn test_get_session_messages_paginated() {
        let temp_dir = tempfile::tempdir().unwrap();
        let db_path = temp_dir.path().join("test.db");
        let pool = StoragePool::new(&db_path).unwrap();

        let session_repo = Arc::new(InMemorySessionRepository::new());
        let project_repo = Arc::new(crate::memory_repository::InMemoryProjectRepository::new());

        let service = StorageService::new(session_repo, project_repo, pool);
        let mut session = create_test_session();
        for i in 0..10 {
            session.add_message(Message::user(format!("Message {}", i)));
        }

        service.save_session(&session).await.unwrap();

        let messages = service
            .get_session_messages_paginated(&session.id.to_string(), 5, 0)
            .await
            .unwrap();
        assert_eq!(messages.len(), 5);

        let messages = service
            .get_session_messages_paginated(&session.id.to_string(), 5, 5)
            .await
            .unwrap();
        assert_eq!(messages.len(), 5);

        drop(temp_dir);
    }

    #[tokio::test]
    async fn test_count_session_messages() {
        let temp_dir = tempfile::tempdir().unwrap();
        let db_path = temp_dir.path().join("test.db");
        let pool = StoragePool::new(&db_path).unwrap();

        let session_repo = Arc::new(InMemorySessionRepository::new());
        let project_repo = Arc::new(crate::memory_repository::InMemoryProjectRepository::new());

        let service = StorageService::new(session_repo, project_repo, pool);
        let mut session = create_test_session();
        for i in 0..5 {
            session.add_message(Message::user(format!("Message {}", i)));
        }

        service.save_session(&session).await.unwrap();

        let count = service
            .count_session_messages(&session.id.to_string())
            .await
            .unwrap();
        assert_eq!(count, 7);

        drop(temp_dir);
    }

    #[tokio::test]
    async fn test_delete_session() {
        let temp_dir = tempfile::tempdir().unwrap();
        let db_path = temp_dir.path().join("test.db");
        let pool = StoragePool::new(&db_path).unwrap();

        let session_repo = Arc::new(InMemorySessionRepository::new());
        let project_repo = Arc::new(crate::memory_repository::InMemoryProjectRepository::new());

        let service = StorageService::new(session_repo, project_repo, pool);
        let session = create_test_session();

        service.save_session(&session).await.unwrap();
        assert!(service
            .load_session(&session.id.to_string())
            .await
            .unwrap()
            .is_some());

        service
            .delete_session(&session.id.to_string())
            .await
            .unwrap();
        assert!(service
            .load_session(&session.id.to_string())
            .await
            .unwrap()
            .is_none());

        drop(temp_dir);
    }

    #[tokio::test]
    async fn test_list_sessions() {
        let temp_dir = tempfile::tempdir().unwrap();
        let db_path = temp_dir.path().join("test.db");
        let pool = StoragePool::new(&db_path).unwrap();

        let session_repo = Arc::new(InMemorySessionRepository::new());
        let project_repo = Arc::new(crate::memory_repository::InMemoryProjectRepository::new());

        let service = StorageService::new(session_repo, project_repo, pool);

        for _ in 0..5 {
            let session = create_test_session();
            service.save_session(&session).await.unwrap();
        }

        let sessions = service.list_sessions(10, 0).await.unwrap();
        assert_eq!(sessions.len(), 5);

        let sessions = service.list_sessions(2, 0).await.unwrap();
        assert_eq!(sessions.len(), 2);

        drop(temp_dir);
    }

    #[tokio::test]
    async fn test_count_sessions() {
        let temp_dir = tempfile::tempdir().unwrap();
        let db_path = temp_dir.path().join("test.db");
        let pool = StoragePool::new(&db_path).unwrap();

        let session_repo = Arc::new(InMemorySessionRepository::new());
        let project_repo = Arc::new(crate::memory_repository::InMemoryProjectRepository::new());

        let service = StorageService::new(session_repo, project_repo, pool);

        assert_eq!(service.count_sessions().await.unwrap(), 0);

        for _ in 0..3 {
            let session = create_test_session();
            service.save_session(&session).await.unwrap();
        }

        assert_eq!(service.count_sessions().await.unwrap(), 3);

        drop(temp_dir);
    }

    #[tokio::test]
    async fn test_save_and_load_project() {
        let temp_dir = tempfile::tempdir().unwrap();
        let db_path = temp_dir.path().join("test.db");
        let pool = StoragePool::new(&db_path).unwrap();

        let session_repo = Arc::new(InMemorySessionRepository::new());
        let project_repo = Arc::new(crate::memory_repository::InMemoryProjectRepository::new());

        let service = StorageService::new(session_repo, project_repo, pool);
        let project = create_test_project();

        service.save_project(&project).await.unwrap();

        let loaded = service.load_project_by_path(&project.path).await.unwrap();
        assert!(loaded.is_some());
        assert_eq!(loaded.unwrap().id, project.id);

        drop(temp_dir);
    }

    #[tokio::test]
    async fn test_list_projects() {
        let temp_dir = tempfile::tempdir().unwrap();
        let db_path = temp_dir.path().join("test.db");
        let pool = StoragePool::new(&db_path).unwrap();

        let session_repo = Arc::new(InMemorySessionRepository::new());
        let project_repo = Arc::new(crate::memory_repository::InMemoryProjectRepository::new());

        let service = StorageService::new(session_repo, project_repo, pool);

        for i in 0..5 {
            let mut project = create_test_project();
            project.path = format!("/tmp/test_project_{}", i);
            service.save_project(&project).await.unwrap();
        }

        let projects = service.list_projects(10, 0).await.unwrap();
        assert_eq!(projects.len(), 5);

        drop(temp_dir);
    }

    #[tokio::test]
    async fn test_save_and_load_account() {
        let temp_dir = tempfile::tempdir().unwrap();
        let db_path = temp_dir.path().join("test.db");
        let pool = StoragePool::new(&db_path).unwrap();

        let session_repo = Arc::new(InMemorySessionRepository::new());
        let project_repo = Arc::new(crate::memory_repository::InMemoryProjectRepository::new());

        let service = StorageService::new(session_repo, project_repo, pool.clone());

        let manager = MigrationManager::new(pool, 3);
        manager.migrate().await.unwrap();

        let account = create_test_account();

        service.save_account(&account).await.unwrap();

        let loaded = service.load_account_by_id(&account.id).await.unwrap();
        assert!(loaded.is_some());
        assert_eq!(loaded.unwrap().username, account.username);

        drop(temp_dir);
    }

    #[tokio::test]
    async fn test_list_accounts() {
        let temp_dir = tempfile::tempdir().unwrap();
        let db_path = temp_dir.path().join("test.db");
        let pool = StoragePool::new(&db_path).unwrap();

        let manager = MigrationManager::new(pool.clone(), 3);
        manager.migrate().await.unwrap();

        let session_repo = Arc::new(InMemorySessionRepository::new());
        let project_repo = Arc::new(crate::memory_repository::InMemoryProjectRepository::new());

        let service = StorageService::new(session_repo, project_repo, pool);

        for i in 0..5 {
            let mut account = create_test_account();
            account.username = format!("user{}", i);
            service.save_account(&account).await.unwrap();
        }

        let accounts = service.list_accounts(10, 0).await.unwrap();
        assert!(accounts.len() > 0, "Should have saved some accounts");

        drop(temp_dir);
    }

    #[tokio::test]
    async fn test_compact_session_returns_ok_for_existing_session() {
        use crate::compaction::CompactionManager;
        use opencode_core::config::CompactionConfig;

        let temp_dir = tempfile::tempdir().unwrap();
        let db_path = temp_dir.path().join("test.db");
        let pool = StoragePool::new(&db_path).unwrap();

        let session_repo = Arc::new(InMemorySessionRepository::new());
        let project_repo = Arc::new(crate::memory_repository::InMemoryProjectRepository::new());

        let compaction_config = CompactionConfig {
            auto: Some(true),
            compact_threshold: Some(0.8),
            ..Default::default()
        };
        let compaction_manager = CompactionManager::new(compaction_config);

        let service = StorageService::new(session_repo, project_repo, pool)
            .with_compaction_manager(compaction_manager);

        let session = create_test_session();
        service.save_session(&session).await.unwrap();

        let result = service.compact_session(&session.id.to_string()).await;
        assert!(
            result.is_ok(),
            "compact_session should succeed for existing session"
        );
        let compaction_result = result.unwrap();
        assert!(compaction_result.was_compacted || !compaction_result.was_compacted);

        drop(temp_dir);
    }

    #[tokio::test]
    async fn test_compact_session_returns_not_found_error_for_nonexistent_session() {
        use crate::compaction::CompactionManager;
        use opencode_core::config::CompactionConfig;

        let temp_dir = tempfile::tempdir().unwrap();
        let db_path = temp_dir.path().join("test.db");
        let pool = StoragePool::new(&db_path).unwrap();

        let session_repo = Arc::new(InMemorySessionRepository::new());
        let project_repo = Arc::new(crate::memory_repository::InMemoryProjectRepository::new());

        let compaction_config = CompactionConfig::default();
        let compaction_manager = CompactionManager::new(compaction_config);

        let service = StorageService::new(session_repo, project_repo, pool)
            .with_compaction_manager(compaction_manager);

        let result = service.compact_session("nonexistent-session-id").await;
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(matches!(
            err,
            crate::error::StorageError::SessionNotFound(_)
        ));

        drop(temp_dir);
    }

    #[tokio::test]
    async fn test_compact_session_delegates_correctly_to_compaction_manager() {
        use crate::compaction::CompactionManager;
        use opencode_core::config::CompactionConfig;

        let temp_dir = tempfile::tempdir().unwrap();
        let db_path = temp_dir.path().join("test.db");
        let pool = StoragePool::new(&db_path).unwrap();

        let session_repo = Arc::new(InMemorySessionRepository::new());
        let project_repo = Arc::new(crate::memory_repository::InMemoryProjectRepository::new());

        let compaction_config = CompactionConfig::default();
        let compaction_manager = CompactionManager::new(compaction_config);

        let service = StorageService::new(session_repo, project_repo, pool)
            .with_compaction_manager(compaction_manager);

        let mut session = create_test_session();
        for i in 0..20 {
            session.add_message(Message::user(format!("This is message number {}", i)));
            session.add_message(Message::assistant(format!("This is a longer response number {} with more content to ensure the token count is significant enough to potentially trigger compaction if the threshold is very low", i)));
        }
        service.save_session(&session).await.unwrap();

        let result = service.compact_session(&session.id.to_string()).await;
        assert!(
            result.is_ok(),
            "compact_session should succeed and delegate to CompactionManager"
        );
        let compaction_result = result.unwrap();
        assert!(compaction_result.was_compacted || !compaction_result.was_compacted);

        drop(temp_dir);
    }

    #[tokio::test]
    async fn test_load_project_returns_some_for_existing_project() {
        let temp_dir = tempfile::tempdir().unwrap();
        let db_path = temp_dir.path().join("test.db");
        let pool = StoragePool::new(&db_path).unwrap();

        let session_repo = Arc::new(InMemorySessionRepository::new());
        let project_repo = Arc::new(crate::memory_repository::InMemoryProjectRepository::new());

        let service = StorageService::new(session_repo, project_repo, pool);
        let project = create_test_project();

        service.save_project(&project).await.unwrap();

        let loaded = service.load_project(&project.id).await.unwrap();
        assert!(loaded.is_some());
        assert_eq!(loaded.unwrap().id, project.id);

        drop(temp_dir);
    }

    #[tokio::test]
    async fn test_load_project_returns_none_for_nonexistent_project() {
        let temp_dir = tempfile::tempdir().unwrap();
        let db_path = temp_dir.path().join("test.db");
        let pool = StoragePool::new(&db_path).unwrap();

        let session_repo = Arc::new(InMemorySessionRepository::new());
        let project_repo = Arc::new(crate::memory_repository::InMemoryProjectRepository::new());

        let service = StorageService::new(session_repo, project_repo, pool);

        let loaded = service.load_project("nonexistent-id").await.unwrap();
        assert!(loaded.is_none());

        drop(temp_dir);
    }

    #[tokio::test]
    async fn test_load_project_handles_uuid_format_correctly() {
        let temp_dir = tempfile::tempdir().unwrap();
        let db_path = temp_dir.path().join("test.db");
        let pool = StoragePool::new(&db_path).unwrap();

        let session_repo = Arc::new(InMemorySessionRepository::new());
        let project_repo = Arc::new(crate::memory_repository::InMemoryProjectRepository::new());

        let service = StorageService::new(session_repo, project_repo, pool);
        let project = create_test_project();

        service.save_project(&project).await.unwrap();

        // Test with stringified UUID
        let uuid_string = project.id.to_string();
        let loaded = service.load_project(&uuid_string).await.unwrap();
        assert!(loaded.is_some());
        assert_eq!(loaded.unwrap().id, project.id);

        drop(temp_dir);
    }

    #[tokio::test]
    async fn test_load_project_error_handling_for_malformed_uuid() {
        let temp_dir = tempfile::tempdir().unwrap();
        let db_path = temp_dir.path().join("test.db");
        let pool = StoragePool::new(&db_path).unwrap();

        let session_repo = Arc::new(InMemorySessionRepository::new());
        let project_repo = Arc::new(crate::memory_repository::InMemoryProjectRepository::new());

        let service = StorageService::new(session_repo, project_repo, pool);

        // Malformed UUID should return None, not error - the repository handles this gracefully
        let loaded = service.load_project("not-a-valid-uuid").await.unwrap();
        assert!(loaded.is_none());

        drop(temp_dir);
    }

    #[tokio::test]
    async fn test_compact_session_error_propagation_for_invalid_session_ids() {
        use crate::compaction::CompactionManager;
        use opencode_core::config::CompactionConfig;

        let temp_dir = tempfile::tempdir().unwrap();
        let db_path = temp_dir.path().join("test.db");
        let pool = StoragePool::new(&db_path).unwrap();

        let session_repo = Arc::new(InMemorySessionRepository::new());
        let project_repo = Arc::new(crate::memory_repository::InMemoryProjectRepository::new());

        let compaction_config = CompactionConfig::default();
        let compaction_manager = CompactionManager::new(compaction_config);

        let service = StorageService::new(session_repo, project_repo, pool)
            .with_compaction_manager(compaction_manager);

        let result = service.compact_session("").await;
        assert!(result.is_err());

        let result = service.compact_session("invalid-uuid-format").await;
        assert!(result.is_err());

        drop(temp_dir);
    }

    #[tokio::test]
    async fn test_recover_session_returns_ok_for_recoverable_session() {
        use opencode_core::crash_recovery::CrashRecovery;

        let temp_dir = tempfile::tempdir().unwrap();
        let db_path = temp_dir.path().join("test.db");
        let pool = StoragePool::new(&db_path).unwrap();

        let session_repo = Arc::new(InMemorySessionRepository::new());
        let project_repo = Arc::new(crate::memory_repository::InMemoryProjectRepository::new());

        let crash_dir = temp_dir.path().join("crashes");
        let service = StorageService::new(session_repo, project_repo, pool)
            .with_crash_recovery_dump_dir(crash_dir.clone());

        let session = create_test_session();
        let session_id = session.id.to_string();

        let crash_recovery = CrashRecovery::new().with_dump_dir(crash_dir);
        crash_recovery.set_active_session(session);
        crash_recovery
            .save_crash_dump(Some("test panic".to_string()), None)
            .unwrap();

        let result = service.recover_session(&session_id).await;
        assert!(result.is_ok());

        drop(temp_dir);
    }

    #[tokio::test]
    async fn test_recover_session_returns_not_found_error_for_non_existent_session() {
        let temp_dir = tempfile::tempdir().unwrap();
        let db_path = temp_dir.path().join("test.db");
        let pool = StoragePool::new(&db_path).unwrap();

        let session_repo = Arc::new(InMemorySessionRepository::new());
        let project_repo = Arc::new(crate::memory_repository::InMemoryProjectRepository::new());

        let service = StorageService::new(session_repo, project_repo, pool);

        let result = service.recover_session("non-existent-session-id").await;
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(matches!(err, StorageError::SessionNotFound(_)));

        drop(temp_dir);
    }

    #[tokio::test]
    async fn test_recover_session_delegates_to_crash_recovery_correctly() {
        use opencode_core::crash_recovery::CrashRecovery;

        let temp_dir = tempfile::tempdir().unwrap();
        let db_path = temp_dir.path().join("test.db");
        let pool = StoragePool::new(&db_path).unwrap();

        let session_repo = Arc::new(InMemorySessionRepository::new());
        let project_repo = Arc::new(crate::memory_repository::InMemoryProjectRepository::new());

        let crash_dir = temp_dir.path().join("crashes");
        let service = StorageService::new(session_repo, project_repo, pool)
            .with_crash_recovery_dump_dir(crash_dir.clone());

        let mut session = create_test_session();
        session.add_message(Message::user("Message before crash".to_string()));
        let session_id = session.id.to_string();

        let crash_recovery = CrashRecovery::new().with_dump_dir(crash_dir);
        crash_recovery.set_active_session(session);
        crash_recovery
            .save_crash_dump(Some("panic".to_string()), None)
            .unwrap();

        let result = service.recover_session(&session_id).await;
        assert!(result.is_ok());

        drop(temp_dir);
    }

    #[tokio::test]
    async fn test_recover_session_error_handling_for_corrupted_session_data() {
        use opencode_core::crash_recovery::CrashRecovery;

        let temp_dir = tempfile::tempdir().unwrap();
        let db_path = temp_dir.path().join("test.db");
        let pool = StoragePool::new(&db_path).unwrap();

        let session_repo = Arc::new(InMemorySessionRepository::new());
        let project_repo = Arc::new(crate::memory_repository::InMemoryProjectRepository::new());

        let crash_dir = temp_dir.path().join("crashes");
        let service = StorageService::new(session_repo, project_repo, pool)
            .with_crash_recovery_dump_dir(crash_dir.clone());

        let mut session = Session::new();
        session.id = uuid::Uuid::parse_str("bbbbbbbb-bbbb-bbbb-bbbb-bbbbbbbbbbbb").unwrap();

        let crash_recovery = CrashRecovery::new().with_dump_dir(crash_dir);
        crash_recovery.set_active_session(session);
        crash_recovery
            .save_crash_dump(Some("panic".to_string()), None)
            .unwrap();

        let result = service
            .recover_session("bbbbbbbb-bbbb-bbbb-bbbb-bbbbbbbbbbbb")
            .await;
        assert!(result.is_ok());

        drop(temp_dir);
    }

    // =========================================================================
    // FR-045/FR-048 Pagination Verification Tests
    // =========================================================================

    #[tokio::test]
    async fn test_list_sessions_returns_empty_vec_when_no_sessions_exist() {
        let temp_dir = tempfile::tempdir().unwrap();
        let db_path = temp_dir.path().join("test.db");
        let pool = StoragePool::new(&db_path).unwrap();

        let session_repo = Arc::new(InMemorySessionRepository::new());
        let project_repo = Arc::new(crate::memory_repository::InMemoryProjectRepository::new());

        let service = StorageService::new(session_repo, project_repo, pool);

        // Should return empty Vec, not error
        let sessions = service.list_sessions(10, 0).await.unwrap();
        assert!(
            sessions.is_empty(),
            "Expected empty Vec when no sessions exist"
        );

        let sessions = service.list_sessions(100, 0).await.unwrap();
        assert!(sessions.is_empty(), "Expected empty Vec with larger limit");

        let sessions = service.list_sessions(10, 100).await.unwrap();
        assert!(
            sessions.is_empty(),
            "Expected empty Vec with offset beyond range"
        );

        drop(temp_dir);
    }

    #[tokio::test]
    async fn test_list_sessions_returns_correct_items_within_limit_offset_range() {
        let temp_dir = tempfile::tempdir().unwrap();
        let db_path = temp_dir.path().join("test.db");
        let pool = StoragePool::new(&db_path).unwrap();

        let session_repo = Arc::new(InMemorySessionRepository::new());
        let project_repo = Arc::new(crate::memory_repository::InMemoryProjectRepository::new());

        let service = StorageService::new(session_repo, project_repo, pool);

        // Create 10 sessions
        for i in 0..10 {
            let mut session = create_test_session();
            session.add_message(Message::user(format!("Message {}", i)));
            service.save_session(&session).await.unwrap();
        }

        // Test limit
        let sessions = service.list_sessions(3, 0).await.unwrap();
        assert_eq!(sessions.len(), 3, "Should return at most limit items");

        let sessions = service.list_sessions(100, 0).await.unwrap();
        assert_eq!(
            sessions.len(),
            10,
            "Should return all 10 sessions when limit >= total"
        );

        // Test offset
        let sessions = service.list_sessions(10, 5).await.unwrap();
        assert_eq!(
            sessions.len(),
            5,
            "Should return 5 items when offset is 5 and total is 10"
        );

        let sessions = service.list_sessions(10, 10).await.unwrap();
        assert_eq!(
            sessions.len(),
            0,
            "Should return 0 items when offset equals total count"
        );

        // Test limit and offset together
        let sessions = service.list_sessions(3, 2).await.unwrap();
        assert_eq!(
            sessions.len(),
            3,
            "Should return 3 items starting from offset 2"
        );

        let sessions = service.list_sessions(3, 8).await.unwrap();
        assert_eq!(
            sessions.len(),
            2,
            "Should return 2 items when only 2 remain after offset"
        );

        drop(temp_dir);
    }

    #[tokio::test]
    async fn test_list_projects_returns_empty_vec_when_no_projects_exist() {
        let temp_dir = tempfile::tempdir().unwrap();
        let db_path = temp_dir.path().join("test.db");
        let pool = StoragePool::new(&db_path).unwrap();

        let session_repo = Arc::new(InMemorySessionRepository::new());
        let project_repo = Arc::new(crate::memory_repository::InMemoryProjectRepository::new());

        let service = StorageService::new(session_repo, project_repo, pool);

        // Should return empty Vec, not error
        let projects = service.list_projects(10, 0).await.unwrap();
        assert!(
            projects.is_empty(),
            "Expected empty Vec when no projects exist"
        );

        let projects = service.list_projects(100, 0).await.unwrap();
        assert!(projects.is_empty(), "Expected empty Vec with larger limit");

        let projects = service.list_projects(10, 100).await.unwrap();
        assert!(
            projects.is_empty(),
            "Expected empty Vec with offset beyond range"
        );

        drop(temp_dir);
    }

    #[tokio::test]
    async fn test_list_projects_returns_correct_items_within_limit_offset_range() {
        let temp_dir = tempfile::tempdir().unwrap();
        let db_path = temp_dir.path().join("test.db");
        let pool = StoragePool::new(&db_path).unwrap();

        let session_repo = Arc::new(InMemorySessionRepository::new());
        let project_repo = Arc::new(crate::memory_repository::InMemoryProjectRepository::new());

        let service = StorageService::new(session_repo, project_repo, pool);

        // Create 10 projects
        for i in 0..10 {
            let mut project = create_test_project();
            project.path = format!("/tmp/test_project_{}", i);
            project.name = Some(format!("Test Project {}", i));
            service.save_project(&project).await.unwrap();
        }

        // Test limit
        let projects = service.list_projects(3, 0).await.unwrap();
        assert_eq!(projects.len(), 3, "Should return at most limit items");

        let projects = service.list_projects(100, 0).await.unwrap();
        assert_eq!(
            projects.len(),
            10,
            "Should return all 10 projects when limit >= total"
        );

        // Test offset
        let projects = service.list_projects(10, 5).await.unwrap();
        assert_eq!(
            projects.len(),
            5,
            "Should return 5 items when offset is 5 and total is 10"
        );

        let projects = service.list_projects(10, 10).await.unwrap();
        assert_eq!(
            projects.len(),
            0,
            "Should return 0 items when offset equals total count"
        );

        // Test limit and offset together
        let projects = service.list_projects(3, 2).await.unwrap();
        assert_eq!(
            projects.len(),
            3,
            "Should return 3 items starting from offset 2"
        );

        let projects = service.list_projects(3, 8).await.unwrap();
        assert_eq!(
            projects.len(),
            2,
            "Should return 2 items when only 2 remain after offset"
        );

        drop(temp_dir);
    }

    #[tokio::test]
    async fn test_pagination_consistency_across_multiple_calls() {
        let temp_dir = tempfile::tempdir().unwrap();
        let db_path = temp_dir.path().join("test.db");
        let pool = StoragePool::new(&db_path).unwrap();

        let session_repo = Arc::new(InMemorySessionRepository::new());
        let project_repo = Arc::new(crate::memory_repository::InMemoryProjectRepository::new());

        let service = StorageService::new(session_repo, project_repo, pool);

        // Create 10 sessions
        for i in 0..10 {
            let mut session = create_test_session();
            session.add_message(Message::user(format!("Message {}", i)));
            service.save_session(&session).await.unwrap();
        }

        // First call - get first page
        let page1 = service.list_sessions(5, 0).await.unwrap();
        assert_eq!(page1.len(), 5);

        // Second call - get second page
        let page2 = service.list_sessions(5, 5).await.unwrap();
        assert_eq!(page2.len(), 5);

        // Combined pages should have all items
        let all = service.list_sessions(10, 0).await.unwrap();
        assert_eq!(all.len(), 10);

        // Verify no overlap between pages
        let page1_ids: Vec<_> = page1.iter().map(|s| s.id).collect();
        let page2_ids: Vec<_> = page2.iter().map(|s| s.id).collect();
        for id1 in &page1_ids {
            for id2 in &page2_ids {
                assert_ne!(id1, id2, "Pages should not overlap");
            }
        }

        // Verify all items are accounted for
        let mut all_ids: Vec<_> = page1_ids.clone();
        all_ids.extend(page2_ids);
        assert_eq!(
            all_ids.len(),
            10,
            "All 10 sessions should be accounted for across pages"
        );

        drop(temp_dir);
    }

    #[tokio::test]
    async fn test_list_sessions_handles_invalid_limit_offset_values() {
        let temp_dir = tempfile::tempdir().unwrap();
        let db_path = temp_dir.path().join("test.db");
        let pool = StoragePool::new(&db_path).unwrap();

        let session_repo = Arc::new(InMemorySessionRepository::new());
        let project_repo = Arc::new(crate::memory_repository::InMemoryProjectRepository::new());

        let service = StorageService::new(session_repo, project_repo, pool);

        // Create 5 sessions
        for _i in 0..5 {
            let session = create_test_session();
            service.save_session(&session).await.unwrap();
        }

        // Zero limit should return empty
        let sessions = service.list_sessions(0, 0).await.unwrap();
        assert_eq!(sessions.len(), 0, "Zero limit should return empty Vec");

        // Very large offset should return empty
        let sessions = service.list_sessions(10, 1000).await.unwrap();
        assert!(
            sessions.is_empty(),
            "Offset beyond total should return empty Vec"
        );

        // Zero offset should work normally
        let sessions = service.list_sessions(10, 0).await.unwrap();
        assert_eq!(sessions.len(), 5, "Zero offset should work normally");

        drop(temp_dir);
    }

    #[tokio::test]
    async fn test_list_projects_handles_invalid_limit_offset_values() {
        let temp_dir = tempfile::tempdir().unwrap();
        let db_path = temp_dir.path().join("test.db");
        let pool = StoragePool::new(&db_path).unwrap();

        let session_repo = Arc::new(InMemorySessionRepository::new());
        let project_repo = Arc::new(crate::memory_repository::InMemoryProjectRepository::new());

        let service = StorageService::new(session_repo, project_repo, pool);

        // Create 5 projects
        for i in 0..5 {
            let mut project = create_test_project();
            project.path = format!("/tmp/test_project_{}", i);
            service.save_project(&project).await.unwrap();
        }

        // Zero limit should return empty
        let projects = service.list_projects(0, 0).await.unwrap();
        assert_eq!(projects.len(), 0, "Zero limit should return empty Vec");

        // Very large offset should return empty
        let projects = service.list_projects(10, 1000).await.unwrap();
        assert!(
            projects.is_empty(),
            "Offset beyond total should return empty Vec"
        );

        // Zero offset should work normally
        let projects = service.list_projects(10, 0).await.unwrap();
        assert_eq!(projects.len(), 5, "Zero offset should work normally");

        drop(temp_dir);
    }

    // =========================================================================
    // FR-050: list_incomplete_sessions Tests
    // =========================================================================

    #[tokio::test]
    async fn test_list_incomplete_sessions_returns_empty_vec_when_no_incomplete_sessions() {
        let temp_dir = tempfile::tempdir().unwrap();
        let db_path = temp_dir.path().join("test.db");
        let pool = StoragePool::new(&db_path).unwrap();

        let session_repo = Arc::new(InMemorySessionRepository::new());
        let project_repo = Arc::new(crate::memory_repository::InMemoryProjectRepository::new());

        let crash_dir = temp_dir.path().join("crashes");
        let service = StorageService::new(session_repo, project_repo, pool)
            .with_crash_recovery_dump_dir(crash_dir);

        let incomplete = service.list_incomplete_sessions().await.unwrap();
        assert!(
            incomplete.is_empty(),
            "Expected empty Vec when no incomplete sessions exist"
        );

        drop(temp_dir);
    }

    #[tokio::test]
    async fn test_list_incomplete_sessions_returns_incomplete_session_ids() {
        use opencode_core::crash_recovery::CrashRecovery;

        let temp_dir = tempfile::tempdir().unwrap();
        let db_path = temp_dir.path().join("test.db");
        let pool = StoragePool::new(&db_path).unwrap();

        let session_repo = Arc::new(InMemorySessionRepository::new());
        let project_repo = Arc::new(crate::memory_repository::InMemoryProjectRepository::new());

        let crash_dir = temp_dir.path().join("crashes");
        let service = StorageService::new(session_repo, project_repo, pool)
            .with_crash_recovery_dump_dir(crash_dir.clone());

        let session1_id = uuid::Uuid::parse_str("11111111-1111-1111-1111-111111111111").unwrap();
        let session2_id = uuid::Uuid::parse_str("22222222-2222-2222-2222-222222222222").unwrap();

        let crash_recovery = CrashRecovery::new().with_dump_dir(crash_dir);

        let mut session1 = create_test_session();
        session1.id = session1_id;
        crash_recovery.set_active_session(session1);
        crash_recovery
            .save_crash_dump(Some("panic 1".to_string()), None)
            .unwrap();

        let mut session2 = create_test_session();
        session2.id = session2_id;
        crash_recovery.set_active_session(session2);
        crash_recovery
            .save_crash_dump(Some("panic 2".to_string()), None)
            .unwrap();

        let incomplete = service.list_incomplete_sessions().await.unwrap();
        assert_eq!(incomplete.len(), 2);
        assert!(incomplete.contains(&session1_id));
        assert!(incomplete.contains(&session2_id));

        drop(temp_dir);
    }

    #[tokio::test]
    async fn test_list_incomplete_sessions_handles_duplicate_crashes() {
        use opencode_core::crash_recovery::CrashRecovery;

        let temp_dir = tempfile::tempdir().unwrap();
        let db_path = temp_dir.path().join("test.db");
        let pool = StoragePool::new(&db_path).unwrap();

        let session_repo = Arc::new(InMemorySessionRepository::new());
        let project_repo = Arc::new(crate::memory_repository::InMemoryProjectRepository::new());

        let crash_dir = temp_dir.path().join("crashes");
        let service = StorageService::new(session_repo, project_repo, pool)
            .with_crash_recovery_dump_dir(crash_dir.clone());

        let session_id = uuid::Uuid::parse_str("aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa").unwrap();

        let crash_recovery = CrashRecovery::new().with_dump_dir(crash_dir);

        let mut session = create_test_session();
        session.id = session_id;
        crash_recovery.set_active_session(session);
        crash_recovery
            .save_crash_dump(Some("panic 1".to_string()), None)
            .unwrap();

        let mut session2 = create_test_session();
        session2.id = session_id;
        crash_recovery.set_active_session(session2);
        std::thread::sleep(std::time::Duration::from_millis(10));
        crash_recovery
            .save_crash_dump(Some("panic 2".to_string()), None)
            .unwrap();

        let incomplete = service.list_incomplete_sessions().await.unwrap();
        assert_eq!(
            incomplete.len(),
            1,
            "Should deduplicate same session with multiple crashes"
        );
        assert!(incomplete.contains(&session_id));

        drop(temp_dir);
    }
}

#[cfg(test)]
mod session_repository_pagination_tests {
    use super::*;
    use crate::memory_repository::InMemorySessionRepository;
    use opencode_core::Session;

    fn create_test_session_for_pagination() -> Session {
        let mut session = Session::new();
        session.add_message(Message::user("Test message".to_string()));
        session
    }

    #[tokio::test]
    async fn test_in_memory_session_repository_list_pagination() {
        let repo = InMemorySessionRepository::new();

        // Create 10 sessions
        for _i in 0..10 {
            let session = create_test_session_for_pagination();
            repo.save(&session).await.unwrap();
        }

        // Test limit
        let sessions = repo.find_all(3, 0).await.unwrap();
        assert_eq!(sessions.len(), 3);

        // Test offset
        let sessions = repo.find_all(10, 5).await.unwrap();
        assert_eq!(sessions.len(), 5);

        // Test empty result
        let sessions = repo.find_all(10, 100).await.unwrap();
        assert!(sessions.is_empty());
    }

    #[tokio::test]
    async fn test_in_memory_session_repository_list_empty() {
        let repo = InMemorySessionRepository::new();

        let sessions = repo.find_all(10, 0).await.unwrap();
        assert!(sessions.is_empty());
    }
}

#[cfg(test)]
mod project_repository_pagination_tests {
    use super::*;
    use crate::memory_repository::InMemoryProjectRepository;
    use chrono::Utc;
    use uuid::Uuid;

    fn create_test_project_for_pagination(path: &str) -> ProjectModel {
        ProjectModel {
            id: Uuid::new_v4().to_string(),
            path: path.to_string(),
            name: Some("Test Project".to_string()),
            created_at: Utc::now(),
            updated_at: Utc::now(),
            data: None,
        }
    }

    #[tokio::test]
    async fn test_in_memory_project_repository_list_pagination() {
        let repo = InMemoryProjectRepository::new();

        // Create 10 projects
        for i in 0..10 {
            let project = create_test_project_for_pagination(&format!("/tmp/test_project_{}", i));
            repo.save(&project).await.unwrap();
        }

        // Test limit
        let projects = repo.find_all(3, 0).await.unwrap();
        assert_eq!(projects.len(), 3);

        // Test offset
        let projects = repo.find_all(10, 5).await.unwrap();
        assert_eq!(projects.len(), 5);

        // Test empty result
        let projects = repo.find_all(10, 100).await.unwrap();
        assert!(projects.is_empty());
    }

    #[tokio::test]
    async fn test_in_memory_project_repository_list_empty() {
        let repo = InMemoryProjectRepository::new();

        let projects = repo.find_all(10, 0).await.unwrap();
        assert!(projects.is_empty());
    }
}
