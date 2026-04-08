//! Session management for OpenCode SDK.
//!
//! Provides types and operations for managing OpenCode sessions.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::error::SdkError;

/// Session information returned by the API.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionInfo {
    /// Unique session identifier.
    pub id: Uuid,

    /// Session creation timestamp.
    pub created_at: DateTime<Utc>,

    /// Last update timestamp.
    pub updated_at: DateTime<Utc>,

    /// Number of messages in the session.
    #[serde(default)]
    pub message_count: usize,

    /// Preview of the last message (if available).
    #[serde(default)]
    pub preview: Option<String>,
}

/// Session creation request.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateSessionRequest {
    /// Optional initial prompt to start the session with.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub initial_prompt: Option<String>,
}

/// Session creation response from the API.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateSessionResponse {
    /// The created session ID.
    pub session_id: Uuid,

    /// Creation timestamp.
    pub created_at: String,

    /// Session status.
    pub status: String,

    /// Number of messages.
    pub message_count: usize,
}

/// Fork session request.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ForkSessionRequest {
    /// Message index to fork at.
    pub fork_at_message_index: usize,
}

/// Fork session response.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ForkSessionResponse {
    /// New session ID after fork.
    pub id: Uuid,

    /// Parent session ID.
    #[serde(default)]
    pub parent_session_id: Option<String>,

    /// Message count in forked session.
    pub message_count: usize,
}

/// Add message request.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AddMessageRequest {
    /// Message role (user, assistant, system).
    #[serde(default = "default_role")]
    pub role: String,

    /// Message content.
    pub content: String,
}

fn default_role() -> String {
    "user".to_string()
}

/// Add message response.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AddMessageResponse {
    /// Session ID.
    pub session_id: Uuid,

    /// Total message count after adding.
    pub message_count: usize,
}

/// Session summary information.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionSummary {
    /// Summary text.
    pub summary: String,

    /// Summary creation timestamp.
    pub created_at: DateTime<Utc>,
}

/// Message type for session messages.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionMessage {
    /// Message role.
    pub role: String,

    /// Message content.
    pub content: String,
}

impl SessionInfo {
    /// Creates a new SessionInfo with the given ID.
    pub fn new(id: Uuid) -> Self {
        let now = Utc::now();
        Self {
            id,
            created_at: now,
            updated_at: now,
            message_count: 0,
            preview: None,
        }
    }

    /// Validates that the session ID is valid.
    pub fn validate_id(id: &str) -> Result<Uuid, SdkError> {
        Uuid::parse_str(id).map_err(|_| SdkError::session_not_found(id))
    }
}

impl SdkSession {
    /// Creates a new SDK session with the given ID.
    pub fn new(id: Uuid) -> Self {
        Self {
            id,
            messages: Vec::new(),
            created_at: Utc::now(),
            updated_at: Utc::now(),
            state: SessionState::Idle,
        }
    }

    /// Adds a user message to the session.
    pub fn add_user_message(&mut self, content: impl Into<String>) {
        self.messages.push(SessionMessage {
            role: "user".to_string(),
            content: content.into(),
        });
        self.updated_at = Utc::now();
    }

    /// Adds an assistant message to the session.
    pub fn add_assistant_message(&mut self, content: impl Into<String>) {
        self.messages.push(SessionMessage {
            role: "assistant".to_string(),
            content: content.into(),
        });
        self.updated_at = Utc::now();
    }
}

/// SDK Session representation for local use.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SdkSession {
    /// Unique session identifier.
    pub id: Uuid,

    /// Session messages.
    pub messages: Vec<SessionMessage>,

    /// Creation timestamp.
    pub created_at: DateTime<Utc>,

    /// Last update timestamp.
    pub updated_at: DateTime<Utc>,

    /// Current session state.
    pub state: SessionState,
}

/// Session state enumeration.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum SessionState {
    /// Session is idle and ready for input.
    Idle,

    /// Session is processing a request.
    Processing,

    /// Session has been aborted.
    Aborted,

    /// Session has completed.
    Completed,

    /// Session encountered an error.
    Error,
}

impl Default for SessionState {
    fn default() -> Self {
        Self::Idle
    }
}

impl std::fmt::Display for SessionState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Idle => write!(f, "idle"),
            Self::Processing => write!(f, "processing"),
            Self::Aborted => write!(f, "aborted"),
            Self::Completed => write!(f, "completed"),
            Self::Error => write!(f, "error"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_session_info_new() {
        let id = Uuid::new_v4();
        let info = SessionInfo::new(id);
        assert_eq!(info.id, id);
        assert_eq!(info.message_count, 0);
        assert!(info.preview.is_none());
    }

    #[test]
    fn test_session_info_validate_id() {
        let id = Uuid::new_v4();
        let result = SessionInfo::validate_id(&id.to_string());
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), id);

        let result = SessionInfo::validate_id("invalid");
        assert!(result.is_err());
    }

    #[test]
    fn test_sdk_session_new() {
        let id = Uuid::new_v4();
        let session = SdkSession::new(id);
        assert_eq!(session.id, id);
        assert!(session.messages.is_empty());
        assert_eq!(session.state, SessionState::Idle);
    }

    #[test]
    fn test_sdk_session_add_messages() {
        let mut session = SdkSession::new(Uuid::new_v4());
        session.add_user_message("Hello");
        session.add_assistant_message("Hi there!");

        assert_eq!(session.messages.len(), 2);
        assert_eq!(session.messages[0].role, "user");
        assert_eq!(session.messages[1].role, "assistant");
    }

    #[test]
    fn test_session_state_display() {
        assert_eq!(SessionState::Idle.to_string(), "idle");
        assert_eq!(SessionState::Processing.to_string(), "processing");
        assert_eq!(SessionState::Aborted.to_string(), "aborted");
    }
}
