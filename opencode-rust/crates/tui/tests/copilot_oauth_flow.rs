use opencode_tui::{App, AppMode};
use opencode_llm::{BrowserAuthModelInfo, CopilotOAuthSession};

#[test]
fn test_copilot_oauth_success_shows_connect_model_dialog() {
    let mut app = App::new();

    app.complete_copilot_auth_for_test(
        CopilotOAuthSession {
            access_token: "test_copilot_token_abc123".to_string(),
            expires_at_epoch_ms: chrono::Utc::now().timestamp_millis() + 3600000,
            token_type: "Bearer".to_string(),
        },
        vec![
            BrowserAuthModelInfo {
                id: "gpt-4o".to_string(),
                name: "GPT-4o".to_string(),
            },
            BrowserAuthModelInfo {
                id: "o1".to_string(),
                name: "o1".to_string(),
            },
            BrowserAuthModelInfo {
                id: "o1-mini".to_string(),
                name: "o1 Mini".to_string(),
            },
        ],
    );

    assert_eq!(app.mode, AppMode::ConnectModel);
    assert!(app.connect_model_dialog.is_some(), "ConnectModelDialog should be shown after Copilot OAuth success");
    assert!(app.pending_copilot_session.is_some(), "Copilot session should be stored");
}

#[test]
fn test_copilot_oauth_passes_credentials_to_model_dialog() {
    let mut app = App::new();

    let expected_token = "copilot_test_token_xyz789".to_string();
    let models = vec![
        BrowserAuthModelInfo {
            id: "gpt-4o".to_string(),
            name: "GPT-4o".to_string(),
        },
        BrowserAuthModelInfo {
            id: "o1-preview".to_string(),
            name: "o1 Preview".to_string(),
        },
    ];

    app.complete_copilot_auth_for_test(
        CopilotOAuthSession {
            access_token: expected_token.clone(),
            expires_at_epoch_ms: chrono::Utc::now().timestamp_millis() + 3600000,
            token_type: "Bearer".to_string(),
        },
        models.clone(),
    );

    assert!(app.pending_copilot_session.is_some(), "Session should be stored");
    let stored_session = app.pending_copilot_session.clone().unwrap();
    assert_eq!(stored_session.access_token, expected_token, "Access token should be passed to model dialog state");
    assert_eq!(app.pending_browser_models.len(), 2, "Model list should be passed to dialog state");
    assert_eq!(app.pending_browser_models[0].id, "gpt-4o");
    assert_eq!(app.pending_browser_models[1].id, "o1-preview");
}

#[test]
fn test_copilot_oauth_model_selection_transitions_to_chat() {
    let mut app = App::new();

    app.complete_copilot_auth_for_test(
        CopilotOAuthSession {
            access_token: "test_token".to_string(),
            expires_at_epoch_ms: chrono::Utc::now().timestamp_millis() + 3600000,
            token_type: "Bearer".to_string(),
        },
        vec![
            BrowserAuthModelInfo {
                id: "gpt-4o".to_string(),
                name: "GPT-4o".to_string(),
            },
        ],
    );

    assert_eq!(app.mode, AppMode::ConnectModel);

    let result = app.confirm_model_for_copilot_auth_for_test("gpt-4o");
    assert!(result.is_ok());

    assert_eq!(app.provider, "copilot", "Provider should be set to copilot");
    assert_eq!(app.mode, AppMode::Chat, "Should transition to Chat after model selection");
    assert!(app.connect_model_dialog.is_none(), "Model dialog should be closed");
    assert!(app.pending_copilot_session.is_none(), "Pending session should be cleared");
}

#[test]
fn test_copilot_oauth_sets_copilot_provider_after_model_selection() {
    let mut app = App::new();

    app.prime_copilot_connect_state_for_test();

    let result = app.confirm_model_for_copilot_auth_for_test("gpt-4o");
    assert!(result.is_ok());

    assert_eq!(app.provider, "copilot", "Provider should be copilot after OAuth flow");
    assert!(app.llm_provider.is_some(), "LLM provider should be set");
}

#[test]
fn test_copilot_oauth_session_persists_across_model_selection() {
    let mut app = App::new();

    app.complete_copilot_auth_for_test(
        CopilotOAuthSession {
            access_token: "persistent_token_abc".to_string(),
            expires_at_epoch_ms: chrono::Utc::now().timestamp_millis() + 3600000,
            token_type: "Bearer".to_string(),
        },
        vec![BrowserAuthModelInfo {
            id: "o1-mini".to_string(),
            name: "o1 Mini".to_string(),
        }],
    );

    assert!(app.pending_copilot_session.is_some());
    let session_before = app.pending_copilot_session.clone();

    let result = app.confirm_model_for_copilot_auth_for_test("o1-mini");
    assert!(result.is_ok());

    assert!(app.pending_copilot_session.is_none(), "Session should be consumed after model selection");
}