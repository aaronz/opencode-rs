use chrono::Utc;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AcpStatus {
    pub connected: bool,
    pub client_id: Option<String>,
    pub capabilities: Vec<String>,
    pub server_url: Option<String>,
    pub version: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HandshakeRequest {
    pub client_id: String,
    pub capabilities: Vec<String>,
    pub version: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HandshakeResponse {
    pub server_id: String,
    pub accepted_capabilities: Vec<String>,
    pub session_token: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConnectRequest {
    pub url: String,
    pub client_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AckRequest {
    pub handshake_id: String,
    pub accepted: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AcpMessage {
    pub from: String,
    pub to: String,
    pub message_type: String,
    pub payload: serde_json::Value,
    pub timestamp: i64,
}

impl AcpMessage {
    pub fn new(from: String, to: String, message_type: String, payload: serde_json::Value) -> Self {
        Self {
            from,
            to,
            message_type,
            payload,
            timestamp: Utc::now().timestamp(),
        }
    }
}
