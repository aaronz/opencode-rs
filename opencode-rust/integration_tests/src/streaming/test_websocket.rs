use actix_web::{web, App, HttpServer};
use futures_util::{SinkExt, StreamExt};
use opencode_server::routes::ws::SessionHub;
use std::sync::Arc;
use std::time::Duration;
use tokio_tungstenite::tungstenite::Message as WsMessage;

fn create_ws_test_server_state() -> opencode_server::ServerState {
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
        session_hub: Arc::new(SessionHub::new(256)),
        server_start_time: std::time::SystemTime::now(),
        permission_manager: Arc::new(std::sync::RwLock::new(
            opencode_core::PermissionManager::default(),
        )),
        approval_queue: Arc::new(std::sync::RwLock::new(
            opencode_permission::ApprovalQueue::default(),
        )),
        audit_log: None,
        runtime: opencode_server::build_placeholder_runtime(),
    }
}

async fn start_ws_test_server(port: u16) -> (String, tokio::task::JoinHandle<()>) {
    let state = create_ws_test_server_state();
    let state_data = web::Data::new(state);

    let bind_addr = format!("127.0.0.1:{}", port);
    let std_listener = std::net::TcpListener::bind(&bind_addr).unwrap();
    let actual_port = std_listener.local_addr().unwrap().port();
    let ws_url = format!("ws://127.0.0.1:{}/ws/test-session", actual_port);

    let handle = tokio::spawn(async move {
        HttpServer::new(move || {
            App::new()
                .app_data(state_data.clone())
                .service(web::scope("/ws").configure(opencode_server::routes::ws::init))
        })
        .listen(std_listener)
        .unwrap()
        .run()
        .await
        .unwrap();
    });

    tokio::time::sleep(Duration::from_millis(100)).await;

    (ws_url, handle)
}

async fn ws_close(
    ws: &mut tokio_tungstenite::WebSocketStream<
        tokio_tungstenite::MaybeTlsStream<tokio::net::TcpStream>,
    >,
) {
    let close_msg = WsMessage::Close(None);
    let _ = ws.send(close_msg).await;
    let _ = ws.next().await;
}

#[tokio::test]
async fn test_ws_connect() {
    let (ws_url, server_handle) = start_ws_test_server(0).await;

    let ws_url_with_session = ws_url.replace("/test-session", "?session_id=test-session");

    let (mut ws, _) = tokio_tungstenite::connect_async(&ws_url_with_session)
        .await
        .expect("Should connect to WebSocket endpoint");

    let msg = ws.next().await;
    assert!(msg.is_some(), "Should receive a message after connection");

    let ws_msg = msg.unwrap().expect("Message should not be error");
    let text = ws_msg.into_text().expect("Should be text message");

    let parsed: serde_json::Value = serde_json::from_str(&text).expect("Should parse as JSON");

    assert_eq!(
        parsed.get("type").and_then(|v| v.as_str()),
        Some("connected"),
        "First message should be Connected type"
    );

    assert_eq!(
        parsed.get("session_id").and_then(|v| v.as_str()),
        Some("test-session"),
        "Session ID should be test-session"
    );

    ws_close(&mut ws).await;
    server_handle.abort();
}

#[tokio::test]
async fn test_ws_execute_stream() {
    use opencode_core::bus::InternalEvent;

    let (ws_url, server_handle, state_data) = start_ws_test_server_with_state(0).await;

    let ws_url_with_session = ws_url.replace("/test-session", "?session_id=execute-stream-session");

    let (mut ws, _) = tokio_tungstenite::connect_async(&ws_url_with_session)
        .await
        .expect("Should connect to WebSocket endpoint");

    let connected_msg = ws.next().await;
    assert!(connected_msg.is_some(), "Should receive connected message");
    let connected_text = connected_msg
        .unwrap()
        .expect("Message should not be error")
        .into_text()
        .expect("Should be text message");
    let connected_parsed: serde_json::Value =
        serde_json::from_str(&connected_text).expect("Should parse as JSON");
    assert_eq!(
        connected_parsed.get("type").and_then(|v| v.as_str()),
        Some("connected"),
        "First message should be Connected type"
    );

    let event_bus = &state_data.event_bus;
    event_bus.publish(InternalEvent::ToolCallStarted {
        session_id: "execute-stream-session".to_string(),
        tool_name: "read".to_string(),
        call_id: "call-exec-123".to_string(),
    });

    let tool_call_parsed = recv_next_non_heartbeat(&mut ws).await;
    assert!(
        tool_call_parsed.is_some(),
        "Should receive tool call event via WebSocket"
    );

    let tool_call_parsed = tool_call_parsed.unwrap();

    assert_eq!(
        tool_call_parsed.get("type").and_then(|v| v.as_str()),
        Some("tool_call"),
        "Should receive tool_call type"
    );
    assert_eq!(
        tool_call_parsed.get("session_id").and_then(|v| v.as_str()),
        Some("execute-stream-session"),
        "Session ID should match"
    );
    assert_eq!(
        tool_call_parsed.get("tool_name").and_then(|v| v.as_str()),
        Some("read"),
        "Tool name should be 'read'"
    );
    assert_eq!(
        tool_call_parsed.get("call_id").and_then(|v| v.as_str()),
        Some("call-exec-123"),
        "Call ID should match"
    );

    event_bus.publish(InternalEvent::ToolCallEnded {
        session_id: "execute-stream-session".to_string(),
        call_id: "call-exec-123".to_string(),
        success: true,
    });

    let tool_result_parsed = recv_next_non_heartbeat(&mut ws).await;
    assert!(
        tool_result_parsed.is_some(),
        "Should receive tool result event via WebSocket"
    );

    let tool_result_parsed = tool_result_parsed.unwrap();

    assert_eq!(
        tool_result_parsed.get("type").and_then(|v| v.as_str()),
        Some("tool_result"),
        "Should receive tool_result type"
    );
    assert_eq!(
        tool_result_parsed.get("call_id").and_then(|v| v.as_str()),
        Some("call-exec-123"),
        "Call ID should match"
    );
    assert_eq!(
        tool_result_parsed.get("success").and_then(|v| v.as_bool()),
        Some(true),
        "Success should be true"
    );

    ws_close(&mut ws).await;
    server_handle.abort();
}

#[tokio::test]
async fn test_ws_multiple_streams() {
    use opencode_core::bus::InternalEvent;

    let (ws_url, server_handle, state_data) = start_ws_test_server_with_state(0).await;

    let ws_url_session1 = ws_url.replace("/test-session", "?session_id=stream-session-1");
    let ws_url_session2 = ws_url.replace("/test-session", "?session_id=stream-session-2");

    let (mut ws1, _) = tokio_tungstenite::connect_async(&ws_url_session1)
        .await
        .expect("First client should connect to WebSocket endpoint");

    let (mut ws2, _) = tokio_tungstenite::connect_async(&ws_url_session2)
        .await
        .expect("Second client should connect to WebSocket endpoint");

    let connected_msg1 = ws1.next().await;
    assert!(
        connected_msg1.is_some(),
        "First client should receive connected message"
    );
    let connected_text1 = connected_msg1
        .unwrap()
        .expect("Message should not be error")
        .into_text()
        .expect("Should be text message");
    let connected_parsed1: serde_json::Value =
        serde_json::from_str(&connected_text1).expect("Should parse as JSON");
    assert_eq!(
        connected_parsed1.get("type").and_then(|v| v.as_str()),
        Some("connected"),
        "First client first message should be Connected type"
    );
    assert_eq!(
        connected_parsed1.get("session_id").and_then(|v| v.as_str()),
        Some("stream-session-1"),
        "First client session ID should be stream-session-1"
    );

    let connected_msg2 = ws2.next().await;
    assert!(
        connected_msg2.is_some(),
        "Second client should receive connected message"
    );
    let connected_text2 = connected_msg2
        .unwrap()
        .expect("Message should not be error")
        .into_text()
        .expect("Should be text message");
    let connected_parsed2: serde_json::Value =
        serde_json::from_str(&connected_text2).expect("Should parse as JSON");
    assert_eq!(
        connected_parsed2.get("type").and_then(|v| v.as_str()),
        Some("connected"),
        "Second client first message should be Connected type"
    );
    assert_eq!(
        connected_parsed2.get("session_id").and_then(|v| v.as_str()),
        Some("stream-session-2"),
        "Second client session ID should be stream-session-2"
    );

    let event_bus = &state_data.event_bus;

    event_bus.publish(InternalEvent::ToolCallStarted {
        session_id: "stream-session-1".to_string(),
        tool_name: "read".to_string(),
        call_id: "call-s1-123".to_string(),
    });

    event_bus.publish(InternalEvent::ToolCallStarted {
        session_id: "stream-session-2".to_string(),
        tool_name: "write".to_string(),
        call_id: "call-s2-456".to_string(),
    });

    let tool_call_parsed1 = recv_next_non_heartbeat(&mut ws1).await;
    let tool_call_parsed2 = recv_next_non_heartbeat(&mut ws2).await;

    assert!(
        tool_call_parsed1.is_some(),
        "First client should receive tool call event"
    );
    assert!(
        tool_call_parsed2.is_some(),
        "Second client should receive tool call event"
    );

    let tool_call_parsed1 = tool_call_parsed1.unwrap();
    let tool_call_parsed2 = tool_call_parsed2.unwrap();

    assert_eq!(
        tool_call_parsed1.get("type").and_then(|v| v.as_str()),
        Some("tool_call"),
        "First client should receive tool_call type"
    );
    assert_eq!(
        tool_call_parsed2.get("type").and_then(|v| v.as_str()),
        Some("tool_call"),
        "Second client should receive tool_call type"
    );
    assert_eq!(
        tool_call_parsed1.get("session_id").and_then(|v| v.as_str()),
        Some("stream-session-1"),
        "First client session ID should match"
    );
    assert_eq!(
        tool_call_parsed2.get("session_id").and_then(|v| v.as_str()),
        Some("stream-session-2"),
        "Second client session ID should match"
    );
    assert_eq!(
        tool_call_parsed1.get("tool_name").and_then(|v| v.as_str()),
        Some("read"),
        "First client tool name should be 'read'"
    );
    assert_eq!(
        tool_call_parsed2.get("tool_name").and_then(|v| v.as_str()),
        Some("write"),
        "Second client tool name should be 'write'"
    );

    event_bus.publish(InternalEvent::ToolCallEnded {
        session_id: "stream-session-1".to_string(),
        call_id: "call-s1-123".to_string(),
        success: true,
    });

    event_bus.publish(InternalEvent::ToolCallEnded {
        session_id: "stream-session-2".to_string(),
        call_id: "call-s2-456".to_string(),
        success: true,
    });

    let tool_result_parsed1 = recv_next_non_heartbeat(&mut ws1).await;
    let tool_result_parsed2 = recv_next_non_heartbeat(&mut ws2).await;

    assert!(
        tool_result_parsed1.is_some(),
        "First client should receive tool result event"
    );
    assert!(
        tool_result_parsed2.is_some(),
        "Second client should receive tool result event"
    );

    let tool_result_parsed1 = tool_result_parsed1.unwrap();
    let tool_result_parsed2 = tool_result_parsed2.unwrap();

    assert_eq!(
        tool_result_parsed1.get("type").and_then(|v| v.as_str()),
        Some("tool_result"),
        "First client should receive tool_result type"
    );
    assert_eq!(
        tool_result_parsed2.get("type").and_then(|v| v.as_str()),
        Some("tool_result"),
        "Second client should receive tool_result type"
    );
    assert_eq!(
        tool_result_parsed1.get("call_id").and_then(|v| v.as_str()),
        Some("call-s1-123"),
        "First client call ID should match"
    );
    assert_eq!(
        tool_result_parsed2.get("call_id").and_then(|v| v.as_str()),
        Some("call-s2-456"),
        "Second client call ID should match"
    );

    let _ = ws_close(&mut ws1).await;
    let _ = ws_close(&mut ws2).await;
    server_handle.abort();
}

#[tokio::test]
async fn test_ws_empty_response() {
    use opencode_core::bus::InternalEvent;

    let (ws_url, server_handle, state_data) = start_ws_test_server_with_state(0).await;

    let ws_url_with_session = ws_url.replace("/test-session", "?session_id=empty-response-session");

    let (mut ws, _) = tokio_tungstenite::connect_async(&ws_url_with_session)
        .await
        .expect("Should connect to WebSocket endpoint");

    let connected_msg = ws.next().await;
    assert!(connected_msg.is_some(), "Should receive connected message");

    let event_bus = &state_data.event_bus;

    event_bus.publish(InternalEvent::MessageAdded {
        session_id: "empty-response-session".to_string(),
        message_id: "msg-empty-001".to_string(),
    });

    tokio::time::sleep(Duration::from_millis(200)).await;

    event_bus.publish(InternalEvent::AgentStatusChanged {
        session_id: "empty-response-session".to_string(),
        status: "processing".to_string(),
    });

    event_bus.publish(InternalEvent::MessageAdded {
        session_id: "empty-response-session".to_string(),
        message_id: "msg-empty-002".to_string(),
    });

    event_bus.publish(InternalEvent::AgentStatusChanged {
        session_id: "empty-response-session".to_string(),
        status: "completed".to_string(),
    });

    let mut found_status_or_message = false;
    let mut found_complete = false;
    let mut iterations = 0;

    while let Some(msg) = ws.next().await {
        iterations += 1;
        if iterations > 100 {
            break;
        }
        if let Ok(text) = msg.unwrap().into_text() {
            if let Ok(parsed) = serde_json::from_str::<serde_json::Value>(&text) {
                match parsed.get("type").and_then(|v| v.as_str()) {
                    Some("heartbeat") => continue,
                    Some("session_update") => {
                        found_status_or_message = true;
                        if parsed.get("status").and_then(|v| v.as_str()) == Some("completed") {
                            found_complete = true;
                            break;
                        }
                    }
                    Some("message") => {
                        found_status_or_message = true;
                    }
                    Some("connected") => continue,
                    _ => {}
                }
            }
        }
    }

    assert!(found_status_or_message, "Should handle events gracefully");
    assert!(found_complete, "Should complete even with empty response");

    ws_close(&mut ws).await;
    server_handle.abort();
}

#[tokio::test]
async fn test_ws_long_response() {
    use opencode_core::bus::InternalEvent;

    let (ws_url, server_handle, state_data) = start_ws_test_server_with_state(0).await;

    let ws_url_with_session = ws_url.replace("/test-session", "?session_id=long-response-session");

    let (mut ws, _) = tokio_tungstenite::connect_async(&ws_url_with_session)
        .await
        .expect("Should connect to WebSocket endpoint");

    let connected_msg = ws.next().await;
    assert!(connected_msg.is_some(), "Should receive connected message");

    let event_bus = &state_data.event_bus;

    event_bus.publish(InternalEvent::MessageAdded {
        session_id: "long-response-session".to_string(),
        message_id: "msg-long-001".to_string(),
    });

    tokio::time::sleep(Duration::from_millis(200)).await;

    event_bus.publish(InternalEvent::MessageUpdated {
        session_id: "long-response-session".to_string(),
        message_id: "msg-long-001".to_string(),
    });

    event_bus.publish(InternalEvent::MessageAdded {
        session_id: "long-response-session".to_string(),
        message_id: "msg-long-002".to_string(),
    });

    event_bus.publish(InternalEvent::MessageUpdated {
        session_id: "long-response-session".to_string(),
        message_id: "msg-long-002".to_string(),
    });

    let mut received_messages = 0;
    let mut iterations = 0;

    while let Some(msg) = ws.next().await {
        iterations += 1;
        if iterations > 100 {
            break;
        }
        if let Ok(text) = msg.unwrap().into_text() {
            if let Ok(parsed) = serde_json::from_str::<serde_json::Value>(&text) {
                match parsed.get("type").and_then(|v| v.as_str()) {
                    Some("heartbeat") => continue,
                    Some("message") => {
                        received_messages += 1;
                    }
                    _ => {}
                }
            }
        }
    }

    assert!(
        received_messages >= 2,
        "Should handle multiple streaming messages"
    );

    ws_close(&mut ws).await;
    server_handle.abort();
}

#[tokio::test]
async fn test_ws_connection_drop() {
    use opencode_core::bus::InternalEvent;

    let (ws_url, server_handle, state_data) = start_ws_test_server_with_state(0).await;

    let ws_url_with_session = ws_url.replace("/test-session", "?session_id=drop-test-session");

    let (mut ws, _) = tokio_tungstenite::connect_async(&ws_url_with_session)
        .await
        .expect("Should connect to WebSocket endpoint");

    let connected_msg = ws.next().await;
    assert!(connected_msg.is_some(), "Should receive connected message");

    let event_bus = &state_data.event_bus;

    event_bus.publish(InternalEvent::ToolCallStarted {
        session_id: "drop-test-session".to_string(),
        tool_name: "read".to_string(),
        call_id: "call-drop-001".to_string(),
    });

    let tool_call_parsed = recv_next_non_heartbeat(&mut ws).await;
    assert!(tool_call_parsed.is_some(), "Should receive tool call event");

    drop(ws);

    tokio::time::sleep(Duration::from_millis(100)).await;

    event_bus.publish(InternalEvent::ToolCallEnded {
        session_id: "drop-test-session".to_string(),
        call_id: "call-drop-001".to_string(),
        success: false,
    });

    tokio::time::sleep(Duration::from_millis(100)).await;

    let (ws_url2, server_handle2) = start_ws_test_server(0).await;
    let ws_url_with_session2 =
        ws_url2.replace("/test-session", "?session_id=new-session-after-drop");

    let result = tokio_tungstenite::connect_async(&ws_url_with_session2).await;
    assert!(
        result.is_ok(),
        "Server should still be operating after client connection drop"
    );

    let (mut ws2, _) = result.expect("Should reconnect to new server");
    let msg2 = ws2.next().await;
    assert!(
        msg2.is_some(),
        "Should receive connection message on new client"
    );

    ws_close(&mut ws2).await;
    server_handle.abort();
    server_handle2.abort();
}

async fn recv_next_non_heartbeat(
    ws: &mut tokio_tungstenite::WebSocketStream<
        tokio_tungstenite::MaybeTlsStream<tokio::net::TcpStream>,
    >,
) -> Option<serde_json::Value> {
    while let Some(msg) = ws.next().await {
        if let Ok(text) = msg.unwrap().into_text() {
            if let Ok(parsed) = serde_json::from_str::<serde_json::Value>(&text) {
                if parsed.get("type").and_then(|v| v.as_str()) == Some("heartbeat") {
                    continue;
                }
                return Some(parsed);
            }
        }
    }
    None
}

async fn start_ws_test_server_with_state(
    port: u16,
) -> (
    String,
    tokio::task::JoinHandle<()>,
    web::Data<opencode_server::ServerState>,
) {
    let state = create_ws_test_server_state();
    let state_data = web::Data::new(state);

    let bind_addr = format!("127.0.0.1:{}", port);
    let std_listener = std::net::TcpListener::bind(&bind_addr).unwrap();
    let actual_port = std_listener.local_addr().unwrap().port();
    let ws_url = format!("ws://127.0.0.1:{}/ws/test-session", actual_port);

    let state_for_server = state_data.clone();
    let handle = tokio::spawn(async move {
        HttpServer::new(move || {
            App::new()
                .app_data(state_for_server.clone())
                .service(web::scope("/ws").configure(opencode_server::routes::ws::init))
        })
        .listen(std_listener)
        .unwrap()
        .run()
        .await
        .unwrap();
    });

    tokio::time::sleep(Duration::from_millis(100)).await;

    (ws_url, handle, state_data)
}
