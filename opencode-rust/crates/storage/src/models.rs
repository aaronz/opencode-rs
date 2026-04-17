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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_args_hash_deterministic() {
        let args1 = serde_json::json!({"path": "/tmp/test.txt", "offset": 0});
        let args2 = serde_json::json!({"path": "/tmp/test.txt", "offset": 0});
        let hash1 = compute_args_hash(&args1);
        let hash2 = compute_args_hash(&args2);
        assert_eq!(hash1, hash2, "Same arguments should produce same hash");
    }

    #[test]
    fn test_args_hash_different_for_different_args() {
        let args1 = serde_json::json!({"path": "/tmp/test1.txt"});
        let args2 = serde_json::json!({"path": "/tmp/test2.txt"});
        let hash1 = compute_args_hash(&args1);
        let hash2 = compute_args_hash(&args2);
        assert_ne!(
            hash1, hash2,
            "Different arguments should produce different hashes"
        );
    }

    #[test]
    fn test_args_hash_sha256_format() {
        let args = serde_json::json!({"key": "value"});
        let hash = compute_args_hash(&args);
        assert_eq!(hash.len(), 64, "SHA256 hash should be 64 hex characters");
        assert!(
            hash.chars().all(|c| c.is_ascii_hexdigit()),
            "Hash should only contain hex characters"
        );
    }

    #[test]
    fn test_tool_invocation_includes_hash() {
        let session_id = Uuid::new_v4();
        let message_id = Uuid::new_v4();
        let args = serde_json::json!({"path": "/test.txt"});
        let invocation =
            ToolInvocation::new(session_id, message_id, "read".to_string(), args.clone());
        let expected_hash = compute_args_hash(&args);
        assert_eq!(invocation.args_hash, expected_hash);
    }

    #[test]
    fn test_tool_invocation_hash_consistency() {
        let session_id = Uuid::new_v4();
        let message_id = Uuid::new_v4();
        let args = serde_json::json!({"path": "/test.txt", "lines": 100});
        let inv1 = ToolInvocation::new(session_id, message_id, "read".to_string(), args.clone());
        let inv2 = ToolInvocation::new(session_id, message_id, "read".to_string(), args);
        assert_eq!(inv1.args_hash, inv2.args_hash);
    }

    #[test]
    fn test_invocation_status_display() {
        assert_eq!(InvocationStatus::Running.to_string(), "running");
        assert_eq!(InvocationStatus::Completed.to_string(), "completed");
        assert_eq!(InvocationStatus::Failed.to_string(), "failed");
    }

    #[test]
    fn test_invocation_status_default() {
        let status = InvocationStatus::default();
        assert_eq!(status, InvocationStatus::Running);
    }

    #[test]
    fn test_tool_invocation_complete() {
        let session_id = Uuid::new_v4();
        let message_id = Uuid::new_v4();
        let mut invocation = ToolInvocation::new(
            session_id,
            message_id,
            "read".to_string(),
            serde_json::json!({}),
        );

        let result = serde_json::json!({"content": "test result", "success": true});
        invocation.complete(result.clone());

        assert_eq!(invocation.status, InvocationStatus::Completed);
        assert!(invocation.completed_at.is_some());
        assert!(invocation.latency_ms.is_some());
        assert!(invocation.result.is_some());
        assert!(invocation.result_summary.is_some());
    }

    #[test]
    fn test_tool_invocation_fail() {
        let session_id = Uuid::new_v4();
        let message_id = Uuid::new_v4();
        let mut invocation = ToolInvocation::new(
            session_id,
            message_id,
            "read".to_string(),
            serde_json::json!({}),
        );

        invocation.fail();

        assert_eq!(invocation.status, InvocationStatus::Failed);
        assert!(invocation.completed_at.is_some());
        assert!(invocation.latency_ms.is_some());
        assert!(invocation.result.is_none());
    }

    #[test]
    fn test_tool_invocation_set_permission_request_id() {
        let session_id = Uuid::new_v4();
        let message_id = Uuid::new_v4();
        let mut invocation = ToolInvocation::new(
            session_id,
            message_id,
            "read".to_string(),
            serde_json::json!({}),
        );

        assert!(invocation.permission_request_id.is_none());

        let request_id = Uuid::new_v4();
        invocation.set_permission_request_id(request_id);

        assert_eq!(invocation.permission_request_id, Some(request_id));
    }

    #[test]
    fn test_redact_sensitive_info() {
        let input = r#"api_key=sk-1234567890abcdef token=my_secret_token password=secret123"#;
        let result = redact_sensitive_info(input);
        assert!(result.contains("***REDACTED***"));
        assert!(!result.contains("sk-1234567890abcdef"));
    }

    #[test]
    fn test_redact_sensitive_info_bearer_token() {
        let input = r#"authorization=bearer eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9"#;
        let result = redact_sensitive_info(input);
        assert!(result.contains("***REDACTED***"));
    }

    #[test]
    fn test_redact_sensitive_info_credentials() {
        let input = r#"credentials=my_credential secret=my_secret token_api=my_token"#;
        let result = redact_sensitive_info(input);
        assert!(result.contains("***REDACTED***"));
    }

    #[test]
    fn test_compute_result_summary_truncation() {
        let long_result = serde_json::json!({
            "content": "x".repeat(2000)
        });
        let summary = compute_result_summary(&long_result);
        assert!(summary.len() <= 1024);
    }

    #[test]
    fn test_compute_result_summary_no_truncation() {
        let short_result = serde_json::json!({"key": "value"});
        let summary = compute_result_summary(&short_result);
        assert_eq!(summary, r#"{"key":"value"}"#);
    }

    #[test]
    fn test_session_model_from_session() {
        use opencode_core::{Message, Session};
        let mut session = Session::new();
        session.add_message(Message::user("Hello".to_string()));
        session.add_message(Message::assistant("Hi there".to_string()));

        let model = SessionModel::from(session.clone());

        assert_eq!(model.id, session.id.to_string());
        assert_eq!(model.created_at, session.created_at);
        assert_eq!(model.updated_at, session.updated_at);
        assert!(!model.data.is_empty());
    }

    #[test]
    fn test_session_try_from_session_model() {
        use opencode_core::{Message, Session};
        let mut session = Session::new();
        session.add_message(Message::user("Hello".to_string()));

        let model = SessionModel::from(session.clone());
        let round_trip = Session::try_from(model).unwrap();

        assert_eq!(round_trip.id, session.id);
        assert_eq!(round_trip.messages.len(), session.messages.len());
    }

    #[test]
    fn test_session_try_from_session_model_invalid_data() {
        let model = SessionModel {
            id: "test".to_string(),
            created_at: Utc::now(),
            updated_at: Utc::now(),
            data: "invalid json".to_string(),
        };

        let result = Session::try_from(model);
        assert!(result.is_err());
    }

    #[test]
    fn test_plugin_state_model() {
        let model = PluginStateModel {
            plugin_id: "test-plugin".to_string(),
            state_data: r#"{"key": "value"}"#.to_string(),
            updated_at: Utc::now(),
        };

        assert_eq!(model.plugin_id, "test-plugin");
        assert_eq!(model.state_data, r#"{"key": "value"}"#);
    }

    #[test]
    fn test_project_model() {
        let model = ProjectModel {
            id: "proj-1".to_string(),
            path: "/tmp/test".to_string(),
            name: Some("Test Project".to_string()),
            created_at: Utc::now(),
            updated_at: Utc::now(),
            data: Some(r#"{"setting": true}"#.to_string()),
        };

        assert_eq!(model.name, Some("Test Project".to_string()));
        assert!(model.data.is_some());
    }

    #[test]
    fn test_account_model_last_login() {
        let last_login = Utc::now();
        let model = AccountModel {
            id: "acc-1".to_string(),
            username: "testuser".to_string(),
            email: Some("test@example.com".to_string()),
            password_hash: "hash123".to_string(),
            created_at: Utc::now(),
            updated_at: Utc::now(),
            last_login_at: Some(last_login),
            is_active: true,
            data: None,
        };

        assert!(model.last_login_at.is_some());
        assert_eq!(model.last_login_at.unwrap(), last_login);
    }
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
    let fallback_regex =
        regex::Regex::new(r"api_key").expect("regex pattern 'api_key' should always be valid");
    let mut result = content.to_string();
    for pattern in sensitive_patterns {
        let regex = regex::Regex::new(&format!(r#"(?i){}"#, pattern))
            .unwrap_or_else(|_| fallback_regex.clone());
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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginStateModel {
    pub plugin_id: String,
    pub state_data: String,
    pub updated_at: DateTime<Utc>,
}
