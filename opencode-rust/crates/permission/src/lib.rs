pub mod audit_log;
pub mod evaluator;
pub mod models;
pub mod queue;
pub mod sensitive_file;

pub use audit_log::{AuditDecision, AuditEntry, AuditLog, DecisionScope, PermissionDecision};
pub use evaluator::{FilePermissionResult, PermissionEvaluator};
pub use models::{AgentPermissionScope, Permission, Role, UserPermissions};
pub use queue::{ApprovalQueue, ApprovalResult, PendingApproval, PermissionScope};
pub use sensitive_file::{
    check_sensitive, get_sensitive_reason, is_external_directory, is_sensitive_directory,
    is_sensitive_path, SensitiveCheckResult,
};

/// Check if a tool should be auto-approved or require approval based on scope.
/// This is the minimal permission integration point for tools.
///
/// In ReadOnly scope: read tools auto-approved, others require approval
/// In Restricted scope: safe tools auto-approved, others require approval  
/// In Full scope: all tools auto-approved
pub fn check_tool_permission(tool_name: &str, scope: PermissionScope) -> ApprovalResult {
    let queue = ApprovalQueue::new(scope);
    queue.check(tool_name)
}

/// Check tool permission using default ReadOnly scope
pub fn check_tool_permission_default(tool_name: &str) -> ApprovalResult {
    check_tool_permission(tool_name, PermissionScope::ReadOnly)
}
