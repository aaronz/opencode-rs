use opencode_permission::{AuditDecision, AuditEntry, AuditLog};
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
    audit_log: Option<AuditLog>,
}

impl PermissionManager {
    pub fn new(config: PermissionConfig) -> Self {
        let audit_log = std::env::var("OPENCODE_PERMISSION_AUDIT_DB")
            .ok()
            .and_then(|path| AuditLog::new(path).ok());
        Self { config, audit_log }
    }

    pub fn with_audit_log(mut self, audit_log: AuditLog) -> Self {
        self.audit_log = Some(audit_log);
        self
    }

    pub fn check(&self, permission: &Permission, pattern: &str) -> bool {
        let mut decision = self.config.allowed.contains(permission);

        for denied in &self.config.always_denied {
            if pattern.contains(denied) {
                decision = false;
                break;
            }
        }

        if !decision {
            for allowed in &self.config.always_allowed {
                if pattern.contains(allowed) {
                    decision = true;
                    break;
                }
            }
        }

        if let Some(audit_log) = &self.audit_log {
            let _ = audit_log.record_decision(AuditEntry {
                timestamp: chrono::Utc::now(),
                tool_name: format!("{:?}:{}", permission, pattern),
                decision: if decision {
                    AuditDecision::Allow
                } else {
                    AuditDecision::Deny
                },
                session_id: uuid::Uuid::nil().to_string(),
                user_response: None,
            });
        }

        decision
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
