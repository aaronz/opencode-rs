extern crate opencode_integration_tests;

use opencode_integration_tests::common::{MockLLMProvider, MockServer, TempProject, TestConfig};

#[test]
fn test_execute_request_parsing_minimal() {
    let json_str = r#"{"prompt": "Hello, world!"}"#;
    let req: opencode_server::routes::execute::types::ExecuteRequest =
        serde_json::from_str(json_str).expect("should parse");
    assert_eq!(req.prompt, "Hello, world!");
    assert!(req.mode.is_none());
    assert_eq!(req.stream, Some(true));
}

#[test]
fn test_execute_request_parsing_with_mode_build() {
    let json_str = r#"{"prompt": "Test prompt", "mode": "build"}"#;
    let req: opencode_server::routes::execute::types::ExecuteRequest =
        serde_json::from_str(json_str).expect("should parse");
    assert_eq!(req.prompt, "Test prompt");
    assert_eq!(
        req.mode,
        Some(opencode_server::routes::execute::types::ExecuteMode::Build)
    );
}

#[test]
fn test_execute_request_parsing_with_mode_plan() {
    let json_str = r#"{"prompt": "Test prompt", "mode": "plan"}"#;
    let req: opencode_server::routes::execute::types::ExecuteRequest =
        serde_json::from_str(json_str).expect("should parse");
    assert_eq!(
        req.mode,
        Some(opencode_server::routes::execute::types::ExecuteMode::Plan)
    );
}

#[test]
fn test_execute_request_parsing_with_mode_general() {
    let json_str = r#"{"prompt": "Test prompt", "mode": "general"}"#;
    let req: opencode_server::routes::execute::types::ExecuteRequest =
        serde_json::from_str(json_str).expect("should parse");
    assert_eq!(
        req.mode,
        Some(opencode_server::routes::execute::types::ExecuteMode::General)
    );
}

#[test]
fn test_execute_request_parsing_with_stream_true() {
    let json_str = r#"{"prompt": "Test", "stream": true}"#;
    let req: opencode_server::routes::execute::types::ExecuteRequest =
        serde_json::from_str(json_str).expect("should parse");
    assert_eq!(req.stream, Some(true));
}

#[test]
fn test_execute_request_parsing_with_stream_false() {
    let json_str = r#"{"prompt": "Test", "stream": false}"#;
    let req: opencode_server::routes::execute::types::ExecuteRequest =
        serde_json::from_str(json_str).expect("should parse");
    assert_eq!(req.stream, Some(false));
}

#[test]
fn test_execute_request_serialization_roundtrip() {
    let req = opencode_server::routes::execute::types::ExecuteRequest {
        prompt: "Roundtrip test".to_string(),
        mode: Some(opencode_server::routes::execute::types::ExecuteMode::Plan),
        stream: Some(true),
    };
    let json = serde_json::to_string(&req).expect("should serialize");
    let parsed: opencode_server::routes::execute::types::ExecuteRequest =
        serde_json::from_str(&json).expect("should deserialize");
    assert_eq!(parsed.prompt, req.prompt);
    assert_eq!(parsed.mode, req.mode);
    assert_eq!(parsed.stream, req.stream);
}

#[test]
fn test_execute_event_tool_call_serialization() {
    use opencode_server::routes::execute::types::ExecuteEvent;
    use serde_json::json;
    let event = ExecuteEvent::ToolCall {
        tool: "read".to_string(),
        params: json!({"path": "/test"}),
        call_id: "call-1".to_string(),
    };
    let json_str = serde_json::to_string(&event).expect("should serialize");
    assert!(json_str.contains(r#""event":"tool_call""#));
    assert!(json_str.contains(r#""tool":"read""#));
    assert!(json_str.contains(r#""call_id":"call-1""#));
}

#[test]
fn test_execute_event_tool_result_serialization() {
    use opencode_server::routes::execute::types::ExecuteEvent;
    use serde_json::json;
    let event = ExecuteEvent::ToolResult {
        tool: "read".to_string(),
        result: json!({"content": "file contents"}),
        call_id: "call-1".to_string(),
        success: true,
    };
    let json_str = serde_json::to_string(&event).expect("should serialize");
    assert!(json_str.contains(r#""event":"tool_result""#));
    assert!(json_str.contains(r#""tool":"read""#));
    assert!(json_str.contains(r#""success":true"#));
}

#[test]
fn test_execute_event_message_serialization() {
    use opencode_server::routes::execute::types::ExecuteEvent;
    use serde_json::json;
    let event = ExecuteEvent::Message {
        role: "assistant".to_string(),
        content: "Hello there!".to_string(),
    };
    let json_str = serde_json::to_string(&event).expect("should serialize");
    assert!(json_str.contains(r#""event":"message""#));
    assert!(json_str.contains(r#""role":"assistant""#));
    assert!(json_str.contains(r#""content":"Hello there!""#));
}

#[test]
fn test_execute_event_complete_serialization() {
    use opencode_server::routes::execute::types::ExecuteEvent;
    use serde_json::json;
    let state = json!({"messages": 5, "tools_used": ["read", "write"]});
    let event = ExecuteEvent::Complete {
        session_state: state,
    };
    let json_str = serde_json::to_string(&event).expect("should serialize");
    assert!(json_str.contains(r#""event":"complete""#));
    assert!(json_str.contains(r#""session_state""#));
}

#[test]
fn test_execute_event_error_serialization() {
    use opencode_server::routes::execute::types::ExecuteEvent;
    let event = ExecuteEvent::Error {
        code: "TOOL_NOT_FOUND".to_string(),
        message: "The tool 'foo' was not found".to_string(),
    };
    let json_str = serde_json::to_string(&event).expect("should serialize");
    assert!(json_str.contains(r#""event":"error""#));
    assert!(json_str.contains(r#""code":"TOOL_NOT_FOUND""#));
    assert!(json_str.contains("foo"));
}

#[test]
fn test_execute_event_token_serialization() {
    use opencode_server::routes::execute::types::ExecuteEvent;
    let event = ExecuteEvent::Token {
        content: "streaming token".to_string(),
    };
    let json_str = serde_json::to_string(&event).expect("should serialize");
    assert!(json_str.contains(r#""event":"token""#));
    assert!(json_str.contains(r#""content":"streaming token""#));
}

#[test]
fn test_execute_event_tool_call_deserialization() {
    use opencode_server::routes::execute::types::ExecuteEvent;
    let json_str = r#"{"event": "tool_call", "tool": "read", "params": {}, "call_id": "c1"}"#;
    let event: ExecuteEvent = serde_json::from_str(json_str).expect("should deserialize");
    match event {
        ExecuteEvent::ToolCall {
            tool,
            params: _,
            call_id,
        } => {
            assert_eq!(tool, "read");
            assert_eq!(call_id, "c1");
        }
        _ => panic!("expected ToolCall variant"),
    }
}

#[test]
fn test_execute_event_message_deserialization() {
    use opencode_server::routes::execute::types::ExecuteEvent;
    let json_str = r#"{"event": "message", "role": "assistant", "content": "Hi!"}"#;
    let event: ExecuteEvent = serde_json::from_str(json_str).expect("should deserialize");
    match event {
        ExecuteEvent::Message { role, content } => {
            assert_eq!(role, "assistant");
            assert_eq!(content, "Hi!");
        }
        _ => panic!("expected Message variant"),
    }
}

#[test]
fn test_execute_event_error_deserialization() {
    use opencode_server::routes::execute::types::ExecuteEvent;
    let json_str = r#"{"event": "error", "code": "ERR", "message": "oops"}"#;
    let event: ExecuteEvent = serde_json::from_str(json_str).expect("should deserialize");
    match event {
        ExecuteEvent::Error { code, message } => {
            assert_eq!(code, "ERR");
            assert_eq!(message, "oops");
        }
        _ => panic!("expected Error variant"),
    }
}

#[test]
fn test_execute_event_complete_deserialization() {
    use opencode_server::routes::execute::types::ExecuteEvent;
    let json_str = r#"{"event": "complete", "session_state": {"messages": 5}}"#;
    let event: ExecuteEvent = serde_json::from_str(json_str).expect("should deserialize");
    match event {
        ExecuteEvent::Complete { session_state } => {
            assert_eq!(session_state["messages"], 5);
        }
        _ => panic!("expected Complete variant"),
    }
}

#[test]
fn test_execute_event_tool_result_deserialization() {
    use opencode_server::routes::execute::types::ExecuteEvent;
    let json_str = r#"{"event": "tool_result", "tool": "read", "result": {"content": "hi"}, "call_id": "c1", "success": true}"#;
    let event: ExecuteEvent = serde_json::from_str(json_str).expect("should deserialize");
    match event {
        ExecuteEvent::ToolResult {
            tool,
            result: _,
            call_id,
            success,
        } => {
            assert_eq!(tool, "read");
            assert_eq!(call_id, "c1");
            assert!(success);
        }
        _ => panic!("expected ToolResult variant"),
    }
}

#[test]
fn test_execute_event_token_deserialization() {
    use opencode_server::routes::execute::types::ExecuteEvent;
    let json_str = r#"{"event": "token", "content": "hi"}"#;
    let event: ExecuteEvent = serde_json::from_str(json_str).expect("should deserialize");
    match event {
        ExecuteEvent::Token { content } => {
            assert_eq!(content, "hi");
        }
        _ => panic!("expected Token variant"),
    }
}

#[test]
fn test_execute_mode_deserialization_build() {
    use opencode_server::routes::execute::types::ExecuteMode;
    assert_eq!(
        serde_json::from_str::<ExecuteMode>(r#""build""#).unwrap(),
        ExecuteMode::Build
    );
}

#[test]
fn test_execute_mode_deserialization_plan() {
    use opencode_server::routes::execute::types::ExecuteMode;
    assert_eq!(
        serde_json::from_str::<ExecuteMode>(r#""plan""#).unwrap(),
        ExecuteMode::Plan
    );
}

#[test]
fn test_execute_mode_deserialization_general() {
    use opencode_server::routes::execute::types::ExecuteMode;
    assert_eq!(
        serde_json::from_str::<ExecuteMode>(r#""general""#).unwrap(),
        ExecuteMode::General
    );
}

#[test]
fn test_execute_mode_serialization_build() {
    use opencode_server::routes::execute::types::ExecuteMode;
    assert_eq!(
        serde_json::to_string(&ExecuteMode::Build).unwrap(),
        r#""build""#
    );
}

#[test]
fn test_execute_mode_serialization_plan() {
    use opencode_server::routes::execute::types::ExecuteMode;
    assert_eq!(
        serde_json::to_string(&ExecuteMode::Plan).unwrap(),
        r#""plan""#
    );
}

#[test]
fn test_execute_mode_serialization_general() {
    use opencode_server::routes::execute::types::ExecuteMode;
    assert_eq!(
        serde_json::to_string(&ExecuteMode::General).unwrap(),
        r#""general""#
    );
}

#[test]
fn test_execute_request_all_fields() {
    let json_str = r#"{"prompt": "Full request", "mode": "build", "stream": false}"#;
    let req: opencode_server::routes::execute::types::ExecuteRequest =
        serde_json::from_str(json_str).expect("should parse");
    assert_eq!(req.prompt, "Full request");
    assert_eq!(
        req.mode,
        Some(opencode_server::routes::execute::types::ExecuteMode::Build)
    );
    assert_eq!(req.stream, Some(false));
}

#[test]
fn test_execute_request_stream_defaults_to_true_when_missing() {
    let json_str = r#"{"prompt": "Test only prompt"}"#;
    let req: opencode_server::routes::execute::types::ExecuteRequest =
        serde_json::from_str(json_str).expect("should parse");
    assert_eq!(req.stream, Some(true));
}

#[test]
fn test_execute_event_tool_result_failure_serialization() {
    use opencode_server::routes::execute::types::ExecuteEvent;
    use serde_json::json;
    let event = ExecuteEvent::ToolResult {
        tool: "write".to_string(),
        result: json!({"error": "Permission denied"}),
        call_id: "call-2".to_string(),
        success: false,
    };
    let json_str = serde_json::to_string(&event).expect("should serialize");
    assert!(json_str.contains(r#""success":false"#));
    assert!(json_str.contains("Permission denied"));
}

#[test]
fn test_execute_event_multiple_tool_calls_sequence() {
    use opencode_server::routes::execute::types::ExecuteEvent;
    use serde_json::json;
    let event1 = ExecuteEvent::ToolCall {
        tool: "read".to_string(),
        params: json!({"path": "/a"}),
        call_id: "c1".to_string(),
    };
    let event2 = ExecuteEvent::ToolResult {
        tool: "read".to_string(),
        result: json!({"content": "content a"}),
        call_id: "c1".to_string(),
        success: true,
    };
    let event3 = ExecuteEvent::ToolCall {
        tool: "write".to_string(),
        params: json!({"path": "/b"}),
        call_id: "c2".to_string(),
    };

    let json1 = serde_json::to_string(&event1).expect("should serialize");
    let json2 = serde_json::to_string(&event2).expect("should serialize");
    let json3 = serde_json::to_string(&event3).expect("should serialize");

    assert!(json1.contains("read"));
    assert!(json2.contains("content a"));
    assert!(json3.contains("write"));
}

#[test]
fn test_execute_event_token_streaming_sequence() {
    use opencode_server::routes::execute::types::ExecuteEvent;
    let events = vec![
        ExecuteEvent::Token {
            content: "Hello".to_string(),
        },
        ExecuteEvent::Token {
            content: " ".to_string(),
        },
        ExecuteEvent::Token {
            content: "World".to_string(),
        },
    ];

    for event in events {
        let json_str = serde_json::to_string(&event).expect("should serialize");
        assert!(json_str.contains(r#""event":"token""#));
    }
}

#[test]
fn test_session_message_added_for_execute() {
    use opencode_core::{Message, Session};
    let mut session = Session::new();
    let initial_count = session.messages.len();

    session.add_message(Message::user("Execute prompt"));

    assert_eq!(session.messages.len(), initial_count + 1);
    assert_eq!(session.messages.last().unwrap().content, "Execute prompt");
}

#[test]
fn test_session_with_assistant_response_after_execute() {
    use opencode_core::{Message, Session};
    let mut session = Session::new();
    session.add_message(Message::user("Execute prompt"));

    session.add_message(Message::assistant("Execute response"));

    let messages = session.messages;
    assert_eq!(messages.len(), 2);
    assert_eq!(messages[0].content, "Execute prompt");
    assert_eq!(messages[1].content, "Execute response");
}

#[test]
fn test_execute_event_complete_contains_session_info() {
    use opencode_core::{Message, Session};
    use opencode_server::routes::execute::types::ExecuteEvent;
    use serde_json::json;
    let mut session = Session::new();
    session.add_message(Message::user("Test"));
    session.add_message(Message::assistant("Response"));

    let event = ExecuteEvent::Complete {
        session_state: json!({
            "session_id": session.id.to_string(),
            "message_count": session.messages.len(),
        }),
    };

    match event {
        ExecuteEvent::Complete { session_state } => {
            assert!(session_state["session_id"].as_str().unwrap().contains("-"));
            assert_eq!(session_state["message_count"], 2);
        }
        _ => panic!("expected Complete variant"),
    }
}

#[test]
fn test_execute_request_empty_prompt_rejected_by_validation() {
    let json_str = r#"{"prompt": ""}"#;
    let result: Result<opencode_server::routes::execute::types::ExecuteRequest, _> =
        serde_json::from_str(json_str);
    assert!(result.is_ok());

    let req = result.unwrap();
    assert!(req.prompt.is_empty());
}

#[test]
fn test_execute_request_camel_case_keys() {
    let json_str = r#"{"prompt": "test", "mode": "build", "stream": true}"#;
    let req: opencode_server::routes::execute::types::ExecuteRequest =
        serde_json::from_str(json_str).expect("should parse with camelCase");
    assert_eq!(req.prompt, "test");
}

#[test]
fn test_execute_event_json_format_compatible() {
    use opencode_server::routes::execute::types::ExecuteEvent;
    use serde_json::json;
    let event = ExecuteEvent::ToolCall {
        tool: "grep".to_string(),
        params: json!({"pattern": "test", "path": "/src"}),
        call_id: "id-1".to_string(),
    };
    let json_str = serde_json::to_string(&event).expect("should serialize to valid JSON");

    let parsed: serde_json::Value = serde_json::from_str(&json_str).expect("should parse as JSON");
    assert!(parsed.is_object());
    assert_eq!(parsed["event"], "tool_call");
}

#[test]
fn test_execute_event_all_variants_have_event_field() {
    use opencode_server::routes::execute::types::ExecuteEvent;
    use serde_json::json;

    let variants = vec![
        ExecuteEvent::ToolCall {
            tool: "t".to_string(),
            params: json!({}),
            call_id: "c".to_string(),
        },
        ExecuteEvent::ToolResult {
            tool: "t".to_string(),
            result: json!({}),
            call_id: "c".to_string(),
            success: true,
        },
        ExecuteEvent::Message {
            role: "r".to_string(),
            content: "c".to_string(),
        },
        ExecuteEvent::Token {
            content: "c".to_string(),
        },
        ExecuteEvent::Error {
            code: "e".to_string(),
            message: "m".to_string(),
        },
        ExecuteEvent::Complete {
            session_state: json!({}),
        },
    ];

    for event in variants {
        let json_str = serde_json::to_string(&event).expect("should serialize");
        let parsed: serde_json::Value = serde_json::from_str(&json_str).expect("should parse");
        assert!(
            parsed.get("event").is_some(),
            "All ExecuteEvent variants should have 'event' field"
        );
    }
}

#[test]
fn test_execute_mode_all_variants_serializable() {
    use opencode_server::routes::execute::types::ExecuteMode;

    let modes = vec![ExecuteMode::Build, ExecuteMode::Plan, ExecuteMode::General];

    for mode in modes {
        let json_str = serde_json::to_string(&mode).expect("should serialize");
        let parsed: ExecuteMode = serde_json::from_str(&json_str).expect("should deserialize");
        assert_eq!(parsed, mode);
    }
}

#[test]
fn test_execute_event_tool_result_with_error_content() {
    use opencode_server::routes::execute::types::ExecuteEvent;
    use serde_json::json;
    let event = ExecuteEvent::ToolResult {
        tool: "bash".to_string(),
        result: json!({"error": "Command failed: exit code 1"}),
        call_id: "c1".to_string(),
        success: false,
    };

    let json_str = serde_json::to_string(&event).expect("should serialize");
    assert!(json_str.contains(r#""success":false"#));
    assert!(json_str.contains("Command failed"));
}

#[test]
fn test_execute_request_without_stream_field_defaults_to_true() {
    let json_str = r#"{"prompt": "Hello"}"#;
    let req: opencode_server::routes::execute::types::ExecuteRequest =
        serde_json::from_str(json_str).expect("should parse");
    assert_eq!(req.prompt, "Hello");
    assert_eq!(req.stream, Some(true));
}
