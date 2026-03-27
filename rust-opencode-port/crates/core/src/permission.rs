use serde::{Deserialize, Serialize};
use std::collections::HashSet;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum Permission {
    FileRead,
    FileWrite,
    FileDelete,
    BashExecute,
    NetworkAccess,
    ExternalDirectory,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PermissionConfig {
    pub allowed: HashSet<Permission>,
    pub always_allowed: Vec<String>,
    pub always_denied: Vec<String>,
}

impl Default for PermissionConfig {
    fn default() -> Self {
        let mut allowed = HashSet::new();
        allowed.insert(Permission::FileRead);

        Self {
            allowed,
            always_allowed: Vec::new(),
            always_denied: Vec::new(),
        }
    }
}

pub struct PermissionManager {
    config: PermissionConfig,
}

impl PermissionManager {
    pub fn new(config: PermissionConfig) -> Self {
        Self { config }
    }

    pub fn check(&self, permission: &Permission, pattern: &str) -> bool {
        for denied in &self.config.always_denied {
            if pattern.contains(denied) {
                return false;
            }
        }

        for allowed in &self.config.always_allowed {
            if pattern.contains(allowed) {
                return true;
            }
        }

        self.config.allowed.contains(permission)
    }

    pub fn grant(&mut self, permission: Permission) {
        self.config.allowed.insert(permission);
    }

    pub fn revoke(&mut self, permission: &Permission) {
        self.config.allowed.remove(permission);
    }
}

impl Default for PermissionManager {
    fn default() -> Self {
        Self::new(PermissionConfig::default())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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
}
