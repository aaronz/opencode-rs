use serde::{Deserialize, Serialize};
use tokio::sync::broadcast;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Event {
    SessionCreated(String),
    SessionUpdated(String),
    SessionCompacted(String),
    MessageUpdated(String),
    ToolExecuteBefore(String),
    ToolExecuteAfter(String),
    PermissionAsked(String),
    PermissionResolved(String),
    FileEdited(String),
    LspUpdated(String),
    ShellEnv(String),
    ToastShow(String),
}

pub struct EventBus {
    tx: broadcast::Sender<Event>,
}

impl EventBus {
    pub fn new() -> Self {
        let (tx, _) = broadcast::channel(100);
        Self { tx }
    }

    pub fn subscribe(&self) -> broadcast::Receiver<Event> {
        self.tx.subscribe()
    }

    pub fn publish(&self, event: Event) {
        let _ = self.tx.send(event);
    }
}

impl Default for EventBus {
    fn default() -> Self {
        Self::new()
    }
}
