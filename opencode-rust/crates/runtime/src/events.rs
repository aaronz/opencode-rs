use opencode_core::bus::InternalEvent;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum RuntimeEvent {
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
}

impl RuntimeEvent {
    pub fn from_internal_event(event: &InternalEvent) -> Option<Self> {
        match event {
            InternalEvent::SessionStarted(session_id) => Some(Self::SessionStarted {
                session_id: session_id.clone(),
            }),
            InternalEvent::SessionEnded(session_id) => Some(Self::SessionEnded {
                session_id: session_id.clone(),
            }),
            InternalEvent::MessageAdded {
                session_id,
                message_id,
            } => Some(Self::MessageAdded {
                session_id: session_id.clone(),
                message_id: message_id.clone(),
            }),
            InternalEvent::MessageUpdated {
                session_id,
                message_id,
            } => Some(Self::MessageUpdated {
                session_id: session_id.clone(),
                message_id: message_id.clone(),
            }),
            InternalEvent::ToolCallStarted {
                session_id,
                tool_name,
                call_id,
            } => Some(Self::ToolCallStarted {
                session_id: session_id.clone(),
                tool_name: tool_name.clone(),
                call_id: call_id.clone(),
            }),
            InternalEvent::ToolCallEnded {
                session_id,
                call_id,
                success,
            } => Some(Self::ToolCallEnded {
                session_id: session_id.clone(),
                call_id: call_id.clone(),
                success: *success,
            }),
            InternalEvent::ToolCallOutput {
                session_id,
                call_id,
                output,
            } => Some(Self::ToolCallOutput {
                session_id: session_id.clone(),
                call_id: call_id.clone(),
                output: output.clone(),
            }),
            InternalEvent::AgentStatusChanged { session_id, status } => {
                Some(Self::AgentStatusChanged {
                    session_id: session_id.clone(),
                    status: status.clone(),
                })
            }
            InternalEvent::Error { source, message } => Some(Self::Error {
                source: source.clone(),
                message: message.clone(),
            }),
            _ => None,
        }
    }
}
