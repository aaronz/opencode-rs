use crate::common::{TestHarness, EMPTY_VEC};

mod common;

#[test]
fn test_sidebar_toggle() {
    let harness = TestHarness::setup();

    let output = harness.run_cli(&["ui", "sidebar", "toggle"]);
    assert!(output.status.success());
}

#[test]
fn test_sidebar_sessions_list() {
    let harness = TestHarness::setup();

    let _session1 = harness.create_session("sidebar-session-1");
    let _session2 = harness.create_session("sidebar-session-2");

    let output = harness.run_cli(&["ui", "sidebar", "sessions", "--json"]);
    assert!(output.status.success());

    let stdout = String::from_utf8_lossy(&output.stdout);
    let json: serde_json::Value = serde_json::from_str(&stdout).unwrap_or_default();

    let sessions = json.as_array().unwrap_or(&EMPTY_VEC);
    assert!(sessions.len() >= 2);
}

#[test]
fn test_sidebar_recent_sessions() {
    let harness = TestHarness::setup();

    harness.create_session("recent-1");
    harness.create_session("recent-2");
    harness.create_session("recent-3");

    let output = harness.run_cli(&["ui", "sidebar", "recent", "--limit", "2", "--json"]);
    assert!(output.status.success());

    let stdout = String::from_utf8_lossy(&output.stdout);
    let json: serde_json::Value = serde_json::from_str(&stdout).unwrap_or_default();

    let sessions = json.as_array().unwrap_or(&EMPTY_VEC);
    assert!(sessions.len() <= 2);
}
