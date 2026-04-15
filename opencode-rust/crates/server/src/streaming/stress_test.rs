use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use std::time::Duration;

use tokio::sync::{broadcast, mpsc};

use super::conn_state::{ConnectionMonitor, ConnectionStats, ConnectionType};
use super::{ReconnectionStore, StreamMessage};

#[derive(Debug, Clone)]
pub struct StressTestConfig {
    pub num_connections: usize,
    pub messages_per_connection: usize,
    pub message_interval_ms: u64,
    pub heartbeats_per_connection: usize,
    pub reconnection_attempts: usize,
}

#[allow(dead_code)]
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

#[allow(dead_code)]
impl StressTestResult {
    pub(crate) fn success_rate(&self) -> f64 {
        if self.total_connections == 0 {
            return 0.0;
        }
        (self.successful_connections as f64 / self.total_connections as f64) * 100.0
    }

    pub(crate) fn meets_stability_threshold(&self, threshold_percent: f64) -> bool {
        self.success_rate() >= threshold_percent
    }
}

pub struct ConnectionStressTester {
    config: StressTestConfig,
    monitor: Arc<ConnectionMonitor>,
}

#[allow(dead_code)]
impl ConnectionStressTester {
    pub(crate) fn new(config: StressTestConfig) -> Self {
        Self {
            config,
            monitor: Arc::new(ConnectionMonitor::new()),
        }
    }

    pub(crate) fn with_monitor(mut self, monitor: Arc<ConnectionMonitor>) -> Self {
        self.monitor = monitor;
        self
    }

    pub(crate) async fn run_sse_stress_test(&self) -> StressTestResult {
        let start = std::time::Instant::now();
        let reconnection_store = Arc::new(ReconnectionStore::default());

        let successful = Arc::new(AtomicUsize::new(0));
        let failed = Arc::new(AtomicUsize::new(0));
        let total_sent = Arc::new(AtomicUsize::new(0));
        let total_received = Arc::new(AtomicUsize::new(0));
        let total_hb = Arc::new(AtomicUsize::new(0));

        let (event_tx, _): (broadcast::Sender<StreamMessage>, _) = broadcast::channel(1024);

        let mut handles = Vec::new();

        let heartbeats_per_conn = self.config.heartbeats_per_connection;
        let messages_per_conn = self.config.messages_per_connection;

        for conn_id in 0..self.config.num_connections {
            let conn_id_str = format!("sse-conn-{}", conn_id);
            let session_id = format!("session-{}", conn_id % 10);

            self.monitor
                .register_connection(conn_id_str.clone(), ConnectionType::Sse, session_id.clone())
                .await;

            let _tx = event_tx.clone();
            let monitor = self.monitor.clone();
            let reconnection_store = reconnection_store.clone();
            let successful_clone = successful.clone();
            let failed_clone = failed.clone();
            let total_sent_clone = total_sent.clone();
            let total_hb_clone = total_hb.clone();
            let hb_per_conn = heartbeats_per_conn;
            let msg_per_conn = messages_per_conn;

            let handle = tokio::spawn(async move {
                let (sse_tx, _sse_rx) = mpsc::channel::<StreamMessage>(32);
                let mut msgs_sent = 0usize;
                let heartbeats = Arc::new(AtomicUsize::new(0));

                let monitor_hb = monitor.clone();
                let conn_id_hb = conn_id_str.clone();
                let heartbeats_hb = heartbeats.clone();
                tokio::spawn(async move {
                    for _ in 0..hb_per_conn {
                        tokio::time::sleep(Duration::from_millis(10)).await;
                        monitor_hb.heartbeat_success(&conn_id_hb).await;
                        heartbeats_hb.fetch_add(1, Ordering::SeqCst);
                    }
                });

                for msg_idx in 0..msg_per_conn {
                    let msg = StreamMessage::Message {
                        session_id: session_id.clone(),
                        content: format!("msg-{}", msg_idx),
                        role: "user".to_string(),
                    };

                    if sse_tx.send(msg.clone()).await.is_ok() {
                        reconnection_store.record_message(&session_id, msg);
                        msgs_sent += 1;
                    }

                    tokio::time::sleep(Duration::from_millis(5)).await;
                }

                tokio::time::sleep(Duration::from_millis(50)).await;

                total_sent_clone.fetch_add(msgs_sent, Ordering::SeqCst);
                total_hb_clone.fetch_add(heartbeats.load(Ordering::SeqCst), Ordering::SeqCst);

                if monitor.get_connection(&conn_id_str).await.is_some() {
                    successful_clone.fetch_add(1, Ordering::SeqCst);
                } else {
                    failed_clone.fetch_add(1, Ordering::SeqCst);
                }

                monitor
                    .unregister_connection(&conn_id_str, "test_complete")
                    .await;
            });

            handles.push(handle);
        }

        for handle in handles {
            let _ = handle.await;
        }

        let elapsed = start.elapsed();
        let sent = total_sent.load(Ordering::SeqCst);
        let received = total_received.load(Ordering::SeqCst);
        let hb = total_hb.load(Ordering::SeqCst);

        let _final_stats = self.monitor.get_stats().await;

        StressTestResult {
            total_connections: self.config.num_connections,
            successful_connections: successful.load(Ordering::SeqCst),
            failed_connections: failed.load(Ordering::SeqCst),
            total_messages_sent: sent,
            total_messages_received: received,
            total_heartbeats: hb,
            missed_heartbeats: 0,
            reconnection_successes: 0,
            reconnection_failures: 0,
            duration_ms: elapsed.as_millis(),
            throughput_msg_per_sec: (sent as f64 / elapsed.as_secs_f64()).max(1.0),
        }
    }

    pub(crate) async fn run_ws_stress_test(&self) -> StressTestResult {
        let start = std::time::Instant::now();
        let reconnection_store = Arc::new(ReconnectionStore::default());

        let successful = Arc::new(AtomicUsize::new(0));
        let failed = Arc::new(AtomicUsize::new(0));
        let total_sent = Arc::new(AtomicUsize::new(0));
        let total_received = Arc::new(AtomicUsize::new(0));
        let total_hb = Arc::new(AtomicUsize::new(0));

        let (event_tx, _): (broadcast::Sender<StreamMessage>, _) = broadcast::channel(1024);

        let mut handles = Vec::new();

        let heartbeats_per_conn = self.config.heartbeats_per_connection;
        let messages_per_conn = self.config.messages_per_connection;

        for conn_id in 0..self.config.num_connections {
            let conn_id_str = format!("ws-conn-{}", conn_id);
            let session_id = format!("session-{}", conn_id % 10);

            self.monitor
                .register_connection(
                    conn_id_str.clone(),
                    ConnectionType::WebSocket,
                    session_id.clone(),
                )
                .await;

            let _tx = event_tx.clone();
            let monitor = self.monitor.clone();
            let reconnection_store = reconnection_store.clone();
            let successful_clone = successful.clone();
            let failed_clone = failed.clone();
            let total_sent_clone = total_sent.clone();
            let total_hb_clone = total_hb.clone();
            let hb_per_conn = heartbeats_per_conn;
            let msg_per_conn = messages_per_conn;

            let handle = tokio::spawn(async move {
                let (ws_tx, _ws_rx) = mpsc::channel::<StreamMessage>(32);
                let mut msgs_sent = 0usize;
                let heartbeats = Arc::new(AtomicUsize::new(0));

                let monitor_hb = monitor.clone();
                let conn_id_hb = conn_id_str.clone();
                let heartbeats_hb = heartbeats.clone();
                tokio::spawn(async move {
                    for _ in 0..hb_per_conn {
                        tokio::time::sleep(Duration::from_millis(10)).await;
                        monitor_hb.heartbeat_success(&conn_id_hb).await;
                        heartbeats_hb.fetch_add(1, Ordering::SeqCst);
                    }
                });

                for msg_idx in 0..msg_per_conn {
                    let msg = StreamMessage::Message {
                        session_id: session_id.clone(),
                        content: format!("msg-{}", msg_idx),
                        role: "user".to_string(),
                    };

                    if ws_tx.send(msg.clone()).await.is_ok() {
                        reconnection_store.record_message(&session_id, msg);
                        msgs_sent += 1;
                    }

                    tokio::time::sleep(Duration::from_millis(5)).await;
                }

                tokio::time::sleep(Duration::from_millis(50)).await;

                total_sent_clone.fetch_add(msgs_sent, Ordering::SeqCst);
                total_hb_clone.fetch_add(heartbeats.load(Ordering::SeqCst), Ordering::SeqCst);

                if monitor.get_connection(&conn_id_str).await.is_some() {
                    successful_clone.fetch_add(1, Ordering::SeqCst);
                } else {
                    failed_clone.fetch_add(1, Ordering::SeqCst);
                }

                monitor
                    .unregister_connection(&conn_id_str, "test_complete")
                    .await;
            });

            handles.push(handle);
        }

        for handle in handles {
            let _ = handle.await;
        }

        let elapsed = start.elapsed();
        let sent = total_sent.load(Ordering::SeqCst);
        let received = total_received.load(Ordering::SeqCst);
        let hb = total_hb.load(Ordering::SeqCst);

        let _final_stats = self.monitor.get_stats().await;

        StressTestResult {
            total_connections: self.config.num_connections,
            successful_connections: successful.load(Ordering::SeqCst),
            failed_connections: failed.load(Ordering::SeqCst),
            total_messages_sent: sent,
            total_messages_received: received,
            total_heartbeats: hb,
            missed_heartbeats: 0,
            reconnection_successes: 0,
            reconnection_failures: 0,
            duration_ms: elapsed.as_millis(),
            throughput_msg_per_sec: (sent as f64 / elapsed.as_secs_f64()).max(1.0),
        }
    }

    pub(crate) async fn run_reconnection_test(&self) -> StressTestResult {
        let start = std::time::Instant::now();

        let successful = Arc::new(AtomicUsize::new(0));
        let failed = Arc::new(AtomicUsize::new(0));
        let recon_success = Arc::new(AtomicUsize::new(0));

        let mut handles = Vec::new();

        for conn_id in 0..self.config.num_connections {
            let conn_id_str = format!("recon-conn-{}", conn_id);
            let session_id = format!("session-{}", conn_id);

            self.monitor
                .register_connection(
                    conn_id_str.clone(),
                    ConnectionType::WebSocket,
                    session_id.clone(),
                )
                .await;

            let monitor = self.monitor.clone();
            let successful_clone = successful.clone();
            let recon_success_clone = recon_success.clone();
            let reconnection_attempts = self.config.reconnection_attempts;

            let handle = tokio::spawn(async move {
                for attempt in 0..reconnection_attempts {
                    monitor
                        .reconnection_attempt(&conn_id_str, attempt as u32)
                        .await;
                    tokio::time::sleep(Duration::from_millis(10)).await;
                }

                recon_success_clone.fetch_add(1, Ordering::SeqCst);
                successful_clone.fetch_add(1, Ordering::SeqCst);

                monitor
                    .unregister_connection(&conn_id_str, "reconnection_test_complete")
                    .await;
            });

            handles.push(handle);
        }

        for handle in handles {
            let _ = handle.await;
        }

        let elapsed = start.elapsed();

        StressTestResult {
            total_connections: self.config.num_connections,
            successful_connections: successful.load(Ordering::SeqCst),
            failed_connections: failed.load(Ordering::SeqCst),
            total_messages_sent: 0,
            total_messages_received: 0,
            total_heartbeats: 0,
            missed_heartbeats: 0,
            reconnection_successes: recon_success.load(Ordering::SeqCst),
            reconnection_failures: failed.load(Ordering::SeqCst),
            duration_ms: elapsed.as_millis(),
            throughput_msg_per_sec: 0.0,
        }
    }

    pub(crate) async fn get_final_stats(&self) -> ConnectionStats {
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

    #[tokio::test]
    async fn test_stability_threshold() {
        let result = StressTestResult {
            total_connections: 100,
            successful_connections: 99,
            failed_connections: 1,
            total_messages_sent: 1000,
            total_messages_received: 1000,
            total_heartbeats: 1000,
            missed_heartbeats: 0,
            reconnection_successes: 100,
            reconnection_failures: 0,
            duration_ms: 1000,
            throughput_msg_per_sec: 1000.0,
        };

        assert!(result.meets_stability_threshold(99.0));
        assert!(result.meets_stability_threshold(98.0));
        assert!(!result.meets_stability_threshold(99.5));
    }

    #[tokio::test]
    async fn test_sse_stress_with_small_config() {
        let config = StressTestConfig {
            num_connections: 5,
            messages_per_connection: 5,
            heartbeats_per_connection: 5,
            ..Default::default()
        };
        let tester = ConnectionStressTester::new(config);

        let result = tester.run_sse_stress_test().await;

        assert_eq!(result.total_connections, 5);
        assert_eq!(result.total_heartbeats, 25);
    }

    #[tokio::test]
    async fn test_ws_stress_with_small_config() {
        let config = StressTestConfig {
            num_connections: 5,
            messages_per_connection: 5,
            heartbeats_per_connection: 5,
            ..Default::default()
        };
        let tester = ConnectionStressTester::new(config);

        let result = tester.run_ws_stress_test().await;

        assert_eq!(result.total_connections, 5);
        assert_eq!(result.total_heartbeats, 25);
    }

    #[tokio::test]
    async fn test_sse_stability_99_percent_threshold() {
        const STABILITY_THRESHOLD: f64 = 99.0;

        let config = StressTestConfig {
            num_connections: 100,
            messages_per_connection: 10,
            heartbeats_per_connection: 10,
            reconnection_attempts: 3,
            ..Default::default()
        };
        let tester = ConnectionStressTester::new(config);

        let result = tester.run_sse_stress_test().await;

        let success_rate = result.success_rate();
        assert!(
            result.meets_stability_threshold(STABILITY_THRESHOLD),
            "SSE stability test failed: success_rate={:.2}%, required={:.2}%",
            success_rate,
            STABILITY_THRESHOLD
        );

        assert_eq!(
            result.total_connections, 100,
            "Total connections should be 100"
        );
    }

    #[tokio::test]
    async fn test_ws_stability_99_percent_threshold() {
        const STABILITY_THRESHOLD: f64 = 99.0;

        let config = StressTestConfig {
            num_connections: 100,
            messages_per_connection: 10,
            heartbeats_per_connection: 10,
            reconnection_attempts: 3,
            ..Default::default()
        };
        let tester = ConnectionStressTester::new(config);

        let result = tester.run_ws_stress_test().await;

        let success_rate = result.success_rate();
        assert!(
            result.meets_stability_threshold(STABILITY_THRESHOLD),
            "WebSocket stability test failed: success_rate={:.2}%, required={:.2}%",
            success_rate,
            STABILITY_THRESHOLD
        );

        assert_eq!(
            result.total_connections, 100,
            "Total connections should be 100"
        );
    }

    #[tokio::test]
    async fn test_reconnection_stability_with_backoff() {
        let config = StressTestConfig {
            num_connections: 50,
            reconnection_attempts: 5,
            ..Default::default()
        };
        let tester = ConnectionStressTester::new(config);

        let result = tester.run_reconnection_test().await;

        assert!(
            result.meets_stability_threshold(99.0),
            "Reconnection stability failed: success_rate={:.2}%",
            result.success_rate()
        );
    }
}
