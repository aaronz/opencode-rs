use crate::common::TestHarness;

#[test]
fn test_settings_general() {
    let harness = TestHarness::setup();
    let result = harness.run_cli_json(&["config", "--json"]);

    assert!(result.get("theme").is_some(), "Config should have theme");
    assert!(result.get("editor").is_some(), "Config should have editor");
}

#[test]
fn test_settings_keybinds() {
    let harness = TestHarness::setup();
    let result = harness.run_cli_json(&["config", "--keybinds", "--json"]);

    assert!(
        result.get("keybinds").is_some(),
        "Should have keybinds section"
    );
    let keybinds = result["keybinds"].as_object().unwrap();
    assert!(
        keybinds.contains_key("commands"),
        "Should have commands keybind"
    );
    assert!(
        keybinds.contains_key("timeline"),
        "Should have timeline keybind"
    );
}

#[test]
fn test_settings_models() {
    let harness = TestHarness::setup();
    let result = harness.run_cli_json(&["config", "--models", "--json"]);

    assert!(
        result.get("default_model").is_some(),
        "Should have default model"
    );
    assert!(
        result.get("available_models").is_some(),
        "Should have available models list"
    );
}

#[test]
fn test_settings_providers() {
    let harness = TestHarness::setup();
    let result = harness.run_cli_json(&["config", "--providers", "--json"]);

    assert!(
        result.get("providers").is_some(),
        "Should have providers configuration"
    );
}

#[test]
fn test_settings_validation() {
    let harness = TestHarness::setup();
    let result = harness.run_cli(&["config", "--set", "invalid-key", "invalid-value"]);

    assert!(!result.status.success(), "Should fail for invalid setting");
}
