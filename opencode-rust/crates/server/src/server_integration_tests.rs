#[cfg(test)]
mod tests {
    use actix_web::http::StatusCode;
    use actix_web::test::TestRequest;
    use actix_web::Responder;

    #[actix_web::test]
    async fn test_health_check() {
        let req = TestRequest::default().to_http_request();
        let resp = crate::health_check().await.respond_to(&req);
        assert_eq!(resp.status(), StatusCode::OK);
    }

    fn create_test_state() -> crate::ServerState {
        let temp_dir = tempfile::tempdir().unwrap();
        let db_path = temp_dir.path().join("test.db");
        crate::ServerState {
            storage: std::sync::Arc::new(opencode_storage::StorageService::new(
                opencode_storage::database::StoragePool::new(&db_path).unwrap(),
            )),
            models: std::sync::Arc::new(opencode_llm::ModelRegistry::new()),
            config: std::sync::Arc::new(std::sync::RwLock::new(opencode_core::Config::default())),
            event_bus: opencode_core::bus::SharedEventBus::default(),
            reconnection_store: crate::streaming::ReconnectionStore::default(),
            connection_monitor: std::sync::Arc::new(
                crate::streaming::conn_state::ConnectionMonitor::new(),
            ),
            share_server: std::sync::Arc::new(std::sync::RwLock::new(
                crate::routes::share::ShareServer::with_default_config(),
            )),
            acp_enabled: true,
        }
    }

    fn create_test_state_with_api_key(api_key: Option<String>) -> crate::ServerState {
        let temp_dir = tempfile::tempdir().unwrap();
        let db_path = temp_dir.path().join("test.db");
        let mut config = opencode_core::Config::default();
        config.api_key = api_key;
        crate::ServerState {
            storage: std::sync::Arc::new(opencode_storage::StorageService::new(
                opencode_storage::database::StoragePool::new(&db_path).unwrap(),
            )),
            models: std::sync::Arc::new(opencode_llm::ModelRegistry::new()),
            config: std::sync::Arc::new(std::sync::RwLock::new(config)),
            event_bus: opencode_core::bus::SharedEventBus::default(),
            reconnection_store: crate::streaming::ReconnectionStore::default(),
            connection_monitor: std::sync::Arc::new(
                crate::streaming::conn_state::ConnectionMonitor::new(),
            ),
            share_server: std::sync::Arc::new(std::sync::RwLock::new(
                crate::routes::share::ShareServer::with_default_config(),
            )),
            acp_enabled: true,
        }
    }

    #[actix_web::test]
    async fn test_permission_reply_allows_allow_decision() {
        use actix_web::web;

        let req = TestRequest::default().to_http_request();
        let resp = crate::routes::session::permission_reply(
            web::Data::new(create_test_state()),
            web::Path::from(("test-session".to_string(), "test-req".to_string())),
            web::Json(crate::routes::session::PermissionReplyRequest {
                decision: "allow".to_string(),
            }),
        )
        .await
        .respond_to(&req);

        assert_eq!(resp.status(), StatusCode::OK);
    }

    #[actix_web::test]
    async fn test_permission_reply_allows_deny_decision() {
        use actix_web::web;

        let req = TestRequest::default().to_http_request();
        let resp = crate::routes::session::permission_reply(
            web::Data::new(create_test_state()),
            web::Path::from(("test-session".to_string(), "test-req".to_string())),
            web::Json(crate::routes::session::PermissionReplyRequest {
                decision: "deny".to_string(),
            }),
        )
        .await
        .respond_to(&req);

        assert_eq!(resp.status(), StatusCode::OK);
    }

    #[actix_web::test]
    async fn test_permission_reply_rejects_invalid_decision() {
        use actix_web::web;

        let req = TestRequest::default().to_http_request();
        let resp = crate::routes::session::permission_reply(
            web::Data::new(create_test_state()),
            web::Path::from(("test-session".to_string(), "test-req".to_string())),
            web::Json(crate::routes::session::PermissionReplyRequest {
                decision: "invalid".to_string(),
            }),
        )
        .await
        .respond_to(&req);

        assert_eq!(resp.status(), StatusCode::UNPROCESSABLE_ENTITY);
    }

    // Auth enforcement tests

    #[actix_web::test]
    async fn test_auth_no_api_key_allows_request() {
        use actix_web::web;

        let state = create_test_state_with_api_key(None);
        let req = TestRequest::default()
            .app_data(web::Data::new(state))
            .to_srv_request();
        let authorized = crate::middleware::is_api_key_authorized(&req);
        assert!(authorized, "Request should be allowed when no API key is configured");
    }

    #[actix_web::test]
    async fn test_auth_missing_header_returns_unauthorized() {
        use actix_web::web;

        let state = create_test_state_with_api_key(Some("test-api-key".to_string()));
        let req = TestRequest::default()
            .app_data(web::Data::new(state))
            .to_srv_request();
        let authorized = crate::middleware::is_api_key_authorized(&req);
        assert!(!authorized, "Request should be rejected when API key is configured but header is missing");
    }

    #[actix_web::test]
    async fn test_auth_invalid_credentials_returns_unauthorized() {
        use actix_web::web;

        let state = create_test_state_with_api_key(Some("test-api-key".to_string()));
        let req = TestRequest::default()
            .app_data(web::Data::new(state))
            .insert_header((actix_web::http::header::HeaderName::from_static("x-api-key"), "wrong-api-key"))
            .to_srv_request();
        let authorized = crate::middleware::is_api_key_authorized(&req);
        assert!(!authorized, "Request should be rejected when API key is invalid");
    }

    #[actix_web::test]
    async fn test_auth_valid_credentials_allows_request() {
        use actix_web::web;

        let state = create_test_state_with_api_key(Some("test-api-key".to_string()));
        let req = TestRequest::default()
            .app_data(web::Data::new(state))
            .insert_header((actix_web::http::header::HeaderName::from_static("x-api-key"), "test-api-key"))
            .to_srv_request();
        let authorized = crate::middleware::is_api_key_authorized(&req);
        assert!(authorized, "Request should be allowed when API key is valid");
    }

    #[actix_web::test]
    async fn test_auth_empty_api_key_allows_request() {
        use actix_web::web;

        let state = create_test_state_with_api_key(Some("".to_string()));
        let req = TestRequest::default()
            .app_data(web::Data::new(state))
            .to_srv_request();
        let authorized = crate::middleware::is_api_key_authorized(&req);
        assert!(authorized, "Request should be allowed when API key is empty string");
    }

    // Streaming tests for P1-11: Streaming endpoints

    #[tokio::test]
    async fn test_stream_message_serialization_roundtrip() {
        use crate::streaming::StreamMessage;

        let msg = StreamMessage::Message {
            session_id: "test-session".to_string(),
            content: "Hello, streaming!".to_string(),
            role: "user".to_string(),
        };

        let json = serde_json::to_string(&msg).expect("should serialize");
        let parsed: StreamMessage = serde_json::from_str(&json).expect("should deserialize");

        match parsed {
            StreamMessage::Message { session_id, content, role } => {
                assert_eq!(session_id, "test-session");
                assert_eq!(content, "Hello, streaming!");
                assert_eq!(role, "user");
            }
            _ => panic!("expected Message variant"),
        }
    }

    #[tokio::test]
    async fn test_stream_message_session_id_extraction() {
        use crate::streaming::StreamMessage;

        let msg = StreamMessage::ToolCall {
            session_id: "session-123".to_string(),
            tool_name: "read".to_string(),
            args: serde_json::json!({"path": "/test"}),
            call_id: "call-456".to_string(),
        };

        assert_eq!(msg.session_id(), Some("session-123"));
    }

    #[tokio::test]
    async fn test_reconnection_store_records_and_replays_messages() {
        use crate::streaming::{ReconnectionStore, StreamMessage};

        let store = ReconnectionStore::new(10);

        // Record some messages
        let seq1 = store.record_message(
            "session-x",
            StreamMessage::SessionUpdate {
                session_id: "session-x".to_string(),
                status: "started".to_string(),
            },
        );
        assert_eq!(seq1, 1);

        let seq2 = store.record_message(
            "session-x",
            StreamMessage::Message {
                session_id: "session-x".to_string(),
                content: "Hello".to_string(),
                role: "user".to_string(),
            },
        );
        assert_eq!(seq2, 2);

        // Replay from sequence 0 should return all messages
        let replayed = store.replay_from("session-x", 0);
        assert_eq!(replayed.len(), 2);

        // Replay from sequence 1 should return only the second message
        let replayed = store.replay_from("session-x", 1);
        assert_eq!(replayed.len(), 1);
        assert_eq!(replayed[0].sequence, 2);
    }

    #[tokio::test]
    async fn test_reconnection_store_token_generation_and_validation() {
        use crate::streaming::ReconnectionStore;

        let store = ReconnectionStore::new(10);

        // Generate a token
        let token = store.generate_token("session-y", Some(5));

        // Validate the token
        let validated = store.validate_token("session-y", &token);
        assert_eq!(validated, Some(5));

        // Invalid token should return None
        assert_eq!(store.validate_token("session-y", "invalid-token"), None);

        // Token for different session should return None
        assert_eq!(store.validate_token("session-z", &token), None);
    }

    #[tokio::test]
    async fn test_connection_monitor_registers_and_unregisters() {
        use crate::streaming::conn_state::{ConnectionMonitor, ConnectionType};

        let monitor = ConnectionMonitor::new();

        // Register a connection
        monitor
            .register_connection(
                "conn-test-1".to_string(),
                ConnectionType::Sse,
                "session-test".to_string(),
            )
            .await;

        // Verify connection is tracked
        let info = monitor.get_connection("conn-test-1").await;
        assert!(info.is_some());
        assert_eq!(info.unwrap().connection_type, ConnectionType::Sse);

        // Get stats
        let stats = monitor.get_stats().await;
        assert_eq!(stats.active_connections, 1);
        assert_eq!(stats.sse_connections, 1);

        // Unregister
        monitor
            .unregister_connection("conn-test-1", "test_complete")
            .await;

        let info = monitor.get_connection("conn-test-1").await;
        assert!(info.is_none());

        let stats = monitor.get_stats().await;
        assert_eq!(stats.active_connections, 0);
    }

    #[tokio::test]
    async fn test_connection_monitor_heartbeat_tracking() {
        use crate::streaming::conn_state::{ConnectionMonitor, ConnectionType};

        let monitor = ConnectionMonitor::new();

        monitor
            .register_connection(
                "conn-hb".to_string(),
                ConnectionType::WebSocket,
                "session-hb".to_string(),
            )
            .await;

        // Record failures
        monitor.heartbeat_failure("conn-hb").await;
        monitor.heartbeat_failure("conn-hb").await;

        let info = monitor.get_connection("conn-hb").await.unwrap();
        assert_eq!(info.heartbeat_failures, 2);

        // Record success - should reset failures
        monitor.heartbeat_success("conn-hb").await;

        let info = monitor.get_connection("conn-hb").await.unwrap();
        assert_eq!(info.heartbeat_failures, 0);
    }

    #[tokio::test]
    async fn test_connection_monitor_session_connections() {
        use crate::streaming::conn_state::{ConnectionMonitor, ConnectionType};

        let monitor = ConnectionMonitor::new();

        // Register multiple connections for same session
        monitor
            .register_connection(
                "conn-s1".to_string(),
                ConnectionType::Sse,
                "session-shared".to_string(),
            )
            .await;
        monitor
            .register_connection(
                "conn-s2".to_string(),
                ConnectionType::WebSocket,
                "session-shared".to_string(),
            )
            .await;
        // And one for different session
        monitor
            .register_connection(
                "conn-s3".to_string(),
                ConnectionType::Sse,
                "session-other".to_string(),
            )
            .await;

        let shared_conns = monitor.get_session_connections("session-shared").await;
        assert_eq!(shared_conns.len(), 2);

        let other_conns = monitor.get_session_connections("session-other").await;
        assert_eq!(other_conns.len(), 1);
    }

    #[tokio::test]
    async fn test_stream_message_from_internal_event() {
        use crate::streaming::StreamMessage;
        use opencode_core::bus::InternalEvent;

        // Test MessageAdded event conversion
        let event = InternalEvent::MessageAdded {
            session_id: "evt-session".to_string(),
            message_id: "msg-1".to_string(),
        };

        let msg = StreamMessage::from_internal_event(&event);
        assert!(msg.is_some());

        let msg = msg.unwrap();
        assert_eq!(msg.session_id(), Some("evt-session"));

        // Test AgentStatusChanged event
        let event = InternalEvent::AgentStatusChanged {
            session_id: "evt-session-2".to_string(),
            status: "running".to_string(),
        };

        let msg = StreamMessage::from_internal_event(&event);
        assert!(msg.is_some());

        match msg.unwrap() {
            StreamMessage::SessionUpdate { session_id, status } => {
                assert_eq!(session_id, "evt-session-2");
                assert_eq!(status, "running");
            }
            _ => panic!("expected SessionUpdate"),
        }
    }

    #[tokio::test]
    async fn test_error_stream_message_format() {
        use crate::streaming::StreamMessage;

        let error = StreamMessage::Error {
            session_id: Some("err-session".to_string()),
            code: "TEST_ERROR".to_string(),
            message: "This is a test error".to_string(),
        };

        let json = serde_json::to_value(&error).expect("should serialize");
        assert_eq!(json["type"], "error");
        assert_eq!(json["code"], "TEST_ERROR");
        assert_eq!(json["message"], "This is a test error");
        assert_eq!(json["session_id"], "err-session");
    }

    #[tokio::test]
    async fn test_reconnection_store_respects_replay_limit() {
        use crate::streaming::{ReconnectionStore, StreamMessage};

        let store = ReconnectionStore::new(3); // Small limit

        // Add more messages than the limit
        for i in 0..5 {
            store.record_message(
                "session-limit",
                StreamMessage::Message {
                    session_id: "session-limit".to_string(),
                    content: format!("message {}", i),
                    role: "user".to_string(),
                },
            );
        }

        // Should only have 3 messages (the most recent 3)
        let replayed = store.replay_from("session-limit", 0);
        assert_eq!(replayed.len(), 3);
        assert_eq!(replayed[0].sequence, 3);
        assert_eq!(replayed[1].sequence, 4);
        assert_eq!(replayed[2].sequence, 5);
    }

    #[tokio::test]
    async fn test_connection_monitor_multiple_connection_types() {
        use crate::streaming::conn_state::{ConnectionMonitor, ConnectionType};

        let monitor = ConnectionMonitor::new();

        monitor
            .register_connection("c1".to_string(), ConnectionType::Sse, "s1".to_string())
            .await;
        monitor
            .register_connection("c2".to_string(), ConnectionType::Sse, "s1".to_string())
            .await;
        monitor
            .register_connection("c3".to_string(), ConnectionType::WebSocket, "s1".to_string())
            .await;

        let stats = monitor.get_stats().await;
        assert_eq!(stats.total_connections, 3);
        assert_eq!(stats.active_connections, 3);
        assert_eq!(stats.sse_connections, 2);
        assert_eq!(stats.ws_connections, 1);

        // Unregister one SSE connection
        monitor.unregister_connection("c1", "test").await;

        let stats = monitor.get_stats().await;
        assert_eq!(stats.active_connections, 2);
        assert_eq!(stats.sse_connections, 1);
        assert_eq!(stats.ws_connections, 1);
    }

    #[tokio::test]
    async fn test_stream_message_heartbeat_variant() {
        use crate::streaming::StreamMessage;

        let heartbeat = StreamMessage::Heartbeat { timestamp: 1234567890 };

        // Heartbeat has no session_id
        assert_eq!(heartbeat.session_id(), None);

        let json = serde_json::to_string(&heartbeat).expect("should serialize");
        assert!(json.contains("\"type\":\"heartbeat\""));
    }

    #[tokio::test]
    async fn test_stream_message_connected_variant() {
        use crate::streaming::StreamMessage;

        let connected = StreamMessage::Connected {
            session_id: Some("new-session".to_string()),
        };

        assert_eq!(connected.session_id(), None); // Connected has no session_id in session_id()
    }

    #[tokio::test]
    async fn test_connection_monitor_active_connections_filter() {
        use crate::streaming::conn_state::{ConnectionMonitor, ConnectionType};

        let monitor = ConnectionMonitor::new();

        monitor
            .register_connection("active1".to_string(), ConnectionType::Sse, "s1".to_string())
            .await;
        monitor
            .register_connection("active2".to_string(), ConnectionType::WebSocket, "s1".to_string())
            .await;

        let active = monitor.get_active_connections().await;
        assert_eq!(active.len(), 2);
    }
}
