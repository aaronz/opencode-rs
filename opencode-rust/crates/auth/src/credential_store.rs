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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NamedCredential {
    pub name: String,
    pub credential: Credential,
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
        self.store_named(provider_id, "default", credential)
    }

    pub fn store_named(
        &self,
        provider_id: &str,
        name: &str,
        credential: &Credential,
    ) -> Result<(), OpenCodeError> {
        let mut all = self.load_all()?;
        let provider_creds = all.entry(provider_id.to_string()).or_insert_with(Vec::new);

        if let Some(existing) = provider_creds.iter_mut().find(|c| c.name == name) {
            existing.credential = credential.clone();
        } else {
            provider_creds.push(NamedCredential {
                name: name.to_string(),
                credential: credential.clone(),
            });
        }

        self.save_all(&all)
    }

    pub fn load(&self, provider_id: &str) -> Result<Option<Credential>, OpenCodeError> {
        let all = self.load_all()?;
        Ok(all
            .get(provider_id)
            .and_then(|creds| creds.iter().find(|c| c.name == "default"))
            .map(|c| c.credential.clone()))
    }

    pub fn load_named(
        &self,
        provider_id: &str,
        name: &str,
    ) -> Result<Option<Credential>, OpenCodeError> {
        let all = self.load_all()?;
        Ok(all
            .get(provider_id)
            .and_then(|creds| creds.iter().find(|c| c.name == name))
            .map(|c| c.credential.clone()))
    }

    pub fn list_credentials(&self, provider_id: &str) -> Result<Vec<String>, OpenCodeError> {
        let all = self.load_all()?;
        Ok(all
            .get(provider_id)
            .map(|creds| creds.iter().map(|c| c.name.clone()).collect())
            .unwrap_or_default())
    }

    pub fn delete(&self, provider_id: &str) -> Result<(), OpenCodeError> {
        let mut all = self.load_all()?;
        all.remove(provider_id);
        self.save_all(&all)
    }

    pub fn delete_named(&self, provider_id: &str, name: &str) -> Result<bool, OpenCodeError> {
        let mut all = self.load_all()?;
        let mut removed = false;

        if let Some(creds) = all.get_mut(provider_id) {
            let original_len = creds.len();
            creds.retain(|c| c.name != name);
            removed = original_len != creds.len();
            if creds.is_empty() {
                all.remove(provider_id);
            }
        }
        self.save_all(&all)?;
        Ok(removed)
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

    fn save_all(&self, credentials: &HashMap<String, Vec<NamedCredential>>) -> Result<(), OpenCodeError> {
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

    fn load_all(&self) -> Result<HashMap<String, Vec<NamedCredential>>, OpenCodeError> {
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

    #[test]
    fn store_named_stores_multiple_credentials_per_provider() {
        let tmp = tempfile::tempdir().unwrap();
        let store = test_store(&tmp);

        let cred1 = Credential {
            api_key: "sk-key-1".to_string(),
            base_url: None,
            metadata: HashMap::new(),
        };
        let cred2 = Credential {
            api_key: "sk-key-2".to_string(),
            base_url: None,
            metadata: HashMap::new(),
        };
        let cred3 = Credential {
            api_key: "sk-key-3".to_string(),
            base_url: None,
            metadata: HashMap::new(),
        };

        store.store_named("anthropic", "work", &cred1).unwrap();
        store.store_named("anthropic", "personal", &cred2).unwrap();
        store.store_named("anthropic", "backup", &cred3).unwrap();

        let names = store.list_credentials("anthropic").unwrap();
        assert_eq!(names.len(), 3);
        assert!(names.contains(&"work".to_string()));
        assert!(names.contains(&"personal".to_string()));
        assert!(names.contains(&"backup".to_string()));
    }

    #[test]
    fn list_credentials_lists_all_stored_credentials_for_provider() {
        let tmp = tempfile::tempdir().unwrap();
        let store = test_store(&tmp);

        store
            .store_named(
                "openai",
                "primary",
                &Credential {
                    api_key: "sk-primary".to_string(),
                    base_url: None,
                    metadata: HashMap::new(),
                },
            )
            .unwrap();
        store
            .store_named(
                "openai",
                "secondary",
                &Credential {
                    api_key: "sk-secondary".to_string(),
                    base_url: None,
                    metadata: HashMap::new(),
                },
            )
            .unwrap();
        store
            .store_named(
                "openai",
                "tertiary",
                &Credential {
                    api_key: "sk-tertiary".to_string(),
                    base_url: None,
                    metadata: HashMap::new(),
                },
            )
            .unwrap();

        let names = store.list_credentials("openai").unwrap();
        assert_eq!(names.len(), 3);
        assert!(names.contains(&"primary".to_string()));
        assert!(names.contains(&"secondary".to_string()));
        assert!(names.contains(&"tertiary".to_string()));
    }

    #[test]
    fn delete_named_removes_specific_named_credential() {
        let tmp = tempfile::tempdir().unwrap();
        let store = test_store(&tmp);

        store
            .store_named(
                "github",
                "token1",
                &Credential {
                    api_key: "ghp_token1".to_string(),
                    base_url: None,
                    metadata: HashMap::new(),
                },
            )
            .unwrap();
        store
            .store_named(
                "github",
                "token2",
                &Credential {
                    api_key: "ghp_token2".to_string(),
                    base_url: None,
                    metadata: HashMap::new(),
                },
            )
            .unwrap();

        let deleted = store.delete_named("github", "token1").unwrap();
        assert!(deleted);

        let remaining = store.list_credentials("github").unwrap();
        assert_eq!(remaining.len(), 1);
        assert!(remaining.contains(&"token2".to_string()));

        let loaded = store.load_named("github", "token1").unwrap();
        assert_eq!(loaded, None);
    }

    #[test]
    fn load_named_loads_specific_named_credential() {
        let tmp = tempfile::tempdir().unwrap();
        let store = test_store(&tmp);

        let cred1 = Credential {
            api_key: "sk-personal-key".to_string(),
            base_url: Some("https://personal.example.com".to_string()),
            metadata: HashMap::from([("env".to_string(), "personal".to_string())]),
        };
        let cred2 = Credential {
            api_key: "sk-work-key".to_string(),
            base_url: Some("https://work.example.com".to_string()),
            metadata: HashMap::from([("env".to_string(), "work".to_string())]),
        };

        store.store_named("provider", "personal", &cred1).unwrap();
        store.store_named("provider", "work", &cred2).unwrap();

        let loaded_personal = store.load_named("provider", "personal").unwrap();
        assert_eq!(loaded_personal, Some(cred1));

        let loaded_work = store.load_named("provider", "work").unwrap();
        assert_eq!(loaded_work, Some(cred2));
    }

    #[test]
    fn delete_named_returns_false_for_nonexistent() {
        let tmp = tempfile::tempdir().unwrap();
        let store = test_store(&tmp);

        let deleted = store.delete_named("nonexistent", "name").unwrap();
        assert!(!deleted);
    }

    #[test]
    fn store_named_overwrites_existing_name() {
        let tmp = tempfile::tempdir().unwrap();
        let store = test_store(&tmp);

        let cred1 = Credential {
            api_key: "sk-old".to_string(),
            base_url: None,
            metadata: HashMap::new(),
        };
        let cred2 = Credential {
            api_key: "sk-new".to_string(),
            base_url: None,
            metadata: HashMap::new(),
        };

        store.store_named("test", "default", &cred1).unwrap();
        assert_eq!(
            store.load_named("test", "default").unwrap(),
            Some(cred1.clone())
        );

        store.store_named("test", "default", &cred2).unwrap();
        assert_eq!(store.list_credentials("test").unwrap().len(), 1);
        assert_eq!(
            store.load_named("test", "default").unwrap(),
            Some(cred2)
        );
    }

    #[test]
    fn store_uses_default_name() {
        let tmp = tempfile::tempdir().unwrap();
        let store = test_store(&tmp);

        let cred = Credential {
            api_key: "sk-default-test".to_string(),
            base_url: None,
            metadata: HashMap::new(),
        };

        store.store("anthropic", &cred).unwrap();

        let names = store.list_credentials("anthropic").unwrap();
        assert_eq!(names, vec!["default"]);

        let loaded = store.load("anthropic").unwrap();
        assert_eq!(loaded, Some(cred));
    }

    #[test]
    fn delete_provider_removes_all_named_credentials() {
        let tmp = tempfile::tempdir().unwrap();
        let store = test_store(&tmp);

        store
            .store_named(
                "openai",
                "first",
                &Credential {
                    api_key: "sk-first".to_string(),
                    base_url: None,
                    metadata: HashMap::new(),
                },
            )
            .unwrap();
        store
            .store_named(
                "openai",
                "second",
                &Credential {
                    api_key: "sk-second".to_string(),
                    base_url: None,
                    metadata: HashMap::new(),
                },
            )
            .unwrap();

        store.delete("openai").unwrap();

        let names = store.list_credentials("openai").unwrap();
        assert!(names.is_empty());
    }
}
