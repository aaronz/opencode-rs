mod common;
use common::TestHarness;

#[test]
fn test_models_list_contains_registry_fields() {
    let harness = TestHarness::setup();
    let result = harness.run_cli_json(&["models"]);

    let models = result["models"].as_array().unwrap();
    assert!(!models.is_empty(), "Should have at least one model");

    let first = &models[0];
    assert!(first.get("id").is_some(), "Model should have ID");
    assert!(first.get("name").is_some(), "Model should have name");
    assert!(
        first.get("provider").is_some(),
        "Model should have provider"
    );
    assert!(
        first.get("supports_streaming").is_some(),
        "Model should expose streaming support"
    );
    assert!(
        first.get("max_input_tokens").is_some(),
        "Model should expose max input tokens"
    );
}

#[test]
fn test_model_switch() {
    let harness = TestHarness::setup();
    let models = harness.run_cli_json(&["models"]);
    let model_id = models["models"][0]["id"].as_str().unwrap();

    let switch_result = harness.run_cli_json(&["models", "--switch", model_id]);
    assert_eq!(
        switch_result["active_model"], model_id,
        "Should switch to selected model"
    );

    let config = harness.run_cli_json(&["config"]);
    assert_eq!(
        config["model"], model_id,
        "Config should persist model selection"
    );
}

#[test]
fn test_provider_list_contains_real_status_fields() {
    let harness = TestHarness::setup();
    let result = harness.run_cli_json(&["providers"]);

    assert_eq!(result["action"], "list");
    let providers = result["providers"].as_array().unwrap();
    assert!(!providers.is_empty(), "Should have at least one provider");

    for provider in providers {
        assert!(
            provider.get("id").is_some(),
            "Provider should have id field"
        );
        assert!(
            provider.get("name").is_some(),
            "Provider should have name field"
        );
        assert!(
            provider.get("status").is_some(),
            "Provider should have status field"
        );
        assert!(
            provider.get("enabled").is_some(),
            "Provider should have enabled field"
        );
    }
}

#[test]
fn test_model_selection_dialog() {
    let harness = TestHarness::setup();
    let output = harness.run_cli(&["tui", "--json", "--action", "open-model-dialog"]);

    assert!(output.status.success(), "TUI dialog action should succeed");
    let stdout = String::from_utf8_lossy(&output.stdout);
    let result: serde_json::Value = serde_json::from_str(&stdout).unwrap();

    assert!(result.get("dialog").is_some(), "Should open dialog");
    assert_eq!(
        result["dialog"], "model-selection",
        "Should be model selection dialog"
    );
}

#[test]
fn test_tui_confirm_model_switch_persists_selection() {
    let harness = TestHarness::setup();
    let models = harness.run_cli_json(&["models"]);
    let model_id = models["models"][0]["id"].as_str().unwrap();

    let output = harness.run_cli(&[
        "tui",
        "--json",
        "--action",
        "confirm-model-switch",
        "--model",
        model_id,
    ]);

    assert!(
        output.status.success(),
        "confirm-model-switch should succeed"
    );

    let stdout = String::from_utf8_lossy(&output.stdout);
    let result: serde_json::Value = serde_json::from_str(&stdout).unwrap();
    assert_eq!(result["active_model"], model_id);
    assert_eq!(result["status"], "confirmed");

    let config = harness.run_cli_json(&["config"]);
    assert_eq!(
        config["model"], model_id,
        "selection should persist to config"
    );
}

#[test]
fn test_tui_confirm_model_switch_requires_model_argument() {
    let harness = TestHarness::setup();
    let output = harness.run_cli(&["tui", "--json", "--action", "confirm-model-switch"]);

    assert!(
        !output.status.success(),
        "confirm-model-switch should fail without --model"
    );
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("Missing --model for confirm-model-switch"),
        "stderr should explain the missing model argument"
    );
}

#[test]
fn test_tui_confirm_model_switch_rejects_unknown_model() {
    let harness = TestHarness::setup();
    let output = harness.run_cli(&[
        "tui",
        "--json",
        "--action",
        "confirm-model-switch",
        "--model",
        "not-a-real-model",
    ]);

    assert!(
        !output.status.success(),
        "confirm-model-switch should fail for an unknown model"
    );
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("Unknown model: not-a-real-model"),
        "stderr should explain the invalid model"
    );
}

#[test]
fn test_tui_close_model_dialog_returns_closed_status() {
    let harness = TestHarness::setup();
    let output = harness.run_cli(&["tui", "--json", "--action", "close-model-dialog"]);

    assert!(output.status.success(), "close-model-dialog should succeed");
    let stdout = String::from_utf8_lossy(&output.stdout);
    let result: serde_json::Value = serde_json::from_str(&stdout).unwrap();

    assert_eq!(result["dialog"], "model-selection");
    assert_eq!(result["status"], "closed");
}

#[test]
fn test_tui_cancel_model_switch_keeps_existing_selection() {
    let harness = TestHarness::setup();
    let models = harness.run_cli_json(&["models"]);
    let original_model = models["models"][0]["id"].as_str().unwrap();
    let next_model = models["models"][1]["id"].as_str().unwrap();

    let switch_output = harness.run_cli(&["models", "--switch", original_model]);
    assert!(
        switch_output.status.success(),
        "setup switch should succeed"
    );

    let output = harness.run_cli(&[
        "tui",
        "--json",
        "--action",
        "cancel-model-switch",
        "--model",
        next_model,
    ]);

    assert!(
        output.status.success(),
        "cancel-model-switch should succeed"
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    let result: serde_json::Value = serde_json::from_str(&stdout).unwrap();
    assert_eq!(result["active_model"], original_model);
    assert_eq!(result["status"], "cancelled");

    let config = harness.run_cli_json(&["config"]);
    assert_eq!(
        config["model"], original_model,
        "cancel should keep config unchanged"
    );
}

#[test]
fn test_tui_cancel_model_switch_without_existing_selection_returns_null_active_model() {
    let harness = TestHarness::setup();
    let output = harness.run_cli(&[
        "tui",
        "--json",
        "--action",
        "cancel-model-switch",
        "--model",
        "not-a-real-model",
    ]);

    assert!(
        output.status.success(),
        "cancel-model-switch should succeed without an existing selection"
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    let result: serde_json::Value = serde_json::from_str(&stdout).unwrap();
    assert!(result["active_model"].is_null());
    assert_eq!(result["status"], "cancelled");
}

#[test]
fn test_tui_cancel_model_switch_ignores_invalid_pending_model() {
    let harness = TestHarness::setup();
    let models = harness.run_cli_json(&["models"]);
    let original_model = models["models"][0]["id"].as_str().unwrap();

    let switch_output = harness.run_cli(&["models", "--switch", original_model]);
    assert!(
        switch_output.status.success(),
        "setup switch should succeed"
    );

    let output = harness.run_cli(&[
        "tui",
        "--json",
        "--action",
        "cancel-model-switch",
        "--model",
        "not-a-real-model",
    ]);

    assert!(
        output.status.success(),
        "cancel-model-switch should ignore invalid pending model input"
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    let result: serde_json::Value = serde_json::from_str(&stdout).unwrap();
    assert_eq!(result["active_model"], original_model);
    assert_eq!(result["status"], "cancelled");

    let config = harness.run_cli_json(&["config"]);
    assert_eq!(
        config["model"], original_model,
        "cancel should preserve config"
    );
}

#[test]
fn test_tui_unknown_action_returns_explicit_error() {
    let harness = TestHarness::setup();
    let output = harness.run_cli(&["tui", "--json", "--action", "not-a-real-action"]);

    assert!(!output.status.success(), "unknown tui action should fail");
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("Unknown tui action: not-a-real-action"),
        "stderr should explain the unknown action"
    );
}

#[test]
fn test_provider_connection_error() {
    let harness = TestHarness::setup();
    let result = harness.run_cli(&["providers", "--test-connection", "invalid-provider"]);

    assert!(!result.status.success(), "Should fail for invalid provider");
    let stderr = String::from_utf8_lossy(&result.stderr);
    assert!(
        stderr.contains("Unknown provider: invalid-provider"),
        "Should show an explicit unknown provider error"
    );
}
