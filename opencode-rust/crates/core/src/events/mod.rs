//! Domain events for the OpenCode event bus.
//!
//! `DomainEvent` is the single source of truth for all event-driven communication
//! in OpenCode. All other event types (`RuntimeFacadeEvent`, `StreamMessage`, etc.) are
//! projections derived from `DomainEvent`.
//!
//! ## Event categories
//!
//! - **Session lifecycle**: SessionStarted, SessionEnded, SessionForked, SessionShared
//! - **Turn lifecycle**: TurnStarted, TurnCompleted, TurnFailed
//! - **Message lifecycle**: MessageAdded, MessageUpdated, UserMessageReceived
//! - **Context lifecycle**: ContextBuildStarted, ContextBuilt
//! - **Tool lifecycle**: ToolCallStarted, ToolCallRequested, ToolPermissionRequested, ToolCallEnded, ToolCallOutput
//! - **File lifecycle**: FilePatchProposed, FilePatchApplied
//! - **Agent lifecycle**: AgentStarted, AgentStopped, AgentStatusChanged
//! - **LLM lifecycle**: LlmRequestStarted, LlmTokenStreamed, LlmResponseCompleted, LlmError
//! - **Task lifecycle**: TaskStarted, TaskProgress, TaskCompleted, TaskFailed, TaskCancelled, CancellationRequested
//! - **Validation lifecycle**: ValidationStarted, ValidationPassed, ValidationFailed
//! - **Hook lifecycle**: HookTriggered, HookFailed
//! - **Permission**: PermissionAsked, PermissionReplied, PermissionGranted, PermissionDenied
//! - **Infrastructure**: ConfigUpdated, AuthChanged, ProviderChanged, ModelChanged
//! - **Integration**: Plugin*, MCP*, ACP*, FileWatch, LSP, ShellEnv, Server*, SessionPersisted
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
    SessionPersisted {
        session_id: String,
    },

    // -------------------------------------------------------------------------
    // Turn lifecycle
    // -------------------------------------------------------------------------
    TurnStarted {
        session_id: String,
        turn_id: String,
    },
    TurnCompleted {
        session_id: String,
        turn_id: String,
    },
    TurnFailed {
        session_id: String,
        turn_id: String,
        error: String,
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
    UserMessageReceived {
        session_id: String,
        message_id: String,
    },

    // -------------------------------------------------------------------------
    // Context lifecycle
    // -------------------------------------------------------------------------
    ContextBuildStarted {
        session_id: String,
        turn_id: String,
    },
    ContextBuilt {
        context_id: String,
        session_id: String,
        turn_id: String,
        token_count: usize,
        item_count: usize,
        truncated: bool,
    },

    // -------------------------------------------------------------------------
    // Tool lifecycle
    // -------------------------------------------------------------------------
    ToolCallRequested {
        session_id: String,
        turn_id: String,
        call_id: String,
        tool_name: String,
    },
    ToolCallStarted {
        session_id: String,
        tool_name: String,
        call_id: String,
    },
    ToolPermissionRequested {
        session_id: String,
        call_id: String,
        tool_name: String,
        risk_level: String,
        reason: String,
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
    // File lifecycle
    // -------------------------------------------------------------------------
    FilePatchProposed {
        session_id: String,
        patch_id: String,
        path: String,
    },
    FilePatchApplied {
        session_id: String,
        patch_id: String,
        path: String,
    },
    FilePatchRejected {
        session_id: String,
        patch_id: String,
        path: String,
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
    // Runtime lifecycle
    // -------------------------------------------------------------------------
    RuntimeStatusChanged {
        session_id: Option<String>,
        from_status: String,
        to_status: String,
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
    CancellationRequested {
        session_id: String,
        turn_id: String,
        task_id: String,
    },

    // -------------------------------------------------------------------------
    // Validation lifecycle
    // -------------------------------------------------------------------------
    ValidationStarted {
        session_id: String,
        turn_id: String,
        task_id: String,
        command: String,
    },
    ValidationPassed {
        session_id: String,
        turn_id: String,
        task_id: String,
    },
    ValidationFailed {
        session_id: String,
        turn_id: String,
        task_id: String,
        reason: String,
    },

    // -------------------------------------------------------------------------
    // Hook lifecycle
    // -------------------------------------------------------------------------
    HookTriggered {
        session_id: String,
        turn_id: String,
        hook_id: String,
        hook_name: String,
    },
    HookFailed {
        session_id: String,
        turn_id: String,
        hook_id: String,
        hook_name: String,
        error: String,
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

    // -------------------------------------------------------------------------
    // Command / Shell execution
    // -------------------------------------------------------------------------
    CommandStarted {
        session_id: String,
        execution_id: String,
        command: String,
    },
    CommandOutputReceived {
        session_id: String,
        execution_id: String,
        stream: OutputStreamKind,
        chunk: String,
    },
    CommandCompleted {
        session_id: String,
        execution_id: String,
        exit_code: Option<i32>,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum OutputStreamKind {
    Stdout,
    Stderr,
}

impl DomainEvent {
    /// Extract the `session_id` from this event, if present.
    pub fn session_id(&self) -> Option<&str> {
        match self {
            Self::SessionStarted(id) | Self::SessionEnded(id) => Some(id),
            Self::SessionForked { original_id, .. } => Some(original_id),
            Self::SessionShared { session_id, .. } => Some(session_id),
            Self::SessionPersisted { session_id, .. } => Some(session_id),
            Self::TurnStarted { session_id, .. } => Some(session_id),
            Self::TurnCompleted { session_id, .. } => Some(session_id),
            Self::TurnFailed { session_id, .. } => Some(session_id),
            Self::MessageAdded { session_id, .. } => Some(session_id),
            Self::MessageUpdated { session_id, .. } => Some(session_id),
            Self::UserMessageReceived { session_id, .. } => Some(session_id),
            Self::ContextBuildStarted { session_id, .. } => Some(session_id),
            Self::ContextBuilt { session_id, .. } => Some(session_id),
            Self::ToolCallRequested { session_id, .. } => Some(session_id),
            Self::ToolCallStarted { session_id, .. } => Some(session_id),
            Self::ToolPermissionRequested { session_id, .. } => Some(session_id),
            Self::ToolCallEnded { session_id, .. } => Some(session_id),
            Self::ToolCallOutput { session_id, .. } => Some(session_id),
            Self::FilePatchProposed { session_id, .. } => Some(session_id),
            Self::FilePatchApplied { session_id, .. } => Some(session_id),
            Self::FilePatchRejected { session_id, .. } => Some(session_id),
            Self::AgentStatusChanged { session_id, .. } => Some(session_id),
            Self::AgentStarted { session_id, .. } => Some(session_id),
            Self::AgentStopped { session_id, .. } => Some(session_id),
            Self::CompactionTriggered { session_id, .. } => Some(session_id),
            Self::CompactionCompleted { session_id, .. } => Some(session_id),
            Self::CommandStarted { session_id, .. } => Some(session_id),
            Self::CommandOutputReceived { session_id, .. } => Some(session_id),
            Self::CommandCompleted { session_id, .. } => Some(session_id),
            Self::TaskStarted { session_id, .. } => Some(session_id),
            Self::TaskProgress { session_id, .. } => Some(session_id),
            Self::TaskCompleted { session_id, .. } => Some(session_id),
            Self::TaskFailed { session_id, .. } => Some(session_id),
            Self::TaskCancelled { session_id, .. } => Some(session_id),
            Self::CancellationRequested { session_id, .. } => Some(session_id),
            Self::ValidationStarted { session_id, .. } => Some(session_id),
            Self::ValidationPassed { session_id, .. } => Some(session_id),
            Self::ValidationFailed { session_id, .. } => Some(session_id),
            Self::HookTriggered { session_id, .. } => Some(session_id),
            Self::HookFailed { session_id, .. } => Some(session_id),
            Self::LlmRequestStarted { session_id, .. } => Some(session_id),
            Self::LlmTokenStreamed { session_id, .. } => Some(session_id),
            Self::LlmResponseCompleted { session_id, .. } => Some(session_id),
            Self::LlmError { session_id, .. } => Some(session_id),
            Self::RuntimeStatusChanged { session_id, .. } => session_id.as_deref(),
            _ => None,
        }
    }
}
