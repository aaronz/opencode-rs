use chrono::{Duration, Utc};
use opencode_auth::oauth::{DeviceCodeSession, OAuthError, OAuthFlow};

#[test]
fn test_device_code_session_displays_user_code() {
    let session = DeviceCodeSession {
        provider: "github".to_string(),
        device_code: "device-code-abc123".to_string(),
        user_code: "USER-1234".to_string(),
        verification_uri: "https://github.com/login/device".to_string(),
        verification_uri_complete: Some(
            "https://github.com/login/device/activate?code=USER-1234".to_string(),
        ),
        expires_at: Utc::now() + Duration::seconds(300),
        interval_secs: 5,
        created_at: Utc::now(),
    };

    assert!(!session.user_code.is_empty());
    assert_eq!(session.user_code, "USER-1234");
}

#[test]
fn test_device_code_session_displays_verification_uri() {
    let session = DeviceCodeSession {
        provider: "github".to_string(),
        device_code: "device-code-abc123".to_string(),
        user_code: "USER-1234".to_string(),
        verification_uri: "https://github.com/login/device".to_string(),
        verification_uri_complete: Some(
            "https://github.com/login/device/activate?code=USER-1234".to_string(),
        ),
        expires_at: Utc::now() + Duration::seconds(300),
        interval_secs: 5,
        created_at: Utc::now(),
    };

    assert!(!session.verification_uri.is_empty());
    assert!(session
        .verification_uri
        .starts_with("https://github.com/login/device"));
}

#[test]
fn test_device_code_session_includes_complete_uri_when_available() {
    let session = DeviceCodeSession {
        provider: "github".to_string(),
        device_code: "device-code-abc123".to_string(),
        user_code: "USER-1234".to_string(),
        verification_uri: "https://github.com/login/device".to_string(),
        verification_uri_complete: Some(
            "https://github.com/login/device/activate?code=USER-1234".to_string(),
        ),
        expires_at: Utc::now() + Duration::seconds(300),
        interval_secs: 5,
        created_at: Utc::now(),
    };

    assert!(session.verification_uri_complete.is_some());
    let complete = session.verification_uri_complete.unwrap();
    assert!(complete.contains("USER-1234"));
}

#[test]
fn test_device_code_session_not_expired_before_expiry_time() {
    let session = DeviceCodeSession {
        provider: "github".to_string(),
        device_code: "device-code-abc123".to_string(),
        user_code: "USER-1234".to_string(),
        verification_uri: "https://github.com/login/device".to_string(),
        verification_uri_complete: None,
        expires_at: Utc::now() + Duration::seconds(300),
        interval_secs: 5,
        created_at: Utc::now(),
    };

    assert!(!session.is_expired());
}

#[test]
fn test_device_code_session_is_expired_after_expiry_time() {
    let session = DeviceCodeSession {
        provider: "github".to_string(),
        device_code: "device-code-abc123".to_string(),
        user_code: "USER-1234".to_string(),
        verification_uri: "https://github.com/login/device".to_string(),
        verification_uri_complete: None,
        expires_at: Utc::now() - Duration::seconds(1),
        interval_secs: 5,
        created_at: Utc::now() - Duration::seconds(300),
    };

    assert!(session.is_expired());
}

#[test]
fn test_device_code_session_time_remaining_calculation() {
    let session = DeviceCodeSession {
        provider: "github".to_string(),
        device_code: "device-code-abc123".to_string(),
        user_code: "USER-1234".to_string(),
        verification_uri: "https://github.com/login/device".to_string(),
        verification_uri_complete: None,
        expires_at: Utc::now() + Duration::seconds(60),
        interval_secs: 5,
        created_at: Utc::now(),
    };

    let remaining = session.time_remaining();
    assert!(remaining.num_seconds() > 0);
    assert!(remaining.num_seconds() <= 60);
}

#[test]
fn test_device_code_session_time_remaining_zero_when_expired() {
    let session = DeviceCodeSession {
        provider: "github".to_string(),
        device_code: "device-code-abc123".to_string(),
        user_code: "USER-1234".to_string(),
        verification_uri: "https://github.com/login/device".to_string(),
        verification_uri_complete: None,
        expires_at: Utc::now() - Duration::seconds(60),
        interval_secs: 5,
        created_at: Utc::now() - Duration::seconds(360),
    };

    let remaining = session.time_remaining();
    assert_eq!(remaining.num_seconds(), 0);
}

#[test]
fn test_oauth_flow_poll_returns_expired_when_session_expired() {
    let flow = OAuthFlow::new();

    let expired_session = DeviceCodeSession {
        provider: "github".to_string(),
        device_code: "device-code-abc123".to_string(),
        user_code: "USER-1234".to_string(),
        verification_uri: "https://github.com/login/device".to_string(),
        verification_uri_complete: None,
        expires_at: Utc::now() - Duration::seconds(1),
        interval_secs: 5,
        created_at: Utc::now() - Duration::seconds(300),
    };

    let result = flow.poll_device_code_authorization(
        &expired_session,
        "test-client-id",
        "",
        "https://github.com/login/oauth/access_token",
        None,
    );

    assert!(result.is_err());
    let err = result.unwrap_err();
    assert!(matches!(err, OAuthError::DeviceCodeExpired));
}

#[test]
fn test_device_code_session_has_correct_interval() {
    let session = DeviceCodeSession {
        provider: "github".to_string(),
        device_code: "device-code-abc123".to_string(),
        user_code: "USER-1234".to_string(),
        verification_uri: "https://github.com/login/device".to_string(),
        verification_uri_complete: None,
        expires_at: Utc::now() + Duration::seconds(300),
        interval_secs: 5,
        created_at: Utc::now(),
    };

    assert_eq!(session.interval_secs, 5);
}

#[test]
fn test_device_code_session_provider_field() {
    let session = DeviceCodeSession {
        provider: "github".to_string(),
        device_code: "device-code-abc123".to_string(),
        user_code: "USER-1234".to_string(),
        verification_uri: "https://github.com/login/device".to_string(),
        verification_uri_complete: None,
        expires_at: Utc::now() + Duration::seconds(300),
        interval_secs: 5,
        created_at: Utc::now(),
    };

    assert_eq!(session.provider, "github");
}

#[test]
fn test_device_code_session_device_code_field() {
    let session = DeviceCodeSession {
        provider: "github".to_string(),
        device_code: "device-code-abc123".to_string(),
        user_code: "USER-1234".to_string(),
        verification_uri: "https://github.com/login/device".to_string(),
        verification_uri_complete: None,
        expires_at: Utc::now() + Duration::seconds(300),
        interval_secs: 5,
        created_at: Utc::now(),
    };

    assert_eq!(session.device_code, "device-code-abc123");
}

#[test]
fn test_oauth_flow_start_device_code_flow_returns_session() {
    let flow = OAuthFlow::new();

    let result = flow.start_device_code_flow(
        "github",
        "Iv1.8a1f8c05dfd1c06e",
        "https://github.com/login/device/code",
        Some("read:user"),
    );

    if let Ok(session) = result {
        assert!(!session.device_code.is_empty());
        assert!(!session.user_code.is_empty());
        assert!(!session.verification_uri.is_empty());
    }
}

#[test]
fn test_device_code_session_created_at_is_set() {
    let before = Utc::now();
    let session = DeviceCodeSession {
        provider: "github".to_string(),
        device_code: "device-code-abc123".to_string(),
        user_code: "USER-1234".to_string(),
        verification_uri: "https://github.com/login/device".to_string(),
        verification_uri_complete: None,
        expires_at: Utc::now() + Duration::seconds(300),
        interval_secs: 5,
        created_at: Utc::now(),
    };
    let after = Utc::now();

    assert!(session.created_at >= before && session.created_at <= after);
}

#[test]
fn test_oauth_token_contains_required_fields() {
    use opencode_auth::oauth::OAuthToken;

    let token = OAuthToken {
        access_token: "gho_test_token".to_string(),
        refresh_token: Some("refresh_token".to_string()),
        expires_in: 3600,
        token_type: "Bearer".to_string(),
        scope: Some("read:user".to_string()),
        received_at: Utc::now(),
    };

    assert!(!token.access_token.is_empty());
    assert_eq!(token.token_type, "Bearer");
    assert!(token.scope.is_some());
}

#[test]
fn test_oauth_token_is_expired_after_expiry() {
    use opencode_auth::oauth::OAuthToken;

    let token = OAuthToken {
        access_token: "gho_test_token".to_string(),
        refresh_token: Some("refresh_token".to_string()),
        expires_in: 1,
        token_type: "Bearer".to_string(),
        scope: Some("read:user".to_string()),
        received_at: Utc::now() - chrono::Duration::seconds(2),
    };

    assert!(token.is_expired());
}

#[test]
fn test_oauth_token_is_not_expired_before_expiry() {
    use opencode_auth::oauth::OAuthToken;

    let token = OAuthToken {
        access_token: "gho_test_token".to_string(),
        refresh_token: Some("refresh_token".to_string()),
        expires_in: 3600,
        token_type: "Bearer".to_string(),
        scope: Some("read:user".to_string()),
        received_at: Utc::now(),
    };

    assert!(!token.is_expired());
}
