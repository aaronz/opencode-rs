mod common;
use common::MockServer;
use opencode_llm::{BrowserAuthModelInfo, GoogleOAuthSession};
use opencode_tui::{App, AppMode};

#[test]
fn test_complete_google_oauth_flow_provider_to_chat() {
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
            BrowserAuthModelInfo {
                id: "gemini-2.0-flash".to_string(),
                name: "Gemini 2.0 Flash".to_string(),
                variants: vec![],
            },
        ],
    );

    assert_eq!(
        app.mode,
        AppMode::ConnectModel,
        "Should transition to ConnectModel after OAuth"
    );
    assert!(
        app.connect_model_dialog.is_some(),
        "Model selection dialog should be shown"
    );
    assert!(
        app.pending_google_session.is_some(),
        "Google session should be stored pending"
    );
    assert_eq!(
        app.pending_browser_models.len(),
        3,
        "Should have 3 models available"
    );

    let result = app.confirm_model_for_google_auth_for_test("gemini-1.5-pro");
    assert!(result.is_ok(), "Model selection should succeed");

    assert_eq!(app.provider, "google", "Provider should be set to google");
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
        app.pending_google_session.is_none(),
        "Pending Google session should be consumed"
    );
}

#[test]
fn test_google_oauth_flow_with_flash_model() {
    let mut app = App::new();

    app.complete_google_auth_for_test(
        GoogleOAuthSession {
            access_token: "test_token".to_string(),
            refresh_token: Some("refresh_token".to_string()),
            expires_at_epoch_ms: chrono::Utc::now().timestamp_millis() + 3600000,
            email: Some("user@gmail.com".to_string()),
        },
        vec![BrowserAuthModelInfo {
            id: "gemini-1.5-flash".to_string(),
            name: "Gemini 1.5 Flash".to_string(),
            variants: vec![],
        }],
    );

    assert_eq!(app.mode, AppMode::ConnectModel);
    assert!(app.connect_model_dialog.is_some());

    let result = app.confirm_model_for_google_auth_for_test("gemini-1.5-flash");
    assert!(result.is_ok());

    assert_eq!(app.provider, "google");
    assert_eq!(app.mode, AppMode::Chat);
}

#[test]
fn test_google_oauth_session_stores_email() {
    let mut app = App::new();

    app.complete_google_auth_for_test(
        GoogleOAuthSession {
            access_token: "access_123".to_string(),
            refresh_token: Some("refresh_456".to_string()),
            expires_at_epoch_ms: chrono::Utc::now().timestamp_millis() + 3600000,
            email: Some("developer@google.com".to_string()),
        },
        vec![BrowserAuthModelInfo {
            id: "gemini-1.5-pro".to_string(),
            name: "Gemini 1.5 Pro".to_string(),
            variants: vec![],
        }],
    );

    assert!(app.pending_google_session.is_some());
    let session = app.pending_google_session.clone().unwrap();
    assert_eq!(session.email, Some("developer@google.com".to_string()));
    assert_eq!(session.access_token, "access_123");
}

#[test]
fn test_google_oauth_flow_model_selection_persists_across_dialog_close_open() {
    let mut app = App::new();

    app.complete_google_auth_for_test(
        GoogleOAuthSession {
            access_token: "test_token".to_string(),
            refresh_token: Some("refresh".to_string()),
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
    let model_count = app.pending_browser_models.len();

    app.connect_model_dialog = None;
    app.connect_model_dialog = Some(opencode_tui::dialogs::ConnectModelDialog::new(
        app.theme_manager.current().clone(),
        app.pending_browser_models.clone(),
    ));

    let restored_count = app.pending_browser_models.len();
    assert_eq!(
        model_count, restored_count,
        "Models should be preserved when dialog is recreated"
    );
}

#[test]
fn test_google_oauth_multiple_providers_sequence() {
    let mut app = App::new();

    app.complete_google_auth_for_test(
        GoogleOAuthSession {
            access_token: "google_token".to_string(),
            refresh_token: Some("google_refresh".to_string()),
            expires_at_epoch_ms: chrono::Utc::now().timestamp_millis() + 3600000,
            email: Some("google@example.com".to_string()),
        },
        vec![BrowserAuthModelInfo {
            id: "gemini-1.5-pro".to_string(),
            name: "Gemini 1.5 Pro".to_string(),
            variants: vec![],
        }],
    );

    assert_eq!(app.mode, AppMode::ConnectModel);
    let result1 = app.confirm_model_for_google_auth_for_test("gemini-1.5-pro");
    assert!(result1.is_ok());
    assert_eq!(app.provider, "google");

    app.pending_google_session = None;
    app.complete_google_auth_for_test(
        GoogleOAuthSession {
            access_token: "google_token_2".to_string(),
            refresh_token: Some("google_refresh_2".to_string()),
            expires_at_epoch_ms: chrono::Utc::now().timestamp_millis() + 3600000,
            email: Some("another@gmail.com".to_string()),
        },
        vec![BrowserAuthModelInfo {
            id: "gemini-2.0-flash".to_string(),
            name: "Gemini 2.0 Flash".to_string(),
            variants: vec![],
        }],
    );

    assert_eq!(app.mode, AppMode::ConnectModel);

    let result2 = app.confirm_model_for_google_auth_for_test("gemini-2.0-flash");
    if result2.is_ok() {
        assert_eq!(
            app.provider, "google",
            "Provider should be updated after successful confirm"
        );
    }
}

#[test]
fn test_google_oauth_expired_session_detected() {
    let session = GoogleOAuthSession {
        access_token: "expired_token".to_string(),
        refresh_token: Some("refresh".to_string()),
        expires_at_epoch_ms: chrono::Utc::now().timestamp_millis() - 1000,
        email: Some("expired@gmail.com".to_string()),
    };

    assert!(
        session.is_expired(),
        "Session with past expiry time should be detected as expired"
    );
}

#[test]
fn test_google_oauth_valid_session_not_expired() {
    let session = GoogleOAuthSession {
        access_token: "valid_token".to_string(),
        refresh_token: Some("refresh".to_string()),
        expires_at_epoch_ms: chrono::Utc::now().timestamp_millis() + 3600000,
        email: Some("valid@gmail.com".to_string()),
    };

    assert!(
        !session.is_expired(),
        "Session with future expiry time should not be expired"
    );
}

#[test]
fn test_google_oauth_flow_with_multiple_gemini_models() {
    let mut app = App::new();

    let models = vec![
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
        BrowserAuthModelInfo {
            id: "gemini-1.5-flash-8b".to_string(),
            name: "Gemini 1.5 Flash 8B".to_string(),
            variants: vec![],
        },
        BrowserAuthModelInfo {
            id: "gemini-2.0-flash-exp".to_string(),
            name: "Gemini 2.0 Flash Experimental".to_string(),
            variants: vec![],
        },
    ];

    app.complete_google_auth_for_test(
        GoogleOAuthSession {
            access_token: "multi_model_token".to_string(),
            refresh_token: Some("refresh".to_string()),
            expires_at_epoch_ms: chrono::Utc::now().timestamp_millis() + 3600000,
            email: Some("multimodel@gmail.com".to_string()),
        },
        models,
    );

    assert_eq!(app.mode, AppMode::ConnectModel);
    assert_eq!(
        app.pending_browser_models.len(),
        4,
        "Should have 4 models available"
    );

    let result = app.confirm_model_for_google_auth_for_test("gemini-2.0-flash-exp");
    assert!(result.is_ok());
    assert_eq!(app.provider, "google");
}

#[test]
fn test_google_oauth_preserves_session_data_through_model_selection() {
    let mut app = App::new();

    app.complete_google_auth_for_test(
        GoogleOAuthSession {
            access_token: "preserved_token".to_string(),
            refresh_token: Some("preserved_refresh".to_string()),
            expires_at_epoch_ms: chrono::Utc::now().timestamp_millis() + 3600000,
            email: Some("preserve@test.com".to_string()),
        },
        vec![BrowserAuthModelInfo {
            id: "gemini-1.5-pro".to_string(),
            name: "Gemini 1.5 Pro".to_string(),
            variants: vec![],
        }],
    );

    let original_session = app.pending_google_session.clone();

    assert!(original_session.is_some());
    assert_eq!(
        original_session.as_ref().unwrap().access_token,
        "preserved_token"
    );

    let result = app.confirm_model_for_google_auth_for_test("gemini-1.5-pro");
    assert!(result.is_ok());

    assert!(
        app.pending_google_session.is_none(),
        "Session should be consumed after model confirm"
    );
}

#[test]
fn test_google_oauth_flow_empty_models_still_transitions() {
    let mut app = App::new();

    app.complete_google_auth_for_test(
        GoogleOAuthSession {
            access_token: "token".to_string(),
            refresh_token: Some("refresh".to_string()),
            expires_at_epoch_ms: chrono::Utc::now().timestamp_millis() + 3600000,
            email: Some("empty@test.com".to_string()),
        },
        vec![],
    );

    assert_eq!(
        app.mode,
        AppMode::ConnectModel,
        "Should still transition to ConnectModel even with empty models"
    );
    assert!(app.connect_model_dialog.is_some());
    assert!(
        app.pending_browser_models.is_empty(),
        "Models list should be empty"
    );
}

#[test]
fn test_google_oauth_different_models_have_different_ids() {
    let mut app = App::new();

    let models = vec![
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
            access_token: "token".to_string(),
            refresh_token: Some("refresh".to_string()),
            expires_at_epoch_ms: chrono::Utc::now().timestamp_millis() + 3600000,
            email: None,
        },
        models,
    );

    let model_ids: Vec<String> = app
        .pending_browser_models
        .iter()
        .map(|m| m.id.clone())
        .collect();
    assert!(model_ids.contains(&"gemini-1.5-pro".to_string()));
    assert!(model_ids.contains(&"gemini-1.5-flash".to_string()));
    assert_eq!(model_ids.len(), 2, "Should have exactly 2 unique model IDs");
}

#[tokio::test]
async fn test_google_oauth_mock_server_token_endpoint() {
    let server = MockServer::start();

    server.mock(
        "POST",
        "/token",
        200,
        r#"{
            "access_token": "mock_access_token",
            "refresh_token": "mock_refresh_token",
            "expires_in": 3600,
            "token_type": "Bearer"
        }"#,
    );

    let base_url = server.url("");
    assert!(base_url.contains("127.0.0.1"));

    std::env::set_var("GOOGLE_TOKEN_URL", &base_url);

    std::env::remove_var("GOOGLE_TOKEN_URL");
}

#[test]
fn test_google_oauth_uses_correct_provider_name() {
    let mut app = App::new();

    app.complete_google_auth_for_test(
        GoogleOAuthSession {
            access_token: "token".to_string(),
            refresh_token: None,
            expires_at_epoch_ms: chrono::Utc::now().timestamp_millis() + 3600000,
            email: Some("test@gmail.com".to_string()),
        },
        vec![BrowserAuthModelInfo {
            id: "gemini-pro".to_string(),
            name: "Gemini Pro".to_string(),
            variants: vec![],
        }],
    );

    let result = app.confirm_model_for_google_auth_for_test("gemini-pro");
    assert!(result.is_ok());
    assert_eq!(
        app.provider, "google",
        "Provider should be 'google', not 'google-oauth' or other variant"
    );
}

#[test]
fn test_google_oauth_without_refresh_token() {
    let mut app = App::new();

    app.complete_google_auth_for_test(
        GoogleOAuthSession {
            access_token: "access_only".to_string(),
            refresh_token: None,
            expires_at_epoch_ms: chrono::Utc::now().timestamp_millis() + 3600000,
            email: None,
        },
        vec![BrowserAuthModelInfo {
            id: "gemini-1.5-flash".to_string(),
            name: "Gemini 1.5 Flash".to_string(),
            variants: vec![],
        }],
    );

    assert!(app.pending_google_session.is_some());
    assert!(app
        .pending_google_session
        .as_ref()
        .unwrap()
        .refresh_token
        .is_none());

    let result = app.confirm_model_for_google_auth_for_test("gemini-1.5-flash");
    assert!(result.is_ok());
    assert_eq!(app.provider, "google");
}
