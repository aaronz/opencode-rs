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
    assert_eq!(
        parts.len(),
        2,
        "Version output should have 2 parts (name and version)"
    );

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

#[test]
fn cli_contract_invalid_option_exits_with_nonzero_code() {
    let harness = TestHarness::setup();
    let output = harness.run_cli(&["--invalid-option"]);

    let exit_code = output.status.code();
    assert!(
        exit_code == Some(1) || exit_code == Some(2),
        "Invalid option should exit with code 1 or 2, got {:?}",
        exit_code
    );
}

#[test]
fn cli_contract_invalid_option_error_message_format() {
    let harness = TestHarness::setup();
    let output = harness.run_cli(&["--invalid-option"]);

    let stderr = String::from_utf8_lossy(&output.stderr);
    let stdout = String::from_utf8_lossy(&output.stdout);

    assert!(
        stderr.contains("unexpected argument") || stdout.contains("unexpected argument"),
        "Error message should contain 'unexpected argument', got stderr: '{}', stdout: '{}'",
        stderr,
        stdout
    );
}

#[test]
fn cli_contract_invalid_option_shows_usage_tip() {
    let harness = TestHarness::setup();
    let output = harness.run_cli(&["--invalid-option"]);

    let stderr = String::from_utf8_lossy(&output.stderr);
    let stdout = String::from_utf8_lossy(&output.stdout);

    assert!(
        stderr.contains("tip:") || stdout.contains("tip:"),
        "Error message should contain 'tip:', got stderr: '{}', stdout: '{}'",
        stderr,
        stdout
    );
}

#[test]
fn cli_contract_invalid_option_shows_help_suggestion() {
    let harness = TestHarness::setup();
    let output = harness.run_cli(&["--invalid-option"]);

    let stderr = String::from_utf8_lossy(&output.stderr);
    let stdout = String::from_utf8_lossy(&output.stdout);

    assert!(
        stderr.contains("--help") || stdout.contains("--help"),
        "Error message should suggest --help, got stderr: '{}', stdout: '{}'",
        stderr,
        stdout
    );
}

#[test]
fn cli_contract_invalid_short_option_exits_with_nonzero_code() {
    let harness = TestHarness::setup();
    let output = harness.run_cli(&["-z"]);

    let exit_code = output.status.code();
    assert!(
        exit_code == Some(1) || exit_code == Some(2),
        "Invalid short option should exit with code 1 or 2, got {:?}",
        exit_code
    );
}

#[test]
fn cli_contract_invalid_option_various_patterns() {
    let harness = TestHarness::setup();

    let invalid_options = vec![
        "--unknown-flag",
        "--invalid",
        "-x",
        "-9",
        "--foo",
        "--bar-baz",
    ];

    for option in invalid_options {
        let output = harness.run_cli(&[option]);
        let exit_code = output.status.code();
        assert!(
            exit_code == Some(1) || exit_code == Some(2),
            "Option '{}' should exit with code 1 or 2, got {:?}",
            option,
            exit_code
        );
    }
}

#[test]
fn cli_contract_invalid_option_does_not_panic() {
    let harness = TestHarness::setup();
    let output = harness.run_cli(&["--invalid-option"]);

    assert!(
        output.status.success()
            || output.status.code() == Some(1)
            || output.status.code() == Some(2),
        "Invalid option should not cause a panic/signal, got status: {:?}",
        output.status
    );
}

#[test]
fn cli_contract_config_show_exits_with_code_zero() {
    let harness = TestHarness::setup();
    let output = harness.run_cli(&["config", "show"]);

    assert_eq!(
        output.status.code(),
        Some(0),
        "config show should exit with code 0, got {:?}",
        output.status.code()
    );
}

#[test]
fn cli_contract_config_show_output_goes_to_stdout_not_stderr() {
    let harness = TestHarness::setup();
    let output = harness.run_cli(&["config", "show"]);

    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);

    assert!(
        !stdout.is_empty(),
        "config show output should go to stdout, but stdout was empty"
    );

    assert!(
        stderr.is_empty(),
        "config show output should go to stdout, not stderr. stderr contained: {}",
        stderr
    );
}

#[test]
fn cli_contract_config_show_output_format_is_stable() {
    let harness = TestHarness::setup();

    let output1 = harness.run_cli(&["config", "show"]);
    let output2 = harness.run_cli(&["config", "show"]);
    let output3 = harness.run_cli(&["config", "show"]);

    let stdout1 = String::from_utf8_lossy(&output1.stdout).trim().to_string();
    let stdout2 = String::from_utf8_lossy(&output2.stdout).trim().to_string();
    let stdout3 = String::from_utf8_lossy(&output3.stdout).trim().to_string();

    assert_eq!(
        stdout1, stdout2,
        "config show output should be stable across runs. First: '{}', Second: '{}'",
        stdout1, stdout2
    );

    assert_eq!(
        stdout2, stdout3,
        "config show output should be stable across runs. Second: '{}', Third: '{}'",
        stdout2, stdout3
    );
}

#[test]
fn cli_contract_config_show_contains_config_path() {
    let harness = TestHarness::setup();
    let output = harness.run_cli(&["config", "show"]);

    let stdout = String::from_utf8_lossy(&output.stdout);

    assert!(
        stdout.contains("Config path:"),
        "config show output should contain 'Config path:', got: '{}'",
        stdout
    );
}

#[test]
fn cli_contract_config_show_output_format_lines_consistent() {
    let harness = TestHarness::setup();
    let output = harness.run_cli(&["config", "show"]);

    let stdout = String::from_utf8_lossy(&output.stdout);
    let line_count = stdout.lines().count();

    assert!(
        line_count >= 1,
        "config show output should have at least 1 line, got {} lines: '{}'",
        line_count,
        stdout
    );

    let output2 = harness.run_cli(&["config", "show"]);
    let stdout2 = String::from_utf8_lossy(&output2.stdout);
    let line_count2 = stdout2.lines().count();

    assert_eq!(
        line_count, line_count2,
        "config show output should have consistent line count across runs. First run: {} lines, Second run: {} lines",
        line_count, line_count2
    );
}

#[test]
fn cli_contract_config_show_matches_opencode_cli_format() {
    let harness = TestHarness::setup();
    let output = harness.run_cli(&["config", "show"]);

    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);

    assert!(
        stderr.is_empty(),
        "config show should not output to stderr. stderr contained: {}",
        stderr
    );

    assert!(
        stdout.starts_with("Config path:"),
        "config show output should start with 'Config path:'. Got: '{}'",
        stdout
    );

    let lines: Vec<&str> = stdout.lines().collect();
    for line in &lines {
        assert!(
            !line.contains("Invalid") && !line.contains("error") && !line.contains("Error"),
            "config show should not contain error messages. Found line: '{}'",
            line
        );
    }
}

#[test]
fn cli_contract_config_exits_with_code_zero() {
    let harness = TestHarness::setup();
    let output = harness.run_cli(&["config"]);

    assert_eq!(
        output.status.code(),
        Some(0),
        "config (without subcommand) should exit with code 0, got {:?}",
        output.status.code()
    );
}

#[test]
fn cli_contract_config_json_exits_with_code_zero() {
    let harness = TestHarness::setup();
    let output = harness.run_cli(&["config", "--json"]);

    assert_eq!(
        output.status.code(),
        Some(0),
        "config --json should exit with code 0, got {:?}",
        output.status.code()
    );
}

#[test]
fn cli_contract_config_json_output_is_valid_json() {
    let harness = TestHarness::setup();
    let output = harness.run_cli(&["config", "--json"]);

    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);

    assert!(
        stderr.is_empty(),
        "config --json should not output to stderr. stderr contained: {}",
        stderr
    );

    let trimmed = stdout.trim();
    assert!(
        trimmed.starts_with('{'),
        "config --json output should be JSON object. Got: '{}'",
        trimmed
    );

    let parse_result = serde_json::from_str::<serde_json::Value>(trimmed);
    assert!(
        parse_result.is_ok(),
        "config --json output should be valid JSON. Parse error: {:?}. Output: '{}'",
        parse_result.err(),
        trimmed
    );
}

#[test]
fn cli_contract_verbose_help_exits_with_code_zero() {
    let harness = TestHarness::setup();
    let output = harness.run_cli(&["--verbose", "--help"]);

    assert_eq!(
        output.status.code(),
        Some(0),
        "--verbose --help should exit with code 0, got {:?}",
        output.status.code()
    );
}

#[test]
fn cli_contract_verbose_help_output_goes_to_stdout_not_stderr() {
    let harness = TestHarness::setup();
    let output = harness.run_cli(&["--verbose", "--help"]);

    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);

    assert!(
        !stdout.is_empty(),
        "--verbose --help output should go to stdout, but stdout was empty"
    );

    assert!(
        stderr.is_empty(),
        "--verbose --help output should go to stdout, not stderr. stderr contained: {}",
        stderr
    );
}

#[test]
fn cli_contract_verbose_help_contains_expected_content() {
    let harness = TestHarness::setup();
    let output = harness.run_cli(&["--verbose", "--help"]);

    let stdout = String::from_utf8_lossy(&output.stdout);

    assert!(
        stdout.contains("Usage:"),
        "--verbose --help output should contain 'Usage:'"
    );
    assert!(
        stdout.contains("Commands:"),
        "--verbose --help output should contain 'Commands:'"
    );
    assert!(
        stdout.contains("Options:"),
        "--verbose --help output should contain 'Options:'"
    );
    assert!(
        stdout.contains("-h, --help"),
        "--verbose --help output should contain '-h, --help'"
    );
    assert!(
        stdout.contains("opencode-rs"),
        "--verbose --help output should contain 'opencode-rs'"
    );
}

#[test]
fn cli_contract_verbose_help_shows_verbose_flag_in_options() {
    let harness = TestHarness::setup();
    let output = harness.run_cli(&["--verbose", "--help"]);

    let stdout = String::from_utf8_lossy(&output.stdout);

    assert!(
        stdout.contains("--verbose"),
        "--verbose --help output should show the --verbose flag in Options"
    );
}

#[test]
fn cli_contract_verbose_help_matches_plain_help_output() {
    let harness = TestHarness::setup();

    let verbose_help_output = harness.run_cli(&["--verbose", "--help"]);
    let plain_help_output = harness.run_cli(&["--help"]);

    let verbose_stdout = String::from_utf8_lossy(&verbose_help_output.stdout);
    let plain_stdout = String::from_utf8_lossy(&plain_help_output.stdout);

    assert_eq!(
        verbose_stdout, plain_stdout,
        "--verbose --help output should match plain --help output. Verbose: '{}', Plain: '{}'",
        verbose_stdout, plain_stdout
    );
}

#[test]
fn cli_contract_verbose_flag_does_not_interfere_with_help() {
    let harness = TestHarness::setup();
    let output = harness.run_cli(&["--verbose", "--help"]);

    let stdout = String::from_utf8_lossy(&output.stdout);

    assert!(
        stdout.contains("AI coding agent"),
        "Help content should not be affected by verbose flag"
    );
    assert!(
        stdout.contains("tui"),
        "Help should show 'tui' command even with verbose flag"
    );
    assert!(
        stdout.contains("-h, --help"),
        "Help should still show help option with verbose flag"
    );
}
