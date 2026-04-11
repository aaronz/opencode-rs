mod common;
use common::TestHarness;

#[tokio::test]
async fn acp_handshake_connection_establishes_correctly() {
    use opencode_core::acp::{AcpHandshakeRequest, AcpProtocol};

    let protocol = AcpProtocol::new("test-server", "1.0");
    let request = AcpHandshakeRequest {
        version: "1.0".to_string(),
        client_id: "test-client".to_string(),
        capabilities: vec!["chat".to_string(), "tasks".to_string()],
    };

    let response = protocol.process_handshake(request);

    assert!(response.accepted, "Handshake should be accepted");
    assert!(response.error.is_none(), "Should have no error");
    assert!(
        !response.session_id.is_empty(),
        "Session ID should be generated"
    );
    assert_eq!(response.server_id, "test-server");
    assert_eq!(response.version, "1.0");
}

#[tokio::test]
async fn acp_handshake_capabilities_negotiated_properly() {
    use opencode_core::acp::{AcpHandshakeRequest, AcpProtocol};

    let protocol = AcpProtocol::new("test-server", "1.0");
    let request = AcpHandshakeRequest {
        version: "1.0".to_string(),
        client_id: "test-client".to_string(),
        capabilities: vec![
            "chat".to_string(),
            "tasks".to_string(),
            "code_review".to_string(),
        ],
    };

    let response = protocol.process_handshake(request);

    assert!(
        response.accepted,
        "Handshake with capabilities should be accepted"
    );
    assert!(
        !response.session_id.is_empty(),
        "Session should be established with capabilities"
    );
}

#[tokio::test]
async fn acp_handshake_invalid_version_rejected() {
    use opencode_core::acp::{AcpHandshakeRequest, AcpProtocol};

    let protocol = AcpProtocol::new("test-server", "2.0");
    let request = AcpHandshakeRequest {
        version: "1.0".to_string(),
        client_id: "test-client".to_string(),
        capabilities: vec!["chat".to_string()],
    };

    let response = protocol.process_handshake(request);

    assert!(
        !response.accepted,
        "Handshake with wrong version should be rejected"
    );
    assert!(response.error.is_some(), "Should have an error message");
    let error = response.error.unwrap();
    assert!(
        error.contains("Version mismatch"),
        "Error should mention version mismatch: {}",
        error
    );
    assert!(
        response.session_id.is_empty(),
        "Session ID should be empty on rejection"
    );
}

#[tokio::test]
async fn acp_handshake_confirm_with_valid_session() {
    use opencode_core::acp::{AcpHandshakeAck, AcpHandshakeRequest, AcpProtocol};

    let protocol = AcpProtocol::new("test-server", "1.0");
    let request = AcpHandshakeRequest {
        version: "1.0".to_string(),
        client_id: "test-client".to_string(),
        capabilities: vec![],
    };

    let response = protocol.process_handshake(request);
    assert!(response.accepted, "Initial handshake should succeed");

    let ack = AcpHandshakeAck {
        session_id: response.session_id.clone(),
        confirmed: true,
    };

    let confirmed = protocol.confirm_handshake(ack);
    assert!(
        confirmed,
        "Handshake should be confirmable with valid session"
    );
}

#[tokio::test]
async fn acp_handshake_confirm_rejected_with_empty_session() {
    use opencode_core::acp::{AcpHandshakeAck, AcpProtocol};

    let protocol = AcpProtocol::new("test-server", "1.0");
    let ack = AcpHandshakeAck {
        session_id: String::new(),
        confirmed: true,
    };

    let confirmed = protocol.confirm_handshake(ack);
    assert!(
        !confirmed,
        "Confirmation should be rejected with empty session"
    );
}

#[tokio::test]
async fn acp_handshake_confirm_rejected_when_not_confirmed() {
    use opencode_core::acp::{AcpHandshakeAck, AcpProtocol};

    let protocol = AcpProtocol::new("test-server", "1.0");
    let ack = AcpHandshakeAck {
        session_id: "valid-session".to_string(),
        confirmed: false,
    };

    let confirmed = protocol.confirm_handshake(ack);
    assert!(
        !confirmed,
        "Confirmation should be rejected when confirmed=false"
    );
}

#[tokio::test]
async fn acp_handshake_different_client_ids() {
    use opencode_core::acp::{AcpHandshakeRequest, AcpProtocol};

    let protocol = AcpProtocol::new("test-server", "1.0");

    let client_ids = vec!["client-a", "client-b", "test-client-123"];

    for client_id in client_ids {
        let request = AcpHandshakeRequest {
            version: "1.0".to_string(),
            client_id: client_id.to_string(),
            capabilities: vec![],
        };

        let response = protocol.process_handshake(request);
        assert!(
            response.accepted,
            "Handshake should succeed for client_id: {}",
            client_id
        );
        assert!(
            !response.session_id.is_empty(),
            "Session ID should be generated for client_id: {}",
            client_id
        );
    }
}

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
    let result = harness.run_cli_json(&["acp", "start"]);

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
