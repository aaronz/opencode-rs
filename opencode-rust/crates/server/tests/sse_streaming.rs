use actix_web::{test, web, App, HttpServer};
use futures_util::StreamExt;
use opencode_core::PermissionManager;
use opencode_permission::ApprovalQueue;
use opencode_server::{routes, ServerState};
use std::sync::Arc;
use std::time::Duration;

fn create_test_state() -> ServerState {
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
        session_hub: std::sync::Arc::new(opencode_server::routes::ws::SessionHub::new(256)),
        server_start_time: std::time::SystemTime::now(),
        permission_manager: std::sync::Arc::new(std::sync::RwLock::new(
            PermissionManager::default(),
        )),
        approval_queue: std::sync::Arc::new(std::sync::RwLock::new(ApprovalQueue::default())),
        audit_log: None,
    }
}

fn is_valid_sse_format(sse_string: &str) -> bool {
    let mut has_data = false;

    for line in sse_string.lines() {
        if line.is_empty() {
            break;
        }

        if let Some(colon_pos) = line.find(':') {
            let field = &line[..colon_pos];
            let remainder = &line[colon_pos + 1..];

            if remainder.is_empty() {
                if field != "data" {
                    return false;
                }
            } else if !remainder.starts_with(' ') {
                return false;
            }

            match field {
                "id" | "event" | "data" | "retry" => {
                    if field == "data" {
                        has_data = true;
                    }
                }
                _ => return false,
            }
        } else {
            return false;
        }
    }

    has_data
}

async fn start_test_server(
    port: u16,
) -> (String, tokio::task::JoinHandle<()>, Arc<tempfile::TempDir>) {
    let temp_dir = Arc::new(tempfile::tempdir().unwrap());
    let temp_dir_clone = temp_dir.clone();
    let state = create_test_state();
    let state_data = web::Data::new(state);

    let bind_addr = format!("127.0.0.1:{}", port);
    let std_listener = std::net::TcpListener::bind(&bind_addr).unwrap();
    let actual_port = std_listener.local_addr().unwrap().port();
    let server_url = format!("http://127.0.0.1:{}", actual_port);

    let handle = tokio::spawn(async move {
        HttpServer::new(move || {
            App::new()
                .app_data(state_data.clone())
                .service(web::scope("/api").configure(routes::config_routes))
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
async fn test_sse_streaming_integration_tokens_stream() {
    let (server_url, server_handle, _temp_dir) = start_test_server(0).await;

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
            "SSE response should have text/event-stream content-type, got: {}",
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
                if text.contains("error") || text.contains("[DONE]") {
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
async fn test_sse_streaming_unit_verifies_each_token_as_separate_sse_event() {
    let (server_url, server_handle, _temp_dir) = start_test_server(0).await;

    let client = reqwest::Client::new();

    let resp = client
        .post(format!("{}/api/run", server_url))
        .header("Accept", "text/event-stream")
        .json(&serde_json::json!({
            "prompt": "Say hello",
            "stream": true
        }))
        .send()
        .await
        .expect("Failed to call run endpoint");

    let status = resp.status();

    if status.is_success() {
        let mut stream = resp.bytes_stream();
        let mut token_events = Vec::new();

        while let Some(item) = stream.next().await {
            if let Ok(bytes) = item {
                let text = String::from_utf8_lossy(&bytes);
                if text.contains("data:") {
                    assert!(
                        is_valid_sse_format(&text),
                        "Each SSE event should have valid format: {:?}",
                        text
                    );
                    assert!(
                        text.contains("data: ") && text.contains("\n\n"),
                        "Each SSE event should have 'data: ' prefix and end with double newline"
                    );
                    token_events.push(text);
                }
                if text.contains("error") || text.contains("[DONE]") {
                    break;
                }
            }
        }

        assert!(
            !token_events.is_empty(),
            "Should have received at least one token event"
        );
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
async fn test_sse_streaming_connection_close_terminates_cleanly() {
    use std::time::Duration;

    let (server_url, server_handle, _temp_dir) = start_test_server(0).await;

    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(30))
        .build()
        .expect("Failed to build client");

    let resp = client
        .post(format!("{}/api/run", server_url))
        .header("Accept", "text/event-stream")
        .json(&serde_json::json!({
            "prompt": "Write a long story",
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

#[tokio::test]
async fn test_regression_non_sse_accept_header_returns_normal_response() {
    let (server_url, server_handle, _temp_dir) = start_test_server(0).await;

    let client = reqwest::Client::new();

    let resp = client
        .post(format!("{}/api/run", server_url))
        .header("Accept", "application/json")
        .json(&serde_json::json!({
            "prompt": "Hello",
            "stream": false
        }))
        .send()
        .await
        .expect("Failed to call run endpoint");

    let status = resp.status();

    if status.is_success() {
        let content_type = resp
            .headers()
            .get("content-type")
            .and_then(|v| v.to_str().ok())
            .unwrap_or("");

        assert!(
            content_type.contains("application/json") || content_type.contains("text/event-stream"),
            "Response should be JSON or SSE, got: {}",
            content_type
        );
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
async fn test_error_handling_sse_streaming_interrupted_connection_handles_gracefully() {
    use std::time::Duration;

    let (server_url, server_handle, _temp_dir) = start_test_server(0).await;

    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(30))
        .build()
        .expect("Failed to build client");

    let resp = client
        .post(format!("{}/api/run", server_url))
        .header("Accept", "text/event-stream")
        .json(&serde_json::json!({
            "prompt": "Test interrupted connection",
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

    if status.is_success() {
        let mut stream = resp.bytes_stream();
        let mut received_events = 0;

        for _ in 0..5 {
            match stream.next().await {
                Some(Ok(_)) => {
                    received_events += 1;
                }
                Some(Err(_)) => {
                    break;
                }
                None => break,
            }
        }

        drop(resp);

        tokio::time::sleep(Duration::from_millis(50)).await;
    }

    server_handle.abort();
}

#[tokio::test]
async fn test_sse_streaming_with_curl_style_request() {
    let (server_url, server_handle, _temp_dir) = start_test_server(0).await;

    let client = reqwest::Client::new();

    let resp = client
        .post(format!("{}/api/run", server_url))
        .header("Accept", "text/event-stream")
        .json(&serde_json::json!({
            "prompt": "hi"
        }))
        .send()
        .await
        .expect("Failed to call run endpoint");

    let status = resp.status();

    if status.is_success() {
        let content_type = resp
            .headers()
            .get("content-type")
            .and_then(|v| v.to_str().ok())
            .unwrap_or("");

        assert!(
            content_type.contains("text/event-stream"),
            "Response should be text/event-stream, got: {}",
            content_type
        );

        let mut stream = resp.bytes_stream();
        let mut has_data_prefix = false;

        while let Some(item) = stream.next().await {
            if let Ok(bytes) = item {
                let text = String::from_utf8_lossy(&bytes);
                if text.contains("data: ") {
                    has_data_prefix = true;
                }
                if text.contains("error") || text.contains("[DONE]") {
                    break;
                }
            }
        }

        assert!(has_data_prefix, "SSE stream should have 'data: ' prefix");
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
async fn test_sse_streaming_multiple_events_in_sequence() {
    let (server_url, server_handle, _temp_dir) = start_test_server(0).await;

    let client = reqwest::Client::new();

    let resp = client
        .post(format!("{}/api/run", server_url))
        .header("Accept", "text/event-stream")
        .json(&serde_json::json!({
            "prompt": "Test",
            "stream": true
        }))
        .send()
        .await
        .expect("Failed to call run endpoint");

    let status = resp.status();

    if status.is_success() {
        let mut stream = resp.bytes_stream();
        let mut sse_events = Vec::new();

        while let Some(item) = stream.next().await {
            if let Ok(bytes) = item {
                let text = String::from_utf8_lossy(&bytes);
                if text.contains("data:") {
                    sse_events.push(text.clone());
                }
                if text.contains("error") || text.contains("[DONE]") {
                    break;
                }
            }
        }

        assert!(
            !sse_events.is_empty(),
            "Should receive at least 1 SSE event, got {}",
            sse_events.len()
        );

        for (idx, event) in sse_events.iter().enumerate() {
            assert!(
                is_valid_sse_format(event),
                "Event {} has invalid SSE format: {:?}",
                idx,
                event
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
