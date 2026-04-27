use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CrashDump {
    pub version: String,
    pub crashed_at: DateTime<Utc>,
    pub session_id: String,
    pub messages_summary: Vec<MessageSummary>,
    pub tool_invocations_summary: Vec<ToolInvocationSummary>,
    pub state: String,
    pub panic_message: Option<String>,
    pub stack_trace: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MessageSummary {
    pub role: String,
    pub content_preview: String,
    pub timestamp: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolInvocationSummary {
    pub tool_name: String,
    pub started_at: DateTime<Utc>,
    pub completed_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone)]
pub struct ActiveSession {
    pub session: crate::Session,
}

impl ActiveSession {
    pub fn capture_messages_summary(&self, max_messages: usize) -> Vec<MessageSummary> {
        let messages_to_capture = self.session.messages.len().min(max_messages);
        self.session
            .messages
            .iter()
            .rev()
            .take(messages_to_capture)
            .map(|msg| {
                let preview = if msg.content.len() > 200 {
                    format!("{}...", &msg.content[..200])
                } else {
                    msg.content.clone()
                };
                MessageSummary {
                    role: format!("{:?}", msg.role),
                    content_preview: preview,
                    timestamp: msg.timestamp,
                }
            })
            .collect()
    }

    pub fn capture_tool_invocations_summary(
        &self,
        max_invocations: usize,
    ) -> Vec<ToolInvocationSummary> {
        self.session
            .tool_invocations
            .iter()
            .rev()
            .take(max_invocations)
            .map(|inv| ToolInvocationSummary {
                tool_name: inv.tool_name.clone(),
                started_at: inv.started_at,
                completed_at: inv.completed_at,
            })
            .collect()
    }
}

#[derive(Debug, thiserror::Error)]
pub enum CrashRecoveryError {
    #[error("no active session to save")]
    NoActiveSession,
    #[error("IO error: {0}")]
    IoError(String),
    #[error("serialization error: {0}")]
    SerializationError(String),
    #[error("deserialization error: {0}")]
    DeserializationError(String),
}

impl From<std::io::Error> for CrashRecoveryError {
    fn from(e: std::io::Error) -> Self {
        CrashRecoveryError::IoError(e.to_string())
    }
}