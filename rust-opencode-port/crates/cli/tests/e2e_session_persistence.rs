use crate::common::TestHarness;

mod common;

#[test]
fn test_session_persistence_after_reload() {
    let harness = TestHarness::setup();
    let session_id = harness.create_session("persistence-test");

    harness.send_message(&session_id, "Persistent message");

    let output = harness.run_cli(&["session", "show", "--id", &session_id, "--json"]);
    assert!(output.status.success());

    let stdout = String::from_utf8_lossy(&output.stdout);
    let json: serde_json::Value = serde_json::from_str(&stdout).unwrap_or_default();

    assert_eq!(json["messages"].as_array().unwrap().len(), 1);
    assert_eq!(json["messages"][0]["content"], "Persistent message");
}

#[test]
fn test_session_list_shows_all_sessions() {
    let harness = TestHarness::setup();

    let session1 = harness.create_session("session-1");
    let session2 = harness.create_session("session-2");

    let output = harness.run_cli(&["list", "--json"]);
    assert!(output.status.success());

    let stdout = String::from_utf8_lossy(&output.stdout);
    let json: serde_json::Value = serde_json::from_str(&stdout).unwrap_or_default();

    let sessions = json.as_array().expect("Expected array of sessions");
    assert!(sessions.len() >= 2);
}

#[test]
fn test_session_delete_removes_session() {
    let harness = TestHarness::setup();
    let session_id = harness.create_session("delete-test");

    let output = harness.run_cli(&["session", "delete", "--id", &session_id]);
    assert!(output.status.success());

    let show_output = harness.run_cli(&["session", "show", "--id", &session_id]);
    assert!(!show_output.status.success());
}

#[test]
fn test_session_resume() {
    let harness = TestHarness::setup();
    let session_id = harness.create_session("resume-test");

    harness.send_message(&session_id, "Message before resume");

    let output = harness.run_cli(&["session", "show", "--id", &session_id, "--json"]);
    assert!(output.status.success());

    let stdout = String::from_utf8_lossy(&output.stdout);
    let json: serde_json::Value = serde_json::from_str(&stdout).unwrap_or_default();

    assert_eq!(json["id"], session_id);
    assert!(!json["messages"].as_array().unwrap().is_empty());
}
