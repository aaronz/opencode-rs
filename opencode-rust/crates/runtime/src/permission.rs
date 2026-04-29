use std::sync::Arc;

use opencode_core::permission::{Permission, PermissionManager};
use opencode_permission::{AuditLog, PermissionScope};
use tokio::sync::RwLock;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RuntimePermissionDecision {
    Allow,
    Deny,
    RequiresApproval,
}

#[derive(Clone)]
pub struct RuntimePermissionAdapter {
    manager: Arc<RwLock<PermissionManager>>,
    approval_queue: Arc<RwLock<opencode_permission::ApprovalQueue>>,
    audit_log: Option<Arc<AuditLog>>,
}

impl RuntimePermissionAdapter {
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

    pub fn check(&self, permission: Permission, pattern: &str) -> RuntimePermissionDecision {
        let manager = self.manager.blocking_read();
        if manager.check(&permission, pattern) {
            RuntimePermissionDecision::Allow
        } else {
            RuntimePermissionDecision::Deny
        }
    }

    pub fn check_tool(&self, tool_name: &str) -> RuntimePermissionDecision {
        let queue = self.approval_queue.blocking_read();
        match queue.check(tool_name) {
            opencode_permission::ApprovalResult::AutoApprove => RuntimePermissionDecision::Allow,
            opencode_permission::ApprovalResult::Denied => RuntimePermissionDecision::Deny,
            opencode_permission::ApprovalResult::RequireApproval => {
                RuntimePermissionDecision::RequiresApproval
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

impl Default for RuntimePermissionAdapter {
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
