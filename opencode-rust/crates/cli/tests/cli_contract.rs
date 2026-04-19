mod common;

use common::TestHarness;

#[test]
fn cli_contract_help_exits_with_code_zero() {
    let harness = TestHarness::setup();
    let output = harness.run_cli(&["--help"]);

    assert_eq!(
        output.status.code(),
        Some(0),
        "--help should exit with code 0, got {:?}",
        output.status.code()
    );
}

#[test]
fn cli_contract_help_output_goes_to_stdout_not_stderr() {
    let harness = TestHarness::setup();
    let output = harness.run_cli(&["--help"]);

    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);

    assert!(
        !stdout.is_empty(),
        "--help output should go to stdout, but stdout was empty"
    );

    assert!(
        stderr.is_empty(),
        "--help output should go to stdout, not stderr. stderr contained: {}",
        stderr
    );

    assert!(
        stdout.contains("AI coding agent"),
        "stdout should contain help content"
    );
}

#[test]
fn cli_contract_help_output_matches_expected_format() {
    let harness = TestHarness::setup();
    let output = harness.run_cli(&["--help"]);

    let stdout = String::from_utf8_lossy(&output.stdout);

    assert!(
        stdout.contains("Usage:"),
        "Help output should contain 'Usage:'"
    );
    assert!(
        stdout.contains("Commands:"),
        "Help output should contain 'Commands:'"
    );
    assert!(
        stdout.contains("Options:"),
        "Help output should contain 'Options:'"
    );
    assert!(
        stdout.contains("-h, --help"),
        "Help output should contain '-h, --help'"
    );
}

#[test]
fn cli_contract_help_contains_required_sections() {
    let harness = TestHarness::setup();
    let output = harness.run_cli(&["--help"]);

    let stdout = String::from_utf8_lossy(&output.stdout);

    let required_content = ["Usage:", "Commands:", "Options:", "opencode-rs"];

    for content in required_content {
        assert!(
            stdout.contains(content),
            "Help output should contain '{}', got:\n{}",
            content,
            stdout
        );
    }
}

#[test]
fn cli_contract_help_shows_tui_as_default_command() {
    let harness = TestHarness::setup();
    let output = harness.run_cli(&["--help"]);

    let stdout = String::from_utf8_lossy(&output.stdout);

    assert!(
        stdout.contains("tui"),
        "Help output should mention 'tui' as default command"
    );
}
