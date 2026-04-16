use actix_web::{test, web, App};
use opencode_core::PermissionManager;
use opencode_permission::ApprovalQueue;
use opencode_server::{routes, ServerState};
use std::sync::Arc;

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

#[actix_web::test]
async fn test_run_endpoint_request_validation_requires_prompt() {
    let app = test::init_service(
        App::new()
            .app_data(web::Data::new(create_test_state()))
            .service(web::scope("/api").configure(routes::config_routes)),
    )
    .await;

    let req = test::TestRequest::post()
        .uri("/api/run")
        .set_json(serde_json::json!({}))
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert!(resp.status().is_client_error());
}

#[actix_web::test]
async fn test_run_endpoint_accepts_valid_request() {
    let app = test::init_service(
        App::new()
            .app_data(web::Data::new(create_test_state()))
            .service(web::scope("/api").configure(routes::config_routes)),
    )
    .await;

    let req = test::TestRequest::post()
        .uri("/api/run")
        .set_json(serde_json::json!({
            "prompt": "Hello, world!",
            "stream": false
        }))
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert!(resp.status().is_server_error() || resp.status().is_success());
}

#[actix_web::test]
async fn test_run_endpoint_validates_agent_type() {
    let app = test::init_service(
        App::new()
            .app_data(web::Data::new(create_test_state()))
            .service(web::scope("/api").configure(routes::config_routes)),
    )
    .await;

    let req = test::TestRequest::post()
        .uri("/api/run")
        .set_json(serde_json::json!({
            "prompt": "Hello",
            "agent": "invalid_agent"
        }))
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert!(resp.status().is_client_error());
}

#[actix_web::test]
async fn test_run_endpoint_accepts_build_agent() {
    let app = test::init_service(
        App::new()
            .app_data(web::Data::new(create_test_state()))
            .service(web::scope("/api").configure(routes::config_routes)),
    )
    .await;

    let req = test::TestRequest::post()
        .uri("/api/run")
        .set_json(serde_json::json!({
            "prompt": "Hello",
            "agent": "build"
        }))
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert!(resp.status().is_server_error() || resp.status().is_success());
}

#[actix_web::test]
async fn test_run_endpoint_accepts_plan_agent() {
    let app = test::init_service(
        App::new()
            .app_data(web::Data::new(create_test_state()))
            .service(web::scope("/api").configure(routes::config_routes)),
    )
    .await;

    let req = test::TestRequest::post()
        .uri("/api/run")
        .set_json(serde_json::json!({
            "prompt": "Hello",
            "agent": "plan"
        }))
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert!(resp.status().is_server_error() || resp.status().is_success());
}

#[actix_web::test]
async fn test_run_endpoint_accepts_general_agent() {
    let app = test::init_service(
        App::new()
            .app_data(web::Data::new(create_test_state()))
            .service(web::scope("/api").configure(routes::config_routes)),
    )
    .await;

    let req = test::TestRequest::post()
        .uri("/api/run")
        .set_json(serde_json::json!({
            "prompt": "Hello",
            "agent": "general"
        }))
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert!(resp.status().is_server_error() || resp.status().is_success());
}

#[actix_web::test]
async fn test_run_request_deserialization() {
    let json = r#"{"prompt": "hello", "model": "gpt-4", "agent": "build", "stream": true}"#;
    let req: routes::run::RunRequest = serde_json::from_str(json).unwrap();
    assert_eq!(req.prompt, "hello");
    assert_eq!(req.model, Some("gpt-4".to_string()));
    assert_eq!(req.agent, Some("build".to_string()));
    assert!(req.stream);
}

#[actix_web::test]
async fn test_run_request_deserialization_defaults_stream_to_false() {
    let json = r#"{"prompt": "hello"}"#;
    let req: routes::run::RunRequest = serde_json::from_str(json).unwrap();
    assert_eq!(req.prompt, "hello");
    assert!(!req.stream);
}
