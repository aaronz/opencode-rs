//! Domain events for the OpenCode event bus.
//!
//! `DomainEvent` is the single source of truth for all event-driven communication
//! in OpenCode. All other event types (`RuntimeFacadeEvent`, `StreamMessage`, etc.) are
//! projections derived from `DomainEvent`.
//!
//! ## Event categories
//!
//! - **Session lifecycle**: SessionStarted, SessionEnded, SessionForked, SessionShared
//! - **Message lifecycle**: MessageAdded, MessageUpdated
//! - **Tool lifecycle**: ToolCallStarted, ToolCallEnded, ToolCallOutput
//! - **Agent lifecycle**: AgentStarted, AgentStopped, AgentStatusChanged
//! - **LLM lifecycle**: LlmRequestStarted, LlmTokenStreamed, LlmResponseCompleted, LlmError
//! - **Task lifecycle**: TaskStarted, TaskProgress, TaskCompleted, TaskFailed, TaskCancelled
//! - **Permission**: PermissionAsked, PermissionReplied, PermissionGranted, PermissionDenied
//! - **Infrastructure**: ConfigUpdated, AuthChanged, ProviderChanged, ModelChanged
//! - **Integration**: Plugin*, MCP*, ACP*, FileWatch, LSP, ShellEnv, Server*
//!
//! ## Projections
//!
//! Projections to downstream event types are defined in the `server` crate
//! at `server/streaming/projections.rs` since [`StreamMessage`] lives there.
//!
//! [`DomainEvent`]: crate::events::DomainEvent

use serde::{Deserialize, Serialize};

/// Domain event — the single source of truth for all events in OpenCode.
///
/// This enum represents every meaningful state change in the system.
/// Subscribers should project from this type into their own event format.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DomainEvent {
    // -------------------------------------------------------------------------
    // Session lifecycle
    // -------------------------------------------------------------------------
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

    // -------------------------------------------------------------------------
    // Message lifecycle
    // -------------------------------------------------------------------------
    MessageAdded {
        session_id: String,
        message_id: String,
    },
    MessageUpdated {
        session_id: String,
        message_id: String,
    },

    // -------------------------------------------------------------------------
    // Tool lifecycle
    // -------------------------------------------------------------------------
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

    // -------------------------------------------------------------------------
    // Agent lifecycle
    // -------------------------------------------------------------------------
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

    // -------------------------------------------------------------------------
    // Provider / model
    // -------------------------------------------------------------------------
    ProviderChanged {
        provider: String,
        model: String,
    },
    ModelChanged {
        model: String,
    },

    // -------------------------------------------------------------------------
    // Compaction (context truncation)
    // -------------------------------------------------------------------------
    CompactionTriggered {
        session_id: String,
        pruned_count: usize,
    },
    CompactionCompleted {
        session_id: String,
        summary_inserted: bool,
    },

    // -------------------------------------------------------------------------
    // Config / auth
    // -------------------------------------------------------------------------
    ConfigUpdated,
    AuthChanged {
        user_id: Option<String>,
    },

    // -------------------------------------------------------------------------
    // Permission
    // -------------------------------------------------------------------------
    PermissionGranted {
        user_id: String,
        permission: String,
    },
    PermissionDenied {
        user_id: String,
        permission: String,
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

    // -------------------------------------------------------------------------
    // IDE integration
    // -------------------------------------------------------------------------
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

    // -------------------------------------------------------------------------
    // UI
    // -------------------------------------------------------------------------
    UiToastShow {
        message: String,
        level: String,
    },

    // -------------------------------------------------------------------------
    // Plugin lifecycle
    // -------------------------------------------------------------------------
    PluginLoaded {
        name: String,
    },
    PluginUnloaded {
        name: String,
    },

    // -------------------------------------------------------------------------
    // MCP (Model Context Protocol)
    // -------------------------------------------------------------------------
    McpServerConnected {
        name: String,
    },
    McpServerDisconnected {
        name: String,
    },

    // -------------------------------------------------------------------------
    // ACP (Agent Communication Protocol)
    // -------------------------------------------------------------------------
    AcpEventReceived {
        agent_id: String,
        event_type: String,
    },
    AcpConnected {
        server_id: String,
        capabilities: Vec<String>,
    },
    AcpDisconnected,

    // -------------------------------------------------------------------------
    // Server lifecycle
    // -------------------------------------------------------------------------
    ServerStarted {
        port: u16,
    },
    ServerStopped,

    // -------------------------------------------------------------------------
    // Generic error
    // -------------------------------------------------------------------------
    Error {
        source: String,
        message: String,
    },

    // -------------------------------------------------------------------------
    // Task lifecycle
    // -------------------------------------------------------------------------
    TaskStarted {
        session_id: String,
        turn_id: String,
        task_id: String,
        task_kind: String,
    },
    TaskProgress {
        session_id: String,
        turn_id: String,
        task_id: String,
        message: String,
    },
    TaskCompleted {
        session_id: String,
        turn_id: String,
        task_id: String,
    },
    TaskFailed {
        session_id: String,
        turn_id: String,
        task_id: String,
        error: String,
    },
    TaskCancelled {
        session_id: String,
        turn_id: String,
        task_id: String,
    },

    // -------------------------------------------------------------------------
    // LLM lifecycle
    // -------------------------------------------------------------------------
    LlmRequestStarted {
        session_id: String,
        provider: String,
        model: String,
    },
    LlmTokenStreamed {
        session_id: String,
        delta: String,
    },
    LlmResponseCompleted {
        session_id: String,
        total_tokens: Option<u64>,
    },
    /// LLM error — distinct from the generic [`DomainEvent::Error`] variant.
    LlmError {
        session_id: String,
        error: String,
    },
}

impl DomainEvent {
    /// Extract the `session_id` from this event, if present.
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
