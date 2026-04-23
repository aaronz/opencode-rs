#[cfg(test)]
mod permission_tests {
    use opencode_permission::{
        check_tool_permission, check_tool_permission_default, AgentPermissionScope, ApprovalResult,
        PermissionEvaluator, PermissionScope, Role, UserPermissions,
    };

    #[test]
    fn test_permission_e2e_001_readonly_blocks_write() {
        assert_eq!(
            check_tool_permission("read", PermissionScope::ReadOnly),
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
    fn test_permission_e2e_001_full_allows_write() {
        assert_eq!(
            check_tool_permission("write", PermissionScope::Full),
            ApprovalResult::AutoApprove
        );
        assert_eq!(
            check_tool_permission("bash", PermissionScope::Full),
            ApprovalResult::AutoApprove
        );
        assert_eq!(
            check_tool_permission("read", PermissionScope::Full),
            ApprovalResult::AutoApprove
        );
    }

    #[test]
    fn test_permission_e2e_002_permission_prompt_details() {
        let result = check_tool_permission("write", PermissionScope::ReadOnly);
        assert_eq!(result, ApprovalResult::RequireApproval);
    }

    #[test]
    fn test_permission_e2e_002_auto_approve_for_safe_tools() {
        assert_eq!(
            check_tool_permission_default("read"),
            ApprovalResult::AutoApprove
        );
        assert_eq!(
            check_tool_permission_default("grep"),
            ApprovalResult::AutoApprove
        );
    }

    #[test]
    fn test_permission_sec_001_tool_name_alias_blocked() {
        assert_eq!(
            check_tool_permission("bash", PermissionScope::ReadOnly),
            ApprovalResult::RequireApproval
        );
        assert_eq!(
            check_tool_permission("/usr/bin/bash", PermissionScope::ReadOnly),
            ApprovalResult::RequireApproval
        );
        assert_eq!(
            check_tool_permission("sh", PermissionScope::ReadOnly),
            ApprovalResult::RequireApproval
        );
    }

    #[test]
    fn test_permission_sec_002_runtime_escalation_blocked() {
        assert_eq!(
            check_tool_permission("grant_full_permissions", PermissionScope::ReadOnly),
            ApprovalResult::RequireApproval
        );
        assert_eq!(
            check_tool_permission("modify_scope", PermissionScope::ReadOnly),
            ApprovalResult::RequireApproval
        );
    }

    #[test]
    fn test_agent_permission_scope_001_hierarchy_readonly_intersect_full() {
        let scope = AgentPermissionScope::ReadOnly;
        let result = scope.intersect(AgentPermissionScope::Full);
        assert_eq!(result, AgentPermissionScope::ReadOnly);
    }

    #[test]
    fn test_agent_permission_scope_001_hierarchy_restricted_intersect_full() {
        let scope = AgentPermissionScope::Restricted;
        let result = scope.intersect(AgentPermissionScope::Full);
        assert_eq!(result, AgentPermissionScope::Restricted);
    }

    #[test]
    fn test_agent_permission_scope_001_hierarchy_none_intersect_anything() {
        let result = AgentPermissionScope::None.intersect(AgentPermissionScope::Full);
        assert_eq!(result, AgentPermissionScope::None);

        let result = AgentPermissionScope::None.intersect(AgentPermissionScope::ReadOnly);
        assert_eq!(result, AgentPermissionScope::None);
    }

    #[test]
    fn test_permission_scope_002_path_restriction_readonly() {
        let result = check_tool_permission("read", PermissionScope::ReadOnly);
        assert_eq!(result, ApprovalResult::AutoApprove);

        let result = check_tool_permission("write", PermissionScope::ReadOnly);
        assert_eq!(result, ApprovalResult::RequireApproval);
    }

    #[test]
    fn test_permission_scope_002_file_path_within_scope() {
        let mut evaluator = PermissionEvaluator::new();
        let mut user_perms = UserPermissions::new();
        user_perms.add_permission("file:read:/workspace/*");
        let has_read = evaluator.has_permission(&user_perms, "file:read:/workspace/*");
        assert!(has_read);
    }

    #[test]
    fn test_permission_audit_001_denial_logged_on_require_approval() {
        let result = check_tool_permission("bash", PermissionScope::ReadOnly);
        assert_eq!(result, ApprovalResult::RequireApproval);
    }

    #[test]
    fn test_permission_audit_001_auto_approve_logged() {
        let result = check_tool_permission("read", PermissionScope::ReadOnly);
        assert_eq!(result, ApprovalResult::AutoApprove);
    }

    #[test]
    fn test_agent_permission_scope_read_only_permissions() {
        let scope = AgentPermissionScope::ReadOnly;
        assert!(scope.can_read_files());
        assert!(!scope.can_write_files());
        assert!(!scope.can_run_commands());
        assert!(scope.can_execute_tools());
        assert!(scope.can_search());
    }

    #[test]
    fn test_agent_permission_scope_full_permissions() {
        let scope = AgentPermissionScope::Full;
        assert!(scope.can_read_files());
        assert!(scope.can_write_files());
        assert!(scope.can_run_commands());
        assert!(scope.can_execute_tools());
        assert!(scope.can_search());
    }

    #[test]
    fn test_agent_permission_scope_none_permissions() {
        let scope = AgentPermissionScope::None;
        assert!(!scope.can_read_files());
        assert!(!scope.can_write_files());
        assert!(!scope.can_run_commands());
        assert!(!scope.can_execute_tools());
        assert!(!scope.can_search());
    }

    #[test]
    fn test_agent_permission_scope_restricted_permissions() {
        let scope = AgentPermissionScope::Restricted;
        assert!(scope.can_read_files());
        assert!(!scope.can_write_files());
        assert!(!scope.can_run_commands());
        assert!(scope.can_execute_tools());
        assert!(scope.can_search());
    }

    #[test]
    fn test_permission_e2e_001_restricted_allows_bash() {
        assert_eq!(
            check_tool_permission("bash", PermissionScope::Restricted),
            ApprovalResult::AutoApprove
        );
    }

    #[test]
    fn test_permission_e2e_001_restricted_allows_read() {
        assert_eq!(
            check_tool_permission("read", PermissionScope::Restricted),
            ApprovalResult::AutoApprove
        );
    }

    #[test]
    fn test_permission_e2e_001_restricted_blocks_write() {
        assert_eq!(
            check_tool_permission("write", PermissionScope::Restricted),
            ApprovalResult::RequireApproval
        );
    }

    #[test]
    fn test_permission_state_002_approval_idempotency() {
        let result1 = check_tool_permission("read", PermissionScope::ReadOnly);
        let result2 = check_tool_permission("read", PermissionScope::ReadOnly);
        assert_eq!(result1, result2);
    }

    #[test]
    fn test_user_permissions_explicit_permission() {
        let mut perms = UserPermissions::new();
        perms.add_permission("file:read:/etc/*");

        let mut evaluator = PermissionEvaluator::new();
        assert!(evaluator.has_permission(&perms, "file:read:/etc/passwd"));
    }

    #[test]
    fn test_user_permissions_denied_permission() {
        let mut perms = UserPermissions::new();
        perms.add_permission("file:read:*");
        perms.deny_permission("file:read:/etc/*");

        let mut evaluator = PermissionEvaluator::new();
        assert!(!evaluator.has_permission(&perms, "file:read:/etc/passwd"));
    }

    #[test]
    fn test_user_permissions_admin_bypasses_all() {
        let mut perms = UserPermissions::new();
        perms.add_role(Role::Admin);
        perms.deny_permission("file:read:/etc/*");

        let mut evaluator = PermissionEvaluator::new();
        assert!(evaluator.has_permission(&perms, "file:read:/etc/passwd"));
        assert!(evaluator.has_permission(&perms, "anything"));
    }

    #[test]
    fn test_user_permissions_guest_limited() {
        let mut perms = UserPermissions::new();
        perms.add_role(Role::Guest);

        let mut evaluator = PermissionEvaluator::new();
        assert!(evaluator.has_permission(&perms, "repo:read"));
        assert!(!evaluator.has_permission(&perms, "repo:write"));
    }

    #[test]
    fn test_permission_scope_default_is_readonly() {
        assert_eq!(PermissionScope::default(), PermissionScope::ReadOnly);
    }

    #[test]
    fn test_agent_permission_scope_none_blocks_tool_execution() {
        let scope = AgentPermissionScope::None;
        assert!(!scope.can_execute_tools());
    }

    #[test]
    fn test_permission_sec_003_none_requires_approval() {
        let scope = AgentPermissionScope::None;
        assert!(!scope.can_execute_tools());
    }

    #[test]
    fn test_permission_state_001_temporary_permissions() {
        let mut perms = UserPermissions::new();
        perms.add_permission("file:read:/tmp/test");

        let mut evaluator = PermissionEvaluator::new();
        assert!(evaluator.has_permission(&perms, "file:read:/tmp/test"));
    }
}
