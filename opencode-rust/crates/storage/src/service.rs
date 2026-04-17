use std::sync::Arc;

use crate::database::StoragePool;
use crate::models::{AccountModel, ProjectModel};
use crate::repository::{
    AccountRepository, PluginStateRepository, ProjectRepository, SessionRepository,
};
use opencode_core::{Message, OpenCodeError, Session, SessionInfo};
use rusqlite::params;

pub struct StorageService {
    session_repo: Arc<dyn SessionRepository>,
    project_repo: Arc<dyn ProjectRepository>,
    #[allow(dead_code)]
    account_repo: Arc<dyn AccountRepository>,
    #[allow(dead_code)]
    plugin_state_repo: Arc<dyn PluginStateRepository>,
    pool: StoragePool,
}

impl StorageService {
    pub fn new(
        session_repo: Arc<dyn SessionRepository>,
        project_repo: Arc<dyn ProjectRepository>,
        account_repo: Arc<dyn AccountRepository>,
        plugin_state_repo: Arc<dyn PluginStateRepository>,
        pool: StoragePool,
    ) -> Self {
        Self {
            session_repo,
            project_repo,
            account_repo,
            plugin_state_repo,
            pool,
        }
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
}
