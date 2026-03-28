use crate::common::TestHarness;

mod common;

#[test]
fn test_prompt_multiline_input() {
    let harness = TestHarness::setup();
    let session_id = harness.create_session("multiline-test");

    let multiline_content = "Line 1\nLine 2\nLine 3";
    let output = harness.run_cli(&[
        "prompt",
        "--session",
        &session_id,
        "--content",
        multiline_content,
    ]);
    assert!(output.status.success());

    let messages = harness.get_session_messages(&session_id);
    assert!(!messages.is_empty());
}

#[test]
fn test_prompt_multiline_with_code() {
    let harness = TestHarness::setup();
    let session_id = harness.create_session("multiline-code-test");

    let code_content = "```rust\nfn main() {\n    println!(\"Hello\");\n}\n```";
    let output = harness.run_cli(&[
        "prompt",
        "--session",
        &session_id,
        "--content",
        code_content,
    ]);
    assert!(output.status.success());
}

#[test]
fn test_prompt_multiline_submit() {
    let harness = TestHarness::setup();
    let session_id = harness.create_session("multiline-submit-test");

    let output = harness.run_cli(&[
        "prompt",
        "--session",
        &session_id,
        "--multiline",
        "--content",
        "First paragraph\n\nSecond paragraph",
    ]);
    assert!(output.status.success());
}
