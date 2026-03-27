use actix_web::{web, HttpResponse, Responder};
use crate::ServerState;

pub async fn list_permissions(_state: web::Data<ServerState>) -> impl Responder {
    HttpResponse::Ok().json(serde_json::json!({
        "permissions": ["repo:read", "repo:write", "session:all"]
    }))
}

pub fn init(cfg: &mut web::ServiceConfig) {
    cfg.route("", web::get().to(list_permissions));
}
