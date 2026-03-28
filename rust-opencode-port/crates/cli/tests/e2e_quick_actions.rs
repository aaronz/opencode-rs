use crate::common::TestHarness;

mod common;

#[test]
fn test_quick_action_new_session() {
    let harness = TestHarness::setup();

    let output = harness.run_cli(&["quick", "new-session", "--name", "quick-session"]);
    assert!(output.status.success());

    let list_output = harness.run_cli(&["list", "--json"]);
    let stdout = String::from_utf8_lossy(&list_output.stdout);
    assert!(stdout.contains("quick-session"));
}

#[test]
fn test_quick_action_switch_model() {
    let harness = TestHarness::setup();

    let output = harness.run_cli(&["quick", "switch-model", "--model", "gpt-4"]);
    assert!(output.status.success());
}

#[test]
fn test_quick_action_open_settings() {
    let harness = TestHarness::setup();

    let output = harness.run_cli(&["quick", "settings"]);
    assert!(output.status.success());
}

#[test]
fn test_quick_action_toggle_sidebar() {
    let harness = TestHarness::setup();

    let output = harness.run_cli(&["quick", "toggle-sidebar"]);
    assert!(output.status.success());
}
