#[cfg(test)]
mod tests {
    use opencode_permission::{
        AgentPermissionScope, AuditDecision, AuditEntry, AuditLog, FilePermissionResult,
        PermissionEvaluator, PermissionScope, Role, UserPermissions,
    };

    #[test]
    fn security_authorization_admin_bypasses_all_permissions() {
        let mut evaluator = PermissionEvaluator::new();
        let mut perms = UserPermissions::new();
        perms.add_role(Role::Admin);

        assert!(evaluator.has_permission(&perms, "session:delete"));
        assert!(evaluator.has_permission(&perms, "repo:write"));
        assert!(evaluator.has_permission(&perms, "admin:all"));
        assert!(evaluator.has_permission(&perms, "any:permission"));
    }

    #[test]
    fn security_authorization_user_role_permissions() {
        let mut evaluator = PermissionEvaluator::new();
        let mut perms = UserPermissions::new();
        perms.add_role(Role::User);

        assert!(
            evaluator.has_permission(&perms, "repo:read"),
            "User should have repo:read"
        );
        assert!(
            evaluator.has_permission(&perms, "session:read"),
            "User should have session:read"
        );
        assert!(
            evaluator.has_permission(&perms, "session:create"),
            "User should have session:create"
        );
        assert!(
            !evaluator.has_permission(&perms, "repo:write"),
            "User should not have repo:write"
        );
        assert!(
            !evaluator.has_permission(&perms, "session:delete"),
            "User should not have session:delete"
        );
    }

    #[test]
    fn security_authorization_guest_role_restricted_permissions() {
        let mut evaluator = PermissionEvaluator::new();
        let mut perms = UserPermissions::new();
        perms.add_role(Role::Guest);

        assert!(
            evaluator.has_permission(&perms, "repo:read"),
            "Guest should have repo:read"
        );
        assert!(
            !evaluator.has_permission(&perms, "session:create"),
            "Guest should not have session:create"
        );
        assert!(
            !evaluator.has_permission(&perms, "session:delete"),
            "Guest should not have session:delete"
        );
    }

    #[test]
    fn security_authorization_explicit_permissions_override_roles() {
        let mut evaluator = PermissionEvaluator::new();
        let mut perms = UserPermissions::new();
        perms.add_role(Role::Guest);
        perms.add_permission("session:create");

        assert!(
            evaluator.has_permission(&perms, "session:create"),
            "Explicit permission should work"
        );
        assert!(
            !evaluator.has_permission(&perms, "session:delete"),
            "Guest still shouldn't have delete"
        );
    }

    #[test]
    fn security_authorization_denied_permissions_block_non_admin() {
        let mut evaluator = PermissionEvaluator::new();
        let mut perms = UserPermissions::new();
        perms.add_role(Role::User);
        perms.deny_permission("session:delete");

        assert!(
            !evaluator.has_permission(&perms, "session:delete"),
            "Denied permission should block User"
        );
    }

    #[test]
    fn security_authorization_wildcard_permission_matching() {
        let mut evaluator = PermissionEvaluator::new();
        let mut perms = UserPermissions::new();
        perms.add_permission("repo:*");

        assert!(evaluator.has_permission(&perms, "repo:read"));
        assert!(evaluator.has_permission(&perms, "repo:write"));
        assert!(evaluator.has_permission(&perms, "repo:delete"));
        assert!(!evaluator.has_permission(&perms, "session:read"));
    }

    #[test]
    fn security_authorization_question_mark_wildcard_matches_single_char() {
        let mut evaluator = PermissionEvaluator::new();
        let mut perms = UserPermissions::new();
        perms.add_permission("session:?");

        assert!(evaluator.has_permission(&perms, "session:x"));
        assert!(evaluator.has_permission(&perms, "session:a"));
        assert!(
            !evaluator.has_permission(&perms, "session:read"),
            "? only matches single char, not 'read'"
        );
        assert!(!evaluator.has_permission(&perms, "repo:read"));
    }

    #[test]
    fn security_data_protection_sensitive_files_denied() {
        let evaluator = PermissionEvaluator::new();

        assert!(evaluator.check_file_path(".env").is_denied());
        assert!(evaluator.check_file_path(".env.local").is_denied());
        assert!(evaluator.check_file_path(".env.production").is_denied());
        assert!(evaluator.check_file_path("credentials.json").is_denied());
        assert!(evaluator.check_file_path("secrets.json").is_denied());
        assert!(evaluator.check_file_path("api_key").is_denied());
        assert!(evaluator.check_file_path("id_rsa").is_denied());
    }

    #[test]
    fn security_data_protection_sensitive_paths_denied() {
        let evaluator = PermissionEvaluator::new();

        assert!(evaluator
            .check_file_path("/home/user/.aws/credentials")
            .is_denied());
        assert!(evaluator
            .check_file_path("/home/user/.aws/config")
            .is_denied());
        assert!(evaluator
            .check_file_path("~/.gcloud/application_credentials.json")
            .is_denied());
    }

    #[test]
    fn security_data_protection_pem_and_key_files_denied() {
        let evaluator = PermissionEvaluator::new();

        assert!(evaluator.check_file_path("server.pem").is_denied());
        assert!(evaluator.check_file_path("private.key").is_denied());
        assert!(evaluator.check_file_path("cert.p12").is_denied());
        assert!(evaluator.check_file_path("keystore.jks").is_denied());
    }

    #[test]
    fn security_data_protection_git_credentials_denied() {
        let evaluator = PermissionEvaluator::new();

        assert!(evaluator.check_file_path(".git-credentials").is_denied());
        assert!(evaluator.check_file_path(".gitconfig").is_denied());
    }

    #[test]
    fn security_data_protection_normal_files_allowed() {
        let evaluator = PermissionEvaluator::new();

        assert!(evaluator.check_file_path("source_code.rs").is_allowed());
        assert!(evaluator.check_file_path("README.md").is_allowed());
        assert!(evaluator.check_file_path("src/main.rs").is_allowed());
        assert!(evaluator.check_file_path("Cargo.toml").is_allowed());
        assert!(evaluator.check_file_path("tests/test_main.py").is_allowed());
    }

    #[test]
    fn security_data_protection_external_directory_blocked() {
        let evaluator = PermissionEvaluator::new().with_allowed_base("/home/project".to_string());

        assert!(evaluator.check_file_path("/etc/passwd").is_denied());
        assert!(evaluator
            .check_file_path("/home/other_project/secret.txt")
            .is_denied());
        assert!(evaluator
            .check_file_path("/home/project/src/main.rs")
            .is_allowed());
    }

    #[test]
    fn security_data_protection_sensitive_files_can_override() {
        let evaluator = PermissionEvaluator::new();
        let result = evaluator.check_file_path(".env");

        assert!(result.is_denied());
        assert!(
            result.can_override(),
            ".env should be overridable with explicit permission"
        );
        assert!(result.denied_reason().is_some());
    }

    #[test]
    fn security_data_protection_sensitive_directory_blocked() {
        let evaluator = PermissionEvaluator::new();

        assert!(evaluator.check_file_path("/etc/ssh/some_file").is_denied());
        assert!(evaluator
            .check_file_path("/root/.ssh/authorized_keys")
            .is_denied());
    }

    #[test]
    fn security_audit_log_records_decisions() {
        let tmp = tempfile::tempdir().unwrap();
        let log = AuditLog::new(tmp.path().join("audit.db")).expect("Should create audit log");

        let entry = AuditEntry {
            timestamp: chrono::Utc::now(),
            tool_name: "bash".to_string(),
            decision: AuditDecision::Allow,
            session_id: "session-123".to_string(),
            user_response: Some("approved".to_string()),
        };

        log.record_decision(entry.clone()).expect("Should record");

        let recent = log.get_recent_entries(10).expect("Should query");
        assert!(!recent.is_empty(), "Audit log should have entries");
        assert_eq!(recent[0].tool_name, "bash");
    }

    #[test]
    fn security_audit_log_queries_by_tool_name() {
        let tmp = tempfile::tempdir().unwrap();
        let log = AuditLog::new(tmp.path().join("audit.db")).expect("Should create audit log");

        log.record_decision(AuditEntry {
            timestamp: chrono::Utc::now(),
            tool_name: "read".to_string(),
            decision: AuditDecision::Allow,
            session_id: "s1".to_string(),
            user_response: None,
        })
        .expect("Should record");

        let by_tool = log.query_by_tool_name("read").expect("Should query");
        assert_eq!(by_tool.len(), 1);
        assert_eq!(by_tool[0].tool_name, "read");
    }

    #[test]
    fn security_audit_log_queries_by_time_range() {
        let tmp = tempfile::tempdir().unwrap();
        let log = AuditLog::new(tmp.path().join("audit.db")).expect("Should create audit log");

        let now = chrono::Utc::now();
        log.record_decision(AuditEntry {
            timestamp: now,
            tool_name: "read".to_string(),
            decision: AuditDecision::Allow,
            session_id: "s1".to_string(),
            user_response: None,
        })
        .expect("Should record");

        let range = log
            .query_by_time_range(
                now - chrono::Duration::minutes(1),
                now + chrono::Duration::minutes(1),
            )
            .expect("Should query");

        assert_eq!(range.len(), 1);
        assert_eq!(range[0].tool_name, "read");
    }

    #[test]
    fn security_audit_log_cleanup_old_entries() {
        let tmp = tempfile::tempdir().unwrap();
        let log = AuditLog::new(tmp.path().join("audit.db")).expect("Should create audit log");

        let old_entry = AuditEntry {
            timestamp: chrono::Utc::now() - chrono::Duration::days(60),
            tool_name: "old".to_string(),
            decision: AuditDecision::Allow,
            session_id: "s1".to_string(),
            user_response: None,
        };
        log.record_decision(old_entry).expect("Should record");

        let deleted = log.cleanup_older_than(30).expect("Should cleanup");
        assert!(deleted >= 1, "Should have deleted old entries");
    }

    #[test]
    fn security_permission_scope_read_only_blocks_write_tools() {
        use opencode_permission::check_tool_permission;

        assert_eq!(
            check_tool_permission("read", PermissionScope::ReadOnly),
            opencode_permission::ApprovalResult::AutoApprove
        );
        assert_eq!(
            check_tool_permission("write", PermissionScope::ReadOnly),
            opencode_permission::ApprovalResult::RequireApproval
        );
        assert_eq!(
            check_tool_permission("edit", PermissionScope::ReadOnly),
            opencode_permission::ApprovalResult::RequireApproval
        );
    }

    #[test]
    fn security_permission_scope_full_allows_all_tools() {
        use opencode_permission::check_tool_permission;

        assert_eq!(
            check_tool_permission("read", PermissionScope::Full),
            opencode_permission::ApprovalResult::AutoApprove
        );
        assert_eq!(
            check_tool_permission("write", PermissionScope::Full),
            opencode_permission::ApprovalResult::AutoApprove
        );
        assert_eq!(
            check_tool_permission("bash", PermissionScope::Full),
            opencode_permission::ApprovalResult::AutoApprove
        );
    }

    #[test]
    fn security_permission_scope_restricted_allows_safe_tools() {
        use opencode_permission::check_tool_permission;

        assert_eq!(
            check_tool_permission("read", PermissionScope::Restricted),
            opencode_permission::ApprovalResult::AutoApprove
        );
        assert_eq!(
            check_tool_permission("todowrite", PermissionScope::Restricted),
            opencode_permission::ApprovalResult::AutoApprove
        );
        assert_eq!(
            check_tool_permission("write", PermissionScope::Restricted),
            opencode_permission::ApprovalResult::RequireApproval
        );
    }

    #[test]
    fn security_agent_permission_scope_intersection() {
        assert_eq!(
            AgentPermissionScope::Full.intersect(AgentPermissionScope::ReadOnly),
            AgentPermissionScope::ReadOnly
        );
        assert_eq!(
            AgentPermissionScope::Full.intersect(AgentPermissionScope::None),
            AgentPermissionScope::None
        );
        assert_eq!(
            AgentPermissionScope::Restricted.intersect(AgentPermissionScope::ReadOnly),
            AgentPermissionScope::ReadOnly
        );
    }

    #[test]
    fn security_agent_permission_scope_can_write() {
        assert!(AgentPermissionScope::Full.can_write_files());
        assert!(!AgentPermissionScope::Restricted.can_write_files());
        assert!(!AgentPermissionScope::ReadOnly.can_write_files());
        assert!(!AgentPermissionScope::None.can_write_files());
    }

    #[test]
    fn security_agent_permission_scope_can_run_commands() {
        assert!(AgentPermissionScope::Full.can_run_commands());
        assert!(!AgentPermissionScope::Restricted.can_run_commands());
        assert!(!AgentPermissionScope::ReadOnly.can_run_commands());
    }

    #[test]
    fn security_agent_permission_scope_can_read() {
        assert!(AgentPermissionScope::Full.can_read_files());
        assert!(AgentPermissionScope::Restricted.can_read_files());
        assert!(AgentPermissionScope::ReadOnly.can_read_files());
        assert!(!AgentPermissionScope::None.can_read_files());
    }

    #[test]
    fn security_agent_permission_scope_can_search() {
        assert!(AgentPermissionScope::Full.can_search());
        assert!(AgentPermissionScope::Restricted.can_search());
        assert!(AgentPermissionScope::ReadOnly.can_search());
        assert!(!AgentPermissionScope::None.can_search());
    }

    #[test]
    fn security_agent_permission_scope_from_permissions() {
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

    #[test]
    fn security_user_permissions_builder_pattern() {
        let mut perms = UserPermissions::new();
        perms.add_role(Role::User);
        perms.add_permission("session:delete");
        perms.deny_permission("repo:admin");

        assert!(perms.roles.contains(&Role::User));
        assert!(perms
            .explicit_permissions
            .contains(&opencode_permission::Permission(
                "session:delete".to_string()
            )));
        assert!(perms
            .denied_permissions
            .contains(&opencode_permission::Permission("repo:admin".to_string())));
    }

    #[test]
    fn security_file_permission_result_helpers() {
        let allowed = FilePermissionResult::Allowed;
        assert!(allowed.is_allowed());
        assert!(!allowed.is_denied());
        assert_eq!(allowed.denied_reason(), None);
        assert!(!allowed.can_override());

        let denied = FilePermissionResult::Denied {
            reason: "Sensitive file".to_string(),
            can_override: true,
        };
        assert!(!denied.is_allowed());
        assert!(denied.is_denied());
        assert_eq!(denied.denied_reason(), Some("Sensitive file"));
        assert!(denied.can_override());
    }
}
