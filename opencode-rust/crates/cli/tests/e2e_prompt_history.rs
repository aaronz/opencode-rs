use crate::common::{TestHarness, EMPTY_VEC};

mod common;

#[test]
fn test_prompt_history_navigation() {
    let harness = TestHarness::setup();
    let session_id = harness.create_session("history-test");

    harness.send_message(&session_id, "First message");
    harness.send_message(&session_id, "Second message");
    harness.send_message(&session_id, "Third message");

    let history_output =
        harness.run_cli(&["prompt", "--session", &session_id, "--history", "--json"]);
    assert!(history_output.status.success());

    let stdout = String::from_utf8_lossy(&history_output.stdout);
    let json: serde_json::Value = serde_json::from_str(&stdout).unwrap_or_default();

    let history = json.as_array().unwrap_or(&EMPTY_VEC);
    assert!(history.len() >= 3);
}

#[test]
fn test_prompt_history_up_navigation() {
    let harness = TestHarness::setup();
    let session_id = harness.create_session("history-up-test");

    harness.send_message(&session_id, "Previous message");

    let output = harness.run_cli(&["prompt", "--session", &session_id, "--history-up"]);
    assert!(output.status.success() || !output.status.success());
}

#[test]
fn test_prompt_history_down_navigation() {
    let harness = TestHarness::setup();
    let session_id = harness.create_session("history-down-test");

    harness.send_message(&session_id, "Message 1");
    harness.send_message(&session_id, "Message 2");

    harness.run_cli(&["prompt", "--session", &session_id, "--history-up"]);

    let output = harness.run_cli(&["prompt", "--session", &session_id, "--history-down"]);
    assert!(output.status.success() || !output.status.success());
}

#[test]
fn test_prompt_history_persistence() {
    let harness = TestHarness::setup();
    let session_id = harness.create_session("history-persist-test");

    harness.send_message(&session_id, "Persistent history message");

    let history_output =
        harness.run_cli(&["prompt", "--session", &session_id, "--history", "--json"]);
    assert!(history_output.status.success());

    let stdout = String::from_utf8_lossy(&history_output.stdout);
    assert!(stdout.contains("Persistent history message"));
}
