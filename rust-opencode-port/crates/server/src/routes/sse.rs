use actix_web::{web, HttpResponse, Error, HttpRequest};
use futures::stream::{self, Stream};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::mpsc;
use tracing::info;
use opencode_core::{Message as CoreMessage, Session};
use crate::ServerState;

#[derive(Debug, Serialize)]
#[serde(tag = "type")]
pub enum SseEvent {
    #[serde(rename = "chunk")]
    Chunk { content: String, session_id: String, event_id: u64 },
    #[serde(rename = "start")]
    Start { session_id: String, model: String },
    #[serde(rename = "end")]
    End { session_id: String, message_count: usize },
    #[serde(rename = "error")]
    Error { message: String, code: Option<String> },
}

#[derive(Debug, Deserialize)]
pub struct SseQuery {
    pub session_id: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct SseMessageRequest {
    pub message: String,
    pub model: Option<String>,
}

pub async fn sse_index(
    state: web::Data<ServerState>,
    req: HttpRequest,
    query: web::Query<SseQuery>,
) -> Result<HttpResponse, Error> {
    let session_id = query.session_id.clone().unwrap_or_else(|| "default".to_string());
    
    let last_event_id = req
        .headers()
        .get("Last-Event-ID")
        .and_then(|v| v.to_str().ok())
        .and_then(|v| v.parse::<u64>().ok())
        .unwrap_or(0);

    info!("SSE connection: session={}, last_event_id={}", session_id, last_event_id);

    let state_clone = state.into_inner();
    let stream = create_event_stream(session_id, last_event_id, state_clone);

    Ok(HttpResponse::Ok()
        .content_type("text/event-stream")
        .insert_header(("Cache-Control", "no-cache"))
        .insert_header(("Connection", "keep-alive"))
        .insert_header(("Access-Control-Allow-Origin", "*"))
        .insert_header(("X-Accel-Buffering", "no"))
        .streaming(Box::pin(stream)))
}

fn create_event_stream(
    session_id: String,
    last_event_id: u64,
    _state: Arc<ServerState>,
) -> impl Stream<Item = Result<web::Bytes, Error>> {
    let (tx, rx) = mpsc::channel::<SseEvent>(100);

    let session_id_clone = session_id.clone();
    actix_rt::spawn(async move {
        let _ = tx.send(SseEvent::Start {
            session_id: session_id_clone.clone(),
            model: "connected".to_string(),
        }).await;

        let welcome = SseEvent::Chunk {
            content: "Connected to OpenCode SSE".to_string(),
            session_id: session_id_clone.clone(),
            event_id: last_event_id + 1,
        };
        let _ = tx.send(welcome).await;
    });

    stream::unfold(rx, |mut rx| async move {
        match rx.recv().await {
            Some(event) => {
                let event_id = match &event {
                    SseEvent::Chunk { event_id, .. } => *event_id,
                    _ => 0,
                };
                
                let data = serde_json::to_string(&event).unwrap_or_default();
                let sse_format = if event_id > 0 {
                    format!("id: {}\ndata: {}\n\n", event_id, data)
                } else {
                    format!("data: {}\n\n", data)
                };
                
                Some((Ok::<_, Error>(web::Bytes::from(sse_format)), rx))
            }
            None => None,
        }
    })
}

pub async fn sse_send_message(
    state: web::Data<ServerState>,
    session_path: web::Path<String>,
    req: web::Json<SseMessageRequest>,
) -> Result<HttpResponse, Error> {
    let session_id = session_path.into_inner();
    info!("SSE message: session={}", session_id);

    let mut core_session = match state.storage.load_session(&session_id).await {
        Ok(Some(s)) => s,
        Ok(None) => Session::new(),
        Err(e) => {
            return Ok(HttpResponse::InternalServerError()
                .json(serde_json::json!({"error": e.to_string()})));
        }
    };

    core_session.add_message(CoreMessage::user(req.message.clone()));

    if let Err(e) = state.storage.save_session(&core_session).await {
        tracing::warn!("Failed to save session: {}", e);
    }

    Ok(HttpResponse::Ok().json(serde_json::json!({
        "session_id": session_id,
        "message": "Message received. Agent execution not yet integrated.",
        "message_count": core_session.messages.len(),
    })))
}

pub fn init(cfg: &mut web::ServiceConfig) {
    cfg.route("", web::get().to(sse_index));
    cfg.route("/{session_id}/message", web::post().to(sse_send_message));
}
