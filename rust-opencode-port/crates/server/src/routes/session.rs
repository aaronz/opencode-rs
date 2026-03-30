use actix_web::{web, HttpResponse, Responder};
use serde::Deserialize;
use opencode_core::{Message, Role, Session};
use crate::ServerState;

#[derive(Deserialize)]
pub struct PaginationParams {
    pub limit: Option<usize>,
    pub offset: Option<usize>,
}

#[derive(Deserialize)]
pub struct CreateSessionRequest {
    pub initial_prompt: Option<String>,
}

#[derive(Deserialize)]
pub struct AddMessageRequest {
    pub role: Option<String>,
    pub content: String,
}

fn parse_role(role: Option<String>) -> Role {
    match role
        .as_deref()
        .unwrap_or("user")
        .to_ascii_lowercase()
        .as_str()
    {
        "assistant" => Role::Assistant,
        "system" => Role::System,
        _ => Role::User,
    }
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

pub async fn create_session(
    state: web::Data<ServerState>,
    req: web::Json<CreateSessionRequest>,
) -> impl Responder {
    let mut session = Session::new();
    if let Some(prompt) = &req.initial_prompt {
        session.add_message(Message::user(prompt.clone()));
    }

    match state.storage.save_session(&session).await {
        Ok(_) => HttpResponse::Ok().json(serde_json::json!({
            "session_id": session.id.to_string(),
            "created_at": session.created_at.to_rfc3339(),
            "status": "created",
        })),
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

pub async fn fork_session(
    state: web::Data<ServerState>,
    id: web::Path<String>,
) -> impl Responder {
    match state.storage.load_session(&id).await {
        Ok(Some(session)) => {
            let mut forked = Session::new();
            forked.messages = session.messages.clone();
            forked.undo_history = session.undo_history.clone();
            forked.redo_history = session.redo_history.clone();

            match state.storage.save_session(&forked).await {
                Ok(_) => HttpResponse::Ok().json(serde_json::json!({
                    "session_id": forked.id.to_string(),
                    "forked_from": id.as_str(),
                })),
                Err(e) => HttpResponse::InternalServerError().body(e.to_string()),
            }
        }
        Ok(None) => HttpResponse::NotFound().body("Session not found"),
        Err(e) => HttpResponse::InternalServerError().body(e.to_string()),
    }
}

pub async fn prompt_session(
    state: web::Data<ServerState>,
    id: web::Path<String>,
    req: web::Json<AddMessageRequest>,
) -> impl Responder {
    add_message_to_session(state, id, req).await
}

pub async fn add_message_to_session(
    state: web::Data<ServerState>,
    id: web::Path<String>,
    req: web::Json<AddMessageRequest>,
) -> impl Responder {
    match state.storage.load_session(&id).await {
        Ok(Some(mut session)) => {
            let role = parse_role(req.role.clone());
            let message = Message::new(role, req.content.clone());
            session.add_message(message);

            match state.storage.save_session(&session).await {
                Ok(_) => HttpResponse::Ok().json(serde_json::json!({
                    "session_id": session.id.to_string(),
                    "message_count": session.messages.len(),
                })),
                Err(e) => HttpResponse::InternalServerError().body(e.to_string()),
            }
        }
        Ok(None) => HttpResponse::NotFound().body("Session not found"),
        Err(e) => HttpResponse::InternalServerError().body(e.to_string()),
    }
}

pub async fn list_messages(
    state: web::Data<ServerState>,
    id: web::Path<String>,
    params: web::Query<PaginationParams>,
) -> impl Responder {
    let limit = params.limit.unwrap_or(50);
    let offset = params.offset.unwrap_or(0);

    match state.storage.get_session_messages_paginated(&id, limit, offset).await {
        Ok(messages) => HttpResponse::Ok().json(messages),
        Err(e) => HttpResponse::InternalServerError().body(e.to_string()),
    }
}

pub async fn get_message(
    state: web::Data<ServerState>,
    path: web::Path<(String, usize)>,
) -> impl Responder {
    let (id, index) = path.into_inner();

    match state.storage.load_session(&id).await {
        Ok(Some(session)) => {
            if let Some(message) = session.messages.get(index) {
                HttpResponse::Ok().json(message)
            } else {
                HttpResponse::NotFound().body("Message not found")
            }
        }
        Ok(None) => HttpResponse::NotFound().body("Session not found"),
        Err(e) => HttpResponse::InternalServerError().body(e.to_string()),
    }
}

pub async fn share_session(
    state: web::Data<ServerState>,
    id: web::Path<String>,
) -> impl Responder {
    match state.storage.load_session(&id).await {
        Ok(Some(_)) => HttpResponse::Ok().json(serde_json::json!({
            "share_url": format!("https://opencode-rs.local/share/{}", id.as_str()),
        })),
        Ok(None) => HttpResponse::NotFound().body("Session not found"),
        Err(e) => HttpResponse::InternalServerError().body(e.to_string()),
    }
}

pub fn init(cfg: &mut web::ServiceConfig) {
    cfg.route("", web::get().to(list_sessions));
    cfg.route("", web::post().to(create_session));
    cfg.route("/{id}/fork", web::post().to(fork_session));
    cfg.route("/{id}/prompt", web::post().to(prompt_session));
    cfg.route("/{id}/messages", web::get().to(list_messages));
    cfg.route("/{id}/messages", web::post().to(add_message_to_session));
    cfg.route("/{id}/messages/{msg_index}", web::get().to(get_message));
    cfg.route("/{id}/share", web::post().to(share_session));
    cfg.route("/{id}", web::get().to(get_session));
    cfg.route("/{id}", web::delete().to(delete_session));
}
