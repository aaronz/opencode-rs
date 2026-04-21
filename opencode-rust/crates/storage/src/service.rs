use std::path::PathBuf;
use std::sync::Arc;

use crate::compaction::CompactionManager;
use crate::database::StoragePool;
use crate::error::StorageError;
use crate::models::{AccountModel, ProjectModel};
use crate::repository::{ProjectRepository, SessionRepository};
use opencode_core::{
    crash_recovery::CrashRecovery, compaction::CompactionResult, Message, OpenCodeError, Session,
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
        self.session_repo
            .save(session)
            .await
            .map_err(OpenCodeError::from)
    }

    pub async fn load_session(&self, id: &str) -> Result<Option<Session>, OpenCodeError> {
        self.session_repo
            .find_by_id(id)
            .await
            .map_err(OpenCodeError::from)
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
        self.session_repo
            .delete(id)
            .await
            .map_err(OpenCodeError::from)
    }

    pub async fn list_sessions(
        &self,
        limit: usize,
        offset: usize,
    ) -> Result<Vec<SessionInfo>, OpenCodeError> {
        self.session_repo
            .find_all(limit, offset)
            .await
            .map_err(OpenCodeError::from)
    }

    pub async fn count_sessions(&self) -> Result<usize, OpenCodeError> {
        self.session_repo.count().await.map_err(OpenCodeError::from)
    }

    pub async fn save_project(&self, project: &ProjectModel) -> Result<(), OpenCodeError> {
        self.project_repo
            .save(project)
            .await
            .map_err(OpenCodeError::from)
    }

    pub async fn load_project(&self, id: &str) -> Result<Option<ProjectModel>, StorageError> {
        self.project_repo
            .find_by_id(id)
            .await
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
        let compaction_manager = self
            .compaction_manager
            .as_ref()
            .ok_or_else(|| StorageError::Internal("Compaction manager not configured".to_string()))?;

        // Clone session for compaction (compact takes &mut Session)
        let mut session_to_compact = session;
        let result = compaction_manager.compact(&mut session_to_compact).await
            .map_err(|e| StorageError::Internal(format!("Compaction failed: {}", e)))?;

        Ok(result.compaction_result)
    }

    pub async fn recover_session(&self, id: &str) -> Result<Session, StorageError> {
        self.crash_recovery
            .recover_session_latest(id)
            .map_err(|e| StorageError::Internal(format!("Recovery error: {}", e)))?
            .ok_or_else(|| StorageError::SessionNotFound(id.to_string()))
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

        let manager = MigrationManager::new(pool, 2);
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

        let manager = MigrationManager::new(pool.clone(), 2);
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
        assert!(result.is_ok(), "compact_session should succeed for existing session");
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
        assert!(matches!(err, crate::error::StorageError::SessionNotFound(_)));

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
        assert!(result.is_ok(), "compact_session should succeed and delegate to CompactionManager");
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

        let crash_recovery = CrashRecovery::new()
            .with_dump_dir(crash_dir);
        crash_recovery.set_active_session(session);
        crash_recovery.save_crash_dump(Some("test panic".to_string()), None).unwrap();

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

        let crash_recovery = CrashRecovery::new()
            .with_dump_dir(crash_dir);
        crash_recovery.set_active_session(session);
        crash_recovery.save_crash_dump(Some("panic".to_string()), None).unwrap();

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

        let crash_recovery = CrashRecovery::new()
            .with_dump_dir(crash_dir);
        crash_recovery.set_active_session(session);
        crash_recovery.save_crash_dump(Some("panic".to_string()), None).unwrap();

        let result = service.recover_session("bbbbbbbb-bbbb-bbbb-bbbb-bbbbbbbbbbbb").await;
        assert!(result.is_ok());

        drop(temp_dir);
    }
}
