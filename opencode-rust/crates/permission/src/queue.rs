use crate::audit_log::{AuditDecision, AuditEntry, AuditLog};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use tokio::sync::broadcast;
use uuid::Uuid;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
#[derive(Default)]
pub enum PermissionScope {
    #[default]
    ReadOnly,
    Restricted,
    Full,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PendingApproval {
    pub id: Uuid,
    pub session_id: Uuid,
    pub tool_name: String,
    pub arguments: serde_json::Value,
    pub requested_at: DateTime<Utc>,
}

impl PendingApproval {
    pub fn new(session_id: Uuid, tool_name: String, arguments: serde_json::Value) -> Self {
        Self {
            id: Uuid::new_v4(),
            session_id,
            tool_name,
            arguments,
            requested_at: Utc::now(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApprovedCommand {
    pub id: Uuid,
    pub session_id: Uuid,
    pub tool_name: String,
    pub arguments: serde_json::Value,
    pub approved_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ApprovalResult {
    AutoApprove,
    RequireApproval,
    Denied,
}

fn is_read_tool(tool_name: &str) -> bool {
    matches!(
        tool_name,
        "read"
            | "grep"
            | "glob"
            | "ls"
            | "look_at"
            | "codesearch"
            | "webfetch"
            | "session_info"
            | "session_load"
            | "lsp_goto_definition"
            | "lsp_find_references"
            | "lsp_symbols"
    )
}

fn is_safe_tool(tool_name: &str) -> bool {
    is_read_tool(tool_name) || matches!(tool_name, "todowrite" | "bash")
}

#[derive(Debug, Clone)]
pub enum ApprovalDecision {
    Approved(ApprovedCommand),
    Rejected(Uuid),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApprovalQueue {
    pub scope: PermissionScope,
    #[serde(skip)]
    audit_log: Option<AuditLog>,
    #[serde(skip)]
    pending: Vec<PendingApproval>,
    #[serde(skip)]
    history: Vec<ApprovedCommand>,
    #[serde(skip)]
    notification_tx: Option<broadcast::Sender<ApprovalDecision>>,
}

impl Default for ApprovalQueue {
    fn default() -> Self {
        Self::new(PermissionScope::default())
    }
}

impl ApprovalQueue {
    pub fn new(scope: PermissionScope) -> Self {
        let (notification_tx, _) = broadcast::channel(1024);
        Self {
            scope,
            audit_log: None,
            pending: Vec::new(),
            history: Vec::new(),
            notification_tx: Some(notification_tx),
        }
    }

    pub fn with_audit_log(mut self, audit_log: AuditLog) -> Self {
        self.audit_log = Some(audit_log);
        self
    }

    pub fn set_audit_log(&mut self, audit_log: AuditLog) {
        self.audit_log = Some(audit_log);
    }

    pub fn check(&self, tool_name: &str) -> ApprovalResult {
        tracing::debug!(tool = %tool_name, scope = ?self.scope, "Checking tool permission");

        let decision = match self.scope {
            PermissionScope::Full => {
                tracing::debug!(tool = %tool_name, "Permission granted - Full scope");
                ApprovalResult::AutoApprove
            }
            PermissionScope::ReadOnly => {
                if is_read_tool(tool_name) {
                    tracing::debug!(tool = %tool_name, "Permission granted - read tool in ReadOnly scope");
                    ApprovalResult::AutoApprove
                } else {
                    tracing::warn!(tool = %tool_name, "Permission requires approval - non-read tool in ReadOnly scope");
                    ApprovalResult::RequireApproval
                }
            }
            PermissionScope::Restricted => {
                if is_safe_tool(tool_name) {
                    tracing::debug!(tool = %tool_name, "Permission granted - safe tool in Restricted scope");
                    ApprovalResult::AutoApprove
                } else {
                    tracing::warn!(tool = %tool_name, "Permission requires approval - unsafe tool in Restricted scope");
                    ApprovalResult::RequireApproval
                }
            }
        };

        if let Some(log) = &self.audit_log {
            let _ = log.record_decision(AuditEntry {
                timestamp: Utc::now(),
                tool_name: tool_name.to_string(),
                decision: match decision {
                    ApprovalResult::AutoApprove => AuditDecision::Allow,
                    ApprovalResult::RequireApproval => AuditDecision::Ask,
                    ApprovalResult::Denied => AuditDecision::Deny,
                },
                session_id: Uuid::nil().to_string(),
                user_response: None,
            });
        }

        decision
    }

    pub fn request_approval(&mut self, pending: PendingApproval) {
        self.pending.push(pending);
    }

    pub fn approve(&mut self, approval_id: Uuid) -> Option<ApprovedCommand> {
        if let Some(pos) = self.pending.iter().position(|p| p.id == approval_id) {
            let pending = self.pending.remove(pos);
            let approved = ApprovedCommand {
                id: pending.id,
                session_id: pending.session_id,
                tool_name: pending.tool_name,
                arguments: pending.arguments,
                approved_at: Utc::now(),
            };
            self.history.push(approved.clone());
            if let Some(ref tx) = self.notification_tx {
                let _ = tx.send(ApprovalDecision::Approved(approved.clone()));
            }
            Some(approved)
        } else {
            None
        }
    }

    pub fn reject(&mut self, approval_id: Uuid) -> bool {
        if let Some(pos) = self.pending.iter().position(|p| p.id == approval_id) {
            self.pending.remove(pos);
            if let Some(ref tx) = self.notification_tx {
                let _ = tx.send(ApprovalDecision::Rejected(approval_id));
            }
            true
        } else {
            false
        }
    }

    pub fn get_pending(&self, session_id: Uuid) -> Vec<&PendingApproval> {
        self.pending
            .iter()
            .filter(|p| p.session_id == session_id)
            .collect()
    }

    pub fn get_history(&self, session_id: Uuid) -> Vec<&ApprovedCommand> {
        self.history
            .iter()
            .filter(|c| c.session_id == session_id)
            .collect()
    }

    pub fn set_scope(&mut self, scope: PermissionScope) {
        self.scope = scope;
    }

    pub fn subscribe(&self) -> Option<broadcast::Receiver<ApprovalDecision>> {
        self.notification_tx.as_ref().map(|tx| tx.subscribe())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::audit_log::AuditLog;

    #[test]
    fn test_read_only_scope_allows_read_tools() {
        let queue = ApprovalQueue::new(PermissionScope::ReadOnly);
        assert_eq!(queue.check("read"), ApprovalResult::AutoApprove);
        assert_eq!(queue.check("grep"), ApprovalResult::AutoApprove);
    }

    #[test]
    fn test_read_only_scope_blocks_write_tools() {
        let queue = ApprovalQueue::new(PermissionScope::ReadOnly);
        assert_eq!(queue.check("write"), ApprovalResult::RequireApproval);
        assert_eq!(queue.check("edit"), ApprovalResult::RequireApproval);
    }

    #[test]
    fn test_full_scope_allows_everything() {
        let queue = ApprovalQueue::new(PermissionScope::Full);
        assert_eq!(queue.check("write"), ApprovalResult::AutoApprove);
    }

    #[test]
    fn test_approval_request_flow() {
        let mut queue = ApprovalQueue::new(PermissionScope::ReadOnly);

        let pending = PendingApproval::new(
            Uuid::new_v4(),
            "write".to_string(),
            serde_json::json!({"path": "/test.txt"}),
        );
        let approval_id = pending.id;
        queue.request_approval(pending);

        assert_eq!(queue.pending.len(), 1);

        let approved = queue.approve(approval_id);
        assert!(approved.is_some());

        assert_eq!(queue.pending.len(), 0);
        assert_eq!(queue.history.len(), 1);
    }

    #[test]
    fn test_check_records_audit_entry_when_configured() {
        let tmp = tempfile::tempdir().unwrap();
        let log = AuditLog::new(tmp.path().join("audit.db")).unwrap();
        let queue = ApprovalQueue::new(PermissionScope::ReadOnly).with_audit_log(log.clone());

        let result = queue.check("write");
        assert_eq!(result, ApprovalResult::RequireApproval);

        let entries = log.get_recent_entries(10).unwrap();
        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].tool_name, "write");
        assert_eq!(entries[0].decision, crate::audit_log::AuditDecision::Ask);
    }

    #[tokio::test]
    async fn test_approve_notifies_subscribers() {
        let mut queue = ApprovalQueue::new(PermissionScope::ReadOnly);
        let pending = PendingApproval::new(
            Uuid::new_v4(),
            "write".to_string(),
            serde_json::json!({"path": "/test.txt"}),
        );
        let approval_id = pending.id;
        queue.request_approval(pending);

        let mut receiver = queue
            .subscribe()
            .expect("ApprovalQueue should have notification channel");
        let approved = queue.approve(approval_id);
        assert!(approved.is_some());

        let decision = receiver.recv().await.unwrap();
        match decision {
            ApprovalDecision::Approved(cmd) => {
                assert_eq!(cmd.tool_name, "write");
            }
            _ => panic!("Expected Approved decision"),
        }
    }

    #[tokio::test]
    async fn test_reject_notifies_subscribers() {
        let mut queue = ApprovalQueue::new(PermissionScope::ReadOnly);
        let pending = PendingApproval::new(
            Uuid::new_v4(),
            "write".to_string(),
            serde_json::json!({"path": "/test.txt"}),
        );
        let approval_id = pending.id;
        queue.request_approval(pending);

        let mut receiver = queue
            .subscribe()
            .expect("ApprovalQueue should have notification channel");
        let rejected = queue.reject(approval_id);
        assert!(rejected);

        let decision = receiver.recv().await.unwrap();
        match decision {
            ApprovalDecision::Rejected(id) => {
                assert_eq!(id, approval_id);
            }
            _ => panic!("Expected Rejected decision"),
        }
    }

    #[tokio::test]
    async fn test_reevaluate_after_decision() {
        let mut queue = ApprovalQueue::new(PermissionScope::ReadOnly);
        let pending = PendingApproval::new(
            Uuid::new_v4(),
            "write".to_string(),
            serde_json::json!({"path": "/test.txt"}),
        );
        let approval_id = pending.id;
        queue.request_approval(pending);

        let mut receiver = queue
            .subscribe()
            .expect("ApprovalQueue should have notification channel");

        let approved = queue.approve(approval_id);
        assert!(approved.is_some());

        let decision = receiver.recv().await;
        assert!(decision.is_ok());
    }
}
