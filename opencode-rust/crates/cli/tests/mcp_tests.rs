mod common;

use common::TestHarness;

#[test]
fn test_mcp_list_command() {
    let harness = TestHarness::setup();
    let result = harness.run_cli_json(&["mcp", "list"]);

    assert!(result.get("action").is_some());
    assert_eq!(result["action"], "list");
    assert!(result.get("servers").is_some());
}

#[test]
fn test_mcp_list_json_flag() {
    let harness = TestHarness::setup();
    let output = harness.run_cli(&["mcp", "--json", "list"]);

    assert!(
        output.status.success(),
        "mcp list --json should succeed, got: {:?}",
        output.status
    );

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("\"action\"") && stdout.contains("\"list\""));
}

#[test]
fn test_mcp_add_command_nonexistent() {
    let harness = TestHarness::setup();
    let output = harness.run_cli(&[
        "mcp",
        "add",
        "nonexistent-server",
        "nonexistent_command_xyz",
    ]);

    assert!(
        !output.status.success(),
        "mcp add with nonexistent command should fail"
    );

    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("not found") || stderr.contains("Not found"),
        "Error should mention command not found, got: {}",
        stderr
    );
}

#[test]
fn test_mcp_remove_command() {
    let harness = TestHarness::setup();
    let output = harness.run_cli(&["mcp", "remove", "some-server"]);

    assert!(
        !output.status.success(),
        "mcp remove should fail when server doesn't exist"
    );

    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("No MCP servers configured") || stderr.contains("not found"),
        "Error message should indicate server not found, got: {}",
        stderr
    );
}

#[test]
fn test_mcp_help_shows_options() {
    let harness = TestHarness::setup();
    let output = harness.run_cli(&["mcp", "--help"]);

    assert!(
        output.status.success(),
        "mcp --help should succeed, got: {:?}",
        output.status
    );

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("list") || stdout.contains("add") || stdout.contains("remove"),
        "Help should contain list/add/remove options, got: {}",
        stdout
    );
}

#[test]
fn test_mcp_add_help() {
    let harness = TestHarness::setup();
    let output = harness.run_cli(&["mcp", "add", "--help"]);

    assert!(
        output.status.success(),
        "mcp add --help should succeed, got: {:?}",
        output.status
    );

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("NAME") && stdout.contains("COMMAND"),
        "Help should contain NAME and COMMAND arguments, got: {}",
        stdout
    );
}

#[test]
fn test_mcp_auth_command() {
    let harness = TestHarness::setup();
    let output = harness.run_cli(&["mcp", "auth", "list"]);

    assert!(
        output.status.success() || output.status.code() == Some(0),
        "mcp auth list should succeed, got: {:?}",
        output.status
    );
}
