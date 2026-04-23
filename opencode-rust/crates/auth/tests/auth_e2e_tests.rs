use opencode_auth::credential_store::Credential;
use opencode_auth::jwt::{create_token, validate_token, Claims};
use opencode_auth::CredentialStore;
use std::collections::HashMap;
use tempfile::TempDir;

fn create_temp_credential_store() -> (CredentialStore, TempDir) {
    let temp_dir = TempDir::new().unwrap();
    let store_path = temp_dir.path().join("credentials.enc.json");
    let key_path = temp_dir.path().join("credentials.key");
    let store = CredentialStore::with_paths(store_path, key_path);
    (store, temp_dir)
}

mod jwt_tests {
    use super::*;

    #[test]
    fn test_create_and_validate_token() {
        let user_id = "user-123";
        let secret = "test-secret-key-12345678901234567890";

        let token = create_token(user_id, secret).unwrap();
        assert!(!token.is_empty());

        let claims = validate_token(&token, secret).unwrap();
        assert_eq!(claims.sub, user_id);
    }

    #[test]
    fn test_validate_token_invalid_secret() {
        let user_id = "user-123";
        let secret = "correct-secret-key-123456789012345";
        let wrong_secret = "wrong-secret-key-123456789012345";

        let token = create_token(user_id, secret).unwrap();
        let result = validate_token(&token, wrong_secret);
        assert!(result.is_err());
    }

    #[test]
    fn test_validate_token_malformed_token() {
        let secret = "test-secret-key-12345678901234567890";
        let result = validate_token("not.a.valid.token", secret);
        assert!(result.is_err());
    }

    #[test]
    fn test_validate_token_empty() {
        let secret = "test-secret-key-12345678901234567890";
        let result = validate_token("", secret);
        assert!(result.is_err());
    }

    #[test]
    fn test_claims_struct_fields() {
        let claims = Claims {
            sub: "test-user".to_string(),
            exp: 1234567890,
            iat: 1234500000,
        };
        assert_eq!(claims.sub, "test-user");
        assert_eq!(claims.exp, 1234567890);
        assert_eq!(claims.iat, 1234500000);
    }
}

mod credential_store_tests {
    use super::*;

    #[test]
    fn test_store_and_load_credential() {
        let (store, _temp_dir) = create_temp_credential_store();

        let credential = Credential {
            api_key: "sk-test-12345".to_string(),
            base_url: Some("https://api.test.com".to_string()),
            metadata: HashMap::new(),
        };

        store.store("test-provider", &credential).unwrap();

        let loaded = store.load("test-provider").unwrap().unwrap();
        assert_eq!(loaded.api_key, "sk-test-12345");
        assert_eq!(loaded.base_url, Some("https://api.test.com".to_string()));
    }

    #[test]
    fn test_load_nonexistent_credential() {
        let (store, _temp_dir) = create_temp_credential_store();

        let result = store.load("nonexistent-provider").unwrap();
        assert!(result.is_none());
    }

    #[test]
    fn test_delete_credential() {
        let (store, _temp_dir) = create_temp_credential_store();

        let credential = Credential {
            api_key: "sk-test-delete".to_string(),
            base_url: None,
            metadata: HashMap::new(),
        };

        store.store("delete-me", &credential).unwrap();
        assert!(store.load("delete-me").unwrap().is_some());

        store.delete("delete-me").unwrap();
        assert!(store.load("delete-me").unwrap().is_none());
    }

    #[test]
    fn test_store_multiple_providers() {
        let (store, _temp_dir) = create_temp_credential_store();

        let cred1 = Credential {
            api_key: "key1".to_string(),
            base_url: None,
            metadata: HashMap::new(),
        };
        let cred2 = Credential {
            api_key: "key2".to_string(),
            base_url: None,
            metadata: HashMap::new(),
        };

        store.store("provider1", &cred1).unwrap();
        store.store("provider2", &cred2).unwrap();

        assert_eq!(store.load("provider1").unwrap().unwrap().api_key, "key1");
        assert_eq!(store.load("provider2").unwrap().unwrap().api_key, "key2");
    }

    #[test]
    fn test_update_existing_credential() {
        let (store, _temp_dir) = create_temp_credential_store();

        let cred1 = Credential {
            api_key: "old-key".to_string(),
            base_url: None,
            metadata: HashMap::new(),
        };
        store.store("update-test", &cred1).unwrap();

        let cred2 = Credential {
            api_key: "new-key".to_string(),
            base_url: Some("https://new-url.com".to_string()),
            metadata: HashMap::new(),
        };
        store.store("update-test", &cred2).unwrap();

        let loaded = store.load("update-test").unwrap().unwrap();
        assert_eq!(loaded.api_key, "new-key");
        assert_eq!(loaded.base_url, Some("https://new-url.com".to_string()));
    }

    #[test]
    fn test_store_named_credential() {
        let (store, _temp_dir) = create_temp_credential_store();

        let cred1 = Credential {
            api_key: "key-prod".to_string(),
            base_url: None,
            metadata: HashMap::new(),
        };
        let cred2 = Credential {
            api_key: "key-dev".to_string(),
            base_url: None,
            metadata: HashMap::new(),
        };

        store.store_named("provider", "production", &cred1).unwrap();
        store
            .store_named("provider", "development", &cred2)
            .unwrap();

        assert_eq!(
            store
                .load_named("provider", "production")
                .unwrap()
                .unwrap()
                .api_key,
            "key-prod"
        );
        assert_eq!(
            store
                .load_named("provider", "development")
                .unwrap()
                .unwrap()
                .api_key,
            "key-dev"
        );
    }

    #[test]
    fn test_list_credentials_for_provider() {
        let (store, _temp_dir) = create_temp_credential_store();

        let cred1 = Credential {
            api_key: "key1".to_string(),
            base_url: None,
            metadata: HashMap::new(),
        };
        let cred2 = Credential {
            api_key: "key2".to_string(),
            base_url: None,
            metadata: HashMap::new(),
        };

        store.store_named("multi", "first", &cred1).unwrap();
        store.store_named("multi", "second", &cred2).unwrap();

        let names = store.list_credentials("multi").unwrap();
        assert_eq!(names.len(), 2);
        assert!(names.contains(&"first".to_string()));
        assert!(names.contains(&"second".to_string()));
    }

    #[test]
    fn test_credential_with_metadata() {
        let (store, _temp_dir) = create_temp_credential_store();

        let mut metadata = HashMap::new();
        metadata.insert("environment".to_string(), "production".to_string());
        metadata.insert("version".to_string(), "1.0".to_string());

        let credential = Credential {
            api_key: "key-with-meta".to_string(),
            base_url: None,
            metadata,
        };

        store.store("meta-test", &credential).unwrap();

        let loaded = store.load("meta-test").unwrap().unwrap();
        assert_eq!(loaded.metadata.get("environment").unwrap(), "production");
        assert_eq!(loaded.metadata.get("version").unwrap(), "1.0");
    }
}

mod credential_encryption_tests {
    use super::*;

    #[test]
    fn test_stored_credentials_encrypted() {
        let (store, temp_dir) = create_temp_credential_store();

        let credential = Credential {
            api_key: "super-secret-key-12345".to_string(),
            base_url: None,
            metadata: HashMap::new(),
        };

        store.store("encrypt-test", &credential).unwrap();

        let raw_file_path = temp_dir.path().join("credentials.enc.json");
        let raw_content = std::fs::read_to_string(&raw_file_path).unwrap();

        assert!(!raw_content.contains("super-secret-key"));
        assert!(raw_content.contains("nonce") || raw_content.contains("ciphertext"));
    }

    #[test]
    fn test_store_with_password_different_encryption() {
        let temp_dir = TempDir::new().unwrap();
        let store_path = temp_dir.path().join("credentials.enc.json");
        let key_path = temp_dir.path().join("credentials.key");

        let store = CredentialStore::with_paths_and_password(
            store_path.clone(),
            key_path.clone(),
            "master-password-123".to_string(),
        );

        let credential = Credential {
            api_key: "password-protected-key".to_string(),
            base_url: None,
            metadata: HashMap::new(),
        };

        store.store("pw-test", &credential).unwrap();

        let raw_content = std::fs::read_to_string(&store_path).unwrap();
        assert!(!raw_content.contains("password-protected-key"));
    }

    #[test]
    fn test_load_with_wrong_password_fails() {
        let temp_dir = TempDir::new().unwrap();
        let store_path = temp_dir.path().join("credentials.enc.json");
        let key_path = temp_dir.path().join("credentials.key");

        let store = CredentialStore::with_paths_and_password(
            store_path.clone(),
            key_path.clone(),
            "correct-password".to_string(),
        );

        let credential = Credential {
            api_key: "secret-key".to_string(),
            base_url: None,
            metadata: HashMap::new(),
        };

        store.store("wrong-pw-test", &credential).unwrap();

        let wrong_pw_store = CredentialStore::with_paths_and_password(
            store_path,
            key_path,
            "wrong-password".to_string(),
        );

        let result = wrong_pw_store.load("wrong-pw-test");
        assert!(result.is_err());
    }
}
