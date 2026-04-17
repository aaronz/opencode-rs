use actix_web::{web, App, HttpServer};
use futures_util::{SinkExt, StreamExt};
use opencode_core::PermissionManager;
use opencode_permission::ApprovalQueue;
use opencode_server::routes::ws::SessionHub;
use opencode_server::streaming::{ReconnectionStore, StreamMessage};
use opencode_server::ServerState;
use serde_json::json;
use std::sync::Arc;
use std::time::Duration;
use tokio_tungstenite::tungstenite::Message as WsMessage;

fn create_ws_agent_streaming_test_state() -> ServerState {
    let temp_dir = tempfile::tempdir().unwrap();
    let db_path = temp_dir.path().join("test.db");
    ServerState {
        storage: {
            let pool = opencode_storage::database::StoragePool::new(&db_path).unwrap();
            let session_repo =
                Arc::new(opencode_storage::SqliteSessionRepository::new(pool.clone()));
            let project_repo =
                Arc::new(opencode_storage::SqliteProjectRepository::new(pool.clone()));
            let account_repo =
                Arc::new(opencode_storage::SqliteAccountRepository::new(pool.clone()));
            let plugin_state_repo = Arc::new(opencode_storage::SqlitePluginStateRepository::new(
                pool.clone(),
            ));
            Arc::new(opencode_storage::StorageService::new(
                session_repo,
                project_repo,
                account_repo,
                plugin_state_repo,
                pool,
            ))
        },
        models: std::sync::Arc::new(opencode_llm::ModelRegistry::new()),
        config: std::sync::Arc::new(std::sync::RwLock::new(opencode_core::Config::default())),
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
        session_hub: std::sync::Arc::new(SessionHub::new(256)),
        server_start_time: std::time::SystemTime::now(),
        permission_manager: std::sync::Arc::new(std::sync::RwLock::new(
            PermissionManager::default(),
        )),
        approval_queue: std::sync::Arc::new(std::sync::RwLock::new(ApprovalQueue::default())),
        audit_log: None,
    }
}

async fn start_ws_test_server(port: u16) -> (String, tokio::task::JoinHandle<()>) {
    let state = create_ws_agent_streaming_test_state();
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

async fn start_ws_test_server_with_state(
    port: u16,
) -> (String, tokio::task::JoinHandle<()>, web::Data<ServerState>) {
    let state = create_ws_agent_streaming_test_state();
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

async fn ws_close(
    ws: &mut tokio_tungstenite::WebSocketStream<
        tokio_tungstenite::MaybeTlsStream<tokio::net::TcpStream>,
    >,
) {
    let close_msg = WsMessage::Close(None);
    let _ = ws.send(close_msg).await;
    let _ = ws.next().await;
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

#[tokio::test]
async fn test_ws_agent_streaming_connects_successfully() {
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
async fn test_ws_agent_streaming_tool_events_broadcast() {
    use opencode_core::bus::InternalEvent;

    let (ws_url, server_handle, state_data) = start_ws_test_server_with_state(0).await;

    let ws_url_with_session = ws_url.replace("/test-session", "?session_id=test-agent-session");

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
        session_id: "test-agent-session".to_string(),
        tool_name: "read".to_string(),
        call_id: "call-agent-123".to_string(),
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
        Some("test-agent-session"),
        "Session ID should match"
    );
    assert_eq!(
        tool_call_parsed.get("tool_name").and_then(|v| v.as_str()),
        Some("read"),
        "Tool name should be 'read'"
    );
    assert_eq!(
        tool_call_parsed.get("call_id").and_then(|v| v.as_str()),
        Some("call-agent-123"),
        "Call ID should match"
    );

    event_bus.publish(InternalEvent::ToolCallEnded {
        session_id: "test-agent-session".to_string(),
        call_id: "call-agent-123".to_string(),
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
        tool_result_parsed.get("success").and_then(|v| v.as_bool()),
        Some(true),
        "Success should be true"
    );

    ws_close(&mut ws).await;
    server_handle.abort();
}

#[tokio::test]
async fn test_ws_agent_streaming_multiple_concurrent_connections() {
    use opencode_core::bus::InternalEvent;

    let (ws_url, server_handle, state_data) = start_ws_test_server_with_state(0).await;

    let ws_url_with_session =
        ws_url.replace("/test-session", "?session_id=test-multi-conn-session");

    let (mut ws1, _) = tokio_tungstenite::connect_async(&ws_url_with_session)
        .await
        .expect("First client should connect to WebSocket endpoint");

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
        "First message should be Connected type"
    );
    assert_eq!(
        connected_parsed1.get("session_id").and_then(|v| v.as_str()),
        Some("test-multi-conn-session"),
        "Session ID should be test-multi-conn-session"
    );

    let (mut ws2, _) = tokio_tungstenite::connect_async(&ws_url_with_session)
        .await
        .expect("Second client should connect to WebSocket endpoint");

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
        Some("test-multi-conn-session"),
        "Session ID should be test-multi-conn-session"
    );

    let event_bus = &state_data.event_bus;

    event_bus.publish(InternalEvent::ToolCallStarted {
        session_id: "test-multi-conn-session".to_string(),
        tool_name: "read".to_string(),
        call_id: "call-multi-conn-123".to_string(),
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
        Some("test-multi-conn-session"),
        "First client session ID should match"
    );
    assert_eq!(
        tool_call_parsed2.get("session_id").and_then(|v| v.as_str()),
        Some("test-multi-conn-session"),
        "Second client session ID should match"
    );

    event_bus.publish(InternalEvent::AgentStatusChanged {
        session_id: "test-multi-conn-session".to_string(),
        status: "processing".to_string(),
    });

    let status_parsed1 = recv_next_non_heartbeat(&mut ws1).await;
    let status_parsed2 = recv_next_non_heartbeat(&mut ws2).await;

    assert!(
        status_parsed1.is_some(),
        "First client should receive status update"
    );
    assert!(
        status_parsed2.is_some(),
        "Second client should receive status update"
    );

    let status_parsed1 = status_parsed1.unwrap();
    let status_parsed2 = status_parsed2.unwrap();

    assert_eq!(
        status_parsed1.get("type").and_then(|v| v.as_str()),
        Some("session_update"),
        "First client should receive session_update type"
    );
    assert_eq!(
        status_parsed2.get("type").and_then(|v| v.as_str()),
        Some("session_update"),
        "Second client should receive session_update type"
    );
    assert_eq!(
        status_parsed1.get("status").and_then(|v| v.as_str()),
        Some("processing"),
        "First client status should be 'processing'"
    );
    assert_eq!(
        status_parsed2.get("status").and_then(|v| v.as_str()),
        Some("processing"),
        "Second client status should be 'processing'"
    );

    let _ = ws_close(&mut ws1).await;
    let _ = ws_close(&mut ws2).await;
    server_handle.abort();
}

#[tokio::test]
async fn test_ws_agent_streaming_client_disconnect_handled_gracefully() {
    let (ws_url, server_handle) = start_ws_test_server(0).await;

    let ws_url_with_session =
        ws_url.replace("/test-session", "?session_id=disconnect-graceful-test");

    let (mut ws, _) = tokio_tungstenite::connect_async(&ws_url_with_session)
        .await
        .expect("Should connect to WebSocket endpoint");

    let msg = ws.next().await;
    assert!(msg.is_some(), "Should receive a message after connection");
    drop(ws);

    tokio::time::sleep(Duration::from_millis(100)).await;

    let (ws_url2, server_handle2) = start_ws_test_server(0).await;
    let ws_url_with_session2 = ws_url2.replace("/test-session", "?session_id=new-session");

    let result = tokio_tungstenite::connect_async(&ws_url_with_session2).await;
    assert!(
        result.is_ok(),
        "Server should still be operating after client disconnect"
    );

    let (mut ws2, _) = result.expect("Should reconnect");
    let msg2 = ws2.next().await;
    assert!(
        msg2.is_some(),
        "Should receive connection message on new client"
    );

    ws_close(&mut ws2).await;
    server_handle.abort();
    server_handle2.abort();
}

#[tokio::test]
async fn test_ws_agent_streaming_session_hub_multiple_clients() {
    let hub = SessionHub::new(256);

    let session_id = "hub-multi-client-test";

    let _r1 = hub.register_client(session_id, "client-1").await;
    let _r2 = hub.register_client(session_id, "client-2").await;
    let _r3 = hub.register_client(session_id, "client-3").await;

    assert_eq!(hub.get_session_client_count(session_id).await, 3);
    assert_eq!(hub.total_client_count().await, 3);
    assert_eq!(hub.session_count().await, 1);
}

#[tokio::test]
async fn test_ws_agent_streaming_session_hub_broadcast() {
    let hub = SessionHub::new(256);

    let session_id = "hub-broadcast-test";
    let mut receiver1 = hub.register_client(session_id, "client-1").await;
    let mut receiver2 = hub.register_client(session_id, "client-2").await;

    let msg = StreamMessage::Message {
        session_id: session_id.to_string(),
        content: "Broadcast message".to_string(),
        role: "assistant".to_string(),
    };

    hub.broadcast(session_id, msg).await;

    let received1 = receiver1.recv().await.expect("client-1 should receive");
    let received2 = receiver2.recv().await.expect("client-2 should receive");

    match (&received1, &received2) {
        (
            StreamMessage::Message { content: c1, .. },
            StreamMessage::Message { content: c2, .. },
        ) => {
            assert_eq!(c1, "Broadcast message");
            assert_eq!(c2, "Broadcast message");
        }
        _ => panic!("Expected Message variant"),
    }
}

#[tokio::test]
async fn test_ws_agent_streaming_event_emitter_integration() {
    use opencode_core::bus::InternalEvent;

    let (ws_url, server_handle, state_data) = start_ws_test_server_with_state(0).await;

    let ws_url_with_session = ws_url.replace("/test-session", "?session_id=test-emitter-session");

    let (mut ws, _) = tokio_tungstenite::connect_async(&ws_url_with_session)
        .await
        .expect("Should connect to WebSocket endpoint");

    let _connected_msg = ws.next().await;
    assert!(_connected_msg.is_some(), "Should receive connected message");

    let event_bus = &state_data.event_bus;

    event_bus.publish(InternalEvent::ToolCallStarted {
        session_id: "test-emitter-session".to_string(),
        tool_name: "grep".to_string(),
        call_id: "call-emitter-456".to_string(),
    });

    let tool_call_parsed = recv_next_non_heartbeat(&mut ws).await;
    assert!(
        tool_call_parsed.is_some(),
        "Should receive tool call event via event bus"
    );

    let tool_call_parsed = tool_call_parsed.unwrap();
    assert_eq!(
        tool_call_parsed.get("type").and_then(|v| v.as_str()),
        Some("tool_call"),
        "Should receive tool_call type"
    );
    assert_eq!(
        tool_call_parsed.get("tool_name").and_then(|v| v.as_str()),
        Some("grep"),
        "Tool name should be 'grep'"
    );

    ws_close(&mut ws).await;
    server_handle.abort();
}

#[test]
fn test_ws_agent_streaming_stream_message_serialization() {
    let msg = StreamMessage::Message {
        session_id: "test-session".to_string(),
        content: "Hello, WebSocket!".to_string(),
        role: "assistant".to_string(),
    };

    let json_str = serde_json::to_string(&msg).expect("should serialize");
    assert!(json_str.contains(r#""type":"message""#));
    assert!(json_str.contains("test-session"));
    assert!(json_str.contains("Hello, WebSocket!"));
}

#[test]
fn test_ws_agent_streaming_stream_message_tool_call_serialization() {
    let msg = StreamMessage::ToolCall {
        session_id: "test-session".to_string(),
        tool_name: "read".to_string(),
        args: json!({"path": "/test/file.txt"}),
        call_id: "call-123".to_string(),
    };

    let json_str = serde_json::to_string(&msg).expect("should serialize");
    assert!(json_str.contains(r#""type":"tool_call""#));
    assert!(json_str.contains("read"));
    assert!(json_str.contains("call-123"));
}

#[test]
fn test_ws_agent_streaming_stream_message_tool_result_serialization() {
    let msg = StreamMessage::ToolResult {
        session_id: "test-session".to_string(),
        call_id: "call-123".to_string(),
        output: "file contents here".to_string(),
        success: true,
    };

    let json_str = serde_json::to_string(&msg).expect("should serialize");
    assert!(json_str.contains(r#""type":"tool_result""#));
    assert!(json_str.contains("call-123"));
    assert!(json_str.contains(r#""success":true"#));
}

#[test]
fn test_ws_agent_streaming_reconnection_store_record_and_replay() {
    let store = ReconnectionStore::new(100);

    store.record_message(
        "session-replay-test",
        StreamMessage::Message {
            session_id: "session-replay-test".to_string(),
            content: "message 1".to_string(),
            role: "assistant".to_string(),
        },
    );

    store.record_message(
        "session-replay-test",
        StreamMessage::Message {
            session_id: "session-replay-test".to_string(),
            content: "message 2".to_string(),
            role: "assistant".to_string(),
        },
    );

    let entries = store.replay_from("session-replay-test", 0);
    assert_eq!(entries.len(), 2);
    assert_eq!(entries[0].sequence, 1);
    assert_eq!(entries[1].sequence, 2);
}

#[test]
fn test_ws_agent_streaming_reconnection_store_token_generation_and_validation() {
    let store = ReconnectionStore::new(100);

    store.record_message(
        "session-token-test",
        StreamMessage::Connected { session_id: None },
    );

    let token = store.generate_token("session-token-test", None);
    assert!(!token.is_empty());

    let validated_seq = store.validate_token("session-token-test", &token);
    assert!(validated_seq.is_some());
    assert_eq!(validated_seq.unwrap(), 1);

    let invalid = store.validate_token("session-token-test", "invalid-token");
    assert!(invalid.is_none());
}

#[tokio::test]
async fn test_ws_agent_streaming_client_unregister_no_crash() {
    let hub = SessionHub::new(256);

    let session_id = "unregister-test";
    let client_id = "unregister-client";

    let _receiver = hub.register_client(session_id, client_id).await;
    assert_eq!(hub.get_session_client_count(session_id).await, 1);

    hub.unregister_client(session_id, client_id).await;
    assert_eq!(hub.get_session_client_count(session_id).await, 0);

    hub.unregister_client(session_id, client_id).await;
    assert_eq!(hub.get_session_client_count(session_id).await, 0);
    assert_eq!(hub.session_count().await, 0);
}
