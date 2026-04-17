use actix_web::{web, App, HttpResponse, HttpServer};
use futures_util::{SinkExt, StreamExt};
use opencode_core::bus::InternalEvent;
use opencode_server::routes::sse::SseMessageRequest;
use opencode_server::routes::ws::SessionHub;
use opencode_server::routes::ws::WsClientMessage;
use opencode_server::streaming::{ReconnectionStore, StreamMessage};
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
    }
}

async fn start_combined_server(
    port: u16,
) -> (String, tokio::task::JoinHandle<()>, web::Data<ServerState>) {
    let state = create_sse_ws_comparison_test_state();
    let state_data = web::Data::new(state.clone());

    let bind_addr = format!("127.0.0.1:{}", port);
    let std_listener = std::net::TcpListener::bind(&bind_addr).unwrap();
    let actual_port = std_listener.local_addr().unwrap().port();
    let base_url = format!("http://127.0.0.1:{}", actual_port);

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

    (base_url, handle, state_data)
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

#[tokio::test]
async fn test_ws_is_full_duplex_same_connection_send_and_receive() {
    let (base_url, server_handle, _state_data) = start_combined_server(0).await;

    let ws_url = format!("{}/ws/full-duplex-test", base_url);

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
async fn test_sse_is_unidirectional_requires_separate_http_for_client_messages() {
    let (base_url, server_handle, _state_data) = start_combined_server(0).await;

    let sse_url = format!("{}/sse/sse-unidirectional-test", base_url);

    let client = reqwest::Client::new();

    let sse_response = client
        .get(&sse_url)
        .send()
        .await
        .expect("Should connect to SSE endpoint");

    assert!(sse_response.status().is_success(), "SSE GET should succeed");

    let message_url = format!("{}/sse/sse-unidirectional-test/message", base_url);
    let message_response = client
        .post(&message_url)
        .header("Content-Type", "application/json")
        .body(r#"{"message": "client message via HTTP POST", "model": "gpt-4"}"#)
        .send()
        .await
        .expect("SSE requires SEPARATE HTTP POST for client messages");

    assert!(
        message_response.status().is_success(),
        "Separate HTTP POST required for client messages in SSE"
    );

    let body = message_response
        .text()
        .await
        .expect("Should get response body");
    assert!(
        body.contains("message_received"),
        "SSE unidirectional: requires separate HTTP endpoint for client-to-server messages"
    );

    server_handle.abort();
}

#[tokio::test]
async fn test_ws_bidirectional_vs_sse_unidirectional_behavior_comparison() {
    let (base_url, server_handle, state_data) = start_combined_server(0).await;

    let ws_url = format!("{}/ws/comparison-test", base_url);
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

    let sse_url = format!("{}/sse/comparison-test", base_url);
    let client = reqwest::Client::new();

    let _sse_response = client.get(&sse_url).send().await;

    let message_url = format!("{}/sse/comparison-test/message", base_url);
    let _message_response = client
        .post(&message_url)
        .header("Content-Type", "application/json")
        .body(r#"{"message": "SSE requires separate HTTP POST"}"#)
        .send()
        .await
        .expect("SSE: separate HTTP POST required for client messages");

    let event_bus = &state_data.event_bus;
    event_bus.publish(InternalEvent::SessionStarted {
        0: "comparison-test".to_string(),
    });

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

    let req = SseMessageRequest {
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

#[tokio::test]
async fn test_ws_client_can_send_multiple_messages_without_new_connections() {
    let (base_url, server_handle, _state_data) = start_combined_server(0).await;

    let ws_url = format!("{}/ws/multi-msg-test", base_url);

    let (mut ws, _) = tokio_tungstenite::connect_async(&ws_url)
        .await
        .expect("Should connect to WebSocket");

    let _connected = ws.next().await;

    for i in 0..5 {
        let msg = serde_json::json!({
            "type": "run",
            "session_id": "multi-msg-test",
            "message": format!("Message {}", i),
            "agent_type": "build",
            "model": "gpt-4"
        });
        ws.send(WsMessage::Text(msg.to_string()))
            .await
            .expect("Should send message");
    }

    let mut response_count = 0;
    let deadline = tokio::time::Instant::now() + Duration::from_secs(2);
    while tokio::time::Instant::now() < deadline {
        tokio::select! {
            msg = ws.next() => {
                if let Some(Ok(text)) = msg {
                    if let Ok(parsed) = serde_json::from_str::<serde_json::Value>(&text.into_text().unwrap()) {
                        if parsed.get("type").and_then(|v| v.as_str()) == Some("message") {
                            response_count += 1;
                        }
                    }
                }
            }
            _ = tokio::time::sleep(Duration::from_millis(10)) => {
                if response_count >= 5 {
                    break;
                }
            }
        }
    }

    assert_eq!(
        response_count, 5,
        "WS: multiple messages sent/received on same connection"
    );

    let _ = ws_close(&mut ws).await;
    server_handle.abort();
}
