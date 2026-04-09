use std::sync::Arc;
use std::time::Duration;

use tokio::sync::Semaphore;
use tokio::time::timeout;

use crate::client::{McpClient, McpError, McpTransport};

#[derive(Debug, Clone)]
pub struct PoolConfig {
    pub max_connections_per_endpoint: usize,
    pub max_total_connections: usize,
    pub min_idle_per_endpoint: usize,
    pub max_idle_per_endpoint: usize,
    pub idle_timeout: Duration,
    pub acquire_timeout: Duration,
    pub max_acquire_retries: usize,
    pub retry_base_delay: Duration,
}

impl Default for PoolConfig {
    fn default() -> Self {
        Self {
            max_connections_per_endpoint: 4,
            max_total_connections: 16,
            min_idle_per_endpoint: 1,
            max_idle_per_endpoint: 2,
            idle_timeout: Duration::from_secs(300),
            acquire_timeout: Duration::from_secs(30),
            max_acquire_retries: 3,
            retry_base_delay: Duration::from_millis(100),
        }
    }
}

impl PoolConfig {
    pub fn new(max_connections_per_endpoint: usize, max_total_connections: usize) -> Self {
        Self {
            max_connections_per_endpoint,
            max_total_connections,
            ..Default::default()
        }
    }

    pub fn with_idle_timeout(mut self, timeout: Duration) -> Self {
        self.idle_timeout = timeout;
        self
    }

    pub fn with_acquire_timeout(mut self, timeout: Duration) -> Self {
        self.acquire_timeout = timeout;
        self
    }
}

#[derive(Debug, Clone)]
pub struct EndpointPoolStats {
    pub endpoint: String,
    pub total_connections: usize,
    pub idle_connections: usize,
}

#[derive(Debug, Clone)]
pub struct PoolStats {
    pub total_connections: usize,
    pub total_idle: usize,
    pub endpoints: Vec<EndpointPoolStats>,
}

pub struct McpConnectionPool {
    config: PoolConfig,
    total_connections: usize,
    semaphore: Arc<Semaphore>,
}

impl std::fmt::Debug for McpConnectionPool {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("McpConnectionPool")
            .field("config", &self.config)
            .field("total_connections", &self.total_connections)
            .finish()
    }
}

pub struct PooledClient {
    inner: Option<McpClient>,
    released: bool,
}

impl std::fmt::Debug for PooledClient {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("PooledClient")
            .field("released", &self.released)
            .finish()
    }
}

impl PooledClient {
    pub fn client(&self) -> &McpClient {
        self.inner.as_ref().expect("client already released")
    }

    pub fn client_mut(&mut self) -> &mut McpClient {
        self.inner.as_mut().expect("client already released")
    }

    pub async fn release(mut self) {
        self.released = true;
        if let Some(client) = self.inner.take() {
            let _ = client.disconnect();
        }
    }
}

impl Drop for PooledClient {
    fn drop(&mut self) {}
}

impl std::ops::Deref for PooledClient {
    type Target = McpClient;

    fn deref(&self) -> &Self::Target {
        self.client()
    }
}

impl std::ops::DerefMut for PooledClient {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.client_mut()
    }
}

impl McpConnectionPool {
    pub fn new(config: PoolConfig) -> Self {
        let max_connections = config.max_total_connections;
        Self {
            config,
            total_connections: 0,
            semaphore: Arc::new(Semaphore::new(max_connections)),
        }
    }

    pub fn config(&self) -> &PoolConfig {
        &self.config
    }

    pub fn stats(&self) -> PoolStats {
        PoolStats {
            total_connections: self.total_connections,
            total_idle: 0,
            endpoints: Vec::new(),
        }
    }

    pub async fn get_client(&self, endpoint: &str) -> Result<PooledClient, McpError> {
        let mut last_error = None;

        for attempt in 0..self.config.max_acquire_retries {
            let permit = match timeout(self.config.acquire_timeout, self.semaphore.acquire()).await
            {
                Ok(Ok(p)) => p,
                Ok(Err(_)) => {
                    return Err(McpError::Other("semaphore closed".to_string()));
                }
                Err(_) => {
                    return Err(McpError::Timeout(self.config.acquire_timeout));
                }
            };

            drop(permit);

            let client = match self.get_or_create_connection(endpoint).await {
                Ok(c) => c,
                Err(e) => {
                    last_error = Some(e.clone());

                    if attempt < self.config.max_acquire_retries - 1 {
                        let delay = self
                            .config
                            .retry_base_delay
                            .saturating_mul(1u32 << attempt.min(7));
                        tokio::time::sleep(delay).await;
                        continue;
                    } else {
                        return Err(e);
                    }
                }
            };

            return Ok(PooledClient {
                inner: Some(client),
                released: false,
            });
        }

        Err(last_error.unwrap_or_else(|| McpError::Other("failed to acquire client".to_string())))
    }

    pub async fn return_client(&self, _endpoint: &str, client: McpClient) {
        let needs_reconnect = !client.is_connected().await;

        if needs_reconnect {
            if let Err(e) = client.connect().await {
                tracing::warn!("failed to reconnect client: {}", e);
                drop(client);
                return;
            }
        }

        drop(client);
    }

    async fn get_or_create_connection(&self, endpoint: &str) -> Result<McpClient, McpError> {
        let transport = self.parse_endpoint(endpoint)?;
        let client = McpClient::new(transport).with_timeout(self.config.acquire_timeout);

        client.connect().await?;

        Ok(client)
    }

    fn parse_endpoint(&self, endpoint: &str) -> Result<McpTransport, McpError> {
        if endpoint.starts_with("stdio://") {
            let cmd = endpoint.trim_start_matches("stdio://");
            Ok(McpTransport::Stdio(crate::client::StdioProcess::new(
                cmd,
                vec![],
            )))
        } else if endpoint.starts_with("sse://") {
            let url = endpoint.trim_start_matches("sse://");
            Ok(McpTransport::Sse(url.to_string()))
        } else if endpoint.starts_with("http://") || endpoint.starts_with("https://") {
            Ok(McpTransport::Sse(endpoint.to_string()))
        } else {
            Ok(McpTransport::Stdio(crate::client::StdioProcess::new(
                endpoint,
                vec![],
            )))
        }
    }

    pub async fn close(&mut self) {
        self.total_connections = 0;
    }

    pub fn num_endpoints(&self) -> usize {
        0
    }

    pub fn num_connections(&self) -> usize {
        self.total_connections
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pool_config_default() {
        let config = PoolConfig::default();
        assert_eq!(config.max_connections_per_endpoint, 4);
        assert_eq!(config.max_total_connections, 16);
        assert_eq!(config.min_idle_per_endpoint, 1);
        assert_eq!(config.idle_timeout, Duration::from_secs(300));
    }

    #[test]
    fn test_pool_config_builder() {
        let config = PoolConfig::new(10, 50)
            .with_idle_timeout(Duration::from_secs(600))
            .with_acquire_timeout(Duration::from_secs(60));

        assert_eq!(config.max_connections_per_endpoint, 10);
        assert_eq!(config.max_total_connections, 50);
        assert_eq!(config.idle_timeout, Duration::from_secs(600));
        assert_eq!(config.acquire_timeout, Duration::from_secs(60));
    }

    #[tokio::test]
    async fn test_pool_stats() {
        let config = PoolConfig::default();
        let pool = McpConnectionPool::new(config);

        let stats = pool.stats();
        assert_eq!(stats.total_connections, 0);
        assert_eq!(stats.total_idle, 0);
        assert!(stats.endpoints.is_empty());
    }

    #[tokio::test]
    async fn test_parse_endpoint_stdio() {
        let config = PoolConfig::default();
        let pool = McpConnectionPool::new(config);

        let transport = pool.parse_endpoint("stdio://my-server").unwrap();
        match transport {
            McpTransport::Stdio(process) => {
                assert_eq!(process.command, "my-server");
            }
            _ => panic!("expected Stdio transport"),
        }
    }

    #[tokio::test]
    async fn test_parse_endpoint_sse() {
        let config = PoolConfig::default();
        let pool = McpConnectionPool::new(config);

        let transport = pool
            .parse_endpoint("sse://http://localhost:3000/sse")
            .unwrap();
        match transport {
            McpTransport::Sse(url) => {
                assert_eq!(url, "http://localhost:3000/sse");
            }
            _ => panic!("expected Sse transport"),
        }
    }

    #[tokio::test]
    async fn test_parse_endpoint_http() {
        let config = PoolConfig::default();
        let pool = McpConnectionPool::new(config);

        let transport = pool.parse_endpoint("http://localhost:3000/sse").unwrap();
        match transport {
            McpTransport::Sse(url) => {
                assert_eq!(url, "http://localhost:3000/sse");
            }
            _ => panic!("expected Sse transport"),
        }
    }

    #[tokio::test]
    async fn test_parse_endpoint_default_stdio() {
        let config = PoolConfig::default();
        let pool = McpConnectionPool::new(config);

        let transport = pool.parse_endpoint("my-server").unwrap();
        match transport {
            McpTransport::Stdio(process) => {
                assert_eq!(process.command, "my-server");
            }
            _ => panic!("expected Stdio transport"),
        }
    }
}
