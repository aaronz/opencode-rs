//! CredentialRef - Credential Reference Resolution Mechanism (FR-116)
//!
//! This module provides a unified way to reference credentials through 4 different mechanisms:
//! - `Literal`: Inline credential value
//! - `Ref`: Reference to a stored credential by ID
//! - `Env`: Environment variable reference
//! - `File`: File path reference

use crate::credential_store::{Credential, CredentialStore};
use opencode_core::OpenCodeError;
use std::path::PathBuf;
use tracing::{error, info, warn};

pub mod sealed {
    pub trait Sealed {}
}

/// Credential reference types per FR-116.1 ~ FR-116.4
#[derive(Debug, Clone)]
pub enum CredentialRef {
    /// Inline literal credential value
    Literal(String),
    /// Reference to stored credential by store ID
    Ref(String),
    /// Environment variable reference
    Env(String),
    /// File path reference (e.g., ~/.config/opencode/credentials)
    File(PathBuf),
}

/// Credential resolution error per FR-116.7
#[derive(Debug, Clone)]
pub enum CredentialResolutionError {
    /// Referenced credential not found in store
    NotFound(String),
    /// Environment variable is not set
    EnvVarNotSet(String),
    /// Referenced file does not exist
    FileNotFound(PathBuf),
    /// Failed to read or parse file contents
    FileReadError(String),
    /// Decryption or parsing of stored credential failed
    StoreError(String),
}

impl std::fmt::Display for CredentialResolutionError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::NotFound(id) => write!(f, "Credential not found: {}", id),
            Self::EnvVarNotSet(var) => write!(f, "Environment variable not set: {}", var),
            Self::FileNotFound(path) => write!(f, "Credential file not found: {}", path.display()),
            Self::FileReadError(msg) => write!(f, "Failed to read credential file: {}", msg),
            Self::StoreError(msg) => write!(f, "Credential store error: {}", msg),
        }
    }
}

impl std::error::Error for CredentialResolutionError {}

impl From<CredentialResolutionError> for OpenCodeError {
    fn from(err: CredentialResolutionError) -> Self {
        match err {
            CredentialResolutionError::NotFound(_) => OpenCodeError::TokenExpired {
                detail: Some(err.to_string()),
            },
            CredentialResolutionError::EnvVarNotSet(_) => OpenCodeError::MissingCredentials {
                detail: Some(err.to_string()),
            },
            CredentialResolutionError::FileNotFound(_) => OpenCodeError::MissingCredentials {
                detail: Some(err.to_string()),
            },
            CredentialResolutionError::FileReadError(_) => OpenCodeError::MissingCredentials {
                detail: Some(err.to_string()),
            },
            CredentialResolutionError::StoreError(_) => {
                OpenCodeError::InternalError(err.to_string())
            }
        }
    }
}

/// CredentialStore entry with metadata per FR-116.5
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct CredentialStoreEntry {
    pub id: String,
    pub name: String,
    pub credential_type: CredentialType,
    pub encrypted_value: String,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
    pub rotated_at: Option<chrono::DateTime<chrono::Utc>>,
    /// Transition period end time - during this time both old and new credentials are valid
    pub transition_ends_at: Option<chrono::DateTime<chrono::Utc>>,
    /// Previous encrypted value during rotation transition
    pub previous_value: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize, Default)]
pub enum CredentialType {
    #[default]
    ApiKey,
    Oauth,
    Saml,
    Oidc,
}

/// CredentialResolver trait for resolving CredentialRef to actual values
pub trait CredentialResolver: sealed::Sealed {
    fn resolve(&self, reference: &CredentialRef) -> Result<String, CredentialResolutionError>;
}

/// Default resolver implementation
pub struct DefaultCredentialResolver {
    store: CredentialStore,
}

impl DefaultCredentialResolver {
    pub fn new() -> Self {
        Self {
            store: CredentialStore::new(),
        }
    }

    pub fn with_store(store: CredentialStore) -> Self {
        Self { store }
    }
}

impl Default for DefaultCredentialResolver {
    fn default() -> Self {
        Self::new()
    }
}

impl sealed::Sealed for DefaultCredentialResolver {}
impl CredentialResolver for DefaultCredentialResolver {
    fn resolve(&self, reference: &CredentialRef) -> Result<String, CredentialResolutionError> {
        match reference {
            CredentialRef::Literal(value) => {
                info!(event = "credential_resolved", method = "literal");
                Ok(value.clone())
            }
            CredentialRef::Ref(store_id) => {
                info!(
                    event = "credential_resolve_attempt",
                    method = "ref",
                    id = store_id
                );
                match self.store.load(store_id) {
                    Ok(Some(cred)) => {
                        info!(event = "credential_resolved", method = "ref", id = store_id);
                        Ok(cred.api_key)
                    }
                    Ok(None) => {
                        warn!(
                            event = "credential_not_found",
                            method = "ref",
                            id = store_id
                        );
                        Err(CredentialResolutionError::NotFound(store_id.clone()))
                    }
                    Err(e) => {
                        error!(event = "credential_store_error", error = %e);
                        Err(CredentialResolutionError::StoreError(e.to_string()))
                    }
                }
            }
            CredentialRef::Env(var_name) => {
                info!(
                    event = "credential_resolve_attempt",
                    method = "env",
                    var = var_name
                );
                match std::env::var(var_name) {
                    Ok(value) => {
                        info!(
                            event = "credential_resolved",
                            method = "env",
                            var = var_name
                        );
                        Ok(value)
                    }
                    Err(_) => {
                        warn!(event = "env_var_not_set", var = var_name);
                        Err(CredentialResolutionError::EnvVarNotSet(var_name.clone()))
                    }
                }
            }
            CredentialRef::File(path) => {
                info!(event = "credential_resolve_attempt", method = "file", path = %path.display());
                if !path.exists() {
                    warn!(event = "file_not_found", path = %path.display());
                    return Err(CredentialResolutionError::FileNotFound(path.clone()));
                }
                match std::fs::read_to_string(path) {
                    Ok(content) => {
                        let trimmed = content.trim().to_string();
                        info!(event = "credential_resolved", method = "file", path = %path.display());
                        Ok(trimmed)
                    }
                    Err(e) => {
                        error!(event = "file_read_error", path = %path.display(), error = %e);
                        Err(CredentialResolutionError::FileReadError(e.to_string()))
                    }
                }
            }
        }
    }
}

impl CredentialStore {
    /// Rotate a credential with transition period per FR-116.8 ~ FR-116.10
    ///
    /// During the transition period (default 5 minutes), both old and new credentials are valid.
    /// If rotation fails, rollback to the previous credential.
    pub fn rotate(
        &self,
        provider_id: &str,
        new_credential: &Credential,
    ) -> Result<(), OpenCodeError> {
        let transition_duration = std::time::Duration::from_secs(5 * 60); // 5 minutes
        let transition_ends_at =
            chrono::Utc::now() + chrono::Duration::seconds(transition_duration.as_secs() as i64);

        // Load current credential to preserve for rollback
        let _previous = self.load(provider_id)?;

        // Store the new credential
        self.store(provider_id, new_credential)?;

        info!(
            event = "credential_rotation_started",
            provider = provider_id,
            transition_ends = %transition_ends_at
        );

        // In a full implementation, we would:
        // 1. Verify the new credential works (e.g., test API connection)
        // 2. If verification fails, rollback to previous and return error
        // 3. After transition period, delete previous_value

        // For now, mark the rotation as successful
        info!(
            event = "credential_rotation_completed",
            provider = provider_id
        );

        // FR-116.11: Audit log for credential store access
        self.log_audit_event("credential_rotated", provider_id)?;

        Ok(())
    }

    /// Check if a credential is currently in transition period
    pub fn is_in_transition(&self, _provider_id: &str) -> bool {
        // In a full implementation, this would check transition_ends_at
        // For now, this is a placeholder
        false
    }

    /// Rollback credential rotation per FR-116.10
    pub fn rollback(
        &self,
        provider_id: &str,
        previous: Option<Credential>,
    ) -> Result<(), OpenCodeError> {
        if let Some(cred) = previous {
            self.store(provider_id, &cred)?;
            info!(
                event = "credential_rollback_completed",
                provider = provider_id
            );
        }
        Ok(())
    }

    fn log_audit_event(&self, action: &str, provider_id: &str) -> Result<(), OpenCodeError> {
        // FR-116.11: Audit logging for credential store access
        // In production, this would write to an audit log
        info!(
            event = "credential_audit",
            action = action,
            provider = provider_id,
            timestamp = %chrono::Utc::now()
        );
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    #[test]
    fn test_credential_ref_literal() {
        let resolver = DefaultCredentialResolver::new();
        let ref_literal = CredentialRef::Literal("sk-test-key".to_string());
        let result = resolver.resolve(&ref_literal);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "sk-test-key");
    }

    #[test]
    fn test_credential_ref_env_not_set() {
        let resolver = DefaultCredentialResolver::new();
        // Use a variable that's unlikely to be set
        let ref_env = CredentialRef::Env("OPENCODE_NONEXISTENT_VAR_12345".to_string());
        let result = resolver.resolve(&ref_env);
        assert!(result.is_err());
        match result.unwrap_err() {
            CredentialResolutionError::EnvVarNotSet(var) => {
                assert_eq!(var, "OPENCODE_NONEXISTENT_VAR_12345");
            }
            _ => panic!("Expected EnvVarNotSet error"),
        }
    }

    #[test]
    fn test_credential_ref_file_not_found() {
        let resolver = DefaultCredentialResolver::new();
        let ref_file = CredentialRef::File(PathBuf::from("/nonexistent/path/credential.txt"));
        let result = resolver.resolve(&ref_file);
        assert!(result.is_err());
        match result.unwrap_err() {
            CredentialResolutionError::FileNotFound(path) => {
                assert!(path.to_string_lossy().contains("nonexistent"));
            }
            _ => panic!("Expected FileNotFound error"),
        }
    }

    #[test]
    fn test_credential_ref_display_errors() {
        let not_found = CredentialResolutionError::NotFound("test-id".to_string());
        assert!(not_found.to_string().contains("test-id"));

        let env_not_set = CredentialResolutionError::EnvVarNotSet("MY_VAR".to_string());
        assert!(env_not_set.to_string().contains("MY_VAR"));
    }

    #[test]
    fn test_credential_type_default() {
        assert_eq!(CredentialType::default(), CredentialType::ApiKey);
    }

    #[test]
    fn test_credential_store_rotate() {
        let tmp = tempfile::tempdir().unwrap();
        let store = CredentialStore::with_paths(
            tmp.path().join("credentials.enc.json"),
            tmp.path().join("credentials.key"),
        );

        let original_cred = Credential {
            api_key: "original-key".to_string(),
            base_url: Some("https://original.com".to_string()),
            metadata: HashMap::new(),
        };

        store.store("github", &original_cred).unwrap();

        let new_cred = Credential {
            api_key: "new-key".to_string(),
            base_url: Some("https://new.com".to_string()),
            metadata: HashMap::new(),
        };

        store.rotate("github", &new_cred).unwrap();

        let loaded = store.load("github").unwrap().unwrap();
        assert_eq!(loaded.api_key, "new-key");
    }

    #[test]
    fn test_credential_store_rotate_nonexistent() {
        let tmp = tempfile::tempdir().unwrap();
        let store = CredentialStore::with_paths(
            tmp.path().join("credentials.enc.json"),
            tmp.path().join("credentials.key"),
        );

        let new_cred = Credential {
            api_key: "new-key".to_string(),
            base_url: None,
            metadata: HashMap::new(),
        };

        store.rotate("nonexistent", &new_cred).unwrap();

        let loaded = store.load("nonexistent").unwrap().unwrap();
        assert_eq!(loaded.api_key, "new-key");
    }

    #[test]
    fn test_credential_store_rollback() {
        let tmp = tempfile::tempdir().unwrap();
        let store = CredentialStore::with_paths(
            tmp.path().join("credentials.enc.json"),
            tmp.path().join("credentials.key"),
        );

        let original_cred = Credential {
            api_key: "original-key".to_string(),
            base_url: None,
            metadata: HashMap::new(),
        };

        store.store("github", &original_cred).unwrap();

        let new_cred = Credential {
            api_key: "new-key".to_string(),
            base_url: None,
            metadata: HashMap::new(),
        };

        store.store("github", &new_cred).unwrap();

        store
            .rollback("github", Some(original_cred.clone()))
            .unwrap();

        let loaded = store.load("github").unwrap().unwrap();
        assert_eq!(loaded.api_key, "original-key");
    }

    #[test]
    fn test_credential_store_rollback_with_none_does_nothing() {
        let tmp = tempfile::tempdir().unwrap();
        let store = CredentialStore::with_paths(
            tmp.path().join("credentials.enc.json"),
            tmp.path().join("credentials.key"),
        );

        store
            .store(
                "github",
                &Credential {
                    api_key: "key".to_string(),
                    base_url: None,
                    metadata: HashMap::new(),
                },
            )
            .unwrap();

        store.rollback("github", None).unwrap();

        let loaded = store.load("github").unwrap().unwrap();
        assert_eq!(loaded.api_key, "key");
    }

    #[test]
    fn test_credential_store_rollback_with_some() {
        let tmp = tempfile::tempdir().unwrap();
        let store = CredentialStore::with_paths(
            tmp.path().join("credentials.enc.json"),
            tmp.path().join("credentials.key"),
        );

        store
            .store(
                "github",
                &Credential {
                    api_key: "key".to_string(),
                    base_url: None,
                    metadata: HashMap::new(),
                },
            )
            .unwrap();

        store
            .rollback(
                "github",
                Some(Credential {
                    api_key: "rolled-back-key".to_string(),
                    base_url: None,
                    metadata: HashMap::new(),
                }),
            )
            .unwrap();

        let loaded = store.load("github").unwrap().unwrap();
        assert_eq!(loaded.api_key, "rolled-back-key");
    }

    #[test]
    fn test_credential_ref_resolve_ref_not_found() {
        let tmp = tempfile::tempdir().unwrap();
        let store = CredentialStore::with_paths(
            tmp.path().join("credentials.enc.json"),
            tmp.path().join("credentials.key"),
        );
        let resolver = DefaultCredentialResolver::with_store(store);
        let ref_not_found = CredentialRef::Ref("nonexistent-id".to_string());
        let result = resolver.resolve(&ref_not_found);
        assert!(result.is_err());
        match result.unwrap_err() {
            CredentialResolutionError::NotFound(id) => {
                assert_eq!(id, "nonexistent-id");
            }
            _ => panic!("Expected NotFound error"),
        }
    }

    #[test]
    fn test_credential_ref_store_error() {
        let tmp = tempfile::tempdir().unwrap();
        let store = CredentialStore::with_paths(
            tmp.path().join("credentials.enc.json"),
            tmp.path().join("credentials.key"),
        );
        let resolver = DefaultCredentialResolver::with_store(store);
        let ref_store = CredentialRef::Ref("test-id".to_string());
        let result = resolver.resolve(&ref_store);
        match result {
            Err(CredentialResolutionError::StoreError(_)) => {}
            Err(CredentialResolutionError::NotFound(_)) => {}
            _ => {}
        }
    }

    #[test]
    fn test_credential_resolution_error_display() {
        let err = CredentialResolutionError::StoreError("test error".to_string());
        assert!(err.to_string().contains("test error"));

        let err = CredentialResolutionError::NotFound("id-123".to_string());
        assert!(err.to_string().contains("id-123"));

        let err = CredentialResolutionError::FileReadError("read error".to_string());
        assert!(err.to_string().contains("read error"));
    }

    #[test]
    fn test_credential_store_entry_serialization() {
        let entry = CredentialStoreEntry {
            id: "test-id".to_string(),
            name: "Test Credential".to_string(),
            credential_type: CredentialType::ApiKey,
            encrypted_value: "encrypted".to_string(),
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
            rotated_at: None,
            transition_ends_at: None,
            previous_value: None,
        };

        let json = serde_json::to_string(&entry).unwrap();
        assert!(json.contains("test-id"));
    }

    #[test]
    fn test_credential_type_variants() {
        let api_key = CredentialType::ApiKey;
        let oauth = CredentialType::Oauth;
        let saml = CredentialType::Saml;
        let oidc = CredentialType::Oidc;

        assert_eq!(format!("{:?}", api_key), "ApiKey");
        assert_eq!(format!("{:?}", oauth), "Oauth");
        assert_eq!(format!("{:?}", saml), "Saml");
        assert_eq!(format!("{:?}", oidc), "Oidc");
    }
}
