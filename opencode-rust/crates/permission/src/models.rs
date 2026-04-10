use serde::{Deserialize, Serialize};
use std::collections::HashSet;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct Permission(pub String);

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Default)]
pub enum AgentPermissionScope {
    #[default]
    Full,
    Restricted,
    ReadOnly,
    None,
}

impl AgentPermissionScope {
    pub fn intersect(self, other: AgentPermissionScope) -> AgentPermissionScope {
        match (self, other) {
            (AgentPermissionScope::None, _) | (_, AgentPermissionScope::None) => {
                AgentPermissionScope::None
            }
            (AgentPermissionScope::Full, other) | (other, AgentPermissionScope::Full) => other,
            (AgentPermissionScope::Restricted, other)
            | (other, AgentPermissionScope::Restricted) => match other {
                AgentPermissionScope::None => AgentPermissionScope::None,
                _ => other,
            },
            (AgentPermissionScope::ReadOnly, _) | (_, AgentPermissionScope::ReadOnly) => {
                AgentPermissionScope::ReadOnly
            }
        }
    }

    pub fn can_write_files(&self) -> bool {
        matches!(self, AgentPermissionScope::Full)
    }

    pub fn can_run_commands(&self) -> bool {
        matches!(self, AgentPermissionScope::Full)
    }

    pub fn can_execute_tools(&self) -> bool {
        !matches!(self, AgentPermissionScope::None)
    }

    pub fn can_read_files(&self) -> bool {
        matches!(
            self,
            AgentPermissionScope::ReadOnly
                | AgentPermissionScope::Restricted
                | AgentPermissionScope::Full
        )
    }

    pub fn can_search(&self) -> bool {
        matches!(
            self,
            AgentPermissionScope::ReadOnly
                | AgentPermissionScope::Restricted
                | AgentPermissionScope::Full
        )
    }

    pub fn from_agent_permissions(can_write: bool, can_run_commands: bool) -> Self {
        if !can_write && !can_run_commands {
            AgentPermissionScope::ReadOnly
        } else if can_write || can_run_commands {
            AgentPermissionScope::Full
        } else {
            AgentPermissionScope::Restricted
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum Role {
    Admin,
    User,
    Guest,
    Custom(String),
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct UserPermissions {
    pub roles: HashSet<Role>,
    pub explicit_permissions: HashSet<Permission>,
    pub denied_permissions: HashSet<Permission>,
}

impl UserPermissions {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add_role(&mut self, role: Role) {
        self.roles.insert(role);
    }

    pub fn add_permission(&mut self, permission: &str) {
        self.explicit_permissions
            .insert(Permission(permission.to_string()));
    }

    pub fn deny_permission(&mut self, permission: &str) {
        self.denied_permissions
            .insert(Permission(permission.to_string()));
    }
}
