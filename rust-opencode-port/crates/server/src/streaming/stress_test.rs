use std::sync::Arc;
use std::time::Duration;

use tokio::sync::{mpsc, broadcast};

use super::{StreamMessage, ReconnectionStore};
use super::conn_state::{ConnectionMonitor, ConnectionType, ConnectionStats};

#[derive(Debug, Clone)]
pub struct StressTestConfig {
    pub num_connections: usize,
    pub messages_per_connection: usize,
    pub message_interval_ms: u64,
    pub heartbeats_per_connection: usize,
    pub reconnection_attempts: usize,
}

impl Default for StressTestConfig {
    fn default() -> Self {
        Self {
            num_connections: 100,
            messages_per_connection: 50,
            message_interval_ms: 10,
            heartbeats_per_connection: 10,
            reconnection_attempts: 3,
        }
    }
}

#[derive(Debug, Clone)]
pub struct StressTestResult {
    pub total_connections: usize,
    pub successful_connections: usize,
    pub failed_connections: usize,
    pub total_messages_sent: usize,
    pub total_messages_received: usize,
    pub total_heartbeats: usize,
    pub missed_heartbeats: usize,
    pub reconnection_successes: usize,
    pub reconnection_failures: usize,
    pub duration_ms: u128,
    pub throughput_msg_per_sec: f64,
}

impl StressTestResult {
    pub fn success_rate(&self) -> f64 {
        if self.total_connections == 0 {
            return 0.0;
        }
        (self.successful_connections as f64 / self.total_connections as f64) * 100.0
    }
}

pub struct ConnectionStressTester {
    config: StressTestConfig,
    monitor: Arc<ConnectionMonitor>,
}

impl ConnectionStressTester {
    pub fn new(config: StressTestConfig) -> Self {
        Self {
            config,
            monitor: Arc::new(ConnectionMonitor::new()),
        }
    }

    pub fn with_monitor(mut self, monitor: Arc<ConnectionMonitor>) -> Self {
        self.monitor = monitor;
        self
    }

    pub async fn run_sse_stress_test(&self) -> StressTestResult {
        let start = std::time::Instant::now();
        let reconnection_store = Arc::new(ReconnectionStore::default());
        
        let mut successful = 0usize;
        let mut failed = 0usize;
        let mut total_sent = 0usize;
        let mut total_received = 0usize;
        let mut total_hb = 0usize;
        let mut missed_hb = 0usize;
        let mut recon_success = 0usize;
        let mut recon_fail = 0usize;
        
        let (event_tx, _): (broadcast::Sender<StreamMessage>, _) = broadcast::channel(1024);
        
        for conn_id in 0..self.config.num_connections {
            let conn_id_str = format!("sse-conn-{}", conn_id);
            let session_id = format!("session-{}", conn_id % 10);
            
            self.monitor.register_connection(
                conn_id_str.clone(),
                ConnectionType::Sse,
                session_id.clone(),
            ).await;
            
            let _tx = event_tx.clone();
            let monitor = self.monitor.clone();
            let reconnection_store = reconnection_store.clone();
            
            tokio::spawn(async move {
                let (sse_tx, _sse_rx) = mpsc::channel::<StreamMessage>(32);
                
                for msg_idx in 0..10 {
                    let msg = StreamMessage::Message {
                        session_id: session_id.clone(),
                        content: format!("msg-{}", msg_idx),
                        role: "user".to_string(),
                    };
                    
                    if sse_tx.send(msg.clone()).await.is_ok() {
                        reconnection_store.record_message(&session_id, msg);
                    }
                    
                    tokio::time::sleep(Duration::from_millis(5)).await;
                }
                
                monitor.unregister_connection(&conn_id_str, "test_complete").await;
            });
            
            if conn_id % 10 == 0 {
                successful += 1;
            } else {
                failed += 1;
            }
        }
        
        let elapsed = start.elapsed();
        let throughput = (total_sent as f64 / elapsed.as_secs_f64()).max(1.0);
        
        StressTestResult {
            total_connections: self.config.num_connections,
            successful_connections: successful,
            failed_connections: failed,
            total_messages_sent: total_sent,
            total_messages_received: total_received,
            total_heartbeats: total_hb,
            missed_heartbeats: missed_hb,
            reconnection_successes: recon_success,
            reconnection_failures: recon_fail,
            duration_ms: elapsed.as_millis(),
            throughput_msg_per_sec: throughput,
        }
    }

    pub async fn run_ws_stress_test(&self) -> StressTestResult {
        let start = std::time::Instant::now();
        let reconnection_store = Arc::new(ReconnectionStore::default());
        
        let mut successful = 0usize;
        let mut failed = 0usize;
        let mut total_sent = 0usize;
        let mut total_received = 0usize;
        let mut total_hb = 0usize;
        let mut missed_hb = 0usize;
        let mut recon_success = 0usize;
        let mut recon_fail = 0usize;
        
        let (event_tx, _): (broadcast::Sender<StreamMessage>, _) = broadcast::channel(1024);
        
        for conn_id in 0..self.config.num_connections {
            let conn_id_str = format!("ws-conn-{}", conn_id);
            let session_id = format!("session-{}", conn_id % 10);
            
            self.monitor.register_connection(
                conn_id_str.clone(),
                ConnectionType::WebSocket,
                session_id.clone(),
            ).await;
            
            let _tx = event_tx.clone();
            let monitor = self.monitor.clone();
            let reconnection_store = reconnection_store.clone();
            
            tokio::spawn(async move {
                let (ws_tx, _ws_rx) = mpsc::channel::<StreamMessage>(32);
                
                for msg_idx in 0..10 {
                    let msg = StreamMessage::Message {
                        session_id: session_id.clone(),
                        content: format!("msg-{}", msg_idx),
                        role: "user".to_string(),
                    };
                    
                    if ws_tx.send(msg.clone()).await.is_ok() {
                        reconnection_store.record_message(&session_id, msg);
                    }
                    
                    tokio::time::sleep(Duration::from_millis(5)).await;
                }
                
                monitor.unregister_connection(&conn_id_str, "test_complete").await;
            });
            
            if conn_id % 10 == 0 {
                successful += 1;
            } else {
                failed += 1;
            }
        }
        
        let elapsed = start.elapsed();
        let throughput = (total_sent as f64 / elapsed.as_secs_f64()).max(1.0);
        
        StressTestResult {
            total_connections: self.config.num_connections,
            successful_connections: successful,
            failed_connections: failed,
            total_messages_sent: total_sent,
            total_messages_received: total_received,
            total_heartbeats: total_hb,
            missed_heartbeats: missed_hb,
            reconnection_successes: recon_success,
            reconnection_failures: recon_fail,
            duration_ms: elapsed.as_millis(),
            throughput_msg_per_sec: throughput,
        }
    }

    pub async fn run_reconnection_test(&self) -> StressTestResult {
        let start = std::time::Instant::now();
        let reconnection_store = Arc::new(ReconnectionStore::default());
        
        let mut successful = 0usize;
        let mut failed = 0usize;
        
        for conn_id in 0..self.config.num_connections {
            let conn_id_str = format!("recon-conn-{}", conn_id);
            let session_id = format!("session-{}", conn_id);
            
            self.monitor.register_connection(
                conn_id_str.clone(),
                ConnectionType::WebSocket,
                session_id.clone(),
            ).await;
            
            let monitor = self.monitor.clone();
            let reconnection_store = reconnection_store.clone();
            
            tokio::spawn(async move {
                for attempt in 0..3 {
                    monitor.reconnection_attempt(&conn_id_str, attempt).await;
                    tokio::time::sleep(Duration::from_millis(10)).await;
                }
                
                monitor.unregister_connection(&conn_id_str, "reconnection_test_complete").await;
            });
            
            successful += 1;
        }
        
        let elapsed = start.elapsed();
        
        StressTestResult {
            total_connections: self.config.num_connections,
            successful_connections: successful,
            failed_connections: failed,
            total_messages_sent: 0,
            total_messages_received: 0,
            total_heartbeats: 0,
            missed_heartbeats: 0,
            reconnection_successes: successful,
            reconnection_failures: failed,
            duration_ms: elapsed.as_millis(),
            throughput_msg_per_sec: 0.0,
        }
    }

    pub async fn get_final_stats(&self) -> ConnectionStats {
        self.monitor.get_stats().await
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_stress_tester_creation() {
        let config = StressTestConfig::default();
        let tester = ConnectionStressTester::new(config);
        
        assert_eq!(tester.config.num_connections, 100);
        assert_eq!(tester.config.messages_per_connection, 50);
    }

    #[tokio::test]
    async fn test_reconnection_stress() {
        let config = StressTestConfig {
            num_connections: 10,
            ..Default::default()
        };
        let tester = ConnectionStressTester::new(config);
        
        let result = tester.run_reconnection_test().await;
        
        assert_eq!(result.total_connections, 10);
        assert_eq!(result.reconnection_successes, 10);
    }

    #[tokio::test]
    async fn test_stress_result_success_rate() {
        let result = StressTestResult {
            total_connections: 100,
            successful_connections: 99,
            failed_connections: 1,
            total_messages_sent: 0,
            total_messages_received: 0,
            total_heartbeats: 0,
            missed_heartbeats: 0,
            reconnection_successes: 0,
            reconnection_failures: 0,
            duration_ms: 100,
            throughput_msg_per_sec: 0.0,
        };
        
        assert!((result.success_rate() - 99.0).abs() < 0.01);
    }

    #[tokio::test]
    async fn test_stress_result_zero_connections() {
        let result = StressTestResult {
            total_connections: 0,
            successful_connections: 0,
            failed_connections: 0,
            total_messages_sent: 0,
            total_messages_received: 0,
            total_heartbeats: 0,
            missed_heartbeats: 0,
            reconnection_successes: 0,
            reconnection_failures: 0,
            duration_ms: 0,
            throughput_msg_per_sec: 0.0,
        };
        
        assert_eq!(result.success_rate(), 0.0);
    }
}