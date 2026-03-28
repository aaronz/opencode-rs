use crate::common::TestHarness;
use serde_json::json;

#[test]
fn test_session_create_new() {
    let harness = TestHarness::setup();
    let result = harness.run_cli_json(&["session", "--new"]);

    assert!(result.get("id").is_some(), "Session should have an ID");
    assert!(
        result.get("created_at").is_some(),
        "Session should have created_at"
    );
    assert_eq!(
        result["messages"].as_array().unwrap().len(),
        0,
        "New session should have no messages"
    );
}

#[test]
fn test_session_list() {
    let harness = TestHarness::setup();
    let _session1 = harness.run_cli_json(&["session", "--new"]);
    let _session2 = harness.run_cli_json(&["session", "--new"]);
    let result = harness.run_cli_json(&["list", "--json"]);

    assert!(
        result.get("sessions").is_some(),
        "Result should contain sessions array"
    );
    let sessions = result["sessions"].as_array().unwrap();
    assert!(sessions.len() >= 2, "Should have at least 2 sessions");

    for session in sessions {
        assert!(session.get("id").is_some(), "Session should have ID");
        assert!(
            session.get("created_at").is_some(),
            "Session should have created_at"
        );
    }
}

#[test]
fn test_session_resume() {
    let harness = TestHarness::setup();
    let create_result = harness.run_cli_json(&["session", "--new"]);
    let session_id = create_result["id"].as_str().unwrap();
    let _ = harness.run_cli_json(&["session", "--id", session_id, "--message", "Hello"]);
    let resume_result = harness.run_cli_json(&["session", "--id", session_id]);

    assert_eq!(
        resume_result["id"], session_id,
        "Should resume correct session"
    );
    assert!(
        resume_result["messages"].as_array().unwrap().len() > 0,
        "Should have messages"
    );
}

#[test]
fn test_session_persistence() {
    let harness = TestHarness::setup();
    let create_result = harness.run_cli_json(&["session", "--new"]);
    let session_id = create_result["id"].as_str().unwrap();
    let _ = harness.run_cli_json(&["session", "--id", session_id, "--message", "Message 1"]);
    let _ = harness.run_cli_json(&["session", "--id", session_id, "--message", "Message 2"]);
    let list_result = harness.run_cli_json(&["list", "--json"]);
    let sessions = list_result["sessions"].as_array().unwrap();
    let found_session = sessions.iter().find(|s| s["id"] == session_id);
    assert!(found_session.is_some(), "Session should persist in list");
}

#[test]
fn test_session_fork() {
    let harness = TestHarness::setup();
    let create_result = harness.run_cli_json(&["session", "--new"]);
    let session_id = create_result["id"].as_str().unwrap();
    let _ = harness.run_cli_json(&["session", "--id", session_id, "--message", "First"]);
    let _ = harness.run_cli_json(&["session", "--id", session_id, "--message", "Second"]);
    let fork_result = harness.run_cli_json(&["session", "--id", session_id, "--fork"]);

    assert!(
        fork_result.get("new_id").is_some(),
        "Fork should create new session ID"
    );
    assert_ne!(
        fork_result["new_id"], session_id,
        "Fork should have different ID"
    );
}

#[test]
fn test_session_share() {
    let harness = TestHarness::setup();
    let create_result = harness.run_cli_json(&["session", "--new"]);
    let session_id = create_result["id"].as_str().unwrap();
    let share_result = harness.run_cli_json(&["session", "--id", session_id, "--share"]);

    assert!(
        share_result.get("share_url").is_some(),
        "Should return share URL"
    );
    assert!(
        share_result["share_url"].as_str().unwrap().contains("http"),
        "Share URL should be valid"
    );
}
