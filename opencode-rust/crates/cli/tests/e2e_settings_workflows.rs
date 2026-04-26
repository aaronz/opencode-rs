mod common;
use common::TestHarness;

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

#[test]
fn test_config_set_string_value_persists() {
    let harness = TestHarness::setup();

    let config_dir = harness.temp_dir.path().join("config");
    std::fs::create_dir_all(&config_dir).unwrap();
    let config_path = config_dir.join("config.json");

    std::fs::write(&config_path, "{}").unwrap();

    let result = harness.run_cli(&["config", "--set", "model", "gpt-4o"]);

    assert!(
        result.status.success(),
        "config set should succeed: {}",
        String::from_utf8_lossy(&result.stderr)
    );

    let config_content = std::fs::read_to_string(&config_path).unwrap();
    let config_json: serde_json::Value = serde_json::from_str(&config_content).unwrap();

    assert!(
        config_json.get("model").is_some(),
        "Config file should contain model field"
    );
    assert_eq!(
        config_json.get("model").unwrap(),
        "gpt-4o",
        "Model value should be gpt-4o"
    );
}

#[test]
fn test_config_set_nested_value() {
    let harness = TestHarness::setup();

    let config_dir = harness.temp_dir.path().join("config");
    std::fs::create_dir_all(&config_dir).unwrap();
    let config_path = config_dir.join("config.json");

    std::fs::write(&config_path, r#"{"agent": {"agents": {"default": {}}}}"#).unwrap();

    let result = harness.run_cli(&[
        "config",
        "--set",
        "agent.agents.default.model",
        "claude-3.5-sonnet",
    ]);

    assert!(
        result.status.success(),
        "config set should succeed for nested key: {}",
        String::from_utf8_lossy(&result.stderr)
    );

    let config_content = std::fs::read_to_string(&config_path).unwrap();
    let config_json: serde_json::Value = serde_json::from_str(&config_content).unwrap();

    let agent = config_json.get("agent").unwrap();
    let agents = agent.get("agents").unwrap();
    let default_agent = agents.get("default").unwrap();
    let model = default_agent.get("model").unwrap();

    assert_eq!(model.as_str().unwrap(), "claude-3.5-sonnet");
}

#[test]
fn test_config_set_type_validation_rejects_mismatch() {
    let harness = TestHarness::setup();

    let config_dir = harness.temp_dir.path().join("config");
    std::fs::create_dir_all(&config_dir).unwrap();
    let config_path = config_dir.join("config.json");

    std::fs::write(&config_path, r#"{"model": "gpt-4o"}"#).unwrap();

    let result = harness.run_cli(&["config", "--set", "model", "not_a_number_for_model"]);

    assert!(
        result.status.success(),
        "Setting string to string field should succeed"
    );

    std::fs::write(&config_path, r#"{"temperature": 0.7}"#).unwrap();

    let result = harness.run_cli(&["config", "--set", "temperature", "not_a_number"]);

    assert!(
        !result.status.success(),
        "Should fail when setting non-number to number field"
    );
}

#[test]
fn test_config_get_after_config_set() {
    let harness = TestHarness::setup();

    let config_dir = harness.temp_dir.path().join("config");
    std::fs::create_dir_all(&config_dir).unwrap();
    let config_path = config_dir.join("config.json");

    std::fs::write(&config_path, "{}").unwrap();

    let set_result = harness.run_cli(&["config", "--set", "model", "gpt-4o"]);

    assert!(
        set_result.status.success(),
        "config set should succeed: {}",
        String::from_utf8_lossy(&set_result.stderr)
    );

    let config_content = std::fs::read_to_string(&config_path).unwrap();
    let config_json: serde_json::Value = serde_json::from_str(&config_content).unwrap();

    assert!(
        config_json.get("model").is_some(),
        "Config file should contain model field after set"
    );
    assert_eq!(
        config_json.get("model").unwrap().as_str().unwrap(),
        "gpt-4o",
        "Model value should be gpt-4o after config set"
    );
}
