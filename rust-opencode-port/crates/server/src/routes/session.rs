use actix_web::{web, HttpResponse, Responder};
use serde::Deserialize;
use crate::ServerState;

#[derive(Deserialize)]
pub struct PaginationParams {
    pub limit: Option<usize>,
    pub offset: Option<usize>,
}

pub async fn list_sessions(
    state: web::Data<ServerState>,
    params: web::Query<PaginationParams>,
) -> impl Responder {
    let limit = params.limit.unwrap_or(20);
    let offset = params.offset.unwrap_or(0);

    match state.storage.list_sessions(limit, offset).await {
        Ok(sessions) => HttpResponse::Ok().json(sessions),
        Err(e) => HttpResponse::InternalServerError().body(e.to_string()),
    }
}

pub async fn get_session(
    state: web::Data<ServerState>,
    id: web::Path<String>,
) -> impl Responder {
    match state.storage.load_session(&id).await {
        Ok(Some(session)) => HttpResponse::Ok().json(session),
        Ok(None) => HttpResponse::NotFound().body("Session not found"),
        Err(e) => HttpResponse::InternalServerError().body(e.to_string()),
    }
}

pub async fn delete_session(
    state: web::Data<ServerState>,
    id: web::Path<String>,
) -> impl Responder {
    match state.storage.delete_session(&id).await {
        Ok(_) => HttpResponse::NoContent().finish(),
        Err(e) => HttpResponse::InternalServerError().body(e.to_string()),
    }
}

pub fn init(cfg: &mut web::ServiceConfig) {
    cfg.route("", web::get().to(list_sessions));
    cfg.route("/{id}", web::get().to(get_session));
    cfg.route("/{id}", web::delete().to(delete_session));
}