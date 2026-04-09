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

pub struct AcpProtocol {
    handlers: HashMap<String, Box<dyn Fn(AcpMessage) + Send + Sync>>,
}

impl AcpProtocol {
    pub fn new() -> Self {
        Self {
            handlers: HashMap::new(),
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
}

impl Default for AcpProtocol {
    fn default() -> Self {
        Self::new()
    }
}
