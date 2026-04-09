use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Enterprise {
    pub id: String,
    pub name: String,
    pub plan: EnterprisePlan,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub settings: EnterpriseSettings,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum EnterprisePlan {
    Starter,
    Professional,
    Enterprise,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnterpriseSettings {
    pub allow_self_signup: bool,
    pub mfa_required: bool,
    pub session_timeout_minutes: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Policy {
    pub id: String,
    pub name: String,
    pub enterprise_id: String,
    pub permission_profile: PermissionProfile,
    pub allowed_providers: Vec<String>,
    pub mcp_restrictions: McpRestrictions,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PermissionProfile {
    pub tool_permissions: Vec<ToolPermission>,
    pub rate_limit_per_minute: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolPermission {
    pub tool_name: String,
    pub allowed: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpRestrictions {
    pub allowed_servers: Vec<String>,
    pub blocked_servers: Vec<String>,
}

pub struct EnterpriseManager;

impl EnterpriseManager {
    pub fn new() -> Self {
        Self
    }
}

impl Default for EnterpriseManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_enterprise_creation() {
        let enterprise = Enterprise {
            id: "ent-123".to_string(),
            name: "Test Enterprise".to_string(),
            plan: EnterprisePlan::Professional,
            created_at: chrono::Utc::now(),
            settings: EnterpriseSettings {
                allow_self_signup: false,
                mfa_required: true,
                session_timeout_minutes: 60,
            },
        };

        assert_eq!(enterprise.name, "Test Enterprise");
    }
}
