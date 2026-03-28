use crate::common::TestHarness;

mod common;

#[test]
fn test_workspace_new_session() {
    let harness = TestHarness::setup();

    harness.run_cli(&["project", "create", "--name", "workspace-test"]);
    harness.run_cli(&["project", "switch", "--name", "workspace-test"]);

    let session_id = harness.create_session("workspace-session");

    let output = harness.run_cli(&["workspace", "sessions", "--json"]);
    assert!(output.status.success());

    let stdout = String::from_utf8_lossy(&output.stdout);
    let json: serde_json::Value = serde_json::from_str(&stdout).unwrap_or_default();

    let sessions = json.as_array().unwrap_or(&vec![]);
    assert!(sessions
        .iter()
        .any(|s| s["id"].as_str().unwrap_or("").contains(&session_id)));
}

#[test]
fn test_workspace_persistence() {
    let harness = TestHarness::setup();

    harness.run_cli(&["project", "create", "--name", "persist-workspace"]);
    harness.run_cli(&["project", "switch", "--name", "persist-workspace"]);

    let session_id = harness.create_session("persist-session");
    harness.send_message(&session_id, "Workspace message");

    let output = harness.run_cli(&["workspace", "status", "--json"]);
    assert!(output.status.success());

    let stdout = String::from_utf8_lossy(&output.stdout);
    let json: serde_json::Value = serde_json::from_str(&stdout).unwrap_or_default();

    assert_eq!(json["project"], "persist-workspace");
}

#[test]
fn test_workspace_context() {
    let harness = TestHarness::setup();

    let project_path = harness.setup_project("context-project");
    harness.run_cli(&[
        "project",
        "create",
        "--name",
        "context-project",
        "--path",
        project_path.to_str().unwrap(),
    ]);

    let output = harness.run_cli(&["workspace", "context", "--json"]);
    assert!(output.status.success());

    let stdout = String::from_utf8_lossy(&output.stdout);
    let json: serde_json::Value = serde_json::from_str(&stdout).unwrap_or_default();

    assert!(json.get("files").is_some() || json.get("context").is_some());
}
