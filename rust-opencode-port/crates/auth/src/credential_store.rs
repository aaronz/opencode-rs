use aes_gcm::aead::{Aead, KeyInit};
use aes_gcm::{Aes256Gcm, Nonce};
use base64::{engine::general_purpose::STANDARD as BASE64, Engine as _};
use opencode_core::OpenCodeError;
use rand::RngCore;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::HashMap;
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Credential {
    pub api_key: String,
    pub base_url: Option<String>,
    #[serde(default)]
    pub metadata: HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct EncryptedPayload {
    nonce: String,
    ciphertext: String,
}

pub struct CredentialStore {
    store_path: PathBuf,
    key_path: PathBuf,
}

impl Default for CredentialStore {
    fn default() -> Self {
        Self::new()
    }
}

impl CredentialStore {
    pub fn new() -> Self {
        let base_dir = std::env::var("OPENCODE_DATA_DIR")
            .map(PathBuf::from)
            .unwrap_or_else(|_| {
                dirs::home_dir()
                    .map(|h| h.join(".config/opencode-rs"))
                    .unwrap_or_else(|| PathBuf::from(".opencode-rs"))
            });

        Self {
            store_path: base_dir.join("credentials.enc.json"),
            key_path: base_dir.join("credentials.key"),
        }
    }

    pub fn with_paths(store_path: PathBuf, key_path: PathBuf) -> Self {
        Self {
            store_path,
            key_path,
        }
    }

    pub fn store(&self, provider_id: &str, credential: &Credential) -> Result<(), OpenCodeError> {
        let mut all = self.load_all()?;
        all.insert(provider_id.to_string(), credential.clone());
        self.save_all(&all)
    }

    pub fn load(&self, provider_id: &str) -> Result<Option<Credential>, OpenCodeError> {
        let all = self.load_all()?;
        Ok(all.get(provider_id).cloned())
    }

    pub fn delete(&self, provider_id: &str) -> Result<(), OpenCodeError> {
        let mut all = self.load_all()?;
        all.remove(provider_id);
        self.save_all(&all)
    }

    fn save_all(&self, credentials: &HashMap<String, Credential>) -> Result<(), OpenCodeError> {
        if let Some(parent) = self.store_path.parent() {
            std::fs::create_dir_all(parent)?;
        }

        let plaintext = serde_json::to_vec(credentials).map_err(|e| {
            OpenCodeError::Storage(format!("Failed to serialize credentials: {}", e))
        })?;
        let key = self.get_or_create_key()?;
        let cipher = Aes256Gcm::new_from_slice(&key)
            .map_err(|e| OpenCodeError::Storage(format!("Failed to initialize cipher: {}", e)))?;

        let mut nonce_bytes = [0u8; 12];
        rand::rngs::OsRng.fill_bytes(&mut nonce_bytes);
        let nonce = Nonce::from_slice(&nonce_bytes);

        let ciphertext = cipher
            .encrypt(nonce, plaintext.as_ref())
            .map_err(|e| OpenCodeError::Storage(format!("Failed to encrypt credentials: {}", e)))?;

        let payload = EncryptedPayload {
            nonce: BASE64.encode(nonce_bytes),
            ciphertext: BASE64.encode(ciphertext),
        };

        let body = serde_json::to_vec_pretty(&payload)
            .map_err(|e| OpenCodeError::Storage(format!("Failed to serialize payload: {}", e)))?;
        std::fs::write(&self.store_path, body)?;
        Ok(())
    }

    fn load_all(&self) -> Result<HashMap<String, Credential>, OpenCodeError> {
        if !self.store_path.exists() {
            return Ok(HashMap::new());
        }

        let body = std::fs::read_to_string(&self.store_path)?;
        let payload: EncryptedPayload = serde_json::from_str(&body).map_err(|e| {
            OpenCodeError::Storage(format!("Failed to parse credential payload: {}", e))
        })?;

        let nonce_bytes = BASE64
            .decode(payload.nonce)
            .map_err(|e| OpenCodeError::Storage(format!("Failed to decode nonce: {}", e)))?;
        if nonce_bytes.len() != 12 {
            return Err(OpenCodeError::Storage("Invalid nonce length".to_string()));
        }
        let ciphertext = BASE64
            .decode(payload.ciphertext)
            .map_err(|e| OpenCodeError::Storage(format!("Failed to decode ciphertext: {}", e)))?;

        let key = self.get_or_create_key()?;
        let cipher = Aes256Gcm::new_from_slice(&key)
            .map_err(|e| OpenCodeError::Storage(format!("Failed to initialize cipher: {}", e)))?;
        let nonce = Nonce::from_slice(&nonce_bytes);

        let plaintext = cipher
            .decrypt(nonce, ciphertext.as_ref())
            .map_err(|e| OpenCodeError::Storage(format!("Failed to decrypt credentials: {}", e)))?;

        serde_json::from_slice(&plaintext).map_err(|e| {
            OpenCodeError::Storage(format!("Failed to deserialize credentials: {}", e))
        })
    }

    fn get_or_create_key(&self) -> Result<[u8; 32], OpenCodeError> {
        if let Some(parent) = self.key_path.parent() {
            std::fs::create_dir_all(parent)?;
        }

        if self.key_path.exists() {
            let raw = std::fs::read_to_string(&self.key_path)?;
            let key = decode_key(raw.trim())?;
            return Ok(key);
        }

        let mut key = [0u8; 32];
        rand::rngs::OsRng.fill_bytes(&mut key);
        std::fs::write(&self.key_path, BASE64.encode(key))?;
        Ok(key)
    }
}

fn decode_key(encoded: &str) -> Result<[u8; 32], OpenCodeError> {
    let bytes = BASE64
        .decode(encoded)
        .or_else(|_| {
            let digest = Sha256::digest(encoded.as_bytes());
            Ok::<Vec<u8>, base64::DecodeError>(digest.to_vec())
        })
        .map_err(|e| OpenCodeError::Storage(format!("Invalid credential key format: {}", e)))?;

    if bytes.len() < 32 {
        return Err(OpenCodeError::Storage(
            "Credential key must be at least 32 bytes".to_string(),
        ));
    }

    let mut out = [0u8; 32];
    out.copy_from_slice(&bytes[..32]);
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_store(tmp: &tempfile::TempDir) -> CredentialStore {
        CredentialStore::with_paths(
            tmp.path().join("credentials.enc.json"),
            tmp.path().join("credentials.key"),
        )
    }

    #[test]
    fn stores_and_loads_credential() {
        let tmp = tempfile::tempdir().unwrap();
        let store = test_store(&tmp);

        let credential = Credential {
            api_key: "sk-test-1".to_string(),
            base_url: Some("https://example.com".to_string()),
            metadata: HashMap::from([("scope".to_string(), "dev".to_string())]),
        };

        store.store("openai", &credential).unwrap();
        let loaded = store.load("openai").unwrap();

        assert_eq!(loaded, Some(credential));
    }

    #[test]
    fn delete_removes_credential() {
        let tmp = tempfile::tempdir().unwrap();
        let store = test_store(&tmp);

        store
            .store(
                "openai",
                &Credential {
                    api_key: "sk-test-2".to_string(),
                    base_url: None,
                    metadata: HashMap::new(),
                },
            )
            .unwrap();
        store.delete("openai").unwrap();

        assert_eq!(store.load("openai").unwrap(), None);
    }

    #[test]
    fn stored_file_is_encrypted_payload() {
        let tmp = tempfile::tempdir().unwrap();
        let store = test_store(&tmp);

        store
            .store(
                "anthropic",
                &Credential {
                    api_key: "sk-live-secret".to_string(),
                    base_url: None,
                    metadata: HashMap::new(),
                },
            )
            .unwrap();

        let raw = std::fs::read_to_string(tmp.path().join("credentials.enc.json")).unwrap();
        assert!(!raw.contains("sk-live-secret"));
        assert!(raw.contains("ciphertext"));
        assert!(raw.contains("nonce"));
    }
}
