use std::sync::Arc;

use opencode_core::permission::{Permission, PermissionManager};
use opencode_permission::{AuditLog, PermissionScope};
use tokio::sync::RwLock;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RuntimeFacadePermissionDecision {
    Allow,
    Deny,
    RequiresApproval,
}

#[derive(Clone)]
pub struct RuntimeFacadePermissionAdapter {
    manager: Arc<RwLock<PermissionManager>>,
    approval_queue: Arc<RwLock<opencode_permission::ApprovalQueue>>,
    audit_log: Option<Arc<AuditLog>>,
}

impl RuntimeFacadePermissionAdapter {
    pub fn new(
        manager: Arc<RwLock<PermissionManager>>,
        approval_queue: Arc<RwLock<opencode_permission::ApprovalQueue>>,
        audit_log: Option<Arc<AuditLog>>,
    ) -> Self {
        Self {
            manager,
            approval_queue,
            audit_log,
        }
    }

    pub async fn check(&self, permission: Permission, pattern: &str) -> RuntimeFacadePermissionDecision {
        let manager = self.manager.read().await;
        if manager.check(&permission, pattern) {
            RuntimeFacadePermissionDecision::Allow
        } else {
            RuntimeFacadePermissionDecision::Deny
        }
    }

    pub async fn check_tool(&self, tool_name: &str) -> RuntimeFacadePermissionDecision {
        let queue = self.approval_queue.read().await;
        match queue.check(tool_name) {
            opencode_permission::ApprovalResult::AutoApprove => RuntimeFacadePermissionDecision::Allow,
            opencode_permission::ApprovalResult::Denied => RuntimeFacadePermissionDecision::Deny,
            opencode_permission::ApprovalResult::RequireApproval => {
                RuntimeFacadePermissionDecision::RequiresApproval
            }
        }
    }

    pub fn approval_queue(&self) -> Arc<RwLock<opencode_permission::ApprovalQueue>> {
        Arc::clone(&self.approval_queue)
    }

    pub fn audit_log(&self) -> Option<Arc<AuditLog>> {
        self.audit_log.clone()
    }
}

impl Default for RuntimeFacadePermissionAdapter {
    fn default() -> Self {
        Self::new(
            Arc::new(RwLock::new(PermissionManager::default())),
            Arc::new(RwLock::new(opencode_permission::ApprovalQueue::new(
                PermissionScope::default(),
            ))),
            None,
        )
    }
}
