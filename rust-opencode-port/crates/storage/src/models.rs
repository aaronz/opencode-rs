use chrono::{DateTime, Utc};
use opencode_core::{OpenCodeError, Session};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum InvocationStatus {
    Running,
    Completed,
    Failed,
}

impl Default for InvocationStatus {
    fn default() -> Self {
        InvocationStatus::Running
    }
}

impl std::fmt::Display for InvocationStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            InvocationStatus::Running => write!(f, "running"),
            InvocationStatus::Completed => write!(f, "completed"),
            InvocationStatus::Failed => write!(f, "failed"),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolInvocation {
    pub id: Uuid,
    pub session_id: Uuid,
    pub message_id: Uuid,
    pub tool_name: String,
    pub arguments: serde_json::Value,
    pub result: Option<serde_json::Value>,
    pub started_at: DateTime<Utc>,
    pub completed_at: Option<DateTime<Utc>>,
    pub status: InvocationStatus,
}

impl ToolInvocation {
    pub fn new(
        session_id: Uuid,
        message_id: Uuid,
        tool_name: String,
        arguments: serde_json::Value,
    ) -> Self {
        Self {
            id: Uuid::new_v4(),
            session_id,
            message_id,
            tool_name,
            arguments,
            result: None,
            started_at: Utc::now(),
            completed_at: None,
            status: InvocationStatus::Running,
        }
    }

    pub fn complete(&mut self, result: serde_json::Value) {
        self.result = Some(result);
        self.completed_at = Some(Utc::now());
        self.status = InvocationStatus::Completed;
    }

    pub fn fail(&mut self) {
        self.completed_at = Some(Utc::now());
        self.status = InvocationStatus::Failed;
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionModel {
    pub id: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub data: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectModel {
    pub id: String,
    pub path: String,
    pub name: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub data: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccountModel {
    pub id: String,
    pub username: String,
    pub email: Option<String>,
    pub password_hash: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub last_login_at: Option<DateTime<Utc>>,
    pub is_active: bool,
    pub data: Option<String>,
}

impl From<Session> for SessionModel {
    fn from(session: Session) -> Self {
        Self {
            id: session.id.to_string(),
            created_at: session.created_at,
            updated_at: session.updated_at,
            data: serde_json::to_string(&session).unwrap_or_default(),
        }
    }
}

impl TryFrom<SessionModel> for Session {
    type Error = OpenCodeError;

    fn try_from(model: SessionModel) -> Result<Self, Self::Error> {
        serde_json::from_str(&model.data).map_err(|e| OpenCodeError::Storage(e.to_string()))
    }
}
