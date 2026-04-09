use crate::ServerState;
use actix_web::{web, HttpResponse, Responder};

#[derive(serde::Deserialize)]
pub struct PermissionReplyRequest {
    pub decision: String,
}

pub async fn list_permissions(_state: web::Data<ServerState>) -> impl Responder {
    HttpResponse::Ok().json(serde_json::json!({
        "permissions": ["repo:read", "repo:write", "session:all"]
    }))
}

pub async fn permission_reply(
    _state: web::Data<ServerState>,
    path: web::Path<(String, String)>,
    body: web::Json<PermissionReplyRequest>,
) -> impl Responder {
    let (_session_id, _req_id) = path.into_inner();
    let decision = body.decision.to_lowercase();
    if decision != "allow" && decision != "deny" {
        return HttpResponse::BadRequest().json(serde_json::json!({
            "error": "decision must be 'allow' or 'deny'"
        }));
    }
    tracing::info!(
        "Permission reply: session={}, req={}, decision={}",
        _session_id,
        _req_id,
        decision
    );
    HttpResponse::Ok().json(serde_json::json!({
        "status": "ok",
        "session_id": _session_id,
        "request_id": _req_id,
        "decision": decision
    }))
}

pub fn init(cfg: &mut web::ServiceConfig) {
    cfg.route("", web::get().to(list_permissions));
}
