use actix_web::{web, Error, HttpRequest, HttpResponse};
use crate::ServerState;

pub async fn ws_index(
    _state: web::Data<ServerState>,
    _req: HttpRequest,
    _stream: web::Payload,
) -> Result<HttpResponse, Error> {
    Ok(HttpResponse::Ok().body("WebSocket endpoint (stub)"))
}

pub fn init(cfg: &mut web::ServiceConfig) {
    cfg.route("", web::get().to(ws_index));
}
