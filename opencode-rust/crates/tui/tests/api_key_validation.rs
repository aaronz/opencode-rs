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
    assert!(
        matches!(
            error.error_type,
            ApiKeyValidationErrorType::AuthenticationError
                | ApiKeyValidationErrorType::NetworkError
        ),
        "Expected auth or network error for invalid Anthropic key, got: {:?}",
        error.error_type
    );
    assert!(
        error.message.to_lowercase().contains("authentication")
            || error.message.to_lowercase().contains("invalid")
            || error.message.to_lowercase().contains("api key")
            || error.message.to_lowercase().contains("network")
    );
}

#[tokio::test]
async fn test_validate_api_key_openai_invalid_key_returns_auth_error() {
    let result = validate_api_key("openai", "invalid-key-not-real").await;
    assert!(result.is_err());
    let error = result.unwrap_err();
    assert!(
        matches!(
            error.error_type,
            ApiKeyValidationErrorType::AuthenticationError
                | ApiKeyValidationErrorType::NetworkError
        ),
        "Expected auth or network error for invalid OpenAI key, got: {:?}",
        error.error_type
    );
    assert!(
        error.message.to_lowercase().contains("authentication")
            || error.message.to_lowercase().contains("invalid")
            || error.message.to_lowercase().contains("api key")
            || error.message.to_lowercase().contains("network")
    );
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
    assert_eq!(
        network_error.error_type,
        ApiKeyValidationErrorType::NetworkError
    );

    let auth_error = ApiKeyValidationError {
        message: "auth".to_string(),
        error_type: ApiKeyValidationErrorType::AuthenticationError,
        status_code: Some(401),
    };
    assert_eq!(
        auth_error.error_type,
        ApiKeyValidationErrorType::AuthenticationError
    );

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
    assert_eq!(
        server_error.error_type,
        ApiKeyValidationErrorType::ServerError
    );
}

#[test]
fn test_validation_in_progress_initial_state_is_false() {
    let app = App::new();
    assert!(
        !app.validation_in_progress,
        "validation_in_progress should be false initially"
    );
}

#[test]
fn test_validation_in_progress_set_during_validation() {
    let mut app = App::new();
    app.pending_connect_provider = Some("openai".to_string());
    app.pending_api_key_for_validation = Some("sk-test-api-key-12345".to_string());
    app.validation_in_progress = true;
    app.mode = AppMode::ConnectProgress;
    assert!(
        app.validation_in_progress,
        "validation_in_progress should be true during validation"
    );
    assert_eq!(
        app.mode,
        AppMode::ConnectProgress,
        "mode should be ConnectProgress during validation"
    );
}

#[test]
fn test_connect_progress_message_mentions_provider_during_api_key_validation() {
    let mut app = App::new();
    app.pending_connect_provider = Some("openai".to_string());
    app.validation_in_progress = true;
    app.mode = AppMode::ConnectProgress;

    assert_eq!(
        app.get_connect_progress_message_for_testing(),
        "Validating OpenAI API key...",
        "progress message should identify the provider while validation is running"
    );
}

#[test]
fn test_connect_progress_message_prompts_for_browser_auth_completion() {
    let mut app = App::new();
    app.pending_connect_provider = Some("google".to_string());
    app.validation_in_progress = false;
    app.mode = AppMode::ConnectProgress;

    assert_eq!(
        app.get_connect_progress_message_for_testing(),
        "Complete Google authentication in your browser...",
        "progress message should explain the current browser-auth step"
    );
}

#[test]
fn test_connect_progress_syncs_shared_status_bar_activity() {
    let mut app = App::new();
    app.pending_connect_provider = Some("openai".to_string());
    app.validation_in_progress = true;
    app.mode = AppMode::ConnectProgress;

    app.sync_status_bar_state_for_testing();

    assert_eq!(
        app.status_bar.activity_message.as_deref(),
        Some("⏳ Validating OpenAI"),
        "shared status bar should reflect connect validation progress"
    );
    assert_eq!(
        app.status_bar.connection_status,
        opencode_tui::components::status_bar::ConnectionStatus::Disconnected,
        "shared status bar should show disconnected while validation is still in progress"
    );
}

#[test]
fn test_shared_status_bar_activity_shows_ready_when_chat_mode_idle() {
    let mut app = App::new();
    app.status_bar.activity_message = Some("⏳ Validating OpenAI".to_string());
    app.status_bar.connection_status =
        opencode_tui::components::status_bar::ConnectionStatus::Disconnected;
    app.mode = AppMode::Chat;
    app.validation_in_progress = false;

    app.sync_status_bar_state_for_testing();

    assert_eq!(
        app.status_bar.activity_message.as_deref(),
        Some("✓ Ready"),
        "shared status bar should show 'Ready' when Chat mode with Idle state, not clear the activity"
    );
    assert_eq!(
        app.status_bar.connection_status,
        opencode_tui::components::status_bar::ConnectionStatus::Connected,
        "shared status bar should return to connected when connect progress is inactive"
    );
}

#[test]
fn test_validation_error_syncs_shared_status_bar_error_state() {
    let mut app = App::new();
    app.pending_connect_provider = Some("openai".to_string());
    app.mode = AppMode::ConnectApiKeyError;

    app.sync_status_bar_state_for_testing();

    assert_eq!(
        app.status_bar.connection_status,
        opencode_tui::components::status_bar::ConnectionStatus::Error,
        "shared status bar should show error when API key validation fails"
    );
    assert_eq!(
        app.status_bar.activity_message.as_deref(),
        Some("⚠ Fix OpenAI API key"),
        "shared status bar should suggest the next recovery step after validation failure"
    );
}

#[test]
fn test_connect_api_key_mode_keeps_shared_status_bar_disconnected() {
    let mut app = App::new();
    app.pending_connect_provider = Some("anthropic".to_string());
    app.mode = AppMode::ConnectApiKey;

    app.sync_status_bar_state_for_testing();

    assert_eq!(
        app.status_bar.connection_status,
        opencode_tui::components::status_bar::ConnectionStatus::Disconnected,
        "shared status bar should stay disconnected while the user is still entering credentials"
    );
}

#[test]
fn test_connect_provider_mode_shows_select_provider_activity() {
    let mut app = App::new();
    app.mode = AppMode::ConnectProvider;

    app.sync_status_bar_state_for_testing();

    assert_eq!(
        app.status_bar.activity_message.as_deref(),
        Some("🔌 Select provider"),
        "shared status bar should explain the provider-selection step"
    );
}

#[test]
fn test_connect_method_mode_shows_choose_auth_activity() {
    let mut app = App::new();
    app.pending_connect_provider = Some("openai".to_string());
    app.mode = AppMode::ConnectMethod;

    app.sync_status_bar_state_for_testing();

    assert_eq!(
        app.status_bar.activity_message.as_deref(),
        Some("🔐 Choose OpenAI auth"),
        "shared status bar should explain the auth-method selection step"
    );
}

#[test]
fn test_connect_api_key_mode_shows_enter_key_activity() {
    let mut app = App::new();
    app.pending_connect_provider = Some("anthropic".to_string());
    app.mode = AppMode::ConnectApiKey;

    app.sync_status_bar_state_for_testing();

    assert_eq!(
        app.status_bar.activity_message.as_deref(),
        Some("🔑 Enter Anthropic API key"),
        "shared status bar should explain the credential-entry step"
    );
}

#[test]
fn test_connect_model_mode_shows_select_model_activity() {
    let mut app = App::new();
    app.pending_connect_provider = Some("openai".to_string());
    app.mode = AppMode::ConnectModel;

    app.sync_status_bar_state_for_testing();

    assert_eq!(
        app.status_bar.activity_message.as_deref(),
        Some("🤖 Select OpenAI model"),
        "shared status bar should explain the model-selection step"
    );
}

#[test]
fn test_reconnecting_tui_state_shows_shared_status_bar_activity() {
    let mut app = App::new();
    app.start_reconnecting();
    app.mode = AppMode::Chat;

    app.sync_status_bar_state_for_testing();

    assert_eq!(
        app.status_bar.activity_message.as_deref(),
        Some("🔄 Reconnecting"),
        "shared status bar should surface reconnecting activity outside the connect wizard"
    );
    assert_eq!(
        app.status_bar.connection_status,
        opencode_tui::components::status_bar::ConnectionStatus::Disconnected,
        "shared status bar should show a disconnected indicator while reconnecting"
    );
}

#[test]
fn test_streaming_tui_state_shows_shared_status_bar_activity() {
    let mut app = App::new();
    app.start_llm_generation();
    app.mode = AppMode::Chat;

    app.sync_status_bar_state_for_testing();

    assert_eq!(
        app.status_bar.activity_message.as_deref(),
        Some("✍ Streaming response"),
        "shared status bar should surface streaming activity during normal chat mode"
    );
}

#[test]
fn test_aborting_tui_state_shows_shared_status_bar_activity() {
    let mut app = App::new();
    app.set_tui_state(opencode_tui::app::TuiState::Aborting);
    app.mode = AppMode::Chat;

    app.sync_status_bar_state_for_testing();

    assert_eq!(
        app.status_bar.activity_message.as_deref(),
        Some("🛑 Aborting"),
        "shared status bar should surface aborting activity during cancellation"
    );
}

#[test]
fn test_showing_error_tui_state_shows_shared_status_bar_activity() {
    let mut app = App::new();
    app.set_tui_state(opencode_tui::app::TuiState::ShowingError);
    app.mode = AppMode::Chat;

    app.sync_status_bar_state_for_testing();

    assert_eq!(
        app.status_bar.activity_message.as_deref(),
        Some("⚠ Review error"),
        "shared status bar should surface error review activity outside connect flows"
    );
    assert_eq!(
        app.status_bar.connection_status,
        opencode_tui::components::status_bar::ConnectionStatus::Error,
        "shared status bar should switch to error state while reviewing a non-connect error"
    );
}

#[test]
fn test_showing_error_tui_state_prefers_failed_tool_name_when_available() {
    let mut app = App::new();
    app.set_tui_state(opencode_tui::app::TuiState::ShowingError);
    app.mode = AppMode::Chat;
    app.tool_calls
        .push(opencode_tui::app::ToolCall::new("grep").failed(1, "boom"));

    app.sync_status_bar_state_for_testing();

    assert_eq!(
        app.status_bar.activity_message.as_deref(),
        Some("⚠ Tool failed: grep"),
        "shared status bar should prefer the most recent failed tool name during error review"
    );
}

#[test]
fn test_submitting_tui_state_shows_shared_status_bar_activity() {
    let mut app = App::new();
    app.set_tui_state(opencode_tui::app::TuiState::Submitting);
    app.mode = AppMode::Chat;

    app.sync_status_bar_state_for_testing();

    assert_eq!(
        app.status_bar.activity_message.as_deref(),
        Some("📤 Submitting prompt"),
        "shared status bar should surface prompt submission activity"
    );
}

#[test]
fn test_executing_tool_tui_state_shows_shared_status_bar_activity() {
    let mut app = App::new();
    app.set_tui_state(opencode_tui::app::TuiState::ExecutingTool);
    app.mode = AppMode::Chat;

    app.sync_status_bar_state_for_testing();

    assert_eq!(
        app.status_bar.activity_message.as_deref(),
        Some("🛠 Executing tool"),
        "shared status bar should surface active tool execution"
    );
}

#[test]
fn test_executing_tool_tui_state_shows_active_tool_name_when_available() {
    let mut app = App::new();
    app.set_tui_state(opencode_tui::app::TuiState::ExecutingTool);
    app.mode = AppMode::Chat;
    app.tool_calls.push(opencode_tui::app::ToolCall::new("grep"));

    app.sync_status_bar_state_for_testing();

    assert_eq!(
        app.status_bar.activity_message.as_deref(),
        Some("🛠 Running grep"),
        "shared status bar should prefer the active tool name over the generic fallback"
    );
}

#[test]
fn test_executing_tool_tui_state_keeps_generic_message_without_running_tool() {
    let mut app = App::new();
    app.set_tui_state(opencode_tui::app::TuiState::ExecutingTool);
    app.mode = AppMode::Chat;
    app.tool_calls
        .push(opencode_tui::app::ToolCall::new("read").success("done"));

    app.sync_status_bar_state_for_testing();

    assert_eq!(
        app.status_bar.activity_message.as_deref(),
        Some("🛠 Executing tool"),
        "shared status bar should fall back to the generic message when no tool is currently running"
    );
}

#[test]
fn test_awaiting_permission_tui_state_shows_shared_status_bar_activity() {
    let mut app = App::new();
    app.set_tui_state(opencode_tui::app::TuiState::AwaitingPermission);
    app.mode = AppMode::Chat;

    app.sync_status_bar_state_for_testing();

    assert_eq!(
        app.status_bar.activity_message.as_deref(),
        Some("✋ Awaiting permission"),
        "shared status bar should surface approval-blocked activity"
    );
}

#[test]
fn test_paused_tui_state_shows_shared_status_bar_activity() {
    let mut app = App::new();
    app.set_tui_state(opencode_tui::app::TuiState::Paused);
    app.mode = AppMode::Chat;

    app.sync_status_bar_state_for_testing();

    assert_eq!(
        app.status_bar.activity_message.as_deref(),
        Some("⏸ Paused"),
        "shared status bar should surface paused activity"
    );
}

#[test]
fn test_showing_diff_tui_state_shows_shared_status_bar_activity() {
    let mut app = App::new();
    app.set_tui_state(opencode_tui::app::TuiState::ShowingDiff);
    app.mode = AppMode::Chat;

    app.sync_status_bar_state_for_testing();

    assert_eq!(
        app.status_bar.activity_message.as_deref(),
        Some("🧾 Reviewing diff"),
        "shared status bar should surface diff review activity"
    );
}

#[test]
fn test_composing_tui_state_shows_shared_status_bar_activity() {
    let mut app = App::new();
    app.set_tui_state(opencode_tui::app::TuiState::Composing);
    app.mode = AppMode::Chat;

    app.sync_status_bar_state_for_testing();

    assert_eq!(
        app.status_bar.activity_message.as_deref(),
        Some("✎ Composing"),
        "shared status bar should surface composing activity"
    );
}

#[test]
fn test_idle_tui_state_shows_shared_status_bar_activity() {
    let mut app = App::new();
    app.set_tui_state(opencode_tui::app::TuiState::Idle);
    app.mode = AppMode::Chat;

    app.sync_status_bar_state_for_testing();

    assert_eq!(
        app.status_bar.activity_message.as_deref(),
        Some("✓ Ready"),
        "shared status bar should surface an explicit idle-ready state"
    );
}

#[test]
fn test_validation_in_progress_clears_after_validation_completes() {
    use opencode_llm::BrowserAuthModelInfo;

    let temp_dir = tempfile::TempDir::new().unwrap();
    std::env::set_var("OPENCODE_DATA_DIR", temp_dir.path().to_str().unwrap());

    let mut app = App::new();
    app.validation_in_progress = true;
    app.mode = AppMode::ConnectProgress;

    let models = Some(vec![
        BrowserAuthModelInfo {
            id: "gpt-4o".to_string(),
            name: "GPT-4o".to_string(),
            variants: vec![],
        },
        BrowserAuthModelInfo {
            id: "gpt-4o-mini".to_string(),
            name: "GPT-4o Mini".to_string(),
            variants: vec![],
        },
    ]);
    app.simulate_validation_complete_for_testing(true, None, models);

    assert!(
        !app.validation_in_progress,
        "validation_in_progress should be cleared after validation completes"
    );
    assert_eq!(
        app.mode,
        AppMode::ConnectModel,
        "mode should be ConnectModel after successful validation to show model selection"
    );
    assert!(
        app.connect_model_dialog.is_some(),
        "model dialog should be shown after successful validation"
    );
}

#[test]
fn test_input_is_disabled_during_validation() {
    let mut app = App::new();
    app.validation_in_progress = true;
    app.mode = AppMode::ConnectProgress;
    assert_eq!(
        app.mode,
        AppMode::ConnectProgress,
        "ConnectProgress mode should disable normal input handling"
    );
}

#[test]
fn test_invalid_api_key_shows_error_dialog_to_user() {
    let mut app = App::new();
    app.pending_connect_provider = Some("openai".to_string());
    app.pending_api_key_for_validation = Some("invalid-key".to_string());
    app.validation_in_progress = true;
    app.mode = AppMode::ConnectProgress;

    app.simulate_validation_complete_for_testing(
        false,
        Some("Authentication failed: invalid API key".to_string()),
        None,
    );

    assert!(
        !app.validation_in_progress,
        "validation_in_progress should be cleared after validation fails"
    );
    assert_eq!(
        app.mode,
        AppMode::ConnectApiKeyError,
        "mode should be ConnectApiKeyError after validation failure"
    );
    assert!(
        app.validation_error_dialog.is_some(),
        "error dialog should be shown to user"
    );
}

#[test]
fn test_user_can_retry_after_validation_failure() {
    let mut app = App::new();
    app.pending_connect_provider = Some("openai".to_string());
    app.pending_api_key_for_validation = Some("invalid-key".to_string());
    app.validation_in_progress = true;
    app.mode = AppMode::ConnectProgress;

    app.simulate_validation_complete_for_testing(false, Some("Invalid API key".to_string()), None);

    assert_eq!(app.mode, AppMode::ConnectApiKeyError);
    assert!(app.validation_error_dialog.is_some());

    use opencode_tui::dialogs::DialogAction;
    if let Some(dialog) = app.validation_error_dialog.as_mut() {
        use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
        let retry_action = dialog.handle_input(KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE));
        assert_eq!(
            retry_action,
            DialogAction::Confirm("retry".to_string()),
            "Enter on Try Again should return Confirm(retry)"
        );
    }
}

#[test]
fn test_invalid_keys_are_not_persisted_to_credential_store() {
    let mut app = App::new();
    app.pending_connect_provider = Some("openai".to_string());
    app.pending_api_key_for_validation = Some("invalid-key-not-real".to_string());
    app.validation_in_progress = true;
    app.mode = AppMode::ConnectProgress;

    app.simulate_validation_complete_for_testing(
        false,
        Some("Authentication failed".to_string()),
        None,
    );

    assert_eq!(app.mode, AppMode::ConnectApiKeyError);
    assert!(
        app.pending_api_key_for_validation.is_none(),
        "API key should be cleared from memory after validation failure"
    );
    assert!(
        app.validation_error_dialog.is_some(),
        "error dialog should be shown instead of saving"
    );
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

    app.simulate_validation_complete_for_testing(false, Some(network_error.message.clone()), None);

    assert_eq!(app.mode, AppMode::ConnectApiKeyError);
    assert!(
        app.validation_error_dialog.is_some(),
        "network failure should show error dialog to user"
    );
}

#[test]
fn test_validation_error_dialog_has_try_again_button() {
    use opencode_tui::dialogs::ValidationErrorDialog;
    use opencode_tui::theme::ThemeManager;

    let mut theme_manager = ThemeManager::new();
    let _ = theme_manager.load_from_config();
    let theme = theme_manager.current().clone();

    let dialog = ValidationErrorDialog::from_validation_error("Test error", "OpenAI", theme);

    use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
    use opencode_tui::dialogs::DialogAction;

    let mut dialog = dialog;
    let right_key = KeyEvent::new(KeyCode::Right, KeyModifiers::NONE);
    dialog.handle_input(right_key);

    let enter_key = KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE);
    let action = dialog.handle_input(enter_key);
    assert_eq!(
        action,
        DialogAction::Close,
        "Cancel selected should return Close"
    );
}

#[test]
fn test_validation_error_dialog_try_again_returns_retry_action() {
    use opencode_tui::dialogs::ValidationErrorDialog;
    use opencode_tui::theme::ThemeManager;

    let mut theme_manager = ThemeManager::new();
    let _ = theme_manager.load_from_config();
    let theme = theme_manager.current().clone();

    let dialog =
        ValidationErrorDialog::from_validation_error("Authentication failed", "Anthropic", theme);

    use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
    use opencode_tui::dialogs::DialogAction;

    let mut dialog = dialog;
    let enter_key = KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE);
    let action = dialog.handle_input(enter_key);
    assert_eq!(
        action,
        DialogAction::Confirm("retry".to_string()),
        "Try Again should return Confirm(retry)"
    );
}

#[tokio::test]
async fn test_anthropic_uses_correct_validation_endpoint() {
    let result = validate_api_key("anthropic", "test-key").await;
    assert!(result.is_err());
    let error = result.unwrap_err();
    assert!(
        matches!(
            error.error_type,
            ApiKeyValidationErrorType::AuthenticationError
                | ApiKeyValidationErrorType::NetworkError
        ),
        "Expected auth or network error for invalid Anthropic key"
    );
}

#[tokio::test]
async fn test_openai_uses_correct_validation_endpoint() {
    let result = validate_api_key("openai", "test-key").await;
    assert!(result.is_err());
    let error = result.unwrap_err();
    assert!(
        matches!(
            error.error_type,
            ApiKeyValidationErrorType::AuthenticationError
                | ApiKeyValidationErrorType::NetworkError
        ),
        "Expected auth or network error for invalid OpenAI key"
    );
}

#[tokio::test]
async fn test_lm_studio_uses_api_tags_endpoint() {
    std::env::set_var("LMSTUDIO_BASE_URL", "http://localhost:1234");
    let result = validate_api_key("lmstudio", "test-key").await;
    std::env::remove_var("LMSTUDIO_BASE_URL");
    assert!(result.is_err());
    let error = result.unwrap_err();
    assert!(
        matches!(error.error_type, ApiKeyValidationErrorType::NetworkError),
        "LM Studio with invalid base URL should cause network error, got: {:?}",
        error.error_type
    );
}

#[tokio::test]
async fn test_lm_studio_variant_names_use_api_tags_endpoint() {
    std::env::set_var("LMSTUDIO_BASE_URL", "http://localhost:1234");
    let result = validate_api_key("lm_studio", "test-key").await;
    std::env::remove_var("LMSTUDIO_BASE_URL");
    assert!(result.is_err());
    let error = result.unwrap_err();
    assert!(
        matches!(error.error_type, ApiKeyValidationErrorType::NetworkError),
        "lm_studio variant should use /api/tags endpoint"
    );
}

#[tokio::test]
async fn test_unknown_provider_falls_back_to_v1_models() {
    std::env::set_var(
        "CUSTOM_PROVIDER_BASE_URL",
        "https://custom-provider.example.com",
    );
    let result = validate_api_key("custom_provider", "test-key").await;
    std::env::remove_var("CUSTOM_PROVIDER_BASE_URL");
    assert!(result.is_err());
    let error = result.unwrap_err();
    assert!(
        matches!(
            error.error_type,
            ApiKeyValidationErrorType::AuthenticationError
                | ApiKeyValidationErrorType::NetworkError
        ),
        "Custom provider should use /v1/models endpoint"
    );
}

#[tokio::test]
async fn test_openai_compatible_uses_models_endpoint() {
    let result = validate_api_key("openai", "test-key").await;
    assert!(result.is_err());
    let error = result.unwrap_err();
    assert!(
        matches!(
            error.error_type,
            ApiKeyValidationErrorType::AuthenticationError
                | ApiKeyValidationErrorType::NetworkError
        ),
        "OpenAI-compatible should use /v1/models endpoint"
    );
}

use opencode_llm::BrowserAuthModelInfo;

#[test]
fn test_connect_model_dialog_shown_after_successful_validation() {
    let temp_dir = tempfile::TempDir::new().unwrap();
    std::env::set_var("OPENCODE_DATA_DIR", temp_dir.path().to_str().unwrap());

    let mut app = App::new();
    app.validation_in_progress = true;
    app.mode = AppMode::ConnectProgress;
    app.pending_connect_provider = Some("openai".to_string());
    app.pending_api_key_for_validation = Some("sk-valid-key".to_string());

    let models = vec![
        BrowserAuthModelInfo {
            id: "gpt-4o".to_string(),
            name: "GPT-4o".to_string(),
            variants: vec![],
        },
        BrowserAuthModelInfo {
            id: "gpt-4o-mini".to_string(),
            name: "GPT-4o Mini".to_string(),
            variants: vec![],
        },
    ];
    app.simulate_validation_complete_for_testing(true, None, Some(models));

    assert_eq!(
        app.mode,
        AppMode::ConnectModel,
        "Mode should be ConnectModel after successful validation"
    );
    assert!(
        app.connect_model_dialog.is_some(),
        "ConnectModelDialog should be shown after successful validation"
    );
    assert!(
        !app.validation_in_progress,
        "validation_in_progress should be cleared"
    );
}

#[test]
fn test_validated_credentials_and_models_passed_to_dialog() {
    let temp_dir = tempfile::TempDir::new().unwrap();
    std::env::set_var("OPENCODE_DATA_DIR", temp_dir.path().to_str().unwrap());

    let mut app = App::new();
    app.validation_in_progress = true;
    app.mode = AppMode::ConnectProgress;
    app.pending_connect_provider = Some("anthropic".to_string());
    app.pending_api_key_for_validation = Some("sk-ant-valid-key".to_string());

    let models = vec![
        BrowserAuthModelInfo {
            id: "claude-sonnet-4-20250514".to_string(),
            name: "Claude Sonnet 4".to_string(),
            variants: vec![],
        },
        BrowserAuthModelInfo {
            id: "claude-haiku-3".to_string(),
            name: "Claude Haiku 3".to_string(),
            variants: vec![],
        },
    ];
    app.simulate_validation_complete_for_testing(true, None, Some(models.clone()));

    assert_eq!(
        app.mode,
        AppMode::ConnectModel,
        "Mode should be ConnectModel"
    );
    assert_eq!(
        app.pending_api_key_models, models,
        "Validated models should be stored in pending_api_key_models"
    );
    assert_eq!(
        app.pending_api_key_for_provider,
        Some("sk-ant-valid-key".to_string()),
        "API key should be preserved for provider setup"
    );
}

#[test]
fn test_model_selection_uses_api_key_auth_flow() {
    let mut app = App::new();
    app.complete_api_key_auth_for_test(
        "openai",
        "sk-api-key-12345",
        vec![BrowserAuthModelInfo {
            id: "gpt-4o".to_string(),
            name: "GPT-4o".to_string(),
            variants: vec![],
        }],
    );

    assert_eq!(app.mode, AppMode::ConnectModel);
    assert!(
        app.connect_model_dialog.is_some(),
        "Model dialog should be shown"
    );

    let result = app.confirm_model_for_api_key_auth_for_test("gpt-4o");
    assert!(
        result.is_ok(),
        "Model confirmation should succeed with API key auth"
    );

    assert_eq!(app.provider, "openai", "Provider should be set correctly");
    assert_eq!(
        app.mode,
        AppMode::Chat,
        "Should return to Chat mode after model selection"
    );
    assert!(
        app.connect_model_dialog.is_none(),
        "Dialog should be cleared after selection"
    );
    assert!(
        app.pending_api_key_for_provider.is_none(),
        "Pending API key should be cleared"
    );
}
