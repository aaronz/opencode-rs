mod types;

pub use types::{EventBus, InternalEvent, SharedEventBus};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_event_bus_new() {
        let bus = EventBus::new();
        assert_eq!(bus.subscriber_count(), 0);
    }

    #[test]
    fn test_event_bus_publish_and_subscribe() {
        let bus = EventBus::new();
        let mut rx = bus.subscribe();
        assert_eq!(bus.subscriber_count(), 1);

        bus.publish(InternalEvent::SessionStarted("test-session".to_string()));

        let event = rx.try_recv();
        assert!(event.is_ok());
        match event.unwrap() {
            InternalEvent::SessionStarted(id) => assert_eq!(id, "test-session"),
            _ => panic!("Expected SessionStarted event"),
        }
    }

    #[test]
    fn test_event_bus_multiple_subscribers() {
        let bus = EventBus::new();
        let mut rx1 = bus.subscribe();
        let mut rx2 = bus.subscribe();
        assert_eq!(bus.subscriber_count(), 2);

        bus.publish(InternalEvent::ConfigUpdated);

        let event1 = rx1.try_recv();
        let event2 = rx2.try_recv();
        assert!(event1.is_ok());
        assert!(event2.is_ok());
    }

    #[test]
    fn test_internal_event_session_id() {
        let event = InternalEvent::SessionStarted("session-123".to_string());
        assert_eq!(event.session_id(), Some("session-123"));

        let event = InternalEvent::SessionForked {
            original_id: "orig".to_string(),
            new_id: "new".to_string(),
            fork_point: 5,
        };
        assert_eq!(event.session_id(), Some("orig"));

        let event = InternalEvent::ConfigUpdated;
        assert_eq!(event.session_id(), None);
    }

    #[tokio::test]
    async fn test_event_bus_async_subscribe() {
        let bus = EventBus::new();
        let mut rx = bus.subscribe();

        bus.publish(InternalEvent::ServerStarted { port: 8080 });

        let event = rx.recv().await;
        assert!(event.is_ok());
        match event.unwrap() {
            InternalEvent::ServerStarted { port } => assert_eq!(port, 8080),
            _ => panic!("Expected ServerStarted event"),
        }
    }

    #[test]
    fn test_event_bus_late_subscriber() {
        let bus = EventBus::new();

        bus.publish(InternalEvent::SessionEnded("session-1".to_string()));

        let mut rx = bus.subscribe();
        bus.publish(InternalEvent::SessionEnded("session-2".to_string()));

        let event = rx.try_recv();
        assert!(event.is_ok());
        match event.unwrap() {
            InternalEvent::SessionEnded(id) => assert_eq!(id, "session-2"),
            _ => panic!("Expected SessionEnded event"),
        }
    }
}