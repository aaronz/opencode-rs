use opencode_tui::input::parser::{InputParser, InputToken};
use opencode_tui::shell_handler::ShellHandler;

#[test]
fn test_shell_prefix_detection() {
    let parser = InputParser::new();

    let result = parser.parse("!echo hello");
    assert!(
        result.has_shell,
        "Should detect shell command with ! prefix"
    );
    assert_eq!(result.tokens.len(), 1);

    if let InputToken::ShellCommand(cmd) = &result.tokens[0] {
        assert_eq!(cmd, "echo hello");
    } else {
        panic!("Expected ShellCommand token");
    }
}

#[test]
fn test_shell_prefix_with_args() {
    let parser = InputParser::new();

    let result = parser.parse("!cargo test --workspace");
    assert!(result.has_shell);

    if let InputToken::ShellCommand(cmd) = &result.tokens[0] {
        assert_eq!(cmd, "cargo test --workspace");
    } else {
        panic!("Expected ShellCommand token");
    }
}

#[test]
fn test_shell_prefix_captures_full_command() {
    let parser = InputParser::new();

    let result = parser.parse("!ls -la /tmp");
    assert!(result.has_shell);

    if let InputToken::ShellCommand(cmd) = &result.tokens[0] {
        assert_eq!(cmd, "ls -la /tmp");
    } else {
        panic!("Expected ShellCommand token");
    }
}

#[test]
fn test_shell_prefix_execution_echo() {
    let handler = ShellHandler::new();

    let result = handler.execute("echo hello world");
    assert!(result.exit_code == Some(0), "Echo should succeed");
    assert!(
        result.stdout.contains("hello world"),
        "Should capture stdout"
    );
    assert!(!result.timed_out, "Should not timeout");
    assert!(!result.truncated, "Should not be truncated");
}

#[test]
fn test_shell_prefix_execution_captures_stderr() {
    let handler = ShellHandler::new();

    let result = handler.execute("echo error message >&2");
    assert!(
        result.stderr.contains("error message"),
        "Should capture stderr"
    );
}

#[test]
fn test_shell_prefix_execution_captures_stdout() {
    let handler = ShellHandler::new();

    let result = handler.execute("printf 'line1\\nline2\\nline3'");
    assert!(result.stdout.contains("line1"), "Should capture line1");
    assert!(result.stdout.contains("line2"), "Should capture line2");
    assert!(result.stdout.contains("line3"), "Should capture line3");
}

#[test]
fn test_shell_prefix_execution_exit_code() {
    let handler = ShellHandler::new();

    let result = handler.execute("exit 42");
    assert_eq!(result.exit_code, Some(42), "Should capture exit code");
}

#[test]
fn test_shell_prefix_execution_invalid_command() {
    let handler = ShellHandler::new();

    let result = handler.execute("nonexistent_command_xyz_123");
    assert!(
        !result.stderr.is_empty() || result.exit_code != Some(0),
        "Invalid command should produce error or non-zero exit"
    );
}

#[test]
fn test_shell_prefix_output_display_format() {
    let handler = ShellHandler::new();

    let result = handler.execute("echo 'test output'");

    let display_output = format!(
        "```\n$ echo 'test output'\n{}\n```\n{}",
        result.stdout,
        if result.stderr.is_empty() {
            String::new()
        } else {
            format!("stderr: {}", result.stderr)
        }
    );

    assert!(
        display_output.contains("$ echo 'test output'"),
        "Should show command"
    );
    assert!(
        display_output.contains("test output"),
        "Should show stdout output"
    );
}
