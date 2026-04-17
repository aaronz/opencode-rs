use crate::models::{Role, UserPermissions};
use crate::sensitive_file::{check_sensitive, is_external_directory};
use regex::Regex;
use std::collections::HashMap;
use std::path::Path;

pub struct PermissionEvaluator {
    regex_cache: HashMap<String, Regex>,
    /// Optional allowed base directory for external directory checks.
    allowed_base: Option<String>,
}

impl Default for PermissionEvaluator {
    fn default() -> Self {
        Self::new()
    }
}

impl PermissionEvaluator {
    pub fn new() -> Self {
        Self {
            regex_cache: HashMap::new(),
            allowed_base: None,
        }
    }

    pub fn with_allowed_base(mut self, base: String) -> Self {
        self.allowed_base = Some(base);
        self
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

        #[expect(clippy::expect_used)]
        let regex = self
            .regex_cache
            .entry(regex_pattern.clone())
            .or_insert_with(|| {
                Regex::new(&regex_pattern).expect("permission pattern regex is valid")
            });

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

    pub fn check_file_path(&self, path: &str) -> FilePermissionResult {
        let path = Path::new(path);

        if check_sensitive(path).is_sensitive {
            return FilePermissionResult::Denied {
                reason: "Sensitive file pattern matched".to_string(),
                can_override: true,
            };
        }

        if let Some(ref base) = self.allowed_base {
            if is_external_directory(path, base.as_str()) {
                return FilePermissionResult::Denied {
                    reason: format!("Path is outside allowed directory: {}", base),
                    can_override: true,
                };
            }
        }

        FilePermissionResult::Allowed
    }
}

#[derive(Debug, Clone)]
pub enum FilePermissionResult {
    Allowed,
    Denied { reason: String, can_override: bool },
}

impl FilePermissionResult {
    pub fn is_allowed(&self) -> bool {
        matches!(self, FilePermissionResult::Allowed)
    }

    pub fn is_denied(&self) -> bool {
        matches!(self, FilePermissionResult::Denied { .. })
    }

    pub fn denied_reason(&self) -> Option<&str> {
        match self {
            FilePermissionResult::Denied { reason, .. } => Some(reason),
            _ => None,
        }
    }

    pub fn can_override(&self) -> bool {
        match self {
            FilePermissionResult::Denied { can_override, .. } => *can_override,
            _ => false,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_env_file_denied_by_default() {
        let evaluator = PermissionEvaluator::new();

        assert!(evaluator.check_file_path(".env").is_denied());
        assert!(evaluator.check_file_path(".env.local").is_denied());
        assert!(evaluator.check_file_path(".env.production").is_denied());
        assert!(evaluator.check_file_path("/path/to/.env").is_denied());

        let result = evaluator.check_file_path(".env");
        assert!(result.can_override());
        assert!(result.denied_reason().is_some());
    }

    #[test]
    fn test_credentials_file_denied_by_default() {
        let evaluator = PermissionEvaluator::new();

        assert!(evaluator.check_file_path("credentials.json").is_denied());
        assert!(evaluator.check_file_path("credentials.yaml").is_denied());
        assert!(evaluator
            .check_file_path("/home/user/credentials.json")
            .is_denied());
    }

    #[test]
    fn test_pem_files_denied_by_default() {
        let evaluator = PermissionEvaluator::new();

        assert!(evaluator.check_file_path("server.pem").is_denied());
        assert!(evaluator.check_file_path("private.key").is_denied());
        assert!(evaluator
            .check_file_path("/etc/ssl/certs/cert.crt")
            .is_denied());
        assert!(evaluator.check_file_path("keystore.jks").is_denied());
    }

    #[test]
    fn test_secrets_files_denied_by_default() {
        let evaluator = PermissionEvaluator::new();

        assert!(evaluator.check_file_path("secrets.json").is_denied());
        assert!(evaluator.check_file_path("secret.token").is_denied());
        assert!(evaluator.check_file_path(".secrets").is_denied());
    }

    #[test]
    fn test_normal_files_allowed() {
        let evaluator = PermissionEvaluator::new();

        assert!(evaluator.check_file_path("source_code.rs").is_allowed());
        assert!(evaluator.check_file_path("README.md").is_allowed());
        assert!(evaluator.check_file_path("src/main.rs").is_allowed());
        assert!(evaluator.check_file_path("Cargo.toml").is_allowed());
    }

    #[test]
    fn test_aws_credentials_denied() {
        let evaluator = PermissionEvaluator::new();

        assert!(evaluator
            .check_file_path("/home/user/.aws/credentials")
            .is_denied());
        assert!(evaluator.check_file_path(".aws/config").is_denied());
    }

    #[test]
    fn test_external_directory_denied() {
        let evaluator = PermissionEvaluator::new().with_allowed_base("/home/project".to_string());

        assert!(evaluator.check_file_path("/etc/passwd").is_denied());
        assert!(evaluator
            .check_file_path("/home/other_project/secret.txt")
            .is_denied());
    }

    #[test]
    fn test_path_within_allowed_base_allowed() {
        let evaluator = PermissionEvaluator::new().with_allowed_base("/home/project".to_string());

        assert!(evaluator
            .check_file_path("/home/project/src/main.rs")
            .is_allowed());
        assert!(evaluator.check_file_path("/home/project/.env").is_denied());
    }

    #[test]
    fn test_denied_can_be_overridden() {
        let evaluator = PermissionEvaluator::new();
        let result = evaluator.check_file_path(".env");
        assert!(result.is_denied());
        assert!(result.can_override());
    }

    #[test]
    fn test_file_permission_result_helpers() {
        let denied = FilePermissionResult::Denied {
            reason: "test".to_string(),
            can_override: true,
        };
        assert!(denied.is_denied());
        assert!(!denied.is_allowed());
        assert_eq!(denied.denied_reason(), Some("test"));
        assert!(denied.can_override());

        let allowed = FilePermissionResult::Allowed;
        assert!(allowed.is_allowed());
        assert!(!allowed.is_denied());
        assert_eq!(allowed.denied_reason(), None);
        assert!(!allowed.can_override());
    }
}
