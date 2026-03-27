use actix_web::{web, HttpResponse, Responder};
use serde::Deserialize;
use opencode_core::{Message, Session};
use crate::ServerState;

#[derive(Deserialize)]
pub struct RunRequest {
    pub prompt: String,
    pub model: Option<String>,
    pub agent: Option<String>,
}

pub async fn run_prompt(
    state: web::Data<ServerState>,
    req: web::Json<RunRequest>,
) -> impl Responder {
    let mut session = Session::new();
    session.add_message(Message::user(&req.prompt));
    
    match state.storage.save_session(&session).await {
        Ok(_) => HttpResponse::Ok().json(serde_json::json!({
            "session_id": session.id.to_string(),
            "status": "started",
            "message": "Agent processing started (stub)"
        })),
        Err(e) => HttpResponse::InternalServerError().body(e.to_string()),
    }
}

pub fn init(cfg: &mut web::ServiceConfig) {
    cfg.route("", web::post().to(run_prompt));
}
