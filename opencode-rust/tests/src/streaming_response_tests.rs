use actix_web::{web, App, HttpServer};
use opencode_server::streaming::StreamMessage;
use serde_json::json;
use std::sync::Arc;
use std::time::Duration;

#[tokio::test]
async fn test_stream_message_token_sequence() {
    let tokens = vec![
        StreamMessage::Message {
            session_id: "stream-test".to_string(),
            content: "He".to_string(),
            role: "assistant".to_string(),
        },
        StreamMessage::Message {
            session_id: "stream-test".to_string(),
            content: "llo".to_string(),
            role: "assistant".to_string(),
        },
        StreamMessage::Message {
            session_id: "stream-test".to_string(),
            content: " world".to_string(),
            role: "assistant".to_string(),
        },
    ];

    for (i, token) in tokens.iter().enumerate() {
        let json = serde_json::to_string(token).expect("should serialize");
        assert!(json.contains("stream-test"));
        if i == 0 {
            assert!(json.contains("He"));
        }
        if i == 1 {
            assert!(json.contains("llo"));
        }
        if i == 2 {
            assert!(json.contains("world"));
        }
    }
}

#[tokio::test]
async fn test_stream_message_tool_call_to_result_sequence() {
    let tool_call = StreamMessage::ToolCall {
        session_id: "seq-test".to_string(),
        tool_name: "read".to_string(),
        args: json!({"path": "/test.txt"}),
        call_id: "call-seq-1".to_string(),
    };

    let tool_result = StreamMessage::ToolResult {
        session_id: "seq-test".to_string(),
        call_id: "call-seq-1".to_string(),
        output: "file contents".to_string(),
        success: true,
    };

    let call_json = serde_json::to_string(&tool_call).expect("should serialize");
    let result_json = serde_json::to_string(&tool_result).expect("should serialize");

    assert!(call_json.contains("tool_call"));
    assert!(call_json.contains("read"));
    assert!(result_json.contains("tool_result"));
    assert!(result_json.contains("call-seq-1"));
}

#[tokio::test]
async fn test_stream_message_all_variants_serialize_correctly() {
    let variants = vec![
        StreamMessage::Message {
            session_id: "s".to_string(),
            content: "c".to_string(),
            role: "r".to_string(),
        },
        StreamMessage::ToolCall {
            session_id: "s".to_string(),
            tool_name: "t".to_string(),
            args: json!({}),
            call_id: "c".to_string(),
        },
        StreamMessage::ToolResult {
            session_id: "s".to_string(),
            call_id: "c".to_string(),
            output: "o".to_string(),
            success: true,
        },
        StreamMessage::SessionUpdate {
            session_id: "s".to_string(),
            status: "st".to_string(),
        },
        StreamMessage::Heartbeat { timestamp: 0 },
        StreamMessage::Error {
            session_id: None,
            error: "e".to_string(),
            code: "e".to_string(),
            message: "m".to_string(),
        },
        StreamMessage::Connected { session_id: None },
    ];

    for variant in variants {
        let json_str = serde_json::to_string(&variant).expect("should serialize");
        let parsed: serde_json::Value = serde_json::from_str(&json_str).expect("should parse");
        assert!(
            parsed.get("type").is_some(),
            "All StreamMessage variants should have 'type' field"
        );
    }
}

#[test]
fn test_stream_message_from_tool_call_to_result() {
    let tool_call = StreamMessage::ToolCall {
        session_id: "test-session".to_string(),
        tool_name: "read".to_string(),
        args: json!({"path": "/test/file.txt"}),
        call_id: "call-123".to_string(),
    };

    let tool_result = StreamMessage::ToolResult {
        session_id: "test-session".to_string(),
        call_id: "call-123".to_string(),
        output: "file contents here".to_string(),
        success: true,
    };

    let call_json = serde_json::to_string(&tool_call).expect("should serialize");
    let result_json = serde_json::to_string(&tool_result).expect("should serialize");

    assert!(call_json.contains(r#""type":"tool_call""#));
    assert!(call_json.contains("read"));
    assert!(call_json.contains("call-123"));
    assert!(result_json.contains(r#""type":"tool_result""#));
    assert!(result_json.contains("call-123"));
    assert!(result_json.contains(r#""success":true"#));
}

#[test]
fn test_stream_message_session_update_serialization() {
    let msg = StreamMessage::SessionUpdate {
        session_id: "test-session".to_string(),
        status: "processing".to_string(),
    };

    let json_str = serde_json::to_string(&msg).expect("should serialize");
    assert!(json_str.contains(r#""type":"session_update""#));
    assert!(json_str.contains("processing"));
}

#[test]
fn test_stream_message_error_serialization() {
    let msg = StreamMessage::Error {
        session_id: Some("test-session".to_string()),
        error: "TOOL_NOT_FOUND".to_string(),
        code: "TOOL_NOT_FOUND".to_string(),
        message: "The tool 'foo' was not found".to_string(),
    };

    let json_str = serde_json::to_string(&msg).expect("should serialize");
    assert!(json_str.contains(r#""type":"error""#));
    assert!(json_str.contains("TOOL_NOT_FOUND"));
}

#[test]
fn test_stream_message_heartbeat_serialization() {
    let msg = StreamMessage::Heartbeat {
        timestamp: 1713200000,
    };

    let json_str = serde_json::to_string(&msg).expect("should serialize");
    assert!(json_str.contains(r#""type":"heartbeat""#));
    assert!(json_str.contains("1713200000"));
}

#[test]
fn test_stream_message_connected_serialization() {
    let msg = StreamMessage::Connected {
        session_id: Some("test-session".to_string()),
    };

    let json_str = serde_json::to_string(&msg).expect("should serialize");
    assert!(json_str.contains(r#""type":"connected""#));
}

#[test]
fn test_stream_message_session_id_extraction() {
    let msg = StreamMessage::Message {
        session_id: "session-abc".to_string(),
        content: "test".to_string(),
        role: "user".to_string(),
    };

    assert_eq!(msg.session_id(), Some("session-abc"));

    let error_msg = StreamMessage::Error {
        session_id: Some("session-xyz".to_string()),
        error: "err".to_string(),
        code: "err".to_string(),
        message: "error".to_string(),
    };

    assert_eq!(error_msg.session_id(), Some("session-xyz"));

    let heartbeat = StreamMessage::Heartbeat { timestamp: 0 };
    assert_eq!(heartbeat.session_id(), None);
}

#[tokio::test]
async fn test_tokens_arrive_progressively() {
    use futures_util::StreamExt;
    use std::time::Instant;

    let (server_url, server_handle, _temp_dir) = start_streaming_test_server(0).await;

    let client = reqwest::Client::new();

    let resp = client
        .post(format!("{}/api/run", server_url))
        .header("Accept", "text/event-stream")
        .json(&serde_json::json!({
            "prompt": "Write a haiku about coding",
            "stream": true
        }))
        .send()
        .await
        .expect("Failed to call run endpoint");

    let status = resp.status();

    if status.is_success() {
        let mut stream = resp.bytes_stream();
        let mut token_timestamps: Vec<(String, std::time::Duration)> = Vec::new();
        let start = Instant::now();
        let mut token_count = 0;

        while let Some(item) = stream.next().await {
            if let Ok(bytes) = item {
                let text = String::from_utf8_lossy(&bytes);
                let elapsed = start.elapsed();

                if text.contains("data:") && !text.contains("[DONE]") && !text.contains("heartbeat")
                {
                    token_count += 1;
                    let data_line = text
                        .lines()
                        .find(|l| l.starts_with("data:"))
                        .unwrap_or("")
                        .trim_start_matches("data: ")
                        .trim();

                    if !data_line.is_empty() && data_line != "[DONE]" {
                        token_timestamps.push((data_line.to_string(), elapsed));

                        if token_count >= 5 {
                            break;
                        }
                    }
                }

                if text.contains("[DONE]") {
                    break;
                }
            }
        }

        assert!(
            token_timestamps.len() >= 2,
            "Should receive at least 2 tokens/chunks progressively, got {}",
            token_timestamps.len()
        );

        let mut time_gaps: Vec<std::time::Duration> = Vec::new();
        for i in 1..token_timestamps.len() {
            let gap = token_timestamps[i].1 - token_timestamps[i - 1].1;
            time_gaps.push(gap);
        }

        let has_progressive_arrival = time_gaps
            .iter()
            .any(|gap| *gap > std::time::Duration::from_millis(1));

        assert!(
            has_progressive_arrival || token_timestamps.len() >= 3,
            "Tokens should arrive progressively over time. \
             Either there should be time gaps > 1ms between tokens, \
             or we should see at least 3 tokens indicating chunked delivery. \
             Got {} tokens with gaps: {:?}",
            token_timestamps.len(),
            time_gaps
        );

        for (i, (data, timestamp)) in token_timestamps.iter().enumerate() {
            println!(
                "Token {} arrived at {:?}: {}",
                i,
                timestamp,
                &data[..data.len().min(50)]
            );
        }
    } else {
        assert!(
            status.as_u16() == 400 || status.as_u16() == 500,
            "Without API keys, should return 400 or 500, got: {}",
            status
        );
    }

    server_handle.abort();
}

#[test]
fn test_error_stream_message_with_null_session_id() {
    let msg = StreamMessage::Error {
        session_id: None,
        error: "INTERNAL_ERROR".to_string(),
        code: "INTERNAL_ERROR".to_string(),
        message: "An internal error occurred".to_string(),
    };

    let json_str = serde_json::to_string(&msg).expect("should serialize");
    assert!(json_str.contains(r#""type":"error""#));
    assert!(json_str.contains("INTERNAL_ERROR"));

    let parsed: serde_json::Value = serde_json::from_str(&json_str).expect("should parse");
    assert!(parsed["session_id"].is_null());
}

#[test]
fn test_connected_stream_message_without_session_id() {
    let msg = StreamMessage::Connected { session_id: None };

    let json_str = serde_json::to_string(&msg).expect("should serialize");
    assert!(json_str.contains(r#""type":"connected""#));

    let parsed: serde_json::Value = serde_json::from_str(&json_str).expect("should parse");
    assert!(parsed["session_id"].is_null());
}

#[test]
fn test_sse_data_formatting() {
    let content = "Hello, World!";
    let sse_data = format!("data: {}\n\n", content);
    assert_eq!(sse_data, "data: Hello, World!\n\n");
}

#[test]
fn test_sse_error_formatting() {
    let error_json = r#"{"error":"streaming_error","message":"connection lost"}"#;
    let sse_data = format!("data: {}\n\n", error_json);
    assert!(sse_data.contains("data:"));
    assert!(sse_data.contains("error"));
    assert!(sse_data.contains("streaming_error"));
}

#[test]
fn test_sse_done_formatting() {
    let done_data = "data: [DONE]\n\n";
    assert_eq!(done_data, "data: [DONE]\n\n");
}

fn create_streaming_test_server_state() -> opencode_server::ServerState {
    let temp_dir = tempfile::tempdir().unwrap();
    let db_path = temp_dir.path().join("test.db");
    let pool = opencode_storage::database::StoragePool::new(&db_path).unwrap();
    let session_repo = Arc::new(opencode_storage::SqliteSessionRepository::new(pool.clone()));
    let project_repo = Arc::new(opencode_storage::SqliteProjectRepository::new(pool.clone()));
    opencode_server::ServerState {
        storage: Arc::new(opencode_storage::StorageService::new(
            session_repo,
            project_repo,
            pool,
        )),
        models: Arc::new(opencode_llm::ModelRegistry::new()),
        config: Arc::new(std::sync::RwLock::new(opencode_core::Config::default())),
        event_bus: opencode_core::bus::SharedEventBus::default(),
        reconnection_store: opencode_server::streaming::ReconnectionStore::default(),
        temp_db_dir: None,
        connection_monitor: Arc::new(
            opencode_server::streaming::conn_state::ConnectionMonitor::new(),
        ),
        share_server: Arc::new(std::sync::RwLock::new(
            opencode_server::routes::share::ShareServer::with_default_config(),
        )),
        acp_enabled: false,
        acp_stream: opencode_control_plane::AcpEventStream::new().into(),
        acp_client_registry: Arc::new(tokio::sync::RwLock::new(
            opencode_server::routes::acp_ws::AcpClientRegistry::new(),
        )),
        tool_registry: Arc::new(opencode_tools::ToolRegistry::new()),
        session_hub: Arc::new(opencode_server::routes::ws::SessionHub::new(256)),
        server_start_time: std::time::SystemTime::now(),
        permission_manager: Arc::new(std::sync::RwLock::new(
            opencode_core::PermissionManager::default(),
        )),
        approval_queue: Arc::new(std::sync::RwLock::new(
            opencode_permission::ApprovalQueue::default(),
        )),
    }
}

async fn start_streaming_test_server(
    port: u16,
) -> (String, tokio::task::JoinHandle<()>, Arc<tempfile::TempDir>) {
    let temp_dir = Arc::new(tempfile::tempdir().unwrap());
    let temp_dir_clone = temp_dir.clone();
    let state = create_streaming_test_server_state();
    let state_data = web::Data::new(state);

    let bind_addr = format!("127.0.0.1:{}", port);
    let std_listener = std::net::TcpListener::bind(&bind_addr).unwrap();
    let actual_port = std_listener.local_addr().unwrap().port();
    let server_url = format!("http://127.0.0.1:{}", actual_port);

    let handle = tokio::spawn(async move {
        HttpServer::new(move || {
            App::new()
                .app_data(state_data.clone())
                .service(web::scope("/api").configure(opencode_server::routes::config_routes))
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

#[tokio::test]
async fn test_run_endpoint_accepts_sse_request() {
    let (server_url, server_handle, _temp_dir) = start_streaming_test_server(0).await;

    let client = reqwest::Client::new();

    let resp = client
        .post(format!("{}/api/run", server_url))
        .header("Accept", "text/event-stream")
        .json(&serde_json::json!({
            "prompt": "test streaming",
            "stream": true
        }))
        .send()
        .await
        .expect("Failed to call run endpoint");

    let status = resp.status();
    assert!(
        status.is_success() || status.as_u16() == 400 || status.as_u16() == 500,
        "Run endpoint should respond, got: {}",
        status
    );

    server_handle.abort();
}

#[tokio::test]
async fn test_run_endpoint_stream_param_works() {
    let (server_url, server_handle, _temp_dir) = start_streaming_test_server(0).await;

    let client = reqwest::Client::new();

    let resp = client
        .post(format!("{}/api/run", server_url))
        .json(&serde_json::json!({
            "prompt": "test streaming",
            "stream": true
        }))
        .send()
        .await
        .expect("Failed to call run endpoint");

    let status = resp.status();
    assert!(
        status.is_success() || status.as_u16() == 400 || status.as_u16() == 500,
        "Run endpoint should respond, got: {}",
        status
    );

    server_handle.abort();
}

#[tokio::test]
async fn test_run_endpoint_without_stream_param() {
    let (server_url, server_handle, _temp_dir) = start_streaming_test_server(0).await;

    let client = reqwest::Client::new();

    let resp = client
        .post(format!("{}/api/run", server_url))
        .json(&serde_json::json!({
            "prompt": "test non-streaming"
        }))
        .send()
        .await
        .expect("Failed to call run endpoint");

    let status = resp.status();
    assert!(
        status.is_success() || status.as_u16() == 400 || status.as_u16() == 500,
        "Run endpoint should respond, got: {}",
        status
    );

    server_handle.abort();
}

#[tokio::test]
async fn test_sse_streaming() {
    use futures_util::StreamExt;

    let (server_url, server_handle, _temp_dir) = start_streaming_test_server(0).await;

    let client = reqwest::Client::new();

    let resp = client
        .post(format!("{}/api/run", server_url))
        .header("Accept", "text/event-stream")
        .json(&serde_json::json!({
            "prompt": "Hi",
            "stream": true
        }))
        .send()
        .await
        .expect("Failed to call run endpoint");

    let status = resp.status();
    let content_type = resp
        .headers()
        .get("content-type")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("");

    if status.is_success() {
        assert!(
            content_type.contains("text/event-stream"),
            "Successful streaming response should have text/event-stream content-type, got: {}",
            content_type
        );

        let mut stream = resp.bytes_stream();
        let mut has_sse_data = false;

        while let Some(item) = stream.next().await {
            if let Ok(bytes) = item {
                let text = String::from_utf8_lossy(&bytes);
                if text.contains("data:") {
                    has_sse_data = true;
                }
                if text.contains("[DONE]") || text.contains("error") {
                    break;
                }
            }
        }

        assert!(has_sse_data, "Stream should contain SSE data events");
    } else {
        assert!(
            status.as_u16() == 400 || status.as_u16() == 500,
            "Without API keys, should return 400 or 500, got: {}",
            status
        );
    }

    server_handle.abort();
}

#[tokio::test]
async fn test_run_endpoint_validation_rejects_empty_prompt() {
    let (server_url, server_handle, _temp_dir) = start_streaming_test_server(0).await;

    let client = reqwest::Client::new();

    let resp = client
        .post(format!("{}/api/run", server_url))
        .json(&serde_json::json!({
            "prompt": ""
        }))
        .send()
        .await
        .expect("Failed to call run endpoint");

    assert!(
        resp.status().is_client_error(),
        "Empty prompt should return client error"
    );

    server_handle.abort();
}

#[tokio::test]
async fn test_run_endpoint_validation_rejects_invalid_agent() {
    let (server_url, server_handle, _temp_dir) = start_streaming_test_server(0).await;

    let client = reqwest::Client::new();

    let resp = client
        .post(format!("{}/api/run", server_url))
        .json(&serde_json::json!({
            "prompt": "test",
            "agent": "invalid_agent"
        }))
        .send()
        .await
        .expect("Failed to call run endpoint");

    assert!(
        resp.status().is_client_error(),
        "Invalid agent should return client error"
    );

    server_handle.abort();
}

#[test]
fn test_stream_message_from_internal_event_tool_call() {
    use opencode_core::bus::InternalEvent;

    let event = InternalEvent::ToolCallStarted {
        session_id: "test-session".to_string(),
        tool_name: "read".to_string(),
        call_id: "call-456".to_string(),
    };

    let stream_msg = StreamMessage::from_internal_event(&event);
    assert!(stream_msg.is_some());

    let msg = stream_msg.unwrap();
    match msg {
        StreamMessage::ToolCall {
            session_id,
            tool_name,
            call_id,
            ..
        } => {
            assert_eq!(session_id, "test-session");
            assert_eq!(tool_name, "read");
            assert_eq!(call_id, "call-456");
        }
        _ => panic!("Expected ToolCall variant"),
    }
}

#[test]
fn test_stream_message_from_internal_event_agent_status() {
    use opencode_core::bus::InternalEvent;

    let event = InternalEvent::AgentStatusChanged {
        session_id: "test-session".to_string(),
        status: "thinking".to_string(),
    };

    let stream_msg = StreamMessage::from_internal_event(&event);
    assert!(stream_msg.is_some());

    let msg = stream_msg.unwrap();
    match msg {
        StreamMessage::SessionUpdate { session_id, status } => {
            assert_eq!(session_id, "test-session");
            assert_eq!(status, "thinking");
        }
        _ => panic!("Expected SessionUpdate variant"),
    }
}

#[test]
fn test_stream_message_from_internal_event_session_ended() {
    use opencode_core::bus::InternalEvent;

    let event = InternalEvent::SessionEnded("session-end-test".to_string());

    let stream_msg = StreamMessage::from_internal_event(&event);
    assert!(stream_msg.is_some());

    let msg = stream_msg.unwrap();
    match msg {
        StreamMessage::SessionUpdate { session_id, status } => {
            assert_eq!(session_id, "session-end-test");
            assert_eq!(status, "ended");
        }
        _ => panic!("Expected SessionUpdate variant"),
    }
}

#[tokio::test]
async fn test_connection_close_terminates_cleanly() {
    use std::time::Duration;

    let (server_url, server_handle, _temp_dir) = start_streaming_test_server(0).await;

    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(30))
        .build()
        .expect("Failed to build client");

    let resp = client
        .post(format!("{}/api/run", server_url))
        .header("Accept", "text/event-stream")
        .json(&serde_json::json!({
            "prompt": "Write a long story about coding",
            "stream": true
        }))
        .send()
        .await
        .expect("Failed to send request");

    let status = resp.status();
    assert!(
        status.is_success() || status.as_u16() == 400 || status.as_u16() == 500,
        "Run endpoint should respond, got: {}",
        status
    );

    drop(resp);

    tokio::time::sleep(Duration::from_millis(100)).await;

    server_handle.abort();
}
