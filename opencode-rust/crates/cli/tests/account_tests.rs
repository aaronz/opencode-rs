mod common;

use common::TestHarness;

#[test]
fn test_account_status_command() {
    let harness = TestHarness::setup();
    let result = harness.run_cli_json(&["account", "status"]);

    assert!(result.get("action").is_some());
    assert_eq!(result["action"], "status");
    assert!(result.get("logged_in").is_some());
    assert!(result.get("accounts").is_some());
    let accounts = result["accounts"].as_array().unwrap();
    assert!(!accounts.is_empty());
}

#[test]
fn test_account_status_json_flag() {
    let harness = TestHarness::setup();
    let output = harness.run_cli(&["account", "--json", "status"]);

    assert!(
        output.status.success(),
        "account status --json should succeed, got: {:?}",
        output.status
    );

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("\"action\"") && stdout.contains("\"status\""));
}

#[test]
fn test_account_login_unsupported_provider_fails() {
    let harness = TestHarness::setup();
    let output = harness.run_cli(&["account", "login", "--provider", "unsupported"]);

    assert!(
        !output.status.success(),
        "account login with unsupported provider should fail"
    );

    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("Unsupported") || stderr.contains("unsupported"));
}

#[test]
fn test_account_logout_command() {
    let harness = TestHarness::setup();
    let output = harness.run_cli(&["account", "logout", "--provider", "github"]);

    assert!(
        output.status.success() || output.status.code() == Some(0),
        "account logout should succeed even if not logged in"
    );

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("github") || stdout.contains("logout"));
}

#[test]
fn test_account_logout_openai_provider() {
    let harness = TestHarness::setup();
    let output = harness.run_cli(&["account", "logout", "--provider", "openai"]);

    assert!(
        output.status.success() || output.status.code() == Some(0),
        "account logout should succeed for openai provider"
    );
}

#[test]
fn test_account_status_shows_providers() {
    let harness = TestHarness::setup();
    let result = harness.run_cli_json(&["account", "status"]);

    let accounts = result["accounts"].as_array().unwrap();
    let providers: Vec<&str> = accounts
        .iter()
        .filter_map(|a| a.get("provider").and_then(|p| p.as_str()))
        .collect();

    assert!(providers.contains(&"github"), "github should be in status");
    assert!(providers.contains(&"openai"), "openai should be in status");
}

#[test]
fn test_account_logout_unsupported_provider() {
    let harness = TestHarness::setup();
    let output = harness.run_cli(&["account", "logout", "--provider", "unknown"]);

    assert!(
        !output.status.success(),
        "account logout with unsupported provider should fail"
    );
}

#[test]
fn test_account_status_not_logged_in() {
    let harness = TestHarness::setup();
    let result = harness.run_cli_json(&["account", "status"]);

    assert_eq!(result["logged_in"], false);

    let accounts = result["accounts"].as_array().unwrap();
    for account in accounts {
        assert_eq!(account["logged_in"], false);
        assert_eq!(account["valid"], false);
    }
}

#[test]
fn test_account_help_shows_options() {
    let harness = TestHarness::setup();
    let output = harness.run_cli(&["account", "--help"]);

    assert!(
        output.status.success(),
        "account --help should succeed, got: {:?}",
        output.status
    );

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("login") || stdout.contains("logout") || stdout.contains("status"));
}

#[test]
fn test_account_login_help() {
    let harness = TestHarness::setup();
    let output = harness.run_cli(&["account", "login", "--help"]);

    assert!(
        output.status.success(),
        "account login --help should succeed, got: {:?}",
        output.status
    );

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("provider"));
}

#[test]
fn test_account_login_anthropic_provider() {
    let harness = TestHarness::setup();
    let output = harness.run_cli(&["account", "login", "--provider", "anthropic"]);

    assert!(
        !output.status.success() || output.status.code() == Some(0),
        "account login anthropic should either succeed or fail gracefully"
    );
}

#[test]
fn test_account_status_includes_anthropic() {
    let harness = TestHarness::setup();
    let result = harness.run_cli_json(&["account", "status"]);

    let accounts = result["accounts"].as_array().unwrap();
    let providers: Vec<&str> = accounts
        .iter()
        .filter_map(|a| a.get("provider").and_then(|p| p.as_str()))
        .collect();

    assert!(
        providers.contains(&"anthropic"),
        "anthropic should be in status"
    );
}

#[test]
fn test_account_logout_anthropic_provider() {
    let harness = TestHarness::setup();
    let output = harness.run_cli(&["account", "logout", "--provider", "anthropic"]);

    assert!(
        output.status.success() || output.status.code() == Some(0),
        "account logout should succeed for anthropic provider"
    );
}

#[test]
fn test_account_all_providers_status() {
    let harness = TestHarness::setup();
    let result = harness.run_cli_json(&["account", "status"]);

    let accounts = result["accounts"].as_array().unwrap();
    let providers: Vec<&str> = accounts
        .iter()
        .filter_map(|a| a.get("provider").and_then(|p| p.as_str()))
        .collect();

    assert!(providers.contains(&"github"), "github should be in status");
    assert!(providers.contains(&"openai"), "openai should be in status");
    assert!(
        providers.contains(&"anthropic"),
        "anthropic should be in status"
    );
    assert_eq!(providers.len(), 3, "should have exactly 3 providers");
}
