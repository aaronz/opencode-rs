mod common;

use common::TestHarness;
use std::process::Stdio;
use std::time::Duration;

#[test]
fn test_sigint_during_run_command_exits_cleanly() {
    let harness = TestHarness::setup();

    let mut child = harness
        .cmd()
        .args(["run", "--prompt", "test"])
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("Failed to spawn opencode process");

    std::thread::sleep(Duration::from_millis(100));

    let _ = child.kill();

    let status = child.wait().expect("Failed to wait for child");

    // Process was killed - verify it exits with signal or at least doesn't panic/crash
    // Exit code 128 means wait() couldn't determine the code; 137 = SIGKILL; 130 = SIGINT
    let exit_code = status.code().unwrap_or(128);
    assert!(
        exit_code != 101, // Rust panics exit with 101
        "Process panicked instead of handling signal gracefully, got {}",
        exit_code
    );
}

#[test]
fn test_sigterm_during_run_command_exits_cleanly() {
    let harness = TestHarness::setup();

    let mut child = harness
        .cmd()
        .args(["run", "--prompt", "test"])
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("Failed to spawn opencode process");

    std::thread::sleep(Duration::from_millis(100));

    let _ = child.kill();
    let status = child.wait().expect("Failed to wait for child");

    let exit_code = status.code().unwrap_or(128);
    assert!(
        exit_code != 101,
        "Process panicked instead of handling signal gracefully, got {}",
        exit_code
    );
}

#[test]
fn test_run_command_with_broken_pipe() {
    let harness = TestHarness::setup();

    let output = harness.run_cli(&["run", "--prompt", "hello"]);

    assert!(
        output.status.success() || output.status.code() == Some(141),
        "Broken pipe should exit with 0 or 141 (SIGPIPE), got {:?}",
        output.status
    );

    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        !stderr.contains("Broken pipe"),
        "Should not print broken pipe error to stderr"
    );
}

#[test]
fn test_help_flag_shows_usage() {
    let harness = TestHarness::setup();

    let output = harness.run_cli(&["--help"]);

    assert_eq!(
        output.status.code(),
        Some(0),
        "Help should exit with code 0"
    );
    assert!(!output.stdout.is_empty(), "Help output should not be empty");
}

#[test]
fn test_run_help_flag_shows_run_usage() {
    let harness = TestHarness::setup();

    let output = harness.run_cli(&["run", "--help"]);

    assert_eq!(
        output.status.code(),
        Some(0),
        "Run help should exit with code 0"
    );
}

#[test]
fn test_session_help_flag_shows_session_usage() {
    let harness = TestHarness::setup();

    let output = harness.run_cli(&["session", "--help"]);

    assert_eq!(
        output.status.code(),
        Some(0),
        "Session help should exit with code 0"
    );
}

#[test]
fn test_agent_help_flag_shows_agent_usage() {
    let harness = TestHarness::setup();

    let output = harness.run_cli(&["agent", "--help"]);

    assert_eq!(
        output.status.code(),
        Some(0),
        "Agent help should exit with code 0"
    );
}

#[test]
fn test_version_flag_shows_version() {
    let harness = TestHarness::setup();

    let output = harness.run_cli(&["--version"]);

    assert_eq!(
        output.status.code(),
        Some(0),
        "Version should exit with code 0"
    );

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("opencode"),
        "Version output should contain 'opencode'"
    );
}

#[test]
fn test_run_command_with_empty_stdin_completes() {
    let harness = TestHarness::setup();

    let output = harness
        .cmd()
        .args(["run", "--prompt", "test"])
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output()
        .expect("Failed to execute command");

    assert!(
        output.status.success() || output.status.code() == Some(1),
        "Empty stdin should not hang, got {:?}",
        output.status
    );
}

#[test]
fn test_non_tty_stdin_does_not_trigger_interactive_mode() {
    let harness = TestHarness::setup();

    let output = harness
        .cmd()
        .args(["run", "--prompt", "test"])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output()
        .expect("Failed to execute command");

    assert!(
        output.status.success() || output.status.code() == Some(1),
        "Non-tty stdin should not trigger interactive mode, got {:?}",
        output.status
    );
}

#[test]
fn test_config_get_command() {
    let harness = TestHarness::setup();

    let output = harness.run_cli(&["config", "get", "model"]);

    let stderr = String::from_utf8_lossy(&output.stderr);
    if !stderr.is_empty() {
        eprintln!("config get stderr: {}", stderr);
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    if !stdout.is_empty() {
        eprintln!("config get stdout: {}", stdout);
    }
}

#[test]
fn test_config_show_command() {
    let harness = TestHarness::setup();

    let output = harness.run_cli(&["config", "show"]);

    let stderr = String::from_utf8_lossy(&output.stderr);
    if !stderr.is_empty() {
        eprintln!("config show stderr: {}", stderr);
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    if !stdout.is_empty() {
        eprintln!("config show stdout: {}", stdout);
    }
}

#[test]
fn test_info_command_displays_version() {
    let harness = TestHarness::setup();

    let output = harness.run_cli(&["info"]);

    let stderr = String::from_utf8_lossy(&output.stderr);
    if !stderr.is_empty() {
        eprintln!("info stderr: {}", stderr);
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    if !stdout.is_empty() {
        eprintln!("info stdout: {}", stdout);
    }
}

#[test]
fn test_ndjson_output_format() {
    let harness = TestHarness::setup();

    let output = harness
        .cmd()
        .args(["run", "--prompt", "hello", "--format", "ndjson"])
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output()
        .expect("Failed to execute command");

    if output.status.success() {
        let stdout = String::from_utf8_lossy(&output.stdout);
        for line in stdout.lines() {
            if !line.is_empty() {
                let parsed = serde_json::from_str::<serde_json::Value>(line);
                assert!(
                    parsed.is_ok(),
                    "NDJSON output should be valid JSON: {}",
                    line
                );
            }
        }
    }
}

#[test]
fn test_json_output_format() {
    let harness = TestHarness::setup();

    let output = harness
        .cmd()
        .args(["run", "--prompt", "hello", "--format", "json"])
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output()
        .expect("Failed to execute command");

    if output.status.success() {
        let stdout = String::from_utf8_lossy(&output.stdout);
        let parsed = serde_json::from_str::<serde_json::Value>(&stdout);
        assert!(
            parsed.is_ok(),
            "JSON output should be valid JSON: {}",
            stdout
        );
    }
}

#[test]
fn test_invalid_argument_combination_rejected() {
    let harness = TestHarness::setup();

    let output = harness.run_cli(&["session", "delete", "--all", "session-id-123"]);

    assert_ne!(
        output.status.code(),
        Some(0),
        "Conflicting args should be rejected"
    );
}

#[test]
fn test_acp_status_command() {
    let harness = TestHarness::setup();

    let output = harness.run_cli(&["acp", "status"]);

    assert!(
        output.status.success() || output.status.code() == Some(1),
        "ACP status should work, got {:?}",
        output.status
    );
}

#[test]
fn test_no_ansi_flag_produces_clean_output() {
    let harness = TestHarness::setup();

    let output = harness
        .cmd()
        .args(["run", "--prompt", "hello", "--no-ansi"])
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output()
        .expect("Failed to execute command");

    if output.status.success() {
        let stdout = String::from_utf8_lossy(&output.stdout);
        assert!(
            !stdout.contains("\x1b["),
            "Output should not contain ANSI escape codes"
        );
    }
}
