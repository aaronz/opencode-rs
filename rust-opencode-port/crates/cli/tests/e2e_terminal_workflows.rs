use crate::common::TestHarness;
use std::time::Duration;

#[test]
fn test_terminal_execute() {
    let harness = TestHarness::setup();
    let result = harness.run_cli_json(&["bash", "--command", "echo 'Hello World'", "--json"]);

    assert!(result.get("stdout").is_some(), "Should have stdout");
    assert!(result["stdout"].as_str().unwrap().contains("Hello World"));
    assert_eq!(result["exit_code"], 0, "Should exit successfully");
}

#[test]
fn test_terminal_output() {
    let harness = TestHarness::setup();
    let result = harness.run_cli_json(&[
        "bash",
        "--command",
        "echo 'Line 1' && echo 'Line 2'",
        "--json",
    ]);

    let stdout = result["stdout"].as_str().unwrap();
    assert!(stdout.contains("Line 1"), "Should contain first line");
    assert!(stdout.contains("Line 2"), "Should contain second line");
}

#[test]
fn test_terminal_timeout() {
    let harness = TestHarness::setup();
    let result = harness.run_cli(&["bash", "--command", "sleep 60", "--timeout", "1"]);

    assert!(!result.status.success(), "Should timeout");
    let stderr = String::from_utf8_lossy(&result.stderr);
    assert!(
        stderr.contains("timeout") || stderr.contains("Timeout"),
        "Should indicate timeout"
    );
}

#[test]
fn test_terminal_interactive_detection() {
    let harness = TestHarness::setup();
    let result = harness.run_cli(&["bash", "--command", "read -p 'Input: ' var"]);

    let stderr = String::from_utf8_lossy(&result.stderr);
    assert!(
        stderr.contains("interactive") || !result.status.success(),
        "Should detect or fail on interactive command"
    );
}
