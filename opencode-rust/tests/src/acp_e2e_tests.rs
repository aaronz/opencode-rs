#[cfg(test)]
mod tests {
    use actix_web::{web, App, HttpServer};
    use futures_util::{SinkExt, StreamExt};
    use opencode_tools::ToolRegistry;
    use std::sync::Arc;
    use std::time::Duration;
    use tokio::time::timeout;
    use tokio_tungstenite::tungstenite::Message;

    fn create_test_server_state() -> opencode_server::ServerState {
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
            acp_enabled: true,
            acp_stream: opencode_control_plane::AcpEventStream::new().into(),
            acp_client_registry: std::sync::Arc::new(tokio::sync::RwLock::new(
                opencode_server::routes::acp_ws::AcpClientRegistry::new(),
            )),
            tool_registry: std::sync::Arc::new(ToolRegistry::new()),
        }
    }

    async fn start_test_server(port: u16) -> (String, tokio::task::JoinHandle<()>) {
        let state = create_test_server_state();
        let state_data = web::Data::new(state);

        let bind_addr = format!("127.0.0.1:{}", port);
        let std_listener = std::net::TcpListener::bind(&bind_addr).unwrap();
        let actual_port = std_listener.local_addr().unwrap().port();
        let server_url = format!("http://127.0.0.1:{}", actual_port);

        let handle = tokio::spawn(async move {
            HttpServer::new(move || {
                App::new()
                    .app_data(state_data.clone())
                    .service(web::scope("/api/acp").configure(opencode_server::routes::acp::init))
            })
            .listen(std_listener)
            .unwrap()
            .run()
            .await
            .unwrap();
        });

        tokio::time::sleep(Duration::from_millis(100)).await;

        (server_url, handle)
    }

    async fn start_ws_test_server(port: u16) -> (String, tokio::task::JoinHandle<()>) {
        let state = create_test_server_state();
        let state_data = web::Data::new(state);

        let bind_addr = format!("127.0.0.1:{}", port);
        let std_listener = std::net::TcpListener::bind(&bind_addr).unwrap();
        let actual_port = std_listener.local_addr().unwrap().port();
        let ws_url = format!("ws://127.0.0.1:{}/api/acpws", actual_port);

        let handle = tokio::spawn(async move {
            HttpServer::new(move || {
                App::new().app_data(state_data.clone()).service(
                    web::scope("/api/acpws").configure(opencode_server::routes::acp_ws::init),
                )
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

    #[derive(Debug, serde::Serialize, serde::Deserialize)]
    #[serde(tag = "type", rename_all = "snake_case")]
    enum ClientWsMessage {
        Handshake {
            version: String,
            client_id: String,
            capabilities: Vec<String>,
        },
        HandshakeAck {
            session_id: String,
            confirmed: bool,
        },
        EditorMessage {
            session_id: String,
            content: String,
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
        Status {
            status: String,
        },
        Ping,
        Pong,
        Close,
    }

    #[derive(Debug, serde::Serialize, serde::Deserialize)]
    #[serde(tag = "type", rename_all = "snake_case")]
    enum ServerWsMessage {
        HandshakeResponse {
            version: String,
            server_id: String,
            session_id: String,
            accepted: bool,
            error: Option<String>,
        },
        SessionMessage {
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
        StatusUpdate {
            session_id: String,
            status: String,
        },
        Heartbeat {
            timestamp: i64,
        },
        Error {
            code: String,
            message: String,
        },
        Connected {
            session_id: Option<String>,
        },
    }

    async fn ws_send(
        ws: &mut tokio_tungstenite::WebSocketStream<
            tokio_tungstenite::MaybeTlsStream<tokio::net::TcpStream>,
        >,
        msg: ClientWsMessage,
    ) {
        let text = serde_json::to_string(&msg).unwrap();
        ws.send(Message::Text(text.into())).await.unwrap();
    }

    async fn ws_recv_text(
        ws: &mut tokio_tungstenite::WebSocketStream<
            tokio_tungstenite::MaybeTlsStream<tokio::net::TcpStream>,
        >,
    ) -> String {
        let msg = ws.next().await.unwrap().unwrap();
        msg.into_text().unwrap().to_string()
    }

    async fn ws_recv_skip_heartbeat(
        ws: &mut tokio_tungstenite::WebSocketStream<
            tokio_tungstenite::MaybeTlsStream<tokio::net::TcpStream>,
        >,
    ) -> String {
        loop {
            let text = ws_recv_text(ws).await;
            if !text.contains("\"type\":\"heartbeat\"") {
                break text;
            }
        }
    }

    async fn ws_recv_until_tool_call(
        ws: &mut tokio_tungstenite::WebSocketStream<
            tokio_tungstenite::MaybeTlsStream<tokio::net::TcpStream>,
        >,
    ) -> String {
        loop {
            let text = ws_recv_text(ws).await;
            if text.contains("\"type\":\"tool_call\"")
                || text.contains("\"type\":\"session_message\"")
            {
                break text;
            }
        }
    }

    async fn ws_close(
        ws: &mut tokio_tungstenite::WebSocketStream<
            tokio_tungstenite::MaybeTlsStream<tokio::net::TcpStream>,
        >,
    ) {
        let _ = ws.send(Message::Close(None)).await;
    }

    // =========================================================================
    // Test 1: ACP server starts successfully with ACP enabled
    // =========================================================================

    #[tokio::test]
    async fn test_acp_server_starts_with_acp_enabled() {
        let (server_url, server_handle) = start_test_server(0).await;

        let client = reqwest::Client::new();
        let resp = client
            .get(format!("{}/api/acp/status", server_url))
            .send()
            .await
            .expect("Should be able to call ACP status endpoint");

        assert_eq!(
            resp.status().as_u16(),
            200,
            "ACP status should return 200 OK"
        );

        let body: serde_json::Value = resp.json().await.expect("Should parse JSON");
        assert_eq!(
            body["status"].as_str(),
            Some("ready"),
            "ACP should be ready"
        );
        assert_eq!(
            body["acp_enabled"].as_bool(),
            Some(true),
            "ACP should be enabled"
        );
        assert_eq!(
            body["version"].as_str(),
            Some("1.0"),
            "Version should be 1.0"
        );

        server_handle.abort();
    }

    // =========================================================================
    // Test 2: AcpTransportClient creates correctly
    // =========================================================================

    #[test]
    fn test_acp_transport_client_initialization() {
        use opencode_control_plane::{AcpConnectionState, AcpTransportClient};

        let client = AcpTransportClient::new(
            "ws://localhost:8080/acp".to_string(),
            "test-editor".to_string(),
        );

        assert_eq!(client.client_id(), "test-editor");
        assert_eq!(client.server_url(), "ws://localhost:8080/acp");
        assert_eq!(client.state(), &AcpConnectionState::Disconnected);
        assert!(client.session_id().is_none());
        assert!(!client.is_connected());
    }

    #[test]
    fn test_acp_transport_client_with_capabilities() {
        use opencode_control_plane::AcpTransportClient;

        let client = AcpTransportClient::new(
            "ws://localhost:8080/acp".to_string(),
            "test-editor".to_string(),
        )
        .with_capabilities(vec!["chat".to_string(), "tools".to_string()]);

        assert_eq!(client.client_info().capabilities, vec!["chat", "tools"]);
    }

    // =========================================================================
    // Test 3: ACP handshake completes via HTTP
    // =========================================================================

    #[tokio::test]
    async fn test_acp_http_handshake_completes_successfully() {
        let (server_url, server_handle) = start_test_server(0).await;

        let client = reqwest::Client::new();

        let resp = client
            .post(format!("{}/api/acp/handshake", server_url))
            .json(&serde_json::json!({
                "version": "1.0",
                "client_id": "test-client-e2e",
                "capabilities": ["chat", "tools"]
            }))
            .send()
            .await
            .expect("Should send handshake request");

        assert_eq!(
            resp.status().as_u16(),
            200,
            "Handshake should return 200 OK"
        );

        let body: serde_json::Value = resp.json().await.expect("Should parse JSON");
        assert_eq!(
            body["accepted"].as_bool(),
            Some(true),
            "Handshake should be accepted"
        );
        assert!(
            !body["session_id"].as_str().unwrap().is_empty(),
            "Session ID should be generated"
        );
        assert_eq!(
            body["server_id"].as_str(),
            Some("server"),
            "Server ID should be 'server'"
        );
        assert_eq!(
            body["version"].as_str(),
            Some("1.0"),
            "Version should be 1.0"
        );

        server_handle.abort();
    }

    #[tokio::test]
    async fn test_acp_http_handshake_version_mismatch_rejected() {
        let (server_url, server_handle) = start_test_server(0).await;

        let client = reqwest::Client::new();

        let resp = client
            .post(format!("{}/api/acp/handshake", server_url))
            .json(&serde_json::json!({
                "version": "2.0",
                "client_id": "test-client",
                "capabilities": []
            }))
            .send()
            .await
            .expect("Should send handshake request");

        assert_eq!(resp.status().as_u16(), 200);

        let body: serde_json::Value = resp.json().await.expect("Should parse JSON");
        assert_eq!(
            body["accepted"].as_bool(),
            Some(false),
            "Version mismatch should be rejected"
        );
        assert!(
            body["error"].as_str().unwrap().contains("Version mismatch"),
            "Error should mention version mismatch"
        );

        server_handle.abort();
    }

    #[tokio::test]
    async fn test_acp_http_handshake_ack_confirms_session() {
        let (server_url, server_handle) = start_test_server(0).await;

        let client = reqwest::Client::new();

        let resp = client
            .post(format!("{}/api/acp/handshake", server_url))
            .json(&serde_json::json!({
                "version": "1.0",
                "client_id": "ack-test-client",
                "capabilities": []
            }))
            .send()
            .await
            .unwrap();

        let handshake: serde_json::Value = resp.json().await.unwrap();
        let session_id = handshake["session_id"].as_str().unwrap().to_string();

        let ack_resp = client
            .post(format!("{}/api/acp/ack", server_url))
            .json(&serde_json::json!({
                "session_id": session_id,
                "confirmed": true
            }))
            .send()
            .await
            .expect("Should send ack");

        assert_eq!(ack_resp.status().as_u16(), 200);

        let ack_body: serde_json::Value = ack_resp.json().await.unwrap();
        assert_eq!(
            ack_body["confirmed"].as_bool(),
            Some(true),
            "Session should be confirmed"
        );

        server_handle.abort();
    }

    #[tokio::test]
    async fn test_acp_http_connect_establishes_connection() {
        let (server_url, server_handle) = start_test_server(0).await;

        let client = reqwest::Client::new();

        let resp = client
            .post(format!("{}/api/acp/connect", server_url))
            .json(&serde_json::json!({
                "url": "http://localhost:9000"
            }))
            .send()
            .await
            .expect("Should send connect request");

        assert_eq!(resp.status().as_u16(), 200);

        let body: serde_json::Value = resp.json().await.unwrap();
        assert_eq!(body["status"].as_str(), Some("connected"));
        assert!(body["connection_id"].as_str().is_some());

        server_handle.abort();
    }

    // =========================================================================
    // Test 4: ACP WebSocket connection and message exchange
    // =========================================================================

    #[tokio::test]
    async fn test_acp_ws_connection_establishes_and_handshake_completes() {
        let (ws_url, server_handle) = start_ws_test_server(0).await;

        let (mut ws, _) = tokio_tungstenite::connect_async(&ws_url)
            .await
            .expect("Should connect to ACP WebSocket");

        let connected_text = ws_recv_text(&mut ws).await;
        let connected: ServerWsMessage =
            serde_json::from_str(&connected_text).expect("Should parse Connected message");

        assert!(
            matches!(connected, ServerWsMessage::Connected { session_id } if session_id.is_none()),
            "Should receive Connected message"
        );

        ws_send(
            &mut ws,
            ClientWsMessage::Handshake {
                version: "1.0".to_string(),
                client_id: "ws-test-client".to_string(),
                capabilities: vec!["chat".to_string(), "tools".to_string()],
            },
        )
        .await;

        let resp_text = ws_recv_text(&mut ws).await;
        let handshake_resp: ServerWsMessage =
            serde_json::from_str(&resp_text).expect("Should parse HandshakeResponse");

        match handshake_resp {
            ServerWsMessage::HandshakeResponse {
                version,
                accepted,
                session_id,
                ..
            } => {
                assert_eq!(version, "1.0");
                assert!(accepted, "Handshake should be accepted");
                assert!(!session_id.is_empty(), "Session ID should be generated");
            }
            _ => panic!("Expected HandshakeResponse, got {:?}", resp_text),
        }

        ws_close(&mut ws).await;
        server_handle.abort();
    }

    #[tokio::test]
    async fn test_acp_ws_message_exchange_end_to_end() {
        let (ws_url, server_handle) = start_ws_test_server(0).await;

        let (mut ws, _) = tokio_tungstenite::connect_async(&ws_url)
            .await
            .expect("Should connect");

        let _connected_text = ws_recv_text(&mut ws).await;

        ws_send(
            &mut ws,
            ClientWsMessage::Handshake {
                version: "1.0".to_string(),
                client_id: "e2e-test-client".to_string(),
                capabilities: vec!["chat".to_string()],
            },
        )
        .await;

        let resp_text = ws_recv_text(&mut ws).await;
        let handshake_resp: ServerWsMessage = serde_json::from_str(&resp_text).unwrap();

        let ServerWsMessage::HandshakeResponse {
            session_id,
            accepted,
            ..
        } = handshake_resp
        else {
            panic!("Expected HandshakeResponse");
        };
        assert!(accepted);

        ws_send(
            &mut ws,
            ClientWsMessage::HandshakeAck {
                session_id: session_id.clone(),
                confirmed: true,
            },
        )
        .await;

        ws_send(
            &mut ws,
            ClientWsMessage::EditorMessage {
                session_id: session_id.clone(),
                content: "Hello from E2E test".to_string(),
            },
        )
        .await;

        let echo_text = ws_recv_skip_heartbeat(&mut ws).await;
        let echo: ServerWsMessage = serde_json::from_str(&echo_text).unwrap();

        match echo {
            ServerWsMessage::SessionMessage { content, role, .. } => {
                assert_eq!(content, "Message received by server");
                assert_eq!(role, "system");
            }
            _ => panic!("Expected SessionMessage, got {:?}", echo_text),
        }

        let tool_call = ClientWsMessage::ToolCall {
            session_id: session_id.clone(),
            tool_name: "read".to_string(),
            args: serde_json::json!({"path": "/tmp/test.txt"}),
            call_id: "call-001".to_string(),
        };
        ws_send(&mut ws, tool_call).await;

        let tool_text = ws_recv_until_tool_call(&mut ws).await;
        let tool_result: ServerWsMessage = serde_json::from_str(&tool_text).unwrap();

        match tool_result {
            ServerWsMessage::ToolCall {
                tool_name,
                call_id,
                session_id: sid,
                ..
            } => {
                assert_eq!(tool_name, "read");
                assert_eq!(call_id, "call-001");
                assert_eq!(sid, session_id);
            }
            ServerWsMessage::SessionMessage { .. } => {
                let next_text = ws_recv_until_tool_call(&mut ws).await;
                let next_result: ServerWsMessage = serde_json::from_str(&next_text).unwrap();
                match next_result {
                    ServerWsMessage::ToolCall {
                        tool_name, call_id, ..
                    } => {
                        assert_eq!(tool_name, "read");
                        assert_eq!(call_id, "call-001");
                    }
                    _ => panic!("Expected ToolCall, got {:?}", next_text),
                }
            }
            _ => panic!("Expected ToolCall or SessionMessage, got {:?}", tool_text),
        }

        ws_send(
            &mut ws,
            ClientWsMessage::ToolResult {
                session_id: session_id.clone(),
                call_id: "call-001".to_string(),
                output: "File content here".to_string(),
                success: true,
            },
        )
        .await;

        ws_close(&mut ws).await;
        server_handle.abort();
    }

    #[tokio::test]
    async fn test_acp_ws_status_update_roundtrip() {
        let (ws_url, server_handle) = start_ws_test_server(0).await;

        let (mut ws, _) = tokio_tungstenite::connect_async(&ws_url)
            .await
            .expect("Should connect");

        let _connected_text = ws_recv_text(&mut ws).await;

        ws_send(
            &mut ws,
            ClientWsMessage::Handshake {
                version: "1.0".to_string(),
                client_id: "event-test-client".to_string(),
                capabilities: vec![],
            },
        )
        .await;

        let resp_text = ws_recv_text(&mut ws).await;
        let handshake_resp: ServerWsMessage = serde_json::from_str(&resp_text).unwrap();

        let ServerWsMessage::HandshakeResponse {
            session_id,
            accepted,
            ..
        } = handshake_resp
        else {
            panic!("Expected HandshakeResponse");
        };
        assert!(accepted);

        ws_send(
            &mut ws,
            ClientWsMessage::HandshakeAck {
                session_id,
                confirmed: true,
            },
        )
        .await;

        ws_send(
            &mut ws,
            ClientWsMessage::Status {
                status: "running".to_string(),
            },
        )
        .await;

        let status_text = ws_recv_text(&mut ws).await;
        let status_update: ServerWsMessage = serde_json::from_str(&status_text).unwrap();

        match status_update {
            ServerWsMessage::StatusUpdate { status, .. } => {
                assert_eq!(status, "running");
            }
            _ => panic!("Expected StatusUpdate, got {:?}", status_text),
        }

        ws_close(&mut ws).await;
        server_handle.abort();
    }

    // =========================================================================
    // Test 5: Error handling - connection failures and timeouts
    // =========================================================================

    #[tokio::test]
    async fn test_acp_ws_invalid_handshake_version_rejected() {
        let (ws_url, server_handle) = start_ws_test_server(0).await;

        let (mut ws, _) = tokio_tungstenite::connect_async(&ws_url)
            .await
            .expect("Should connect");

        let _connected_text = ws_recv_text(&mut ws).await;

        ws_send(
            &mut ws,
            ClientWsMessage::Handshake {
                version: "99.0".to_string(),
                client_id: "bad-version-client".to_string(),
                capabilities: vec![],
            },
        )
        .await;

        let resp_text = ws_recv_text(&mut ws).await;
        let resp_msg: ServerWsMessage = serde_json::from_str(&resp_text).unwrap();

        match resp_msg {
            ServerWsMessage::HandshakeResponse {
                accepted, error, ..
            } => {
                assert!(!accepted, "Version mismatch should be rejected");
                assert!(
                    error.unwrap().contains("Version mismatch"),
                    "Error should mention version mismatch"
                );
            }
            _ => panic!("Expected HandshakeResponse"),
        }

        ws_close(&mut ws).await;
        server_handle.abort();
    }

    #[tokio::test]
    async fn test_acp_ws_editor_message_before_handshake_rejected() {
        let (ws_url, server_handle) = start_ws_test_server(0).await;

        let (mut ws, _) = tokio_tungstenite::connect_async(&ws_url)
            .await
            .expect("Should connect");

        let _connected_text = ws_recv_text(&mut ws).await;

        ws_send(
            &mut ws,
            ClientWsMessage::EditorMessage {
                session_id: "no-session".to_string(),
                content: "Should fail".to_string(),
            },
        )
        .await;

        let resp_text = ws_recv_text(&mut ws).await;
        let resp_msg: ServerWsMessage = serde_json::from_str(&resp_text).unwrap();

        match resp_msg {
            ServerWsMessage::Error { code, .. } => {
                assert!(
                    code == "invalid_session" || code == "handshake_not_completed",
                    "Should get session error, got {}",
                    code
                );
            }
            _ => {}
        }

        ws_close(&mut ws).await;
        server_handle.abort();
    }

    #[tokio::test]
    async fn test_acp_ws_heartbeat_ping_pong() {
        let (ws_url, server_handle) = start_ws_test_server(0).await;

        let (mut ws, _) = tokio_tungstenite::connect_async(&ws_url)
            .await
            .expect("Should connect");

        let _connected_text = ws_recv_text(&mut ws).await;

        ws.send(Message::Ping("heartbeat".as_bytes().to_vec().into()))
            .await
            .expect("Should send ping");

        let resp = ws.next().await.unwrap().unwrap();
        assert!(
            matches!(resp, Message::Pong(_)),
            "Should receive pong response"
        );

        ws_close(&mut ws).await;
        server_handle.abort();
    }

    #[tokio::test]
    async fn test_acp_ws_close_message_closes_connection() {
        let (ws_url, server_handle) = start_ws_test_server(0).await;

        let (mut ws, _) = tokio_tungstenite::connect_async(&ws_url)
            .await
            .expect("Should connect");

        let _connected_text = ws_recv_text(&mut ws).await;

        ws_send(&mut ws, ClientWsMessage::Close).await;

        let resp = timeout(Duration::from_secs(5), ws.next()).await;
        assert!(resp.is_ok(), "Should receive close acknowledgment");

        server_handle.abort();
    }

    #[tokio::test]
    async fn test_acp_ws_tool_call_and_result_roundtrip() {
        let (ws_url, server_handle) = start_ws_test_server(0).await;

        let (mut ws, _) = tokio_tungstenite::connect_async(&ws_url)
            .await
            .expect("Should connect");

        let _connected_text = ws_recv_text(&mut ws).await;

        ws_send(
            &mut ws,
            ClientWsMessage::Handshake {
                version: "1.0".to_string(),
                client_id: "tool-roundtrip-client".to_string(),
                capabilities: vec!["tools".to_string()],
            },
        )
        .await;

        let resp_text = ws_recv_text(&mut ws).await;
        let ServerWsMessage::HandshakeResponse {
            session_id,
            accepted,
            ..
        } = serde_json::from_str(&resp_text).unwrap()
        else {
            panic!("Expected HandshakeResponse");
        };
        assert!(accepted);

        ws_send(
            &mut ws,
            ClientWsMessage::HandshakeAck {
                session_id: session_id.clone(),
                confirmed: true,
            },
        )
        .await;

        let call_id = "call-e2e-123";
        let tool_call = ClientWsMessage::ToolCall {
            session_id: session_id.clone(),
            tool_name: "grep".to_string(),
            args: serde_json::json!({
                "pattern": "fn main",
                "path": "/src/main.rs"
            }),
            call_id: call_id.to_string(),
        };
        ws_send(&mut ws, tool_call).await;

        let server_call_text = ws_recv_text(&mut ws).await;
        let ServerWsMessage::ToolCall {
            tool_name,
            call_id: resp_call_id,
            args,
            session_id: sid,
        } = serde_json::from_str(&server_call_text).unwrap()
        else {
            panic!("Expected ToolCall");
        };

        assert_eq!(tool_name, "grep");
        assert_eq!(resp_call_id, call_id);
        assert_eq!(sid, session_id);
        assert_eq!(args["pattern"], "fn main");

        ws_send(
            &mut ws,
            ClientWsMessage::ToolResult {
                session_id: session_id.clone(),
                call_id: call_id.to_string(),
                output: "Found 3 matches in main.rs".to_string(),
                success: true,
            },
        )
        .await;

        ws_close(&mut ws).await;
        server_handle.abort();
    }

    #[tokio::test]
    async fn test_acp_ws_connection_refused_on_invalid_port() {
        let result = tokio_tungstenite::connect_async("ws://127.0.0.1:1/acpws").await;

        assert!(result.is_err(), "Connection to invalid port should fail");
    }

    #[tokio::test]
    async fn test_acp_multiple_client_connections() {
        let (ws_url, server_handle) = start_ws_test_server(0).await;

        let (mut ws1, _) = tokio_tungstenite::connect_async(&ws_url)
            .await
            .expect("Client 1 should connect");
        let _connected1 = ws_recv_text(&mut ws1).await;

        let (mut ws2, _) = tokio_tungstenite::connect_async(&ws_url)
            .await
            .expect("Client 2 should connect");
        let _connected2 = ws_recv_text(&mut ws2).await;

        ws_send(
            &mut ws1,
            ClientWsMessage::Handshake {
                version: "1.0".to_string(),
                client_id: "multi-client-1".to_string(),
                capabilities: vec![],
            },
        )
        .await;
        ws_send(
            &mut ws2,
            ClientWsMessage::Handshake {
                version: "1.0".to_string(),
                client_id: "multi-client-2".to_string(),
                capabilities: vec![],
            },
        )
        .await;

        let resp1_text = ws_recv_skip_heartbeat(&mut ws1).await;
        let resp2_text = ws_recv_skip_heartbeat(&mut ws2).await;

        let ServerWsMessage::HandshakeResponse { accepted: a1, .. } =
            serde_json::from_str(&resp1_text).unwrap()
        else {
            panic!("Expected HandshakeResponse");
        };
        let ServerWsMessage::HandshakeResponse { accepted: a2, .. } =
            serde_json::from_str(&resp2_text).unwrap()
        else {
            panic!("Expected HandshakeResponse");
        };

        assert!(a1, "Client 1 handshake should succeed");
        assert!(a2, "Client 2 handshake should succeed");

        ws_close(&mut ws1).await;
        ws_close(&mut ws2).await;
        server_handle.abort();
    }

    #[tokio::test]
    async fn test_acp_transport_client_full_lifecycle() {
        use opencode_control_plane::{
            AcpConnectionState, AcpIncomingMessage, AcpOutgoingMessage, AcpTransportClient,
        };

        let server_url = "ws://localhost:8080/acp".to_string();
        let client_id = "lifecycle-test-client".to_string();

        let mut client = AcpTransportClient::new(server_url.clone(), client_id.clone());

        assert_eq!(client.connection_id(), client.connection_id());
        assert_eq!(client.server_url(), &server_url);
        assert_eq!(client.client_id(), &client_id);
        assert!(client.session_id().is_none());
        assert!(!client.is_connected());
        assert_eq!(client.state(), &AcpConnectionState::Disconnected);

        client.update_state(AcpConnectionState::Connecting);
        assert_eq!(client.state(), &AcpConnectionState::Connecting);

        client.update_state(AcpConnectionState::HandshakeSent);

        let response = AcpIncomingMessage::HandshakeResponse {
            version: "1.0".to_string(),
            server_id: "test-server".to_string(),
            session_id: "session-lifecycle-123".to_string(),
            accepted: true,
            error: None,
        };
        let event = client.handle_incoming_message(response);
        assert!(event.is_some());
        assert_eq!(client.session_id(), Some("session-lifecycle-123"));
        assert_eq!(client.state(), &AcpConnectionState::HandshakeConfirmed);

        client.update_state(AcpConnectionState::Connected);
        assert!(client.is_connected());

        let ack_msg = client.create_handshake_ack("session-lifecycle-123");
        match ack_msg {
            AcpOutgoingMessage::HandshakeAck {
                session_id,
                confirmed,
            } => {
                assert_eq!(session_id, "session-lifecycle-123");
                assert!(confirmed);
            }
            _ => panic!("Expected HandshakeAck"),
        }

        let editor_msg = client.create_editor_message("session-lifecycle-123", "test content");
        match editor_msg {
            AcpOutgoingMessage::EditorMessage { content, .. } => {
                assert_eq!(content, "test content");
            }
            _ => panic!("Expected EditorMessage"),
        }

        let tool_result =
            client.create_tool_result("session-lifecycle-123", "call-1", "output", true);
        match tool_result {
            AcpOutgoingMessage::ToolResult {
                call_id,
                output,
                success,
                ..
            } => {
                assert_eq!(call_id, "call-1");
                assert_eq!(output, "output");
                assert!(success);
            }
            _ => panic!("Expected ToolResult"),
        }
    }

    #[tokio::test]
    async fn test_acp_transport_client_connection_manager() {
        use opencode_control_plane::{AcpConnectionManager, AcpTransportClient};

        let mut manager = AcpConnectionManager::new();

        let client1 = AcpTransportClient::new(
            "ws://localhost:8080/acp".to_string(),
            "manager-client-1".to_string(),
        );
        let conn_id1 = client1.connection_id().to_string();

        let client2 = AcpTransportClient::new(
            "ws://localhost:8080/acp".to_string(),
            "manager-client-2".to_string(),
        );
        let conn_id2 = client2.connection_id().to_string();

        manager.register(conn_id1.clone(), client1);
        assert_eq!(manager.active_connections(), 1);

        manager.register(conn_id2.clone(), client2);
        assert_eq!(manager.active_connections(), 2);

        assert!(manager.get(&conn_id1).is_some());
        assert!(manager.get(&conn_id2).is_some());
        assert_eq!(manager.connection_ids().len(), 2);

        let removed = manager.unregister(&conn_id1);
        assert!(removed.is_some());
        assert_eq!(manager.active_connections(), 1);
        assert!(manager.get(&conn_id1).is_none());
        assert!(manager.get(&conn_id2).is_some());
    }

    #[tokio::test]
    async fn test_acp_handshake_response_rejected_updates_state() {
        use opencode_control_plane::{AcpConnectionState, AcpIncomingMessage, AcpTransportClient};

        let mut client = AcpTransportClient::new(
            "ws://localhost:8080/acp".to_string(),
            "rejected-client".to_string(),
        );

        client.update_state(AcpConnectionState::HandshakeSent);

        let response = AcpIncomingMessage::HandshakeResponse {
            version: "1.0".to_string(),
            server_id: "test-server".to_string(),
            session_id: String::new(),
            accepted: false,
            error: Some("Unauthorized client".to_string()),
        };

        let event = client.handle_incoming_message(response);
        assert!(event.is_some());

        let state = client.state();
        match state {
            AcpConnectionState::Error(msg) => {
                assert!(msg.contains("Unauthorized"));
            }
            _ => panic!("Expected Error state after rejection, got {:?}", state),
        }
    }
}
