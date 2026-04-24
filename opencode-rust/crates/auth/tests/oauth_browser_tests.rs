use opencode_auth::oauth::OAuthFlow;

#[test]
fn test_pkce_code_verifier_generation() {
    let verifier = OAuthFlow::generate_code_verifier();
    assert!(!verifier.is_empty(), "Code verifier should not be empty");
    assert!(
        verifier.len() >= 43,
        "Code verifier should be at least 43 chars"
    );
}

#[test]
fn test_pkce_code_challenge_generation() {
    let verifier = OAuthFlow::generate_code_verifier();
    let challenge = OAuthFlow::generate_code_challenge(&verifier);
    assert!(!challenge.is_empty(), "Code challenge should not be empty");
    assert!(
        challenge.len() >= 43,
        "Code challenge should be at least 43 chars"
    );
}

#[test]
fn test_pkce_deterministic_challenge() {
    let verifier = "test_verifier_123456789012345678901234567890";
    let challenge1 = OAuthFlow::generate_code_challenge(verifier);
    let challenge2 = OAuthFlow::generate_code_challenge(verifier);
    assert_eq!(
        challenge1, challenge2,
        "Same verifier should produce same challenge"
    );
}

#[test]
fn test_pkce_different_verifiers() {
    let verifier1 = "test_verifier_123456789012345678901234567890";
    let verifier2 = "test_verifier_987654321098765432109876543210";
    let challenge1 = OAuthFlow::generate_code_challenge(verifier1);
    let challenge2 = OAuthFlow::generate_code_challenge(verifier2);
    assert_ne!(
        challenge1, challenge2,
        "Different verifiers should produce different challenges"
    );
}

#[test]
fn test_oauth_flow_init() {
    let _flow = OAuthFlow::new();
    let verifier = OAuthFlow::generate_code_verifier();
    assert!(
        !verifier.is_empty(),
        "OAuthFlow should be able to generate PKCE verifier"
    );
}

#[test]
fn test_localhost_callback_server_available() {
    use std::net::TcpListener;
    let listener = TcpListener::bind("127.0.0.1:0");
    assert!(
        listener.is_ok(),
        "Should be able to bind to localhost for callback"
    );
}

#[test]
fn test_oauth_token_expiration() {
    use chrono::Utc;
    use opencode_auth::oauth::OAuthToken;

    let token = OAuthToken {
        access_token: "test-token".to_string(),
        refresh_token: Some("refresh".to_string()),
        expires_in: 3600,
        token_type: "Bearer".to_string(),
        scope: Some("read write".to_string()),
        received_at: Utc::now(),
    };

    assert!(!token.is_expired(), "New token should not be expired");
    assert_eq!(token.token_type, "Bearer");
}

#[test]
fn test_oauth_token_expired() {
    use chrono::{Duration, Utc};
    use opencode_auth::oauth::OAuthToken;

    let token = OAuthToken {
        access_token: "test-token".to_string(),
        refresh_token: None,
        expires_in: 1,
        token_type: "Bearer".to_string(),
        scope: None,
        received_at: Utc::now() - Duration::hours(2),
    };

    assert!(token.is_expired(), "Old token should be expired");
}

#[test]
fn test_oauth_token_expires_at_calculation() {
    use chrono::{Duration, Utc};
    use opencode_auth::oauth::OAuthToken;

    let now = Utc::now();
    let token = OAuthToken {
        access_token: "test-token".to_string(),
        refresh_token: None,
        expires_in: 3600,
        token_type: "Bearer".to_string(),
        scope: None,
        received_at: now,
    };

    let expected = now + Duration::seconds(3600);
    assert_eq!(token.expires_at(), expected);
}

#[test]
fn test_pkce_s256_method() {
    let verifier = OAuthFlow::generate_code_verifier();
    let challenge = OAuthFlow::generate_code_challenge(&verifier);
    assert_ne!(
        verifier, challenge,
        "S256 challenge should differ from verifier"
    );
    assert!(
        challenge.len() >= 43,
        "S256 challenge should be base64url encoded"
    );
}
