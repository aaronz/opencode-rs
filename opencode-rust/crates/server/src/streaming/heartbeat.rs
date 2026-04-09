use std::time::Duration;

use chrono::Utc;
use tokio::sync::mpsc;
use tokio::task::JoinHandle;

use super::StreamMessage;

const DEFAULT_HEARTBEAT_INTERVAL_SECS: u64 = 30;

#[derive(Debug, Clone)]
pub struct HeartbeatManager {
    interval: Duration,
}

impl HeartbeatManager {
    pub fn new(interval: Duration) -> Self {
        Self { interval }
    }

    pub fn interval(&self) -> Duration {
        self.interval
    }

    pub fn spawn(&self, tx: mpsc::Sender<StreamMessage>) -> JoinHandle<()> {
        let interval = self.interval;
        tokio::spawn(async move {
            let mut ticker = tokio::time::interval(interval);
            loop {
                ticker.tick().await;
                let heartbeat = StreamMessage::Heartbeat {
                    timestamp: Utc::now().timestamp(),
                };
                if tx.send(heartbeat).await.is_err() {
                    break;
                }
            }
        })
    }
}

impl Default for HeartbeatManager {
    fn default() -> Self {
        Self {
            interval: Duration::from_secs(DEFAULT_HEARTBEAT_INTERVAL_SECS),
        }
    }
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use tokio::sync::mpsc;

    use crate::streaming::{heartbeat::HeartbeatManager, StreamMessage};

    #[tokio::test]
    async fn heartbeat_interval_works() {
        let manager = HeartbeatManager::new(Duration::from_millis(40));
        let (tx, mut rx) = mpsc::channel(8);
        let handle = manager.spawn(tx);

        let first = tokio::time::timeout(Duration::from_millis(150), rx.recv())
            .await
            .expect("first heartbeat should arrive")
            .expect("channel should be open");
        let second = tokio::time::timeout(Duration::from_millis(150), rx.recv())
            .await
            .expect("second heartbeat should arrive")
            .expect("channel should be open");

        assert!(matches!(first, StreamMessage::Heartbeat { .. }));
        assert!(matches!(second, StreamMessage::Heartbeat { .. }));

        drop(rx);
        handle.abort();
    }
}
