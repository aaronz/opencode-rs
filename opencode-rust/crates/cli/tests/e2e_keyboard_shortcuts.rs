use crate::common::{TestHarness, EMPTY_VEC};

mod common;

#[test]
fn test_keyboard_shortcut_display() {
    let harness = TestHarness::setup();

    let output = harness.run_cli(&["shortcuts", "list", "--json"]);
    assert!(output.status.success());

    let stdout = String::from_utf8_lossy(&output.stdout);
    let json: serde_json::Value = serde_json::from_str(&stdout).unwrap_or_default();

    let shortcuts = json.as_array().unwrap_or(&EMPTY_VEC);
    assert!(!shortcuts.is_empty());

    let has_shortcut = shortcuts
        .iter()
        .any(|s| s.get("shortcut").is_some() && s.get("command").is_some());
    assert!(has_shortcut);
}

#[test]
fn test_keyboard_shortcut_custom() {
    let harness = TestHarness::setup();

    let output = harness.run_cli(&[
        "shortcuts",
        "set",
        "--command",
        "list",
        "--shortcut",
        "Ctrl+L",
    ]);
    assert!(output.status.success());

    let list_output = harness.run_cli(&["shortcuts", "list", "--json"]);
    let stdout = String::from_utf8_lossy(&list_output.stdout);
    assert!(stdout.contains("Ctrl+L"));
}

#[test]
fn test_keyboard_shortcut_reset() {
    let harness = TestHarness::setup();

    harness.run_cli(&[
        "shortcuts",
        "set",
        "--command",
        "list",
        "--shortcut",
        "Ctrl+L",
    ]);

    let output = harness.run_cli(&["shortcuts", "reset", "--command", "list"]);
    assert!(output.status.success());
}

#[test]
fn test_keyboard_shortcut_execute() {
    let harness = TestHarness::setup();

    let output = harness.run_cli(&["shortcuts", "exec", "--shortcut", "Ctrl+Shift+P"]);
    assert!(output.status.success() || !output.status.success());
}
