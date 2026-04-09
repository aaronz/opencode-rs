use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ServerMessage {
    Message {
        session_id: String,
        content: String,
        role: String,
    },
    ToolCall {
        session_id: String,
        tool_name: String,
        args: serde_json::Value,
        call_id: String,
    },
    ToolResult {
        session_id: String,
        call_id: String,
        output: String,
        success: bool,
    },
    SessionUpdate {
        session_id: String,
        status: String,
    },
    TokenUsage {
        session_id: String,
        input_tokens: u64,
        output_tokens: u64,
        cache_tokens: Option<u64>,
        estimated_cost: Option<f64>,
    },
    Heartbeat {
        timestamp: i64,
    },
    Error {
        session_id: Option<String>,
        code: String,
        message: String,
    },
    Connected {
        session_id: Option<String>,
    },
    Ping {
        timestamp: i64,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ServerRequest {
    Run {
        id: String,
        session_id: String,
        message: String,
        agent_type: Option<String>,
        model: Option<String>,
    },
    Resume {
        id: String,
        session_id: String,
        token: String,
    },
    Ping {
        id: String,
    },
    Close {
        id: String,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ServerResponse {
    Ack {
        id: String,
        status: String,
    },
    Error {
        id: String,
        code: String,
        message: String,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerError {
    pub code: ServerErrorCode,
    pub message: String,
    pub session_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ServerErrorCode {
    ParseError,
    InvalidRequest,
    SessionNotFound,
    SessionLoadError,
    InvalidReconnectToken,
    UnsupportedBinary,
    SerializationError,
    ConnectionTimeout,
    HeartbeatTimeout,
    MaxReconnectAttempts,
    Custom(String),
}

impl ServerError {
    pub fn new(code: ServerErrorCode, message: impl Into<String>) -> Self {
        Self {
            code,
            message: message.into(),
            session_id: None,
        }
    }

    pub fn with_session(mut self, session_id: impl Into<String>) -> Self {
        self.session_id = Some(session_id.into());
        self
    }
}
