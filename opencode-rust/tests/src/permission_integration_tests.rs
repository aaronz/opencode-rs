use actix_web::{test::TestRequest, web};
use opencode_core::permission::Permission;
use opencode_permission::{
    AgentPermissionScope, ApprovalQueue, ApprovalResult, PendingApproval, PermissionScope,
};
use opencode_server::ServerState;
use opencode_storage::SqliteProjectRepository;
use opencode_storage::SqliteSessionRepository;
use std::sync::Arc;
use uuid::Uuid;

#[derive(serde::Deserialize)]
pub struct PermissionReplyRequest {
    pub decision: String,
}

fn req_id_to_permission(req_id: &str) -> Permission {
    let lower = req_id.to_lowercase();
    if lower.contains("file_read") || lower.contains("read") {
        Permission::FileRead
    } else if lower.contains("file_write") || lower.contains("write") {
        Permission::FileWrite
    } else if lower.contains("file_delete") || lower.contains("delete") {
        Permission::FileDelete
    } else if lower.contains("bash") || lower.contains("execute") {
        Permission::BashExecute
    } else if lower.contains("network") || lower.contains("external") {
        Permission::NetworkAccess
    } else {
        Permission::FileRead
    }
}

fn create_test_state() -> ServerState {
    let temp_dir = tempfile::tempdir().unwrap();
    let db_path = temp_dir.path().join("test.db");
    let pool = opencode_storage::database::StoragePool::new(&db_path).unwrap();

    let session_repo = Arc::new(SqliteSessionRepository::new(pool.clone()));
    let project_repo = Arc::new(SqliteProjectRepository::new(pool.clone()));

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
        audit_log: None,
    };
    state
}

#[actix_web::test]
async fn test_permission_reply_allow_decision() {
    let state = create_test_state();
    let _req = TestRequest::default().to_http_request();

    let resp = permission_reply_handler(
        web::Data::new(state),
        web::Path::from(("session-1".to_string(), "req-1".to_string())),
        web::Json(PermissionReplyRequest {
            decision: "allow".to_string(),
        }),
    )
    .await;
    assert_eq!(resp.status(), actix_web::http::StatusCode::OK);
}

#[actix_web::test]
async fn test_permission_reply_deny_decision() {
    let state = create_test_state();
    let _req = TestRequest::default().to_http_request();

    let resp = permission_reply_handler(
        web::Data::new(state),
        web::Path::from(("session-2".to_string(), "req-2".to_string())),
        web::Json(PermissionReplyRequest {
            decision: "deny".to_string(),
        }),
    )
    .await;
    assert_eq!(resp.status(), actix_web::http::StatusCode::OK);
}

#[actix_web::test]
async fn test_permission_reply_invalid_decision_rejected() {
    let state = create_test_state();
    let _req = TestRequest::default().to_http_request();

    let resp = permission_reply_handler(
        web::Data::new(state),
        web::Path::from(("session-3".to_string(), "req-3".to_string())),
        web::Json(PermissionReplyRequest {
            decision: "invalid".to_string(),
        }),
    )
    .await;
    assert_eq!(resp.status(), actix_web::http::StatusCode::BAD_REQUEST);
}

#[actix_web::test]
async fn test_approval_queue_check_tool_permission() {
    let queue = ApprovalQueue::new(PermissionScope::ReadOnly);
    assert_eq!(queue.check("read"), ApprovalResult::AutoApprove);
    assert_eq!(queue.check("write"), ApprovalResult::RequireApproval);
    assert_eq!(queue.check("grep"), ApprovalResult::AutoApprove);
}

#[actix_web::test]
async fn test_approval_queue_check_full_scope() {
    let queue = ApprovalQueue::new(PermissionScope::Full);
    assert_eq!(queue.check("read"), ApprovalResult::AutoApprove);
    assert_eq!(queue.check("write"), ApprovalResult::AutoApprove);
    assert_eq!(queue.check("bash"), ApprovalResult::AutoApprove);
}

#[actix_web::test]
async fn test_approval_queue_check_restricted_scope() {
    let queue = ApprovalQueue::new(PermissionScope::Restricted);
    assert_eq!(queue.check("read"), ApprovalResult::AutoApprove);
    assert_eq!(queue.check("bash"), ApprovalResult::AutoApprove);
    assert_eq!(queue.check("write"), ApprovalResult::RequireApproval);
}

#[actix_web::test]
async fn test_approval_queue_approve_pending() {
    let mut queue = ApprovalQueue::new(PermissionScope::ReadOnly);
    let pending = PendingApproval::new(
        Uuid::new_v4(),
        "write".to_string(),
        serde_json::json!({"path": "/test.txt"}),
    );
    let approval_id = pending.id;
    queue.request_approval(pending);

    let approved = queue.approve(approval_id);
    assert!(approved.is_some());
}

#[actix_web::test]
async fn test_approval_queue_reject_pending() {
    let mut queue = ApprovalQueue::new(PermissionScope::ReadOnly);
    let pending = PendingApproval::new(
        Uuid::new_v4(),
        "write".to_string(),
        serde_json::json!({"path": "/test.txt"}),
    );
    let approval_id = pending.id;
    queue.request_approval(pending);

    let rejected = queue.reject(approval_id);
    assert!(rejected);
}

#[actix_web::test]
async fn test_approval_queue_get_pending_by_session() {
    let mut queue = ApprovalQueue::new(PermissionScope::ReadOnly);
    let session_id = Uuid::new_v4();

    queue.request_approval(PendingApproval::new(
        session_id,
        "write".to_string(),
        serde_json::json!({}),
    ));
    queue.request_approval(PendingApproval::new(
        Uuid::new_v4(),
        "write".to_string(),
        serde_json::json!({}),
    ));

    let pending = queue.get_pending(session_id);
    assert_eq!(pending.len(), 1);
}

#[actix_web::test]
async fn test_approval_queue_set_scope() {
    let mut queue = ApprovalQueue::new(PermissionScope::ReadOnly);
    assert_eq!(queue.check("write"), ApprovalResult::RequireApproval);

    queue.set_scope(PermissionScope::Full);
    assert_eq!(queue.check("write"), ApprovalResult::AutoApprove);
}

#[actix_web::test]
async fn test_permission_manager_grant() {
    let mut pm = opencode_core::PermissionManager::default();
    pm.grant(Permission::FileWrite);
    assert!(pm.check(&Permission::FileWrite, "file_write"));
}

#[actix_web::test]
async fn test_permission_manager_revoke() {
    let mut pm = opencode_core::PermissionManager::default();
    pm.grant(Permission::FileWrite);
    pm.revoke(&Permission::FileWrite);
    assert!(!pm.check(&Permission::FileWrite, "file_write"));
}

#[actix_web::test]
async fn test_permission_scope_default_is_readonly() {
    assert_eq!(PermissionScope::default(), PermissionScope::ReadOnly);
}

#[actix_web::test]
async fn test_agent_permission_scope_can_write_files() {
    assert!(AgentPermissionScope::Full.can_write_files());
    assert!(!AgentPermissionScope::ReadOnly.can_write_files());
    assert!(!AgentPermissionScope::Restricted.can_write_files());
    assert!(!AgentPermissionScope::None.can_write_files());
}

#[actix_web::test]
async fn test_agent_permission_scope_can_run_commands() {
    assert!(AgentPermissionScope::Full.can_run_commands());
    assert!(!AgentPermissionScope::ReadOnly.can_run_commands());
    assert!(!AgentPermissionScope::Restricted.can_run_commands());
    assert!(!AgentPermissionScope::None.can_run_commands());
}

#[actix_web::test]
async fn test_agent_permission_scope_intersect() {
    assert_eq!(
        AgentPermissionScope::Full.intersect(AgentPermissionScope::ReadOnly),
        AgentPermissionScope::ReadOnly
    );
    assert_eq!(
        AgentPermissionScope::ReadOnly.intersect(AgentPermissionScope::Full),
        AgentPermissionScope::ReadOnly
    );
    assert_eq!(
        AgentPermissionScope::Restricted.intersect(AgentPermissionScope::Full),
        AgentPermissionScope::Restricted
    );
    assert_eq!(
        AgentPermissionScope::None.intersect(AgentPermissionScope::Full),
        AgentPermissionScope::None
    );
}

#[actix_web::test]
async fn test_approval_triggers_execution() {
    let state = create_test_state();
    let approval_queue = state.approval_queue.clone();
    let permission_manager = state.permission_manager.clone();

    let session_id = Uuid::new_v4();
    let pending = PendingApproval::new(
        session_id,
        "write".to_string(),
        serde_json::json!({"path": "/test.txt"}),
    );
    let approval_id = pending.id;

    {
        let mut aq = approval_queue.write().unwrap();
        aq.request_approval(pending);
    }

    let mut receiver = {
        let aq_guard = approval_queue.read().unwrap();
        aq_guard
            .subscribe()
            .expect("ApprovalQueue should have notification channel")
    };

    let resp = permission_reply_handler(
        web::Data::new(state.clone()),
        web::Path::from((
            session_id.to_string(),
            format!("file_write_{}", approval_id),
        )),
        web::Json(PermissionReplyRequest {
            decision: "allow".to_string(),
        }),
    )
    .await;
    assert_eq!(resp.status(), actix_web::http::StatusCode::OK);

    let decision = receiver.recv().await.unwrap();
    match decision {
        opencode_permission::ApprovalDecision::Approved(cmd) => {
            assert_eq!(cmd.tool_name, "write");
            assert_eq!(cmd.session_id, session_id);
        }
        _ => panic!("Expected Approved decision"),
    }

    let pm = permission_manager.read().unwrap();
    assert!(
        pm.check(&Permission::FileWrite, "/test.txt"),
        "FileWrite permission should be granted after approval"
    );
}

async fn permission_reply_handler(
    state: web::Data<ServerState>,
    path: web::Path<(String, String)>,
    body: web::Json<PermissionReplyRequest>,
) -> actix_web::HttpResponse {
    use actix_web::HttpResponse;

    let (session_id, req_id) = path.into_inner();
    let decision = body.decision.to_lowercase();
    if decision != "allow" && decision != "deny" {
        return HttpResponse::BadRequest().json(serde_json::json!({
            "error": "decision must be 'allow' or 'deny'"
        }));
    }

    let permission = req_id_to_permission(&req_id);

    if let Ok(mut pm) = state.permission_manager.write() {
        match decision.as_str() {
            "allow" => {
                pm.grant(permission.clone());
            }
            "deny" => {
                pm.revoke(&permission);
            }
            _ => {}
        }
    }

    if let Ok(mut aq) = state.approval_queue.write() {
        if let Ok(approval_id) = Uuid::parse_str(&req_id) {
            match decision.as_str() {
                "allow" => {
                    let _ = aq.approve(approval_id);
                }
                "deny" => {
                    let _ = aq.reject(approval_id);
                }
                _ => {}
            }
        }
    }

    HttpResponse::Ok().json(serde_json::json!({
        "status": "ok",
        "session_id": session_id,
        "request_id": req_id,
        "decision": decision
    }))
}
