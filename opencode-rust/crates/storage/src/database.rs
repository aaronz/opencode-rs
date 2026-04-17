use std::path::Path;
use std::sync::Arc;

use deadpool_sqlite::{Config, Manager, Pool, Runtime};
use rusqlite::Connection;

use crate::OpenCodeError;

#[derive(Clone)]
pub struct StoragePool {
    inner: Arc<Pool>,
}

impl StoragePool {
    pub fn new<P: AsRef<Path>>(path: P) -> Result<Self, OpenCodeError> {
        let config = Config::new(path.as_ref().to_path_buf());

        let pool = config
            .create_pool(Runtime::Tokio1)
            .map_err(|e| OpenCodeError::Storage(e.to_string()))?;
        Ok(Self {
            inner: Arc::new(pool),
        })
    }

    pub async fn get(&self) -> Result<PooledConnection, OpenCodeError> {
        let conn = self
            .inner
            .get()
            .await
            .map_err(|e| OpenCodeError::Storage(e.to_string()))?;
        Ok(PooledConnection { inner: conn })
    }
}

pub struct PooledConnection {
    inner: deadpool::managed::Object<Manager>,
}

impl PooledConnection {
    pub async fn execute<F, R>(&self, f: F) -> Result<R, OpenCodeError>
    where
        F: FnOnce(&mut Connection) -> R + Send + 'static,
        R: Send + 'static,
    {
        self.inner
            .interact(f)
            .await
            .map_err(|e| OpenCodeError::Storage(e.to_string()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_storage_pool_new_and_get() {
        let temp_dir = tempfile::tempdir().unwrap();
        let db_path = temp_dir.path().join("test.db");

        let pool = StoragePool::new(&db_path).unwrap();
        let conn = pool.get().await.unwrap();

        let result = conn
            .execute(|c| c.query_row("SELECT 42", [], |row| row.get::<_, i32>(0)))
            .await
            .unwrap()
            .unwrap();

        assert_eq!(result, 42);
        drop(temp_dir);
    }

    #[tokio::test]
    async fn test_pooled_connection_execute() {
        let temp_dir = tempfile::tempdir().unwrap();
        let db_path = temp_dir.path().join("test.db");

        let pool = StoragePool::new(&db_path).unwrap();
        let conn = pool.get().await.unwrap();

        conn.execute(|c| c.execute("CREATE TABLE test (id INTEGER)", []))
            .await
            .unwrap()
            .unwrap();

        conn.execute(|c| c.execute("INSERT INTO test VALUES (1)", []))
            .await
            .unwrap()
            .unwrap();

        let count = conn
            .execute(|c| c.query_row("SELECT COUNT(*) FROM test", [], |row| row.get::<_, i32>(0)))
            .await
            .unwrap()
            .unwrap();

        assert_eq!(count, 1);
        drop(temp_dir);
    }

    #[tokio::test]
    async fn test_storage_pool_clone() {
        let temp_dir = tempfile::tempdir().unwrap();
        let db_path = temp_dir.path().join("test.db");

        let pool1 = StoragePool::new(&db_path).unwrap();
        let pool2 = pool1.clone();

        let conn1 = pool1.get().await.unwrap();
        let conn2 = pool2.get().await.unwrap();

        conn1
            .execute(|c| c.execute("CREATE TABLE clone_test (id INTEGER)", []))
            .await
            .unwrap()
            .unwrap();

        conn1
            .execute(|c| c.execute("INSERT INTO clone_test VALUES (1)", []))
            .await
            .unwrap()
            .unwrap();

        let count = conn2
            .execute(|c| {
                c.query_row("SELECT COUNT(*) FROM clone_test", [], |row| {
                    row.get::<_, i32>(0)
                })
            })
            .await
            .unwrap()
            .unwrap();

        assert_eq!(count, 1);
        drop(temp_dir);
    }
}
