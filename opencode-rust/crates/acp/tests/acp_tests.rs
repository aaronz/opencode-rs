use std::sync::Arc;
use opencode_acp::{
    AcpClient, AcpConnectionState, AcpError, AcpMessage, AcpState, AcpStatus, HandshakeRequest,
};

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

    let client = AcpClient::new(http, client_id.clone(), bus.clone());

    assert_eq!(client.connection_state(), AcpConnectionState::Disconnected);
    let status = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .unwrap()
        .block_on(client.status())
        .unwrap();
    assert!(!status.connected);
    assert_eq!(status.client_id, Some(client_id));
}

#[test]
fn test_acp_client_field_access_via_public_methods() {
    let http = reqwest::Client::new();
    let bus: opencode_core::bus::SharedEventBus = Arc::new(opencode_core::bus::EventBus::new());
    let client_id = "field-access-test".to_string();

    let client = AcpClient::new(http, client_id.clone(), bus);

    let state = client.connection_state();
    assert!(matches!(state, AcpConnectionState::Disconnected));

    let status = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .unwrap()
        .block_on(client.status())
        .unwrap();
    assert_eq!(status.client_id, Some(client_id));
    assert!(!status.connected);
    assert!(status.capabilities.is_empty());
    assert!(status.server_url.is_none());
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
async fn test_disconnect_cleans_up_resources() {
    let client = create_test_client();

    {
        let mut state = client.state().lock().unwrap();
        state.connection_state = AcpConnectionState::Connected;
        state.server_id = Some("srv-123".to_string());
        state.session_token = Some("tok-456".to_string());
        state.capabilities = vec!["chat".to_string(), "tasks".to_string()];
        state.server_url = Some("http://localhost:8080".to_string());
    }

    client.disconnect().await.unwrap();

    let state = client.state().lock().unwrap();
    assert!(matches!(state.connection_state, AcpConnectionState::Disconnected));
    assert_eq!(state.server_id, None);
    assert_eq!(state.session_token, None);
    assert_eq!(state.server_url, None);
}

#[tokio::test]
async fn test_disconnect_from_handshaking_state() {
    use wiremock::{Mock, MockServer, ResponseTemplate};

    let mock_server = MockServer::start().await;

    Mock::given(wiremock::matchers::method("POST"))
        .and(wiremock::matchers::path("/api/acp/handshake"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "server_id": "srv1",
            "accepted_capabilities": ["chat"],
            "session_token": "tok1"
        })))
        .mount(&mock_server)
        .await;

    let client = create_test_client();

    {
        let mut state = client.state().lock().unwrap();
        state.connection_state = AcpConnectionState::Handshaking;
        state.server_url = Some(mock_server.uri());
    }

    let result = client.disconnect().await;
    assert!(result.is_ok());
    assert_eq!(client.connection_state(), AcpConnectionState::Disconnected);
}

#[tokio::test]
async fn test_disconnect_from_connected_state() {
    use wiremock::{Mock, MockServer, ResponseTemplate};

    let mock_server = MockServer::start().await;

    Mock::given(wiremock::matchers::method("POST"))
        .and(wiremock::matchers::path("/api/acp/handshake"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "server_id": "srv1",
            "accepted_capabilities": ["chat"],
            "session_token": "tok1"
        })))
        .expect(1)
        .mount(&mock_server)
        .await;

    let client = create_test_client();
    client.connect(&mock_server.uri(), Some("my-client".to_string())).await.unwrap();
    assert_eq!(client.connection_state(), AcpConnectionState::Connected);

    client.disconnect().await.unwrap();
    assert_eq!(client.connection_state(), AcpConnectionState::Disconnected);

    let state = client.state().lock().unwrap();
    assert_eq!(state.server_id, None);
    assert_eq!(state.session_token, None);
}

#[tokio::test]
async fn test_disconnect_from_failed_state() {
    let client = create_test_client();

    {
        let mut state = client.state().lock().unwrap();
        state.connection_state = AcpConnectionState::Failed("connection refused".to_string());
        state.server_id = Some("srv-123".to_string());
        state.session_token = Some("tok-456".to_string());
        state.server_url = Some("http://localhost:8080".to_string());
    }

    let result = client.disconnect().await;
    assert!(result.is_ok());
    assert_eq!(client.connection_state(), AcpConnectionState::Disconnected);

    let state = client.state().lock().unwrap();
    assert_eq!(state.server_id, None);
    assert_eq!(state.session_token, None);
    assert_eq!(state.server_url, None);
}

#[tokio::test]
async fn test_disconnect_from_already_disconnected_is_idempotent() {
    let client = create_test_client();

    let result1 = client.disconnect().await;
    let result2 = client.disconnect().await;

    assert!(result1.is_ok());
    assert!(result2.is_ok());
    assert_eq!(client.connection_state(), AcpConnectionState::Disconnected);
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

#[test]
fn test_acp_error_all_variants_exist() {
    let _ = AcpError::NotConnected;
    let _ = AcpError::HandshakeFailed("test".to_string());
    let _ = AcpError::ConnectionFailed("test".to_string());
    let _ = AcpError::ServerError("test".to_string());
    let _ = AcpError::InvalidResponse("test".to_string());
    let _ = AcpError::State("test".to_string());
}

#[test]
fn test_acp_error_display_not_connected() {
    let err = AcpError::NotConnected;
    assert_eq!(err.to_string(), "Not connected");
}

#[test]
fn test_acp_error_display_handshake_failed() {
    let err = AcpError::HandshakeFailed("timeout".to_string());
    assert_eq!(err.to_string(), "Handshake failed: timeout");
}

#[test]
fn test_acp_error_display_connection_failed() {
    let err = AcpError::ConnectionFailed("refused".to_string());
    assert_eq!(err.to_string(), "Connection failed: refused");
}

#[test]
fn test_acp_error_display_server_error() {
    let err = AcpError::ServerError("internal error".to_string());
    assert_eq!(err.to_string(), "Server returned error: internal error");
}

#[test]
fn test_acp_error_display_invalid_response() {
    let err = AcpError::InvalidResponse("malformed json".to_string());
    assert_eq!(err.to_string(), "Invalid response: malformed json");
}

#[test]
fn test_acp_error_display_state() {
    let err = AcpError::State("lock poisoned".to_string());
    assert_eq!(err.to_string(), "State error: lock poisoned");
}

#[test]
fn test_acp_error_http_variant_supports_from_trait() {
    fn _assert_from<T: From<reqwest::Error>>() {}
    _assert_from::<AcpError>();
    let _err: AcpError = AcpError::NotConnected;
}

#[test]
fn test_acp_status_instantiation() {
    let status = AcpStatus {
        connected: true,
        client_id: Some("client-abc".to_string()),
        capabilities: vec!["chat".to_string(), "tasks".to_string()],
        server_url: Some("http://localhost:8080".to_string()),
    };
    assert!(status.connected);
    assert_eq!(status.client_id, Some("client-abc".to_string()));
    assert_eq!(status.capabilities.len(), 2);
    assert_eq!(status.server_url, Some("http://localhost:8080".to_string()));
}

#[test]
fn test_acp_status_serialize_deserialize() {
    let status = AcpStatus {
        connected: true,
        client_id: Some("client-xyz".to_string()),
        capabilities: vec!["files".to_string(), "search".to_string()],
        server_url: Some("https://acp.example.com".to_string()),
    };

    let json = serde_json::to_string(&status).unwrap();
    assert!(json.contains("\"connected\":true"));
    assert!(json.contains("\"client_id\":\"client-xyz\""));
    assert!(json.contains("\"capabilities\""));
    assert!(json.contains("\"server_url\""));

    let deserialized: AcpStatus = serde_json::from_str(&json).unwrap();
    assert!(deserialized.connected);
    assert_eq!(deserialized.client_id, Some("client-xyz".to_string()));
    assert_eq!(deserialized.capabilities, vec!["files".to_string(), "search".to_string()]);
    assert_eq!(deserialized.server_url, Some("https://acp.example.com".to_string()));
}

#[test]
fn test_acp_status_disconnected_state() {
    let status = AcpStatus {
        connected: false,
        client_id: None,
        capabilities: Vec::new(),
        server_url: None,
    };
    assert!(!status.connected);
    assert!(status.client_id.is_none());
    assert!(status.capabilities.is_empty());
    assert!(status.server_url.is_none());

    let json = serde_json::to_string(&status).unwrap();
    assert!(json.contains("\"connected\":false"));
    assert!(json.contains("\"client_id\":null"));
    assert!(json.contains("\"capabilities\":[]"));
    assert!(json.contains("\"server_url\":null"));
}

#[tokio::test]
async fn test_connect_transitions_state_from_disconnected_to_handshaking() {
    use wiremock::{Mock, MockServer, ResponseTemplate};

    let mock_server = MockServer::start().await;

    Mock::given(wiremock::matchers::method("POST"))
        .and(wiremock::matchers::path("/api/acp/handshake"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "server_id": "srv1",
            "accepted_capabilities": ["chat"],
            "session_token": "tok1"
        })))
        .expect(1)
        .mount(&mock_server)
        .await;

    let client = create_test_client();
    assert_eq!(client.connection_state(), AcpConnectionState::Disconnected);

    let client_clone = client.clone();
    let uri = mock_server.uri();
    let handle = tokio::spawn(async move {
        client_clone.connect(&uri, Some("my-client".to_string())).await
    });

    tokio::time::sleep(std::time::Duration::from_millis(100)).await;

    let state_after_delay = client.connection_state();
    assert!(
        state_after_delay == AcpConnectionState::Handshaking ||
        state_after_delay == AcpConnectionState::Connected,
        "State should be Handshaking or Connected during/after connection, got {:?}",
        state_after_delay
    );

    let _ = handle.await.unwrap();
    assert_eq!(client.connection_state(), AcpConnectionState::Connected);
}

#[tokio::test]
async fn test_connect_transitions_state_to_connected_on_success() {
    use wiremock::{Mock, MockServer, ResponseTemplate};

    let mock_server = MockServer::start().await;

    Mock::given(wiremock::matchers::method("POST"))
        .and(wiremock::matchers::path("/api/acp/handshake"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "server_id": "srv1",
            "accepted_capabilities": ["chat"],
            "session_token": "tok1"
        })))
        .expect(1)
        .mount(&mock_server)
        .await;

    let client = create_test_client();
    let uri = mock_server.uri();

    client.connect(&uri, Some("my-client".to_string())).await.unwrap();

    assert_eq!(client.connection_state(), AcpConnectionState::Connected);
}

#[tokio::test]
async fn test_connect_handles_connection_failures() {
    use wiremock::{MockServer};

    let mock_server = MockServer::start().await;

    let client = create_test_client();
    let uri = mock_server.uri();

    let result = client.connect(&uri, Some("my-client".to_string())).await;

    assert!(result.is_err());
    assert_eq!(client.connection_state(), AcpConnectionState::Disconnected);
}

#[test]
fn test_acp_status_roundtrip() {
    let status = AcpStatus {
        connected: true,
        client_id: Some("test-client".to_string()),
        capabilities: vec!["chat".to_string()],
        server_url: Some("http://127.0.0.1:3000".to_string()),
    };

    let json = serde_json::to_string(&status).unwrap();
    let roundtrip: AcpStatus = serde_json::from_str(&json).unwrap();

    assert_eq!(roundtrip.connected, status.connected);
    assert_eq!(roundtrip.client_id, status.client_id);
    assert_eq!(roundtrip.capabilities, status.capabilities);
    assert_eq!(roundtrip.server_url, status.server_url);
}

#[tokio::test]
async fn test_handshake_sends_correct_request() {
    use wiremock::{Mock, MockServer, ResponseTemplate, matchers::{method, path, body_json}};

    let mock_server = MockServer::start().await;

    let expected_request = HandshakeRequest {
        client_id: "my-client-id".to_string(),
        capabilities: vec!["chat".to_string(), "tasks".to_string()],
        version: "1.0".to_string(),
    };

    Mock::given(method("POST"))
        .and(path("/api/acp/handshake"))
        .and(body_json(&expected_request))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "server_id": "server-123",
            "accepted_capabilities": ["chat", "tasks"],
            "session_token": "session-abc"
        })))
        .expect(1)
        .mount(&mock_server)
        .await;

    let client = create_test_client();
    let result = client
        .handshake(&mock_server.uri(), "my-client-id".to_string(), vec!["chat".to_string(), "tasks".to_string()])
        .await;

    assert!(result.is_ok());
    let response = result.unwrap();
    assert_eq!(response.server_id, "server-123");
    assert_eq!(response.accepted_capabilities, vec!["chat", "tasks"]);
    assert_eq!(response.session_token, Some("session-abc".to_string()));
}

#[tokio::test]
async fn test_handshake_parses_handshake_response_correctly() {
    use wiremock::{Mock, MockServer, ResponseTemplate, matchers::{method, path}};

    let mock_server = MockServer::start().await;

    Mock::given(method("POST"))
        .and(path("/api/acp/handshake"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "server_id": "remote-server-xyz",
            "accepted_capabilities": ["chat", "files", "search"],
            "session_token": "token-12345"
        })))
        .expect(1)
        .mount(&mock_server)
        .await;

    let client = create_test_client();
    let response = client
        .handshake(&mock_server.uri(), "client-abc".to_string(), vec!["chat".to_string()])
        .await
        .unwrap();

    assert_eq!(response.server_id, "remote-server-xyz");
    assert_eq!(response.accepted_capabilities, vec!["chat", "files", "search"]);
    assert_eq!(response.session_token, Some("token-12345".to_string()));
}

#[tokio::test]
async fn test_handshake_fails_when_server_returns_error() {
    use wiremock::{Mock, MockServer, ResponseTemplate, matchers::{method, path}};

    let mock_server = MockServer::start().await;

    Mock::given(method("POST"))
        .and(path("/api/acp/handshake"))
        .respond_with(ResponseTemplate::new(500).set_body_string("Internal Server Error"))
        .expect(1)
        .mount(&mock_server)
        .await;

    let client = create_test_client();
    let result = client
        .handshake(&mock_server.uri(), "my-client".to_string(), vec!["chat".to_string()])
        .await;

    assert!(result.is_err());
    assert!(matches!(result.unwrap_err(), AcpError::ServerError(_)));
}

#[tokio::test]
async fn test_handshake_fails_on_invalid_response() {
    use wiremock::{Mock, MockServer, ResponseTemplate, matchers::{method, path}};

    let mock_server = MockServer::start().await;

    Mock::given(method("POST"))
        .and(path("/api/acp/handshake"))
        .respond_with(ResponseTemplate::new(200).set_body_string("not valid json"))
        .expect(1)
        .mount(&mock_server)
        .await;

    let client = create_test_client();
    let result = client
        .handshake(&mock_server.uri(), "my-client".to_string(), vec!["chat".to_string()])
        .await;

    assert!(result.is_err());
    assert!(matches!(result.unwrap_err(), AcpError::InvalidResponse(_)));
}

#[tokio::test]
async fn test_ack_sends_correct_request() {
    use wiremock::{Mock, MockServer, ResponseTemplate, matchers::{method, path, body_json}};
    use opencode_acp::AckRequest;

    let mock_server = MockServer::start().await;

    let expected_request = AckRequest {
        handshake_id: "handshake-456".to_string(),
        accepted: true,
    };

    Mock::given(method("POST"))
        .and(path("/api/acp/ack"))
        .and(body_json(&expected_request))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({ "ok": true })))
        .expect(1)
        .mount(&mock_server)
        .await;

    let client = create_test_client();

    {
        let mut state = client.state().lock().unwrap();
        state.connection_state = opencode_acp::AcpConnectionState::Connected;
        state.server_url = Some(mock_server.uri());
    }

    let result = client.ack("handshake-456", true).await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_ack_returns_success_response() {
    use wiremock::{Mock, MockServer, ResponseTemplate, matchers::{method, path}};

    let mock_server = MockServer::start().await;

    Mock::given(method("POST"))
        .and(path("/api/acp/ack"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({ "ok": true })))
        .expect(1)
        .mount(&mock_server)
        .await;

    let client = create_test_client();

    {
        let mut state = client.state().lock().unwrap();
        state.connection_state = opencode_acp::AcpConnectionState::Connected;
        state.server_url = Some(mock_server.uri());
    }

    let result = client.ack("handshake-789", false).await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_send_message_sends_successfully_when_connected() {
    use wiremock::{Mock, MockServer, ResponseTemplate, matchers::{method, path}};

    let mock_server = MockServer::start().await;

    Mock::given(method("POST"))
        .and(path("/api/acp/message"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({ "ok": true })))
        .expect(1)
        .mount(&mock_server)
        .await;

    let client = create_test_client();

    {
        let mut state = client.state().lock().unwrap();
        state.connection_state = opencode_acp::AcpConnectionState::Connected;
        state.server_url = Some(mock_server.uri());
    }

    let result = client
        .send_message("srv", "chat", serde_json::json!({"text": "hello"}))
        .await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_send_message_handles_network_errors() {
    let client = create_test_client();

    {
        let mut state = client.state().lock().unwrap();
        state.connection_state = opencode_acp::AcpConnectionState::Connected;
        state.server_url = Some("http://localhost:1".to_string());
    }

    let result = client
        .send_message("srv", "chat", serde_json::json!({"text": "hello"}))
        .await;
    assert!(result.is_err());
    assert!(matches!(result.unwrap_err(), AcpError::Http(_)));
}