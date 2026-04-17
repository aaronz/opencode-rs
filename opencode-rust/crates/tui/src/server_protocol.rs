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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_server_error_new() {
        let error = ServerError::new(ServerErrorCode::ParseError, "Invalid input");
        assert!(matches!(error.code, ServerErrorCode::ParseError));
        assert_eq!(error.message, "Invalid input");
        assert!(error.session_id.is_none());
    }

    #[test]
    fn test_server_error_with_session() {
        let error = ServerError::new(ServerErrorCode::SessionNotFound, "Session missing")
            .with_session("session-123");
        assert!(matches!(error.code, ServerErrorCode::SessionNotFound));
        assert_eq!(error.session_id, Some("session-123".to_string()));
    }

    #[test]
    fn test_server_error_code_variants() {
        assert!(matches!(
            ServerErrorCode::ParseError,
            ServerErrorCode::ParseError
        ));
        assert!(matches!(
            ServerErrorCode::InvalidRequest,
            ServerErrorCode::InvalidRequest
        ));
        assert!(matches!(
            ServerErrorCode::SessionNotFound,
            ServerErrorCode::SessionNotFound
        ));
        let custom_code = ServerErrorCode::Custom("test".to_string());
        assert!(matches!(custom_code, ServerErrorCode::Custom(_)));
    }

    #[test]
    fn test_server_message_serialization() {
        let msg = ServerMessage::Message {
            session_id: "sess-1".to_string(),
            content: "Hello".to_string(),
            role: "user".to_string(),
        };
        let json = serde_json::to_string(&msg).unwrap();
        assert!(json.contains("\"type\":\"message\""));
        assert!(json.contains("\"session_id\":\"sess-1\""));
        assert!(json.contains("\"content\":\"Hello\""));
    }

    #[test]
    fn test_server_message_deserialization() {
        let json = r#"{"type":"message","session_id":"sess-2","content":"Hi","role":"assistant"}"#;
        let msg: ServerMessage = serde_json::from_str(json).unwrap();
        match msg {
            ServerMessage::Message {
                session_id,
                content,
                role,
            } => {
                assert_eq!(session_id, "sess-2");
                assert_eq!(content, "Hi");
                assert_eq!(role, "assistant");
            }
            _ => panic!("Expected Message variant"),
        }
    }

    #[test]
    fn test_server_request_run_serialization() {
        let req = ServerRequest::Run {
            id: "req-1".to_string(),
            session_id: "sess-1".to_string(),
            message: "test".to_string(),
            agent_type: Some("code".to_string()),
            model: None,
        };
        let json = serde_json::to_string(&req).unwrap();
        assert!(json.contains("\"type\":\"run\""));
        assert!(json.contains("\"id\":\"req-1\""));
    }

    #[test]
    fn test_server_response_ack_deserialization() {
        let json = r#"{"type":"ack","id":"req-1","status":"ok"}"#;
        let resp: ServerResponse = serde_json::from_str(json).unwrap();
        match resp {
            ServerResponse::Ack { id, status } => {
                assert_eq!(id, "req-1");
                assert_eq!(status, "ok");
            }
            _ => panic!("Expected Ack variant"),
        }
    }

    #[test]
    fn test_server_error_custom_code() {
        let error = ServerError::new(ServerErrorCode::Custom("E123".to_string()), "Custom error");
        match error.code {
            ServerErrorCode::Custom(code) => assert_eq!(code, "E123"),
            _ => panic!("Expected Custom variant"),
        }
    }

    #[test]
    fn test_server_message_tool_call() {
        let msg = ServerMessage::ToolCall {
            session_id: "sess-1".to_string(),
            tool_name: "read_file".to_string(),
            args: serde_json::json!({"path": "/tmp/test.txt"}),
            call_id: "call-1".to_string(),
        };
        let json = serde_json::to_string(&msg).unwrap();
        assert!(json.contains("\"type\":\"tool_call\""));
        assert!(json.contains("\"tool_name\":\"read_file\""));
    }

    #[test]
    fn test_server_message_token_usage() {
        let msg = ServerMessage::TokenUsage {
            session_id: "sess-1".to_string(),
            input_tokens: 100,
            output_tokens: 200,
            cache_tokens: Some(50),
            estimated_cost: Some(0.05),
        };
        let json = serde_json::to_string(&msg).unwrap();
        assert!(json.contains("\"type\":\"token_usage\""));
        assert!(json.contains("\"input_tokens\":100"));
    }

    #[test]
    fn test_server_request_close_serialization() {
        let req = ServerRequest::Close {
            id: "close-1".to_string(),
        };
        let json = serde_json::to_string(&req).unwrap();
        assert!(json.contains("\"type\":\"close\""));
    }

    #[test]
    fn test_server_error_code_serialization() {
        let code = ServerErrorCode::ConnectionTimeout;
        let json = serde_json::to_string(&code).unwrap();
        assert_eq!(json, "\"connection_timeout\"");
    }
}
