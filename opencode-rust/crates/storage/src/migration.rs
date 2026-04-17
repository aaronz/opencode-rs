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
            } else if version == 2 {
                c.execute_batch(
                    "CREATE TABLE plugin_states (
                        plugin_id TEXT PRIMARY KEY,
                        state_data TEXT NOT NULL,
                        updated_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP
                    );
                    CREATE INDEX idx_plugin_states_updated_at ON plugin_states(updated_at);",
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::StoragePool;

    #[tokio::test]
    async fn test_migration_manager_new() {
        let temp_dir = tempfile::tempdir().unwrap();
        let db_path = temp_dir.path().join("test.db");
        let pool = StoragePool::new(&db_path).unwrap();

        let manager = MigrationManager::new(pool, 2);
        assert_eq!(manager.current_version, 2);

        drop(temp_dir);
    }

    #[tokio::test]
    async fn test_migrate_creates_tables() {
        let temp_dir = tempfile::tempdir().unwrap();
        let db_path = temp_dir.path().join("test.db");
        let pool = StoragePool::new(&db_path).unwrap();

        let manager = MigrationManager::new(pool.clone(), 2);
        manager.migrate().await.unwrap();

        let conn = pool.get().await.unwrap();
        let session_count = conn
            .execute(|c| {
                c.query_row("SELECT COUNT(*) FROM sessions", [], |row| {
                    row.get::<_, i32>(0)
                })
            })
            .await
            .unwrap()
            .unwrap();
        assert_eq!(session_count, 0);

        let project_count = conn
            .execute(|c| {
                c.query_row("SELECT COUNT(*) FROM projects", [], |row| {
                    row.get::<_, i32>(0)
                })
            })
            .await
            .unwrap()
            .unwrap();
        assert_eq!(project_count, 0);

        drop(temp_dir);
    }

    #[tokio::test]
    async fn test_migrate_idempotent() {
        let temp_dir = tempfile::tempdir().unwrap();
        let db_path = temp_dir.path().join("test.db");
        let pool = StoragePool::new(&db_path).unwrap();

        let manager = MigrationManager::new(pool.clone(), 2);

        manager.migrate().await.unwrap();
        manager.migrate().await.unwrap();

        let conn = pool.get().await.unwrap();
        let version = conn
            .execute(|c| {
                c.query_row("SELECT MAX(version) FROM schema_migrations", [], |row| {
                    row.get::<_, i32>(0)
                })
            })
            .await
            .unwrap()
            .unwrap();
        assert_eq!(version, 2);

        drop(temp_dir);
    }

    #[tokio::test]
    async fn test_migration_version_tracking() {
        let temp_dir = tempfile::tempdir().unwrap();
        let db_path = temp_dir.path().join("test.db");
        let pool = StoragePool::new(&db_path).unwrap();

        let manager_v1 = MigrationManager::new(pool.clone(), 1);
        manager_v1.migrate().await.unwrap();

        let conn = pool.get().await.unwrap();
        let version = conn
            .execute(|c| {
                c.query_row("SELECT MAX(version) FROM schema_migrations", [], |row| {
                    row.get::<_, i32>(0)
                })
            })
            .await
            .unwrap()
            .unwrap();
        assert_eq!(version, 1);

        drop(temp_dir);
    }
}
