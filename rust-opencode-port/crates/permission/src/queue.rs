use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PermissionScope {
    ReadOnly,
    Restricted,
    Full,
}

impl Default for PermissionScope {
    fn default() -> Self {
        PermissionScope::ReadOnly
    }
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
            | "lsp_goto_definition"
            | "lsp_find_references"
            | "lsp_symbols"
    )
}

fn is_safe_tool(tool_name: &str) -> bool {
    is_read_tool(tool_name) || matches!(tool_name, "todowrite" | "bash")
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApprovalQueue {
    pub scope: PermissionScope,
    #[serde(skip)]
    pending: Vec<PendingApproval>,
    #[serde(skip)]
    history: Vec<ApprovedCommand>,
}

impl Default for ApprovalQueue {
    fn default() -> Self {
        Self::new(PermissionScope::default())
    }
}

impl ApprovalQueue {
    pub fn new(scope: PermissionScope) -> Self {
        Self {
            scope,
            pending: Vec::new(),
            history: Vec::new(),
        }
    }

    pub fn check(&self, tool_name: &str) -> ApprovalResult {
        match self.scope {
            PermissionScope::Full => ApprovalResult::AutoApprove,
            PermissionScope::ReadOnly => {
                if is_read_tool(tool_name) {
                    ApprovalResult::AutoApprove
                } else {
                    ApprovalResult::RequireApproval
                }
            }
            PermissionScope::Restricted => {
                if is_safe_tool(tool_name) {
                    ApprovalResult::AutoApprove
                } else {
                    ApprovalResult::RequireApproval
                }
            }
        }
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
            Some(approved)
        } else {
            None
        }
    }

    pub fn reject(&mut self, approval_id: Uuid) -> bool {
        if let Some(pos) = self.pending.iter().position(|p| p.id == approval_id) {
            self.pending.remove(pos);
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
}

#[cfg(test)]
mod tests {
    use super::*;

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
}
