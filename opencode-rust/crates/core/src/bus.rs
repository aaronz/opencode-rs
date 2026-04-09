use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::broadcast;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum InternalEvent {
    SessionStarted(String),
    SessionEnded(String),
    SessionForked {
        original_id: String,
        new_id: String,
        fork_point: usize,
    },
    SessionShared {
        session_id: String,
        share_url: String,
    },
    MessageAdded {
        session_id: String,
        message_id: String,
    },
    MessageUpdated {
        session_id: String,
        message_id: String,
    },
    ToolCallStarted {
        session_id: String,
        tool_name: String,
        call_id: String,
    },
    ToolCallEnded {
        session_id: String,
        call_id: String,
        success: bool,
    },
    ToolCallOutput {
        session_id: String,
        call_id: String,
        output: String,
    },
    AgentStatusChanged {
        session_id: String,
        status: String,
    },
    AgentStarted {
        session_id: String,
        agent: String,
    },
    AgentStopped {
        session_id: String,
        agent: String,
    },
    ProviderChanged {
        provider: String,
        model: String,
    },
    ModelChanged {
        model: String,
    },
    CompactionTriggered {
        session_id: String,
        pruned_count: usize,
    },
    CompactionCompleted {
        session_id: String,
        summary_inserted: bool,
    },
    ConfigUpdated,
    AuthChanged {
        user_id: Option<String>,
    },
    PermissionGranted {
        user_id: String,
        permission: String,
    },
    PermissionDenied {
        user_id: String,
        permission: String,
    },
    FileWatchEvent {
        path: String,
        kind: String,
    },
    LspDiagnosticsUpdated {
        path: String,
        count: usize,
    },
    ShellEnvChanged {
        key: String,
        value: String,
    },
    UiToastShow {
        message: String,
        level: String,
    },
    PermissionAsked {
        session_id: String,
        request_id: String,
        permission: String,
    },
    PermissionReplied {
        session_id: String,
        request_id: String,
        granted: bool,
    },
    PluginLoaded {
        name: String,
    },
    PluginUnloaded {
        name: String,
    },
    McpServerConnected {
        name: String,
    },
    McpServerDisconnected {
        name: String,
    },
    AcpEventReceived {
        agent_id: String,
        event_type: String,
    },
    ServerStarted {
        port: u16,
    },
    ServerStopped,
    Error {
        source: String,
        message: String,
    },
}

impl InternalEvent {
    pub fn session_id(&self) -> Option<&str> {
        match self {
            Self::SessionStarted(id) | Self::SessionEnded(id) => Some(id),
            Self::SessionForked { original_id, .. } => Some(original_id),
            Self::SessionShared { session_id, .. } => Some(session_id),
            Self::MessageAdded { session_id, .. } => Some(session_id),
            Self::MessageUpdated { session_id, .. } => Some(session_id),
            Self::ToolCallStarted { session_id, .. } => Some(session_id),
            Self::ToolCallEnded { session_id, .. } => Some(session_id),
            Self::ToolCallOutput { session_id, .. } => Some(session_id),
            Self::AgentStatusChanged { session_id, .. } => Some(session_id),
            Self::AgentStarted { session_id, .. } => Some(session_id),
            Self::AgentStopped { session_id, .. } => Some(session_id),
            Self::CompactionTriggered { session_id, .. } => Some(session_id),
            Self::CompactionCompleted { session_id, .. } => Some(session_id),
            _ => None,
        }
    }
}

pub struct EventBus {
    tx: broadcast::Sender<InternalEvent>,
}

impl EventBus {
    pub fn new() -> Self {
        let (tx, _) = broadcast::channel(256);
        Self { tx }
    }

    pub fn subscribe(&self) -> broadcast::Receiver<InternalEvent> {
        self.tx.subscribe()
    }

    pub fn publish(&self, event: InternalEvent) {
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

pub type SharedEventBus = Arc<EventBus>;

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
