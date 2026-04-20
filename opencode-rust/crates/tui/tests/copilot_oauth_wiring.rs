use opencode_tui::{App, AppMode};
use opencode_llm::{BrowserAuthModelInfo, CopilotOAuthSession};

#[test]
fn test_copilot_oauth_completes_auth_and_shows_model_picker() {
    let mut app = App::new();

    app.complete_copilot_auth_for_test(
        CopilotOAuthSession {
            access_token: "test_access_token".to_string(),
            expires_at_epoch_ms: chrono::Utc::now().timestamp_millis() + 3600000,
            token_type: "Bearer".to_string(),
        },
        vec![
            BrowserAuthModelInfo {
                id: "gpt-4o".to_string(),
                name: "GPT-4o".to_string(),
                variants: vec![],
            },
            BrowserAuthModelInfo {
                id: "o1".to_string(),
                name: "o1".to_string(),
                variants: vec![],
            },
        ],
    );

    assert_eq!(app.mode, AppMode::ConnectModel);
    assert!(app.connect_model_dialog.is_some());
    assert!(app.pending_copilot_session.is_some());
}

#[test]
fn test_copilot_auth_flow_sets_copilot_provider_on_model_confirm() {
    let mut app = App::new();
    app.prime_copilot_connect_state_for_test();

    let result = app.confirm_model_for_copilot_auth_for_test("gpt-4o");
    assert!(result.is_ok());
    assert_eq!(app.provider, "copilot");
}

#[test]
fn test_copilot_oauth_stores_session_in_credential_store() {
    let mut app = App::new();
    app.prime_copilot_connect_state_for_test();

    assert!(app.pending_copilot_session.is_some());
    let session = app.pending_copilot_session.clone().unwrap();
    assert_eq!(session.access_token, "test_access_token");
}

#[test]
fn test_copilot_oauth_provides_model_list() {
    let mut app = App::new();

    let models = vec![
        BrowserAuthModelInfo {
            id: "gpt-4o".to_string(),
            name: "GPT-4o".to_string(),
            variants: vec![],
        },
        BrowserAuthModelInfo {
            id: "o1".to_string(),
            name: "o1".to_string(),
            variants: vec![],
        },
        BrowserAuthModelInfo {
            id: "o1-mini".to_string(),
            name: "o1 Mini".to_string(),
            variants: vec![],
        },
    ];

    app.complete_copilot_auth_for_test(
        CopilotOAuthSession {
            access_token: "test_access_token".to_string(),
            expires_at_epoch_ms: chrono::Utc::now().timestamp_millis() + 3600000,
            token_type: "Bearer".to_string(),
        },
        models.clone(),
    );

    assert!(app.connect_model_dialog.is_some());
    assert_eq!(app.pending_browser_models.len(), 3);
}

#[test]
fn test_copilot_oauth_flow_model_to_chat() {
    let mut app = App::new();

    app.complete_copilot_auth_for_test(
        CopilotOAuthSession {
            access_token: "test_access_token".to_string(),
            expires_at_epoch_ms: chrono::Utc::now().timestamp_millis() + 3600000,
            token_type: "Bearer".to_string(),
        },
        vec![
            BrowserAuthModelInfo {
                id: "gpt-4o".to_string(),
                name: "GPT-4o".to_string(),
                variants: vec![],
            },
            BrowserAuthModelInfo {
                id: "o1".to_string(),
                name: "o1".to_string(),
                variants: vec![],
            },
        ],
    );

    assert_eq!(app.mode, AppMode::ConnectModel);
    assert!(app.connect_model_dialog.is_some());
    assert!(app.pending_copilot_session.is_some());

    let result = app.confirm_model_for_copilot_auth_for_test("gpt-4o");
    assert!(result.is_ok());

    assert_eq!(app.provider, "copilot");
    assert_eq!(app.mode, AppMode::Chat);
    assert!(app.connect_model_dialog.is_none());
    assert!(app.pending_copilot_session.is_none());
}