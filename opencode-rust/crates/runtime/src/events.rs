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
            InternalEvent::TaskStarted {
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
            InternalEvent::TaskProgress {
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
            InternalEvent::TaskCompleted {
                session_id,
                turn_id,
                task_id,
            } => Some(Self::TaskCompleted {
                session_id: session_id.clone(),
                turn_id: turn_id.clone(),
                task_id: task_id.clone(),
            }),
            InternalEvent::TaskFailed {
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
            InternalEvent::TaskCancelled {
                session_id,
                turn_id,
                task_id,
            } => Some(Self::TaskCancelled {
                session_id: session_id.clone(),
                turn_id: turn_id.clone(),
                task_id: task_id.clone(),
            }),
            _ => None,
        }
    }
}

impl From<RuntimeEvent> for InternalEvent {
    fn from(event: RuntimeEvent) -> Self {
        match event {
            RuntimeEvent::SessionStarted { session_id } => Self::SessionStarted(session_id),
            RuntimeEvent::SessionEnded { session_id } => Self::SessionEnded(session_id),
            RuntimeEvent::MessageAdded {
                session_id,
                message_id,
            } => Self::MessageAdded {
                session_id,
                message_id,
            },
            RuntimeEvent::MessageUpdated {
                session_id,
                message_id,
            } => Self::MessageUpdated {
                session_id,
                message_id,
            },
            RuntimeEvent::ToolCallStarted {
                session_id,
                tool_name,
                call_id,
            } => Self::ToolCallStarted {
                session_id,
                tool_name,
                call_id,
            },
            RuntimeEvent::ToolCallEnded {
                session_id,
                call_id,
                success,
            } => Self::ToolCallEnded {
                session_id,
                call_id,
                success,
            },
            RuntimeEvent::ToolCallOutput {
                session_id,
                call_id,
                output,
            } => Self::ToolCallOutput {
                session_id,
                call_id,
                output,
            },
            RuntimeEvent::AgentStatusChanged { session_id, status } => {
                Self::AgentStatusChanged { session_id, status }
            }
            RuntimeEvent::Error { source, message } => Self::Error { source, message },
            RuntimeEvent::TaskStarted {
                session_id,
                turn_id,
                task_id,
                task_kind,
            } => Self::TaskStarted {
                session_id,
                turn_id,
                task_id,
                task_kind,
            },
            RuntimeEvent::TaskProgress {
                session_id,
                turn_id,
                task_id,
                message,
            } => Self::TaskProgress {
                session_id,
                turn_id,
                task_id,
                message,
            },
            RuntimeEvent::TaskCompleted {
                session_id,
                turn_id,
                task_id,
            } => Self::TaskCompleted {
                session_id,
                turn_id,
                task_id,
            },
            RuntimeEvent::TaskFailed {
                session_id,
                turn_id,
                task_id,
                error,
            } => Self::TaskFailed {
                session_id,
                turn_id,
                task_id,
                error,
            },
            RuntimeEvent::TaskCancelled {
                session_id,
                turn_id,
                task_id,
            } => Self::TaskCancelled {
                session_id,
                turn_id,
                task_id,
            },
        }
    }
}
