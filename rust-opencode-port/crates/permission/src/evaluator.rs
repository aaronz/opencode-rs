use crate::models::{Role, UserPermissions};
use regex::Regex;
use std::collections::HashMap;

pub struct PermissionEvaluator {
    regex_cache: HashMap<String, Regex>,
}

impl PermissionEvaluator {
    pub fn new() -> Self {
        Self {
            regex_cache: HashMap::new(),
        }
    }

    pub fn has_permission(&mut self, user_perms: &UserPermissions, required: &str) -> bool {
        if user_perms.roles.contains(&Role::Admin) {
            return true;
        }

        for denied in &user_perms.denied_permissions {
            if self.matches(&denied.0, required) {
                return false;
            }
        }

        for explicit in &user_perms.explicit_permissions {
            if self.matches(&explicit.0, required) {
                return true;
            }
        }

        for role in &user_perms.roles {
            if self.role_has_permission(role, required) {
                return true;
            }
        }

        false
    }

    fn matches(&mut self, pattern: &str, required: &str) -> bool {
        if pattern == required || pattern == "*" {
            return true;
        }

        if !pattern.contains('*') && !pattern.contains('?') {
            return pattern == required;
        }

        let regex_pattern = format!(
            "^{}$",
            pattern
                .replace(".", "\\.")
                .replace("*", ".*")
                .replace("?", ".")
        );

        let regex = self
            .regex_cache
            .entry(regex_pattern.clone())
            .or_insert_with(|| Regex::new(&regex_pattern).expect("Invalid permission pattern"));

        regex.is_match(required)
    }

    fn role_has_permission(&self, role: &Role, required: &str) -> bool {
        match role {
            Role::Admin => true,
            Role::User => {
                required.starts_with("repo:read")
                    || required.starts_with("session:read")
                    || required.starts_with("session:create")
            }
            Role::Guest => required.starts_with("repo:read"),
            Role::Custom(_) => false,
        }
    }
}
