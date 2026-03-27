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

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Debug, Clone)]
    struct TestEvent {
        message: String,
    }

    impl Event for TestEvent {
        fn event_type(&self) -> &'static str {
            "TestEvent"
        }
    }

    #[tokio::test]
    async fn test_event_bus_subscribe() {
        let bus = EventBus::new();
        let mut rx = bus.subscribe::<TestEvent>().await;
        
        bus.publish(TestEvent { message: "test".to_string() }).await;
        
        let event = rx.recv().await.unwrap();
        assert_eq!(event.message, "test");
    }

    #[tokio::test]
    async fn test_event_bus_publish_multiple() {
        let bus = EventBus::new();
        let mut rx1 = bus.subscribe::<TestEvent>().await;
        let mut rx2 = bus.subscribe::<TestEvent>().await;
        
        bus.publish(TestEvent { message: "hello".to_string() }).await;
        
        let event1 = rx1.recv().await.unwrap();
        let event2 = rx2.recv().await.unwrap();
        
        assert_eq!(event1.message, "hello");
        assert_eq!(event2.message, "hello");
    }

    #[tokio::test]
    async fn test_event_bus_no_subscribers() {
        let bus = EventBus::new();
        
        bus.publish(TestEvent { message: "no subscriber".to_string() }).await;
    }
}
