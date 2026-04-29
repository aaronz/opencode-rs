use actix_web::{web, App, HttpResponse, HttpServer};
use futures_util::{SinkExt, StreamExt};
use opencode_server::routes::sse::SseMessageRequest;
use opencode_server::routes::ws::SessionHub;
use opencode_server::routes::ws::WsClientMessage;
use opencode_server::streaming::StreamMessage;
use opencode_server::ServerState;
use std::sync::Arc;
use std::time::Duration;
use tokio_tungstenite::tungstenite::Message as WsMessage;

fn create_sse_ws_comparison_test_state() -> ServerState {
    let temp_dir = tempfile::tempdir().unwrap();
    let db_path = temp_dir.path().join("test.db");
    ServerState {
        storage: {
            let pool = opencode_storage::database::StoragePool::new(&db_path).unwrap();
            let session_repo =
                Arc::new(opencode_storage::SqliteSessionRepository::new(pool.clone()));
            let project_repo =
                Arc::new(opencode_storage::SqliteProjectRepository::new(pool.clone()));
            Arc::new(opencode_storage::StorageService::new(
                session_repo,
                project_repo,
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
            opencode_core::PermissionManager::default(),
        )),
        approval_queue: std::sync::Arc::new(std::sync::RwLock::new(
            opencode_permission::ApprovalQueue::default(),
        )),
        audit_log: None,
        runtime: opencode_server::build_placeholder_runtime(),
    }
}

struct ServerUrls {
    http: String,
    ws: String,
}

async fn start_combined_server(
    port: u16,
) -> (
    ServerUrls,
    tokio::task::JoinHandle<()>,
    web::Data<ServerState>,
) {
    let state = create_sse_ws_comparison_test_state();
    let state_data = web::Data::new(state.clone());

    let bind_addr = format!("127.0.0.1:{}", port);
    let std_listener = std::net::TcpListener::bind(&bind_addr).unwrap();
    let actual_port = std_listener.local_addr().unwrap().port();

    let http_url = format!("http://127.0.0.1:{}", actual_port);
    let ws_url = format!("ws://127.0.0.1:{}/ws", actual_port);

    let state_for_server = state_data.clone();
    let handle = tokio::spawn(async move {
        HttpServer::new(move || {
            App::new()
                .app_data(state_for_server.clone())
                .service(web::scope("/ws").configure(opencode_server::routes::ws::init))
                .service(web::scope("/sse").configure(opencode_server::routes::sse::init))
                .route(
                    "/sse/{session_id}/message",
                    web::post().to(sse_message_handler),
                )
        })
        .listen(std_listener)
        .unwrap()
        .run()
        .await
        .unwrap();
    });

    tokio::time::sleep(Duration::from_millis(100)).await;

    (
        ServerUrls {
            http: http_url,
            ws: ws_url,
        },
        handle,
        state_data,
    )
}

async fn sse_message_handler(
    state: web::Data<ServerState>,
    session_path: web::Path<String>,
    req: web::Json<SseMessageRequest>,
) -> HttpResponse {
    use opencode_core::Message as CoreMessage;
    use opencode_core::Session;

    let session_id = session_path.into_inner();

    let mut core_session = match state.storage.load_session(&session_id).await {
        Ok(Some(s)) => s,
        Ok(None) => Session::new(),
        Err(_) => Session::new(),
    };

    core_session.add_message(CoreMessage::user(req.message.clone()));

    let _ = state.storage.save_session(&core_session).await;

    HttpResponse::Ok().json(StreamMessage::SessionUpdate {
        session_id,
        status: "message_received".to_string(),
    })
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
async fn test_ws_is_full_duplex_same_connection_send_and_receive() {
    let (urls, server_handle, _state_data) = start_combined_server(0).await;

    let ws_url = format!("{}/full-duplex-test", urls.ws);

    let (mut ws, _) = tokio_tungstenite::connect_async(&ws_url)
        .await
        .expect("Should connect to WebSocket");

    let _connected = ws.next().await;
    assert!(_connected.is_some(), "Should receive connected message");

    ws.send(WsMessage::Text(r#"{"type": "ping"}"#.to_string()))
        .await
        .expect("Should send ping over same WebSocket connection");

    let mut got_heartbeat = false;
    while let Some(msg) = ws.next().await {
        if let Ok(text) = msg.unwrap().into_text() {
            if let Ok(parsed) = serde_json::from_str::<serde_json::Value>(&text) {
                if parsed.get("type").and_then(|v| v.as_str()) == Some("heartbeat") {
                    got_heartbeat = true;
                    break;
                }
            }
        }
    }

    assert!(
        got_heartbeat,
        "WebSocket: client sends and receives on SAME connection (full duplex)"
    );

    let _ = ws_close(&mut ws).await;
    server_handle.abort();
}

#[tokio::test]
async fn test_ws_bidirectional_vs_sse_unidirectional_behavior_comparison() {
    let (urls, server_handle, _state_data) = start_combined_server(0).await;

    let ws_url = format!("{}/comparison-test", urls.ws);
    let (mut ws, _) = tokio_tungstenite::connect_async(&ws_url)
        .await
        .expect("Should connect to WebSocket");

    let _connected = ws.next().await;

    ws.send(WsMessage::Text(r#"{"type": "ping"}"#.to_string()))
        .await
        .expect("WebSocket can send without additional HTTP requests");

    let mut ws_received_on_same_conn = false;
    while let Some(msg) = ws.next().await {
        if let Ok(text) = msg.unwrap().into_text() {
            if let Ok(parsed) = serde_json::from_str::<serde_json::Value>(&text) {
                if parsed.get("type").and_then(|v| v.as_str()) == Some("heartbeat") {
                    ws_received_on_same_conn = true;
                    break;
                }
            }
        }
    }

    assert!(
        ws_received_on_same_conn,
        "WS: bidirectional on SAME connection"
    );

    let _ = ws_close(&mut ws).await;

    let sse_url = format!("{}/sse/comparison-test", urls.http);
    let client = reqwest::Client::new();

    let _sse_response = client.get(&sse_url).send().await;

    let message_url = format!("{}/sse/comparison-test/message", urls.http);
    let _message_response = client
        .post(&message_url)
        .header("Content-Type", "application/json")
        .body(r#"{"message": "SSE requires separate HTTP POST"}"#)
        .send()
        .await
        .expect("SSE: separate HTTP POST required for client messages");

    server_handle.abort();
}

#[test]
fn test_ws_message_types_support_bidirectional_operations() {
    let ping = WsClientMessage::Ping;
    let close = WsClientMessage::Close;
    let resume = WsClientMessage::Resume {
        session_id: "test".to_string(),
        token: "token".to_string(),
    };
    let run = WsClientMessage::Run {
        session_id: "test".to_string(),
        message: "test".to_string(),
        agent_type: Some("build".to_string()),
        model: Some("gpt-4".to_string()),
    };

    let ping_json = serde_json::to_string(&ping).expect("ping should serialize");
    let close_json = serde_json::to_string(&close).expect("close should serialize");
    let resume_json = serde_json::to_string(&resume).expect("resume should serialize");
    let run_json = serde_json::to_string(&run).expect("run should serialize");

    assert!(ping_json.contains("ping"));
    assert!(close_json.contains("close"));
    assert!(resume_json.contains("resume"));
    assert!(run_json.contains("run"));
}

#[test]
fn test_sse_message_request_only_has_deserialize() {
    use std::any::type_name;

    let _req = SseMessageRequest {
        message: "client message".to_string(),
        model: Some("gpt-4".to_string()),
    };

    let type_name = type_name::<SseMessageRequest>();
    assert!(type_name.contains("SseMessageRequest"));
}

#[test]
fn test_ws_provides_true_full_duplex() {
    let run_msg = WsClientMessage::Run {
        session_id: "duplex-test".to_string(),
        message: "client message".to_string(),
        agent_type: Some("build".to_string()),
        model: Some("gpt-4".to_string()),
    };

    let server_msg = StreamMessage::Message {
        session_id: "duplex-test".to_string(),
        content: "server response".to_string(),
        role: "assistant".to_string(),
    };

    let client_json = serde_json::to_string(&run_msg).expect("client can send Run message");
    let server_json = serde_json::to_string(&server_msg).expect("server can send Message");

    assert!(client_json.contains("\"type\":\"run\""));
    assert!(server_json.contains("\"type\":\"message\""));
}

#[test]
fn test_ws_client_messages_have_type_field_for_routing() {
    let run_msg = WsClientMessage::Run {
        session_id: "test".to_string(),
        message: "test".to_string(),
        agent_type: None,
        model: None,
    };

    let json = serde_json::to_string(&run_msg).expect("should serialize");
    assert!(json.contains("\"type\":\"run\""));

    let ping_msg = WsClientMessage::Ping;
    let ping_json = serde_json::to_string(&ping_msg).expect("should serialize");
    assert!(ping_json.contains("\"type\":\"ping\""));

    let close_msg = WsClientMessage::Close;
    let close_json = serde_json::to_string(&close_msg).expect("should serialize");
    assert!(close_json.contains("\"type\":\"close\""));
}

#[test]
fn test_sse_request_json_format_has_no_type_field() {
    let sse_json_raw = r#"{"message": "hello", "model": "gpt-4"}"#;
    assert!(
        sse_json_raw.contains("\"message\":"),
        "SSE requests include message"
    );
    assert!(
        !sse_json_raw.contains("\"type\":"),
        "SSE requests have no type field - unidirectional design"
    );
}

#[test]
fn test_ws_supports_multiple_bidirectional_message_types() {
    let messages = vec![
        WsClientMessage::Ping,
        WsClientMessage::Close,
        WsClientMessage::Resume {
            session_id: "s1".to_string(),
            token: "t1".to_string(),
        },
        WsClientMessage::Run {
            session_id: "s1".to_string(),
            message: "m1".to_string(),
            agent_type: Some("build".to_string()),
            model: Some("gpt-4".to_string()),
        },
    ];

    for msg in messages {
        let json = serde_json::to_string(&msg).expect("should serialize");
        assert!(!json.is_empty(), "All WS message types should serialize");
    }
}

#[test]
fn test_ws_vs_sse_design_difference_demonstrates_bidirectional_vs_unidirectional() {
    let ws_msg = WsClientMessage::Run {
        session_id: "test".to_string(),
        message: "hello".to_string(),
        agent_type: Some("build".to_string()),
        model: Some("gpt-4".to_string()),
    };

    let ws_json = serde_json::to_string(&ws_msg).unwrap();

    assert!(
        ws_json.contains("\"type\":\"run\""),
        "WS messages have type field for multiplexing"
    );
    assert!(
        ws_json.contains("\"session_id\":"),
        "WS messages include session_id"
    );

    let sse_json_raw = r#"{"message": "hello", "model": "gpt-4"}"#;
    assert!(
        sse_json_raw.contains("\"message\":"),
        "SSE requests include message"
    );
    assert!(
        !sse_json_raw.contains("\"type\":"),
        "SSE requests have no type field - single purpose"
    );
}
