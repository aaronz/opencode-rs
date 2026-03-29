use crate::common::{TestHarness, EMPTY_VEC};

mod common;

#[test]
fn test_model_visibility_hide_show() {
    let harness = TestHarness::setup();

    let hide_output = harness.run_cli(&["models", "visibility", "--hide", "gpt-4o"]);
    assert!(hide_output.status.success());

    let json = harness.run_cli_json(&["models", "--visibility", "visible"]);
    let models = json["models"].as_array().unwrap_or(&EMPTY_VEC);
    let has_model_1 = models.iter().any(|m| m["id"] == "gpt-4o");
    assert!(
        !has_model_1,
        "Hidden model should not appear in visible list"
    );
}

#[test]
fn test_model_visibility_list_hidden() {
    let harness = TestHarness::setup();

    harness.run_cli(&["models", "visibility", "--hide", "gpt-4o"]);

    let list_output = harness.run_cli(&["models", "visibility", "--list-hidden"]);
    assert!(list_output.status.success());

    let stdout = String::from_utf8_lossy(&list_output.stdout);
    assert!(stdout.contains("gpt-4o"));
}

#[test]
fn test_model_visibility_show_restores() {
    let harness = TestHarness::setup();

    harness.run_cli(&["models", "visibility", "--hide", "gpt-4o"]);

    let show_output = harness.run_cli(&["models", "visibility", "--show", "gpt-4o"]);
    assert!(show_output.status.success());

    let json = harness.run_cli_json(&["models", "--visibility", "visible"]);
    let models = json["models"].as_array().unwrap_or(&EMPTY_VEC);
    let has_model_1 = models.iter().any(|m| m["id"] == "gpt-4o");
    assert!(has_model_1, "Shown model should appear in visible list");
}
