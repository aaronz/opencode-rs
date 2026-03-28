mod common;
use common::TestHarness;

#[test]
fn test_models_list() {
    let harness = TestHarness::setup();
    let result = harness.run_cli_json(&["models", "--json"]);

    assert!(
        result.get("models").is_some(),
        "Result should contain models array"
    );
    let models = result["models"].as_array().unwrap();
    assert!(!models.is_empty(), "Should have at least one model");

    for model in models {
        assert!(model.get("id").is_some(), "Model should have ID");
        assert!(model.get("name").is_some(), "Model should have name");
        assert!(
            model.get("provider").is_some(),
            "Model should have provider"
        );
    }
}

#[test]
fn test_model_switch() {
    let harness = TestHarness::setup();
    let models = harness.run_cli_json(&["models", "--json"]);
    let model_id = models["models"][0]["id"].as_str().unwrap();

    let switch_result = harness.run_cli_json(&["models", "--switch", model_id]);
    assert_eq!(
        switch_result["active_model"], model_id,
        "Should switch to selected model"
    );

    let config = harness.run_cli_json(&["config", "--json"]);
    assert_eq!(
        config["model"], model_id,
        "Config should persist model selection"
    );
}

#[test]
fn test_provider_list() {
    let harness = TestHarness::setup();
    let result = harness.run_cli_json(&["providers", "--json"]);

    assert!(
        result.get("providers").is_some(),
        "Result should contain providers array"
    );
    let providers = result["providers"].as_array().unwrap();
    assert!(!providers.is_empty(), "Should have at least one provider");

    for provider in providers {
        assert!(provider.get("id").is_some(), "Provider should have ID");
        assert!(provider.get("name").is_some(), "Provider should have name");
        assert!(
            provider.get("status").is_some(),
            "Provider should have status"
        );
    }
}

#[test]
fn test_model_selection_dialog() {
    let harness = TestHarness::setup();
    let result = harness.run_cli_json(&["tui", "--action", "open-model-dialog"]);

    assert!(result.get("dialog").is_some(), "Should open dialog");
    assert_eq!(
        result["dialog"], "model-selection",
        "Should be model selection dialog"
    );
}

#[test]
fn test_provider_connection_error() {
    let harness = TestHarness::setup();
    let result = harness.run_cli(&["providers", "--test-connection", "invalid-provider"]);

    assert!(!result.status.success(), "Should fail for invalid provider");
    let stderr = String::from_utf8_lossy(&result.stderr);
    assert!(
        stderr.contains("error") || stderr.contains("Error"),
        "Should show error message"
    );
}
