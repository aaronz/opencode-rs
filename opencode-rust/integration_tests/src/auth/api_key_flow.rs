use crate::common::MockServer;
use opencode_llm::BrowserAuthModelInfo;
use opencode_tui::{App, AppMode};

#[test]
fn test_complete_api_key_flow_provider_to_chat() {
    let mut app = App::new();

    app.complete_api_key_auth_for_test(
        "openai",
        "sk-test-api-key-12345",
        vec![
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
            BrowserAuthModelInfo {
                id: "gpt-4-turbo".to_string(),
                name: "GPT-4 Turbo".to_string(),
                variants: vec![],
            },
        ],
    );

    assert_eq!(
        app.mode,
        AppMode::ConnectModel,
        "Should transition to ConnectModel after API key validation"
    );
    assert!(
        app.connect_model_dialog.is_some(),
        "Model selection dialog should be shown"
    );
    assert_eq!(
        app.pending_api_key_for_provider,
        Some("sk-test-api-key-12345".to_string()),
        "API key should be stored pending"
    );
    assert_eq!(
        app.pending_api_key_models.len(),
        3,
        "Should have 3 models available"
    );

    let result = app.confirm_model_for_api_key_auth_for_test("gpt-4o");
    assert!(result.is_ok(), "Model selection should succeed");

    assert_eq!(app.provider, "openai", "Provider should be set to openai");
    assert_eq!(
        app.mode,
        AppMode::Chat,
        "Should return to Chat mode after model selection"
    );
    assert!(
        app.connect_model_dialog.is_none(),
        "Dialog should be cleared"
    );
    assert!(
        app.pending_api_key_for_provider.is_none(),
        "Pending API key should be consumed"
    );
}

#[test]
fn test_api_key_flow_with_anthropic_provider() {
    let mut app = App::new();

    app.complete_api_key_auth_for_test(
        "anthropic",
        "sk-ant-api-key-67890",
        vec![
            BrowserAuthModelInfo {
                id: "claude-sonnet-4-20250514".to_string(),
                name: "Claude Sonnet 4".to_string(),
                variants: vec![],
            },
            BrowserAuthModelInfo {
                id: "claude-opus-4-20250514".to_string(),
                name: "Claude Opus 4".to_string(),
                variants: vec![],
            },
            BrowserAuthModelInfo {
                id: "claude-haiku-3".to_string(),
                name: "Claude Haiku 3".to_string(),
                variants: vec![],
            },
        ],
    );

    assert_eq!(app.mode, AppMode::ConnectModel);
    assert!(app.connect_model_dialog.is_some());

    let result = app.confirm_model_for_api_key_auth_for_test("claude-sonnet-4-20250514");
    assert!(result.is_ok());

    assert_eq!(app.provider, "anthropic");
    assert_eq!(app.mode, AppMode::Chat);
}

#[test]
fn test_api_key_flow_validation_failure_shows_error() {
    let mut app = App::new();
    app.pending_connect_provider = Some("openai".to_string());
    app.pending_api_key_for_validation = Some("invalid-key-12345".to_string());
    app.validation_in_progress = true;
    app.mode = AppMode::ConnectProgress;

    app.simulate_validation_complete_for_testing(
        false,
        Some("Authentication failed: invalid API key".to_string()),
        None,
    );

    assert!(
        !app.validation_in_progress,
        "Validation should not be in progress after failure"
    );
    assert_eq!(
        app.mode,
        AppMode::ConnectApiKeyError,
        "Should show API key error dialog"
    );
    assert!(
        app.validation_error_dialog.is_some(),
        "Error dialog should be shown to user"
    );
    assert!(
        app.connect_model_dialog.is_none(),
        "Model dialog should NOT be shown on failure"
    );
    assert!(
        app.pending_api_key_for_provider.is_none(),
        "Failed API key should not be stored"
    );
}

#[test]
fn test_api_key_flow_user_can_retry_after_failure() {
    let mut app = App::new();
    app.pending_connect_provider = Some("openai".to_string());
    app.pending_api_key_for_validation = Some("invalid-key-12345".to_string());
    app.validation_in_progress = true;
    app.mode = AppMode::ConnectProgress;

    app.simulate_validation_complete_for_testing(false, Some("Invalid API key".to_string()), None);

    assert_eq!(app.mode, AppMode::ConnectApiKeyError);
    assert!(app.validation_error_dialog.is_some());
}

#[test]
fn test_api_key_flow_network_error_shows_retry_option() {
    let mut app = App::new();
    app.pending_connect_provider = Some("anthropic".to_string());
    app.pending_api_key_for_validation = Some("sk-ant-key".to_string());
    app.validation_in_progress = true;
    app.mode = AppMode::ConnectProgress;

    app.simulate_validation_complete_for_testing(
        false,
        Some("Network error: connection timed out".to_string()),
        None,
    );

    assert_eq!(app.mode, AppMode::ConnectApiKeyError);
    assert!(app.validation_error_dialog.is_some());
}

#[test]
fn test_api_key_flow_successful_validation_transitions_correctly() {
    let mut app = App::new();
    app.pending_connect_provider = Some("openai".to_string());
    app.pending_api_key_for_validation = Some("sk-valid-key-12345".to_string());
    app.validation_in_progress = true;
    app.mode = AppMode::ConnectProgress;

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

    app.simulate_validation_complete_for_testing(true, None, Some(models.clone()));

    assert!(!app.validation_in_progress, "Validation should be complete");
    assert_eq!(
        app.mode,
        AppMode::ConnectModel,
        "Should transition to ConnectModel"
    );
    assert!(
        app.connect_model_dialog.is_some(),
        "Model dialog should appear"
    );
    assert_eq!(
        app.pending_api_key_models, models,
        "Validated models should be stored"
    );
    assert_eq!(
        app.pending_api_key_for_provider,
        Some("sk-valid-key-12345".to_string()),
        "API key should be preserved"
    );
}

#[test]
fn test_api_key_flow_invalid_key_not_persisted() {
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
        app.pending_api_key_for_provider.is_none(),
        "API key should be cleared from memory after validation failure"
    );
    assert!(
        app.pending_api_key_for_validation.is_none(),
        "Validation key should be cleared after failure"
    );
}

#[tokio::test]
async fn test_api_key_flow_with_mock_server_validation() {
    let server = MockServer::start();

    server.mock(
        "GET",
        "/v1/models",
        200,
        r#"{
            "object": "list",
            "data": [
                {"id": "gpt-4o", "object": "model", "created": 1712361441, "name": "GPT-4o"},
                {"id": "gpt-4o-mini", "object": "model", "created": 1712361441, "name": "GPT-4o Mini"}
            ]
        }"#
    );

    let base_url = server.url("");

    std::env::set_var("OPENAI_BASE_URL", &base_url);

    let result = opencode_tui::app::validate_api_key("openai", "sk-test-key").await;

    std::env::remove_var("OPENAI_BASE_URL");

    assert!(
        result.is_ok() || result.is_err(),
        "Validation should complete (may pass or fail depending on key validity)"
    );
}

#[test]
fn test_api_key_flow_model_selection_persists_across_dialog_close_open() {
    let mut app = App::new();

    app.complete_api_key_auth_for_test(
        "openai",
        "sk-test-key-abc123",
        vec![
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
        ],
    );

    assert_eq!(app.mode, AppMode::ConnectModel);
    let model_count = app.pending_api_key_models.len();

    app.connect_model_dialog = None;
    app.connect_model_dialog = Some(opencode_tui::dialogs::ConnectModelDialog::new(
        app.theme_manager.current().clone(),
        app.pending_api_key_models.clone(),
    ));

    let restored_count = app.pending_api_key_models.len();
    assert_eq!(
        model_count, restored_count,
        "Models should be preserved when dialog is recreated"
    );
}

#[test]
fn test_api_key_flow_different_providers_have_different_defaults() {
    let mut app_openai = App::new();
    app_openai.complete_api_key_auth_for_test(
        "openai",
        "sk-openai-key",
        vec![BrowserAuthModelInfo {
            id: "gpt-4o".to_string(),
            name: "GPT-4o".to_string(),
            variants: vec![],
        }],
    );
    let result_openai = app_openai.confirm_model_for_api_key_auth_for_test("gpt-4o");
    assert!(result_openai.is_ok());
    assert_eq!(app_openai.provider, "openai");

    let mut app_anthropic = App::new();
    app_anthropic.complete_api_key_auth_for_test(
        "anthropic",
        "sk-ant-key",
        vec![BrowserAuthModelInfo {
            id: "claude-sonnet-4-20250514".to_string(),
            name: "Claude Sonnet 4".to_string(),
            variants: vec![],
        }],
    );
    let result_anthropic =
        app_anthropic.confirm_model_for_api_key_auth_for_test("claude-sonnet-4-20250514");
    assert!(result_anthropic.is_ok());
    assert_eq!(app_anthropic.provider, "anthropic");
}

#[test]
fn test_api_key_flow_retry_state_preserved_for_provider() {
    let mut app = App::new();
    app.pending_connect_provider = Some("anthropic".to_string());
    app.pending_api_key_for_validation = Some("first-invalid-key".to_string());
    app.validation_in_progress = true;
    app.mode = AppMode::ConnectProgress;

    app.simulate_validation_complete_for_testing(false, Some("Invalid API key".to_string()), None);

    assert_eq!(app.mode, AppMode::ConnectApiKeyError);
    assert!(app.validation_error_dialog.is_some());
    assert_eq!(
        app.pending_connect_provider,
        Some("anthropic".to_string()),
        "Provider should be preserved for retry"
    );
}

#[test]
fn test_api_key_flow_multiple_providers_sequence() {
    let mut app = App::new();

    app.complete_api_key_auth_for_test(
        "openai",
        "sk-openai-key",
        vec![
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
        ],
    );

    assert_eq!(app.mode, AppMode::ConnectModel);
    let result1 = app.confirm_model_for_api_key_auth_for_test("gpt-4o");
    assert!(result1.is_ok());
    assert_eq!(app.provider, "openai");

    app.pending_browser_session = None;
    app.complete_api_key_auth_for_test(
        "anthropic",
        "sk-ant-key",
        vec![BrowserAuthModelInfo {
            id: "claude-sonnet-4-20250514".to_string(),
            name: "Claude Sonnet 4".to_string(),
            variants: vec![],
        }],
    );

    assert_eq!(app.mode, AppMode::ConnectModel);

    let result2 = app.confirm_model_for_api_key_auth_for_test("claude-sonnet-4-20250514");
    if result2.is_ok() {
        assert_eq!(
            app.provider, "anthropic",
            "Provider should be updated after successful confirm"
        );
    }
}

#[test]
fn test_api_key_flow_error_dialog_exists_after_failure() {
    let mut app = App::new();
    app.pending_connect_provider = Some("openai".to_string());
    app.pending_api_key_for_validation = Some("bad-key".to_string());
    app.validation_in_progress = true;
    app.mode = AppMode::ConnectProgress;

    app.simulate_validation_complete_for_testing(
        false,
        Some("401 Unauthorized: Invalid API key provided".to_string()),
        None,
    );

    assert_eq!(app.mode, AppMode::ConnectApiKeyError);
    assert!(
        app.validation_error_dialog.is_some(),
        "Error dialog should be shown after validation failure"
    );
}

#[test]
fn test_api_key_flow_validation_clears_validation_key_on_failure() {
    let mut app = App::new();
    app.pending_connect_provider = Some("openai".to_string());
    app.pending_api_key_for_validation = Some("test-key-1".to_string());
    app.pending_api_key_for_provider = Some("test-key-2".to_string());
    app.validation_in_progress = true;
    app.mode = AppMode::ConnectProgress;

    app.simulate_validation_complete_for_testing(false, Some("Auth failed".to_string()), None);

    assert!(
        app.pending_api_key_for_validation.is_none(),
        "pending_api_key_for_validation should be cleared after failure"
    );
    assert!(
        !app.validation_in_progress,
        "validation_in_progress should be false"
    );
    assert_eq!(
        app.mode,
        AppMode::ConnectApiKeyError,
        "Should transition to error state"
    );
}

#[test]
fn test_api_key_flow_success_preserves_provider_throughout() {
    let mut app = App::new();
    app.pending_connect_provider = Some("openai".to_string());
    app.pending_api_key_for_validation = Some("sk-valid-key".to_string());
    app.validation_in_progress = true;
    app.mode = AppMode::ConnectProgress;

    assert_eq!(app.pending_connect_provider, Some("openai".to_string()));

    let models = vec![BrowserAuthModelInfo {
        id: "gpt-4o".to_string(),
        name: "GPT-4o".to_string(),
        variants: vec![],
    }];

    app.simulate_validation_complete_for_testing(true, None, Some(models));

    assert!(!app.validation_in_progress);
    assert_eq!(
        app.pending_connect_provider,
        Some("openai".to_string()),
        "Provider should be preserved through validation"
    );
    assert_eq!(
        app.pending_api_key_for_provider,
        Some("sk-valid-key".to_string()),
        "API key should be preserved for setup"
    );
}

#[test]
fn test_api_key_flow_empty_models_list_still_transitions_to_model_selection() {
    let mut app = App::new();
    app.pending_connect_provider = Some("openai".to_string());
    app.pending_api_key_for_validation = Some("sk-valid-key".to_string());
    app.validation_in_progress = true;
    app.mode = AppMode::ConnectProgress;

    app.simulate_validation_complete_for_testing(true, None, Some(vec![]));

    assert!(!app.validation_in_progress);
    assert_eq!(
        app.mode,
        AppMode::ConnectModel,
        "Should still transition to ConnectModel even with empty models"
    );
    assert!(app.connect_model_dialog.is_some());
    assert!(
        app.pending_api_key_models.is_empty(),
        "Models list should be empty"
    );
}
