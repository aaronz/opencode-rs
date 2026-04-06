use serde::{Deserialize, Serialize};
use std::collections::HashSet;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AccessControlResult {
    Allowed,
    Denied(String),
    ProviderNotFound(String),
}

pub struct RuntimeAccessControl {
    provider_allowlist: Option<HashSet<String>>,
    provider_denylist: HashSet<String>,
    server_basic_auth_enabled: bool,
    mcp_token_required: bool,
}

impl RuntimeAccessControl {
    pub fn new() -> Self {
        Self {
            provider_allowlist: None,
            provider_denylist: HashSet::new(),
            server_basic_auth_enabled: false,
            mcp_token_required: false,
        }
    }

    pub fn with_allowlist(mut self, providers: HashSet<String>) -> Self {
        self.provider_allowlist = Some(providers);
        self
    }

    pub fn with_denylist(mut self, providers: HashSet<String>) -> Self {
        self.provider_denylist = providers;
        self
    }

    pub fn with_server_basic_auth(mut self, enabled: bool) -> Self {
        self.server_basic_auth_enabled = enabled;
        self
    }

    pub fn with_mcp_token_required(mut self, required: bool) -> Self {
        self.mcp_token_required = required;
        self
    }

    pub fn check_provider_access(&self, provider_id: &str) -> AccessControlResult {
        if self.provider_denylist.contains(provider_id) {
            return AccessControlResult::Denied(format!("Provider '{}' is disabled", provider_id));
        }

        if let Some(allowlist) = &self.provider_allowlist {
            if !allowlist.contains(provider_id) {
                return AccessControlResult::Denied(format!(
                    "Provider '{}' is not in the allowlist",
                    provider_id
                ));
            }
        }

        AccessControlResult::Allowed
    }

    pub fn is_server_auth_required(&self) -> bool {
        self.server_basic_auth_enabled
    }

    pub fn is_mcp_token_required(&self) -> bool {
        self.mcp_token_required
    }
}

impl Default for RuntimeAccessControl {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_allows_all_providers() {
        let acl = RuntimeAccessControl::new();
        assert!(matches!(
            acl.check_provider_access("openai"),
            AccessControlResult::Allowed
        ));
    }

    #[test]
    fn test_denylist_blocks_provider() {
        let mut denylist = HashSet::new();
        denylist.insert("ollama".to_string());
        let acl = RuntimeAccessControl::new().with_denylist(denylist);

        assert!(matches!(
            acl.check_provider_access("ollama"),
            AccessControlResult::Denied(_)
        ));
        assert!(matches!(
            acl.check_provider_access("openai"),
            AccessControlResult::Allowed
        ));
    }

    #[test]
    fn test_allowlist_blocks_unlisted_provider() {
        let mut allowlist = HashSet::new();
        allowlist.insert("openai".to_string());
        allowlist.insert("anthropic".to_string());
        let acl = RuntimeAccessControl::new().with_allowlist(allowlist);

        assert!(matches!(
            acl.check_provider_access("openai"),
            AccessControlResult::Allowed
        ));
        assert!(matches!(
            acl.check_provider_access("ollama"),
            AccessControlResult::Denied(_)
        ));
    }

    #[test]
    fn test_denylist_takes_precedence_over_allowlist() {
        let mut allowlist = HashSet::new();
        allowlist.insert("openai".to_string());
        let mut denylist = HashSet::new();
        denylist.insert("openai".to_string());

        let acl = RuntimeAccessControl::new()
            .with_allowlist(allowlist)
            .with_denylist(denylist);

        assert!(matches!(
            acl.check_provider_access("openai"),
            AccessControlResult::Denied(_)
        ));
    }

    #[test]
    fn test_server_auth_flag() {
        let acl = RuntimeAccessControl::new().with_server_basic_auth(true);
        assert!(acl.is_server_auth_required());

        let acl2 = RuntimeAccessControl::new().with_server_basic_auth(false);
        assert!(!acl2.is_server_auth_required());
    }
}
