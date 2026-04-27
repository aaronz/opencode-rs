use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AcpMessage {
    pub id: String,
    pub message_type: String,
    pub sender: String,
    pub receiver: String,
    pub payload: serde_json::Value,
    pub timestamp: DateTime<Utc>,
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