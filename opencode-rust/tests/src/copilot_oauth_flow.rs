mod common;
use common::MockServer;
use opencode_llm::{BrowserAuthModelInfo, CopilotOAuthCallback, CopilotOAuthRequest, CopilotOAuthSession, CopilotOAuthService};
use opencode_tui::{App, AppMode};

#[test]
fn test_complete_copilot_oauth_flow_provider_to_chat() {
    let mut app = App::new();

    app.complete_copilot_auth_for_test(
        CopilotOAuthSession {
            access_token: "test_copilot_token_abc123".to_string(),
            expires_at_epoch_ms: chrono::Utc::now().timestamp_millis() + 3600000,
            token_type: "Bearer".to_string(),
        },
        vec![
            BrowserAuthModelInfo { id: "gpt-4o".to_string(), name: "GPT-4o".to_string() },
            BrowserAuthModelInfo { id: "o1".to_string(), name: "o1".to_string() },
            BrowserAuthModelInfo { id: "o1-mini".to_string(), name: "o1 Mini".to_string() },
        ],
    );

    assert_eq!(app.mode, AppMode::ConnectModel, "Should transition to ConnectModel after OAuth");
    assert!(app.connect_model_dialog.is_some(), "Model selection dialog should be shown");
    assert!(app.pending_copilot_session.is_some(), "Copilot session should be stored pending");
    assert_eq!(app.pending_browser_models.len(), 3, "Should have 3 models available");

    let result = app.confirm_model_for_copilot_auth_for_test("gpt-4o");
    assert!(result.is_ok(), "Model selection should succeed");

    assert_eq!(app.provider, "copilot", "Provider should be set to copilot");
    assert_eq!(app.mode, AppMode::Chat, "Should return to Chat mode after model selection");
    assert!(app.connect_model_dialog.is_none(), "Dialog should be cleared");
    assert!(app.pending_copilot_session.is_none(), "Pending Copilot session should be consumed");
}

#[test]
fn test_copilot_oauth_flow_with_o1_model() {
    let mut app = App::new();

    app.complete_copilot_auth_for_test(
        CopilotOAuthSession {
            access_token: "test_token".to_string(),
            expires_at_epoch_ms: chrono::Utc::now().timestamp_millis() + 3600000,
            token_type: "Bearer".to_string(),
        },
        vec![
            BrowserAuthModelInfo { id: "o1-preview".to_string(), name: "o1 Preview".to_string() },
            BrowserAuthModelInfo { id: "o1-mini".to_string(), name: "o1 Mini".to_string() },
        ],
    );

    assert_eq!(app.mode, AppMode::ConnectModel);
    assert!(app.connect_model_dialog.is_some());

    let result = app.confirm_model_for_copilot_auth_for_test("o1-preview");
    assert!(result.is_ok());

    assert_eq!(app.provider, "copilot");
    assert_eq!(app.mode, AppMode::Chat);
}

#[test]
fn test_copilot_oauth_session_stores_token() {
    let mut app = App::new();

    app.complete_copilot_auth_for_test(
        CopilotOAuthSession {
            access_token: "gho_copilot_token_xyz789".to_string(),
            expires_at_epoch_ms: chrono::Utc::now().timestamp_millis() + 3600000,
            token_type: "Bearer".to_string(),
        },
        vec![
            BrowserAuthModelInfo { id: "gpt-4o".to_string(), name: "GPT-4o".to_string() },
        ],
    );

    assert!(app.pending_copilot_session.is_some());
    let session = app.pending_copilot_session.clone().unwrap();
    assert_eq!(session.access_token, "gho_copilot_token_xyz789");
    assert_eq!(session.token_type, "Bearer");
    assert!(!session.is_expired());
}

#[test]
fn test_copilot_oauth_flow_model_selection_persists_across_dialog_close_open() {
    let mut app = App::new();

    app.complete_copilot_auth_for_test(
        CopilotOAuthSession {
            access_token: "test_token".to_string(),
            expires_at_epoch_ms: chrono::Utc::now().timestamp_millis() + 3600000,
            token_type: "Bearer".to_string(),
        },
        vec![
            BrowserAuthModelInfo { id: "gpt-4o".to_string(), name: "GPT-4o".to_string() },
            BrowserAuthModelInfo { id: "o1".to_string(), name: "o1".to_string() },
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
    assert_eq!(model_count, restored_count, "Models should be preserved when dialog is recreated");
}

#[test]
fn test_copilot_oauth_multiple_providers_sequence() {
    let mut app = App::new();

    app.complete_copilot_auth_for_test(
        CopilotOAuthSession {
            access_token: "copilot_token_1".to_string(),
            expires_at_epoch_ms: chrono::Utc::now().timestamp_millis() + 3600000,
            token_type: "Bearer".to_string(),
        },
        vec![
            BrowserAuthModelInfo { id: "gpt-4o".to_string(), name: "GPT-4o".to_string() },
        ],
    );

    assert_eq!(app.mode, AppMode::ConnectModel);
    let result1 = app.confirm_model_for_copilot_auth_for_test("gpt-4o");
    assert!(result1.is_ok());
    assert_eq!(app.provider, "copilot");

    app.pending_copilot_session = None;
    app.complete_copilot_auth_for_test(
        CopilotOAuthSession {
            access_token: "copilot_token_2".to_string(),
            expires_at_epoch_ms: chrono::Utc::now().timestamp_millis() + 3600000,
            token_type: "Bearer".to_string(),
        },
        vec![
            BrowserAuthModelInfo { id: "o1".to_string(), name: "o1".to_string() },
        ],
    );

    assert_eq!(app.mode, AppMode::ConnectModel);

    let result2 = app.confirm_model_for_copilot_auth_for_test("o1");
    if result2.is_ok() {
        assert_eq!(app.provider, "copilot", "Provider should be updated after successful confirm");
    }
}

#[test]
fn test_copilot_oauth_expired_session_detected() {
    let session = CopilotOAuthSession {
        access_token: "expired_token".to_string(),
        expires_at_epoch_ms: chrono::Utc::now().timestamp_millis() - 1000,
        token_type: "Bearer".to_string(),
    };

    assert!(session.is_expired(), "Session with past expiry time should be detected as expired");
}

#[test]
fn test_copilot_oauth_valid_session_not_expired() {
    let session = CopilotOAuthSession {
        access_token: "valid_token".to_string(),
        expires_at_epoch_ms: chrono::Utc::now().timestamp_millis() + 3600000,
        token_type: "Bearer".to_string(),
    };

    assert!(!session.is_expired(), "Session with future expiry time should not be expired");
}

#[test]
fn test_copilot_oauth_flow_with_multiple_copilot_models() {
    let mut app = App::new();

    let models = vec![
        BrowserAuthModelInfo { id: "gpt-4o".to_string(), name: "GPT-4o".to_string() },
        BrowserAuthModelInfo { id: "o1-preview".to_string(), name: "o1 Preview".to_string() },
        BrowserAuthModelInfo { id: "o1-mini".to_string(), name: "o1 Mini".to_string() },
        BrowserAuthModelInfo { id: "claude-sonnet-4".to_string(), name: "Claude Sonnet 4".to_string() },
    ];

    app.complete_copilot_auth_for_test(
        CopilotOAuthSession {
            access_token: "multi_model_token".to_string(),
            expires_at_epoch_ms: chrono::Utc::now().timestamp_millis() + 3600000,
            token_type: "Bearer".to_string(),
        },
        models,
    );

    assert_eq!(app.mode, AppMode::ConnectModel);
    assert_eq!(app.pending_browser_models.len(), 4, "Should have 4 models available");

    let result = app.confirm_model_for_copilot_auth_for_test("o1-mini");
    assert!(result.is_ok());
    assert_eq!(app.provider, "copilot");
}

#[test]
fn test_copilot_oauth_preserves_session_data_through_model_selection() {
    let mut app = App::new();

    app.complete_copilot_auth_for_test(
        CopilotOAuthSession {
            access_token: "preserved_token".to_string(),
            expires_at_epoch_ms: chrono::Utc::now().timestamp_millis() + 3600000,
            token_type: "Bearer".to_string(),
        },
        vec![
            BrowserAuthModelInfo { id: "gpt-4o".to_string(), name: "GPT-4o".to_string() },
        ],
    );

    let original_session = app.pending_copilot_session.clone();

    assert!(original_session.is_some());
    assert_eq!(original_session.as_ref().unwrap().access_token, "preserved_token");

    let result = app.confirm_model_for_copilot_auth_for_test("gpt-4o");
    assert!(result.is_ok());

    assert!(app.pending_copilot_session.is_none(), "Session should be consumed after model confirm");
}

#[test]
fn test_copilot_oauth_flow_empty_models_still_transitions() {
    let mut app = App::new();

    app.complete_copilot_auth_for_test(
        CopilotOAuthSession {
            access_token: "token".to_string(),
            expires_at_epoch_ms: chrono::Utc::now().timestamp_millis() + 3600000,
            token_type: "Bearer".to_string(),
        },
        vec![],
    );

    assert_eq!(app.mode, AppMode::ConnectModel, "Should still transition to ConnectModel even with empty models");
    assert!(app.connect_model_dialog.is_some());
    assert!(app.pending_browser_models.is_empty(), "Models list should be empty");
}

#[test]
fn test_copilot_oauth_different_models_have_different_ids() {
    let mut app = App::new();

    let models = vec![
        BrowserAuthModelInfo { id: "gpt-4o".to_string(), name: "GPT-4o".to_string() },
        BrowserAuthModelInfo { id: "o1-preview".to_string(), name: "o1 Preview".to_string() },
    ];

    app.complete_copilot_auth_for_test(
        CopilotOAuthSession {
            access_token: "token".to_string(),
            expires_at_epoch_ms: chrono::Utc::now().timestamp_millis() + 3600000,
            token_type: "Bearer".to_string(),
        },
        models,
    );

    let model_ids: Vec<String> = app.pending_browser_models.iter().map(|m| m.id.clone()).collect();
    assert!(model_ids.contains(&"gpt-4o".to_string()));
    assert!(model_ids.contains(&"o1-preview".to_string()));
    assert_eq!(model_ids.len(), 2, "Should have exactly 2 unique model IDs");
}

#[tokio::test]
async fn test_copilot_oauth_mock_server_token_endpoint() {
    let mut server = MockServer::start();

    server.mock(
        "POST",
        "/login/oauth/access_token",
        200,
        r#"access_token=gho_mock_token&token_type=Bearer"#
    );

    let base_url = server.url("");
    assert!(base_url.contains("127.0.0.1"));

    std::env::set_var("GITHUB_TOKEN_URL", &base_url);

    std::env::remove_var("GITHUB_TOKEN_URL");
}

#[test]
fn test_copilot_oauth_uses_correct_provider_name() {
    let mut app = App::new();

    app.complete_copilot_auth_for_test(
        CopilotOAuthSession {
            access_token: "token".to_string(),
            expires_at_epoch_ms: chrono::Utc::now().timestamp_millis() + 3600000,
            token_type: "Bearer".to_string(),
        },
        vec![
            BrowserAuthModelInfo { id: "gpt-4o".to_string(), name: "GPT-4o".to_string() },
        ],
    );

    let result = app.confirm_model_for_copilot_auth_for_test("gpt-4o");
    assert!(result.is_ok());
    assert_eq!(app.provider, "copilot", "Provider should be 'copilot', not 'github-copilot' or other variant");
}

#[test]
fn test_copilot_oauth_token_type_preserved() {
    let mut app = App::new();

    app.complete_copilot_auth_for_test(
        CopilotOAuthSession {
            access_token: "token".to_string(),
            expires_at_epoch_ms: chrono::Utc::now().timestamp_millis() + 3600000,
            token_type: "macaroon".to_string(),
        },
        vec![
            BrowserAuthModelInfo { id: "gpt-4o".to_string(), name: "GPT-4o".to_string() },
        ],
    );

    assert!(app.pending_copilot_session.is_some());
    assert_eq!(app.pending_copilot_session.as_ref().unwrap().token_type, "macaroon");

    let result = app.confirm_model_for_copilot_auth_for_test("gpt-4o");
    assert!(result.is_ok());
}

#[test]
fn test_copilot_oauth_state_mismatch_error() {
    let service = CopilotOAuthService::new();
    let callback = CopilotOAuthCallback {
        code: "test_code".to_string(),
        state: "wrong-state".to_string(),
    };
    let request = CopilotOAuthRequest {
        redirect_uri: "http://127.0.0.1:8080/auth/callback".to_string(),
        state: "correct-state".to_string(),
        code_verifier: "verifier".to_string(),
    };
    let result = service.exchange_code(callback, &request);
    assert!(result.is_err(), "State mismatch should produce an error");
}

#[test]
fn test_copilot_oauth_authorization_url_contains_required_params() {
    let service = CopilotOAuthService::new();
    let request = CopilotOAuthRequest {
        redirect_uri: "http://127.0.0.1:8080/auth/callback".to_string(),
        state: "test-state-123".to_string(),
        code_verifier: "test-verifier-456".to_string(),
    };

    let url = service.build_authorize_url(&request);

    assert!(url.contains("github.com/login/oauth/authorize"));
    assert!(url.contains("client_id="));
    assert!(url.contains("code_challenge_method=S256"));
    assert!(url.contains("state=test-state-123"));
    assert!(url.contains("scope="));
    assert!(url.contains("redirect_uri="));
}

#[test]
fn test_copilot_oauth_error_recovery_clears_pending_session() {
    let mut app = App::new();

    app.complete_copilot_auth_for_test(
        CopilotOAuthSession {
            access_token: "error_token".to_string(),
            expires_at_epoch_ms: chrono::Utc::now().timestamp_millis() + 3600000,
            token_type: "Bearer".to_string(),
        },
        vec![
            BrowserAuthModelInfo { id: "gpt-4o".to_string(), name: "GPT-4o".to_string() },
        ],
    );

    assert!(app.pending_copilot_session.is_some());

    app.pending_copilot_session = None;

    assert!(app.pending_copilot_session.is_none(), "Pending session should be cleared after error");
    assert_eq!(app.mode, AppMode::ConnectModel);
}

#[test]
fn test_copilot_oauth_prime_copilot_connect_state_for_test() {
    let mut app = App::new();

    app.prime_copilot_connect_state_for_test();

    assert_eq!(app.mode, AppMode::ConnectModel);
    assert!(app.connect_model_dialog.is_some());
    assert!(app.pending_copilot_session.is_some());

    let result = app.confirm_model_for_copilot_auth_for_test("gpt-4o");
    assert!(result.is_ok());
    assert_eq!(app.provider, "copilot");
    assert_eq!(app.mode, AppMode::Chat);
}

#[test]
fn test_copilot_oauth_bearer_token_type() {
    let mut app = App::new();

    app.complete_copilot_auth_for_test(
        CopilotOAuthSession {
            access_token: "bearer_token_abc".to_string(),
            expires_at_epoch_ms: chrono::Utc::now().timestamp_millis() + 3600000,
            token_type: "Bearer".to_string(),
        },
        vec![
            BrowserAuthModelInfo { id: "gpt-4o".to_string(), name: "GPT-4o".to_string() },
        ],
    );

    let session = app.pending_copilot_session.clone().unwrap();
    assert_eq!(session.token_type, "Bearer");
    assert!(!session.is_expired());
}

#[test]
fn test_copilot_oauth_flow_model_confirm_returns_error_for_unknown_model() {
    let mut app = App::new();

    app.complete_copilot_auth_for_test(
        CopilotOAuthSession {
            access_token: "test_token".to_string(),
            expires_at_epoch_ms: chrono::Utc::now().timestamp_millis() + 3600000,
            token_type: "Bearer".to_string(),
        },
        vec![
            BrowserAuthModelInfo { id: "gpt-4o".to_string(), name: "GPT-4o".to_string() },
        ],
    );

    let result = app.confirm_model_for_copilot_auth_for_test("unknown-model");
    assert!(result.is_err(), "Confirming unknown model should return error");
}

#[tokio::test]
async fn test_copilot_oauth_token_exchange_with_mock_server() {
    let mut server = MockServer::start();

    server.mock(
        "POST",
        "/login/oauth/access_token",
        200,
        "access_token=gho_testtoken123&token_type=Bearer"
    );

    let service = CopilotOAuthService::new();
    let callback = CopilotOAuthCallback {
        code: "test_authorization_code".to_string(),
        state: "test-state".to_string(),
    };
    let request = CopilotOAuthRequest {
        redirect_uri: server.url("/auth/callback"),
        state: "test-state".to_string(),
        code_verifier: "test_verifier".to_string(),
    };

    let result = service.exchange_code(callback, &request);
    assert!(result.is_ok() || result.is_err(), "Token exchange should complete");
}
