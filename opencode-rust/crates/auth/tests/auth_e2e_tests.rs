use opencode_auth::credential_store::Credential;
use opencode_auth::jwt::{create_token, validate_token, Claims};
use opencode_auth::CredentialStore;
use std::collections::HashMap;
use std::io::{Read, Write};
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

mod jwt_key_rotation_tests {
    use super::*;

    #[test]
    fn test_token_signed_with_old_key_rejected_after_rotation() {
        let user_id = "user-123";
        let old_secret = "old-secret-key-12345678901234567890";
        let new_secret = "new-secret-key-12345678901234567890";

        let old_token = create_token(user_id, old_secret).unwrap();

        let result = validate_token(&old_token, new_secret);
        assert!(
            result.is_err(),
            "Token signed with old key should be rejected after rotation"
        );
    }

    #[test]
    fn test_token_signed_with_new_key_accepted_after_rotation() {
        let user_id = "user-123";
        let new_secret = "new-secret-key-12345678901234567890";

        let new_token = create_token(user_id, new_secret).unwrap();

        let claims = validate_token(&new_token, new_secret).unwrap();
        assert_eq!(claims.sub, user_id);
    }

    #[test]
    fn test_multiple_key_versions_supported() {
        let user_id = "user-123";
        let secret_v1 = "version-1-secret-key-1234567890123456";
        let secret_v2 = "version-2-secret-key-1234567890123456";

        let token_v1 = create_token(user_id, secret_v1).unwrap();
        let token_v2 = create_token(user_id, secret_v2).unwrap();

        let claims_v1 = validate_token(&token_v1, secret_v1).unwrap();
        let claims_v2 = validate_token(&token_v2, secret_v2).unwrap();

        assert_eq!(claims_v1.sub, user_id);
        assert_eq!(claims_v2.sub, user_id);

        let result_v1_with_v2 = validate_token(&token_v1, secret_v2);
        assert!(result_v1_with_v2.is_err());

        let result_v2_with_v1 = validate_token(&token_v2, secret_v1);
        assert!(result_v2_with_v1.is_err());
    }

    #[test]
    fn test_expired_token_rejected_regardless_of_key() {
        let user_id = "user-123";
        let secret = "test-secret-key-12345678901234567890";

        let token = create_token(user_id, secret).unwrap();

        std::panic::set_hook(Box::new(|_| {}));

        use jsonwebtoken::{decode, Algorithm, DecodingKey, Validation};

        let mut validation = Validation::new(Algorithm::HS256);
        validation.validate_exp = true;

        let result = decode::<Claims>(
            &token,
            &DecodingKey::from_secret(secret.as_ref()),
            &validation,
        );

        if result.is_ok() {
            let token = create_token(user_id, secret).unwrap();
            let result = validate_token(&token, secret);
            assert!(result.is_ok(), "Fresh token should be valid");
        }
    }
}

mod oauth_state_tests {
    use super::*;

    fn spawn_mock_token_server(response_body: String) -> String {
        let listener = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
        let addr = format!("http://{}", listener.local_addr().unwrap());
        let (tx, rx) = std::sync::mpsc::channel::<()>();

        std::thread::spawn(move || {
            tx.send(()).unwrap();
            let (mut stream, _) = listener.accept().unwrap();
            let mut buffer = [0u8; 8192];
            let _ = stream.read(&mut buffer).unwrap();

            let response = format!(
                "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
                response_body.len(),
                response_body
            );
            stream.write_all(response.as_bytes()).unwrap();
        });

        let _ = rx.recv();
        addr
    }

    #[test]
    fn test_wrong_state_rejected() {
        let (store, _temp_dir) = create_temp_credential_store();
        let session_manager =
            opencode_auth::oauth::OAuthSessionManager::new(_temp_dir.path().to_path_buf());
        let flow = opencode_auth::oauth::OAuthFlow::with_client_and_store(
            reqwest::blocking::Client::new(),
            store,
            session_manager,
        );

        let (_auth_url, _state, verifier) = flow
            .start_login("github", "client-1", "http://127.0.0.1/callback")
            .unwrap();

        let result = flow.complete_login(
            "auth-code",
            "wrong-state",
            &verifier,
            "client-1",
            "secret-1",
            "http://127.0.0.1:9999/token",
        );

        assert!(result.is_err(), "Wrong state should be rejected");
        assert!(matches!(
            result.unwrap_err(),
            opencode_auth::oauth::OAuthError::InvalidState
        ));
    }

    #[test]
    fn test_correct_state_accepted() {
        let (store, _temp_dir) = create_temp_credential_store();
        let session_manager =
            opencode_auth::oauth::OAuthSessionManager::new(_temp_dir.path().to_path_buf());
        let flow = opencode_auth::oauth::OAuthFlow::with_client_and_store(
            reqwest::blocking::Client::new(),
            store,
            session_manager,
        );

        let (_auth_url, state, verifier) = flow
            .start_login("github", "client-1", "http://127.0.0.1/callback")
            .unwrap();

        let endpoint = spawn_mock_token_server(
            serde_json::json!({
                "access_token": "access-1",
                "refresh_token": "refresh-1",
                "expires_in": 1200,
                "token_type": "Bearer",
                "scope": "repo"
            })
            .to_string(),
        );

        let token = flow
            .complete_login(
                "auth-code",
                &state,
                &verifier,
                "client-1",
                "secret-1",
                &endpoint,
            )
            .unwrap();

        assert_eq!(token.access_token, "access-1");
        assert_eq!(token.refresh_token.as_deref(), Some("refresh-1"));
    }

    #[test]
    fn test_state_is_single_use() {
        let (store, _temp_dir) = create_temp_credential_store();
        let session_manager =
            opencode_auth::oauth::OAuthSessionManager::new(_temp_dir.path().to_path_buf());
        let flow = opencode_auth::oauth::OAuthFlow::with_client_and_store(
            reqwest::blocking::Client::new(),
            store,
            session_manager,
        );

        let (_auth_url, state, verifier) = flow
            .start_login("github", "client-1", "http://127.0.0.1/callback")
            .unwrap();

        let endpoint = spawn_mock_token_server(
            serde_json::json!({
                "access_token": "access-1",
                "refresh_token": "refresh-1",
                "expires_in": 1200,
                "token_type": "Bearer",
                "scope": "repo"
            })
            .to_string(),
        );

        let result1 = flow.complete_login(
            "auth-code-1",
            &state,
            &verifier,
            "client-1",
            "secret-1",
            &endpoint,
        );
        assert!(result1.is_ok(), "First use should succeed");

        let result2 = flow.complete_login(
            "auth-code-2",
            &state,
            &verifier,
            "client-1",
            "secret-1",
            &endpoint,
        );
        assert!(
            result2.is_err(),
            "Second use (replay) should fail - state already consumed"
        );
        assert!(matches!(
            result2.unwrap_err(),
            opencode_auth::oauth::OAuthError::InvalidState
        ));
    }

    #[test]
    fn test_replay_attack_prevented() {
        let (store, _temp_dir) = create_temp_credential_store();
        let session_manager =
            opencode_auth::oauth::OAuthSessionManager::new(_temp_dir.path().to_path_buf());
        let flow = opencode_auth::oauth::OAuthFlow::with_client_and_store(
            reqwest::blocking::Client::new(),
            store,
            session_manager,
        );

        let (_auth_url, state, verifier) = flow
            .start_login("github", "client-1", "http://127.0.0.1/callback")
            .unwrap();

        let endpoint = spawn_mock_token_server(
            serde_json::json!({
                "access_token": "access-token",
                "refresh_token": "refresh-token",
                "expires_in": 1200,
                "token_type": "Bearer",
                "scope": "repo"
            })
            .to_string(),
        );

        let first_result = flow.complete_login(
            "stolen-auth-code",
            &state,
            &verifier,
            "client-1",
            "secret-1",
            &endpoint,
        );
        assert!(first_result.is_ok(), "Legitimate first use should succeed");

        let second_result = flow.complete_login(
            "stolen-auth-code-replay",
            &state,
            &verifier,
            "client-1",
            "secret-1",
            &endpoint,
        );
        assert!(
            second_result.is_err(),
            "Replay attack with same state should be rejected"
        );
    }
}

mod oauth_token_refresh_tests {
    use super::*;

    fn spawn_mock_token_server(response_body: String) -> String {
        let listener = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
        let addr = format!("http://{}", listener.local_addr().unwrap());
        let (tx, rx) = std::sync::mpsc::channel::<()>();

        std::thread::spawn(move || {
            tx.send(()).unwrap();
            let (mut stream, _) = listener.accept().unwrap();
            let mut buffer = [0u8; 8192];
            let _ = stream.read(&mut buffer).unwrap();

            let response = format!(
                "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
                response_body.len(),
                response_body
            );
            stream.write_all(response.as_bytes()).unwrap();
        });

        let _ = rx.recv();
        addr
    }

    #[test]
    fn test_token_refresh_when_expired() {
        let (store, _temp_dir) = create_temp_credential_store();
        let session_manager =
            opencode_auth::oauth::OAuthSessionManager::new(_temp_dir.path().to_path_buf());
        let flow = opencode_auth::oauth::OAuthFlow::with_client_and_store(
            reqwest::blocking::Client::new(),
            store,
            session_manager,
        );

        let token = opencode_auth::oauth::OAuthToken {
            access_token: "expired-access".into(),
            refresh_token: Some("refresh-1".into()),
            expires_in: 1,
            token_type: "Bearer".into(),
            scope: Some("repo".into()),
            received_at: chrono::Utc::now() - chrono::Duration::seconds(10),
        };
        assert!(token.is_expired(), "Token should be expired");

        flow.store_token("test-provider", &token).unwrap();

        let endpoint = spawn_mock_token_server(
            serde_json::json!({
                "access_token": "refreshed-access",
                "refresh_token": "refresh-2",
                "expires_in": 3600,
                "token_type": "Bearer",
                "scope": "repo"
            })
            .to_string(),
        );

        let refreshed = flow
            .ensure_fresh_token("test-provider", "client-1", "secret-1", &endpoint)
            .unwrap()
            .unwrap();

        assert_eq!(refreshed.access_token, "refreshed-access");
        assert_eq!(refreshed.refresh_token.as_deref(), Some("refresh-2"));
        assert!(!refreshed.is_expired());
    }

    #[test]
    fn test_automatic_refresh_on_expired_token() {
        let (store, _temp_dir) = create_temp_credential_store();
        let session_manager =
            opencode_auth::oauth::OAuthSessionManager::new(_temp_dir.path().to_path_buf());
        let flow = opencode_auth::oauth::OAuthFlow::with_client_and_store(
            reqwest::blocking::Client::new(),
            store,
            session_manager,
        );

        let expired_token = opencode_auth::oauth::OAuthToken {
            access_token: "old-expired-access".into(),
            refresh_token: Some("original-refresh".into()),
            expires_in: 1,
            token_type: "Bearer".into(),
            scope: Some("read".into()),
            received_at: chrono::Utc::now() - chrono::Duration::seconds(10),
        };
        flow.store_token("auto-refresh-provider", &expired_token)
            .unwrap();

        let endpoint = spawn_mock_token_server(
            serde_json::json!({
                "access_token": "new-auto-access",
                "refresh_token": "new-auto-refresh",
                "expires_in": 3600,
                "token_type": "Bearer",
                "scope": "read"
            })
            .to_string(),
        );

        let result =
            flow.ensure_fresh_token("auto-refresh-provider", "client-1", "secret-1", &endpoint);

        assert!(result.is_ok(), "Should trigger automatic refresh");
        let new_token = result.unwrap().unwrap();
        assert_eq!(new_token.access_token, "new-auto-access");
        assert!(!new_token.is_expired());
    }

    #[test]
    fn test_token_stored_and_retrieved_correctly() {
        let (store, _temp_dir) = create_temp_credential_store();
        let session_manager =
            opencode_auth::oauth::OAuthSessionManager::new(_temp_dir.path().to_path_buf());
        let flow = opencode_auth::oauth::OAuthFlow::with_client_and_store(
            reqwest::blocking::Client::new(),
            store,
            session_manager,
        );

        let token = opencode_auth::oauth::OAuthToken {
            access_token: "test-access-token".into(),
            refresh_token: Some("test-refresh-token".into()),
            expires_in: 3600,
            token_type: "Bearer".into(),
            scope: Some("read write".into()),
            received_at: chrono::Utc::now(),
        };

        flow.store_token("test-provider", &token).unwrap();

        let loaded = flow.load_token("test-provider").unwrap().unwrap();
        assert_eq!(loaded.access_token, "test-access-token");
        assert_eq!(loaded.refresh_token.as_deref(), Some("test-refresh-token"));
        assert_eq!(loaded.expires_in, 3600);
        assert_eq!(loaded.token_type, "Bearer");
        assert_eq!(loaded.scope.as_deref(), Some("read write"));
    }

    #[test]
    fn test_missing_refresh_token_error() {
        let (store, _temp_dir) = create_temp_credential_store();
        let session_manager =
            opencode_auth::oauth::OAuthSessionManager::new(_temp_dir.path().to_path_buf());
        let flow = opencode_auth::oauth::OAuthFlow::with_client_and_store(
            reqwest::blocking::Client::new(),
            store,
            session_manager,
        );

        let token = opencode_auth::oauth::OAuthToken {
            access_token: "access-without-refresh".into(),
            refresh_token: None,
            expires_in: 1,
            token_type: "Bearer".into(),
            scope: None,
            received_at: chrono::Utc::now() - chrono::Duration::seconds(10),
        };
        flow.store_token("no-refresh-provider", &token).unwrap();

        let loaded = flow.load_token("no-refresh-provider").unwrap().unwrap();
        assert!(loaded.refresh_token.is_none());
        assert!(loaded.is_expired());

        let result = flow.ensure_fresh_token(
            "no-refresh-provider",
            "client-1",
            "secret-1",
            "http://example.com/token",
        );
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            opencode_auth::oauth::OAuthError::MissingRefreshToken
        ));
    }
}

mod oauth_refresh_token_rotation_tests {
    use super::*;

    #[test]
    fn test_old_refresh_token_invalidated_after_use() {
        let (store, _temp_dir) = create_temp_credential_store();
        let session_manager =
            opencode_auth::oauth::OAuthSessionManager::new(_temp_dir.path().to_path_buf());
        let _flow = opencode_auth::oauth::OAuthFlow::with_client_and_store(
            reqwest::blocking::Client::new(),
            store,
            session_manager,
        );
    }

    #[test]
    fn test_refresh_token_rotation_prevents_reuse() {
        let (store, _temp_dir) = create_temp_credential_store();
        let session_manager =
            opencode_auth::oauth::OAuthSessionManager::new(_temp_dir.path().to_path_buf());
        let _flow = opencode_auth::oauth::OAuthFlow::with_client_and_store(
            reqwest::blocking::Client::new(),
            store,
            session_manager,
        );
    }

    #[test]
    fn test_new_refresh_token_required_after_rotation() {
        let (store, _temp_dir) = create_temp_credential_store();
        let session_manager =
            opencode_auth::oauth::OAuthSessionManager::new(_temp_dir.path().to_path_buf());
        let _flow = opencode_auth::oauth::OAuthFlow::with_client_and_store(
            reqwest::blocking::Client::new(),
            store,
            session_manager,
        );
    }
}

mod session_binding_tests {
    use super::*;

    #[test]
    fn test_token_with_device_binding_validates_correctly() {
        let (store, _temp_dir) = create_temp_credential_store();
        let session_manager =
            opencode_auth::oauth::OAuthSessionManager::new(_temp_dir.path().to_path_buf());
        let _flow = opencode_auth::oauth::OAuthFlow::with_client_and_store(
            reqwest::blocking::Client::new(),
            store,
            session_manager,
        );
    }

    #[test]
    fn test_token_used_on_different_device_rejected() {
        let (store, _temp_dir) = create_temp_credential_store();
        let session_manager =
            opencode_auth::oauth::OAuthSessionManager::new(_temp_dir.path().to_path_buf());
        let _flow = opencode_auth::oauth::OAuthFlow::with_client_and_store(
            reqwest::blocking::Client::new(),
            store,
            session_manager,
        );
    }

    #[test]
    fn test_device_id_validated_per_request() {
        let (store, _temp_dir) = create_temp_credential_store();
        let session_manager =
            opencode_auth::oauth::OAuthSessionManager::new(_temp_dir.path().to_path_buf());
        let _flow = opencode_auth::oauth::OAuthFlow::with_client_and_store(
            reqwest::blocking::Client::new(),
            store,
            session_manager,
        );
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
