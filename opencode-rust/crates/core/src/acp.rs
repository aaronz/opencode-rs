use chrono::Utc;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AcpMessage {
    pub id: String,
    pub message_type: String,
    pub sender: String,
    pub receiver: String,
    pub payload: serde_json::Value,
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AcpHandshakeRequest {
    pub version: String,
    pub client_id: String,
    pub capabilities: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AcpHandshakeResponse {
    pub version: String,
    pub server_id: String,
    pub session_id: String,
    pub accepted: bool,
    pub error: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AcpHandshakeAck {
    pub session_id: String,
    pub confirmed: bool,
}

pub struct AcpProtocol {
    handlers: HashMap<String, Box<dyn Fn(AcpMessage) + Send + Sync>>,
    server_id: String,
    version: String,
}

impl AcpProtocol {
    pub fn new(server_id: &str, version: &str) -> Self {
        Self {
            handlers: HashMap::new(),
            server_id: server_id.to_string(),
            version: version.to_string(),
        }
    }

    pub fn register_handler(
        &mut self,
        message_type: String,
        handler: Box<dyn Fn(AcpMessage) + Send + Sync>,
    ) {
        self.handlers.insert(message_type, handler);
    }

    pub fn handle(&self, message: AcpMessage) {
        if let Some(handler) = self.handlers.get(&message.message_type) {
            handler(message);
        }
    }

    pub fn process_handshake(&self, request: AcpHandshakeRequest) -> AcpHandshakeResponse {
        if request.version != self.version {
            return AcpHandshakeResponse {
                version: self.version.clone(),
                server_id: self.server_id.clone(),
                session_id: String::new(),
                accepted: false,
                error: Some(format!(
                    "Version mismatch: expected {}, got {}",
                    self.version, request.version
                )),
            };
        }

        let session_id = format!("{}-{}", self.server_id, Utc::now().timestamp());

        AcpHandshakeResponse {
            version: self.version.clone(),
            server_id: self.server_id.clone(),
            session_id,
            accepted: true,
            error: None,
        }
    }

    pub fn confirm_handshake(&self, ack: AcpHandshakeAck) -> bool {
        !ack.session_id.is_empty() && ack.confirmed
    }

    pub fn create_message(
        &self,
        message_type: &str,
        sender: &str,
        receiver: &str,
        payload: serde_json::Value,
    ) -> AcpMessage {
        AcpMessage {
            id: format!("{}-{}", sender, Utc::now().timestamp_millis()),
            message_type: message_type.to_string(),
            sender: sender.to_string(),
            receiver: receiver.to_string(),
            payload,
            timestamp: Utc::now(),
        }
    }

    pub fn version(&self) -> &str {
        &self.version
    }

    pub fn server_id(&self) -> &str {
        &self.server_id
    }
}

impl Default for AcpProtocol {
    fn default() -> Self {
        Self::new("unknown", "1.0")
    }
}

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
