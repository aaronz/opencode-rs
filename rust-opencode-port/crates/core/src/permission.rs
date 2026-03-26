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
