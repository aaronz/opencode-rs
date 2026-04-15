use aes_gcm::{
    aead::{Aead, KeyInit},
    Aes256Gcm, Nonce,
};
use argon2::Argon2;
use chrono::{DateTime, Utc};
use rand::RngCore;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Mutex;
use std::sync::OnceLock;

static CREDENTIAL_STORE: OnceLock<Mutex<HashMap<String, StoredCredential>>> = OnceLock::new();
static ENCRYPTION_KEY: OnceLock<[u8; 32]> = OnceLock::new();

const NONCE_SIZE: usize = 12;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EncryptedData {
    pub nonce: Vec<u8>,
    pub ciphertext: Vec<u8>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StoredCredential {
    pub id: String,
    pub provider_id: String,
    pub encrypted_api_key: EncryptedData,
    pub created_at: DateTime<Utc>,
    pub expires_at: Option<DateTime<Utc>>,
    pub metadata: Option<HashMap<String, String>>,
    pub last_rotated_at: Option<DateTime<Utc>>,
}

#[derive(Debug, thiserror::Error)]
pub enum CryptoError {
    #[error("key derivation failed: {0}")]
    KeyDerivation(String),
    #[error("encryption failed: {0}")]
    Encryption(String),
    #[error("decryption failed: {0}")]
    Decryption(String),
    #[error("invalid key length")]
    InvalidKeyLength,
    #[error("master password not set")]
    MasterPasswordNotSet,
    #[error("credential not found: {0}")]
    CredentialNotFound(String),
}

fn get_encryption_key() -> Result<&'static [u8; 32], CryptoError> {
    ENCRYPTION_KEY
        .get()
        .ok_or(CryptoError::MasterPasswordNotSet)
}

pub fn set_master_password(password: &str) -> Result<(), CryptoError> {
    let salt = b"opencode-credential-store-v1";
    let mut key = [0u8; 32];

    Argon2::default()
        .hash_password_into(password.as_bytes(), salt, &mut key)
        .map_err(|e| CryptoError::KeyDerivation(e.to_string()))?;

    ENCRYPTION_KEY
        .set(key)
        .map_err(|_| CryptoError::KeyDerivation("key already set".to_string()))?;
    Ok(())
}

pub fn has_master_password() -> bool {
    ENCRYPTION_KEY.get().is_some()
}

fn derive_key_from_password(password: &str, salt: &[u8]) -> Result<[u8; 32], CryptoError> {
    let mut key = [0u8; 32];
    Argon2::default()
        .hash_password_into(password.as_bytes(), salt, &mut key)
        .map_err(|e| CryptoError::KeyDerivation(e.to_string()))?;
    Ok(key)
}

pub fn encrypt_data(plaintext: &[u8], key: &[u8; 32]) -> Result<EncryptedData, CryptoError> {
    let cipher =
        Aes256Gcm::new_from_slice(key).map_err(|e| CryptoError::Encryption(e.to_string()))?;

    let mut nonce_bytes = [0u8; NONCE_SIZE];
    rand::thread_rng().fill_bytes(&mut nonce_bytes);
    let nonce = Nonce::from_slice(&nonce_bytes);

    let ciphertext = cipher
        .encrypt(nonce, plaintext)
        .map_err(|e| CryptoError::Encryption(e.to_string()))?;

    Ok(EncryptedData {
        nonce: nonce_bytes.to_vec(),
        ciphertext,
    })
}

pub fn decrypt_data(encrypted: &EncryptedData, key: &[u8; 32]) -> Result<Vec<u8>, CryptoError> {
    let cipher =
        Aes256Gcm::new_from_slice(key).map_err(|e| CryptoError::Decryption(e.to_string()))?;

    if encrypted.nonce.len() != NONCE_SIZE {
        return Err(CryptoError::Decryption("invalid nonce length".to_string()));
    }

    let nonce = Nonce::from_slice(&encrypted.nonce);

    cipher
        .decrypt(nonce, encrypted.ciphertext.as_ref())
        .map_err(|e| CryptoError::Decryption(e.to_string()))
}

pub fn get_credential_store() -> &'static Mutex<HashMap<String, StoredCredential>> {
    CREDENTIAL_STORE.get_or_init(|| Mutex::new(HashMap::new()))
}

pub fn resolve_credential_ref(credential_ref: &str) -> Option<String> {
    let store = get_credential_store();
    let store = store
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());

    if let Some(cred) = store.get(credential_ref) {
        let key = match get_encryption_key() {
            Ok(k) => k,
            Err(_) => return None,
        };

        decrypt_data(&cred.encrypted_api_key, key)
            .ok()
            .and_then(|decrypted| String::from_utf8(decrypted).ok())
    } else {
        None
    }
}

pub fn store_credential(
    provider_id: String,
    api_key: String,
    expires_at: Option<DateTime<Utc>>,
) -> Result<String, CryptoError> {
    let key = get_encryption_key()?;

    let encrypted_api_key = encrypt_data(api_key.as_bytes(), key)?;

    let credential_id = format!("cred-{}", uuid::Uuid::new_v4());
    let stored = StoredCredential {
        id: credential_id.clone(),
        provider_id,
        encrypted_api_key,
        created_at: Utc::now(),
        expires_at,
        metadata: None,
        last_rotated_at: None,
    };

    let store = get_credential_store();
    let mut store = store
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
    store.insert(credential_id.clone(), stored);

    Ok(credential_id)
}

pub fn store_credential_with_password(
    provider_id: String,
    api_key: String,
    expires_at: Option<DateTime<Utc>>,
    master_password: &str,
) -> Result<String, CryptoError> {
    let salt = format!("opencode-{}-v1", provider_id).into_bytes();
    let key = derive_key_from_password(master_password, &salt)?;

    let encrypted_api_key = encrypt_data(api_key.as_bytes(), &key)?;

    let credential_id = format!("cred-{}", uuid::Uuid::new_v4());
    let stored = StoredCredential {
        id: credential_id.clone(),
        provider_id,
        encrypted_api_key,
        created_at: Utc::now(),
        expires_at,
        metadata: None,
        last_rotated_at: None,
    };

    let store = get_credential_store();
    let mut store = store
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
    store.insert(credential_id.clone(), stored);

    Ok(credential_id)
}

pub fn rotate_credential(credential_id: &str, new_api_key: String) -> Result<bool, CryptoError> {
    let key = get_encryption_key()?;

    let encrypted_api_key = encrypt_data(new_api_key.as_bytes(), key)?;

    let store = get_credential_store();
    let mut store = store
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());

    if let Some(cred) = store.get_mut(credential_id) {
        cred.encrypted_api_key = encrypted_api_key;
        cred.last_rotated_at = Some(Utc::now());
        Ok(true)
    } else {
        Ok(false)
    }
}

pub fn rotate_credential_with_password(
    credential_id: &str,
    new_api_key: String,
    master_password: &str,
) -> Result<bool, CryptoError> {
    let store = get_credential_store();
    let store_guard = store
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());

    let provider_id = if let Some(cred) = store_guard.get(credential_id) {
        cred.provider_id.clone()
    } else {
        return Ok(false);
    };
    drop(store_guard);

    let salt = format!("opencode-{}-v1", provider_id).into_bytes();
    let key = derive_key_from_password(master_password, &salt)?;

    let encrypted_api_key = encrypt_data(new_api_key.as_bytes(), &key)?;

    let mut store = store
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
    if let Some(cred) = store.get_mut(credential_id) {
        cred.encrypted_api_key = encrypted_api_key;
        cred.last_rotated_at = Some(Utc::now());
        Ok(true)
    } else {
        Ok(false)
    }
}

pub fn delete_credential(credential_id: &str) -> bool {
    let store = get_credential_store();
    let mut store = store
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
    store.remove(credential_id).is_some()
}

pub fn list_credentials() -> Vec<String> {
    let store = get_credential_store();
    let store = store
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
    store.keys().cloned().collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn init_test_key() {
        let _ = ENCRYPTION_KEY.set([0u8; 32]);
    }

    #[test]
    fn test_encrypt_decrypt_roundtrip() {
        init_test_key();
        let key = get_encryption_key().unwrap();

        let plaintext = b"sk-test-api-key-12345";
        let encrypted = encrypt_data(plaintext, key).unwrap();

        let decrypted = decrypt_data(&encrypted, key).unwrap();
        assert_eq!(decrypted, plaintext);
    }

    #[test]
    fn test_encrypt_decrypt_with_different_nonces() {
        init_test_key();
        let key = get_encryption_key().unwrap();

        let plaintext = b"test-key";
        let encrypted1 = encrypt_data(plaintext, key).unwrap();
        let encrypted2 = encrypt_data(plaintext, key).unwrap();

        assert_ne!(encrypted1.nonce, encrypted2.nonce);
        assert_ne!(encrypted1.ciphertext, encrypted2.ciphertext);

        let decrypted1 = decrypt_data(&encrypted1, key).unwrap();
        let decrypted2 = decrypt_data(&encrypted2, key).unwrap();
        assert_eq!(decrypted1, decrypted2);
    }

    #[test]
    fn test_wrong_key_fails_decryption() {
        init_test_key();
        let key1 = get_encryption_key().unwrap();
        let mut key2 = [0u8; 32];
        key2.copy_from_slice(b"wrong-key-that-is-32-bytes-long!!");

        let plaintext = b"secret-api-key";
        let encrypted = encrypt_data(plaintext, key1).unwrap();

        let result = decrypt_data(&encrypted, &key2);
        assert!(result.is_err());
    }

    #[test]
    fn test_store_and_resolve_credential() {
        init_test_key();

        let cred_id =
            store_credential("test-provider".to_string(), "test-key".to_string(), None).unwrap();

        let resolved = resolve_credential_ref(&cred_id);
        assert_eq!(resolved, Some("test-key".to_string()));
    }

    #[test]
    fn test_rotate_credential() {
        init_test_key();

        let cred_id =
            store_credential("test-provider".to_string(), "old-key".to_string(), None).unwrap();

        let success = rotate_credential(&cred_id, "new-key".to_string()).unwrap();
        assert!(success);

        let resolved = resolve_credential_ref(&cred_id);
        assert_eq!(resolved, Some("new-key".to_string()));
    }

    #[test]
    fn test_rotate_nonexistent_credential() {
        init_test_key();

        let result = rotate_credential("nonexistent", "new-key".to_string());
        assert!(result.is_ok());
        assert!(result.unwrap());
    }

    #[test]
    fn test_store_credential_with_password() {
        let cred_id = store_credential_with_password(
            "test-provider".to_string(),
            "password-protected-key".to_string(),
            None,
            "master-password-123",
        )
        .unwrap();

        let store = get_credential_store();
        let store = store.lock().unwrap();
        assert!(store.contains_key(&cred_id));
    }

    #[test]
    fn test_delete_credential() {
        init_test_key();

        let cred_id = store_credential(
            "test-provider".to_string(),
            "key-to-delete".to_string(),
            None,
        )
        .unwrap();

        assert!(delete_credential(&cred_id));
        assert!(!delete_credential(&cred_id));

        let resolved = resolve_credential_ref(&cred_id);
        assert_eq!(resolved, None);
    }

    #[test]
    fn test_list_credentials() {
        init_test_key();

        let store = get_credential_store();
        *store.lock().unwrap() = HashMap::new();

        let cred_id1 = store_credential("provider1".to_string(), "key1".to_string(), None).unwrap();
        let cred_id2 = store_credential("provider2".to_string(), "key2".to_string(), None).unwrap();

        let creds = list_credentials();
        assert!(creds.contains(&cred_id1));
        assert!(creds.contains(&cred_id2));
        assert_eq!(creds.len(), 2);
    }

    #[test]
    fn test_encrypted_data_serialization() {
        init_test_key();
        let key = get_encryption_key().unwrap();

        let plaintext = b"serialization test";
        let encrypted = encrypt_data(plaintext, key).unwrap();

        let json = serde_json::to_string(&encrypted).unwrap();
        let deserialized: EncryptedData = serde_json::from_str(&json).unwrap();

        let decrypted = decrypt_data(&deserialized, key).unwrap();
        assert_eq!(decrypted, plaintext);
    }
}
