use opencode_core::events::DomainEvent;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum RuntimeFacadeEvent {
    SessionStarted {
        session_id: String,
    },
    SessionEnded {
        session_id: String,
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
    Error {
        source: String,
        message: String,
    },
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
    LlmError {
        session_id: String,
        error: String,
    },
}

impl RuntimeFacadeEvent {
    /// Extract the `session_id` from this event, if present.
    pub fn session_id(&self) -> Option<&str> {
        match self {
            Self::SessionStarted { session_id }
            | Self::SessionEnded { session_id }
            | Self::MessageAdded { session_id, .. }
            | Self::MessageUpdated { session_id, .. }
            | Self::ToolCallStarted { session_id, .. }
            | Self::ToolCallEnded { session_id, .. }
            | Self::ToolCallOutput { session_id, .. }
            | Self::AgentStatusChanged { session_id, .. }
            | Self::TaskStarted { session_id, .. }
            | Self::TaskProgress { session_id, .. }
            | Self::TaskCompleted { session_id, .. }
            | Self::TaskFailed { session_id, .. }
            | Self::TaskCancelled { session_id, .. }
            | Self::LlmRequestStarted { session_id, .. }
            | Self::LlmTokenStreamed { session_id, .. }
            | Self::LlmResponseCompleted { session_id, .. }
            | Self::LlmError { session_id, .. } => Some(session_id),
            Self::Error { .. } => None,
        }
    }

    pub fn from_domain_event(event: &DomainEvent) -> Option<Self> {
        match event {
            DomainEvent::SessionStarted(session_id) => Some(Self::SessionStarted {
                session_id: session_id.clone(),
            }),
            DomainEvent::SessionEnded(session_id) => Some(Self::SessionEnded {
                session_id: session_id.clone(),
            }),
            DomainEvent::MessageAdded {
                session_id,
                message_id,
            } => Some(Self::MessageAdded {
                session_id: session_id.clone(),
                message_id: message_id.clone(),
            }),
            DomainEvent::MessageUpdated {
                session_id,
                message_id,
            } => Some(Self::MessageUpdated {
                session_id: session_id.clone(),
                message_id: message_id.clone(),
            }),
            DomainEvent::ToolCallStarted {
                session_id,
                tool_name,
                call_id,
            } => Some(Self::ToolCallStarted {
                session_id: session_id.clone(),
                tool_name: tool_name.clone(),
                call_id: call_id.clone(),
            }),
            DomainEvent::ToolCallEnded {
                session_id,
                call_id,
                success,
            } => Some(Self::ToolCallEnded {
                session_id: session_id.clone(),
                call_id: call_id.clone(),
                success: *success,
            }),
            DomainEvent::ToolCallOutput {
                session_id,
                call_id,
                output,
            } => Some(Self::ToolCallOutput {
                session_id: session_id.clone(),
                call_id: call_id.clone(),
                output: output.clone(),
            }),
            DomainEvent::AgentStatusChanged { session_id, status } => {
                Some(Self::AgentStatusChanged {
                    session_id: session_id.clone(),
                    status: status.clone(),
                })
            }
            DomainEvent::Error { source, message } => Some(Self::Error {
                source: source.clone(),
                message: message.clone(),
            }),
            DomainEvent::TaskStarted {
                session_id,
                turn_id,
                task_id,
                task_kind,
            } => Some(Self::TaskStarted {
                session_id: session_id.clone(),
                turn_id: turn_id.clone(),
                task_id: task_id.clone(),
                task_kind: task_kind.clone(),
            }),
            DomainEvent::TaskProgress {
                session_id,
                turn_id,
                task_id,
                message,
            } => Some(Self::TaskProgress {
                session_id: session_id.clone(),
                turn_id: turn_id.clone(),
                task_id: task_id.clone(),
                message: message.clone(),
            }),
            DomainEvent::TaskCompleted {
                session_id,
                turn_id,
                task_id,
            } => Some(Self::TaskCompleted {
                session_id: session_id.clone(),
                turn_id: turn_id.clone(),
                task_id: task_id.clone(),
            }),
            DomainEvent::TaskFailed {
                session_id,
                turn_id,
                task_id,
                error,
            } => Some(Self::TaskFailed {
                session_id: session_id.clone(),
                turn_id: turn_id.clone(),
                task_id: task_id.clone(),
                error: error.clone(),
            }),
            DomainEvent::TaskCancelled {
                session_id,
                turn_id,
                task_id,
            } => Some(Self::TaskCancelled {
                session_id: session_id.clone(),
                turn_id: turn_id.clone(),
                task_id: task_id.clone(),
            }),
            DomainEvent::LlmRequestStarted {
                session_id,
                provider,
                model,
            } => Some(Self::LlmRequestStarted {
                session_id: session_id.clone(),
                provider: provider.clone(),
                model: model.clone(),
            }),
            DomainEvent::LlmTokenStreamed { session_id, delta } => Some(Self::LlmTokenStreamed {
                session_id: session_id.clone(),
                delta: delta.clone(),
            }),
            DomainEvent::LlmResponseCompleted {
                session_id,
                total_tokens,
            } => Some(Self::LlmResponseCompleted {
                session_id: session_id.clone(),
                total_tokens: *total_tokens,
            }),
            DomainEvent::LlmError {
                session_id,
                error,
            } => Some(Self::LlmError {
                session_id: session_id.clone(),
                error: error.clone(),
            }),
            _ => None,
        }
    }

    #[deprecated(since = "0.1.0", note = "use from_domain_event instead")]
    pub fn from_internal_event(event: &DomainEvent) -> Option<Self> {
        Self::from_domain_event(event)
    }
}
