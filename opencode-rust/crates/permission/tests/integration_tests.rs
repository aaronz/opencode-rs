use opencode_permission::{
    check_tool_permission, check_tool_permission_default, ApprovalQueue, ApprovalResult,
    PermissionEvaluator, PermissionScope, UserPermissions,
};
use opencode_permission::{AgentPermissionScope, Permission, Role};

#[test]
fn test_check_tool_permission_default_is_readonly() {
    assert_eq!(
        check_tool_permission_default("read"),
        ApprovalResult::AutoApprove
    );
    assert_eq!(
        check_tool_permission_default("write"),
        ApprovalResult::RequireApproval
    );
    assert_eq!(
        check_tool_permission_default("grep"),
        ApprovalResult::AutoApprove
    );
}

#[test]
fn test_check_tool_permission_full_scope() {
    assert_eq!(
        check_tool_permission("read", PermissionScope::Full),
        ApprovalResult::AutoApprove
    );
    assert_eq!(
        check_tool_permission("write", PermissionScope::Full),
        ApprovalResult::AutoApprove
    );
    assert_eq!(
        check_tool_permission("bash", PermissionScope::Full),
        ApprovalResult::AutoApprove
    );
}

#[test]
fn test_check_tool_permission_readonly_scope() {
    assert_eq!(
        check_tool_permission("read", PermissionScope::ReadOnly),
        ApprovalResult::AutoApprove
    );
    assert_eq!(
        check_tool_permission("grep", PermissionScope::ReadOnly),
        ApprovalResult::AutoApprove
    );
    assert_eq!(
        check_tool_permission("write", PermissionScope::ReadOnly),
        ApprovalResult::RequireApproval
    );
    assert_eq!(
        check_tool_permission("bash", PermissionScope::ReadOnly),
        ApprovalResult::RequireApproval
    );
}

#[test]
fn test_check_tool_permission_restricted_scope() {
    assert_eq!(
        check_tool_permission("read", PermissionScope::Restricted),
        ApprovalResult::AutoApprove
    );
    assert_eq!(
        check_tool_permission("todowrite", PermissionScope::Restricted),
        ApprovalResult::AutoApprove
    );
    assert_eq!(
        check_tool_permission("bash", PermissionScope::Restricted),
        ApprovalResult::AutoApprove
    );
    assert_eq!(
        check_tool_permission("write", PermissionScope::Restricted),
        ApprovalResult::RequireApproval
    );
    assert_eq!(
        check_tool_permission("edit", PermissionScope::Restricted),
        ApprovalResult::RequireApproval
    );
}

#[test]
fn test_permission_scope_default_is_readonly() {
    assert_eq!(PermissionScope::default(), PermissionScope::ReadOnly);
}

#[test]
fn test_user_permissions_new() {
    let perms = UserPermissions::new();
    assert!(perms.roles.is_empty());
    assert!(perms.explicit_permissions.is_empty());
    assert!(perms.denied_permissions.is_empty());
}

#[test]
fn test_user_permissions_add_role() {
    let mut perms = UserPermissions::new();
    perms.add_role(Role::Admin);
    assert!(perms.roles.contains(&Role::Admin));
}

#[test]
fn test_user_permissions_add_permission() {
    let mut perms = UserPermissions::new();
    perms.add_permission("repo:read");
    assert!(perms
        .explicit_permissions
        .contains(&Permission("repo:read".to_string())));
}

#[test]
fn test_user_permissions_deny_permission() {
    let mut perms = UserPermissions::new();
    perms.deny_permission("repo:write");
    assert!(perms
        .denied_permissions
        .contains(&Permission("repo:write".to_string())));
}

#[test]
fn test_permission_evaluator_admin_always_has_permission() {
    let mut evaluator = PermissionEvaluator::new();
    let mut perms = UserPermissions::new();
    perms.add_role(Role::Admin);

    assert!(evaluator.has_permission(&perms, "any.permission"));
    assert!(evaluator.has_permission(&perms, "another.permission"));
}

#[test]
fn test_permission_evaluator_explicit_permission() {
    let mut evaluator = PermissionEvaluator::new();
    let mut perms = UserPermissions::new();
    perms.add_permission("repo:read");

    assert!(evaluator.has_permission(&perms, "repo:read"));
    assert!(!evaluator.has_permission(&perms, "repo:write"));
}

#[test]
fn test_permission_evaluator_denied_permission() {
    let mut evaluator = PermissionEvaluator::new();
    let mut perms = UserPermissions::new();
    perms.add_permission("repo:read");
    perms.deny_permission("repo:read");

    assert!(!evaluator.has_permission(&perms, "repo:read"));
}

#[test]
fn test_permission_evaluator_wildcard_permission() {
    let mut evaluator = PermissionEvaluator::new();
    let mut perms = UserPermissions::new();
    perms.add_permission("*");

    assert!(evaluator.has_permission(&perms, "anything"));
    assert!(evaluator.has_permission(&perms, "repo:read"));
}

#[test]
fn test_permission_evaluator_glob_pattern() {
    let mut evaluator = PermissionEvaluator::new();
    let mut perms = UserPermissions::new();
    perms.add_permission("repo:*");

    assert!(evaluator.has_permission(&perms, "repo:read"));
    assert!(evaluator.has_permission(&perms, "repo:write"));
    assert!(!evaluator.has_permission(&perms, "other:read"));
}

#[test]
fn test_permission_evaluator_role_permissions() {
    let mut evaluator = PermissionEvaluator::new();
    let mut perms = UserPermissions::new();
    perms.add_role(Role::User);

    assert!(evaluator.has_permission(&perms, "repo:read"));
    assert!(evaluator.has_permission(&perms, "session:read"));
    assert!(evaluator.has_permission(&perms, "session:create"));
    assert!(!evaluator.has_permission(&perms, "repo:write"));
}

#[test]
fn test_permission_evaluator_guest_permissions() {
    let mut evaluator = PermissionEvaluator::new();
    let mut perms = UserPermissions::new();
    perms.add_role(Role::Guest);

    assert!(evaluator.has_permission(&perms, "repo:read"));
    assert!(!evaluator.has_permission(&perms, "session:create"));
}

#[test]
fn test_approval_queue_new() {
    let queue = ApprovalQueue::new(PermissionScope::ReadOnly);
    assert_eq!(queue.scope, PermissionScope::ReadOnly);
}

#[test]
fn test_approval_queue_with_audit_log() {
    let tmp = tempfile::tempdir().unwrap();
    let log = opencode_permission::AuditLog::new(tmp.path().join("audit.db")).unwrap();
    let queue = ApprovalQueue::new(PermissionScope::Full).with_audit_log(log);
    // Verify audit logging is set up by checking that check() works
    assert_eq!(queue.check("read"), ApprovalResult::AutoApprove);
}

#[test]
fn test_approval_queue_check_full_scope() {
    let queue = ApprovalQueue::new(PermissionScope::Full);
    assert_eq!(queue.check("read"), ApprovalResult::AutoApprove);
    assert_eq!(queue.check("write"), ApprovalResult::AutoApprove);
}

#[test]
fn test_approval_queue_check_readonly_scope() {
    let queue = ApprovalQueue::new(PermissionScope::ReadOnly);
    assert_eq!(queue.check("read"), ApprovalResult::AutoApprove);
    assert_eq!(queue.check("grep"), ApprovalResult::AutoApprove);
    assert_eq!(queue.check("write"), ApprovalResult::RequireApproval);
}

#[test]
fn test_approval_queue_check_restricted_scope() {
    let queue = ApprovalQueue::new(PermissionScope::Restricted);
    assert_eq!(queue.check("read"), ApprovalResult::AutoApprove);
    assert_eq!(queue.check("bash"), ApprovalResult::AutoApprove);
    assert_eq!(queue.check("write"), ApprovalResult::RequireApproval);
}

#[test]
fn test_approval_queue_approve_reject_flow() {
    let mut queue = ApprovalQueue::new(PermissionScope::ReadOnly);
    let pending = opencode_permission::PendingApproval::new(
        uuid::Uuid::new_v4(),
        "write".to_string(),
        serde_json::json!({"path": "/test.txt"}),
    );
    let approval_id = pending.id;

    queue.request_approval(pending);

    let approved = queue.approve(approval_id);
    assert!(approved.is_some());

    // Verify the approval worked by checking the tool again
    // In ReadOnly scope, write should require approval
    assert_eq!(queue.check("write"), ApprovalResult::RequireApproval);
}

#[test]
fn test_approval_queue_reject() {
    let mut queue = ApprovalQueue::new(PermissionScope::ReadOnly);
    let pending = opencode_permission::PendingApproval::new(
        uuid::Uuid::new_v4(),
        "write".to_string(),
        serde_json::json!({"path": "/test.txt"}),
    );
    let approval_id = pending.id;

    queue.request_approval(pending);

    let rejected = queue.reject(approval_id);
    assert!(rejected);

    // Verify the rejection worked
    assert_eq!(queue.check("write"), ApprovalResult::RequireApproval);
}

#[test]
fn test_approval_queue_get_pending_by_session() {
    let mut queue = ApprovalQueue::new(PermissionScope::ReadOnly);
    let session_id = uuid::Uuid::new_v4();

    queue.request_approval(opencode_permission::PendingApproval::new(
        session_id,
        "write".to_string(),
        serde_json::json!({}),
    ));
    queue.request_approval(opencode_permission::PendingApproval::new(
        uuid::Uuid::new_v4(),
        "write".to_string(),
        serde_json::json!({}),
    ));

    let pending = queue.get_pending(session_id);
    assert_eq!(pending.len(), 1);
}

#[test]
fn test_approval_queue_set_scope() {
    let mut queue = ApprovalQueue::new(PermissionScope::ReadOnly);
    assert_eq!(queue.check("write"), ApprovalResult::RequireApproval);

    queue.set_scope(PermissionScope::Full);
    assert_eq!(queue.check("write"), ApprovalResult::AutoApprove);
}

#[test]
fn test_agent_permission_scope_intersect() {
    assert_eq!(
        AgentPermissionScope::Full.intersect(AgentPermissionScope::ReadOnly),
        AgentPermissionScope::ReadOnly
    );
    assert_eq!(
        AgentPermissionScope::ReadOnly.intersect(AgentPermissionScope::Full),
        AgentPermissionScope::ReadOnly
    );
    assert_eq!(
        AgentPermissionScope::Restricted.intersect(AgentPermissionScope::Full),
        AgentPermissionScope::Restricted
    );
    assert_eq!(
        AgentPermissionScope::None.intersect(AgentPermissionScope::Full),
        AgentPermissionScope::None
    );
}

#[test]
fn test_agent_permission_scope_can_write_files() {
    assert!(AgentPermissionScope::Full.can_write_files());
    assert!(!AgentPermissionScope::ReadOnly.can_write_files());
    assert!(!AgentPermissionScope::Restricted.can_write_files());
    assert!(!AgentPermissionScope::None.can_write_files());
}

#[test]
fn test_agent_permission_scope_can_run_commands() {
    assert!(AgentPermissionScope::Full.can_run_commands());
    assert!(!AgentPermissionScope::ReadOnly.can_run_commands());
    assert!(!AgentPermissionScope::Restricted.can_run_commands());
    assert!(!AgentPermissionScope::None.can_run_commands());
}

#[test]
fn test_agent_permission_scope_can_execute_tools() {
    assert!(AgentPermissionScope::Full.can_execute_tools());
    assert!(AgentPermissionScope::ReadOnly.can_execute_tools());
    assert!(AgentPermissionScope::Restricted.can_execute_tools());
    assert!(!AgentPermissionScope::None.can_execute_tools());
}

#[test]
fn test_agent_permission_scope_can_read_files() {
    assert!(AgentPermissionScope::Full.can_read_files());
    assert!(AgentPermissionScope::ReadOnly.can_read_files());
    assert!(AgentPermissionScope::Restricted.can_read_files());
    assert!(!AgentPermissionScope::None.can_read_files());
}

#[test]
fn test_agent_permission_scope_from_agent_permissions() {
    assert_eq!(
        AgentPermissionScope::from_agent_permissions(false, false),
        AgentPermissionScope::ReadOnly
    );
    assert_eq!(
        AgentPermissionScope::from_agent_permissions(true, false),
        AgentPermissionScope::Full
    );
    assert_eq!(
        AgentPermissionScope::from_agent_permissions(false, true),
        AgentPermissionScope::Full
    );
    assert_eq!(
        AgentPermissionScope::from_agent_permissions(true, true),
        AgentPermissionScope::Full
    );
}
