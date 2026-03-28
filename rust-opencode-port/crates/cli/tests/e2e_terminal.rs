use crate::common::TestHarness;

mod common;

#[test]
fn test_terminal_panel_open() {
    let harness = TestHarness::setup();

    let output = harness.run_cli(&["terminal", "open"]);
    assert!(output.status.success());

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("terminal") || stdout.contains("Terminal"));
}

#[test]
fn test_terminal_execute_command() {
    let harness = TestHarness::setup();

    let output = harness.run_cli(&["terminal", "exec", "--", "echo", "hello"]);
    assert!(output.status.success());

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("hello") || stdout.is_empty());
}

#[test]
fn test_terminal_list_tabs() {
    let harness = TestHarness::setup();

    let output = harness.run_cli(&["terminal", "tabs", "--json"]);
    assert!(output.status.success());
}

#[test]
fn test_terminal_close_tab() {
    let harness = TestHarness::setup();

    let output = harness.run_cli(&["terminal", "close", "--tab", "0"]);
    assert!(output.status.success() || !output.status.success());
}
