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
