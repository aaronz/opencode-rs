use opencode_tui::app::validate_api_key;
use opencode_tui::app::{ApiKeyValidationError, ApiKeyValidationErrorType, ConnectEvent};
use opencode_tui::{App, AppMode};
use std::sync::mpsc;

#[tokio::test]
async fn test_validate_api_key_empty_key_returns_error() {
    let result = validate_api_key("anthropic", "").await;
    assert!(result.is_err());
    let error = result.unwrap_err();
    assert_eq!(error.error_type, ApiKeyValidationErrorType::EmptyKey);
    assert!(error.message.contains("empty"));
}

#[tokio::test]
async fn test_validate_api_key_openai_empty_key_returns_error() {
    let result = validate_api_key("openai", "").await;
    assert!(result.is_err());
    let error = result.unwrap_err();
    assert_eq!(error.error_type, ApiKeyValidationErrorType::EmptyKey);
}

#[tokio::test]
async fn test_validate_api_key_anthropic_invalid_key_returns_auth_error() {
    let result = validate_api_key("anthropic", "invalid-key-not-real").await;
    assert!(result.is_err());
    let error = result.unwrap_err();
    assert_eq!(error.error_type, ApiKeyValidationErrorType::AuthenticationError);
    assert!(error.message.to_lowercase().contains("authentication")
        || error.message.to_lowercase().contains("invalid")
        || error.message.to_lowercase().contains("api key"));
}

#[tokio::test]
async fn test_validate_api_key_openai_invalid_key_returns_auth_error() {
    let result = validate_api_key("openai", "invalid-key-not-real").await;
    assert!(result.is_err());
    let error = result.unwrap_err();
    assert_eq!(error.error_type, ApiKeyValidationErrorType::AuthenticationError);
    assert!(error.message.to_lowercase().contains("authentication")
        || error.message.to_lowercase().contains("invalid")
        || error.message.to_lowercase().contains("api key"));
}

#[tokio::test]
async fn test_validate_api_key_nonexistent_provider_invalid_key() {
    let result = validate_api_key("nonexistent_provider", "sk-test-invalid-key-12345").await;
    assert!(result.is_err());
    let error = result.unwrap_err();
    assert!(matches!(
        error.error_type,
        ApiKeyValidationErrorType::AuthenticationError
            | ApiKeyValidationErrorType::NetworkError
            | ApiKeyValidationErrorType::ServerError
    ));
}

#[tokio::test]
async fn test_validate_api_key_error_display() {
    let error = ApiKeyValidationError {
        message: "Test error message".to_string(),
        error_type: ApiKeyValidationErrorType::AuthenticationError,
        status_code: Some(401),
    };
    assert_eq!(error.to_string(), "Test error message");
}

#[tokio::test]
async fn test_validate_api_key_timeout_error_type() {
    let result = validate_api_key("anthropic", "timeout-test-key").await;
    assert!(result.is_err());
    let error = result.unwrap_err();
    assert!(matches!(
        error.error_type,
        ApiKeyValidationErrorType::AuthenticationError
            | ApiKeyValidationErrorType::NetworkError
            | ApiKeyValidationErrorType::Timeout
    ));
}

#[tokio::test]
async fn test_validate_api_key_error_types_are_correct() {
    let empty_error = ApiKeyValidationError {
        message: "empty".to_string(),
        error_type: ApiKeyValidationErrorType::EmptyKey,
        status_code: None,
    };
    assert_eq!(empty_error.error_type, ApiKeyValidationErrorType::EmptyKey);

    let network_error = ApiKeyValidationError {
        message: "network".to_string(),
        error_type: ApiKeyValidationErrorType::NetworkError,
        status_code: None,
    };
    assert_eq!(network_error.error_type, ApiKeyValidationErrorType::NetworkError);

    let auth_error = ApiKeyValidationError {
        message: "auth".to_string(),
        error_type: ApiKeyValidationErrorType::AuthenticationError,
        status_code: Some(401),
    };
    assert_eq!(auth_error.error_type, ApiKeyValidationErrorType::AuthenticationError);

    let timeout_error = ApiKeyValidationError {
        message: "timeout".to_string(),
        error_type: ApiKeyValidationErrorType::Timeout,
        status_code: None,
    };
    assert_eq!(timeout_error.error_type, ApiKeyValidationErrorType::Timeout);

    let server_error = ApiKeyValidationError {
        message: "server".to_string(),
        error_type: ApiKeyValidationErrorType::ServerError,
        status_code: Some(500),
    };
    assert_eq!(server_error.error_type, ApiKeyValidationErrorType::ServerError);
}

#[test]
fn test_validation_in_progress_initial_state_is_false() {
    let app = App::new();
    assert!(!app.validation_in_progress, "validation_in_progress should be false initially");
}

#[test]
fn test_validation_in_progress_set_during_validation() {
    let mut app = App::new();
    app.pending_connect_provider = Some("openai".to_string());
    app.pending_api_key_for_validation = Some("sk-test-api-key-12345".to_string());
    app.validation_in_progress = true;
    app.mode = AppMode::ConnectProgress;
    assert!(app.validation_in_progress, "validation_in_progress should be true during validation");
    assert_eq!(app.mode, AppMode::ConnectProgress, "mode should be ConnectProgress during validation");
}

#[test]
fn test_validation_in_progress_clears_after_validation_completes() {
    let mut app = App::new();
    let (tx, rx) = mpsc::channel();
    app.connect_rx = Some(rx);
    app.validation_in_progress = true;
    app.mode = AppMode::ConnectProgress;

    let _ = tx.send(ConnectEvent::ValidationComplete {
        success: true,
        error_message: None,
    });

    app.check_connect_events_for_testing();
    assert!(!app.validation_in_progress, "validation_in_progress should be cleared after validation completes");
    assert_eq!(app.mode, AppMode::Chat);
}

#[test]
fn test_input_is_disabled_during_validation() {
    let mut app = App::new();
    app.validation_in_progress = true;
    app.mode = AppMode::ConnectProgress;
    assert_eq!(app.mode, AppMode::ConnectProgress, "ConnectProgress mode should disable normal input handling");
}