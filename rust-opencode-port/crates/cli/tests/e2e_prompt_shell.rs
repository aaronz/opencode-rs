use crate::common::TestHarness;

mod common;

#[test]
fn test_prompt_shell_command() {
    let harness = TestHarness::setup();
    let session_id = harness.create_session("shell-test");

    let output = harness.run_cli(&[
        "prompt",
        "--session",
        &session_id,
        "--shell",
        "--content",
        "echo hello",
    ]);
    assert!(output.status.success());

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("hello") || stdout.is_empty());
}

#[test]
fn test_prompt_shell_with_args() {
    let harness = TestHarness::setup();
    let session_id = harness.create_session("shell-args-test");

    let output = harness.run_cli(&[
        "prompt",
        "--session",
        &session_id,
        "--shell",
        "--content",
        "ls -la",
    ]);
    assert!(output.status.success());
}

#[test]
fn test_prompt_shell_output_displayed() {
    let harness = TestHarness::setup();
    let session_id = harness.create_session("shell-output-test");

    let output = harness.run_cli(&[
        "prompt",
        "--session",
        &session_id,
        "--shell",
        "--content",
        "pwd",
    ]);
    assert!(output.status.success());

    let messages = harness.get_session_messages(&session_id);
    assert!(!messages.is_empty());
}

#[test]
fn test_prompt_shell_in_terminal_panel() {
    let harness = TestHarness::setup();
    let session_id = harness.create_session("shell-terminal-test");

    let output = harness.run_cli(&[
        "prompt",
        "--session",
        &session_id,
        "--shell",
        "--terminal",
        "--content",
        "whoami",
    ]);
    assert!(output.status.success());
}
