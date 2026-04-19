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

#[test]
fn cli_contract_version_exits_with_code_zero() {
    let harness = TestHarness::setup();
    let output = harness.run_cli(&["--version"]);

    assert_eq!(
        output.status.code(),
        Some(0),
        "--version should exit with code 0, got {:?}",
        output.status.code()
    );
}

#[test]
fn cli_contract_version_output_goes_to_stdout_not_stderr() {
    let harness = TestHarness::setup();
    let output = harness.run_cli(&["--version"]);

    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);

    assert!(
        !stdout.is_empty(),
        "--version output should go to stdout, but stdout was empty"
    );

    assert!(
        stderr.is_empty(),
        "--version output should go to stdout, not stderr. stderr contained: {}",
        stderr
    );
}

#[test]
fn cli_contract_version_format_matches_expected_pattern() {
    let harness = TestHarness::setup();
    let output = harness.run_cli(&["--version"]);

    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);

    assert!(
        stderr.is_empty(),
        "--version should not output to stderr. stderr contained: {}",
        stderr
    );

    let version_regex = regex::Regex::new(r"^opencode-rs \d+\.\d+\.\d+$").unwrap();
    let trimmed = stdout.trim();
    assert!(
        version_regex.is_match(trimmed),
        "Version output should match pattern 'opencode-rs X.Y.Z', got: '{}'",
        trimmed
    );
}

#[test]
fn cli_contract_version_is_stable_across_runs() {
    let harness = TestHarness::setup();

    let output1 = harness.run_cli(&["--version"]);
    let output2 = harness.run_cli(&["--version"]);
    let output3 = harness.run_cli(&["--version"]);

    let stdout1 = String::from_utf8_lossy(&output1.stdout).trim().to_string();
    let stdout2 = String::from_utf8_lossy(&output2.stdout).trim().to_string();
    let stdout3 = String::from_utf8_lossy(&output3.stdout).trim().to_string();

    assert_eq!(
        stdout1, stdout2,
        "Version output should be stable across runs. First: '{}', Second: '{}'",
        stdout1, stdout2
    );

    assert_eq!(
        stdout2, stdout3,
        "Version output should be stable across runs. Second: '{}', Third: '{}'",
        stdout2, stdout3
    );
}

#[test]
fn cli_contract_version_contains_three_version_components() {
    let harness = TestHarness::setup();
    let output = harness.run_cli(&["--version"]);

    let stdout = String::from_utf8_lossy(&output.stdout);
    let trimmed = stdout.trim();

    let parts: Vec<&str> = trimmed.split_whitespace().collect();
    assert_eq!(parts.len(), 2, "Version output should have 2 parts (name and version)");

    let version_parts: Vec<&str> = parts[1].split('.').collect();
    assert_eq!(
        version_parts.len(),
        3,
        "Version should have 3 components (X.Y.Z), got: '{}'",
        parts[1]
    );

    for (i, part) in version_parts.iter().enumerate() {
        assert!(
            part.parse::<u32>().is_ok(),
            "Version component {} should be a number, got: '{}'",
            i,
            part
        );
    }
}

#[test]
fn cli_contract_workspace_help_exits_with_code_zero() {
    let harness = TestHarness::setup();
    let output = harness.run_cli(&["workspace", "--help"]);

    assert_eq!(
        output.status.code(),
        Some(0),
        "workspace --help should exit with code 0, got {:?}",
        output.status.code()
    );
}

#[test]
fn cli_contract_workspace_help_output_goes_to_stdout_not_stderr() {
    let harness = TestHarness::setup();
    let output = harness.run_cli(&["workspace", "--help"]);

    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);

    assert!(
        !stdout.is_empty(),
        "workspace --help output should go to stdout, but stdout was empty"
    );

    assert!(
        stderr.is_empty(),
        "workspace --help output should go to stdout, not stderr. stderr contained: {}",
        stderr
    );
}

#[test]
fn cli_contract_workspace_help_output_matches_expected_format() {
    let harness = TestHarness::setup();
    let output = harness.run_cli(&["workspace", "--help"]);

    let stdout = String::from_utf8_lossy(&output.stdout);

    assert!(
        stdout.contains("Usage:"),
        "workspace --help output should contain 'Usage:'"
    );
    assert!(
        stdout.contains("Commands:"),
        "workspace --help output should contain 'Commands:'"
    );
    assert!(
        stdout.contains("Options:"),
        "workspace --help output should contain 'Options:'"
    );
    assert!(
        stdout.contains("-h, --help"),
        "workspace --help output should contain '-h, --help'"
    );
}

#[test]
fn cli_contract_workspace_help_shows_workspace_as_title() {
    let harness = TestHarness::setup();
    let output = harness.run_cli(&["workspace", "--help"]);

    let stdout = String::from_utf8_lossy(&output.stdout);

    assert!(
        stdout.contains("workspace"),
        "workspace --help output should mention 'workspace'"
    );
}

#[test]
fn cli_contract_workspace_help_shows_available_subcommands() {
    let harness = TestHarness::setup();
    let output = harness.run_cli(&["workspace", "--help"]);

    let stdout = String::from_utf8_lossy(&output.stdout);

    assert!(
        stdout.contains("sessions"),
        "workspace --help output should contain 'sessions' subcommand"
    );
    assert!(
        stdout.contains("status"),
        "workspace --help output should contain 'status' subcommand"
    );
    assert!(
        stdout.contains("context"),
        "workspace --help output should contain 'context' subcommand"
    );
}

#[test]
fn cli_contract_workspace_sessions_help_exits_with_code_zero() {
    let harness = TestHarness::setup();
    let output = harness.run_cli(&["workspace", "sessions", "--help"]);

    assert_eq!(
        output.status.code(),
        Some(0),
        "workspace sessions --help should exit with code 0, got {:?}",
        output.status.code()
    );
}

#[test]
fn cli_contract_workspace_sessions_help_output_goes_to_stdout() {
    let harness = TestHarness::setup();
    let output = harness.run_cli(&["workspace", "sessions", "--help"]);

    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);

    assert!(
        !stdout.is_empty(),
        "workspace sessions --help output should go to stdout, but stdout was empty"
    );

    assert!(
        stderr.is_empty(),
        "workspace sessions --help output should go to stdout, not stderr. stderr contained: {}",
        stderr
    );
}
