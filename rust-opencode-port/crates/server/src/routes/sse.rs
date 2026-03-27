use actix_web::{web, HttpResponse, Responder};
use crate::ServerState;

pub async fn sse_index(_state: web::Data<ServerState>) -> impl Responder {
    HttpResponse::Ok()
        .content_type("text/event-stream")
        .body("data: SSE endpoint (stub)\n\n")
}

pub fn init(cfg: &mut web::ServiceConfig) {
    cfg.route("", web::get().to(sse_index));
}