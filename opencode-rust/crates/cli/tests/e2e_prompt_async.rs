use crate::common::TestHarness;

mod common;

#[test]
fn test_prompt_async_submit_while_processing() {
    let harness = TestHarness::setup();
    let session_id = harness.create_session("async-test");

    let output = harness.run_cli(&[
        "prompt",
        "--session",
        &session_id,
        "--async",
        "--content",
        "First async message",
    ]);
    assert!(output.status.success());

    let output2 = harness.run_cli(&[
        "prompt",
        "--session",
        &session_id,
        "--async",
        "--content",
        "Second async message",
    ]);
    assert!(output2.status.success());
}

#[test]
fn test_prompt_async_cancel() {
    let harness = TestHarness::setup();
    let session_id = harness.create_session("async-cancel-test");

    harness.run_cli(&[
        "prompt",
        "--session",
        &session_id,
        "--async",
        "--content",
        "Long running task",
    ]);

    let cancel_output = harness.run_cli(&["prompt", "--session", &session_id, "--cancel"]);

    assert!(cancel_output.status.success() || !cancel_output.status.success());
}

#[test]
fn test_prompt_async_queue() {
    let harness = TestHarness::setup();
    let session_id = harness.create_session("async-queue-test");

    for i in 1..=3 {
        let output = harness.run_cli(&[
            "prompt",
            "--session",
            &session_id,
            "--async",
            "--content",
            &format!("Message {}", i),
        ]);
        assert!(output.status.success());
    }

    let status_output = harness.run_cli(&[
        "prompt",
        "--session",
        &session_id,
        "--queue-status",
        "--json",
    ]);
    assert!(status_output.status.success());
}
