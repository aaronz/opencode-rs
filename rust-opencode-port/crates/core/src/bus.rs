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
