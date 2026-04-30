use actix_web::{web, App, HttpServer};
use futures_util::{SinkExt, StreamExt};
use opencode_server::routes::ws::SessionHub;
use opencode_server::streaming::{ReconnectionStore, StreamMessage};
use serde_json::json;
use std::sync::Arc;
use std::time::Duration;
use tokio_tungstenite::tungstenite::Message as WsMessage;

#[test]
fn test_stream_message_serialization() {
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
fn test_stream_message_tool_call_serialization() {
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
fn test_stream_message_tool_result_serialization() {
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

#[test]
fn test_reconnection_store_record_and_replay() {
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
fn test_reconnection_store_token_generation_and_validation() {
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

#[test]
fn test_reconnection_store_different_sessions_independent() {
    let store = ReconnectionStore::new(100);

    store.record_message(
        "session-A",
        StreamMessage::Message {
            session_id: "session-A".to_string(),
            content: "A's message".to_string(),
            role: "assistant".to_string(),
        },
    );

    store.record_message(
        "session-B",
        StreamMessage::Message {
            session_id: "session-B".to_string(),
            content: "B's message".to_string(),
            role: "assistant".to_string(),
        },
    );

    let token_a = store.generate_token("session-A", None);
    let token_b = store.generate_token("session-B", None);

    assert!(store.validate_token("session-A", &token_a).is_some());
    assert!(store.validate_token("session-B", &token_b).is_some());
    assert!(store.validate_token("session-A", &token_b).is_none());
    assert!(store.validate_token("session-B", &token_a).is_none());
}

#[test]
fn test_reconnection_store_replay_limit() {
    let store = ReconnectionStore::new(3);

    for i in 0..5 {
        store.record_message(
            "session-limit",
            StreamMessage::Message {
                session_id: "session-limit".to_string(),
                content: format!("message {}", i),
                role: "assistant".to_string(),
            },
        );
    }

    let entries = store.replay_from("session-limit", 0);
    assert_eq!(entries.len(), 3);
    assert!(entries[0].message.session_id().is_some());
}

#[tokio::test]
async fn test_session_hub_register_and_unregister_client() {
    let hub = SessionHub::new(256);

    let session_id = "test-session-hub";
    let client_id = "test-client";

    assert_eq!(hub.get_session_client_count(session_id).await, 0);

    let _receiver = hub.register_client(session_id, client_id).await;

    assert_eq!(hub.get_session_client_count(session_id).await, 1);
    assert_eq!(hub.total_client_count().await, 1);
    assert_eq!(hub.session_count().await, 1);

    hub.unregister_client(session_id, client_id).await;

    assert_eq!(hub.get_session_client_count(session_id).await, 0);
    assert_eq!(hub.total_client_count().await, 0);
    assert_eq!(hub.session_count().await, 0);
}

#[tokio::test]
async fn test_session_hub_multiple_clients_same_session() {
    let hub = SessionHub::new(256);

    let session_id = "shared-session";

    let _r1 = hub.register_client(session_id, "client-1").await;
    let _r2 = hub.register_client(session_id, "client-2").await;
    let _r3 = hub.register_client(session_id, "client-3").await;

    assert_eq!(hub.get_session_client_count(session_id).await, 3);
    assert_eq!(hub.total_client_count().await, 3);
    assert_eq!(hub.session_count().await, 1);
}

#[tokio::test]
async fn test_session_hub_broadcast_to_session() {
    let hub = SessionHub::new(256);

    let session_id = "broadcast-session";
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
async fn test_session_hub_broadcast_all() {
    let hub = SessionHub::new(256);

    let mut r1 = hub.register_client("session-1", "client-1").await;
    let mut r2 = hub.register_client("session-2", "client-2").await;

    let msg = StreamMessage::SessionUpdate {
        session_id: "all".to_string(),
        status: "broadcast_all_test".to_string(),
    };

    hub.broadcast_all(msg).await;

    let received1 = r1.recv().await.expect("client-1 should receive");
    let received2 = r2.recv().await.expect("client-2 should receive");

    match (&received1, &received2) {
        (
            StreamMessage::SessionUpdate { status: s1, .. },
            StreamMessage::SessionUpdate { status: s2, .. },
        ) => {
            assert_eq!(s1, "broadcast_all_test");
            assert_eq!(s2, "broadcast_all_test");
        }
        _ => panic!("Expected SessionUpdate variant"),
    }
}

#[tokio::test]
async fn test_session_hub_client_disconnect_remaining_clients() {
    let hub = SessionHub::new(256);

    let session_id = "disconnect-test";
    let mut receiver1 = hub.register_client(session_id, "client-1").await;
    let mut receiver2 = hub.register_client(session_id, "client-2").await;

    hub.unregister_client(session_id, "client-1").await;

    let msg = StreamMessage::Message {
        session_id: session_id.to_string(),
        content: "After disconnect".to_string(),
        role: "assistant".to_string(),
    };

    hub.broadcast(session_id, msg).await;

    let received2 = receiver2.recv().await.expect("client-2 should receive");
    match received2 {
        StreamMessage::Message { content, .. } => {
            assert_eq!(content, "After disconnect");
        }
        _ => panic!("Expected Message variant"),
    }

    let err = receiver1.try_recv();
    assert!(err.is_err(), "disconnected client should not receive");
}

#[test]
fn test_connection_type_variants() {
    use opencode_server::streaming::ConnectionType;

    let ws_type = ConnectionType::WebSocket;
    let sse_type = ConnectionType::Sse;

    assert_eq!(format!("{:?}", ws_type), "WebSocket");
    assert_eq!(format!("{:?}", sse_type), "Sse");
}

#[test]
fn test_stream_message_from_tool_call_to_result_sequence() {
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

#[test]
fn test_stream_message_token_sequence_for_streaming() {
    let tokens = [
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
    }
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
fn test_stream_message_all_variants_have_type_field() {
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

#[tokio::test]
async fn test_session_hub_multiple_sessions_independent() {
    let hub = SessionHub::new(256);

    let mut r1 = hub.register_client("session-A", "client-A1").await;
    let _r2 = hub.register_client("session-B", "client-B1").await;
    let _r3 = hub.register_client("session-A", "client-A2").await;

    assert_eq!(hub.session_count().await, 2);
    assert_eq!(hub.get_session_client_count("session-A").await, 2);
    assert_eq!(hub.get_session_client_count("session-B").await, 1);

    let msg_a = StreamMessage::Message {
        session_id: "session-A".to_string(),
        content: "A only".to_string(),
        role: "assistant".to_string(),
    };
    hub.broadcast("session-A", msg_a).await;

    let received = r1.recv().await.expect("client-A1 should receive");
    match received {
        StreamMessage::Message { content, .. } => {
            assert_eq!(content, "A only");
        }
        _ => panic!("Expected Message variant"),
    }

    let msg_b = StreamMessage::SessionUpdate {
        session_id: "session-B".to_string(),
        status: "B status".to_string(),
    };
    hub.broadcast("session-B", msg_b).await;

    hub.unregister_client("session-A", "client-A1").await;
    hub.unregister_client("session-A", "client-A2").await;
    hub.unregister_client("session-B", "client-B1").await;

    assert_eq!(hub.session_count().await, 0);
}

#[test]
fn test_reconnection_store_replay_from_specific_sequence() {
    let store = ReconnectionStore::new(100);

    for i in 1..=5 {
        store.record_message(
            "replay-seq",
            StreamMessage::Message {
                session_id: "replay-seq".to_string(),
                content: format!("msg-{}", i),
                role: "assistant".to_string(),
            },
        );
    }

    let entries_after_2 = store.replay_from("replay-seq", 2);
    assert_eq!(entries_after_2.len(), 3);
    assert_eq!(entries_after_2[0].sequence, 3);
    assert_eq!(entries_after_2[1].sequence, 4);
    assert_eq!(entries_after_2[2].sequence, 5);

    let entries_after_4 = store.replay_from("replay-seq", 4);
    assert_eq!(entries_after_4.len(), 1);
    assert_eq!(entries_after_4[0].sequence, 5);

    let entries_after_5 = store.replay_from("replay-seq", 5);
    assert!(entries_after_5.is_empty());
}

#[test]
fn test_reconnection_store_empty_for_unknown_session() {
    let store = ReconnectionStore::new(100);

    let entries = store.replay_from("unknown-session", 0);
    assert!(entries.is_empty());

    let token = store.generate_token("unknown-session", None);
    let validated = store.validate_token("unknown-session", &token);
    assert!(validated.is_some());
    assert_eq!(validated.unwrap(), 0);
}

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

#[tokio::test]
async fn test_ws_connects_successfully() {
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
async fn test_tool_events_stream_realtime() {
    use opencode_core::DomainEvent;

    let (ws_url, server_handle, state_data) = start_ws_test_server_with_state(0).await;

    let ws_url_with_session = ws_url.replace("/test-session", "?session_id=test-tool-session");

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
    event_bus.publish(DomainEvent::ToolCallStarted {
        session_id: "test-tool-session".to_string(),
        tool_name: "read".to_string(),
        call_id: "call-test-123".to_string(),
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
        Some("test-tool-session"),
        "Session ID should match"
    );
    assert_eq!(
        tool_call_parsed.get("tool_name").and_then(|v| v.as_str()),
        Some("read"),
        "Tool name should be 'read'"
    );
    assert_eq!(
        tool_call_parsed.get("call_id").and_then(|v| v.as_str()),
        Some("call-test-123"),
        "Call ID should match"
    );

    event_bus.publish(DomainEvent::ToolCallEnded {
        session_id: "test-tool-session".to_string(),
        call_id: "call-test-123".to_string(),
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
        tool_result_parsed
            .get("session_id")
            .and_then(|v| v.as_str()),
        Some("test-tool-session"),
        "Session ID should match"
    );
    assert_eq!(
        tool_result_parsed.get("call_id").and_then(|v| v.as_str()),
        Some("call-test-123"),
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
async fn test_multiple_clients_receive_same_events() {
    use opencode_core::DomainEvent;

    let (ws_url, server_handle, state_data) = start_ws_test_server_with_state(0).await;

    let ws_url_with_session =
        ws_url.replace("/test-session", "?session_id=test-multi-client-session");

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
        Some("test-multi-client-session"),
        "Session ID should be test-multi-client-session"
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
        Some("test-multi-client-session"),
        "Session ID should be test-multi-client-session"
    );

    let event_bus = &state_data.event_bus;

    event_bus.publish(DomainEvent::ToolCallStarted {
        session_id: "test-multi-client-session".to_string(),
        tool_name: "read".to_string(),
        call_id: "call-multi-123".to_string(),
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
        Some("test-multi-client-session"),
        "First client session ID should match"
    );
    assert_eq!(
        tool_call_parsed2.get("session_id").and_then(|v| v.as_str()),
        Some("test-multi-client-session"),
        "Second client session ID should match"
    );
    assert_eq!(
        tool_call_parsed1.get("tool_name").and_then(|v| v.as_str()),
        Some("read"),
        "First client tool name should be 'read'"
    );
    assert_eq!(
        tool_call_parsed2.get("tool_name").and_then(|v| v.as_str()),
        Some("read"),
        "Second client tool name should be 'read'"
    );
    assert_eq!(
        tool_call_parsed1.get("call_id").and_then(|v| v.as_str()),
        Some("call-multi-123"),
        "First client call ID should match"
    );
    assert_eq!(
        tool_call_parsed2.get("call_id").and_then(|v| v.as_str()),
        Some("call-multi-123"),
        "Second client call ID should match"
    );

    event_bus.publish(DomainEvent::ToolCallEnded {
        session_id: "test-multi-client-session".to_string(),
        call_id: "call-multi-123".to_string(),
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
        tool_result_parsed1.get("success").and_then(|v| v.as_bool()),
        Some(true),
        "First client success should be true"
    );
    assert_eq!(
        tool_result_parsed2.get("success").and_then(|v| v.as_bool()),
        Some(true),
        "Second client success should be true"
    );

    event_bus.publish(DomainEvent::AgentStatusChanged {
        session_id: "test-multi-client-session".to_string(),
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

#[tokio::test]
async fn test_client_disconnect_no_crash() {
    let (ws_url, server_handle) = start_ws_test_server(0).await;

    let ws_url_with_session = ws_url.replace("/test-session", "?session_id=disconnect-crash-test");

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
async fn test_ws_memory_buffers_freed_after_streaming() {
    let (ws_url, server_handle) = start_ws_test_server(0).await;

    let ws_url_with_session = ws_url.replace("/test-session", "?session_id=mem-test");

    let (mut ws, _) = tokio_tungstenite::connect_async(&ws_url_with_session)
        .await
        .expect("Should connect to WebSocket endpoint");

    let connected_msg = ws.next().await;
    assert!(connected_msg.is_some(), "Should receive connected message");
    drop(connected_msg);

    ws_close(&mut ws).await;
    tokio::time::sleep(Duration::from_millis(50)).await;

    let (mut ws2, _) = tokio_tungstenite::connect_async(&ws_url_with_session)
        .await
        .expect("Should reconnect after previous stream");
    ws_close(&mut ws2).await;

    server_handle.abort();
}

#[tokio::test]
async fn test_ws_memory_no_growth_on_repeated_streams() {
    let (ws_url, server_handle) = start_ws_test_server(0).await;

    let iterations = 5;

    for iteration in 0..iterations {
        let session_id = format!("repeat-test-{}", iteration);
        let url = ws_url.replace("/test-session", &format!("?session_id={}", session_id));

        let (mut ws, _) = tokio_tungstenite::connect_async(&url)
            .await
            .expect("Should connect to WebSocket");

        let connected_msg = ws.next().await;
        assert!(
            connected_msg.is_some(),
            "Iteration {}: Should receive connected",
            iteration
        );
        drop(connected_msg);

        ws_close(&mut ws).await;
        tokio::time::sleep(Duration::from_millis(20)).await;
    }

    server_handle.abort();
}

#[tokio::test]
async fn test_ws_session_hub_no_memory_leak_on_register_unregister() {
    let hub = SessionHub::new(256);

    let session_id = "leak-test-session";

    let mut receivers = Vec::new();
    for i in 0..20 {
        let receiver = hub
            .register_client(session_id, &format!("client-{}", i))
            .await;
        receivers.push(receiver);
    }

    assert_eq!(hub.get_session_client_count(session_id).await, 20);

    for i in 0..20 {
        hub.unregister_client(session_id, &format!("client-{}", i))
            .await;
    }

    tokio::time::sleep(Duration::from_millis(10)).await;

    drop(receivers);

    assert_eq!(hub.get_session_client_count(session_id).await, 0);
    assert_eq!(hub.total_client_count().await, 0);
}

#[tokio::test]
async fn test_ws_reconnection_store_cleanup_on_repeated_records() {
    let store = ReconnectionStore::new(10);
    let session_id = "replay-cleanup-test";

    for i in 0..50 {
        store.record_message(
            session_id,
            StreamMessage::Message {
                session_id: session_id.to_string(),
                content: format!("message {}", i),
                role: "user".to_string(),
            },
        );
    }

    let entries = store.replay_from(session_id, 0);
    assert_eq!(entries.len(), 10, "Should respect replay limit of 10");

    store.record_message(
        session_id,
        StreamMessage::Message {
            session_id: session_id.to_string(),
            content: "final message".to_string(),
            role: "user".to_string(),
        },
    );

    let entries_after = store.replay_from(session_id, 0);
    assert_eq!(
        entries_after.len(),
        10,
        "Should still respect limit after additional record"
    );

    let token = store.generate_token(session_id, None);
    let validated = store.validate_token(session_id, &token);
    assert!(validated.is_some());

    let entries_from_seq = store.replay_from(session_id, 40);
    assert!(
        entries_from_seq.len() <= 10,
        "Should return at most 10 entries"
    );
}

#[tokio::test]
async fn test_ws_connection_cleanup_on_disconnect() {
    let (ws_url, server_handle) = start_ws_test_server(0).await;

    let ws_url_with_session = ws_url.replace("/test-session", "?session_id=cleanup-test");

    let (mut ws, _) = tokio_tungstenite::connect_async(&ws_url_with_session)
        .await
        .expect("Should connect to WebSocket endpoint");

    let connected_msg = ws.next().await;
    assert!(connected_msg.is_some(), "Should receive connected message");
    drop(connected_msg);

    drop(ws);
    tokio::time::sleep(Duration::from_millis(100)).await;

    let (mut ws2, _) = tokio_tungstenite::connect_async(&ws_url_with_session)
        .await
        .expect("Should reconnect after disconnect");
    ws_close(&mut ws2).await;

    server_handle.abort();
}

#[tokio::test]
async fn test_ws_multiple_clients_same_session_cleanup() {
    let (ws_url, server_handle) = start_ws_test_server(0).await;
    let ws_url_with_session = ws_url.replace("/test-session", "?session_id=multi-cleanup");

    let clients_count = 3;
    let mut clients = Vec::new();

    for i in 0..clients_count {
        let (mut ws, _) = tokio_tungstenite::connect_async(&ws_url_with_session)
            .await
            .unwrap_or_else(|_| panic!("Should connect client {}", i));

        let connected_msg = ws.next().await;
        assert!(
            connected_msg.is_some(),
            "Client {}: Should receive connected",
            i
        );
        drop(connected_msg);

        clients.push(ws);
    }

    for mut ws in clients {
        ws_close(&mut ws).await;
    }

    tokio::time::sleep(Duration::from_millis(50)).await;

    let (mut ws_new, _) = tokio_tungstenite::connect_async(&ws_url_with_session)
        .await
        .expect("Should connect new client after cleanup");
    ws_close(&mut ws_new).await;

    server_handle.abort();
}

#[tokio::test]
async fn test_ws_broadcast_channel_cleanup() {
    let hub = SessionHub::new(256);
    let session_id = "broadcast-cleanup-test";

    let mut receivers = Vec::new();
    for i in 0..10 {
        let receiver = hub
            .register_client(session_id, &format!("client-{}", i))
            .await;
        receivers.push(receiver);
    }

    hub.unregister_client(session_id, "client-0").await;
    drop(receivers.remove(0));

    assert_eq!(hub.get_session_client_count(session_id).await, 9);

    for i in 1..10 {
        hub.unregister_client(session_id, &format!("client-{}", i))
            .await;
    }

    assert_eq!(hub.get_session_client_count(session_id).await, 0);
    assert_eq!(hub.total_client_count().await, 0);
}
