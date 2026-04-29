use opencode_core::permission::{Permission, PermissionManager};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RuntimePermissionDecision {
    Allow,
    Deny,
}

pub struct RuntimePermissionAdapter {
    manager: PermissionManager,
}

impl RuntimePermissionAdapter {
    pub fn new(manager: PermissionManager) -> Self {
        Self { manager }
    }

    pub fn check(&self, permission: Permission, pattern: &str) -> RuntimePermissionDecision {
        if self.manager.check(&permission, pattern) {
            RuntimePermissionDecision::Allow
        } else {
            RuntimePermissionDecision::Deny
        }
    }
}

impl Default for RuntimePermissionAdapter {
    fn default() -> Self {
        Self::new(PermissionManager::default())
    }
}
