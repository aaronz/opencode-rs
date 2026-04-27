use opencode_core::acp::{AcpHandshakeAck, AcpHandshakeRequest, AcpProtocol};

#[test]
fn test_acp_transport_handshake_request_serialization() {
    let request = AcpHandshakeRequest {
        version: "1.0".to_string(),
        client_id: "editor-1".to_string(),
        capabilities: vec!["chat".to_string(), "tools".to_string()],
    };

    let json = serde_json::to_string(&request).expect("Should serialize");
    assert!(json.contains("\"version\":\"1.0\""));
    assert!(json.contains("\"client_id\":\"editor-1\""));
    assert!(json.contains("\"capabilities\""));
}

#[test]
fn test_acp_transport_handshake_response_serialization() {
    let response = opencode_core::acp::AcpHandshakeResponse {
        version: "1.0".to_string(),
        server_id: "server-1".to_string(),
        session_id: "session-123".to_string(),
        accepted: true,
        error: None,
    };

    let json = serde_json::to_string(&response).expect("Should serialize");
    assert!(json.contains("\"accepted\":true"));
    assert!(json.contains("\"session_id\":\"session-123\""));
}

#[test]
fn test_acp_transport_handshake_ack_serialization() {
    let ack = AcpHandshakeAck {
        session_id: "session-123".to_string(),
        confirmed: true,
    };

    let json = serde_json::to_string(&ack).expect("Should serialize");
    assert!(json.contains("\"session_id\":\"session-123\""));
    assert!(json.contains("\"confirmed\":true"));
}

#[test]
fn test_acp_protocol_process_handshake_success() {
    let protocol = AcpProtocol::new("server-1", "1.0");
    let request = AcpHandshakeRequest {
        version: "1.0".to_string(),
        client_id: "editor-1".to_string(),
        capabilities: vec!["chat".to_string()],
    };

    let response = protocol.process_handshake(request);
    assert!(response.accepted);
    assert!(response.error.is_none());
    assert_eq!(response.server_id, "server-1");
    assert!(!response.session_id.is_empty());
}

#[test]
fn test_acp_protocol_process_handshake_version_mismatch() {
    let protocol = AcpProtocol::new("server-1", "2.0");
    let request = AcpHandshakeRequest {
        version: "1.0".to_string(),
        client_id: "editor-1".to_string(),
        capabilities: vec![],
    };

    let response = protocol.process_handshake(request);
    assert!(!response.accepted);
    assert!(response.error.is_some());
    assert!(response.error.unwrap().contains("Version mismatch"));
}

#[test]
fn test_acp_protocol_confirm_handshake() {
    let protocol = AcpProtocol::new("server-1", "1.0");
    let ack = AcpHandshakeAck {
        session_id: "session-123".to_string(),
        confirmed: true,
    };

    assert!(protocol.confirm_handshake(ack));
}

#[test]
fn test_acp_protocol_confirm_handshake_negative() {
    let protocol = AcpProtocol::new("server-1", "1.0");
    let ack = AcpHandshakeAck {
        session_id: "".to_string(),
        confirmed: true,
    };

    assert!(!protocol.confirm_handshake(ack));
}

#[test]
fn test_acp_protocol_create_message() {
    let protocol = AcpProtocol::new("server-1", "1.0");
    let message = protocol.create_message(
        "editor_message",
        "editor-1",
        "server-1",
        serde_json::json!({"text": "Hello"}),
    );

    assert_eq!(message.message_type, "editor_message");
    assert_eq!(message.sender, "editor-1");
    assert_eq!(message.receiver, "server-1");
    assert_eq!(message.payload["text"], "Hello");
}

#[test]
fn test_acp_protocol_version_and_server_id() {
    let protocol = AcpProtocol::new("my-server", "1.5");
    assert_eq!(protocol.version(), "1.5");
    assert_eq!(protocol.server_id(), "my-server");
}

#[test]
fn test_acp_message_serialization_roundtrip() {
    use chrono::Utc;
    use opencode_core::acp::AcpMessage;

    let message = AcpMessage {
        id: "msg-123".to_string(),
        message_type: "chat".to_string(),
        sender: "client-1".to_string(),
        receiver: "server-1".to_string(),
        payload: serde_json::json!({"content": "test"}),
        timestamp: Utc::now(),
    };

    let json = serde_json::to_string(&message).expect("Should serialize");
    let deserialized: AcpMessage = serde_json::from_str(&json).expect("Should deserialize");

    assert_eq!(deserialized.id, message.id);
    assert_eq!(deserialized.message_type, message.message_type);
    assert_eq!(deserialized.sender, message.sender);
    assert_eq!(deserialized.receiver, message.receiver);
}
