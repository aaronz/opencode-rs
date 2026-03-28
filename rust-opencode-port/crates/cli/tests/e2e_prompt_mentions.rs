use crate::common::TestHarness;

mod common;

#[test]
fn test_prompt_mention_agent() {
    let harness = TestHarness::setup();
    let session_id = harness.create_session("mention-agent-test");

    let output = harness.run_cli(&[
        "prompt",
        "--session",
        &session_id,
        "--content",
        "@build help me with this code",
    ]);
    assert!(output.status.success());

    let messages = harness.get_session_messages(&session_id);
    assert!(!messages.is_empty());
}

#[test]
fn test_prompt_mention_file() {
    let harness = TestHarness::setup();
    let session_id = harness.create_session("mention-file-test");

    harness.setup_file("test.rs", "fn main() {}");

    let output = harness.run_cli(&[
        "prompt",
        "--session",
        &session_id,
        "--content",
        "@file:test.rs please review",
    ]);
    assert!(output.status.success());
}

#[test]
fn test_prompt_mention_session() {
    let harness = TestHarness::setup();
    let other_session = harness.create_session("other-session");
    let current_session = harness.create_session("mention-session-test");

    let output = harness.run_cli(&[
        "prompt",
        "--session",
        &current_session,
        "--content",
        &format!("@session:{} reference previous work", other_session),
    ]);
    assert!(output.status.success());
}

#[test]
fn test_prompt_mention_multiple() {
    let harness = TestHarness::setup();
    let session_id = harness.create_session("mention-multi-test");

    harness.setup_file("lib.rs", "pub fn add(a: i32, b: i32) -> i32 { a + b }");

    let output = harness.run_cli(&[
        "prompt",
        "--session",
        &session_id,
        "--content",
        "@build @file:lib.rs optimize this function",
    ]);
    assert!(output.status.success());
}
