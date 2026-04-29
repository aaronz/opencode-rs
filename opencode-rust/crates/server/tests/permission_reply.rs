use actix_web::{http::StatusCode, test, web, App};
use opencode_core::PermissionManager;
use opencode_permission::{ApprovalQueue, PendingApproval, PermissionScope};
use opencode_server::{routes, ServerState};
use std::sync::Arc;
use uuid::Uuid;

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
        approval_queue: std::sync::Arc::new(std::sync::RwLock::new(ApprovalQueue::new(
            PermissionScope::ReadOnly,
        ))),
        audit_log: None,
        runtime: opencode_server::build_placeholder_runtime(),
    }
}

#[actix_web::test]
async fn test_user_approval_triggers_approval_queue_update() {
    let app = test::init_service(
        App::new()
            .app_data(web::Data::new(create_test_state()))
            .service(web::scope("/api/permissions").configure(routes::permission::init)),
    )
    .await;

    let approval_id = Uuid::new_v4();
    let session_id = Uuid::new_v4().to_string();

    let req = test::TestRequest::post()
        .uri(&format!(
            "/api/permissions/reply/{}/{}",
            session_id, approval_id
        ))
        .set_json(serde_json::json!({
            "decision": "allow"
        }))
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), StatusCode::OK);
}

#[actix_web::test]
async fn test_user_denial_returns_permission_denied_error() {
    let app = test::init_service(
        App::new()
            .app_data(web::Data::new(create_test_state()))
            .service(web::scope("/api/permissions").configure(routes::permission::init)),
    )
    .await;

    let approval_id = Uuid::new_v4();
    let session_id = Uuid::new_v4().to_string();

    let req = test::TestRequest::post()
        .uri(&format!(
            "/api/permissions/reply/{}/{}",
            session_id, approval_id
        ))
        .set_json(serde_json::json!({
            "decision": "deny"
        }))
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), StatusCode::FORBIDDEN);

    let body = test::read_body(resp).await;
    let json: serde_json::Value =
        serde_json::from_slice(&body).expect("Response should be valid JSON");
    assert_eq!(
        json.get("error").and_then(|v| v.as_str()),
        Some("permission_denied")
    );
}

#[actix_web::test]
async fn test_invalid_decision_returns_bad_request() {
    let app = test::init_service(
        App::new()
            .app_data(web::Data::new(create_test_state()))
            .service(web::scope("/api/permissions").configure(routes::permission::init)),
    )
    .await;

    let approval_id = Uuid::new_v4();
    let session_id = Uuid::new_v4().to_string();

    let req = test::TestRequest::post()
        .uri(&format!(
            "/api/permissions/reply/{}/{}",
            session_id, approval_id
        ))
        .set_json(serde_json::json!({
            "decision": "invalid"
        }))
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), StatusCode::BAD_REQUEST);
}

#[actix_web::test]
async fn test_permission_reply_with_empty_decision_returns_bad_request() {
    let app = test::init_service(
        App::new()
            .app_data(web::Data::new(create_test_state()))
            .service(web::scope("/api/permissions").configure(routes::permission::init)),
    )
    .await;

    let approval_id = Uuid::new_v4();
    let session_id = Uuid::new_v4().to_string();

    let req = test::TestRequest::post()
        .uri(&format!(
            "/api/permissions/reply/{}/{}",
            session_id, approval_id
        ))
        .set_json(serde_json::json!({
            "decision": ""
        }))
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), StatusCode::BAD_REQUEST);
}

#[actix_web::test]
async fn test_approval_queue_updated_with_user_decision_allow() {
    let state = create_test_state();
    let session_id = Uuid::new_v4();

    let approval_id = {
        let mut aq = state.approval_queue.write().unwrap();
        let pending = PendingApproval::new(
            session_id,
            "write".to_string(),
            serde_json::json!({"path": "/test.txt"}),
        );
        let id = pending.id;
        aq.request_approval(pending);
        id
    };

    let app = test::init_service(
        App::new()
            .app_data(web::Data::new(state.clone()))
            .service(web::scope("/api/permissions").configure(routes::permission::init)),
    )
    .await;

    let req = test::TestRequest::post()
        .uri(&format!(
            "/api/permissions/reply/{}/{}",
            session_id, approval_id
        ))
        .set_json(serde_json::json!({
            "decision": "allow"
        }))
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), StatusCode::OK);

    let aq = state.approval_queue.read().unwrap();
    assert_eq!(
        aq.get_history(session_id).len(),
        1,
        "Should have one approved command in history"
    );
}

#[actix_web::test]
async fn test_approval_queue_updated_with_user_decision_deny() {
    let state = create_test_state();
    let session_id = Uuid::new_v4();

    let approval_id = {
        let mut aq = state.approval_queue.write().unwrap();
        let pending = PendingApproval::new(
            session_id,
            "write".to_string(),
            serde_json::json!({"path": "/test.txt"}),
        );
        let id = pending.id;
        aq.request_approval(pending);
        id
    };

    let app = test::init_service(
        App::new()
            .app_data(web::Data::new(state.clone()))
            .service(web::scope("/api/permissions").configure(routes::permission::init)),
    )
    .await;

    let req = test::TestRequest::post()
        .uri(&format!(
            "/api/permissions/reply/{}/{}",
            session_id, approval_id
        ))
        .set_json(serde_json::json!({
            "decision": "deny"
        }))
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), StatusCode::FORBIDDEN);

    let aq = state.approval_queue.read().unwrap();
    assert_eq!(
        aq.get_pending(session_id).len(),
        0,
        "Pending request should be removed after rejection"
    );
}

#[actix_web::test]
async fn test_permission_manager_updated_on_approval() {
    let state = create_test_state();
    let approval_id = Uuid::new_v4();
    let session_id = Uuid::new_v4().to_string();

    let app = test::init_service(
        App::new()
            .app_data(web::Data::new(state.clone()))
            .service(web::scope("/api/permissions").configure(routes::permission::init)),
    )
    .await;

    let req = test::TestRequest::post()
        .uri(&format!(
            "/api/permissions/reply/{}/{}",
            session_id, approval_id
        ))
        .set_json(serde_json::json!({
            "decision": "allow"
        }))
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), StatusCode::OK);
}

#[actix_web::test]
async fn test_permission_manager_updated_on_denial() {
    let state = create_test_state();
    let approval_id = Uuid::new_v4();
    let session_id = Uuid::new_v4().to_string();

    let app = test::init_service(
        App::new()
            .app_data(web::Data::new(state.clone()))
            .service(web::scope("/api/permissions").configure(routes::permission::init)),
    )
    .await;

    let req = test::TestRequest::post()
        .uri(&format!(
            "/api/permissions/reply/{}/{}",
            session_id, approval_id
        ))
        .set_json(serde_json::json!({
            "decision": "deny"
        }))
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), StatusCode::FORBIDDEN);
}

#[actix_web::test]
async fn test_pending_requests_reevaluated_after_approval() {
    let state = create_test_state();
    let session_id = Uuid::new_v4();

    let approval_id = {
        let mut aq = state.approval_queue.write().unwrap();
        let pending = PendingApproval::new(
            session_id,
            "write".to_string(),
            serde_json::json!({"path": "/test.txt"}),
        );
        let id = pending.id;
        aq.request_approval(pending);
        id
    };

    let mut receiver = {
        let aq = state.approval_queue.read().unwrap();
        aq.subscribe().expect("Should have notification channel")
    };

    let app = test::init_service(
        App::new()
            .app_data(web::Data::new(state.clone()))
            .service(web::scope("/api/permissions").configure(routes::permission::init)),
    )
    .await;

    let req = test::TestRequest::post()
        .uri(&format!(
            "/api/permissions/reply/{}/{}",
            session_id, approval_id
        ))
        .set_json(serde_json::json!({
            "decision": "allow"
        }))
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), StatusCode::OK);

    let decision = tokio::time::timeout(std::time::Duration::from_secs(1), receiver.recv())
        .await
        .expect("Should receive notification")
        .expect("Should not error on receive");

    match decision {
        opencode_permission::ApprovalDecision::Approved(cmd) => {
            assert_eq!(cmd.tool_name, "write");
        }
        _ => panic!("Expected Approved decision"),
    }
}

#[actix_web::test]
async fn test_decision_case_insensitive_allow_uppercase() {
    let app = test::init_service(
        App::new()
            .app_data(web::Data::new(create_test_state()))
            .service(web::scope("/api/permissions").configure(routes::permission::init)),
    )
    .await;

    let approval_id = Uuid::new_v4();
    let session_id = Uuid::new_v4().to_string();

    let req = test::TestRequest::post()
        .uri(&format!(
            "/api/permissions/reply/{}/{}",
            session_id, approval_id
        ))
        .set_json(serde_json::json!({
            "decision": "ALLOW"
        }))
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), StatusCode::OK);
}

#[actix_web::test]
async fn test_decision_case_insensitive_deny_uppercase() {
    let app = test::init_service(
        App::new()
            .app_data(web::Data::new(create_test_state()))
            .service(web::scope("/api/permissions").configure(routes::permission::init)),
    )
    .await;

    let approval_id = Uuid::new_v4();
    let session_id = Uuid::new_v4().to_string();

    let req = test::TestRequest::post()
        .uri(&format!(
            "/api/permissions/reply/{}/{}",
            session_id, approval_id
        ))
        .set_json(serde_json::json!({
            "decision": "DENY"
        }))
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), StatusCode::FORBIDDEN);
}

#[actix_web::test]
async fn test_list_permissions_returns_available_permissions() {
    let app = test::init_service(
        App::new()
            .app_data(web::Data::new(create_test_state()))
            .service(web::scope("/api/permissions").configure(routes::permission::init)),
    )
    .await;

    let req = test::TestRequest::get()
        .uri("/api/permissions")
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), StatusCode::OK);

    let body = test::read_body(resp).await;
    let json: serde_json::Value =
        serde_json::from_slice(&body).expect("Response should be valid JSON");
    assert!(json.get("permissions").is_some());
}

#[actix_web::test]
async fn test_non_uuid_req_id_still_processes_permission_manager() {
    let app = test::init_service(
        App::new()
            .app_data(web::Data::new(create_test_state()))
            .service(web::scope("/api/permissions").configure(routes::permission::init)),
    )
    .await;

    let session_id = Uuid::new_v4().to_string();

    let req = test::TestRequest::post()
        .uri(&format!(
            "/api/permissions/reply/{}/{}",
            session_id, "file_write_123"
        ))
        .set_json(serde_json::json!({
            "decision": "allow"
        }))
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), StatusCode::OK);
}
