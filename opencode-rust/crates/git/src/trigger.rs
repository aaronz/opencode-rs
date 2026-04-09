use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(tag = "type")]
pub enum GitHubTrigger {
    #[serde(rename = "issue_comment")]
    IssueComment {
        action: String,
        issue: IssueInfo,
        comment: CommentInfo,
    },
    #[serde(rename = "pull_request_review")]
    PullRequestReview {
        action: String,
        pull_request: PullRequestInfo,
        review: ReviewInfo,
    },
    #[serde(rename = "workflow_dispatch")]
    WorkflowDispatch {
        inputs: Option<HashMap<String, serde_json::Value>>,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct IssueInfo {
    pub number: u64,
    pub title: String,
    pub body: Option<String>,
    pub html_url: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct CommentInfo {
    pub id: u64,
    pub body: String,
    pub user: GitHubUserInfo,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct PullRequestInfo {
    pub number: u64,
    pub title: String,
    pub body: Option<String>,
    pub html_url: String,
    pub state: String,
    pub head: GitHubRefInfo,
    pub base: GitHubRefInfo,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ReviewInfo {
    pub id: u64,
    pub body: Option<String>,
    pub state: String,
    pub user: GitHubUserInfo,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct GitHubUserInfo {
    pub login: String,
    pub id: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct GitHubRefInfo {
    #[serde(rename = "ref")]
    pub ref_name: String,
    pub sha: String,
}

impl GitHubTrigger {
    pub fn parse(payload: &[u8]) -> Result<Self, TriggerParseError> {
        serde_json::from_slice(payload).map_err(|e| TriggerParseError::InvalidJson(e.to_string()))
    }

    #[allow(clippy::should_implement_trait)]
    pub fn from_str(payload: &str) -> Result<Self, TriggerParseError> {
        serde_json::from_str(payload).map_err(|e| TriggerParseError::InvalidJson(e.to_string()))
    }

    pub fn trigger_type(&self) -> &str {
        match self {
            GitHubTrigger::IssueComment { .. } => "issue_comment",
            GitHubTrigger::PullRequestReview { .. } => "pull_request_review",
            GitHubTrigger::WorkflowDispatch { .. } => "workflow_dispatch",
        }
    }

    pub fn is_comment_triggered(&self) -> bool {
        matches!(
            self,
            GitHubTrigger::IssueComment {
                action,
                ..
            } if action == "created"
        )
    }

    pub fn is_pr_review_triggered(&self) -> bool {
        matches!(
            self,
            GitHubTrigger::PullRequestReview {
                action,
                ..
            } if action == "submitted"
        )
    }

    pub fn extract_session_id(&self) -> Option<String> {
        match self {
            GitHubTrigger::IssueComment { issue, .. } => Some(format!("issue-{}", issue.number)),
            GitHubTrigger::PullRequestReview { pull_request, .. } => {
                Some(format!("pr-{}", pull_request.number))
            }
            GitHubTrigger::WorkflowDispatch { .. } => None,
        }
    }

    pub fn extract_context(&self) -> TriggerContext {
        match self {
            GitHubTrigger::IssueComment { issue, comment, .. } => TriggerContext::Issue {
                issue_number: issue.number,
                comment_id: comment.id,
                user: comment.user.login.clone(),
                body: comment.body.clone(),
            },
            GitHubTrigger::PullRequestReview {
                pull_request,
                review,
                ..
            } => TriggerContext::PullRequest {
                pr_number: pull_request.number,
                review_id: review.id,
                review_state: review.state.clone(),
                user: review.user.login.clone(),
                body: review.body.clone(),
            },
            GitHubTrigger::WorkflowDispatch { inputs } => TriggerContext::Workflow {
                inputs: inputs.clone(),
            },
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum TriggerContext {
    Issue {
        issue_number: u64,
        comment_id: u64,
        user: String,
        body: String,
    },
    PullRequest {
        pr_number: u64,
        review_id: u64,
        review_state: String,
        user: String,
        body: Option<String>,
    },
    Workflow {
        inputs: Option<HashMap<String, serde_json::Value>>,
    },
}

#[derive(Debug, thiserror::Error)]
pub enum TriggerParseError {
    #[error("invalid JSON: {0}")]
    InvalidJson(String),
}

pub struct CiSecrets {
    secrets: HashMap<String, String>,
}

impl CiSecrets {
    pub fn from_github_actions() -> Self {
        let mut secrets = HashMap::new();

        if let Ok(token) = std::env::var("GITHUB_TOKEN") {
            secrets.insert("GITHUB_TOKEN".to_string(), token);
        }
        if let Ok(repo) = std::env::var("GITHUB_REPOSITORY") {
            secrets.insert("GITHUB_REPOSITORY".to_string(), repo);
        }
        if let Ok(sha) = std::env::var("GITHUB_SHA") {
            secrets.insert("GITHUB_SHA".to_string(), sha);
        }
        if let Ok(ref_) = std::env::var("GITHUB_REF") {
            secrets.insert("GITHUB_REF".to_string(), ref_);
        }
        if let Ok(run_id) = std::env::var("GITHUB_RUN_ID") {
            secrets.insert("GITHUB_RUN_ID".to_string(), run_id);
        }
        if let Ok(trigger) = std::env::var("GITHUB_EVENT_NAME") {
            secrets.insert("GITHUB_EVENT_NAME".to_string(), trigger);
        }

        if let Ok(api_key) = std::env::var("OPENCODE_API_KEY") {
            secrets.insert("OPENCODE_API_KEY".to_string(), api_key);
        }
        if let Ok(model) = std::env::var("OPENCODE_MODEL") {
            secrets.insert("OPENCODE_MODEL".to_string(), model);
        }

        Self { secrets }
    }

    pub fn from_env_vars(prefix: &str) -> Self {
        let mut secrets = HashMap::new();

        for (key, value) in std::env::vars() {
            if key.starts_with(prefix) {
                let secret_name = key
                    .strip_prefix(prefix)
                    .map(|s| s.to_uppercase())
                    .unwrap_or_else(|| key.clone());
                secrets.insert(secret_name, value);
            }
        }

        Self { secrets }
    }

    pub fn get(&self, key: &str) -> Option<&String> {
        self.secrets.get(key)
    }

    pub fn get_optional(&self, key: &str) -> Option<&str> {
        self.secrets.get(key).map(|s| s.as_str())
    }

    pub fn get_required(&self, key: &str) -> Result<&str, MissingSecretError> {
        self.secrets
            .get(key)
            .map(|s| s.as_str())
            .ok_or_else(|| MissingSecretError(key.to_string()))
    }

    pub fn contains(&self, key: &str) -> bool {
        self.secrets.contains_key(key)
    }

    pub fn keys(&self) -> impl Iterator<Item = &String> {
        self.secrets.keys()
    }
}

#[derive(Debug, thiserror::Error)]
#[error("missing required secret: {0}")]
pub struct MissingSecretError(String);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_issue_comment_trigger() {
        let payload = r#"{
            "type": "issue_comment",
            "action": "created",
            "issue": {
                "number": 123,
                "title": "Test issue",
                "body": "Issue body",
                "html_url": "https://github.com/owner/repo/issues/123"
            },
            "comment": {
                "id": 456,
                "body": "/opencode help",
                "user": {"login": "user1", "id": 1}
            }
        }"#;

        let trigger = GitHubTrigger::from_str(payload).unwrap();
        assert!(matches!(trigger, GitHubTrigger::IssueComment { .. }));
        assert_eq!(trigger.trigger_type(), "issue_comment");
        assert!(trigger.is_comment_triggered());
        assert_eq!(trigger.extract_session_id(), Some("issue-123".to_string()));
    }

    #[test]
    fn test_parse_pull_request_review_trigger() {
        let payload = r#"{
            "type": "pull_request_review",
            "action": "submitted",
            "pull_request": {
                "number": 42,
                "title": "PR title",
                "body": "PR body",
                "html_url": "https://github.com/owner/repo/pull/42",
                "state": "open",
                "head": {"ref": "feature", "sha": "abc123"},
                "base": {"ref": "main", "sha": "def456"}
            },
            "review": {
                "id": 789,
                "body": "LGTM!",
                "state": "approved",
                "user": {"login": "reviewer1", "id": 2}
            }
        }"#;

        let trigger = GitHubTrigger::from_str(payload).unwrap();
        assert!(matches!(trigger, GitHubTrigger::PullRequestReview { .. }));
        assert_eq!(trigger.trigger_type(), "pull_request_review");
        assert!(trigger.is_pr_review_triggered());
        assert_eq!(trigger.extract_session_id(), Some("pr-42".to_string()));
    }

    #[test]
    fn test_parse_workflow_dispatch_trigger() {
        let payload = r#"{
            "type": "workflow_dispatch",
            "inputs": {
                "message": {"type": "string", "value": "Hello"}
            }
        }"#;

        let trigger = GitHubTrigger::from_str(payload).unwrap();
        assert!(matches!(trigger, GitHubTrigger::WorkflowDispatch { .. }));
        assert_eq!(trigger.trigger_type(), "workflow_dispatch");
        assert!(!trigger.is_comment_triggered());
        assert!(!trigger.is_pr_review_triggered());
        assert_eq!(trigger.extract_session_id(), None);
    }

    #[test]
    fn test_trigger_extract_context_issue() {
        let payload = r#"{
            "type": "issue_comment",
            "action": "created",
            "issue": {
                "number": 123,
                "title": "Test issue",
                "body": "Issue body",
                "html_url": "https://github.com/owner/repo/issues/123"
            },
            "comment": {
                "id": 456,
                "body": "/opencode help",
                "user": {"login": "user1", "id": 1}
            }
        }"#;

        let trigger = GitHubTrigger::from_str(payload).unwrap();
        let context = trigger.extract_context();
        assert!(matches!(
            context,
            TriggerContext::Issue {
                issue_number: 123,
                comment_id: 456,
                user,
                body
            } if user == "user1" && body == "/opencode help"
        ));
    }

    #[test]
    fn test_ci_secrets_from_empty_env() {
        let secrets = CiSecrets::from_env_vars("OPENCODE_");
        assert!(!secrets.contains("GITHUB_TOKEN"));
    }

    #[test]
    fn test_ci_secrets_get_operations() {
        let mut secrets = HashMap::new();
        secrets.insert("API_KEY".to_string(), "secret123".to_string());
        let ci_secrets = CiSecrets { secrets };

        assert_eq!(ci_secrets.get("API_KEY"), Some(&"secret123".to_string()));
        assert_eq!(ci_secrets.get_optional("API_KEY"), Some("secret123"));
        assert!(!ci_secrets.contains("MISSING"));
    }

    #[test]
    fn test_ci_secrets_required_missing() {
        let secrets = CiSecrets {
            secrets: HashMap::new(),
        };
        let result = secrets.get_required("MISSING");
        assert!(result.is_err());
    }

    #[test]
    fn test_trigger_parse_error() {
        let invalid_payload = "not valid json";
        let result = GitHubTrigger::from_str(invalid_payload);
        assert!(result.is_err());
    }
}
