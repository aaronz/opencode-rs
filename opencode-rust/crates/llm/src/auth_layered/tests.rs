#[cfg(test)]
mod test_mod {
    use crate::auth_layered::{
        is_oauth_only_provider, AccessControlResult, AnthropicTransport, AuthMechanism,
        AwsSigV4Transport, CompositeCredentialResolver, CopilotOAuthCallback, CopilotOAuthRequest,
        CopilotOAuthService, CopilotOAuthSession, CopilotOAuthStore, CredentialResolver,
        CredentialSource, GoogleOAuthCallback, GoogleOAuthRequest, GoogleOAuthService,
        GoogleOAuthSession, GoogleOAuthStore, OpenAICompatibleTransport, ProviderTransport,
        RuntimeAccessControl, TransportLayer,
    };
    use std::collections::HashMap;

    #[test]
    fn test_layer1_credential_source_env_var() {
        let mut env_vars = HashMap::new();
        env_vars.insert("openai".to_string(), "sk-env-key".to_string());

        let resolver = CompositeCredentialResolver::new().with_inline(env_vars);
        let cred = resolver.resolve("openai", &CredentialSource::ConfigInline);

        assert!(cred.is_some());
        assert_eq!(cred.unwrap().value, "sk-env-key");
    }

    #[test]
    fn test_layer1_credential_source_fallback_order() {
        let mut creds = HashMap::new();
        creds.insert("openai".to_string(), "sk-fallback-key".to_string());

        let resolver = CompositeCredentialResolver::new().with_inline(creds);
        let cred = resolver.resolve_with_fallback(
            "openai",
            &[CredentialSource::OAuthStore, CredentialSource::ConfigInline],
        );

        assert!(cred.is_some());
    }

    #[test]
    fn test_layer2_auth_mechanism_types() {
        assert!(!AuthMechanism::ApiKey.requires_interactive_login());

        assert!(AuthMechanism::OAuthBrowser.requires_interactive_login());
        assert!(AuthMechanism::DeviceCode.requires_interactive_login());
        assert!(!AuthMechanism::BearerToken.requires_interactive_login());

        assert!(AuthMechanism::AwsCredentialChain.supports_refresh());
        assert!(AuthMechanism::OAuthBrowser.supports_refresh());
        assert!(!AuthMechanism::ApiKey.supports_refresh());

        assert!(AuthMechanism::AwsCredentialChain.is_cloud_native());
        assert!(AuthMechanism::ServiceAccountJson.is_cloud_native());
        assert!(!AuthMechanism::ApiKey.is_cloud_native());
    }

    #[test]
    fn test_layer3_openai_transport() {
        let transport = OpenAICompatibleTransport;
        assert_eq!(transport.endpoint_path(), "/v1/chat/completions");

        let headers = transport.required_headers();
        assert!(headers.iter().any(|(k, _)| *k == "Content-Type"));
    }

    #[test]
    fn test_layer3_anthropic_transport() {
        let transport = AnthropicTransport;
        assert_eq!(transport.endpoint_path(), "/v1/messages");

        let headers = transport.required_headers();
        assert!(headers.iter().any(|(k, _)| *k == "Content-Type"));
        assert!(headers.iter().any(|(k, _)| *k == "anthropic-version"));
    }

    #[test]
    fn test_layer3_aws_sigv4_transport() {
        let transport = AwsSigV4Transport::new("us-east-1".to_string(), "bedrock".to_string());
        assert_eq!(transport.endpoint_path(), "/2023-05-31/inference-profiles");
    }

    #[test]
    fn test_layer3_transport_layer_full_url() {
        let layer = TransportLayer::new(
            Box::new(OpenAICompatibleTransport),
            "https://api.openai.com".to_string(),
        );
        assert_eq!(
            layer.full_url(None),
            "https://api.openai.com/v1/chat/completions"
        );
    }

    #[test]
    fn test_layer4_access_control_default_allow() {
        let acl = RuntimeAccessControl::new();
        assert!(matches!(
            acl.check_provider_access("openai"),
            AccessControlResult::Allowed
        ));
    }

    #[test]
    fn test_layer4_access_control_denylist() {
        let mut denylist = std::collections::HashSet::new();
        denylist.insert("disabled".to_string());

        let acl = RuntimeAccessControl::new().with_denylist(denylist);
        assert!(matches!(
            acl.check_provider_access("disabled"),
            AccessControlResult::Denied(_)
        ));
        assert!(matches!(
            acl.check_provider_access("openai"),
            AccessControlResult::Allowed
        ));
    }

    #[test]
    fn test_layer4_access_control_allowlist() {
        let mut allowlist = std::collections::HashSet::new();
        allowlist.insert("openai".to_string());

        let acl = RuntimeAccessControl::new().with_allowlist(allowlist);
        assert!(matches!(
            acl.check_provider_access("openai"),
            AccessControlResult::Allowed
        ));
        assert!(matches!(
            acl.check_provider_access("anthropic"),
            AccessControlResult::Denied(_)
        ));
    }

    #[test]
    fn test_layer4_denylist_precedence() {
        let mut allowlist = std::collections::HashSet::new();
        allowlist.insert("openai".to_string());
        let mut denylist = std::collections::HashSet::new();
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
    fn test_layer4_server_basic_auth() {
        let acl = RuntimeAccessControl::new().with_server_basic_auth(true);
        assert!(acl.is_server_auth_required());

        let acl2 = RuntimeAccessControl::new().with_server_basic_auth(false);
        assert!(!acl2.is_server_auth_required());
    }

    #[test]
    fn test_integration_all_layers() {
        let mut creds = HashMap::new();
        creds.insert("openai".to_string(), "sk-integration".to_string());
        let resolver = CompositeCredentialResolver::new().with_inline(creds);

        let mut denylist = std::collections::HashSet::new();
        denylist.insert("banned".to_string());
        let acl = RuntimeAccessControl::new().with_denylist(denylist);

        let result = acl.check_provider_access("openai");
        assert!(matches!(result, AccessControlResult::Allowed));

        let result = acl.check_provider_access("banned");
        assert!(matches!(result, AccessControlResult::Denied(_)));

        let cred = resolver.resolve("openai", &CredentialSource::ConfigInline);
        assert!(cred.is_some());
        assert_eq!(cred.unwrap().value, "sk-integration");
    }

    #[test]
    fn test_oauth_only_providers_identified() {
        assert!(is_oauth_only_provider("google"));
        assert!(is_oauth_only_provider("copilot"));
        assert!(!is_oauth_only_provider("openai"));
        assert!(!is_oauth_only_provider("anthropic"));
        assert!(!is_oauth_only_provider("ollama"));
        assert!(!is_oauth_only_provider("mistral"));
        assert!(!is_oauth_only_provider("bedrock"));
    }

    #[test]
    fn test_google_oauth_service_creation() {
        let service = GoogleOAuthService::new();
        let result = service.start_local_callback_listener();
        assert!(result.is_ok());
    }

    #[test]
    fn test_google_oauth_authorize_url_generation() {
        let service = GoogleOAuthService::new();
        let request = GoogleOAuthRequest {
            redirect_uri: "http://127.0.0.1:8080/auth/callback".to_string(),
            state: "test-state".to_string(),
            code_verifier: "test-verifier-12345678901234567890123456789012345678901234567890123456"
                .to_string(),
        };
        let url = service.build_authorize_url(&request);
        assert!(url.contains("accounts.google.com"));
        assert!(url.contains("scope="));
        assert!(url.contains("state=test-state"));
    }

    #[test]
    fn test_google_oauth_session_storage() {
        let dir = tempfile::tempdir().unwrap();
        let store = GoogleOAuthStore::new(dir.path().to_path_buf());
        let session = GoogleOAuthSession {
            access_token: "ya29.test".to_string(),
            refresh_token: Some("1//test_refresh".to_string()),
            expires_at_epoch_ms: chrono::Utc::now().timestamp_millis() + 3600000,
            email: Some("test@gmail.com".to_string()),
        };

        assert!(store.save(&session).is_ok());
        let loaded = store.load().unwrap().unwrap();
        assert_eq!(loaded.access_token, "ya29.test");
        assert_eq!(loaded.email, Some("test@gmail.com".to_string()));
    }

    #[test]
    fn test_google_oauth_session_expiration() {
        let expired_session = GoogleOAuthSession {
            access_token: "ya29.expired".to_string(),
            refresh_token: Some("1//expired_refresh".to_string()),
            expires_at_epoch_ms: chrono::Utc::now().timestamp_millis() - 1000,
            email: None,
        };
        assert!(expired_session.is_expired());

        let valid_session = GoogleOAuthSession {
            access_token: "ya29.valid".to_string(),
            refresh_token: Some("1//valid_refresh".to_string()),
            expires_at_epoch_ms: chrono::Utc::now().timestamp_millis() + 3600000,
            email: None,
        };
        assert!(!valid_session.is_expired());
    }

    #[test]
    fn test_google_oauth_exchange_code_state_mismatch() {
        let service = GoogleOAuthService::new();
        let callback = GoogleOAuthCallback {
            code: "test-code".to_string(),
            state: "wrong-state".to_string(),
        };
        let request = GoogleOAuthRequest {
            redirect_uri: "http://127.0.0.1:8080/auth/callback".to_string(),
            state: "correct-state".to_string(),
            code_verifier: "verifier".to_string(),
        };
        let result = service.exchange_code(callback, &request);
        assert!(result.is_err());
    }

    #[test]
    fn test_copilot_oauth_service_creation() {
        let service = CopilotOAuthService::new();
        let result = service.start_local_callback_listener();
        assert!(result.is_ok());
    }

    #[test]
    fn test_copilot_oauth_authorize_url_generation() {
        let service = CopilotOAuthService::new();
        let request = CopilotOAuthRequest {
            redirect_uri: "http://127.0.0.1:8080/auth/callback".to_string(),
            state: "test-state".to_string(),
            code_verifier: "test-verifier-12345678901234567890123456789012345678901234567890123456"
                .to_string(),
        };
        let url = service.build_authorize_url(&request);
        assert!(url.contains("github.com"));
        assert!(url.contains("scope=copilot"));
        assert!(url.contains("state=test-state"));
    }

    #[test]
    fn test_copilot_oauth_session_storage() {
        let dir = tempfile::tempdir().unwrap();
        let store = CopilotOAuthStore::new(dir.path().to_path_buf());
        let session = CopilotOAuthSession {
            access_token: "gho_test_token".to_string(),
            expires_at_epoch_ms: chrono::Utc::now().timestamp_millis() + 3600000,
            token_type: "Bearer".to_string(),
        };

        assert!(store.save(&session).is_ok());
        let loaded = store.load().unwrap().unwrap();
        assert_eq!(loaded.access_token, "gho_test_token");
        assert_eq!(loaded.token_type, "Bearer");
    }

    #[test]
    fn test_copilot_oauth_session_expiration() {
        let expired_session = CopilotOAuthSession {
            access_token: "gho_expired".to_string(),
            expires_at_epoch_ms: chrono::Utc::now().timestamp_millis() - 1000,
            token_type: "Bearer".to_string(),
        };
        assert!(expired_session.is_expired());

        let valid_session = CopilotOAuthSession {
            access_token: "gho_valid".to_string(),
            expires_at_epoch_ms: chrono::Utc::now().timestamp_millis() + 3600000,
            token_type: "Bearer".to_string(),
        };
        assert!(!valid_session.is_expired());
    }

    #[test]
    fn test_copilot_oauth_exchange_code_state_mismatch() {
        let service = CopilotOAuthService::new();
        let callback = CopilotOAuthCallback {
            code: "test-code".to_string(),
            state: "wrong-state".to_string(),
        };
        let request = CopilotOAuthRequest {
            redirect_uri: "http://127.0.0.1:8080/auth/callback".to_string(),
            state: "correct-state".to_string(),
            code_verifier: "verifier".to_string(),
        };
        let result = service.exchange_code(callback, &request);
        assert!(result.is_err());
    }

    #[test]
    fn test_non_oauth_providers_api_key_auth() {
        let providers = vec!["openai", "anthropic", "ollama", "mistral", "groq"];
        for provider in providers {
            assert!(
                !is_oauth_only_provider(provider),
                "{} should support API key auth",
                provider
            );
        }
    }

    #[test]
    fn test_oauth_only_providers_list() {
        let oauth_providers = vec!["google", "copilot"];
        for provider in oauth_providers {
            assert!(
                is_oauth_only_provider(provider),
                "{} should be OAuth-only",
                provider
            );
        }
    }

    #[test]
    fn test_google_oauth_callback_server_request() {
        let service = GoogleOAuthService::new();
        let server = service.start_local_callback_listener().unwrap();
        let req = server.request();
        assert_eq!(req.state.len(), 32);
        assert!(req.redirect_uri.starts_with("http://127.0.0.1:"));
        assert!(req.redirect_uri.ends_with("/auth/callback"));
    }

    #[test]
    fn test_copilot_oauth_callback_server_request() {
        let service = CopilotOAuthService::new();
        let server = service.start_local_callback_listener().unwrap();
        let req = server.request();
        assert_eq!(req.state.len(), 32);
        assert!(req.redirect_uri.starts_with("http://127.0.0.1:"));
        assert!(req.redirect_uri.ends_with("/auth/callback"));
    }
}
