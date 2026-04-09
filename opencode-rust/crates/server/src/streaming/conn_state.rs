use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;

use serde::{Deserialize, Serialize};
use tokio::sync::{broadcast, RwLock};

/// Connection type (SSE or WebSocket)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ConnectionType {
    Sse,
    WebSocket,
}

/// Connection state
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ConnectionStatus {
    Connecting,
    Connected,
    Reconnecting,
    Disconnected,
    Failed,
}

/// Information about a single connection
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConnectionInfo {
    pub id: String,
    pub connection_type: ConnectionType,
    pub session_id: String,
    pub status: ConnectionStatus,
    pub created_at: i64,
    pub last_heartbeat: Option<i64>,
    pub heartbeat_failures: u32,
    pub reconnection_attempts: u32,
    pub bytes_sent: u64,
    pub bytes_received: u64,
}

/// Statistics for connections
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ConnectionStats {
    pub total_connections: usize,
    pub active_connections: usize,
    pub sse_connections: usize,
    pub ws_connections: usize,
    pub total_heartbeat_failures: u32,
    pub total_reconnections: u32,
    pub average_heartbeat_interval_ms: u64,
}

/// Connection monitor events
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ConnectionEvent {
    Connected {
        connection_id: String,
        connection_type: ConnectionType,
        session_id: String,
    },
    Disconnected {
        connection_id: String,
        reason: String,
    },
    HeartbeatSuccess {
        connection_id: String,
    },
    HeartbeatFailure {
        connection_id: String,
        failure_count: u32,
    },
    Reconnecting {
        connection_id: String,
        attempt: u32,
    },
    Failed {
        connection_id: String,
        error: String,
    },
}

/// Global connection monitor for tracking all connections
pub struct ConnectionMonitor {
    connections: Arc<RwLock<HashMap<String, ConnectionInfo>>>,
    event_tx: broadcast::Sender<ConnectionEvent>,
    stats: Arc<RwLock<ConnectionStats>>,
}

impl Default for ConnectionMonitor {
    fn default() -> Self {
        let (event_tx, _) = broadcast::channel(1024);
        Self {
            connections: Arc::new(RwLock::new(HashMap::new())),
            event_tx,
            stats: Arc::new(RwLock::new(ConnectionStats::default())),
        }
    }
}

impl ConnectionMonitor {
    /// Create a new connection monitor
    pub fn new() -> Self {
        Self::default()
    }

    /// Subscribe to connection events
    pub fn subscribe(&self) -> broadcast::Receiver<ConnectionEvent> {
        self.event_tx.subscribe()
    }

    /// Register a new connection
    pub async fn register_connection(
        &self,
        connection_id: String,
        connection_type: ConnectionType,
        session_id: String,
    ) {
        let now = chrono::Utc::now().timestamp();
        let info = ConnectionInfo {
            id: connection_id.clone(),
            connection_type,
            session_id: session_id.clone(),
            status: ConnectionStatus::Connected,
            created_at: now,
            last_heartbeat: Some(now),
            heartbeat_failures: 0,
            reconnection_attempts: 0,
            bytes_sent: 0,
            bytes_received: 0,
        };

        {
            let mut connections = self.connections.write().await;
            connections.insert(connection_id.clone(), info);
        }

        {
            let mut stats = self.stats.write().await;
            stats.total_connections += 1;
            stats.active_connections += 1;
            match connection_type {
                ConnectionType::Sse => stats.sse_connections += 1,
                ConnectionType::WebSocket => stats.ws_connections += 1,
            }
        }

        let _ = self.event_tx.send(ConnectionEvent::Connected {
            connection_id,
            connection_type,
            session_id,
        });
    }

    /// Unregister a connection
    pub async fn unregister_connection(&self, connection_id: &str, reason: &str) {
        let removed = {
            let mut connections = self.connections.write().await;
            connections.remove(connection_id)
        };

        if let Some(info) = removed {
            let mut stats = self.stats.write().await;
            stats.active_connections = stats.active_connections.saturating_sub(1);
            match info.connection_type {
                ConnectionType::Sse => {
                    stats.sse_connections = stats.sse_connections.saturating_sub(1)
                }
                ConnectionType::WebSocket => {
                    stats.ws_connections = stats.ws_connections.saturating_sub(1)
                }
            }
            stats.total_heartbeat_failures += info.heartbeat_failures;
            stats.total_reconnections += info.reconnection_attempts;
        }

        let _ = self.event_tx.send(ConnectionEvent::Disconnected {
            connection_id: connection_id.to_string(),
            reason: reason.to_string(),
        });
    }

    /// Record a successful heartbeat
    pub async fn heartbeat_success(&self, connection_id: &str) {
        let now = chrono::Utc::now().timestamp();
        let mut should_notify = false;
        let mut failure_count = 0u32;

        {
            let mut connections = self.connections.write().await;
            if let Some(info) = connections.get_mut(connection_id) {
                info.last_heartbeat = Some(now);
                if info.heartbeat_failures > 0 {
                    failure_count = info.heartbeat_failures;
                    info.heartbeat_failures = 0;
                    should_notify = true;
                }
            }
        }

        let _ = self.event_tx.send(ConnectionEvent::HeartbeatSuccess {
            connection_id: connection_id.to_string(),
        });

        if should_notify && failure_count > 0 {
            let _ = self.event_tx.send(ConnectionEvent::HeartbeatFailure {
                connection_id: connection_id.to_string(),
                failure_count,
            });
        }
    }

    /// Record a heartbeat failure
    pub async fn heartbeat_failure(&self, connection_id: &str) {
        let mut connections = self.connections.write().await;
        if let Some(info) = connections.get_mut(connection_id) {
            info.heartbeat_failures += 1;

            let _ = self.event_tx.send(ConnectionEvent::HeartbeatFailure {
                connection_id: connection_id.to_string(),
                failure_count: info.heartbeat_failures,
            });
        }
    }

    /// Record a reconnection attempt
    pub async fn reconnection_attempt(&self, connection_id: &str, attempt: u32) {
        let mut connections = self.connections.write().await;
        if let Some(info) = connections.get_mut(connection_id) {
            info.reconnection_attempts = attempt;
            info.status = ConnectionStatus::Reconnecting;
        }

        let _ = self.event_tx.send(ConnectionEvent::Reconnecting {
            connection_id: connection_id.to_string(),
            attempt,
        });
    }

    /// Mark a connection as failed
    pub async fn connection_failed(&self, connection_id: &str, error: &str) {
        let mut connections = self.connections.write().await;
        if let Some(info) = connections.get_mut(connection_id) {
            info.status = ConnectionStatus::Failed;
        }

        let _ = self.event_tx.send(ConnectionEvent::Failed {
            connection_id: connection_id.to_string(),
            error: error.to_string(),
        });
    }

    /// Update connection status
    pub async fn update_status(&self, connection_id: &str, status: ConnectionStatus) {
        let mut connections = self.connections.write().await;
        if let Some(info) = connections.get_mut(connection_id) {
            info.status = status;
        }
    }

    /// Update bytes sent/received
    pub async fn update_bytes(&self, connection_id: &str, sent: u64, received: u64) {
        let mut connections = self.connections.write().await;
        if let Some(info) = connections.get_mut(connection_id) {
            info.bytes_sent += sent;
            info.bytes_received += received;
        }
    }

    /// Get connection info
    pub async fn get_connection(&self, connection_id: &str) -> Option<ConnectionInfo> {
        let connections = self.connections.read().await;
        connections.get(connection_id).cloned()
    }

    /// Get all connections for a session
    pub async fn get_session_connections(&self, session_id: &str) -> Vec<ConnectionInfo> {
        let connections = self.connections.read().await;
        connections
            .values()
            .filter(|c| c.session_id == session_id)
            .cloned()
            .collect()
    }

    /// Get connection statistics
    pub async fn get_stats(&self) -> ConnectionStats {
        let stats = self.stats.read().await;
        stats.clone()
    }

    /// Get all active connections
    pub async fn get_active_connections(&self) -> Vec<ConnectionInfo> {
        let connections = self.connections.read().await;
        connections
            .values()
            .filter(|c| {
                c.status == ConnectionStatus::Connected
                    || c.status == ConnectionStatus::Reconnecting
            })
            .cloned()
            .collect()
    }

    /// Check if a connection is healthy (recent heartbeat)
    pub async fn is_connection_healthy(
        &self,
        connection_id: &str,
        max_heartbeat_age: Duration,
    ) -> bool {
        let connections = self.connections.read().await;
        if let Some(info) = connections.get(connection_id) {
            if let Some(last_heartbeat) = info.last_heartbeat {
                let last_heartbeat_time = chrono::DateTime::from_timestamp(last_heartbeat, 0)
                    .unwrap_or_else(|| chrono::Utc::now());
                let age = chrono::Utc::now().signed_duration_since(last_heartbeat_time);
                return age
                    < chrono::Duration::from_std(max_heartbeat_age)
                        .unwrap_or(chrono::Duration::MAX);
            }
        }
        false
    }

    /// Cleanup stale connections
    pub async fn cleanup_stale_connections(&self, max_heartbeat_age: Duration) -> Vec<String> {
        let mut connections = self.connections.write().await;
        let now = chrono::Utc::now();
        let max_age =
            chrono::Duration::from_std(max_heartbeat_age).unwrap_or(chrono::Duration::MAX);

        let stale_ids: Vec<String> = connections
            .iter()
            .filter(|(_, info)| {
                if info.status == ConnectionStatus::Disconnected
                    || info.status == ConnectionStatus::Failed
                {
                    return true;
                }
                if let Some(last_hb) = info.last_heartbeat {
                    let last_hb_time = chrono::DateTime::from_timestamp(last_hb, 0)
                        .unwrap_or_else(|| chrono::Utc::now());
                    let age = now.signed_duration_since(last_hb_time);
                    age > max_age
                } else {
                    false
                }
            })
            .map(|(id, _)| id.clone())
            .collect();

        for id in &stale_ids {
            connections.remove(id);
        }

        stale_ids
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_register_and_unregister_connection() {
        let monitor = ConnectionMonitor::new();

        monitor
            .register_connection(
                "conn-1".to_string(),
                ConnectionType::Sse,
                "session-1".to_string(),
            )
            .await;

        let info = monitor.get_connection("conn-1").await;
        assert!(info.is_some());
        assert_eq!(info.unwrap().connection_type, ConnectionType::Sse);

        monitor
            .unregister_connection("conn-1", "normal_close")
            .await;

        let info = monitor.get_connection("conn-1").await;
        assert!(info.is_none());
    }

    #[tokio::test]
    async fn test_heartbeat_tracking() {
        let monitor = ConnectionMonitor::new();

        monitor
            .register_connection(
                "conn-1".to_string(),
                ConnectionType::WebSocket,
                "session-1".to_string(),
            )
            .await;

        monitor.heartbeat_failure("conn-1").await;
        monitor.heartbeat_failure("conn-1").await;

        let info = monitor.get_connection("conn-1").await.unwrap();
        assert_eq!(info.heartbeat_failures, 2);

        monitor.heartbeat_success("conn-1").await;

        let info = monitor.get_connection("conn-1").await.unwrap();
        assert_eq!(info.heartbeat_failures, 0);
    }

    #[tokio::test]
    async fn test_stats_tracking() {
        let monitor = ConnectionMonitor::new();

        monitor
            .register_connection(
                "conn-1".to_string(),
                ConnectionType::Sse,
                "session-1".to_string(),
            )
            .await;
        monitor
            .register_connection(
                "conn-2".to_string(),
                ConnectionType::WebSocket,
                "session-1".to_string(),
            )
            .await;

        let stats = monitor.get_stats().await;
        assert_eq!(stats.total_connections, 2);
        assert_eq!(stats.active_connections, 2);
        assert_eq!(stats.sse_connections, 1);
        assert_eq!(stats.ws_connections, 1);

        monitor.unregister_connection("conn-1", "done").await;

        let stats = monitor.get_stats().await;
        assert_eq!(stats.active_connections, 1);
    }

    #[tokio::test]
    async fn test_session_connections() {
        let monitor = ConnectionMonitor::new();

        monitor
            .register_connection(
                "conn-1".to_string(),
                ConnectionType::Sse,
                "session-1".to_string(),
            )
            .await;
        monitor
            .register_connection(
                "conn-2".to_string(),
                ConnectionType::WebSocket,
                "session-2".to_string(),
            )
            .await;
        monitor
            .register_connection(
                "conn-3".to_string(),
                ConnectionType::Sse,
                "session-1".to_string(),
            )
            .await;

        let session1_conns = monitor.get_session_connections("session-1").await;
        assert_eq!(session1_conns.len(), 2);

        let session2_conns = monitor.get_session_connections("session-2").await;
        assert_eq!(session2_conns.len(), 1);
    }
}
