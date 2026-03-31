use actix_web::{web, HttpResponse, Error};
use futures::stream::{self, Stream};
use serde::{Deserialize, Serialize};
use tracing::info;
use crate::ServerState;

/// SSE event types
#[derive(Debug, Serialize)]
#[serde(tag = "type")]
pub enum SseEvent {
    /// Streaming response chunk
    #[serde(rename = "chunk")]
    Chunk { content: String },
    /// Execution started
    #[serde(rename = "start")]
    Start { session_id: String },
    /// Execution completed
    #[serde(rename = "end")]
    End { session_id: String },
    /// Error message
    #[serde(rename = "error")]
    Error { message: String },
}

/// Query parameters for SSE connection
#[derive(Debug, Deserialize)]
pub struct SseQuery {
    pub session_id: Option<String>,
}

pub async fn sse_index(
    state: web::Data<ServerState>,
    query: web::Query<SseQuery>,
) -> Result<HttpResponse, Error> {
    info!("SSE connection established: session={:?}", query.session_id);

    let session_id = query.session_id.clone().unwrap_or_else(|| "default".to_string());

    let start = SseEvent::Start {
        session_id: session_id.clone(),
    };
    let start_data = serde_json::to_string(&start).unwrap_or_default();

    let stream = create_event_stream(session_id);

    Ok(HttpResponse::Ok()
        .content_type("text/event-stream")
        .insert_header(("Cache-Control", "no-cache"))
        .insert_header(("Connection", "keep-alive"))
        .insert_header(("Access-Control-Allow-Origin", "*"))
        .streaming(Box::pin(stream)))
}

fn create_event_stream(session_id: String) -> impl Stream<Item = Result<web::Bytes, Error>> {
    let start = SseEvent::Start {
        session_id: session_id.clone(),
    };
    let start_data = serde_json::to_string(&start).unwrap_or_default();
    let start_bytes = web::Bytes::from(format!("data: {}\n\n", start_data));

    let welcome = web::Bytes::from("data: {\"type\":\"chunk\",\"content\":\"Connected to OpenCode SSE\"}\n\n");

    let end = SseEvent::End {
        session_id: session_id.clone(),
    };
    let end_data = serde_json::to_string(&end).unwrap_or_default();
    let end_bytes = web::Bytes::from(format!("data: {}\n\n", end_data));

    stream::iter(vec![
        Ok::<_, Error>(start_bytes),
        Ok::<_, Error>(welcome),
        Ok::<_, Error>(end_bytes),
    ])
}

pub fn init(cfg: &mut web::ServiceConfig) {
    cfg.route("", web::get().to(sse_index));
}