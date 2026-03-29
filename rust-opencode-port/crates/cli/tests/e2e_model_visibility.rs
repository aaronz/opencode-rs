use crate::common::{TestHarness, EMPTY_VEC};

mod common;

#[test]
fn test_model_visibility_hide_show() {
    let harness = TestHarness::setup();

    harness.create_mock_provider("test-provider", &["model-1", "model-2"]);

    let hide_output = harness.run_cli(&["models", "visibility", "--hide", "model-1"]);
    assert!(hide_output.status.success());

    let list_output = harness.run_cli(&["models", "--visibility", "visible", "--json"]);
    assert!(list_output.status.success());

    let stdout = String::from_utf8_lossy(&list_output.stdout);
    let json: serde_json::Value = serde_json::from_str(&stdout).unwrap_or_default();

    let models = json.as_array().unwrap_or(&EMPTY_VEC);
    let has_model_1 = models.iter().any(|m| m["id"] == "model-1");
    assert!(
        !has_model_1,
        "Hidden model should not appear in visible list"
    );
}

#[test]
fn test_model_visibility_list_hidden() {
    let harness = TestHarness::setup();

    harness.create_mock_provider("test-provider", &["model-1", "model-2"]);

    harness.run_cli(&["models", "visibility", "--hide", "model-1"]);

    let list_output = harness.run_cli(&["models", "visibility", "--list-hidden"]);
    assert!(list_output.status.success());

    let stdout = String::from_utf8_lossy(&list_output.stdout);
    assert!(stdout.contains("model-1"));
}

#[test]
fn test_model_visibility_show_restores() {
    let harness = TestHarness::setup();

    harness.create_mock_provider("test-provider", &["model-1"]);

    harness.run_cli(&["models", "visibility", "--hide", "model-1"]);

    let show_output = harness.run_cli(&["models", "visibility", "--show", "model-1"]);
    assert!(show_output.status.success());

    let list_output = harness.run_cli(&["models", "--visibility", "visible", "--json"]);
    let stdout = String::from_utf8_lossy(&list_output.stdout);
    let json: serde_json::Value = serde_json::from_str(&stdout).unwrap_or_default();

    let models = json.as_array().unwrap_or(&EMPTY_VEC);
    let has_model_1 = models.iter().any(|m| m["id"] == "model-1");
    assert!(has_model_1, "Shown model should appear in visible list");
}
