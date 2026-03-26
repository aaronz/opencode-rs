use std::any::{Any, TypeId};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::broadcast;

pub trait Event: Send + Sync + 'static {
    fn event_type(&self) -> &'static str;
}

pub struct EventBus {
    senders: Arc<tokio::sync::RwLock<HashMap<TypeId, Box<dyn Any + Send + Sync>>>>,
}

impl EventBus {
    pub fn new() -> Self {
        Self {
            senders: Arc::new(tokio::sync::RwLock::new(HashMap::new())),
        }
    }

    pub async fn subscribe<E: Event + Clone>(&self) -> broadcast::Receiver<E> {
        let type_id = TypeId::of::<E>();
        let mut senders = self.senders.write().await;

        if let Some(sender) = senders.get(&type_id) {
            if let Some(sender) = sender.downcast_ref::<broadcast::Sender<E>>() {
                return sender.subscribe();
            }
        }

        let (tx, rx) = broadcast::channel(100);
        senders.insert(type_id, Box::new(tx));
        rx
    }

    pub async fn publish<E: Event + Clone>(&self, event: E) {
        let type_id = TypeId::of::<E>();
        let senders = self.senders.read().await;

        if let Some(sender) = senders.get(&type_id) {
            if let Some(sender) = sender.downcast_ref::<broadcast::Sender<E>>() {
                let _ = sender.send(event);
            }
        }
    }
}

impl Default for EventBus {
    fn default() -> Self {
        Self::new()
    }
}

impl Clone for EventBus {
    fn clone(&self) -> Self {
        Self {
            senders: Arc::clone(&self.senders),
        }
    }
}
