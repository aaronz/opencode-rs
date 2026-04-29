use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use opencode_llm::BrowserAuthModelInfo;
use opencode_tui::action::AppMode;
use opencode_tui::app::App;
use opencode_tui::dialogs::ApiKeyInputDialog;
use opencode_tui::dialogs::DialogAction;
use opencode_tui::Dialog;

#[test]
fn test_validation_success_transitions_to_connect_model() {
    let temp_dir = tempfile::TempDir::new().unwrap();
    std::env::set_var("OPENCODE_DATA_DIR", temp_dir.path().to_str().unwrap());

    let mut app = App::new();

    app.mode = AppMode::ConnectProgress;
    app.validation_in_progress = true;
    app.pending_connect_provider = Some("openai".to_string());
    app.pending_api_key_for_validation = Some("sk-valid-key".to_string());

    app.simulate_validation_complete_for_testing(
        true,
        None,
        Some(vec![BrowserAuthModelInfo {
            id: "gpt-4o".to_string(),
            name: "GPT-4o".to_string(),
            variants: vec![],
        }]),
    );

    assert_eq!(app.mode, AppMode::ConnectModel);
    assert!(!app.validation_in_progress);
}

#[test]
fn test_validation_failure_shows_error_dialog() {
    let temp_dir = tempfile::TempDir::new().unwrap();
    std::env::set_var("OPENCODE_DATA_DIR", temp_dir.path().to_str().unwrap());

    let mut app = App::new();

    app.mode = AppMode::ConnectProgress;
    app.validation_in_progress = true;
    app.pending_connect_provider = Some("minimax-cn".to_string());
    app.pending_api_key_for_validation = Some("bad-key".to_string());

    app.simulate_validation_complete_for_testing(
        false,
        Some("Provider returned 404".to_string()),
        None,
    );

    assert_eq!(app.mode, AppMode::ConnectApiKeyError);
    assert!(!app.validation_in_progress);
    assert!(app.validation_error_dialog.is_some());
}

#[test]
fn test_api_key_input_esc_returns_close_action() {
    let mut dialog = ApiKeyInputDialog::new(
        opencode_tui::theme::Theme::default(),
        "minimax-cn".to_string(),
        "MiniMax".to_string(),
    );

    let action = dialog.handle_input(KeyEvent::new(KeyCode::Esc, KeyModifiers::NONE));

    assert_eq!(action, DialogAction::Close);
}

#[test]
fn test_api_key_input_accepts_valid_key() {
    let mut dialog = ApiKeyInputDialog::new(
        opencode_tui::theme::Theme::default(),
        "openai".to_string(),
        "OpenAI".to_string(),
    );

    for c in "sk-testkey12345".chars() {
        dialog.handle_input(KeyEvent::new(KeyCode::Char(c), KeyModifiers::NONE));
    }

    let action = dialog.handle_input(KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE));

    assert!(matches!(action, DialogAction::Confirm(_)));
}

#[test]
fn test_api_key_input_rejects_short_key() {
    let mut dialog = ApiKeyInputDialog::new(
        opencode_tui::theme::Theme::default(),
        "openai".to_string(),
        "OpenAI".to_string(),
    );

    for c in "short".chars() {
        dialog.handle_input(KeyEvent::new(KeyCode::Char(c), KeyModifiers::NONE));
    }

    let action = dialog.handle_input(KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE));

    assert!(!matches!(action, DialogAction::Confirm(_)));
}

#[test]
fn test_api_key_input_rejects_empty_key() {
    let mut dialog = ApiKeyInputDialog::new(
        opencode_tui::theme::Theme::default(),
        "openai".to_string(),
        "OpenAI".to_string(),
    );

    let action = dialog.handle_input(KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE));

    assert!(!matches!(action, DialogAction::Confirm(_)));
}

#[test]
fn test_api_key_input_handles_backspace() {
    let mut dialog = ApiKeyInputDialog::new(
        opencode_tui::theme::Theme::default(),
        "openai".to_string(),
        "OpenAI".to_string(),
    );

    for c in "test".chars() {
        dialog.handle_input(KeyEvent::new(KeyCode::Char(c), KeyModifiers::NONE));
    }

    dialog.handle_input(KeyEvent::new(KeyCode::Backspace, KeyModifiers::NONE));
    dialog.handle_input(KeyEvent::new(KeyCode::Backspace, KeyModifiers::NONE));

    let action = dialog.handle_input(KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE));

    assert!(!matches!(action, DialogAction::Confirm(_)));
}

#[cfg(test)]
mod connect_flow_regression_tests {
    use super::*;

    #[test]
    fn connect_minimax_success_transitions_to_model() {
        let temp_dir = tempfile::TempDir::new().unwrap();
        std::env::set_var("OPENCODE_DATA_DIR", temp_dir.path().to_str().unwrap());

        let mut app = App::new();
        app.pending_connect_provider = Some("minimax-cn".to_string());
        app.pending_api_key_for_validation = Some("test-api-key-12345".to_string());
        app.validation_in_progress = true;
        app.mode = AppMode::ConnectProgress;

        let models = vec![
            BrowserAuthModelInfo {
                id: "MiniMax-M2.7".to_string(),
                name: "MiniMax M2.7".to_string(),
                variants: vec![],
            },
            BrowserAuthModelInfo {
                id: "MiniMax-M2.5".to_string(),
                name: "MiniMax M2.5".to_string(),
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
            !app.validation_in_progress,
            "validation_in_progress should be cleared"
        );
        assert!(
            app.connect_model_dialog.is_some(),
            "Model selection dialog should appear"
        );
    }

    #[test]
    fn connect_minimax_404_error_is_visible() {
        let temp_dir = tempfile::TempDir::new().unwrap();
        std::env::set_var("OPENCODE_DATA_DIR", temp_dir.path().to_str().unwrap());

        let mut app = App::new();
        app.pending_connect_provider = Some("minimax-cn".to_string());
        app.pending_api_key_for_validation = Some("bad-api-key".to_string());
        app.validation_in_progress = true;
        app.mode = AppMode::ConnectProgress;

        app.simulate_validation_complete_for_testing(
            false,
            Some("Provider returned 404 Not Found".to_string()),
            None,
        );

        assert_eq!(
            app.mode,
            AppMode::ConnectApiKeyError,
            "Mode should be ConnectApiKeyError after 404"
        );
        assert!(
            !app.validation_in_progress,
            "validation_in_progress should be cleared"
        );
        assert!(
            app.validation_error_dialog.is_some(),
            "Error dialog should be visible"
        );
    }

    #[test]
    fn connect_api_key_input_validates_key_format() {
        let mut dialog = ApiKeyInputDialog::new(
            opencode_tui::theme::Theme::default(),
            "minimax-cn".to_string(),
            "MiniMax".to_string(),
        );

        for c in "short".chars() {
            dialog.handle_input(KeyEvent::new(KeyCode::Char(c), KeyModifiers::NONE));
        }

        dialog.handle_input(KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE));

        assert!(
            dialog.get_api_key().len() < 10,
            "API key should be too short"
        );
    }

    #[test]
    fn connect_minimax_api_key_persisted_to_config_after_success() {
        let temp_dir = tempfile::TempDir::new().unwrap();
        std::env::set_var("OPENCODE_DATA_DIR", temp_dir.path().to_str().unwrap());

        let mut app = App::new();
        app.pending_connect_provider = Some("minimax-cn".to_string());
        app.pending_api_key_for_validation = Some("test-api-key-12345".to_string());
        app.validation_in_progress = true;
        app.mode = AppMode::ConnectProgress;

        app.simulate_validation_complete_for_testing(
            true,
            None,
            Some(vec![BrowserAuthModelInfo {
                id: "MiniMax-M2.7".to_string(),
                name: "MiniMax M2.7".to_string(),
                variants: vec![],
            }]),
        );

        let providers = app
            .config
            .providers
            .as_ref()
            .expect("providers should be set");
        let minimax_provider = providers
            .iter()
            .find(|p| p.name == "minimax-cn")
            .expect("minimax-cn should be in providers");
        assert_eq!(
            minimax_provider.api_key,
            Some("test-api-key-12345".to_string()),
            "API key should be persisted in config"
        );
        assert_eq!(
            minimax_provider.default_model,
            Some("MiniMax-M2.7".to_string()),
            "default model should be set"
        );
    }

    #[test]
    fn connect_ollama_local_success_shows_model_dialog() {
        let temp_dir = tempfile::TempDir::new().unwrap();
        std::env::set_var("OPENCODE_DATA_DIR", temp_dir.path().to_str().unwrap());

        let mut app = App::new();
        app.pending_connect_provider = Some("ollama".to_string());
        // For local connect, api_key is typically the provider id itself
        app.pending_api_key_for_validation = Some("ollama".to_string());
        app.validation_in_progress = true;
        app.mode = AppMode::ConnectProgress;

        // Simulate Ollama returning 1 model
        app.simulate_validation_complete_for_testing(
            true,
            None,
            Some(vec![BrowserAuthModelInfo {
                id: "qwen3.5:9b".to_string(),
                name: "qwen3.5:9b".to_string(),
                variants: vec![],
            }]),
        );

        assert_eq!(
            app.mode,
            AppMode::ConnectModel,
            "Mode should be ConnectModel after successful Ollama validation"
        );
        assert!(
            !app.validation_in_progress,
            "validation_in_progress should be cleared"
        );
        assert!(
            app.connect_model_dialog.is_some(),
            "Model selection dialog should appear for Ollama"
        );
    }

    #[test]
    fn connect_ollama_local_with_multiple_models() {
        let temp_dir = tempfile::TempDir::new().unwrap();
        std::env::set_var("OPENCODE_DATA_DIR", temp_dir.path().to_str().unwrap());

        let mut app = App::new();
        app.pending_connect_provider = Some("ollama".to_string());
        app.pending_api_key_for_validation = Some("ollama".to_string());
        app.validation_in_progress = true;
        app.mode = AppMode::ConnectProgress;

        // Simulate Ollama returning multiple models
        app.simulate_validation_complete_for_testing(
            true,
            None,
            Some(vec![
                BrowserAuthModelInfo {
                    id: "llama3".to_string(),
                    name: "Llama 3".to_string(),
                    variants: vec![],
                },
                BrowserAuthModelInfo {
                    id: "qwen2.5".to_string(),
                    name: "Qwen 2.5".to_string(),
                    variants: vec![],
                },
                BrowserAuthModelInfo {
                    id: "mistral".to_string(),
                    name: "Mistral".to_string(),
                    variants: vec![],
                },
            ]),
        );

        assert_eq!(
            app.mode,
            AppMode::ConnectModel,
            "Mode should be ConnectModel after successful Ollama validation"
        );
        assert!(
            app.connect_model_dialog.is_some(),
            "Model selection dialog should appear for Ollama with multiple models"
        );
    }

    #[test]
    fn connect_ollama_local_saves_provider_config() {
        let temp_dir = tempfile::TempDir::new().unwrap();
        std::env::set_var("OPENCODE_DATA_DIR", temp_dir.path().to_str().unwrap());

        let mut app = App::new();
        app.pending_connect_provider = Some("ollama".to_string());
        app.pending_api_key_for_validation = Some("ollama".to_string());
        app.validation_in_progress = true;
        app.mode = AppMode::ConnectProgress;

        app.simulate_validation_complete_for_testing(
            true,
            None,
            Some(vec![BrowserAuthModelInfo {
                id: "llama3".to_string(),
                name: "Llama 3".to_string(),
                variants: vec![],
            }]),
        );

        let providers = app
            .config
            .providers
            .as_ref()
            .expect("providers should be set");
        let ollama_provider = providers
            .iter()
            .find(|p| p.name == "ollama")
            .expect("ollama should be in providers");
        assert_eq!(
            ollama_provider.default_model,
            Some("llama3".to_string()),
            "default model should be set to llama3"
        );
    }

    /// Regression test: verifies that when ValidationComplete is received,
    /// the mode changes AND pending_api_key_models is properly populated.
    #[test]
    fn connect_ollama_validation_complete_populates_pending_models() {
        let temp_dir = tempfile::TempDir::new().unwrap();
        std::env::set_var("OPENCODE_DATA_DIR", temp_dir.path().to_str().unwrap());

        let mut app = App::new();
        app.pending_connect_provider = Some("ollama".to_string());
        app.pending_api_key_for_validation = Some("ollama".to_string());
        app.validation_in_progress = true;
        app.mode = AppMode::ConnectProgress;

        let models = vec![
            BrowserAuthModelInfo {
                id: "qwen3.5:9b".to_string(),
                name: "qwen3.5:9b".to_string(),
                variants: vec![],
            },
            BrowserAuthModelInfo {
                id: "llama3".to_string(),
                name: "Llama 3".to_string(),
                variants: vec![],
            },
        ];

        app.simulate_validation_complete_for_testing(true, None, Some(models));

        assert_eq!(app.mode, AppMode::ConnectModel);
        assert!(app.connect_model_dialog.is_some(), "Dialog should exist");
        assert_eq!(
            app.pending_api_key_models.len(),
            2,
            "pending_api_key_models should have 2 models"
        );
        assert_eq!(
            app.pending_api_key_models[0].id, "qwen3.5:9b",
            "First model should be qwen3.5:9b"
        );
        assert_eq!(
            app.pending_api_key_models[1].id, "llama3",
            "Second model should be llama3"
        );
    }
}
