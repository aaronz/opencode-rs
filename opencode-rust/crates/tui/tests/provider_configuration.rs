use opencode_tui::{App, AppMode};
use opencode_llm::BrowserAuthModelInfo;

#[test]
fn test_selected_model_is_stored_in_provider_config() {
    let mut app = App::new();

    app.complete_api_key_auth_for_test(
        "openai",
        "sk-api-key-12345",
        vec![
            BrowserAuthModelInfo { id: "gpt-4o".to_string(), name: "GPT-4o".to_string() },
            BrowserAuthModelInfo { id: "gpt-4o-mini".to_string(), name: "GPT-4o Mini".to_string() },
        ],
    );

    let result = app.confirm_model_for_api_key_auth_for_test("gpt-4o");
    assert!(result.is_ok(), "Model confirmation should succeed");

    let providers = &app.config.providers;
    assert!(providers.is_some(), "Providers should be set in config");

    let provider = providers.as_ref()
        .and_then(|p| p.iter().find(|p| p.name == "openai"));
    assert!(provider.is_some(), "OpenAI provider should exist in config");

    let provider = provider.unwrap();
    assert_eq!(
        provider.default_model.as_ref().unwrap(),
        "gpt-4o",
        "Selected model should be stored in provider config"
    );
}

#[test]
fn test_active_provider_is_set_after_model_selection() {
    let mut app = App::new();

    app.complete_api_key_auth_for_test(
        "anthropic",
        "sk-ant-api-key",
        vec![
            BrowserAuthModelInfo { id: "claude-sonnet-4-20250514".to_string(), name: "Claude Sonnet 4".to_string() },
            BrowserAuthModelInfo { id: "claude-haiku-3".to_string(), name: "Claude Haiku 3".to_string() },
        ],
    );

    let result = app.confirm_model_for_api_key_auth_for_test("claude-sonnet-4-20250514");
    assert!(result.is_ok(), "Model confirmation should succeed");

    assert_eq!(
        app.provider, "anthropic",
        "Active provider should be set after model selection"
    );
}

#[test]
fn test_app_transitions_to_chat_mode_after_selection() {
    let mut app = App::new();

    app.complete_api_key_auth_for_test(
        "openai",
        "sk-api-key-12345",
        vec![
            BrowserAuthModelInfo { id: "gpt-4o".to_string(), name: "GPT-4o".to_string() },
        ],
    );

    assert_eq!(app.mode, AppMode::ConnectModel);

    let result = app.confirm_model_for_api_key_auth_for_test("gpt-4o");
    assert!(result.is_ok(), "Model confirmation should succeed");

    assert_eq!(
        app.mode, AppMode::Chat,
        "App should transition to Chat mode after model selection"
    );
}

#[test]
fn test_complete_flow_validation_model_select_chat() {
    let mut app = App::new();

    app.validation_in_progress = true;
    app.mode = AppMode::ConnectProgress;
    app.pending_connect_provider = Some("openai".to_string());
    app.pending_api_key_for_validation = Some("sk-api-key-12345".to_string());

    let models = vec![
        BrowserAuthModelInfo { id: "gpt-4o".to_string(), name: "GPT-4o".to_string() },
        BrowserAuthModelInfo { id: "gpt-4o-mini".to_string(), name: "GPT-4o Mini".to_string() },
    ];
    app.simulate_validation_complete_for_testing(true, None, Some(models.clone()));

    assert_eq!(
        app.mode, AppMode::ConnectModel,
        "After validation: mode should be ConnectModel"
    );
    assert!(
        app.connect_model_dialog.is_some(),
        "After validation: ConnectModelDialog should be shown"
    );
    assert_eq!(
        app.pending_api_key_models, models,
        "After validation: models should be stored"
    );

    let result = app.confirm_model_for_api_key_auth_for_test("gpt-4o");
    assert!(result.is_ok(), "Model confirmation should succeed");

    assert_eq!(
        app.provider, "openai",
        "After model selection: provider should be set"
    );
    assert_eq!(
        app.mode, AppMode::Chat,
        "After model selection: mode should be Chat"
    );
    assert!(
        app.connect_model_dialog.is_none(),
        "After model selection: dialog should be closed"
    );
    let providers = &app.config.providers;
    assert!(providers.is_some(), "After model selection: providers should be set");
    let provider = providers.as_ref()
        .and_then(|p| p.iter().find(|p| p.name == "openai"));
    assert!(provider.is_some(), "After model selection: openai provider should exist");
    assert_eq!(
        provider.unwrap().default_model.as_ref().unwrap(),
        "gpt-4o",
        "After model selection: model should be stored in provider config"
    );
}