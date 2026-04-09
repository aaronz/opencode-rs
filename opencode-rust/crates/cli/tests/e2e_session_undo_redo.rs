use crate::common::{TestHarness, EMPTY_VEC};
use std::time::Duration;

mod common;

#[test]
fn test_session_undo_single_message() {
    let harness = TestHarness::setup();
    let session_id = harness.create_session("undo-test");

    harness.send_message(&session_id, "First message");
    harness.send_message(&session_id, "Second message");

    let messages_before = harness.get_session_messages(&session_id);
    assert_eq!(messages_before.len(), 2);

    let output = harness.run_cli(&["session", "undo", "--id", &session_id, "--steps", "1"]);
    assert!(output.status.success());

    let messages_after = harness.get_session_messages(&session_id);
    assert_eq!(messages_after.len(), 1);
    assert_eq!(messages_after[0]["content"], "First message");
}

#[test]
fn test_session_undo_multiple_messages() {
    let harness = TestHarness::setup();
    let session_id = harness.create_session("undo-multi-test");

    harness.send_message(&session_id, "Message 1");
    harness.send_message(&session_id, "Message 2");
    harness.send_message(&session_id, "Message 3");

    let output = harness.run_cli(&["session", "undo", "--id", &session_id, "--steps", "2"]);
    assert!(output.status.success());

    let messages = harness.get_session_messages(&session_id);
    assert_eq!(messages.len(), 1);
    assert_eq!(messages[0]["content"], "Message 1");
}

#[test]
fn test_session_undo_nothing() {
    let harness = TestHarness::setup();
    let session_id = harness.create_session("undo-nothing-test");

    let output = harness.run_cli(&["session", "undo", "--id", &session_id]);
    assert!(!output.status.success());

    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("Nothing to undo") || stderr.contains("Error"));
}

#[test]
fn test_session_redo_after_undo() {
    let harness = TestHarness::setup();
    let session_id = harness.create_session("redo-test");

    harness.send_message(&session_id, "Original message");

    harness.run_cli(&["session", "undo", "--id", &session_id]);

    let messages_after_undo = harness.get_session_messages(&session_id);
    assert!(messages_after_undo.is_empty());

    let output = harness.run_cli(&["session", "redo", "--id", &session_id]);
    assert!(output.status.success());

    let messages_after_redo = harness.get_session_messages(&session_id);
    assert_eq!(messages_after_redo.len(), 1);
    assert_eq!(messages_after_redo[0]["content"], "Original message");
}

#[test]
fn test_session_redo_nothing() {
    let harness = TestHarness::setup();
    let session_id = harness.create_session("redo-nothing-test");

    let output = harness.run_cli(&["session", "redo", "--id", &session_id]);
    assert!(!output.status.success());

    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("Nothing to redo") || stderr.contains("Error"));
}

#[test]
fn test_session_undo_redo_clears_redo_history() {
    let harness = TestHarness::setup();
    let session_id = harness.create_session("undo-clears-redo-test");

    harness.send_message(&session_id, "Message 1");
    harness.send_message(&session_id, "Message 2");

    harness.run_cli(&["session", "undo", "--id", &session_id]);

    harness.send_message(&session_id, "New message after undo");

    let output = harness.run_cli(&["session", "redo", "--id", &session_id]);
    assert!(!output.status.success());
}

#[test]
fn test_session_undo_persistence() {
    let harness = TestHarness::setup();
    let session_id = harness.create_session("undo-persistence-test");

    harness.send_message(&session_id, "Test message");
    harness.run_cli(&["session", "undo", "--id", &session_id]);

    let output = harness.run_cli(&["session", "show", "--id", &session_id, "--json"]);
    assert!(output.status.success());

    let stdout = String::from_utf8_lossy(&output.stdout);
    let json: serde_json::Value = serde_json::from_str(&stdout).unwrap_or_default();

    assert!(json
        .get("redo_history")
        .map(|h| !h.as_array().unwrap_or(&EMPTY_VEC).is_empty())
        .unwrap_or(false));
}
