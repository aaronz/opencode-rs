use opencode_tui::server_protocol::{
    ServerError, ServerErrorCode, ServerMessage, ServerRequest, ServerResponse,
};

#[test]
fn test_server_message_variant_creation() {
    let msg = ServerMessage::Message {
        session_id: "sess123".to_string(),
        content: "Hello".to_string(),
        role: "user".to_string(),
    };
    assert!(matches!(msg, ServerMessage::Message { .. }));
}

#[test]
fn test_server_message_tool_call() {
    let msg = ServerMessage::ToolCall {
        session_id: "sess123".to_string(),
        tool_name: "read".to_string(),
        args: serde_json::json!({"path": "test.txt"}),
        call_id: "call1".to_string(),
    };
    assert!(matches!(msg, ServerMessage::ToolCall { tool_name: _, .. }));
}

#[test]
fn test_server_message_tool_result() {
    let msg = ServerMessage::ToolResult {
        session_id: "sess123".to_string(),
        call_id: "call1".to_string(),
        output: "file contents".to_string(),
        success: true,
    };
    assert!(matches!(
        msg,
        ServerMessage::ToolResult { success: true, .. }
    ));
}

#[test]
fn test_server_message_session_update() {
    let msg = ServerMessage::SessionUpdate {
        session_id: "sess123".to_string(),
        status: "running".to_string(),
    };
    assert!(matches!(msg, ServerMessage::SessionUpdate { .. }));
}

#[test]
fn test_server_message_token_usage() {
    let msg = ServerMessage::TokenUsage {
        session_id: "sess123".to_string(),
        input_tokens: 100,
        output_tokens: 200,
        cache_tokens: Some(50),
        estimated_cost: Some(0.05),
    };
    assert!(matches!(msg, ServerMessage::TokenUsage { .. }));
}

#[test]
fn test_server_message_heartbeat() {
    let msg = ServerMessage::Heartbeat { timestamp: 123456 };
    assert!(matches!(
        msg,
        ServerMessage::Heartbeat { timestamp: 123456 }
    ));
}

#[test]
fn test_server_message_error() {
    let msg = ServerMessage::Error {
        session_id: Some("sess123".to_string()),
        code: "E001".to_string(),
        message: "Something went wrong".to_string(),
    };
    assert!(matches!(msg, ServerMessage::Error { .. }));
}

#[test]
fn test_server_message_connected() {
    let msg = ServerMessage::Connected {
        session_id: Some("sess123".to_string()),
    };
    assert!(matches!(msg, ServerMessage::Connected { .. }));
}

#[test]
fn test_server_message_ping() {
    let msg = ServerMessage::Ping { timestamp: 123456 };
    assert!(matches!(msg, ServerMessage::Ping { timestamp: 123456 }));
}

#[test]
fn test_server_message_serialization() {
    let msg = ServerMessage::Message {
        session_id: "sess123".to_string(),
        content: "Hello".to_string(),
        role: "user".to_string(),
    };
    let json = serde_json::to_string(&msg).unwrap();
    assert!(json.contains("message"));
    let parsed: ServerMessage = serde_json::from_str(&json).unwrap();
    assert!(matches!(parsed, ServerMessage::Message { .. }));
}

#[test]
fn test_server_request_run() {
    let req = ServerRequest::Run {
        id: "req1".to_string(),
        session_id: "sess123".to_string(),
        message: "Hello".to_string(),
        agent_type: Some("build".to_string()),
        model: Some("gpt-4".to_string()),
    };
    assert!(matches!(req, ServerRequest::Run { .. }));
}

#[test]
fn test_server_request_resume() {
    let req = ServerRequest::Resume {
        id: "req1".to_string(),
        session_id: "sess123".to_string(),
        token: "token123".to_string(),
    };
    assert!(matches!(req, ServerRequest::Resume { .. }));
}

#[test]
fn test_server_request_ping() {
    let req = ServerRequest::Ping {
        id: "req1".to_string(),
    };
    assert!(matches!(req, ServerRequest::Ping { .. }));
}

#[test]
fn test_server_request_close() {
    let req = ServerRequest::Close {
        id: "req1".to_string(),
    };
    assert!(matches!(req, ServerRequest::Close { .. }));
}

#[test]
fn test_server_request_serialization() {
    let req = ServerRequest::Run {
        id: "req1".to_string(),
        session_id: "sess123".to_string(),
        message: "Hello".to_string(),
        agent_type: None,
        model: None,
    };
    let json = serde_json::to_string(&req).unwrap();
    assert!(json.contains("run"));
    let parsed: ServerRequest = serde_json::from_str(&json).unwrap();
    assert!(matches!(parsed, ServerRequest::Run { .. }));
}

#[test]
fn test_server_response_ack() {
    let resp = ServerResponse::Ack {
        id: "req1".to_string(),
        status: "ok".to_string(),
    };
    assert!(matches!(resp, ServerResponse::Ack { .. }));
}

#[test]
fn test_server_response_error() {
    let resp = ServerResponse::Error {
        id: "req1".to_string(),
        code: "E001".to_string(),
        message: "Error message".to_string(),
    };
    assert!(matches!(resp, ServerResponse::Error { .. }));
}

#[test]
fn test_server_error_new() {
    let error = ServerError::new(ServerErrorCode::ParseError, "Parse failed");
    assert!(matches!(error.code, ServerErrorCode::ParseError));
    assert_eq!(error.message, "Parse failed");
    assert!(error.session_id.is_none());
}

#[test]
fn test_server_error_with_session() {
    let error = ServerError::new(ServerErrorCode::SessionNotFound, "Session not found")
        .with_session("sess123");
    assert_eq!(error.session_id, Some("sess123".to_string()));
}

#[test]
fn test_server_error_code_parse_error() {
    let code = ServerErrorCode::ParseError;
    assert!(matches!(code, ServerErrorCode::ParseError));
}

#[test]
fn test_server_error_code_invalid_request() {
    let code = ServerErrorCode::InvalidRequest;
    assert!(matches!(code, ServerErrorCode::InvalidRequest));
}

#[test]
fn test_server_error_code_session_not_found() {
    let code = ServerErrorCode::SessionNotFound;
    assert!(matches!(code, ServerErrorCode::SessionNotFound));
}

#[test]
fn test_server_error_code_session_load_error() {
    let code = ServerErrorCode::SessionLoadError;
    assert!(matches!(code, ServerErrorCode::SessionLoadError));
}

#[test]
fn test_server_error_code_invalid_reconnect_token() {
    let code = ServerErrorCode::InvalidReconnectToken;
    assert!(matches!(code, ServerErrorCode::InvalidReconnectToken));
}

#[test]
fn test_server_error_code_unsupported_binary() {
    let code = ServerErrorCode::UnsupportedBinary;
    assert!(matches!(code, ServerErrorCode::UnsupportedBinary));
}

#[test]
fn test_server_error_code_serialization_error() {
    let code = ServerErrorCode::SerializationError;
    assert!(matches!(code, ServerErrorCode::SerializationError));
}

#[test]
fn test_server_error_code_connection_timeout() {
    let code = ServerErrorCode::ConnectionTimeout;
    assert!(matches!(code, ServerErrorCode::ConnectionTimeout));
}

#[test]
fn test_server_error_code_heartbeat_timeout() {
    let code = ServerErrorCode::HeartbeatTimeout;
    assert!(matches!(code, ServerErrorCode::HeartbeatTimeout));
}

#[test]
fn test_server_error_code_max_reconnect_attempts() {
    let code = ServerErrorCode::MaxReconnectAttempts;
    assert!(matches!(code, ServerErrorCode::MaxReconnectAttempts));
}

#[test]
fn test_server_error_code_custom() {
    let code = ServerErrorCode::Custom("CUSTOM_ERROR".to_string());
    assert!(matches!(code, ServerErrorCode::Custom(ref s) if s == "CUSTOM_ERROR"));
}

#[test]
fn test_server_error_code_serialization() {
    let code = ServerErrorCode::ParseError;
    let json = serde_json::to_string(&code).unwrap();
    assert!(json.contains("parse_error"));
    let parsed: ServerErrorCode = serde_json::from_str(&json).unwrap();
    assert!(matches!(parsed, ServerErrorCode::ParseError));
}

#[test]
fn test_server_error_serialization() {
    let error = ServerError::new(ServerErrorCode::ParseError, "Parse failed");
    let json = serde_json::to_string(&error).unwrap();
    assert!(json.contains("parse_error"));
    let parsed: ServerError = serde_json::from_str(&json).unwrap();
    assert!(matches!(parsed.code, ServerErrorCode::ParseError));
}

#[test]
fn test_server_message_all_variants_serialize() {
    let variants = vec![
        ServerMessage::Message {
            session_id: "sess123".to_string(),
            content: "Hello".to_string(),
            role: "user".to_string(),
        },
        ServerMessage::ToolCall {
            session_id: "sess123".to_string(),
            tool_name: "read".to_string(),
            args: serde_json::json!({}),
            call_id: "call1".to_string(),
        },
        ServerMessage::ToolResult {
            session_id: "sess123".to_string(),
            call_id: "call1".to_string(),
            output: "ok".to_string(),
            success: true,
        },
        ServerMessage::SessionUpdate {
            session_id: "sess123".to_string(),
            status: "running".to_string(),
        },
        ServerMessage::TokenUsage {
            session_id: "sess123".to_string(),
            input_tokens: 100,
            output_tokens: 200,
            cache_tokens: None,
            estimated_cost: None,
        },
        ServerMessage::Heartbeat { timestamp: 123 },
        ServerMessage::Error {
            session_id: None,
            code: "E001".to_string(),
            message: "Error".to_string(),
        },
        ServerMessage::Connected { session_id: None },
        ServerMessage::Ping { timestamp: 123 },
    ];

    for msg in variants {
        let json = serde_json::to_string(&msg).unwrap();
        let _parsed: ServerMessage = serde_json::from_str(&json).unwrap();
        assert!(true, "Variant serialized and deserialized");
    }
}

#[test]
fn test_server_request_all_variants_serialize() {
    let variants = vec![
        ServerRequest::Run {
            id: "1".to_string(),
            session_id: "sess1".to_string(),
            message: "Hi".to_string(),
            agent_type: None,
            model: None,
        },
        ServerRequest::Resume {
            id: "1".to_string(),
            session_id: "sess1".to_string(),
            token: "tok".to_string(),
        },
        ServerRequest::Ping {
            id: "1".to_string(),
        },
        ServerRequest::Close {
            id: "1".to_string(),
        },
    ];

    for req in variants {
        let json = serde_json::to_string(&req).unwrap();
        let _parsed: ServerRequest = serde_json::from_str(&json).unwrap();
        assert!(true, "Variant serialized and deserialized");
    }
}

#[test]
fn test_server_error_code_clone() {
    let code1 = ServerErrorCode::ParseError;
    let code2 = code1.clone();
    assert!(matches!(code1, ServerErrorCode::ParseError));
    assert!(matches!(code2, ServerErrorCode::ParseError));
}

#[test]
fn test_server_error_clone() {
    let error1 = ServerError::new(ServerErrorCode::ParseError, "Error");
    let error2 = error1.clone();
    assert_eq!(error1.message, error2.message);
}

#[test]
fn test_server_message_clone() {
    let msg1 = ServerMessage::Ping { timestamp: 123 };
    let msg2 = msg1.clone();
    assert!(matches!(msg2, ServerMessage::Ping { timestamp: 123 }));
}

#[test]
fn test_server_request_clone() {
    let req1 = ServerRequest::Ping {
        id: "1".to_string(),
    };
    let req2 = req1.clone();
    assert!(matches!(req2, ServerRequest::Ping { .. }));
}

#[test]
fn test_server_response_clone() {
    let resp1 = ServerResponse::Ack {
        id: "1".to_string(),
        status: "ok".to_string(),
    };
    let resp2 = resp1.clone();
    assert!(matches!(resp2, ServerResponse::Ack { .. }));
}
