use opencode_llm::BrowserAuthModelInfo;
use opencode_tui::{App, AppMode};

#[test]
fn test_complete_api_key_to_model_selection_flow() {
    let mut app = App::new();

    app.complete_api_key_auth_for_test(
        "openai",
        "sk-api-key-12345",
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

    assert_eq!(
        app.mode,
        AppMode::ConnectModel,
        "Mode should be ConnectModel after successful validation"
    );
    assert!(
        app.connect_model_dialog.is_some(),
        "ConnectModelDialog should be shown after successful validation"
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

#[test]
fn test_api_key_validation_transitions_to_connect_model() {
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
        "Mode should be ConnectModel after successful validation"
    );
    assert!(
        app.connect_model_dialog.is_some(),
        "ConnectModelDialog should be shown after successful validation"
    );
    assert_eq!(
        app.pending_api_key_models, models,
        "Validated models should be stored"
    );
    assert_eq!(
        app.pending_api_key_for_provider,
        Some("sk-ant-valid-key".to_string()),
        "API key should be preserved"
    );
}

#[test]
fn test_invalid_api_key_shows_error_not_model_dialog() {
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
    assert!(
        app.connect_model_dialog.is_none(),
        "ConnectModelDialog should NOT be shown after failed validation"
    );
}
