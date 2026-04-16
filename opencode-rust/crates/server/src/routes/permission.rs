use crate::routes::error::{bad_request, permission_denied_error};
use crate::ServerState;
use actix_web::{web, HttpResponse, Responder};
use opencode_core::permission::Permission;
use uuid::Uuid;

#[derive(serde::Deserialize)]
pub struct PermissionReplyRequest {
    pub decision: String,
}

pub async fn list_permissions(_state: web::Data<ServerState>) -> impl Responder {
    HttpResponse::Ok().json(serde_json::json!({
        "permissions": ["repo:read", "repo:write", "session:all"]
    }))
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

pub async fn permission_reply(
    state: web::Data<ServerState>,
    path: web::Path<(String, String)>,
    body: web::Json<PermissionReplyRequest>,
) -> impl Responder {
    let (session_id, req_id) = path.into_inner();
    let decision = body.decision.to_lowercase();
    if decision != "allow" && decision != "deny" {
        return bad_request("decision must be 'allow' or 'deny'");
    }

    let permission = req_id_to_permission(&req_id);

    if let Ok(mut pm) = state.permission_manager.write() {
        match decision.as_str() {
            "allow" => {
                pm.grant(permission.clone());
                tracing::info!(
                    "Permission granted: session={}, req={}, permission={:?}",
                    session_id,
                    req_id,
                    permission
                );
            }
            "deny" => {
                pm.revoke(&permission);
                tracing::info!(
                    "Permission denied: session={}, req={}, permission={:?}",
                    session_id,
                    req_id,
                    permission
                );
            }
            _ => {}
        }
    }

    if let Ok(mut aq) = state.approval_queue.write() {
        if let Ok(approval_id) = Uuid::parse_str(&req_id) {
            match decision.as_str() {
                "allow" => {
                    if let Some(approved) = aq.approve(approval_id) {
                        tracing::info!(
                            "ApprovalQueue updated: approved tool={} for session={}",
                            approved.tool_name,
                            session_id
                        );
                    }
                }
                "deny" => {
                    if aq.reject(approval_id) {
                        tracing::info!(
                            "ApprovalQueue updated: rejected req_id={} for session={}",
                            req_id,
                            session_id
                        );
                    }
                }
                _ => {}
            }
        }
    }

    match decision.as_str() {
        "allow" => HttpResponse::Ok().json(serde_json::json!({
            "status": "ok",
            "session_id": session_id,
            "request_id": req_id,
            "decision": decision
        })),
        "deny" => permission_denied_error(format!(
            "Permission denied for session={}, request={}",
            session_id, req_id
        )),
        _ => HttpResponse::Ok().json(serde_json::json!({
            "status": "ok",
            "session_id": session_id,
            "request_id": req_id,
            "decision": decision
        })),
    }
}

pub fn init(cfg: &mut web::ServiceConfig) {
    cfg.route("", web::get().to(list_permissions));
}
