use crate::database::{PooledConnection, StoragePool};
use opencode_core::OpenCodeError;
use rusqlite::params;

pub struct MigrationManager {
    pool: StoragePool,
    current_version: i32,
}

impl MigrationManager {
    pub fn new(pool: StoragePool, current_version: i32) -> Self {
        Self {
            pool,
            current_version,
        }
    }

    pub async fn migrate(&self) -> Result<(), OpenCodeError> {
        let conn = self.pool.get().await?;

        conn.execute(|c| {
            c.execute(
                "CREATE TABLE IF NOT EXISTS schema_migrations (
                    version INTEGER PRIMARY KEY,
                    applied_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP
                )",
                [],
            )
        })
        .await
        .map_err(|e| OpenCodeError::Storage(e.to_string()))??;

        let db_version: i32 = conn
            .execute(|c| {
                c.query_row(
                    "SELECT COALESCE(MAX(version), 0) FROM schema_migrations",
                    [],
                    |row| row.get(0),
                )
            })
            .await
            .map_err(|e| OpenCodeError::Storage(e.to_string()))??;

        for version in (db_version + 1)..=self.current_version {
            self.apply_migration(&conn, version).await?;
        }

        Ok(())
    }

    async fn apply_migration(
        &self,
        conn: &PooledConnection,
        version: i32,
    ) -> Result<(), OpenCodeError> {
        conn.execute(move |c| {
            if version == 1 {
                c.execute_batch(
                    "CREATE TABLE sessions (
                        id TEXT PRIMARY KEY,
                        created_at TIMESTAMP NOT NULL,
                        updated_at TIMESTAMP NOT NULL,
                        data TEXT NOT NULL
                    );
                    CREATE INDEX idx_sessions_updated_at ON sessions(updated_at);
                    
                    CREATE TABLE projects (
                        id TEXT PRIMARY KEY,
                        path TEXT NOT NULL UNIQUE,
                        name TEXT,
                        created_at TIMESTAMP NOT NULL,
                        updated_at TIMESTAMP NOT NULL,
                        data TEXT
                    );
                    CREATE INDEX idx_projects_path ON projects(path);
                    CREATE INDEX idx_projects_updated_at ON projects(updated_at);
                    
                    CREATE TABLE accounts (
                        id TEXT PRIMARY KEY,
                        username TEXT NOT NULL UNIQUE,
                        email TEXT UNIQUE,
                        password_hash TEXT NOT NULL,
                        created_at TIMESTAMP NOT NULL,
                        updated_at TIMESTAMP NOT NULL,
                        last_login_at TIMESTAMP,
                        is_active BOOLEAN NOT NULL DEFAULT 1,
                        data TEXT
                    );
                    CREATE INDEX idx_accounts_username ON accounts(username);
                    CREATE INDEX idx_accounts_email ON accounts(email);
                    CREATE INDEX idx_accounts_updated_at ON accounts(updated_at);
                    
                    CREATE TABLE permissions (
                        user_id TEXT NOT NULL,
                        permission TEXT NOT NULL,
                        is_deny BOOLEAN NOT NULL DEFAULT 0,
                        PRIMARY KEY (user_id, permission, is_deny),
                        FOREIGN KEY (user_id) REFERENCES accounts(id)
                    );
                    CREATE INDEX idx_permissions_user_id ON permissions(user_id);",
                )?;

                c.execute(
                    "INSERT INTO schema_migrations (version) VALUES (?)",
                    params![version],
                )?;
            }
            Ok(())
        })
        .await
        .map_err(|e| OpenCodeError::Storage(e.to_string()))?
        .map_err(|e: rusqlite::Error| OpenCodeError::Storage(e.to_string()))
    }
}
