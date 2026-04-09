use serde::{Deserialize, Serialize};
use std::collections::HashSet;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct Permission(pub String);

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
