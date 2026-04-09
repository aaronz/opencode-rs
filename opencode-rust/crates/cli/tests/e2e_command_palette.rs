use crate::common::{TestHarness, EMPTY_VEC};

mod common;

#[test]
fn test_command_palette_open() {
    let harness = TestHarness::setup();

    let output = harness.run_cli(&["palette", "open"]);
    assert!(output.status.success());
}

#[test]
fn test_command_palette_search() {
    let harness = TestHarness::setup();

    let output = harness.run_cli(&["palette", "search", "--query", "session", "--json"]);
    assert!(output.status.success());

    let stdout = String::from_utf8_lossy(&output.stdout);
    let json: serde_json::Value = serde_json::from_str(&stdout).unwrap_or_default();

    let commands = json.as_array().unwrap_or(&EMPTY_VEC);
    assert!(commands.iter().any(|c| c["name"]
        .as_str()
        .unwrap_or("")
        .to_lowercase()
        .contains("session")));
}

#[test]
fn test_command_palette_execute() {
    let harness = TestHarness::setup();

    let output = harness.run_cli(&["palette", "execute", "--command", "list"]);
    assert!(output.status.success());
}

#[test]
fn test_command_palette_recent() {
    let harness = TestHarness::setup();

    harness.run_cli(&["palette", "execute", "--command", "list"]);
    harness.run_cli(&["palette", "execute", "--command", "models"]);

    let output = harness.run_cli(&["palette", "recent", "--json"]);
    assert!(output.status.success());

    let stdout = String::from_utf8_lossy(&output.stdout);
    let json: serde_json::Value = serde_json::from_str(&stdout).unwrap_or_default();

    let recent = json.as_array().unwrap_or(&EMPTY_VEC);
    assert!(!recent.is_empty());
}
