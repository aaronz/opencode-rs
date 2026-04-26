use opencode_acp::{
    AcpClient, AcpConnectionState, AcpError, AcpMessage, AcpState, HandshakeRequest,
};

fn create_test_client() -> AcpClient {
    let http = reqwest::Client::new();
    AcpClient::new(http, "test-client".to_string())
}

#[tokio::test]
async fn test_status_returns_disconnected_initially() {
    let client = create_test_client();
    let status = client.status().await.unwrap();
    assert!(!status.connected);
}

#[tokio::test]
async fn test_connection_state_initially_disconnected() {
    let client = create_test_client();
    assert_eq!(client.connection_state(), AcpConnectionState::Disconnected);
}

#[tokio::test]
async fn test_disconnect_transitions_to_disconnected() {
    let client = create_test_client();
    client.disconnect().await.unwrap();
    let status = client.status().await.unwrap();
    assert!(!status.connected);
}

#[tokio::test]
async fn test_send_message_returns_error_when_not_connected() {
    let client = create_test_client();
    let result = client
        .send_message("srv", "chat", serde_json::json!({"text": "hi"}))
        .await;
    assert!(matches!(result, Err(AcpError::NotConnected)));
}

#[tokio::test]
async fn test_ack_returns_error_when_not_connected() {
    let client = create_test_client();
    let result = client.ack("handshake-123", true).await;
    assert!(matches!(result, Err(AcpError::NotConnected)));
}

#[tokio::test]
async fn test_handshake_request_structure() {
    let request = HandshakeRequest {
        client_id: "client1".to_string(),
        capabilities: vec!["chat".to_string(), "tasks".to_string()],
        version: "1.0".to_string(),
    };
    let json = serde_json::to_string(&request).unwrap();
    assert!(json.contains("client1"));
    assert!(json.contains("chat"));
    assert!(json.contains("tasks"));
}

#[tokio::test]
async fn test_acp_message_creation() {
    let msg = AcpMessage::new(
        "from1".to_string(),
        "to1".to_string(),
        "chat".to_string(),
        serde_json::json!({"text": "hello"}),
    );
    assert_eq!(msg.from, "from1");
    assert_eq!(msg.to, "to1");
    assert_eq!(msg.message_type, "chat");
    assert_eq!(msg.payload["text"], "hello");
}

#[tokio::test]
async fn test_connection_state_display() {
    assert_eq!(AcpConnectionState::Disconnected.to_string(), "Disconnected");
    assert_eq!(AcpConnectionState::Handshaking.to_string(), "Handshaking");
    assert_eq!(AcpConnectionState::Connected.to_string(), "Connected");
    assert_eq!(
        AcpConnectionState::Failed("test".to_string()).to_string(),
        "Failed(test)"
    );
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