use crate::database::StoragePool;
use crate::models::{AccountModel, ProjectModel, SessionModel};
use opencode_core::{Message, OpenCodeError, Session, SessionInfo};
use rusqlite::params;

pub struct StorageService {
    pool: StoragePool,
}

impl StorageService {
    pub fn new(pool: StoragePool) -> Self {
        Self { pool }
    }

    pub async fn save_session(&self, session: &Session) -> Result<(), OpenCodeError> {
        let model = SessionModel::from(session.clone());
        let session_json = model.data.clone();

        let conn = self.pool.get().await?;
        let id = model.id.clone();
        let created_at = model.created_at.to_rfc3339();
        let updated_at = model.updated_at.to_rfc3339();

        conn.execute(move |c| {
            c.execute(
                "INSERT OR REPLACE INTO sessions (id, created_at, updated_at, data) 
                 VALUES (?1, ?2, ?3, ?4)",
                params![id, created_at, updated_at, session_json],
            )
        })
        .await
        .map_err(|e| OpenCodeError::Storage(e.to_string()))??;

        Ok(())
    }

    pub async fn load_session(&self, id: &str) -> Result<Option<Session>, OpenCodeError> {
        let conn = self.pool.get().await?;
        let id_str = id.to_string();

        let res = conn
            .execute(move |c| {
                let mut stmt = c.prepare(
                    "SELECT id, created_at, updated_at, data FROM sessions WHERE id = ?1",
                )?;

                let mut rows = stmt.query_map(params![id_str], |row| {
                    let id: String = row.get(0)?;
                    let created_at: String = row.get(1)?;
                    let updated_at: String = row.get(2)?;
                    let data: String = row.get(3)?;
                    Ok(SessionModel {
                        id,
                        created_at: created_at.parse().unwrap_or_default(),
                        updated_at: updated_at.parse().unwrap_or_default(),
                        data,
                    })
                })?;

                if let Some(Ok(model)) = rows.next() {
                    Ok::<Option<SessionModel>, rusqlite::Error>(Some(model))
                } else {
                    Ok::<Option<SessionModel>, rusqlite::Error>(None)
                }
            })
            .await
            .map_err(|e| OpenCodeError::Storage(e.to_string()))??;

        match res {
            Some(model) => {
                let session =
                    Session::try_from(model).map_err(|e| OpenCodeError::Storage(e.to_string()))?;
                Ok(Some(session))
            }
            None => Ok(None),
        }
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
        let messages = session
            .messages
            .iter()
            .skip(offset)
            .take(limit)
            .cloned()
            .collect();
        Ok(messages)
    }

    pub async fn count_session_messages(&self, id: &str) -> Result<usize, OpenCodeError> {
        let session = self
            .load_session(id)
            .await?
            .ok_or_else(|| OpenCodeError::Storage("Session not found".to_string()))?;
        Ok(session.messages.len())
    }

    pub async fn delete_session(&self, id: &str) -> Result<(), OpenCodeError> {
        let conn = self.pool.get().await?;
        let id_str = id.to_string();

        conn.execute(move |c| c.execute("DELETE FROM sessions WHERE id = ?1", params![id_str]))
            .await
            .map_err(|e| OpenCodeError::Storage(e.to_string()))??;

        Ok(())
    }

    pub async fn list_sessions(
        &self,
        limit: usize,
        offset: usize,
    ) -> Result<Vec<SessionInfo>, OpenCodeError> {
        let conn = self.pool.get().await?;

        let models = conn
            .execute(move |c| {
                let mut stmt = c.prepare(
                    "SELECT id, created_at, updated_at, data FROM sessions 
                 ORDER BY updated_at DESC 
                 LIMIT ?1 OFFSET ?2",
                )?;

                let rows = stmt.query_map(params![limit as i32, offset as i32], |row| {
                    let id: String = row.get(0)?;
                    let created_at: String = row.get(1)?;
                    let updated_at: String = row.get(2)?;
                    let data: String = row.get(3)?;
                    Ok(SessionModel {
                        id,
                        created_at: created_at.parse().unwrap_or_default(),
                        updated_at: updated_at.parse().unwrap_or_default(),
                        data,
                    })
                })?;

                let mut sessions = Vec::new();
                for row in rows {
                    sessions.push(row?);
                }

                Ok::<Vec<SessionModel>, rusqlite::Error>(sessions)
            })
            .await
            .map_err(|e| OpenCodeError::Storage(e.to_string()))??;

        let mut session_infos = Vec::new();
        for model in models {
            let session =
                Session::try_from(model).map_err(|e| OpenCodeError::Storage(e.to_string()))?;
            session_infos.push(SessionInfo {
                id: session.id,
                created_at: session.created_at,
                updated_at: session.updated_at,
                message_count: session.messages.len(),
                preview: session
                    .messages
                    .last()
                    .map(|m| m.content.chars().take(50).collect::<String>())
                    .unwrap_or_default(),
            });
        }

        Ok(session_infos)
    }

    pub async fn save_project(&self, project: &ProjectModel) -> Result<(), OpenCodeError> {
        let conn = self.pool.get().await?;
        let id = project.id.clone();
        let path = project.path.clone();
        let name = project.name.clone();
        let created_at = project.created_at.to_rfc3339();
        let updated_at = project.updated_at.to_rfc3339();
        let data = project.data.clone();

        conn.execute(move |c| {
            let json = data.as_deref().unwrap_or_default();
            c.execute(
                "INSERT OR REPLACE INTO projects (id, path, name, created_at, updated_at, data) 
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
                params![id, path, name, created_at, updated_at, json],
            )
        })
        .await
        .map_err(|e| OpenCodeError::Storage(e.to_string()))??;
        Ok(())
    }

    pub async fn load_project_by_path(
        &self,
        path: &str,
    ) -> Result<Option<ProjectModel>, OpenCodeError> {
        let conn = self.pool.get().await?;
        let path_str = path.to_string();

        let res = conn
            .execute(move |c| {
                let mut stmt = c.prepare(
                "SELECT id, path, name, created_at, updated_at, data FROM projects WHERE path = ?1"
            )?;

                let mut rows = stmt.query_map(params![path_str], |row| {
                    let id: String = row.get(0)?;
                    let path: String = row.get(1)?;
                    let name: Option<String> = row.get(2)?;
                    let created_at: String = row.get(3)?;
                    let updated_at: String = row.get(4)?;
                    let data: Option<String> = row.get(5)?;
                    Ok(ProjectModel {
                        id,
                        path,
                        name,
                        created_at: created_at.parse().unwrap_or_default(),
                        updated_at: updated_at.parse().unwrap_or_default(),
                        data,
                    })
                })?;

                if let Some(Ok(model)) = rows.next() {
                    Ok::<Option<ProjectModel>, rusqlite::Error>(Some(model))
                } else {
                    Ok::<Option<ProjectModel>, rusqlite::Error>(None)
                }
            })
            .await
            .map_err(|e| OpenCodeError::Storage(e.to_string()))??;
        Ok(res)
    }

    pub async fn list_projects(
        &self,
        limit: usize,
        offset: usize,
    ) -> Result<Vec<ProjectModel>, OpenCodeError> {
        let conn = self.pool.get().await?;

        let res = conn
            .execute(move |c| {
                let mut stmt = c.prepare(
                    "SELECT id, path, name, created_at, updated_at, data FROM projects 
                 ORDER BY updated_at DESC 
                 LIMIT ?1 OFFSET ?2",
                )?;

                let rows = stmt.query_map(params![limit as i32, offset as i32], |row| {
                    let id: String = row.get(0)?;
                    let path: String = row.get(1)?;
                    let name: Option<String> = row.get(2)?;
                    let created_at: String = row.get(3)?;
                    let updated_at: String = row.get(4)?;
                    let data: Option<String> = row.get(5)?;
                    Ok(ProjectModel {
                        id,
                        path,
                        name,
                        created_at: created_at.parse().unwrap_or_default(),
                        updated_at: updated_at.parse().unwrap_or_default(),
                        data,
                    })
                })?;

                let mut projects = Vec::new();
                for row in rows {
                    projects.push(row?);
                }

                Ok::<Vec<ProjectModel>, rusqlite::Error>(projects)
            })
            .await
            .map_err(|e| OpenCodeError::Storage(e.to_string()))??;
        Ok(res)
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
