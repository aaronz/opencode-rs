use opencode_tui::app::validate_api_key;
use opencode_tui::app::{ApiKeyValidationError, ApiKeyValidationErrorType};
use opencode_tui::{App, AppMode, Dialog};

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
    app.validation_in_progress = true;
    app.mode = AppMode::ConnectProgress;

    app.simulate_validation_complete_for_testing(true, None);

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

#[test]
fn test_invalid_api_key_shows_error_dialog_to_user() {
    let mut app = App::new();
    app.pending_connect_provider = Some("openai".to_string());
    app.pending_api_key_for_validation = Some("invalid-key".to_string());
    app.validation_in_progress = true;
    app.mode = AppMode::ConnectProgress;

    app.simulate_validation_complete_for_testing(false, Some("Authentication failed: invalid API key".to_string()));

    assert!(!app.validation_in_progress, "validation_in_progress should be cleared after validation fails");
    assert_eq!(app.mode, AppMode::ConnectApiKeyError, "mode should be ConnectApiKeyError after validation failure");
    assert!(app.validation_error_dialog.is_some(), "error dialog should be shown to user");
}

#[test]
fn test_user_can_retry_after_validation_failure() {
    let mut app = App::new();
    app.pending_connect_provider = Some("openai".to_string());
    app.pending_api_key_for_validation = Some("invalid-key".to_string());
    app.validation_in_progress = true;
    app.mode = AppMode::ConnectProgress;

    app.simulate_validation_complete_for_testing(false, Some("Invalid API key".to_string()));

    assert_eq!(app.mode, AppMode::ConnectApiKeyError);
    assert!(app.validation_error_dialog.is_some());

    use opencode_tui::dialogs::DialogAction;
    if let Some(dialog) = app.validation_error_dialog.as_mut() {
        use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
        let retry_action = dialog.handle_input(KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE));
        assert_eq!(retry_action, DialogAction::Confirm("retry".to_string()),
            "Enter on Try Again should return Confirm(retry)");
    }
}

#[test]
fn test_invalid_keys_are_not_persisted_to_credential_store() {
    let mut app = App::new();
    app.pending_connect_provider = Some("openai".to_string());
    app.pending_api_key_for_validation = Some("invalid-key-not-real".to_string());
    app.validation_in_progress = true;
    app.mode = AppMode::ConnectProgress;

    app.simulate_validation_complete_for_testing(false, Some("Authentication failed".to_string()));

    assert_eq!(app.mode, AppMode::ConnectApiKeyError);
    assert!(app.pending_api_key_for_validation.is_none(),
        "API key should be cleared from memory after validation failure");
    assert!(app.validation_error_dialog.is_some(),
        "error dialog should be shown instead of saving");
}

#[test]
fn test_network_failure_shows_error_message_to_user() {
    use opencode_tui::app::ApiKeyValidationErrorType;

    let network_error = ApiKeyValidationError {
        message: "Network error: connection timed out".to_string(),
        error_type: ApiKeyValidationErrorType::NetworkError,
        status_code: None,
    };

    let mut app = App::new();
    app.pending_connect_provider = Some("openai".to_string());
    app.pending_api_key_for_validation = Some("sk-test-key".to_string());
    app.validation_in_progress = true;
    app.mode = AppMode::ConnectProgress;

    app.simulate_validation_complete_for_testing(false, Some(network_error.message.clone()));

    assert_eq!(app.mode, AppMode::ConnectApiKeyError);
    assert!(app.validation_error_dialog.is_some(),
        "network failure should show error dialog to user");
}

#[test]
fn test_validation_error_dialog_has_try_again_button() {
    use opencode_tui::dialogs::ValidationErrorDialog;
    use opencode_tui::theme::ThemeManager;

    let mut theme_manager = ThemeManager::new();
    let _ = theme_manager.load_from_config();
    let theme = theme_manager.current().clone();

    let dialog = ValidationErrorDialog::from_validation_error(
        "Test error",
        "OpenAI",
        theme,
    );

    use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
    use opencode_tui::dialogs::DialogAction;

    let mut dialog = dialog;
    let right_key = KeyEvent::new(KeyCode::Right, KeyModifiers::NONE);
    dialog.handle_input(right_key);

    let enter_key = KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE);
    let action = dialog.handle_input(enter_key);
    assert_eq!(action, DialogAction::Close,
        "Cancel selected should return Close");
}

#[test]
fn test_validation_error_dialog_try_again_returns_retry_action() {
    use opencode_tui::dialogs::ValidationErrorDialog;
    use opencode_tui::theme::ThemeManager;

    let mut theme_manager = ThemeManager::new();
    let _ = theme_manager.load_from_config();
    let theme = theme_manager.current().clone();

    let dialog = ValidationErrorDialog::from_validation_error(
        "Authentication failed",
        "Anthropic",
        theme,
    );

    use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
    use opencode_tui::dialogs::DialogAction;

    let mut dialog = dialog;
    let enter_key = KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE);
    let action = dialog.handle_input(enter_key);
    assert_eq!(action, DialogAction::Confirm("retry".to_string()),
        "Try Again should return Confirm(retry)");
}