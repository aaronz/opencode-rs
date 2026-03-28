use crate::common::TestHarness;

mod common;

#[test]
fn test_project_rename() {
    let harness = TestHarness::setup();

    harness.run_cli(&["project", "create", "--name", "old-name"]);

    let output = harness.run_cli(&[
        "project", "rename", "--from", "old-name", "--to", "new-name",
    ]);
    assert!(output.status.success());

    let list_output = harness.run_cli(&["project", "list", "--json"]);
    let stdout = String::from_utf8_lossy(&list_output.stdout);
    assert!(stdout.contains("new-name"));
    assert!(!stdout.contains("old-name"));
}

#[test]
fn test_project_edit_description() {
    let harness = TestHarness::setup();

    harness.run_cli(&["project", "create", "--name", "desc-project"]);

    let output = harness.run_cli(&[
        "project",
        "edit",
        "--name",
        "desc-project",
        "--description",
        "Updated description",
    ]);
    assert!(output.status.success());

    let show_output = harness.run_cli(&["project", "show", "--name", "desc-project", "--json"]);
    let stdout = String::from_utf8_lossy(&show_output.stdout);
    assert!(stdout.contains("Updated description"));
}

#[test]
fn test_project_edit_settings() {
    let harness = TestHarness::setup();

    harness.run_cli(&["project", "create", "--name", "settings-project"]);

    let output = harness.run_cli(&[
        "project",
        "edit",
        "--name",
        "settings-project",
        "--model",
        "gpt-4",
    ]);
    assert!(output.status.success());
}
