mod types;

pub use types::{AcpHandshakeAck, AcpHandshakeRequest, AcpHandshakeResponse, AcpMessage, AcpProtocol};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_handshake_request_version_mismatch() {
        let protocol = AcpProtocol::new("server1", "2.0");
        let request = AcpHandshakeRequest {
            version: "1.0".to_string(),
            client_id: "client1".to_string(),
            capabilities: vec!["chat".to_string()],
        };

        let response = protocol.process_handshake(request);
        assert!(!response.accepted);
        assert!(response.error.is_some());
        assert!(response.error.unwrap().contains("Version mismatch"));
    }

    #[test]
    fn test_handshake_request_success() {
        let protocol = AcpProtocol::new("server1", "1.0");
        let request = AcpHandshakeRequest {
            version: "1.0".to_string(),
            client_id: "client1".to_string(),
            capabilities: vec!["chat".to_string()],
        };

        let response = protocol.process_handshake(request);
        assert!(response.accepted);
        assert!(response.error.is_none());
        assert!(!response.session_id.is_empty());
        assert_eq!(response.server_id, "server1");
    }

    #[test]
    fn test_create_message() {
        let protocol = AcpProtocol::new("server1", "1.0");
        let message = protocol.create_message(
            "chat",
            "client1",
            "server1",
            serde_json::json!({"text": "hello"}),
        );

        assert_eq!(message.message_type, "chat");
        assert_eq!(message.sender, "client1");
        assert_eq!(message.receiver, "server1");
        assert_eq!(message.payload["text"], "hello");
    }

    #[test]
    fn test_confirm_handshake() {
        let protocol = AcpProtocol::new("server1", "1.0");
        let ack = AcpHandshakeAck {
            session_id: "session123".to_string(),
            confirmed: true,
        };
        assert!(protocol.confirm_handshake(ack));
    }

    #[test]
    fn test_confirm_handshake_empty_session_id() {
        let protocol = AcpProtocol::new("server1", "1.0");
        let ack = AcpHandshakeAck {
            session_id: "".to_string(),
            confirmed: true,
        };
        assert!(!protocol.confirm_handshake(ack));
    }

    #[test]
    fn test_confirm_handshake_not_confirmed() {
        let protocol = AcpProtocol::new("server1", "1.0");
        let ack = AcpHandshakeAck {
            session_id: "session123".to_string(),
            confirmed: false,
        };
        assert!(!protocol.confirm_handshake(ack));
    }

    #[test]
    fn test_acp_protocol_accessors() {
        let protocol = AcpProtocol::new("server1", "1.0");
        assert_eq!(protocol.version(), "1.0");
        assert_eq!(protocol.server_id(), "server1");
    }

    #[test]
    fn test_acp_protocol_register_and_handle() {
        use std::sync::{Arc, Mutex};
        let protocol = Arc::new(Mutex::new(AcpProtocol::new("server1", "1.0")));
        let received = Arc::new(Mutex::new(None));
        let received_clone = received.clone();
        let protocol_clone = protocol.clone();
        protocol_clone.lock().unwrap().register_handler(
            "test_message".to_string(),
            Box::new(move |msg| {
                *received_clone.lock().unwrap() = Some(msg);
            }),
        );
        let msg = protocol_clone.lock().unwrap().create_message(
            "test_message",
            "client1",
            "server1",
            serde_json::json!({"text": "hello"}),
        );
        protocol_clone.lock().unwrap().handle(msg);
        let received = received.lock().unwrap();
        assert!(received.is_some());
        assert_eq!(received.as_ref().unwrap().payload["text"], "hello");
    }
}