use std::sync::Arc;
use opencode_acp::{AcpClient, AcpConnectionState, AcpState};

fn create_test_client() -> AcpClient {
    let http = reqwest::Client::new();
    let bus: opencode_core::bus::SharedEventBus = Arc::new(opencode_core::bus::EventBus::new());
    AcpClient::new(http, "test-client".to_string(), bus)
}

#[test]
fn test_acp_client_instantiation_with_all_fields() {
    let http = reqwest::Client::new();
    let bus: opencode_core::bus::SharedEventBus = Arc::new(opencode_core::bus::EventBus::new());
    let client_id = "test-client-123".to_string();

    let client = AcpClient::new(http, client_id.clone(), bus);

    assert_eq!(client.connection_state(), AcpConnectionState::Disconnected);
}

#[test]
fn test_acp_client_field_access_via_public_methods() {
    let http = reqwest::Client::new();
    let bus: opencode_core::bus::SharedEventBus = Arc::new(opencode_core::bus::EventBus::new());
    let client_id = "field-access-test".to_string();

    let client = AcpClient::new(http, client_id.clone(), bus);

    let state = client.connection_state();
    assert!(matches!(state, AcpConnectionState::Disconnected));
}

#[tokio::test]
async fn test_connection_state_initially_disconnected() {
    let client = create_test_client();
    assert_eq!(client.connection_state(), AcpConnectionState::Disconnected);
}

#[test]
fn test_acp_state_instantiation() {
    let state = AcpState {
        connection_state: AcpConnectionState::Disconnected,
        client_id: "test-client".to_string(),
        server_id: None,
        session_token: None,
        capabilities: Vec::new(),
        server_url: None,
    };
    assert!(matches!(state.connection_state, AcpConnectionState::Disconnected));
    assert_eq!(state.client_id, "test-client");
}

#[test]
fn test_acp_state_fields_accessible() {
    let mut state = AcpState {
        connection_state: AcpConnectionState::Disconnected,
        client_id: "client-123".to_string(),
        server_id: Some("server-456".to_string()),
        session_token: Some("token-789".to_string()),
        capabilities: vec!["chat".to_string(), "tasks".to_string()],
        server_url: Some("http://localhost:8080".to_string()),
    };

    assert_eq!(state.client_id, "client-123");
    assert_eq!(state.server_id, Some("server-456".to_string()));
    assert_eq!(state.session_token, Some("token-789".to_string()));
    assert_eq!(state.capabilities.len(), 2);
    assert_eq!(state.server_url, Some("http://localhost:8080".to_string()));

    state.connection_state = AcpConnectionState::Connected;
    state.capabilities.push("files".to_string());

    assert!(matches!(state.connection_state, AcpConnectionState::Connected));
    assert_eq!(state.capabilities.len(), 3);
}