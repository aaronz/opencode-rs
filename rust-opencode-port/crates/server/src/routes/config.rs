use actix_web::{web, HttpResponse, Responder};
use crate::ServerState;
use opencode_core::Config;

pub async fn get_config(state: web::Data<ServerState>) -> impl Responder {
    HttpResponse::Ok().json(&*state.config)
}

pub async fn update_config(
    _state: web::Data<ServerState>,
    _req: web::Json<Config>,
) -> impl Responder {
    HttpResponse::Ok().json(serde_json::json!({
        "status": "success",
        "message": "Configuration update received (stub)"
    }))
}

pub fn init(cfg: &mut web::ServiceConfig) {
    cfg.route("", web::get().to(get_config));
    cfg.route("", web::patch().to(update_config));
}
