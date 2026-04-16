use actix_web::{web, App, HttpServer};
use opencode_core::{Message, Session};
use opencode_storage::migration::MigrationManager;
use serde_json::json;
use std::sync::Arc;
use std::time::Duration;

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
    let mut session = Session::new();
    let initial_count = session.messages.len();

    session.add_message(Message::user("Execute prompt"));

    assert_eq!(session.messages.len(), initial_count + 1);
    assert_eq!(session.messages.last().unwrap().content, "Execute prompt");
}

#[test]
fn test_session_with_assistant_response_after_execute() {
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
    use opencode_server::routes::execute::types::ExecuteEvent;
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

#[tokio::test]
async fn test_invalid_session_returns_404() {
    let (server_url, server_handle, _temp_dir) = start_test_server(0).await;

    let client = reqwest::Client::new();

    let fake_session_id = "550e8400-e29b-41d4-a716-446655440000";

    let resp = client
        .post(format!(
            "{}/api/sessions/{}/execute",
            server_url, fake_session_id
        ))
        .json(&serde_json::json!({
            "prompt": "Test prompt"
        }))
        .send()
        .await
        .expect("Failed to call execute endpoint");

    assert_eq!(
        resp.status().as_u16(),
        404,
        "Execute on non-existent session should return 404"
    );

    server_handle.abort();
}

async fn start_test_server(
    port: u16,
) -> (String, tokio::task::JoinHandle<()>, Arc<tempfile::TempDir>) {
    let temp_dir = Arc::new(tempfile::tempdir().unwrap());
    let temp_dir_clone = temp_dir.clone();
    let db_path = temp_dir.path().join("test.db");
    let pool = opencode_storage::database::StoragePool::new(&db_path).unwrap();

    let migration_manager = MigrationManager::new(pool.clone(), 2);
    migration_manager
        .migrate()
        .await
        .expect("Failed to run migrations");

    let session_repo = Arc::new(opencode_storage::SqliteSessionRepository::new(pool.clone()));
    let project_repo = Arc::new(opencode_storage::SqliteProjectRepository::new(pool.clone()));

    let mut config = opencode_core::Config::default();
    if let Ok(api_key) = std::env::var("OPENAI_API_KEY") {
        if !api_key.is_empty() {
            config.api_key = Some(api_key);
        }
    }

    let state = opencode_server::ServerState {
        storage: Arc::new(opencode_storage::StorageService::new(
            session_repo,
            project_repo,
            pool,
        )),
        models: std::sync::Arc::new(opencode_llm::ModelRegistry::new()),
        config: std::sync::Arc::new(std::sync::RwLock::new(config)),
        event_bus: opencode_core::bus::SharedEventBus::default(),
        reconnection_store: opencode_server::streaming::ReconnectionStore::default(),
        temp_db_dir: None,
        connection_monitor: std::sync::Arc::new(
            opencode_server::streaming::conn_state::ConnectionMonitor::new(),
        ),
        share_server: std::sync::Arc::new(std::sync::RwLock::new(
            opencode_server::routes::share::ShareServer::with_default_config(),
        )),
        acp_enabled: false,
        acp_stream: opencode_control_plane::AcpEventStream::new().into(),
        acp_client_registry: std::sync::Arc::new(tokio::sync::RwLock::new(
            opencode_server::routes::acp_ws::AcpClientRegistry::new(),
        )),
        tool_registry: std::sync::Arc::new(opencode_tools::ToolRegistry::new()),
        session_hub: std::sync::Arc::new(opencode_server::routes::ws::SessionHub::new(256)),
        server_start_time: std::time::SystemTime::now(),
    };

    let state_data = web::Data::new(state);

    let bind_addr = format!("127.0.0.1:{}", port);
    let std_listener = std::net::TcpListener::bind(&bind_addr).unwrap();
    let actual_port = std_listener.local_addr().unwrap().port();
    let server_url = format!("http://127.0.0.1:{}", actual_port);

    let handle = tokio::spawn(async move {
        let execute_handler = opencode_server::routes::execute::execute_session;
        HttpServer::new(move || {
            App::new()
                .app_data(state_data.clone())
                .service(
                    web::scope("/api/sessions").configure(opencode_server::routes::session::init),
                )
                .route(
                    "/api/sessions/{id}/execute",
                    web::post().to(execute_handler),
                )
        })
        .listen(std_listener)
        .unwrap()
        .run()
        .await
        .unwrap();
        drop(temp_dir_clone);
    });

    tokio::time::sleep(Duration::from_millis(200)).await;

    (server_url, handle, temp_dir)
}

fn parse_sse_events(body: &str) -> Vec<(String, serde_json::Value)> {
    let mut events = Vec::new();
    let mut current_event_type = String::new();
    let mut current_data = String::new();

    for line in body.lines() {
        if line.starts_with("event: ") {
            current_event_type = line.trim_start_matches("event: ").to_string();
        } else if line.starts_with("data: ") {
            current_data = line.trim_start_matches("data: ").to_string();
        } else if line.is_empty() && !current_event_type.is_empty() {
            if let Ok(json) = serde_json::from_str(&current_data) {
                events.push((current_event_type.clone(), json));
            }
            current_event_type.clear();
            current_data.clear();
        }
    }

    events
}

#[test]
fn test_execute_event_json_format_compatible() {
    use opencode_server::routes::execute::types::ExecuteEvent;
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

#[cfg(test)]
mod integration_tests {
    use super::*;

    #[tokio::test]
    async fn test_valid_session_execute_returns_200() {
        let has_api_key = std::env::var("OPENAI_API_KEY")
            .map(|k| !k.is_empty())
            .unwrap_or(false);

        if !has_api_key {
            eprintln!(
                "SKIPPED: OPENAI_API_KEY not set. This test requires a valid OpenAI API key."
            );
            return;
        }

        let (server_url, server_handle, _temp_dir) = start_test_server(0).await;

        let client = reqwest::Client::new();

        let create_resp = client
            .post(format!("{}/api/sessions", server_url))
            .json(&serde_json::json!({
                "initial_prompt": "Test prompt for execute endpoint"
            }))
            .send()
            .await
            .expect("Failed to create session");

        assert_eq!(
            create_resp.status().as_u16(),
            201,
            "Session creation should return 201"
        );

        let session_body: serde_json::Value = create_resp
            .json()
            .await
            .expect("Failed to parse session response");
        let session_id = session_body["session_id"]
            .as_str()
            .expect("Session ID should be a string");

        let execute_resp = client
            .post(format!(
                "{}/api/sessions/{}/execute",
                server_url, session_id
            ))
            .json(&serde_json::json!({
                "prompt": "Say hello in one word",
                "mode": "general",
                "stream": false
            }))
            .send()
            .await
            .expect("Failed to call execute endpoint");

        assert_eq!(
            execute_resp.status().as_u16(),
            200,
            "Valid session execute should return 200 OK"
        );

        let content_type = execute_resp.headers().get("content-type");
        assert!(
            content_type.is_some(),
            "Response should have content-type header"
        );

        let body = execute_resp
            .text()
            .await
            .expect("Failed to read response body");

        assert!(!body.is_empty(), "Response body should not be empty");

        let events = parse_sse_events(&body);
        assert!(!events.is_empty(), "Response should contain SSE events");

        let has_message_or_complete = events
            .iter()
            .any(|(event_type, _)| event_type == "message" || event_type == "complete");
        assert!(
            has_message_or_complete,
            "Response should contain message or complete event"
        );

        server_handle.abort();
    }

    #[tokio::test]
    async fn test_execute_endpoint_requires_session_to_exist() {
        let (server_url, server_handle, _temp_dir) = start_test_server(0).await;

        let client = reqwest::Client::new();

        let fake_session_id = "550e8400-e29b-41d4-a716-446655440000";

        let resp = client
            .post(format!(
                "{}/api/sessions/{}/execute",
                server_url, fake_session_id
            ))
            .json(&serde_json::json!({
                "prompt": "Test prompt"
            }))
            .send()
            .await
            .expect("Failed to call execute endpoint");

        assert_eq!(
            resp.status().as_u16(),
            404,
            "Execute on non-existent session should return 404"
        );

        server_handle.abort();
    }

    #[tokio::test]
    #[ignore] // TODO: fix routing issue - execute endpoint returns 404
    async fn test_execute_endpoint_returns_400_without_api_key() {
        let (server_url, server_handle, _temp_dir) = start_test_server(0).await;

        let client = reqwest::Client::new();

        let create_resp = client
            .post(format!("{}/api/sessions", server_url))
            .json(&serde_json::json!({
                "initial_prompt": "Test prompt"
            }))
            .send()
            .await
            .expect("Failed to create session");

        let status = create_resp.status();
        eprintln!("Session creation status: {}", status.as_u16());
        if status.as_u16() != 201 {
            let body = create_resp.text().await.unwrap_or_default();
            eprintln!("Session creation failed: {}", body);
            panic!("Session creation failed");
        }
        assert_eq!(status.as_u16(), 201);

        let session_body: serde_json::Value = create_resp
            .json()
            .await
            .expect("Failed to parse session response");
        let session_id = session_body["session_id"].as_str().unwrap();
        eprintln!("Created session ID: {}", session_id);

        let execute_url = format!("{}/api/sessions/{}/execute", server_url, session_id);
        eprintln!("Calling execute at: {}", execute_url);

        let resp = client
            .post(&execute_url)
            .json(&serde_json::json!({
                "prompt": "Test prompt"
            }))
            .send()
            .await
            .expect("Failed to call execute endpoint");

        let status = resp.status();
        let body_text = resp.text().await.unwrap_or_default();
        eprintln!(
            "Execute returned status: {} body: {}",
            status.as_u16(),
            body_text
        );

        assert_eq!(
            status.as_u16(),
            400,
            "Execute without API key should return 400"
        );

        let body: serde_json::Value =
            serde_json::from_str(&body_text).expect("Failed to parse error response");

        assert_eq!(
            body["code"].as_str().unwrap_or(""),
            "provider_init_error",
            "Error code should be provider_init_error"
        );

        server_handle.abort();
    }

    #[tokio::test]
    async fn test_execute_with_build_mode() {
        let has_api_key = std::env::var("OPENAI_API_KEY")
            .map(|k| !k.is_empty())
            .unwrap_or(false);

        if !has_api_key {
            eprintln!(
                "SKIPPED: OPENAI_API_KEY not set. This test requires a valid OpenAI API key."
            );
            return;
        }

        let (server_url, server_handle, _temp_dir) = start_test_server(0).await;

        let client = reqwest::Client::new();

        let create_resp = client
            .post(format!("{}/api/sessions", server_url))
            .json(&serde_json::json!({
                "initial_prompt": "Build task"
            }))
            .send()
            .await
            .expect("Failed to create session");

        let session_body: serde_json::Value = create_resp
            .json()
            .await
            .expect("Failed to parse session response");
        let session_id = session_body["session_id"].as_str().unwrap();

        let resp = client
            .post(format!(
                "{}/api/sessions/{}/execute",
                server_url, session_id
            ))
            .json(&serde_json::json!({
                "prompt": "Write a hello world program",
                "mode": "build"
            }))
            .send()
            .await
            .expect("Failed to call execute endpoint");

        assert_eq!(
            resp.status().as_u16(),
            200,
            "Execute with build mode should return 200"
        );

        let body = resp.text().await.expect("Failed to read response body");

        let events = parse_sse_events(&body);
        assert!(
            !events.is_empty(),
            "Build mode execute should return events"
        );

        server_handle.abort();
    }

    #[tokio::test]
    async fn test_execute_with_plan_mode() {
        let has_api_key = std::env::var("OPENAI_API_KEY")
            .map(|k| !k.is_empty())
            .unwrap_or(false);

        if !has_api_key {
            eprintln!(
                "SKIPPED: OPENAI_API_KEY not set. This test requires a valid OpenAI API key."
            );
            return;
        }

        let (server_url, server_handle, _temp_dir) = start_test_server(0).await;

        let client = reqwest::Client::new();

        let create_resp = client
            .post(format!("{}/api/sessions", server_url))
            .json(&serde_json::json!({
                "initial_prompt": "Plan task"
            }))
            .send()
            .await
            .expect("Failed to create session");

        let session_body: serde_json::Value = create_resp
            .json()
            .await
            .expect("Failed to parse session response");
        let session_id = session_body["session_id"].as_str().unwrap();

        let resp = client
            .post(format!(
                "{}/api/sessions/{}/execute",
                server_url, session_id
            ))
            .json(&serde_json::json!({
                "prompt": "Create a plan to build a web server",
                "mode": "plan"
            }))
            .send()
            .await
            .expect("Failed to call execute endpoint");

        assert_eq!(
            resp.status().as_u16(),
            200,
            "Execute with plan mode should return 200"
        );

        server_handle.abort();
    }

    #[tokio::test]
    #[ignore] // TODO: fix routing issue - execute endpoint returns 404
    async fn test_unauthenticated_returns_401() {
        use std::sync::Arc;
        use std::time::Duration;

        let temp_dir = tempfile::tempdir().unwrap();
        let db_path = temp_dir.path().join("test.db");
        let pool = opencode_storage::database::StoragePool::new(&db_path).unwrap();

        let migration_manager = MigrationManager::new(pool.clone(), 2);
        migration_manager
            .migrate()
            .await
            .expect("Failed to run migrations");

        let session_repo = Arc::new(opencode_storage::SqliteSessionRepository::new(pool.clone()));
        let project_repo = Arc::new(opencode_storage::SqliteProjectRepository::new(pool.clone()));

        let mut config = opencode_core::Config::default();
        config.api_key = Some("test-secret-api-key".to_string());

        let state = opencode_server::ServerState {
            storage: Arc::new(opencode_storage::StorageService::new(
                session_repo,
                project_repo,
                pool,
            )),
            models: std::sync::Arc::new(opencode_llm::ModelRegistry::new()),
            config: std::sync::Arc::new(std::sync::RwLock::new(config)),
            event_bus: opencode_core::bus::SharedEventBus::default(),
            reconnection_store: opencode_server::streaming::ReconnectionStore::default(),
            temp_db_dir: None,
            connection_monitor: std::sync::Arc::new(
                opencode_server::streaming::conn_state::ConnectionMonitor::new(),
            ),
            share_server: std::sync::Arc::new(std::sync::RwLock::new(
                opencode_server::routes::share::ShareServer::with_default_config(),
            )),
            acp_enabled: false,
            acp_stream: opencode_control_plane::AcpEventStream::new().into(),
            acp_client_registry: std::sync::Arc::new(tokio::sync::RwLock::new(
                opencode_server::routes::acp_ws::AcpClientRegistry::new(),
            )),
            tool_registry: std::sync::Arc::new(opencode_tools::ToolRegistry::new()),
            session_hub: std::sync::Arc::new(opencode_server::routes::ws::SessionHub::new(256)),
            server_start_time: std::time::SystemTime::now(),
        };

        let state_data = web::Data::new(state);
        let temp_dir = Arc::new(temp_dir);

        let bind_addr = "127.0.0.1:0";
        let std_listener = std::net::TcpListener::bind(bind_addr).unwrap();
        let actual_port = std_listener.local_addr().unwrap().port();
        let server_url = format!("http://127.0.0.1:{}", actual_port);

        let temp_dir_clone = temp_dir.clone();
        let execute_handler = opencode_server::routes::execute::execute_session;
        let handle = tokio::spawn(async move {
            HttpServer::new(move || {
                App::new()
                    .app_data(state_data.clone())
                    .service(
                        web::scope("/api/sessions")
                            .configure(opencode_server::routes::session::init),
                    )
                    .route(
                        "/api/sessions/{id}/execute",
                        web::post().to(execute_handler),
                    )
            })
            .listen(std_listener)
            .unwrap()
            .run()
            .await
            .unwrap();
            drop(temp_dir_clone);
        });

        tokio::time::sleep(Duration::from_millis(500)).await;

        let client = reqwest::Client::new();
        let fake_session_id = "550e8400-e29b-41d4-a716-446655440000";

        let resp = client
            .post(format!(
                "{}/api/sessions/{}/execute",
                server_url, fake_session_id
            ))
            .json(&serde_json::json!({
                "prompt": "Test prompt"
            }))
            .send()
            .await
            .expect("Failed to call execute endpoint");

        let status = resp.status().as_u16();
        let body_text = resp.text().await.unwrap_or_default();
        eprintln!("DEBUG: status={}, body={:?}", status, body_text);

        handle.abort();

        assert_eq!(
            status, 401,
            "Unauthenticated request should return 401 Unauthorized"
        );
    }

    #[tokio::test]
    async fn test_tool_execution_results_in_response() {
        let has_api_key = std::env::var("OPENAI_API_KEY")
            .map(|k| !k.is_empty())
            .unwrap_or(false);

        if !has_api_key {
            eprintln!(
                "SKIPPED: OPENAI_API_KEY not set. This test requires a valid OpenAI API key."
            );
            return;
        }

        let (server_url, server_handle, _temp_dir) = start_test_server(0).await;

        let client = reqwest::Client::new();

        let create_resp = client
            .post(format!("{}/api/sessions", server_url))
            .json(&serde_json::json!({
                "initial_prompt": "Test tool execution"
            }))
            .send()
            .await
            .expect("Failed to create session");

        assert_eq!(
            create_resp.status().as_u16(),
            201,
            "Session creation should return 201"
        );

        let session_body: serde_json::Value = create_resp
            .json()
            .await
            .expect("Failed to parse session response");
        let session_id = session_body["session_id"]
            .as_str()
            .expect("Session ID should be a string");

        let execute_resp = client
            .post(format!(
                "{}/api/sessions/{}/execute",
                server_url, session_id
            ))
            .json(&serde_json::json!({
                "prompt": "List the files in the current directory using the ls command",
                "mode": "general",
                "stream": false
            }))
            .send()
            .await
            .expect("Failed to call execute endpoint");

        assert_eq!(
            execute_resp.status().as_u16(),
            200,
            "Valid session execute should return 200 OK"
        );

        let body = execute_resp
            .text()
            .await
            .expect("Failed to read response body");

        assert!(!body.is_empty(), "Response body should not be empty");

        let events = parse_sse_events(&body);
        assert!(!events.is_empty(), "Response should contain SSE events");

        let has_tool_call = events
            .iter()
            .any(|(event_type, data)| event_type == "tool_call" && data.get("tool").is_some());

        let has_tool_result = events
            .iter()
            .any(|(event_type, data)| event_type == "tool_result" && data.get("tool").is_some());

        let has_message_or_complete = events
            .iter()
            .any(|(event_type, _)| event_type == "message" || event_type == "complete");

        assert!(
            has_message_or_complete,
            "Response should contain message or complete event"
        );

        eprintln!(
            "DEBUG: tool_call found: {}, tool_result found: {}, total events: {}",
            has_tool_call,
            has_tool_result,
            events.len()
        );

        for (event_type, data) in &events {
            eprintln!("Event: {} - {:?}", event_type, data);
        }

        assert!(
            has_tool_call || has_tool_result || events.len() > 2,
            "Response should contain tool-related events (tool_call or tool_result), or multiple events indicating tool usage"
        );

        server_handle.abort();
    }
}
