use crate::database::StoragePool;
use crate::error::StorageError;
use crate::models::SessionModel;
use crate::repository::{sealed, SessionRepository};
use async_trait::async_trait;
use opencode_core::{Session, SessionInfo};
use rusqlite::params;

pub struct SqliteSessionRepository {
    pool: StoragePool,
}

impl SqliteSessionRepository {
    pub fn new(pool: StoragePool) -> Self {
        Self { pool }
    }

    #[allow(dead_code)]
    pub async fn set_project_path(
        &self,
        session_id: &str,
        project_path: &str,
    ) -> Result<(), StorageError> {
        let conn = self.pool.get().await.map_err(StorageError::from)?;
        let id_str = session_id.to_string();
        let path_str = project_path.to_string();
        conn.execute(move |c| {
            c.execute(
                "UPDATE sessions SET project_path = ?1 WHERE id = ?2",
                params![path_str, id_str],
            )
        })
        .await
        .map_err(StorageError::from)??;
        Ok(())
    }
}

impl sealed::Sealed for SqliteSessionRepository {}

#[async_trait]
impl SessionRepository for SqliteSessionRepository {
    async fn find_by_id(&self, id: &str) -> Result<Option<Session>, StorageError> {
        let conn = self.pool.get().await.map_err(StorageError::from)?;
        let id_str = id.to_string();
        let res = conn
            .execute(move |c| {
                let mut stmt = c.prepare(
                    "SELECT id, created_at, updated_at, data, project_path FROM sessions WHERE id = ?1",
                )?;
                let mut rows = stmt.query_map(params![id_str], |row| {
                    Ok(SessionModel {
                        id: row.get(0)?,
                        created_at: row.get::<_, String>(1)?.parse().unwrap_or_default(),
                        updated_at: row.get::<_, String>(2)?.parse().unwrap_or_default(),
                        data: row.get(3)?,
                        project_path: row.get(4)?,
                    })
                })?;
                if let Some(Ok(model)) = rows.next() {
                    Ok::<Option<SessionModel>, rusqlite::Error>(Some(model))
                } else {
                    Ok::<Option<SessionModel>, rusqlite::Error>(None)
                }
            })
            .await
            .map_err(StorageError::from)??;
        match res {
            Some(model) => {
                let session = Session::try_from(model)
                    .map_err(|e| StorageError::Deserialization(e.to_string()))?;
                Ok(Some(session))
            }
            None => Ok(None),
        }
    }

    async fn find_all(
        &self,
        limit: usize,
        offset: usize,
    ) -> Result<Vec<SessionInfo>, StorageError> {
        let conn = self.pool.get().await.map_err(StorageError::from)?;
        let models = conn
            .execute(move |c| {
                let mut stmt = c.prepare(
                    "SELECT id, created_at, updated_at, data, project_path FROM sessions
                     ORDER BY updated_at DESC LIMIT ?1 OFFSET ?2",
                )?;
                let rows = stmt.query_map(params![limit as i32, offset as i32], |row| {
                    Ok(SessionModel {
                        id: row.get(0)?,
                        created_at: row.get::<_, String>(1)?.parse().unwrap_or_default(),
                        updated_at: row.get::<_, String>(2)?.parse().unwrap_or_default(),
                        data: row.get(3)?,
                        project_path: row.get(4)?,
                    })
                })?;
                let mut results = Vec::new();
                for row in rows {
                    results.push(row?);
                }
                Ok::<Vec<SessionModel>, rusqlite::Error>(results)
            })
            .await
            .map_err(StorageError::from)??;
        let mut infos = Vec::new();
        for model in models {
            if let Ok(session) = Session::try_from(model) {
                infos.push(SessionInfo {
                    id: session.id,
                    created_at: session.created_at,
                    updated_at: session.updated_at,
                    message_count: session.messages.len(),
                    preview: session
                        .messages
                        .last()
                        .map(|m| m.content.chars().take(50).collect())
                        .unwrap_or_default(),
                });
            }
        }
        Ok(infos)
    }

    async fn save(&self, session: &Session) -> Result<(), StorageError> {
        let model = SessionModel::from(session.clone());
        let conn = self.pool.get().await.map_err(StorageError::from)?;
        let id = model.id.clone();
        let created_at = model.created_at.to_rfc3339();
        let updated_at = model.updated_at.to_rfc3339();
        let data = model.data;
        let project_path = model.project_path.clone();
        conn.execute(move |c| {
            c.execute(
                "INSERT OR REPLACE INTO sessions (id, created_at, updated_at, data, project_path)
                 VALUES (?1, ?2, ?3, ?4, ?5)",
                params![id, created_at, updated_at, data, project_path.as_deref()],
            )
        })
        .await
        .map_err(StorageError::from)??;
        Ok(())
    }

    async fn delete(&self, id: &str) -> Result<(), StorageError> {
        let conn = self.pool.get().await.map_err(StorageError::from)?;
        let id_str = id.to_string();
        conn.execute(move |c| c.execute("DELETE FROM sessions WHERE id = ?1", params![id_str]))
            .await
            .map_err(StorageError::from)??;
        Ok(())
    }

    async fn count(&self) -> Result<usize, StorageError> {
        let conn = self.pool.get().await.map_err(StorageError::from)?;
        let count: i32 = conn
            .execute(move |c| {
                let mut stmt = c.prepare("SELECT COUNT(*) FROM sessions")?;
                stmt.query_row([], |row| row.get(0))
            })
            .await
            .map_err(StorageError::from)??;
        Ok(count as usize)
    }

    async fn exists(&self, id: &str) -> Result<bool, StorageError> {
        let conn = self.pool.get().await.map_err(StorageError::from)?;
        let id_str = id.to_string();
        let exists: bool = conn
            .execute(move |c| {
                let mut stmt = c.prepare("SELECT EXISTS(SELECT 1 FROM sessions WHERE id = ?1)")?;
                stmt.query_row(params![id_str], |row| row.get(0))
            })
            .await
            .map_err(StorageError::from)??;
        Ok(exists)
    }

    async fn list_by_project(
        &self,
        project_path: &str,
        limit: usize,
        offset: usize,
    ) -> Result<Vec<SessionInfo>, StorageError> {
        let conn = self.pool.get().await.map_err(StorageError::from)?;
        let path_str = project_path.to_string();
        let models = conn
            .execute(move |c| {
                let mut stmt = c.prepare(
                    "SELECT id, created_at, updated_at, data, project_path FROM sessions
                     WHERE project_path = ?1
                     ORDER BY updated_at DESC LIMIT ?2 OFFSET ?3",
                )?;
                let rows =
                    stmt.query_map(params![path_str, limit as i32, offset as i32], |row| {
                        Ok(SessionModel {
                            id: row.get(0)?,
                            created_at: row.get::<_, String>(1)?.parse().unwrap_or_default(),
                            updated_at: row.get::<_, String>(2)?.parse().unwrap_or_default(),
                            data: row.get(3)?,
                            project_path: row.get(4)?,
                        })
                    })?;
                let mut results = Vec::new();
                for row in rows {
                    results.push(row?);
                }
                Ok::<Vec<SessionModel>, rusqlite::Error>(results)
            })
            .await
            .map_err(StorageError::from)??;
        let mut infos = Vec::new();
        for model in models {
            if let Ok(session) = Session::try_from(model) {
                infos.push(SessionInfo {
                    id: session.id,
                    created_at: session.created_at,
                    updated_at: session.updated_at,
                    message_count: session.messages.len(),
                    preview: session
                        .messages
                        .last()
                        .map(|m| m.content.chars().take(50).collect())
                        .unwrap_or_default(),
                });
            }
        }
        Ok(infos)
    }
}

use crate::models::ProjectModel;
use crate::repository::ProjectRepository;

pub struct SqliteProjectRepository {
    pool: StoragePool,
}

impl SqliteProjectRepository {
    pub fn new(pool: StoragePool) -> Self {
        Self { pool }
    }
}

impl sealed::Sealed for SqliteProjectRepository {}

#[async_trait]
impl ProjectRepository for SqliteProjectRepository {
    async fn find_by_id(&self, id: &str) -> Result<Option<ProjectModel>, StorageError> {
        let conn = self.pool.get().await.map_err(StorageError::from)?;
        let id_str = id.to_string();
        let res = conn
            .execute(move |c| {
                let mut stmt = c.prepare(
                    "SELECT id, path, name, created_at, updated_at, data FROM projects WHERE id = ?1",
                )?;
                let mut rows = stmt.query_map(params![id_str], |row| {
                    Ok(ProjectModel {
                        id: row.get(0)?,
                        path: row.get(1)?,
                        name: row.get(2)?,
                        created_at: row.get::<_, String>(3)?.parse().unwrap_or_default(),
                        updated_at: row.get::<_, String>(4)?.parse().unwrap_or_default(),
                        data: row.get(5)?,
                    })
                })?;
                if let Some(Ok(model)) = rows.next() {
                    Ok::<Option<ProjectModel>, rusqlite::Error>(Some(model))
                } else {
                    Ok::<Option<ProjectModel>, rusqlite::Error>(None)
                }
            })
            .await
            .map_err(StorageError::from)??;
        Ok(res)
    }

    async fn find_by_path(&self, path: &str) -> Result<Option<ProjectModel>, StorageError> {
        let conn = self.pool.get().await.map_err(StorageError::from)?;
        let path_str = path.to_string();
        let res = conn
            .execute(move |c| {
                let mut stmt = c.prepare(
                    "SELECT id, path, name, created_at, updated_at, data FROM projects WHERE path = ?1",
                )?;
                let mut rows = stmt.query_map(params![path_str], |row| {
                    Ok(ProjectModel {
                        id: row.get(0)?,
                        path: row.get(1)?,
                        name: row.get(2)?,
                        created_at: row.get::<_, String>(3)?.parse().unwrap_or_default(),
                        updated_at: row.get::<_, String>(4)?.parse().unwrap_or_default(),
                        data: row.get(5)?,
                    })
                })?;
                if let Some(Ok(model)) = rows.next() {
                    Ok::<Option<ProjectModel>, rusqlite::Error>(Some(model))
                } else {
                    Ok::<Option<ProjectModel>, rusqlite::Error>(None)
                }
            })
            .await
            .map_err(StorageError::from)??;
        Ok(res)
    }

    async fn find_all(
        &self,
        limit: usize,
        offset: usize,
    ) -> Result<Vec<ProjectModel>, StorageError> {
        let conn = self.pool.get().await.map_err(StorageError::from)?;
        let res = conn
            .execute(move |c| {
                let mut stmt = c.prepare(
                    "SELECT id, path, name, created_at, updated_at, data FROM projects
                     ORDER BY updated_at DESC LIMIT ?1 OFFSET ?2",
                )?;
                let rows = stmt.query_map(params![limit as i32, offset as i32], |row| {
                    Ok(ProjectModel {
                        id: row.get(0)?,
                        path: row.get(1)?,
                        name: row.get(2)?,
                        created_at: row.get::<_, String>(3)?.parse().unwrap_or_default(),
                        updated_at: row.get::<_, String>(4)?.parse().unwrap_or_default(),
                        data: row.get(5)?,
                    })
                })?;
                let mut results = Vec::new();
                for row in rows {
                    results.push(row?);
                }
                Ok::<Vec<ProjectModel>, rusqlite::Error>(results)
            })
            .await
            .map_err(StorageError::from)??;
        Ok(res)
    }

    async fn save(&self, project: &ProjectModel) -> Result<(), StorageError> {
        let conn = self.pool.get().await.map_err(StorageError::from)?;
        let id = project.id.clone();
        let path = project.path.clone();
        let name = project.name.clone();
        let created_at = project.created_at.to_rfc3339();
        let updated_at = project.updated_at.to_rfc3339();
        let data = project.data.clone();
        conn.execute(move |c| {
            c.execute(
                "INSERT OR REPLACE INTO projects (id, path, name, created_at, updated_at, data)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
                params![
                    id,
                    path,
                    name,
                    created_at,
                    updated_at,
                    data.as_deref().unwrap_or_default()
                ],
            )
        })
        .await
        .map_err(StorageError::from)??;
        Ok(())
    }

    async fn delete(&self, id: &str) -> Result<(), StorageError> {
        let conn = self.pool.get().await.map_err(StorageError::from)?;
        let id_str = id.to_string();
        conn.execute(move |c| c.execute("DELETE FROM projects WHERE id = ?1", params![id_str]))
            .await
            .map_err(StorageError::from)??;
        Ok(())
    }

    async fn count(&self) -> Result<usize, StorageError> {
        let conn = self.pool.get().await.map_err(StorageError::from)?;
        let count: i32 = conn
            .execute(move |c| {
                let mut stmt = c.prepare("SELECT COUNT(*) FROM projects")?;
                stmt.query_row([], |row| row.get(0))
            })
            .await
            .map_err(StorageError::from)??;
        Ok(count as usize)
    }
}

use crate::models::AccountModel;
use crate::repository::AccountRepository;

#[allow(dead_code)]
pub(crate) struct SqliteAccountRepository {
    pool: StoragePool,
}

impl SqliteAccountRepository {
    #[allow(dead_code)]
    pub(crate) fn new(pool: StoragePool) -> Self {
        Self { pool }
    }
}

impl sealed::Sealed for SqliteAccountRepository {}

#[async_trait]
impl AccountRepository for SqliteAccountRepository {
    async fn find_by_id(&self, id: &str) -> Result<Option<AccountModel>, StorageError> {
        let conn = self.pool.get().await.map_err(StorageError::from)?;
        let id_str = id.to_string();
        let res = conn
            .execute(move |c| {
                let mut stmt = c.prepare(
                    "SELECT id, username, email, password_hash, created_at, updated_at,
                     last_login_at, is_active, data FROM accounts WHERE id = ?1",
                )?;
                let mut rows = stmt.query_map(params![id_str], Self::map_row)?;
                if let Some(Ok(model)) = rows.next() {
                    Ok::<Option<AccountModel>, rusqlite::Error>(Some(model))
                } else {
                    Ok::<Option<AccountModel>, rusqlite::Error>(None)
                }
            })
            .await
            .map_err(StorageError::from)??;
        Ok(res)
    }

    async fn find_by_username(&self, username: &str) -> Result<Option<AccountModel>, StorageError> {
        let conn = self.pool.get().await.map_err(StorageError::from)?;
        let username_str = username.to_string();
        let res = conn
            .execute(move |c| {
                let mut stmt = c.prepare(
                    "SELECT id, username, email, password_hash, created_at, updated_at,
                     last_login_at, is_active, data FROM accounts WHERE username = ?1",
                )?;
                let mut rows = stmt.query_map(params![username_str], Self::map_row)?;
                if let Some(Ok(model)) = rows.next() {
                    Ok::<Option<AccountModel>, rusqlite::Error>(Some(model))
                } else {
                    Ok::<Option<AccountModel>, rusqlite::Error>(None)
                }
            })
            .await
            .map_err(StorageError::from)??;
        Ok(res)
    }

    async fn find_by_email(&self, email: &str) -> Result<Option<AccountModel>, StorageError> {
        let conn = self.pool.get().await.map_err(StorageError::from)?;
        let email_str = email.to_string();
        let res = conn
            .execute(move |c| {
                let mut stmt = c.prepare(
                    "SELECT id, username, email, password_hash, created_at, updated_at,
                     last_login_at, is_active, data FROM accounts WHERE email = ?1",
                )?;
                let mut rows = stmt.query_map(params![email_str], Self::map_row)?;
                if let Some(Ok(model)) = rows.next() {
                    Ok::<Option<AccountModel>, rusqlite::Error>(Some(model))
                } else {
                    Ok::<Option<AccountModel>, rusqlite::Error>(None)
                }
            })
            .await
            .map_err(StorageError::from)??;
        Ok(res)
    }

    async fn find_all(
        &self,
        limit: usize,
        offset: usize,
    ) -> Result<Vec<AccountModel>, StorageError> {
        let conn = self.pool.get().await.map_err(StorageError::from)?;
        let models = conn
            .execute(move |c| {
                let mut stmt = c.prepare(
                    "SELECT id, username, email, password_hash, created_at, updated_at,
                     last_login_at, is_active, data FROM accounts
                     ORDER BY updated_at DESC LIMIT ?1 OFFSET ?2",
                )?;
                let rows = stmt.query_map(params![limit as i32, offset as i32], Self::map_row)?;
                let mut results = Vec::new();
                for row in rows {
                    results.push(row?);
                }
                Ok::<Vec<AccountModel>, rusqlite::Error>(results)
            })
            .await
            .map_err(StorageError::from)??;
        Ok(models)
    }

    async fn save(&self, account: &AccountModel) -> Result<(), StorageError> {
        let conn = self.pool.get().await.map_err(StorageError::from)?;
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
                "INSERT OR REPLACE INTO accounts
                 (id, username, email, password_hash, created_at, updated_at, last_login_at, is_active, data)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
                params![id, username, email, password_hash, created_at, updated_at, last_login_at, is_active, json],
            )
        })
        .await
        .map_err(StorageError::from)??;
        Ok(())
    }

    async fn delete(&self, id: &str) -> Result<(), StorageError> {
        let conn = self.pool.get().await.map_err(StorageError::from)?;
        let id_str = id.to_string();
        conn.execute(move |c| c.execute("DELETE FROM accounts WHERE id = ?1", params![id_str]))
            .await
            .map_err(StorageError::from)??;
        Ok(())
    }

    async fn count(&self) -> Result<usize, StorageError> {
        let conn = self.pool.get().await.map_err(StorageError::from)?;
        let count: i32 = conn
            .execute(move |c| {
                let mut stmt = c.prepare("SELECT COUNT(*) FROM accounts")?;
                stmt.query_row([], |row| row.get(0))
            })
            .await
            .map_err(StorageError::from)??;
        Ok(count as usize)
    }
}

impl SqliteAccountRepository {
    #[allow(dead_code)]
    pub(crate) fn map_row(row: &rusqlite::Row<'_>) -> rusqlite::Result<AccountModel> {
        Ok(AccountModel {
            id: row.get(0)?,
            username: row.get(1)?,
            email: row.get(2)?,
            password_hash: row.get(3)?,
            created_at: row.get::<_, String>(4)?.parse().unwrap_or_default(),
            updated_at: row.get::<_, String>(5)?.parse().unwrap_or_default(),
            last_login_at: row
                .get::<_, Option<String>>(6)?
                .and_then(|s| s.parse().ok()),
            is_active: row.get(7)?,
            data: row.get(8)?,
        })
    }
}

use crate::models::PluginStateModel;
use crate::repository::PluginStateRepository;

#[allow(dead_code)]
pub(crate) struct SqlitePluginStateRepository {
    pool: StoragePool,
}

impl SqlitePluginStateRepository {
    #[allow(dead_code)]
    pub(crate) fn new(pool: StoragePool) -> Self {
        Self { pool }
    }
}

impl sealed::Sealed for SqlitePluginStateRepository {}

#[async_trait]
impl PluginStateRepository for SqlitePluginStateRepository {
    async fn find_by_id(&self, plugin_id: &str) -> Result<Option<PluginStateModel>, StorageError> {
        let conn = self.pool.get().await.map_err(StorageError::from)?;
        let id_str = plugin_id.to_string();
        let res = conn
            .execute(move |c| {
                let mut stmt = c.prepare(
                    "SELECT plugin_id, state_data, updated_at FROM plugin_states WHERE plugin_id = ?1",
                )?;
                let mut rows = stmt.query_map(params![id_str], |row| {
                    Ok(PluginStateModel {
                        plugin_id: row.get(0)?,
                        state_data: row.get(1)?,
                        updated_at: row.get::<_, String>(2)?.parse().unwrap_or_default(),
                    })
                })?;
                if let Some(Ok(model)) = rows.next() {
                    Ok::<Option<PluginStateModel>, rusqlite::Error>(Some(model))
                } else {
                    Ok::<Option<PluginStateModel>, rusqlite::Error>(None)
                }
            })
            .await
            .map_err(StorageError::from)??;
        Ok(res)
    }

    async fn save(&self, state: &PluginStateModel) -> Result<(), StorageError> {
        let conn = self.pool.get().await.map_err(StorageError::from)?;
        let plugin_id = state.plugin_id.clone();
        let state_data = state.state_data.clone();
        let updated_at = state.updated_at.to_rfc3339();
        conn.execute(move |c| {
            c.execute(
                "INSERT OR REPLACE INTO plugin_states (plugin_id, state_data, updated_at)
                 VALUES (?1, ?2, ?3)",
                params![plugin_id, state_data, updated_at],
            )
        })
        .await
        .map_err(StorageError::from)??;
        Ok(())
    }

    async fn delete(&self, plugin_id: &str) -> Result<(), StorageError> {
        let conn = self.pool.get().await.map_err(StorageError::from)?;
        let id_str = plugin_id.to_string();
        conn.execute(move |c| {
            c.execute(
                "DELETE FROM plugin_states WHERE plugin_id = ?1",
                params![id_str],
            )
        })
        .await
        .map_err(StorageError::from)??;
        Ok(())
    }
}

#[cfg(test)]
mod session_repository_exists_tests {
    use super::*;
    use crate::database::StoragePool;
    use crate::MigrationManager;
    use opencode_core::Session;
    use uuid::Uuid;

    fn create_temp_db() -> (StoragePool, tempfile::TempDir) {
        let temp_dir = tempfile::tempdir().unwrap();
        let db_path = temp_dir.path().join("test.db");
        let pool = StoragePool::new(&db_path).expect("Failed to create temp database");
        (pool, temp_dir)
    }

    #[tokio::test]
    async fn test_sqlite_session_exists_returns_true_for_existing_session() {
        let (pool, _temp_dir) = create_temp_db();
        MigrationManager::new(pool.clone(), 3)
            .migrate()
            .await
            .unwrap();
        let repo = SqliteSessionRepository::new(pool);
        let session = Session::default();

        repo.save(&session).await.unwrap();

        let id = session.id.to_string();
        assert!(repo.exists(&id).await.unwrap());
    }

    #[tokio::test]
    async fn test_sqlite_session_exists_returns_false_for_non_existent_session() {
        let (pool, _temp_dir) = create_temp_db();
        MigrationManager::new(pool.clone(), 3)
            .migrate()
            .await
            .unwrap();
        let repo = SqliteSessionRepository::new(pool);
        let non_existent_id = Uuid::new_v4().to_string();

        assert!(!repo.exists(&non_existent_id).await.unwrap());
    }

    #[tokio::test]
    async fn test_sqlite_session_exists_does_not_interfere_with_other_operations() {
        let (pool, _temp_dir) = create_temp_db();
        MigrationManager::new(pool.clone(), 3)
            .migrate()
            .await
            .unwrap();
        let repo = SqliteSessionRepository::new(pool);
        let session = Session::default();

        repo.save(&session).await.unwrap();

        let id = session.id.to_string();
        assert!(repo.exists(&id).await.unwrap());

        let found = repo.find_by_id(&id).await.unwrap();
        assert!(found.is_some());
        assert_eq!(found.unwrap().id, session.id);

        let before_count = repo.count().await.unwrap();

        let sessions = repo.find_all(10, 0).await.unwrap();
        assert!(!sessions.is_empty());

        assert!(repo.exists(&id).await.unwrap());

        repo.delete(&id).await.unwrap();

        assert!(!repo.exists(&id).await.unwrap());
        assert_eq!(repo.count().await.unwrap(), before_count - 1);
    }
}

#[cfg(test)]
mod session_repository_list_by_project_tests {
    use super::*;
    use crate::database::StoragePool;
    use crate::MigrationManager;
    use opencode_core::Session;
    

    fn create_temp_db() -> (StoragePool, tempfile::TempDir) {
        let temp_dir = tempfile::tempdir().unwrap();
        let db_path = temp_dir.path().join("test.db");
        let pool = StoragePool::new(&db_path).expect("Failed to create temp database");
        (pool, temp_dir)
    }

    #[tokio::test]
    async fn test_sqlite_list_by_project_returns_sessions_for_given_project() {
        let (pool, _temp_dir) = create_temp_db();
        MigrationManager::new(pool.clone(), 3)
            .migrate()
            .await
            .unwrap();
        let repo = SqliteSessionRepository::new(pool);

        let session1 = Session::default();
        let session2 = Session::default();
        let session3 = Session::default();

        repo.save(&session1).await.unwrap();
        repo.save(&session2).await.unwrap();
        repo.save(&session3).await.unwrap();

        repo.set_project_path(&session1.id.to_string(), "/path/to/project1")
            .await
            .unwrap();
        repo.set_project_path(&session2.id.to_string(), "/path/to/project1")
            .await
            .unwrap();
        repo.set_project_path(&session3.id.to_string(), "/path/to/project2")
            .await
            .unwrap();

        let project1_sessions = repo
            .list_by_project("/path/to/project1", 10, 0)
            .await
            .unwrap();
        assert_eq!(project1_sessions.len(), 2);
        let project1_ids: Vec<_> = project1_sessions.iter().map(|s| s.id).collect();
        assert!(project1_ids.contains(&session1.id));
        assert!(project1_ids.contains(&session2.id));
        assert!(!project1_ids.contains(&session3.id));

        let project2_sessions = repo
            .list_by_project("/path/to/project2", 10, 0)
            .await
            .unwrap();
        assert_eq!(project2_sessions.len(), 1);
        assert_eq!(project2_sessions[0].id, session3.id);
    }

    #[tokio::test]
    async fn test_sqlite_list_by_project_returns_empty_for_nonexistent_project() {
        let (pool, _temp_dir) = create_temp_db();
        MigrationManager::new(pool.clone(), 3)
            .migrate()
            .await
            .unwrap();
        let repo = SqliteSessionRepository::new(pool);
        let session = Session::default();

        repo.save(&session).await.unwrap();
        repo.set_project_path(&session.id.to_string(), "/path/to/project1")
            .await
            .unwrap();

        let result = repo
            .list_by_project("/nonexistent/project", 10, 0)
            .await
            .unwrap();
        assert!(result.is_empty());
    }

    #[tokio::test]
    async fn test_sqlite_list_by_project_with_pagination() {
        let (pool, _temp_dir) = create_temp_db();
        MigrationManager::new(pool.clone(), 3)
            .migrate()
            .await
            .unwrap();
        let repo = SqliteSessionRepository::new(pool);

        for _i in 0..5 {
            let session = Session::default();
            repo.save(&session).await.unwrap();
            repo.set_project_path(&session.id.to_string(), "/path/to/project1")
                .await
                .unwrap();
        }

        let page1 = repo
            .list_by_project("/path/to/project1", 2, 0)
            .await
            .unwrap();
        assert_eq!(page1.len(), 2);

        let page2 = repo
            .list_by_project("/path/to/project1", 2, 2)
            .await
            .unwrap();
        assert_eq!(page2.len(), 2);

        let page3 = repo
            .list_by_project("/path/to/project1", 2, 4)
            .await
            .unwrap();
        assert_eq!(page3.len(), 1);
    }

    #[tokio::test]
    async fn test_sqlite_list_by_project_does_not_affect_other_operations() {
        let (pool, _temp_dir) = create_temp_db();
        MigrationManager::new(pool.clone(), 3)
            .migrate()
            .await
            .unwrap();
        let repo = SqliteSessionRepository::new(pool);

        let session = Session::default();
        repo.save(&session).await.unwrap();
        let id = session.id.to_string();

        repo.set_project_path(&id, "/path/to/project1")
            .await
            .unwrap();

        assert!(repo.exists(&id).await.unwrap());
        let found = repo.find_by_id(&id).await.unwrap();
        assert!(found.is_some());
        assert_eq!(found.unwrap().id, session.id);

        let count_before = repo.count().await.unwrap();
        let list_result = repo
            .list_by_project("/path/to/project1", 10, 0)
            .await
            .unwrap();
        let count_after = repo.count().await.unwrap();
        assert_eq!(count_before, count_after);
        assert_eq!(list_result.len(), 1);

        repo.delete(&id).await.unwrap();
        assert!(!repo.exists(&id).await.unwrap());
        let list_after_delete = repo
            .list_by_project("/path/to/project1", 10, 0)
            .await
            .unwrap();
        assert!(list_after_delete.is_empty());
    }
}
