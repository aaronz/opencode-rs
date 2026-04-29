use actix_web::{http::StatusCode, test, web, App};
use opencode_core::PermissionManager;
use opencode_permission::ApprovalQueue;
use opencode_server::{routes, ServerState};
use std::sync::Arc;
use std::time::Instant;

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
        runtime: opencode_server::build_placeholder_runtime(),
    }
}

macro_rules! create_test_app {
    () => {
        App::new()
            .app_data(web::Data::new(create_test_state()))
            .route("/api/status", web::get().to(routes::status::get_status))
    };
}

#[actix_web::test]
async fn test_status_endpoint_returns_200_with_correct_json_structure() {
    let app = test::init_service(create_test_app!()).await;

    let req = test::TestRequest::get().uri("/api/status").to_request();

    let start = Instant::now();
    let resp = test::call_service(&app, req).await;
    let elapsed = start.elapsed();

    assert_eq!(
        resp.status(),
        StatusCode::OK,
        "Status endpoint should return 200"
    );

    let body = test::read_body(resp).await;
    let json: serde_json::Value =
        serde_json::from_slice(&body).expect("Response should be valid JSON");

    assert!(
        json.get("version").is_some(),
        "Response should contain version field"
    );
    assert!(
        json.get("status").is_some(),
        "Response should contain status field"
    );
    assert!(
        json.get("uptime_seconds").is_some(),
        "Response should contain uptime_seconds field"
    );
    assert!(
        json.get("active_sessions").is_some(),
        "Response should contain active_sessions field"
    );
    assert!(
        json.get("total_sessions").is_some(),
        "Response should contain total_sessions field"
    );
    assert!(
        json.get("providers").is_some(),
        "Response should contain providers field"
    );
    assert!(
        json.get("plugins").is_some(),
        "Response should contain plugins field"
    );

    assert!(
        elapsed.as_millis() < 100,
        "Response time should be under 100ms, got {}ms",
        elapsed.as_millis()
    );
}

#[actix_web::test]
async fn test_status_response_contains_version_status_uptime_seconds() {
    let app = test::init_service(create_test_app!()).await;

    let req = test::TestRequest::get().uri("/api/status").to_request();
    let resp = test::call_service(&app, req).await;

    assert_eq!(resp.status(), StatusCode::OK);

    let body = test::read_body(resp).await;
    let json: serde_json::Value =
        serde_json::from_slice(&body).expect("Response should be valid JSON");

    let version = json.get("version").expect("version should exist");
    assert!(
        version.is_string() && !version.as_str().unwrap().is_empty(),
        "version should be a non-empty string"
    );

    let status = json.get("status").expect("status should exist");
    assert!(
        status.is_string() && !status.as_str().unwrap().is_empty(),
        "status should be a non-empty string"
    );

    let uptime_seconds = json
        .get("uptime_seconds")
        .expect("uptime_seconds should exist");
    assert!(
        uptime_seconds.is_number() && uptime_seconds.as_i64().unwrap_or(-1) >= 0,
        "uptime_seconds should be a non-negative number"
    );
}

#[actix_web::test]
async fn test_status_response_contains_session_counts() {
    let app = test::init_service(create_test_app!()).await;

    let req = test::TestRequest::get().uri("/api/status").to_request();
    let resp = test::call_service(&app, req).await;

    assert_eq!(resp.status(), StatusCode::OK);

    let body = test::read_body(resp).await;
    let json: serde_json::Value =
        serde_json::from_slice(&body).expect("Response should be valid JSON");

    let active_sessions = json
        .get("active_sessions")
        .expect("active_sessions should exist");
    assert!(
        active_sessions.is_number() && active_sessions.as_i64().unwrap_or(-1) >= 0,
        "active_sessions should be a non-negative number"
    );

    let total_sessions = json
        .get("total_sessions")
        .expect("total_sessions should exist");
    assert!(
        total_sessions.is_number() && total_sessions.as_i64().unwrap_or(-1) >= 0,
        "total_sessions should be a non-negative number"
    );
}

#[actix_web::test]
async fn test_status_providers_array_contains_required_fields() {
    let app = test::init_service(create_test_app!()).await;

    let req = test::TestRequest::get().uri("/api/status").to_request();
    let resp = test::call_service(&app, req).await;

    assert_eq!(resp.status(), StatusCode::OK);

    let body = test::read_body(resp).await;
    let json: serde_json::Value =
        serde_json::from_slice(&body).expect("Response should be valid JSON");

    let providers = json.get("providers").expect("providers should exist");
    assert!(providers.is_array(), "providers should be an array");

    let providers_array = providers.as_array().unwrap();
    if !providers_array.is_empty() {
        let first_provider = &providers_array[0];
        assert!(
            first_provider.get("name").is_some(),
            "provider should have name field"
        );
        assert!(
            first_provider.get("status").is_some(),
            "provider should have status field"
        );
        assert!(
            first_provider.get("model").is_some(),
            "provider should have model field"
        );
    }
}

#[actix_web::test]
async fn test_status_plugins_array_contains_required_fields() {
    let app = test::init_service(create_test_app!()).await;

    let req = test::TestRequest::get().uri("/api/status").to_request();
    let resp = test::call_service(&app, req).await;

    assert_eq!(resp.status(), StatusCode::OK);

    let body = test::read_body(resp).await;
    let json: serde_json::Value =
        serde_json::from_slice(&body).expect("Response should be valid JSON");

    let plugins = json.get("plugins").expect("plugins should exist");
    assert!(plugins.is_array(), "plugins should be an array");

    let plugins_array = plugins.as_array().unwrap();
    if !plugins_array.is_empty() {
        let first_plugin = &plugins_array[0];
        assert!(
            first_plugin.get("name").is_some(),
            "plugin should have name field"
        );
        assert!(
            first_plugin.get("version").is_some(),
            "plugin should have version field"
        );
        assert!(
            first_plugin.get("status").is_some(),
            "plugin should have status field"
        );
    }
}

#[actix_web::test]
async fn test_status_endpoint_no_authentication_required() {
    let app = test::init_service(create_test_app!()).await;

    let req = test::TestRequest::get()
        .uri("/api/status")
        .insert_header(("x-api-key", "invalid-key"))
        .to_request();

    let resp = test::call_service(&app, req).await;

    assert_eq!(
        resp.status(),
        StatusCode::OK,
        "Status endpoint should not require authentication and should return 200 even with invalid API key"
    );
}

#[actix_web::test]
async fn test_status_response_time_under_100ms() {
    let app = test::init_service(create_test_app!()).await;

    let req = test::TestRequest::get().uri("/api/status").to_request();

    let start = Instant::now();
    let resp = test::call_service(&app, req).await;
    let elapsed = start.elapsed();

    assert_eq!(resp.status(), StatusCode::OK);

    assert!(
        elapsed.as_millis() < 100,
        "Status endpoint response time should be under 100ms, got {}ms",
        elapsed.as_millis()
    );
}
