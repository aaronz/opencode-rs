use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Mutex;
use std::sync::OnceLock;

static CREDENTIAL_STORE: OnceLock<Mutex<HashMap<String, StoredCredential>>> = OnceLock::new();

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StoredCredential {
    pub id: String,
    pub provider_id: String,
    pub encrypted_api_key: String,
    pub created_at: DateTime<Utc>,
    pub expires_at: Option<DateTime<Utc>>,
    pub metadata: Option<HashMap<String, String>>,
    pub last_rotated_at: Option<DateTime<Utc>>,
}

pub fn get_credential_store() -> &'static Mutex<HashMap<String, StoredCredential>> {
    CREDENTIAL_STORE.get_or_init(|| Mutex::new(HashMap::new()))
}

pub fn resolve_credential_ref(credential_ref: &str) -> Option<String> {
    let store = get_credential_store();
    let store = store.lock().unwrap();
    store
        .get(credential_ref)
        .map(|c| c.encrypted_api_key.clone())
}

pub fn store_credential(
    provider_id: String,
    api_key: String,
    expires_at: Option<DateTime<Utc>>,
) -> String {
    let credential_id = format!("cred-{}", uuid::Uuid::new_v4());
    let stored = StoredCredential {
        id: credential_id.clone(),
        provider_id,
        encrypted_api_key: api_key,
        created_at: Utc::now(),
        expires_at,
        metadata: None,
        last_rotated_at: None,
    };

    let store = get_credential_store();
    let mut store = store.lock().unwrap();
    store.insert(credential_id.clone(), stored);
    credential_id
}

pub fn rotate_credential(credential_id: &str, new_api_key: String) -> bool {
    let store = get_credential_store();
    let mut store = store.lock().unwrap();

    if let Some(cred) = store.get_mut(credential_id) {
        cred.encrypted_api_key = new_api_key;
        cred.last_rotated_at = Some(Utc::now());
        true
    } else {
        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_store_and_resolve_credential() {
        let cred_id = store_credential("test-provider".to_string(), "test-key".to_string(), None);
        let resolved = resolve_credential_ref(&cred_id);
        assert_eq!(resolved, Some("test-key".to_string()));
    }

    #[test]
    fn test_rotate_credential() {
        let cred_id = store_credential("test-provider".to_string(), "old-key".to_string(), None);
        let success = rotate_credential(&cred_id, "new-key".to_string());
        assert!(success);

        let resolved = resolve_credential_ref(&cred_id);
        assert_eq!(resolved, Some("new-key".to_string()));
    }

    #[test]
    fn test_rotate_nonexistent_credential() {
        let success = rotate_credential("nonexistent", "new-key".to_string());
        assert!(!success);
    }
}
