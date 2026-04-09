use crate::common::{TestHarness, EMPTY_VEC};

mod common;

#[test]
fn test_project_create() {
    let harness = TestHarness::setup();

    let output = harness.run_cli(&["project", "create", "--name", "test-project"]);
    assert!(output.status.success());

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("test-project") || stdout.contains("created"));
}

#[test]
fn test_project_list() {
    let harness = TestHarness::setup();

    harness.run_cli(&["project", "create", "--name", "project-1"]);
    harness.run_cli(&["project", "create", "--name", "project-2"]);

    let output = harness.run_cli(&["project", "list", "--json"]);
    assert!(output.status.success());

    let stdout = String::from_utf8_lossy(&output.stdout);
    let json: serde_json::Value = serde_json::from_str(&stdout).unwrap_or_default();

    let projects = json.as_array().unwrap_or(&EMPTY_VEC);
    assert!(projects.len() >= 2);
}

#[test]
fn test_project_delete() {
    let harness = TestHarness::setup();

    harness.run_cli(&["project", "create", "--name", "delete-me"]);

    let output = harness.run_cli(&["project", "delete", "--name", "delete-me", "--confirm"]);
    assert!(output.status.success());

    let list_output = harness.run_cli(&["project", "list", "--json"]);
    let stdout = String::from_utf8_lossy(&list_output.stdout);
    assert!(!stdout.contains("delete-me"));
}

#[test]
fn test_project_switch() {
    let harness = TestHarness::setup();

    harness.run_cli(&["project", "create", "--name", "project-a"]);
    harness.run_cli(&["project", "create", "--name", "project-b"]);

    let output = harness.run_cli(&["project", "switch", "--name", "project-b"]);
    assert!(output.status.success());

    let current_output = harness.run_cli(&["project", "current"]);
    let stdout = String::from_utf8_lossy(&current_output.stdout);
    assert!(stdout.contains("project-b"));
}
