use chrono::{DateTime, Utc};
use opencode_core::{OpenCodeError, Session};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
#[derive(Default)]
pub enum InvocationStatus {
    #[default]
    Running,
    Completed,
    Failed,
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
    pub args_hash: String,
    pub result: Option<serde_json::Value>,
    pub result_summary: Option<String>,
    pub started_at: DateTime<Utc>,
    pub completed_at: Option<DateTime<Utc>>,
    pub latency_ms: Option<u64>,
    pub status: InvocationStatus,
    pub permission_request_id: Option<Uuid>,
}

impl ToolInvocation {
    pub fn new(
        session_id: Uuid,
        message_id: Uuid,
        tool_name: String,
        arguments: serde_json::Value,
    ) -> Self {
        let args_hash = compute_args_hash(&arguments);
        Self {
            id: Uuid::new_v4(),
            session_id,
            message_id,
            tool_name,
            arguments,
            args_hash,
            result: None,
            result_summary: None,
            started_at: Utc::now(),
            completed_at: None,
            latency_ms: None,
            status: InvocationStatus::Running,
            permission_request_id: None,
        }
    }

    pub fn complete(&mut self, result: serde_json::Value) {
        self.completed_at = Some(Utc::now());
        self.status = InvocationStatus::Completed;
        self.latency_ms = self
            .completed_at
            .map(|completed| (completed - self.started_at).num_milliseconds() as u64);
        self.result = Some(result.clone());
        self.result_summary = Some(compute_result_summary(&result));
    }

    pub fn fail(&mut self) {
        self.completed_at = Some(Utc::now());
        self.latency_ms = self
            .completed_at
            .map(|completed| (completed - self.started_at).num_milliseconds() as u64);
        self.status = InvocationStatus::Failed;
    }

    pub fn set_permission_request_id(&mut self, request_id: Uuid) {
        self.permission_request_id = Some(request_id);
    }
}

fn compute_args_hash(arguments: &serde_json::Value) -> String {
    use sha2::{Digest, Sha256};
    let serialized = serde_json::to_string(arguments).unwrap_or_default();
    let mut hasher = Sha256::new();
    hasher.update(serialized.as_bytes());
    format!("{:x}", hasher.finalize())
}

fn compute_result_summary(result: &serde_json::Value) -> String {
    let serialized = serde_json::to_string(result).unwrap_or_default();
    const MAX_SUMMARY_LEN: usize = 1024;
    let truncated = if serialized.len() > MAX_SUMMARY_LEN {
        serialized[..MAX_SUMMARY_LEN].to_string()
    } else {
        serialized
    };
    redact_sensitive_info(&truncated)
}

fn redact_sensitive_info(content: &str) -> String {
    let sensitive_patterns = [
        "api_key",
        "token",
        "password",
        "secret",
        "credential",
        "authorization",
        "bearer",
        "sk-",
        "sk\\d",
        "token_",
        "_token",
    ];
    let mut result = content.to_string();
    for pattern in sensitive_patterns {
        let regex = regex::Regex::new(&format!(r#"(?i){}"#, pattern))
            .unwrap_or_else(|_| regex::Regex::new(r"api_key").unwrap());
        result = regex.replace_all(&result, "***REDACTED***").to_string();
    }
    result
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
