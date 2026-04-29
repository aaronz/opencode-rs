//! Event bus types.

use std::sync::Arc;

pub type SharedEventBus = Arc<EventBus>;

pub struct EventBus {
    tx: tokio::sync::broadcast::Sender<crate::events::DomainEvent>,
}

impl EventBus {
    pub fn new() -> Self {
        let (tx, _) = tokio::sync::broadcast::channel(256);
        Self { tx }
    }

    pub fn subscribe(&self) -> tokio::sync::broadcast::Receiver<crate::events::DomainEvent> {
        self.tx.subscribe()
    }

    pub fn publish(&self, event: crate::events::DomainEvent) {
        let _ = self.tx.send(event);
    }

    pub fn subscriber_count(&self) -> usize {
        self.tx.receiver_count()
    }
}

impl Default for EventBus {
    fn default() -> Self {
        Self::new()
    }
}
