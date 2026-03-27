use std::sync::Arc;
use tokio::sync::broadcast;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum InternalEvent {
    SessionStarted(String),
    SessionEnded(String),
    MessageAdded(String),
    ToolCallStarted(String),
    ToolCallEnded(String, bool),
    AgentStatusChanged(String, String),
    ConfigUpdated,
}

pub struct EventBus {
    tx: broadcast::Sender<InternalEvent>,
}

impl EventBus {
    pub fn new() -> Self {
        let (tx, _) = broadcast::channel(100);
        Self { tx }
    }

    pub fn subscribe(&self) -> broadcast::Receiver<InternalEvent> {
        self.tx.subscribe()
    }

    pub fn publish(&self, event: InternalEvent) {
        let _ = self.tx.send(event);
    }
}

impl Default for EventBus {
    fn default() -> Self {
        Self::new()
    }
}
