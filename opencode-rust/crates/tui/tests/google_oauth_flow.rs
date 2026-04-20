use opencode_tui::{App, AppMode};
use opencode_llm::{BrowserAuthModelInfo, GoogleOAuthSession};

#[test]
fn test_google_oauth_completes_auth_and_shows_model_picker() {
    let mut app = App::new();

    app.complete_google_auth_for_test(
        GoogleOAuthSession {
            access_token: "test_access_token".to_string(),
            refresh_token: Some("test_refresh_token".to_string()),
            expires_at_epoch_ms: chrono::Utc::now().timestamp_millis() + 3600000,
            email: Some("test@gmail.com".to_string()),
        },
        vec![
            BrowserAuthModelInfo {
                id: "gemini-1.5-pro".to_string(),
                name: "Gemini 1.5 Pro".to_string(),
                variants: vec![],
            },
            BrowserAuthModelInfo {
                id: "gemini-1.5-flash".to_string(),
                name: "Gemini 1.5 Flash".to_string(),
                variants: vec![],
            },
        ],
    );

    assert_eq!(app.mode, AppMode::ConnectModel);
    assert!(app.connect_model_dialog.is_some());
    assert!(app.pending_google_session.is_some());
}

#[test]
fn test_google_auth_flow_sets_google_provider_on_model_confirm() {
    let mut app = App::new();
    app.prime_google_connect_state_for_test();

    let result = app.confirm_model_for_google_auth_for_test("gemini-1.5-pro");
    assert!(result.is_ok());
    assert_eq!(app.provider, "google");
}

#[test]
fn test_google_oauth_stores_session_in_credential_store() {
    let mut app = App::new();
    app.prime_google_connect_state_for_test();

    assert!(app.pending_google_session.is_some());
    let session = app.pending_google_session.clone().unwrap();
    assert_eq!(session.access_token, "test_access_token");
    assert_eq!(session.email, Some("test@gmail.com".to_string()));
}

#[test]
fn test_google_oauth_provides_model_list() {
    let mut app = App::new();

    let models = vec![
        BrowserAuthModelInfo {
            id: "gemini-2.0-flash".to_string(),
            name: "Gemini 2.0 Flash".to_string(),
            variants: vec![],
        },
        BrowserAuthModelInfo {
            id: "gemini-1.5-pro".to_string(),
            name: "Gemini 1.5 Pro".to_string(),
            variants: vec![],
        },
        BrowserAuthModelInfo {
            id: "gemini-1.5-flash".to_string(),
            name: "Gemini 1.5 Flash".to_string(),
            variants: vec![],
        },
    ];

    app.complete_google_auth_for_test(
        GoogleOAuthSession {
            access_token: "test_access_token".to_string(),
            refresh_token: Some("test_refresh_token".to_string()),
            expires_at_epoch_ms: chrono::Utc::now().timestamp_millis() + 3600000,
            email: Some("test@gmail.com".to_string()),
        },
        models.clone(),
    );

    assert!(app.connect_model_dialog.is_some());
    assert_eq!(app.pending_browser_models.len(), 3);
}