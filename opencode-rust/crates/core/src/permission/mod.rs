mod types;

pub use types::{Permission, PermissionConfig, PermissionManager};

#[cfg(test)]
mod tests {
    use super::*;
    use opencode_permission::AuditLog;

    #[test]
    fn test_permission_config_default() {
        let config = PermissionConfig::default();
        assert!(config.allowed.contains(&Permission::FileRead));
    }

    #[test]
    fn test_permission_manager_check_allowed() {
        let config = PermissionConfig::default();
        let pm = PermissionManager::new(config);
        assert!(pm.check(&Permission::FileRead, "/some/path"));
    }

    #[test]
    fn test_permission_manager_check_not_allowed() {
        let config = PermissionConfig::default();
        let pm = PermissionManager::new(config);
        assert!(!pm.check(&Permission::BashExecute, "/some/path"));
    }

    #[test]
    fn test_permission_manager_check_always_denied() {
        let mut config = PermissionConfig::default();
        config.always_denied.push("/etc".to_string());
        let pm = PermissionManager::new(config);
        assert!(!pm.check(&Permission::FileRead, "/etc/passwd"));
    }

    #[test]
    fn test_permission_manager_check_always_allowed() {
        let mut config = PermissionConfig::default();
        config.always_allowed.push("/home".to_string());
        let pm = PermissionManager::new(config);
        assert!(pm.check(&Permission::FileRead, "/home/user/file"));
    }

    #[test]
    fn test_permission_manager_grant() {
        let mut pm = PermissionManager::default();
        pm.grant(Permission::BashExecute);
        assert!(pm.check(&Permission::BashExecute, "/test"));
    }

    #[test]
    fn test_permission_manager_revoke() {
        let mut pm = PermissionManager::default();
        pm.revoke(&Permission::FileRead);
        assert!(!pm.check(&Permission::FileRead, "/test"));
    }

    #[test]
    fn test_permission_check_records_audit_decision() {
        let tmp = tempfile::tempdir().unwrap();
        let log = AuditLog::new(tmp.path().join("permission_audit.db")).unwrap();
        let pm = PermissionManager::default().with_audit_log(log.clone());

        let _ = pm.check(&Permission::FileRead, "/tmp/file");
        let entries = log.get_recent_entries(10).unwrap();
        assert_eq!(entries.len(), 1);
    }
}
