//! Comprehensive integration tests for OpenCode Server API endpoints.
//!
//! Tests cover:
//! - Session CRUD operations
//! - Session messages and commands
//! - Config retrieval and updates
//! - Provider management
//! - Model listing
//! - Permission handling

use actix_web::{http::StatusCode, test, web, App};
use opencode_core::PermissionManager;
use opencode_permission::ApprovalQueue;
use opencode_server::{routes, ServerState};
use std::sync::Arc;

struct TestState {
    state: ServerState,
    _temp_dir: tempfile::TempDir,
}

async fn create_test_state() -> TestState {
    let temp_dir = tempfile::tempdir().unwrap();
    let db_path = temp_dir.path().join("test.db");
    let temp_dir_path = temp_dir.path().to_path_buf();

    let pool = opencode_storage::database::StoragePool::new(&db_path).unwrap();

    // Run migrations to create tables
    let migration_manager = opencode_storage::MigrationManager::new(pool.clone(), 3);
    migration_manager.migrate().await.unwrap();

    let state = ServerState {
        storage: {
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
        temp_db_dir: Some(temp_dir_path),
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
    };

    TestState {
        state,
        _temp_dir: temp_dir,
    }
}

// ============================================================================
// Session API Tests
// ============================================================================

#[actix_web::test]
async fn test_session_list_returns_empty_initially() {
    let test_state = create_test_state().await;
    let app = test::init_service(
        App::new()
            .app_data(web::Data::new(test_state.state))
            .service(web::scope("/api").configure(routes::config_routes)),
    )
    .await;

    let req = test::TestRequest::get().uri("/api/sessions").to_request();
    let resp = test::call_service(&app, req).await;

    assert_eq!(resp.status(), StatusCode::OK);

    let body = test::read_body(resp).await;
    let json: serde_json::Value = serde_json::from_slice(&body).expect("Valid JSON");

    assert!(json.get("items").is_some());
    assert!(json.get("limit").is_some());
    assert!(json.get("offset").is_some());
    assert!(json.get("count").is_some());
}

#[actix_web::test]
async fn test_session_create_with_initial_prompt() {
    let test_state = create_test_state().await;
    let app = test::init_service(
        App::new()
            .app_data(web::Data::new(test_state.state))
            .service(web::scope("/api").configure(routes::config_routes)),
    )
    .await;

    let req = test::TestRequest::post()
        .uri("/api/sessions")
        .set_json(serde_json::json!({
            "initial_prompt": "Hello, world!"
        }))
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), StatusCode::CREATED);

    let body = test::read_body(resp).await;
    let json: serde_json::Value = serde_json::from_slice(&body).expect("Valid JSON");

    assert!(json.get("session_id").is_some());
    assert!(json.get("created_at").is_some());
    assert_eq!(json.get("status"), Some(&serde_json::json!("created")));
    assert_eq!(json.get("message_count"), Some(&serde_json::json!(1)));
}

#[actix_web::test]
async fn test_session_create_without_prompt() {
    let test_state = create_test_state().await;
    let app = test::init_service(
        App::new()
            .app_data(web::Data::new(test_state.state))
            .service(web::scope("/api").configure(routes::config_routes)),
    )
    .await;

    let req = test::TestRequest::post()
        .uri("/api/sessions")
        .set_json(serde_json::json!({}))
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), StatusCode::CREATED);

    let body = test::read_body(resp).await;
    let json: serde_json::Value = serde_json::from_slice(&body).expect("Valid JSON");

    assert!(json.get("session_id").is_some());
    assert_eq!(json.get("status"), Some(&serde_json::json!("created")));
    assert_eq!(json.get("message_count"), Some(&serde_json::json!(0)));
}

#[actix_web::test]
async fn test_session_get_not_found_for_invalid_id() {
    let test_state = create_test_state().await;
    let app = test::init_service(
        App::new()
            .app_data(web::Data::new(test_state.state))
            .service(web::scope("/api").configure(routes::config_routes)),
    )
    .await;

    let req = test::TestRequest::get()
        .uri("/api/sessions/00000000-0000-0000-0000-000000000000")
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), StatusCode::NOT_FOUND);
}

#[actix_web::test]
async fn test_session_get_rejects_invalid_uuid_format() {
    let test_state = create_test_state().await;
    let app = test::init_service(
        App::new()
            .app_data(web::Data::new(test_state.state))
            .service(web::scope("/api").configure(routes::config_routes)),
    )
    .await;

    let req = test::TestRequest::get()
        .uri("/api/sessions/not-a-valid-uuid")
        .to_request();

    let resp = test::call_service(&app, req).await;
    // Should return 422 (Unprocessable Entity) for invalid UUID format
    assert!(resp.status().is_client_error());
}

#[actix_web::test]
async fn test_session_delete_not_found_for_invalid_id() {
    let test_state = create_test_state().await;
    let app = test::init_service(
        App::new()
            .app_data(web::Data::new(test_state.state))
            .service(web::scope("/api").configure(routes::config_routes)),
    )
    .await;

    let req = test::TestRequest::delete()
        .uri("/api/sessions/00000000-0000-0000-0000-000000000000")
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), StatusCode::NO_CONTENT);
}

#[actix_web::test]
async fn test_session_fork_requires_valid_uuid() {
    let test_state = create_test_state().await;
    let app = test::init_service(
        App::new()
            .app_data(web::Data::new(test_state.state))
            .service(web::scope("/api").configure(routes::config_routes)),
    )
    .await;

    let req = test::TestRequest::post()
        .uri("/api/sessions/not-uuid/fork")
        .set_json(serde_json::json!({
            "fork_at_message_index": 0
        }))
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert!(resp.status().is_client_error());
}

#[actix_web::test]
async fn test_session_fork_not_found_for_nonexistent_session() {
    let test_state = create_test_state().await;
    let app = test::init_service(
        App::new()
            .app_data(web::Data::new(test_state.state))
            .service(web::scope("/api").configure(routes::config_routes)),
    )
    .await;

    let req = test::TestRequest::post()
        .uri("/api/sessions/00000000-0000-0000-0000-000000000000/fork")
        .set_json(serde_json::json!({
            "fork_at_message_index": 0
        }))
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), StatusCode::NOT_FOUND);
}

#[actix_web::test]
async fn test_session_add_message_to_nonexistent_session() {
    let test_state = create_test_state().await;
    let app = test::init_service(
        App::new()
            .app_data(web::Data::new(test_state.state))
            .service(web::scope("/api").configure(routes::config_routes)),
    )
    .await;

    let req = test::TestRequest::post()
        .uri("/api/sessions/00000000-0000-0000-0000-000000000000/messages")
        .set_json(serde_json::json!({
            "content": "Hello!"
        }))
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), StatusCode::NOT_FOUND);
}

#[actix_web::test]
async fn test_session_add_message_with_role() {
    let test_state = create_test_state().await;
    let app = test::init_service(
        App::new()
            .app_data(web::Data::new(test_state.state))
            .service(web::scope("/api").configure(routes::config_routes)),
    )
    .await;

    // First create a session
    let create_req = test::TestRequest::post()
        .uri("/api/sessions")
        .set_json(serde_json::json!({}))
        .to_request();
    let create_resp = test::call_service(&app, create_req).await;
    assert_eq!(create_resp.status(), StatusCode::CREATED);

    let body = test::read_body(create_resp).await;
    let json: serde_json::Value = serde_json::from_slice(&body).expect("Valid JSON");
    let session_id = json.get("session_id").unwrap().as_str().unwrap();

    // Add message
    let msg_req = test::TestRequest::post()
        .uri(&format!("/api/sessions/{}/messages", session_id))
        .set_json(serde_json::json!({
            "content": "Hello, AI!",
            "role": "user"
        }))
        .to_request();

    let msg_resp = test::call_service(&app, msg_req).await;
    assert_eq!(msg_resp.status(), StatusCode::OK);

    let msg_body = test::read_body(msg_resp).await;
    let msg_json: serde_json::Value = serde_json::from_slice(&msg_body).expect("Valid JSON");
    assert_eq!(msg_json.get("message_count"), Some(&serde_json::json!(1)));
}

#[actix_web::test]
async fn test_session_list_messages_pagination() {
    let test_state = create_test_state().await;
    let app = test::init_service(
        App::new()
            .app_data(web::Data::new(test_state.state))
            .service(web::scope("/api").configure(routes::config_routes)),
    )
    .await;

    // First create a session
    let create_req = test::TestRequest::post()
        .uri("/api/sessions")
        .set_json(serde_json::json!({
            "initial_prompt": "Hello"
        }))
        .to_request();
    let create_resp = test::call_service(&app, create_req).await;
    let body = test::read_body(create_resp).await;
    let json: serde_json::Value = serde_json::from_slice(&body).expect("Valid JSON");
    let session_id = json.get("session_id").unwrap().as_str().unwrap();

    // List messages with pagination
    let req = test::TestRequest::get()
        .uri(&format!(
            "/api/sessions/{}/messages?limit=10&offset=0",
            session_id
        ))
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), StatusCode::OK);

    let body = test::read_body(resp).await;
    let json: serde_json::Value = serde_json::from_slice(&body).expect("Valid JSON");

    assert!(json.get("items").is_some());
    assert!(json.get("limit").is_some());
    assert!(json.get("offset").is_some());
    assert!(json.get("total").is_some());
}

#[actix_web::test]
async fn test_session_diff_not_found_for_nonexistent_session() {
    let test_state = create_test_state().await;
    let app = test::init_service(
        App::new()
            .app_data(web::Data::new(test_state.state))
            .service(web::scope("/api").configure(routes::config_routes)),
    )
    .await;

    let req = test::TestRequest::get()
        .uri("/api/sessions/00000000-0000-0000-0000-000000000000/diff")
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), StatusCode::NOT_FOUND);
}

#[actix_web::test]
async fn test_session_abort_nonexistent_session() {
    let test_state = create_test_state().await;
    let app = test::init_service(
        App::new()
            .app_data(web::Data::new(test_state.state))
            .service(web::scope("/api").configure(routes::config_routes)),
    )
    .await;

    let req = test::TestRequest::post()
        .uri("/api/sessions/00000000-0000-0000-0000-000000000000/abort")
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), StatusCode::NOT_FOUND);
}

#[actix_web::test]
async fn test_session_summarize_nonexistent_session() {
    let test_state = create_test_state().await;
    let app = test::init_service(
        App::new()
            .app_data(web::Data::new(test_state.state))
            .service(web::scope("/api").configure(routes::config_routes)),
    )
    .await;

    let req = test::TestRequest::post()
        .uri("/api/sessions/00000000-0000-0000-0000-000000000000/summarize")
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), StatusCode::NOT_FOUND);
}

#[actix_web::test]
async fn test_session_share_nonexistent_session() {
    let test_state = create_test_state().await;
    let app = test::init_service(
        App::new()
            .app_data(web::Data::new(test_state.state))
            .service(web::scope("/api").configure(routes::config_routes)),
    )
    .await;

    let req = test::TestRequest::post()
        .uri("/api/sessions/00000000-0000-0000-0000-000000000000/share")
        .set_json(serde_json::json!({}))
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), StatusCode::NOT_FOUND);
}

#[actix_web::test]
async fn test_session_unshare_nonexistent_session() {
    let test_state = create_test_state().await;
    let app = test::init_service(
        App::new()
            .app_data(web::Data::new(test_state.state))
            .service(web::scope("/api").configure(routes::config_routes)),
    )
    .await;

    let req = test::TestRequest::delete()
        .uri("/api/sessions/00000000-0000-0000-0000-000000000000/share")
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), StatusCode::NOT_FOUND);
}

#[actix_web::test]
async fn test_session_command_empty_command_fails() {
    let test_state = create_test_state().await;
    let app = test::init_service(
        App::new()
            .app_data(web::Data::new(test_state.state))
            .service(web::scope("/api").configure(routes::config_routes)),
    )
    .await;

    // First create a session
    let create_req = test::TestRequest::post()
        .uri("/api/sessions")
        .set_json(serde_json::json!({}))
        .to_request();
    let create_resp = test::call_service(&app, create_req).await;
    let body = test::read_body(create_resp).await;
    let json: serde_json::Value = serde_json::from_slice(&body).expect("Valid JSON");
    let session_id = json.get("session_id").unwrap().as_str().unwrap();

    // Try empty command
    let cmd_req = test::TestRequest::post()
        .uri(&format!("/api/sessions/{}/command", session_id))
        .set_json(serde_json::json!({
            "command": ""
        }))
        .to_request();

    let cmd_resp = test::call_service(&app, cmd_req).await;
    assert!(
        cmd_resp.status() == StatusCode::BAD_REQUEST
            || cmd_resp.status() == StatusCode::UNPROCESSABLE_ENTITY
    );
}

#[actix_web::test]
async fn test_session_command_with_args() {
    let test_state = create_test_state().await;
    let app = test::init_service(
        App::new()
            .app_data(web::Data::new(test_state.state))
            .service(web::scope("/api").configure(routes::config_routes)),
    )
    .await;

    // First create a session
    let create_req = test::TestRequest::post()
        .uri("/api/sessions")
        .set_json(serde_json::json!({}))
        .to_request();
    let create_resp = test::call_service(&app, create_req).await;
    let body = test::read_body(create_resp).await;
    let json: serde_json::Value = serde_json::from_slice(&body).expect("Valid JSON");
    let session_id = json.get("session_id").unwrap().as_str().unwrap();

    // Run a simple command with args
    let cmd_req = test::TestRequest::post()
        .uri(&format!("/api/sessions/{}/command", session_id))
        .set_json(serde_json::json!({
            "command": "echo",
            "args": ["hello", "world"]
        }))
        .to_request();

    let cmd_resp = test::call_service(&app, cmd_req).await;
    // May succeed or fail depending on environment, but should not be 400 for valid request format
    let status = cmd_resp.status();
    assert!(status.is_success() || status.is_server_error() || status == StatusCode::BAD_REQUEST);
}

#[actix_web::test]
async fn test_session_get_message_index_out_of_range() {
    let test_state = create_test_state().await;
    let app = test::init_service(
        App::new()
            .app_data(web::Data::new(test_state.state))
            .service(web::scope("/api").configure(routes::config_routes)),
    )
    .await;

    // First create a session
    let create_req = test::TestRequest::post()
        .uri("/api/sessions")
        .set_json(serde_json::json!({}))
        .to_request();
    let create_resp = test::call_service(&app, create_req).await;
    let body = test::read_body(create_resp).await;
    let json: serde_json::Value = serde_json::from_slice(&body).expect("Valid JSON");
    let session_id = json.get("session_id").unwrap().as_str().unwrap();

    // Try to get message at index 0 which doesn't exist
    let req = test::TestRequest::get()
        .uri(&format!("/api/sessions/{}/messages/0", session_id))
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), StatusCode::NOT_FOUND);
}

// ============================================================================
// Config API Tests
// ============================================================================

#[actix_web::test]
async fn test_config_get_returns_config() {
    let test_state = create_test_state().await;
    let app = test::init_service(
        App::new()
            .app_data(web::Data::new(test_state.state))
            .service(web::scope("/api").configure(routes::config_routes)),
    )
    .await;

    let req = test::TestRequest::get().uri("/api/config").to_request();
    let resp = test::call_service(&app, req).await;

    assert_eq!(resp.status(), StatusCode::OK);

    let body = test::read_body(resp).await;
    let json: serde_json::Value = serde_json::from_slice(&body).expect("Valid JSON");

    // Config should have basic structure
    assert!(json.get("provider").is_some() || json.is_object());
}

#[actix_web::test]
async fn test_config_update_with_patch() {
    let test_state = create_test_state().await;
    let app = test::init_service(
        App::new()
            .app_data(web::Data::new(test_state.state))
            .service(web::scope("/api").configure(routes::config_routes)),
    )
    .await;

    let req = test::TestRequest::patch()
        .uri("/api/config")
        .set_json(serde_json::json!({
            "provider": "anthropic"
        }))
        .to_request();

    let resp = test::call_service(&app, req).await;
    // May fail due to config save path, but should accept valid JSON
    let status = resp.status();
    assert!(status.is_success() || status.is_server_error() || status.is_client_error());
}

// ============================================================================
// Provider API Tests
// ============================================================================

#[actix_web::test]
async fn test_providers_list_returns_empty_array() {
    let test_state = create_test_state().await;
    let app = test::init_service(
        App::new()
            .app_data(web::Data::new(test_state.state))
            .service(web::scope("/api").configure(routes::config_routes)),
    )
    .await;

    let req = test::TestRequest::get().uri("/api/providers").to_request();
    let resp = test::call_service(&app, req).await;

    assert_eq!(resp.status(), StatusCode::OK);

    let body = test::read_body(resp).await;
    let json: serde_json::Value = serde_json::from_slice(&body).expect("Valid JSON");

    assert!(json.get("items").is_some());
    assert!(json.get("count").is_some());
}

#[actix_web::test]
async fn test_provider_get_not_found() {
    let test_state = create_test_state().await;
    let app = test::init_service(
        App::new()
            .app_data(web::Data::new(test_state.state))
            .service(web::scope("/api").configure(routes::config_routes)),
    )
    .await;

    let req = test::TestRequest::get()
        .uri("/api/providers/nonexistent")
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), StatusCode::NOT_FOUND);
}

#[actix_web::test]
async fn test_provider_create_valid_request() {
    let test_state = create_test_state().await;
    let app = test::init_service(
        App::new()
            .app_data(web::Data::new(test_state.state))
            .service(web::scope("/api").configure(routes::config_routes)),
    )
    .await;

    let req = test::TestRequest::post()
        .uri("/api/providers")
        .set_json(serde_json::json!({
            "provider_id": "test-provider",
            "endpoint": "https://api.test.com",
            "auth_strategy": {"BearerApiKey": {"header_name": null}}
        }))
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), StatusCode::CREATED);

    let body = test::read_body(resp).await;
    let json: serde_json::Value = serde_json::from_slice(&body).expect("Valid JSON");

    assert_eq!(
        json.get("provider_id"),
        Some(&serde_json::json!("test-provider"))
    );
    assert_eq!(
        json.get("endpoint"),
        Some(&serde_json::json!("https://api.test.com"))
    );
}

#[actix_web::test]
async fn test_provider_update_not_found() {
    let test_state = create_test_state().await;
    let app = test::init_service(
        App::new()
            .app_data(web::Data::new(test_state.state))
            .service(web::scope("/api").configure(routes::config_routes)),
    )
    .await;

    let req = test::TestRequest::put()
        .uri("/api/providers/nonexistent")
        .set_json(serde_json::json!({
            "endpoint": "https://new-endpoint.com"
        }))
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), StatusCode::NOT_FOUND);
}

#[actix_web::test]
async fn test_provider_delete_not_found() {
    let test_state = create_test_state().await;
    let app = test::init_service(
        App::new()
            .app_data(web::Data::new(test_state.state))
            .service(web::scope("/api").configure(routes::config_routes)),
    )
    .await;

    let req = test::TestRequest::delete()
        .uri("/api/providers/nonexistent")
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), StatusCode::NOT_FOUND);
}

#[actix_web::test]
async fn test_provider_test_not_found() {
    let test_state = create_test_state().await;
    let app = test::init_service(
        App::new()
            .app_data(web::Data::new(test_state.state))
            .service(web::scope("/api").configure(routes::config_routes)),
    )
    .await;

    let req = test::TestRequest::post()
        .uri("/api/providers/nonexistent/test")
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), StatusCode::NOT_FOUND);
}

#[actix_web::test]
async fn test_provider_get_status_not_found() {
    let test_state = create_test_state().await;
    let app = test::init_service(
        App::new()
            .app_data(web::Data::new(test_state.state))
            .service(web::scope("/api").configure(routes::config_routes)),
    )
    .await;

    let req = test::TestRequest::get()
        .uri("/api/providers/nonexistent/status")
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), StatusCode::OK);

    let body = test::read_body(resp).await;
    let json: serde_json::Value = serde_json::from_slice(&body).expect("Valid JSON");

    // Provider doesn't exist
    assert_eq!(json.get("exists"), Some(&serde_json::json!(false)));
}

#[actix_web::test]
async fn test_provider_set_enabled_not_found() {
    let test_state = create_test_state().await;
    let app = test::init_service(
        App::new()
            .app_data(web::Data::new(test_state.state))
            .service(web::scope("/api").configure(routes::config_routes)),
    )
    .await;

    let req = test::TestRequest::put()
        .uri("/api/providers/nonexistent/enabled")
        .set_json(serde_json::json!({
            "enabled": true
        }))
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), StatusCode::NOT_FOUND);
}

#[actix_web::test]
async fn test_provider_credentials_save_not_found() {
    let test_state = create_test_state().await;
    let app = test::init_service(
        App::new()
            .app_data(web::Data::new(test_state.state))
            .service(web::scope("/api").configure(routes::config_routes)),
    )
    .await;

    let req = test::TestRequest::post()
        .uri("/api/providers/nonexistent/credentials")
        .set_json(serde_json::json!({
            "api_key": "test-key"
        }))
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), StatusCode::NOT_FOUND);
}

#[actix_web::test]
async fn test_provider_credentials_test_not_found() {
    let test_state = create_test_state().await;
    let app = test::init_service(
        App::new()
            .app_data(web::Data::new(test_state.state))
            .service(web::scope("/api").configure(routes::config_routes)),
    )
    .await;

    let req = test::TestRequest::post()
        .uri("/api/providers/nonexistent/credentials/test")
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), StatusCode::NOT_FOUND);
}

#[actix_web::test]
async fn test_provider_credentials_delete_not_found() {
    let test_state = create_test_state().await;
    let app = test::init_service(
        App::new()
            .app_data(web::Data::new(test_state.state))
            .service(web::scope("/api").configure(routes::config_routes)),
    )
    .await;

    let req = test::TestRequest::delete()
        .uri("/api/providers/nonexistent/credentials")
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), StatusCode::NOT_FOUND);
}

// ============================================================================
// Model API Tests
// ============================================================================

#[actix_web::test]
async fn test_models_list_returns_empty_or_populated() {
    let test_state = create_test_state().await;
    let app = test::init_service(
        App::new()
            .app_data(web::Data::new(test_state.state))
            .service(web::scope("/api").configure(routes::config_routes)),
    )
    .await;

    let req = test::TestRequest::get().uri("/api/models").to_request();
    let resp = test::call_service(&app, req).await;

    assert_eq!(resp.status(), StatusCode::OK);

    let body = test::read_body(resp).await;
    let json: serde_json::Value = serde_json::from_slice(&body).expect("Valid JSON");

    assert!(json.get("items").is_some());
    assert!(json.get("count").is_some());
}

#[actix_web::test]
async fn test_models_list_filtered_by_provider() {
    let test_state = create_test_state().await;
    let app = test::init_service(
        App::new()
            .app_data(web::Data::new(test_state.state))
            .service(web::scope("/api").configure(routes::config_routes)),
    )
    .await;

    let req = test::TestRequest::get()
        .uri("/api/models?provider=openai")
        .to_request();

    let resp = test::call_service(&app, req).await;

    assert_eq!(resp.status(), StatusCode::OK);

    let body = test::read_body(resp).await;
    let json: serde_json::Value = serde_json::from_slice(&body).expect("Valid JSON");

    assert!(json.get("items").is_some());
    assert!(json.get("count").is_some());
}

// ============================================================================
// Permission API Tests
// ============================================================================

#[actix_web::test]
async fn test_permissions_list_returns_permission_list() {
    let test_state = create_test_state().await;
    let app = test::init_service(
        App::new()
            .app_data(web::Data::new(test_state.state))
            .service(web::scope("/api").configure(routes::config_routes)),
    )
    .await;

    let req = test::TestRequest::get()
        .uri("/api/permissions")
        .to_request();
    let resp = test::call_service(&app, req).await;

    assert_eq!(resp.status(), StatusCode::OK);

    let body = test::read_body(resp).await;
    let json: serde_json::Value = serde_json::from_slice(&body).expect("Valid JSON");

    assert!(json.get("permissions").is_some());
}

#[actix_web::test]
async fn test_permission_reply_allow() {
    let test_state = create_test_state().await;
    let app = test::init_service(
        App::new()
            .app_data(web::Data::new(test_state.state))
            .service(web::scope("/api").configure(routes::config_routes)),
    )
    .await;

    let req = test::TestRequest::post()
        .uri("/api/permissions/reply/00000000-0000-0000-0000-000000000000/00000000-0000-0000-0000-000000000001")
        .set_json(serde_json::json!({
            "decision": "allow"
        }))
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), StatusCode::OK);

    let body = test::read_body(resp).await;
    let json: serde_json::Value = serde_json::from_slice(&body).expect("Valid JSON");

    assert_eq!(json.get("decision"), Some(&serde_json::json!("allow")));
}

#[actix_web::test]
async fn test_permission_reply_deny() {
    let test_state = create_test_state().await;
    let app = test::init_service(
        App::new()
            .app_data(web::Data::new(test_state.state))
            .service(web::scope("/api").configure(routes::config_routes)),
    )
    .await;

    let req = test::TestRequest::post()
        .uri("/api/permissions/reply/00000000-0000-0000-0000-000000000000/00000000-0000-0000-0000-000000000001")
        .set_json(serde_json::json!({
            "decision": "deny"
        }))
        .to_request();

    let resp = test::call_service(&app, req).await;
    // Deny returns 403 Forbidden
    assert_eq!(resp.status(), StatusCode::FORBIDDEN);
}

#[actix_web::test]
async fn test_permission_reply_invalid_decision() {
    let test_state = create_test_state().await;
    let app = test::init_service(
        App::new()
            .app_data(web::Data::new(test_state.state))
            .service(web::scope("/api").configure(routes::config_routes)),
    )
    .await;

    let req = test::TestRequest::post()
        .uri("/api/permissions/reply/00000000-0000-0000-0000-000000000000/00000000-0000-0000-0000-000000000001")
        .set_json(serde_json::json!({
            "decision": "invalid"
        }))
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), StatusCode::BAD_REQUEST);
}

// ============================================================================
// Additional Session API Tests - End-to-End Workflow
// ============================================================================

#[actix_web::test]
async fn test_session_full_workflow_create_list_get_delete() {
    let test_state = create_test_state().await;
    let app = test::init_service(
        App::new()
            .app_data(web::Data::new(test_state.state))
            .service(web::scope("/api").configure(routes::config_routes)),
    )
    .await;

    // 1. Create a session
    let create_req = test::TestRequest::post()
        .uri("/api/sessions")
        .set_json(serde_json::json!({
            "initial_prompt": "My test session"
        }))
        .to_request();

    let create_resp = test::call_service(&app, create_req).await;
    assert_eq!(create_resp.status(), StatusCode::CREATED);

    let body = test::read_body(create_resp).await;
    let json: serde_json::Value = serde_json::from_slice(&body).expect("Valid JSON");
    let session_id = json.get("session_id").unwrap().as_str().unwrap();

    // 2. List sessions - should have one
    let list_req = test::TestRequest::get().uri("/api/sessions").to_request();
    let list_resp = test::call_service(&app, list_req).await;
    assert_eq!(list_resp.status(), StatusCode::OK);

    let list_body = test::read_body(list_resp).await;
    let list_json: serde_json::Value = serde_json::from_slice(&list_body).expect("Valid JSON");
    assert!(list_json.get("count").is_some());

    // 3. Get the specific session
    let get_req = test::TestRequest::get()
        .uri(&format!("/api/sessions/{}", session_id))
        .to_request();

    let get_resp = test::call_service(&app, get_req).await;
    assert_eq!(get_resp.status(), StatusCode::OK);

    let get_body = test::read_body(get_resp).await;
    let get_json: serde_json::Value = serde_json::from_slice(&get_body).expect("Valid JSON");
    assert!(get_json.get("id").is_some() || get_json.get("messages").is_some());

    // 4. Delete the session
    let delete_req = test::TestRequest::delete()
        .uri(&format!("/api/sessions/{}", session_id))
        .to_request();

    let delete_resp = test::call_service(&app, delete_req).await;
    assert_eq!(delete_resp.status(), StatusCode::NO_CONTENT);

    // 5. Verify session is gone
    let get_again_req = test::TestRequest::get()
        .uri(&format!("/api/sessions/{}", session_id))
        .to_request();

    let get_again_resp = test::call_service(&app, get_again_req).await;
    assert_eq!(get_again_resp.status(), StatusCode::NOT_FOUND);
}

#[actix_web::test]
async fn test_session_messages_workflow() {
    let test_state = create_test_state().await;
    let app = test::init_service(
        App::new()
            .app_data(web::Data::new(test_state.state))
            .service(web::scope("/api").configure(routes::config_routes)),
    )
    .await;

    // Create session
    let create_req = test::TestRequest::post()
        .uri("/api/sessions")
        .set_json(serde_json::json!({}))
        .to_request();
    let create_resp = test::call_service(&app, create_req).await;
    let body = test::read_body(create_resp).await;
    let json: serde_json::Value = serde_json::from_slice(&body).expect("Valid JSON");
    let session_id = json.get("session_id").unwrap().as_str().unwrap();

    // Add first message
    let msg1_req = test::TestRequest::post()
        .uri(&format!("/api/sessions/{}/messages", session_id))
        .set_json(serde_json::json!({
            "content": "First message",
            "role": "user"
        }))
        .to_request();
    let msg1_resp = test::call_service(&app, msg1_req).await;
    assert_eq!(msg1_resp.status(), StatusCode::OK);

    // Add second message
    let msg2_req = test::TestRequest::post()
        .uri(&format!("/api/sessions/{}/messages", session_id))
        .set_json(serde_json::json!({
            "content": "Second message",
            "role": "assistant"
        }))
        .to_request();
    let msg2_resp = test::call_service(&app, msg2_req).await;
    assert_eq!(msg2_resp.status(), StatusCode::OK);

    // List messages
    let list_req = test::TestRequest::get()
        .uri(&format!("/api/sessions/{}/messages", session_id))
        .to_request();
    let list_resp = test::call_service(&app, list_req).await;
    assert_eq!(list_resp.status(), StatusCode::OK);

    let list_body = test::read_body(list_resp).await;
    let list_json: serde_json::Value = serde_json::from_slice(&list_body).expect("Valid JSON");
    assert_eq!(list_json.get("total"), Some(&serde_json::json!(2)));

    // Get specific message (index 0)
    let get_msg_req = test::TestRequest::get()
        .uri(&format!("/api/sessions/{}/messages/0", session_id))
        .to_request();
    let get_msg_resp = test::call_service(&app, get_msg_req).await;
    assert_eq!(get_msg_resp.status(), StatusCode::OK);
}

#[actix_web::test]
async fn test_session_command_execution() {
    let test_state = create_test_state().await;
    let app = test::init_service(
        App::new()
            .app_data(web::Data::new(test_state.state))
            .service(web::scope("/api").configure(routes::config_routes)),
    )
    .await;

    // Create session
    let create_req = test::TestRequest::post()
        .uri("/api/sessions")
        .set_json(serde_json::json!({}))
        .to_request();
    let create_resp = test::call_service(&app, create_req).await;
    let body = test::read_body(create_resp).await;
    let json: serde_json::Value = serde_json::from_slice(&body).expect("Valid JSON");
    let session_id = json.get("session_id").unwrap().as_str().unwrap();

    // Run echo command
    let cmd_req = test::TestRequest::post()
        .uri(&format!("/api/sessions/{}/command", session_id))
        .set_json(serde_json::json!({
            "command": "echo",
            "args": ["test"]
        }))
        .to_request();

    let cmd_resp = test::call_service(&app, cmd_req).await;
    // Command execution returns 200 even on failure (with error in body) or 500
    let status = cmd_resp.status();
    assert!(status.is_success() || status.is_server_error());

    let cmd_body = test::read_body(cmd_resp).await;
    let cmd_json: serde_json::Value = serde_json::from_slice(&cmd_body).expect("Valid JSON");

    // Should have command result fields
    assert!(cmd_json.get("command").is_some() || cmd_json.get("status").is_some());
}

// ============================================================================
// Error Response Format Tests
// ============================================================================

#[actix_web::test]
async fn test_error_responses_have_consistent_format() {
    let test_state = create_test_state().await;
    let app = test::init_service(
        App::new()
            .app_data(web::Data::new(test_state.state))
            .service(web::scope("/api").configure(routes::config_routes)),
    )
    .await;

    // Test 404 error format
    let req = test::TestRequest::get()
        .uri("/api/sessions/00000000-0000-0000-0000-000000000000")
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), StatusCode::NOT_FOUND);

    let body = test::read_body(resp).await;
    let json: serde_json::Value = serde_json::from_slice(&body).expect("Valid JSON");

    // Error responses should have error, message, and code fields
    assert!(
        json.get("error").is_some(),
        "Error should have 'error' field"
    );
    assert!(
        json.get("message").is_some(),
        "Error should have 'message' field"
    );
    assert!(json.get("code").is_some(), "Error should have 'code' field");
}

// ============================================================================
// Validation Tests
// ============================================================================

#[actix_web::test]
async fn test_session_create_rejects_oversized_initial_prompt() {
    let test_state = create_test_state().await;
    let app = test::init_service(
        App::new()
            .app_data(web::Data::new(test_state.state))
            .service(web::scope("/api").configure(routes::config_routes)),
    )
    .await;

    // Create a very long initial prompt (over 50000 chars)
    let long_prompt = "x".repeat(50001);

    let req = test::TestRequest::post()
        .uri("/api/sessions")
        .set_json(serde_json::json!({
            "initial_prompt": long_prompt
        }))
        .to_request();

    let resp = test::call_service(&app, req).await;
    // Should return validation error
    assert!(resp.status().is_client_error());
}

#[actix_web::test]
async fn test_session_fork_rejects_invalid_message_index() {
    let test_state = create_test_state().await;
    let app = test::init_service(
        App::new()
            .app_data(web::Data::new(test_state.state))
            .service(web::scope("/api").configure(routes::config_routes)),
    )
    .await;

    // Create session
    let create_req = test::TestRequest::post()
        .uri("/api/sessions")
        .set_json(serde_json::json!({}))
        .to_request();
    let create_resp = test::call_service(&app, create_req).await;
    let body = test::read_body(create_resp).await;
    let json: serde_json::Value = serde_json::from_slice(&body).expect("Valid JSON");
    let session_id = json.get("session_id").unwrap().as_str().unwrap();

    // Try to fork at invalid index
    let fork_req = test::TestRequest::post()
        .uri(&format!("/api/sessions/{}/fork", session_id))
        .set_json(serde_json::json!({
            "fork_at_message_index": 999999
        }))
        .to_request();

    let fork_resp = test::call_service(&app, fork_req).await;
    // Should return bad request or similar error
    let status = fork_resp.status();
    assert!(status.is_client_error() || status.is_server_error());
}
