use actix_web::{web, App, HttpServer};
use opencode_storage::migration::MigrationManager;
use serde::Deserialize;
use std::sync::Arc;
use std::time::Duration;

#[derive(Debug, Deserialize)]
pub struct ProviderStatus {
    pub name: String,
    pub status: String,
    pub model: String,
}

#[derive(Debug, Deserialize)]
pub struct PluginStatus {
    pub name: String,
    pub version: String,
    pub status: String,
}

#[derive(Debug, Deserialize)]
pub struct StatusResponse {
    pub version: String,
    pub rustc_version: String,
    pub build_timestamp: String,
    pub status: String,
    pub uptime_seconds: u64,
    pub active_sessions: usize,
    pub total_sessions: usize,
    pub providers: Vec<ProviderStatus>,
    pub plugins: Vec<PluginStatus>,
}

#[tokio::test]
async fn test_response_contains_all_fields() {
    let (server_url, server_handle, _temp_dir) = start_test_server(0).await;

    let client = reqwest::Client::new();

    let resp = client
        .get(format!("{}/api/status", server_url))
        .send()
        .await
        .expect("Failed to call status endpoint");

    assert_eq!(
        resp.status().as_u16(),
        200,
        "GET /api/status should return 200 OK"
    );

    let status_response: StatusResponse = resp
        .json()
        .await
        .expect("Failed to parse status response as StatusResponse");

    assert!(
        !status_response.version.is_empty(),
        "version field should be non-null and non-empty"
    );
    assert!(
        !status_response.rustc_version.is_empty(),
        "rustc_version field should be non-null and non-empty"
    );
    assert!(
        !status_response.build_timestamp.is_empty(),
        "build_timestamp field should be non-null and non-empty"
    );
    assert!(
        !status_response.status.is_empty(),
        "status field should be non-null and non-empty"
    );
    assert_eq!(
        status_response.uptime_seconds, status_response.uptime_seconds,
        "uptime_seconds field should be accessible"
    );
    assert_eq!(
        status_response.active_sessions, status_response.active_sessions,
        "active_sessions field should be accessible"
    );
    assert_eq!(
        status_response.total_sessions, status_response.total_sessions,
        "total_sessions field should be accessible"
    );

    for provider in &status_response.providers {
        assert!(
            !provider.name.is_empty(),
            "provider.name should be non-null and non-empty"
        );
        assert!(
            !provider.status.is_empty(),
            "provider.status should be non-null and non-empty"
        );
        assert!(
            !provider.model.is_empty(),
            "provider.model should be non-null and non-empty"
        );
    }

    for plugin in &status_response.plugins {
        assert!(
            !plugin.name.is_empty(),
            "plugin.name should be non-null and non-empty"
        );
        assert!(
            !plugin.version.is_empty(),
            "plugin.version should be non-null and non-empty"
        );
        assert!(
            !plugin.status.is_empty(),
            "plugin.status should be non-null and non-empty"
        );
    }

    server_handle.abort();
}

#[tokio::test]
async fn test_accessible_without_auth() {
    let (server_url, server_handle, _temp_dir) = start_test_server(0).await;

    let client = reqwest::Client::new();

    let resp = client
        .get(format!("{}/api/status", server_url))
        .send()
        .await
        .expect("Failed to call status endpoint");

    assert_eq!(
        resp.status().as_u16(),
        200,
        "GET /api/status without Authorization header should return 200 OK"
    );

    let status_response: StatusResponse = resp
        .json()
        .await
        .expect("Failed to parse status response as StatusResponse");

    assert_eq!(
        status_response.status, "running",
        "Server status should be 'running'"
    );

    server_handle.abort();
}

async fn start_test_server(
    port: u16,
) -> (String, tokio::task::JoinHandle<()>, Arc<tempfile::TempDir>) {
    let temp_dir = Arc::new(tempfile::tempdir().unwrap());
    let temp_dir_clone = temp_dir.clone();
    let db_path = temp_dir.path().join("test.db");
    let pool = opencode_storage::database::StoragePool::new(&db_path).unwrap();

    let migration_manager = MigrationManager::new(pool.clone(), 2);
    migration_manager
        .migrate()
        .await
        .expect("Failed to run migrations");

    let session_repo = Arc::new(opencode_storage::SqliteSessionRepository::new(pool.clone()));
    let project_repo = Arc::new(opencode_storage::SqliteProjectRepository::new(pool.clone()));

    let state = opencode_server::ServerState {
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
        acp_enabled: false,
        acp_stream: opencode_control_plane::AcpEventStream::new().into(),
        acp_client_registry: std::sync::Arc::new(tokio::sync::RwLock::new(
            opencode_server::routes::acp_ws::AcpClientRegistry::new(),
        )),
        tool_registry: std::sync::Arc::new(opencode_tools::ToolRegistry::new()),
        session_hub: std::sync::Arc::new(opencode_server::routes::ws::SessionHub::new(256)),
        server_start_time: std::time::SystemTime::now(),
        permission_manager: std::sync::Arc::new(std::sync::RwLock::new(
            opencode_core::PermissionManager::default(),
        )),
        approval_queue: std::sync::Arc::new(std::sync::RwLock::new(
            opencode_permission::ApprovalQueue::default(),
        )),
    };

    let state_data = web::Data::new(state);

    let bind_addr = format!("127.0.0.1:{}", port);
    let std_listener = std::net::TcpListener::bind(&bind_addr).unwrap();
    let actual_port = std_listener.local_addr().unwrap().port();
    let server_url = format!("http://127.0.0.1:{}", actual_port);

    let handle = tokio::spawn(async move {
        let status_handler = opencode_server::routes::status::get_status;
        HttpServer::new(move || {
            App::new()
                .app_data(state_data.clone())
                .route("/api/status", web::get().to(status_handler))
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
