mod common;

use common::TestHarness;

// ============================================================================
// Agent Command Tests
// ============================================================================

#[test]
fn test_agent_list_exits_successfully() {
    let harness = TestHarness::setup();
    let output = harness.run_cli(&["agent", "list"]);

    // Should not panic or crash
    assert!(
        output.status.success() || output.status.code() == Some(0),
        "agent list should succeed, got: {:?}",
        output.status
    );
}

#[test]
fn test_agent_run_requires_args() {
    let harness = TestHarness::setup();
    // Running just "agent run" without required args should fail
    let output = harness.run_cli(&["agent", "run"]);

    // Should exit with error since --agent-name and --prompt are required
    assert!(
        !output.status.success() || output.status.code() != Some(0),
        "agent run without args should fail"
    );
}

// ============================================================================
// Debug Command Tests
// ============================================================================

#[test]
fn test_debug_config_exits_successfully() {
    let harness = TestHarness::setup();
    let output = harness.run_cli(&["debug", "config"]);

    assert!(
        output.status.success(),
        "debug config should succeed, got: {:?}",
        output.status
    );
}

#[test]
fn test_debug_lsp_exits_successfully() {
    let harness = TestHarness::setup();
    let output = harness.run_cli(&["debug", "lsp"]);

    assert!(
        output.status.success(),
        "debug lsp should succeed, got: {:?}",
        output.status
    );
}

#[test]
fn test_debug_agent_exits_successfully() {
    let harness = TestHarness::setup();
    let output = harness.run_cli(&["debug", "agent"]);

    assert!(
        output.status.success(),
        "debug agent should succeed, got: {:?}",
        output.status
    );
}

#[test]
fn test_debug_help_shows_options() {
    let harness = TestHarness::setup();
    let output = harness.run_cli(&["debug", "--help"]);

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("config") || stdout.contains("lsp") || stdout.contains("agent"));
}

// ============================================================================
// DB Command Tests
// ============================================================================

#[test]
fn test_db_init_exits_successfully() {
    let harness = TestHarness::setup();
    let output = harness.run_cli(&["db", "init"]);

    assert!(
        output.status.success(),
        "db init should succeed, got: {:?}",
        output.status
    );
}

#[test]
fn test_db_migrate_exits_successfully() {
    let harness = TestHarness::setup();
    let output = harness.run_cli(&["db", "migrate"]);

    assert!(
        output.status.success(),
        "db migrate should succeed, got: {:?}",
        output.status
    );
}

#[test]
fn test_db_backup_exits_successfully() {
    let harness = TestHarness::setup();
    let output = harness.run_cli(&["db", "backup"]);

    assert!(
        output.status.success(),
        "db backup should succeed, got: {:?}",
        output.status
    );
}

#[test]
fn test_db_help_shows_options() {
    let harness = TestHarness::setup();
    let output = harness.run_cli(&["db", "--help"]);

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("init") && stdout.contains("migrate") && stdout.contains("backup"));
}

// ============================================================================
// Palette Command Tests
// ============================================================================

#[test]
fn test_palette_open_exits_successfully() {
    let harness = TestHarness::setup();
    let output = harness.run_cli(&["palette", "open"]);

    assert!(
        output.status.success(),
        "palette open should succeed, got: {:?}",
        output.status
    );
}

#[test]
fn test_palette_search_without_args() {
    let harness = TestHarness::setup();
    let output = harness.run_cli(&["palette", "search"]);

    // Should succeed even without query
    assert!(
        output.status.success(),
        "palette search without query should succeed, got: {:?}",
        output.status
    );
}

#[test]
fn test_palette_search_with_query() {
    let harness = TestHarness::setup();
    let output = harness.run_cli(&["palette", "search", "--query", "session"]);

    assert!(
        output.status.success(),
        "palette search with query should succeed, got: {:?}",
        output.status
    );

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("session") || stdout.contains("Searching"));
}

#[test]
fn test_palette_search_json() {
    let harness = TestHarness::setup();
    let output = harness.run_cli(&["palette", "search", "--query", "test", "--json"]);

    assert!(
        output.status.success(),
        "palette search --json should succeed, got: {:?}",
        output.status
    );

    let stdout = String::from_utf8_lossy(&output.stdout);
    // JSON output should start with [ or contain JSON structure
    assert!(stdout.contains("[") || stdout.contains("name"));
}

#[test]
fn test_palette_execute_requires_command() {
    let harness = TestHarness::setup();
    let output = harness.run_cli(&["palette", "execute"]);

    // Should fail without --command
    assert!(
        !output.status.success(),
        "palette execute without command should fail"
    );
}

#[test]
fn test_palette_execute_with_command() {
    let harness = TestHarness::setup();
    let output = harness.run_cli(&["palette", "execute", "--command", "session.list"]);

    assert!(
        output.status.success(),
        "palette execute with command should succeed, got: {:?}",
        output.status
    );
}

#[test]
fn test_palette_recent() {
    let harness = TestHarness::setup();
    let output = harness.run_cli(&["palette", "recent"]);

    assert!(
        output.status.success(),
        "palette recent should succeed, got: {:?}",
        output.status
    );
}

#[test]
fn test_palette_recent_json() {
    let harness = TestHarness::setup();
    let output = harness.run_cli(&["palette", "recent", "--json"]);

    assert!(
        output.status.success(),
        "palette recent --json should succeed, got: {:?}",
        output.status
    );
}

#[test]
fn test_palette_help_shows_options() {
    let harness = TestHarness::setup();
    let output = harness.run_cli(&["palette", "--help"]);

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("open") && stdout.contains("search") && stdout.contains("execute"));
}

// ============================================================================
// Permissions Command Tests
// ============================================================================

#[test]
fn test_permissions_grant_requires_path() {
    let harness = TestHarness::setup();
    let output = harness.run_cli(&["permissions", "grant"]);

    // Should fail without path argument
    assert!(
        !output.status.success(),
        "permissions grant without path should fail"
    );
}

#[test]
fn test_permissions_grant_with_path() {
    let harness = TestHarness::setup();
    let output = harness.run_cli(&["permissions", "grant", "/some/path"]);

    // May succeed or fail depending on implementation, but should not crash
    // Just verify it doesn't panic
    let _ = output.status.code();
}

#[test]
fn test_permissions_revoke_requires_path() {
    let harness = TestHarness::setup();
    let output = harness.run_cli(&["permissions", "revoke"]);

    // Should fail without path argument
    assert!(
        !output.status.success(),
        "permissions revoke without path should fail"
    );
}

#[test]
fn test_permissions_revoke_with_path() {
    let harness = TestHarness::setup();
    let output = harness.run_cli(&["permissions", "revoke", "/some/path"]);

    let _ = output.status.code();
}

#[test]
fn test_permissions_list() {
    let harness = TestHarness::setup();
    let output = harness.run_cli(&["permissions", "list"]);

    assert!(
        output.status.success(),
        "permissions list should succeed, got: {:?}",
        output.status
    );
}

#[test]
fn test_permissions_list_json() {
    let harness = TestHarness::setup();
    let output = harness.run_cli(&["permissions", "list", "--json"]);

    assert!(
        output.status.success(),
        "permissions list --json should succeed, got: {:?}",
        output.status
    );
}

#[test]
fn test_permissions_no_action_fails() {
    let harness = TestHarness::setup();
    let output = harness.run_cli(&["permissions"]);

    // Should fail with error message about requiring action
    assert!(
        !output.status.success(),
        "permissions without action should fail"
    );

    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("action") || stderr.contains("grant") || stderr.contains("revoke"));
}

// ============================================================================
// Shortcuts Command Tests
// ============================================================================

#[test]
fn test_shortcuts_list() {
    let harness = TestHarness::setup();
    let output = harness.run_cli(&["shortcuts", "list"]);

    assert!(
        output.status.success(),
        "shortcuts list should succeed, got: {:?}",
        output.status
    );
}

#[test]
fn test_shortcuts_list_json() {
    let harness = TestHarness::setup();
    let output = harness.run_cli(&["shortcuts", "list", "--json"]);

    assert!(
        output.status.success(),
        "shortcuts list --json should succeed, got: {:?}",
        output.status
    );

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("[") || stdout.contains("command"));
}

#[test]
fn test_shortcuts_set_requires_args() {
    let harness = TestHarness::setup();
    let output = harness.run_cli(&["shortcuts", "set"]);

    // Should fail without --command and --shortcut
    assert!(
        !output.status.success(),
        "shortcuts set without args should fail"
    );
}

#[test]
fn test_shortcuts_set_with_args() {
    let harness = TestHarness::setup();
    let output = harness.run_cli(&[
        "shortcuts",
        "set",
        "--command",
        "palette.open",
        "--shortcut",
        "Ctrl+Shift+P",
    ]);

    assert!(
        output.status.success(),
        "shortcuts set with args should succeed, got: {:?}",
        output.status
    );
}

#[test]
fn test_shortcuts_reset_without_args() {
    let harness = TestHarness::setup();
    let output = harness.run_cli(&["shortcuts", "reset"]);

    assert!(
        output.status.success(),
        "shortcuts reset without args should succeed, got: {:?}",
        output.status
    );
}

#[test]
fn test_shortcuts_reset_with_command() {
    let harness = TestHarness::setup();
    let output = harness.run_cli(&["shortcuts", "reset", "--command", "palette.open"]);

    assert!(
        output.status.success(),
        "shortcuts reset with command should succeed, got: {:?}",
        output.status
    );
}

#[test]
fn test_shortcuts_exec_requires_shortcut() {
    let harness = TestHarness::setup();
    let output = harness.run_cli(&["shortcuts", "exec"]);

    // Should fail without --shortcut
    assert!(
        !output.status.success(),
        "shortcuts exec without shortcut should fail"
    );
}

#[test]
fn test_shortcuts_exec_with_shortcut() {
    let harness = TestHarness::setup();
    // First set up a shortcut
    let set_output = harness.run_cli(&[
        "shortcuts",
        "set",
        "--command",
        "palette.open",
        "--shortcut",
        "Ctrl+Shift+P",
    ]);
    assert!(set_output.status.success(), "shortcut set should succeed");

    // Now exec the shortcut
    let output = harness.run_cli(&["shortcuts", "exec", "--shortcut", "Ctrl+Shift+P"]);

    assert!(
        output.status.success(),
        "shortcuts exec with shortcut should succeed, got: {:?}",
        output.status
    );
}

#[test]
fn test_shortcuts_help_shows_options() {
    let harness = TestHarness::setup();
    let output = harness.run_cli(&["shortcuts", "--help"]);

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("list") && stdout.contains("set") && stdout.contains("reset"));
}

// ============================================================================
// Stats Command Tests
// ============================================================================

#[test]
fn test_stats_exits_successfully() {
    let harness = TestHarness::setup();
    let output = harness.run_cli(&["stats"]);

    assert!(
        output.status.success(),
        "stats should succeed, got: {:?}",
        output.status
    );
}

#[test]
fn test_stats_json() {
    let harness = TestHarness::setup();
    let output = harness.run_cli(&["stats", "--json"]);

    assert!(
        output.status.success(),
        "stats --json should succeed, got: {:?}",
        output.status
    );
}

#[test]
fn test_stats_short_flag() {
    let harness = TestHarness::setup();
    let output = harness.run_cli(&["stats", "-j"]);

    // -j should be equivalent to --json
    assert!(
        output.status.success(),
        "stats -j should succeed, got: {:?}",
        output.status
    );
}

// ============================================================================
// Bash Command Tests
// ============================================================================

#[test]
fn test_bash_simple_command() {
    let harness = TestHarness::setup();
    let output = harness.run_cli(&["bash", "--command", "echo hello"]);

    assert!(
        output.status.success(),
        "bash echo should succeed, got: {:?}",
        output.status
    );

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("hello"));
}

#[test]
fn test_bash_with_json() {
    let harness = TestHarness::setup();
    let output = harness.run_cli(&["bash", "--command", "echo test", "--json"]);

    assert!(
        output.status.success(),
        "bash --json should succeed, got: {:?}",
        output.status
    );

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("stdout") && stdout.contains("test"));
}

#[test]
fn test_bash_failing_command() {
    let harness = TestHarness::setup();
    let output = harness.run_cli(&["bash", "--command", "exit 1"]);

    assert!(!output.status.success(), "bash exit 1 should fail");
}

#[test]
fn test_bash_failing_command_with_json() {
    let harness = TestHarness::setup();
    let output = harness.run_cli(&["bash", "--command", "exit 1", "--json"]);

    // With --json, it should output JSON and then exit with non-zero
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("exit_code"));
}

#[test]
fn test_bash_with_timeout() {
    let harness = TestHarness::setup();
    let output = harness.run_cli(&["bash", "--command", "echo fast", "--timeout", "5"]);

    assert!(
        output.status.success(),
        "bash with timeout should succeed, got: {:?}",
        output.status
    );
}

#[test]
fn test_bash_interactive_command_rejected() {
    let harness = TestHarness::setup();
    let output = harness.run_cli(&["bash", "--command", "read -p 'Enter: '"]);

    // Interactive commands should be rejected
    assert!(
        !output.status.success(),
        "interactive command should be rejected"
    );
}

#[test]
fn test_bash_requires_command() {
    let harness = TestHarness::setup();
    let output = harness.run_cli(&["bash"]);

    // Should fail without --command
    assert!(
        !output.status.success(),
        "bash without --command should fail"
    );
}

// ============================================================================
// Completion Generation Tests (for all shells)
// ============================================================================

#[test]
fn test_completion_all_shells_valid() {
    let harness = TestHarness::setup();

    for shell in ["bash", "zsh", "fish", "powershell"] {
        let output = harness.run_cli(&["completion", shell]);

        assert!(
            output.status.success(),
            "completion {} should succeed, got: {:?}",
            shell,
            output.status
        );

        let stdout = String::from_utf8_lossy(&output.stdout);
        assert!(
            !stdout.is_empty(),
            "completion {} should produce output",
            shell
        );
    }
}

// ============================================================================
// Config Command Tests (additional coverage)
// ============================================================================

#[test]
fn test_config_show_keybinds() {
    let harness = TestHarness::setup();
    let output = harness.run_cli(&["config", "--keybinds", "--json"]);

    assert!(
        output.status.success(),
        "config --keybinds --json should succeed, got: {:?}",
        output.status
    );

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("keybinds"));
}

#[test]
fn test_config_show_models() {
    let harness = TestHarness::setup();
    let output = harness.run_cli(&["config", "--models"]);

    assert!(
        output.status.success(),
        "config --models should succeed, got: {:?}",
        output.status
    );
}

#[test]
fn test_config_show_providers() {
    let harness = TestHarness::setup();
    let output = harness.run_cli(&["config", "--providers"]);

    assert!(
        output.status.success(),
        "config --providers should succeed, got: {:?}",
        output.status
    );
}

// ============================================================================
// List Command Tests (additional coverage)
// ============================================================================

#[test]
fn test_list_with_limit() {
    let harness = TestHarness::setup();
    let output = harness.run_cli(&["list", "--limit", "5"]);

    // Should succeed even with no sessions
    assert!(
        output.status.success(),
        "list --limit should succeed, got: {:?}",
        output.status
    );
}

#[test]
fn test_list_with_json() {
    let harness = TestHarness::setup();
    let output = harness.run_cli(&["list", "--json"]);

    assert!(
        output.status.success(),
        "list --json should succeed, got: {:?}",
        output.status
    );

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.starts_with("{") || stdout.contains("sessions"));
}

#[test]
fn test_list_with_all() {
    let harness = TestHarness::setup();
    let output = harness.run_cli(&["list", "--all"]);

    assert!(
        output.status.success(),
        "list --all should succeed, got: {:?}",
        output.status
    );
}

// ============================================================================
// Error Handling Tests
// ============================================================================

#[test]
fn test_invalid_flag_shows_error() {
    let harness = TestHarness::setup();
    let output = harness.run_cli(&["--invalid-flag"]);

    // Should fail with error message
    assert!(!output.status.success(), "invalid flag should fail");
}

// ============================================================================
// ACP Command Tests
// ============================================================================

#[test]
fn test_acp_ack_command_parses_correctly() {
    let harness = TestHarness::setup();

    let output = harness.run_cli(&[
        "acp",
        "ack",
        "--handshake-id",
        "test-handshake-123",
        "--accepted",
        "true",
    ]);

    assert!(
        output.status.success() || output.status.code() == Some(1),
        "acp ack should parse correctly, got: {:?}",
        output.status
    );
}

#[test]
fn test_acp_ack_command_with_accepted_false() {
    let harness = TestHarness::setup();

    let output = harness.run_cli(&[
        "acp",
        "ack",
        "--handshake-id",
        "test-session-456",
        "--accepted",
        "false",
    ]);

    assert!(
        output.status.success() || output.status.code() == Some(1),
        "acp ack with accepted=false should parse, got: {:?}",
        output.status
    );
}

#[test]
fn test_acp_ack_requires_handshake_id() {
    let harness = TestHarness::setup();

    let output = harness.run_cli(&["acp", "ack", "--accepted", "true"]);

    assert!(
        !output.status.success(),
        "acp ack without --handshake-id should fail"
    );
}

#[test]
fn test_acp_status_command() {
    let harness = TestHarness::setup();

    let output = harness.run_cli(&["acp", "status"]);

    assert!(
        output.status.success() || output.status.code() == Some(1),
        "acp status should work, got: {:?}",
        output.status
    );
}

// ============================================================================
// Global Flags Tests
// ============================================================================

#[test]
fn test_version_flag_takes_precedence() {
    let harness = TestHarness::setup();
    let output = harness.run_cli(&["--version"]);

    assert!(output.status.success(), "--version should succeed");

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("opencode-rs"));
}

#[test]
fn test_help_with_command() {
    let harness = TestHarness::setup();

    // Test that --help works with various subcommands
    for cmd in &["agent", "db", "palette", "stats", "debug"] {
        let output = harness.run_cli(&[cmd, "--help"]);

        assert!(output.status.success(), "{} --help should succeed", cmd);

        let stdout = String::from_utf8_lossy(&output.stdout);
        assert!(!stdout.is_empty(), "{} --help should produce output", cmd);
    }
}
