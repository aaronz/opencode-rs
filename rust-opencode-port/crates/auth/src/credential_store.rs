use aes_gcm::aead::{Aead, KeyInit};
use aes_gcm::{Aes256Gcm, Nonce};
use base64::{engine::general_purpose::STANDARD as BASE64, Engine as _};
use opencode_core::OpenCodeError;
use rand::RngCore;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;

const ARGON2_SALT_LEN: usize = 16;
const ARGON2_KEY_LEN: usize = 32;
const ARGON2_MEMORY_KB: u32 = 65536;
const ARGON2_ITERATIONS: u32 = 3;
const ARGON2_PARALLELISM: u32 = 4;

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
    salt: Option<String>,
}

pub struct CredentialStore {
    store_path: PathBuf,
    key_path: PathBuf,
    password: Option<String>,
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
            password: None,
        }
    }

    pub fn with_password(password: String) -> Self {
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
            password: Some(password),
        }
    }

    pub fn with_paths(store_path: PathBuf, key_path: PathBuf) -> Self {
        Self {
            store_path,
            key_path,
            password: None,
        }
    }

    pub fn with_paths_and_password(
        store_path: PathBuf,
        key_path: PathBuf,
        password: String,
    ) -> Self {
        Self {
            store_path,
            key_path,
            password: Some(password),
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

    fn derive_key(&self, salt: &[u8]) -> Result<[u8; ARGON2_KEY_LEN], OpenCodeError> {
        let password = self.password.as_ref().ok_or_else(|| {
            OpenCodeError::Storage("Password not set for key derivation".to_string())
        })?;

        let params = argon2::Params::new(
            ARGON2_MEMORY_KB,
            ARGON2_ITERATIONS,
            ARGON2_PARALLELISM,
            Some(ARGON2_KEY_LEN),
        )
        .map_err(|e| OpenCodeError::Storage(format!("Invalid Argon2 params: {}", e)))?;

        let argon2 =
            argon2::Argon2::new(argon2::Algorithm::Argon2id, argon2::Version::V0x13, params);

        let mut key = [0u8; ARGON2_KEY_LEN];
        argon2
            .hash_password_into(password.as_bytes(), salt, &mut key)
            .map_err(|e| OpenCodeError::Storage(format!("Argon2 key derivation failed: {}", e)))?;

        Ok(key)
    }

    fn save_all(&self, credentials: &HashMap<String, Credential>) -> Result<(), OpenCodeError> {
        if let Some(parent) = self.store_path.parent() {
            std::fs::create_dir_all(parent)?;
        }

        let plaintext = serde_json::to_vec(credentials).map_err(|e| {
            OpenCodeError::Storage(format!("Failed to serialize credentials: {}", e))
        })?;

        let (key, salt) = if self.password.is_some() {
            if self.store_path.exists() {
                if let Ok(existing_salt) = self.load_salt() {
                    let key = self.derive_key(&existing_salt)?;
                    (key, Some(existing_salt))
                } else {
                    let salt = self.generate_salt()?;
                    let key = self.derive_key(&salt)?;
                    (key, Some(salt))
                }
            } else {
                let salt = self.generate_salt()?;
                let key = self.derive_key(&salt)?;
                (key, Some(salt))
            }
        } else {
            (self.get_or_create_key()?, None)
        };

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
            salt: salt.map(|s| BASE64.encode(s)),
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

        let key = if self.password.is_some() {
            if let Some(salt_b64) = payload.salt {
                let salt = BASE64
                    .decode(&salt_b64)
                    .map_err(|e| OpenCodeError::Storage(format!("Failed to decode salt: {}", e)))?;
                self.derive_key(&salt)?
            } else {
                return Err(OpenCodeError::Storage(
                    "Salt not found for password-protected store".to_string(),
                ));
            }
        } else {
            self.get_or_create_key()?
        };

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

    fn generate_salt(&self) -> Result<[u8; ARGON2_SALT_LEN], OpenCodeError> {
        let mut salt = [0u8; ARGON2_SALT_LEN];
        rand::rngs::OsRng.fill_bytes(&mut salt);
        Ok(salt)
    }

    fn load_salt(&self) -> Result<[u8; ARGON2_SALT_LEN], OpenCodeError> {
        let body = std::fs::read_to_string(&self.store_path)?;
        let payload: EncryptedPayload = serde_json::from_str(&body)
            .map_err(|e| OpenCodeError::Storage(format!("Failed to parse payload: {}", e)))?;

        if let Some(salt_b64) = payload.salt {
            let salt = BASE64
                .decode(&salt_b64)
                .map_err(|e| OpenCodeError::Storage(format!("Failed to decode salt: {}", e)))?;
            if salt.len() != ARGON2_SALT_LEN {
                return Err(OpenCodeError::Storage("Invalid salt length".to_string()));
            }
            let mut out = [0u8; ARGON2_SALT_LEN];
            out.copy_from_slice(&salt);
            Ok(out)
        } else {
            Err(OpenCodeError::Storage("Salt not found".to_string()))
        }
    }
}

fn decode_key(encoded: &str) -> Result<[u8; 32], OpenCodeError> {
    let bytes = BASE64
        .decode(encoded)
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

    #[test]
    fn stores_and_loads_credential_with_password() {
        let tmp = tempfile::tempdir().unwrap();
        let store = CredentialStore::with_paths_and_password(
            tmp.path().join("credentials.enc.json"),
            tmp.path().join("credentials.key"),
            "my-secret-password".to_string(),
        );

        let credential = Credential {
            api_key: "sk-password-test".to_string(),
            base_url: Some("https://api.example.com".to_string()),
            metadata: HashMap::from([("env".to_string(), "prod".to_string())]),
        };

        store.store("provider1", &credential).unwrap();
        let loaded = store.load("provider1").unwrap();

        assert_eq!(loaded, Some(credential));
    }

    #[test]
    fn password_stores_have_salt() {
        let tmp = tempfile::tempdir().unwrap();
        let store = CredentialStore::with_paths_and_password(
            tmp.path().join("credentials.enc.json"),
            tmp.path().join("credentials.key"),
            "password123".to_string(),
        );

        store
            .store(
                "test",
                &Credential {
                    api_key: "sk-key".to_string(),
                    base_url: None,
                    metadata: HashMap::new(),
                },
            )
            .unwrap();

        let raw = std::fs::read_to_string(tmp.path().join("credentials.enc.json")).unwrap();
        assert!(raw.contains("salt"));
    }

    #[test]
    fn password_must_be_correct_to_decrypt() {
        let tmp = tempfile::tempdir().unwrap();
        let store = CredentialStore::with_paths_and_password(
            tmp.path().join("credentials.enc.json"),
            tmp.path().join("credentials.key"),
            "correct-password".to_string(),
        );

        store
            .store(
                "secret",
                &Credential {
                    api_key: "sk-super-secret".to_string(),
                    base_url: None,
                    metadata: HashMap::new(),
                },
            )
            .unwrap();

        drop(store);

        let wrong_password_store = CredentialStore::with_paths_and_password(
            tmp.path().join("credentials.enc.json"),
            tmp.path().join("credentials.key"),
            "wrong-password".to_string(),
        );

        let result = wrong_password_store.load("secret");
        assert!(result.is_err());
    }

    #[test]
    fn non_password_store_has_no_salt() {
        let tmp = tempfile::tempdir().unwrap();
        let store = test_store(&tmp);

        store
            .store(
                "test",
                &Credential {
                    api_key: "sk-key".to_string(),
                    base_url: None,
                    metadata: HashMap::new(),
                },
            )
            .unwrap();

        let raw = std::fs::read_to_string(tmp.path().join("credentials.enc.json")).unwrap();
        assert!(raw.contains("null") || !raw.contains("salt"));
    }

    #[test]
    fn argon2_derives_consistent_key() {
        let tmp = tempfile::tempdir().unwrap();
        let password = "test-password-123";

        let store1 = CredentialStore::with_paths_and_password(
            tmp.path().join("store1.json"),
            tmp.path().join("key1"),
            password.to_string(),
        );

        let store2 = CredentialStore::with_paths_and_password(
            tmp.path().join("store2.json"),
            tmp.path().join("key2"),
            password.to_string(),
        );

        let credential = Credential {
            api_key: "same-key".to_string(),
            base_url: None,
            metadata: HashMap::new(),
        };

        store1.store("test", &credential).unwrap();

        std::fs::copy(
            tmp.path().join("store1.json"),
            tmp.path().join("store2.json"),
        )
        .unwrap();

        let loaded1 = store1.load("test").unwrap();
        let loaded2 = store2.load("test").unwrap();

        assert_eq!(loaded1, loaded2);
    }
}
