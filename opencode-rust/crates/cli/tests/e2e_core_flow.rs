mod common;
use common::TestHarness;

#[test]
fn test_cli_account_status() {
    let harness = TestHarness::setup();
    let result = harness.run_cli_json(&["account", "status"]);

    assert!(result.get("action").is_some());
    assert_eq!(result["action"], "status");
    assert!(result.get("logged_in").is_some());
}

#[test]
fn test_cli_list_sessions() {
    let harness = TestHarness::setup();
    let result = harness.run_cli_json(&["list"]);

    assert!(result.get("action").is_some());
    assert_eq!(result["action"], "list");
    assert!(result.get("sessions").unwrap().is_array());
}

#[test]
fn test_cli_providers_list() {
    let harness = TestHarness::setup();
    let result = harness.run_cli_json(&["providers"]);

    assert_eq!(result["action"], "list");
    let providers = result["providers"].as_array().unwrap();
    assert!(providers.iter().any(|provider| provider["id"] == "openai"));
    assert!(providers
        .iter()
        .any(|provider| provider["id"] == "anthropic"));
    assert!(providers
        .iter()
        .all(|provider| provider.get("status").is_some()));
}

#[test]
fn test_cli_providers_openai_browser_login_action() {
    let harness = TestHarness::setup();
    let result = harness.run_cli_json(&["providers", "--login", "openai", "--browser"]);

    assert_eq!(result["action"], "login");
    assert_eq!(result["provider"], "openai");
    assert_eq!(result["method"], "browser");
}

#[test]
fn test_cli_acp_start() {
    let harness = TestHarness::setup();
    let result = harness.run_cli_json(&["acp", "--action", "start"]);

    assert_eq!(result["component"], "acp");
    assert_eq!(result["action"], "start");
    assert_eq!(result["status"], "ready");
}

#[test]
fn test_cli_mcp_list() {
    let harness = TestHarness::setup();
    let result = harness.run_cli_json(&["mcp", "list"]);

    assert_eq!(result["action"], "list");
    assert!(result.get("servers").unwrap().is_array());
}

#[test]
fn test_cli_uninstall_dry_run() {
    let harness = TestHarness::setup();
    let result = harness.run_cli_json(&["uninstall"]);

    assert_eq!(result["action"], "uninstall");
    assert_eq!(result["status"], "dry_run");
    assert_eq!(result["force"], false);
}
