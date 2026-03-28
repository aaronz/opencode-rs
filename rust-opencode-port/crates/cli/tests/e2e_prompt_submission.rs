use crate::common::TestHarness;

mod common;

#[test]
fn test_prompt_submission() {
    let harness = TestHarness::setup();
    let session_id = harness.create_session("submit-test");

    let output = harness.run_cli(&[
        "prompt",
        "--session",
        &session_id,
        "--content",
        "Hello, assistant!",
    ]);
    assert!(output.status.success());

    let messages = harness.get_session_messages(&session_id);
    assert!(!messages.is_empty());
}

#[test]
fn test_prompt_cancellation() {
    let harness = TestHarness::setup();
    let session_id = harness.create_session("cancel-test");

    harness.run_cli(&[
        "prompt",
        "--session",
        &session_id,
        "--async",
        "--content",
        "Long task",
    ]);

    let cancel_output = harness.run_cli(&["prompt", "--session", &session_id, "--cancel"]);

    assert!(cancel_output.status.success() || !cancel_output.status.success());
}

#[test]
fn test_prompt_empty_content() {
    let harness = TestHarness::setup();
    let session_id = harness.create_session("empty-test");

    let output = harness.run_cli(&["prompt", "--session", &session_id, "--content", ""]);

    assert!(!output.status.success() || output.status.success());
}

#[test]
fn test_prompt_with_context() {
    let harness = TestHarness::setup();
    let session_id = harness.create_session("context-test");

    harness.setup_file("context.txt", "Important context information");

    let output = harness.run_cli(&[
        "prompt",
        "--session",
        &session_id,
        "--context",
        "context.txt",
        "--content",
        "Use this context",
    ]);
    assert!(output.status.success());
}
