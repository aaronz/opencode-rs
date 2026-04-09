use std::sync::Arc;

use actix_web::{Error, HttpRequest, HttpResponse, web};
use futures::stream::{self, Stream};
use opencode_core::bus::InternalEvent;
use opencode_core::{Message as CoreMessage, Session};
use serde::Deserialize;
use tokio::sync::mpsc;
use tracing::{info, warn, debug};

use crate::ServerState;
use crate::streaming::heartbeat::HeartbeatManager;
use crate::streaming::{ReplayEntry, StreamMessage};
use crate::streaming::conn_state::ConnectionType;

#[derive(Debug, Deserialize)]
pub struct SseQuery {
    pub session_id: Option<String>,
    pub reconnect_token: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct SseMessageRequest {
    pub message: String,
    pub model: Option<String>,
}

#[derive(Debug)]
struct OutboundSse {
    message: StreamMessage,
    event_id: Option<u64>,
    record: bool,
}

pub async fn sse_index(
    state: web::Data<ServerState>,
    req: HttpRequest,
    query: web::Query<SseQuery>,
) -> Result<HttpResponse, Error> {
    let session_id = query
        .session_id
        .clone()
        .unwrap_or_else(|| "default".to_string());

    let connection_id = format!("sse-{}-{}", session_id, uuid::Uuid::new_v4());
    
    let last_event_id = req
        .headers()
        .get("Last-Event-ID")
        .and_then(|v| v.to_str().ok())
        .and_then(|v| v.parse::<u64>().ok())
        .unwrap_or(0);

    let resume_from = query
        .reconnect_token
        .as_ref()
        .and_then(|token| state.reconnection_store.validate_token(&session_id, token))
        .unwrap_or(last_event_id);

    let reconnect_token = state
        .reconnection_store
        .generate_token(&session_id, Some(resume_from));
    
    state.connection_monitor.register_connection(
        connection_id.clone(),
        ConnectionType::Sse,
        session_id.clone(),
    ).await;

    info!(
        "SSE connect: session_id={}, last_event_id={}, resume_from={}, connection_id={}",
        session_id, last_event_id, resume_from, connection_id
    );

    let stream = create_event_stream(session_id, resume_from, state.into_inner(), connection_id);

    Ok(HttpResponse::Ok()
        .content_type("text/event-stream")
        .insert_header(("Cache-Control", "no-cache"))
        .insert_header(("Connection", "keep-alive"))
        .insert_header(("Access-Control-Allow-Origin", "*"))
        .insert_header(("X-Accel-Buffering", "no"))
        .insert_header(("X-Reconnect-Token", reconnect_token))
        .streaming(Box::pin(stream)))
}

fn create_event_stream(
    session_id: String,
    resume_from: u64,
    state: Arc<ServerState>,
    connection_id: String,
) -> impl Stream<Item = Result<web::Bytes, Error>> {
    let (tx, rx) = mpsc::channel::<OutboundSse>(128);

    let tx_bootstrap = tx.clone();
    let state_bootstrap = Arc::clone(&state);
    let session_bootstrap = session_id.clone();
    let connection_monitor = Arc::clone(&state.connection_monitor);
    let conn_id_bootstrap = connection_id.clone();
    actix_rt::spawn(async move {
        let _ = tx_bootstrap
            .send(OutboundSse {
                message: StreamMessage::Connected {
                    session_id: Some(session_bootstrap.clone()),
                },
                event_id: None,
                record: false,
            })
            .await;
        
        connection_monitor.heartbeat_success(&conn_id_bootstrap).await;

        for ReplayEntry { sequence, message } in state_bootstrap
            .reconnection_store
            .replay_from(&session_bootstrap, resume_from)
        {
            let _ = tx_bootstrap
                .send(OutboundSse {
                    message,
                    event_id: Some(sequence),
                    record: false,
                })
                .await;
        }
    });

    let (hb_tx, mut hb_rx) = mpsc::channel::<StreamMessage>(32);
    let heartbeat = HeartbeatManager::default();
    let heartbeat_handle = heartbeat.spawn(hb_tx);
    let tx_heartbeat = tx.clone();
    let conn_monitor_hb = Arc::clone(&state.connection_monitor);
    let conn_id_hb = connection_id.clone();
    actix_rt::spawn(async move {
        while let Some(message) = hb_rx.recv().await {
            conn_monitor_hb.heartbeat_success(&conn_id_hb).await;
            if tx_heartbeat
                .send(OutboundSse {
                    message,
                    event_id: None,
                    record: false,
                })
                .await
                .is_err()
            {
                break;
            }
        }
    });

    let tx_bus = tx.clone();
    let mut bus_rx = state.event_bus.subscribe();
    let session_filter = session_id.clone();
    actix_rt::spawn(async move {
        loop {
            match bus_rx.recv().await {
                Ok(event) => {
                    if let Some(message) = event_to_stream_message(event, &session_filter) {
                        if tx_bus
                            .send(OutboundSse {
                                message,
                                event_id: None,
                                record: true,
                            })
                            .await
                            .is_err()
                        {
                            break;
                        }
                    }
                }
                Err(tokio::sync::broadcast::error::RecvError::Lagged(_)) => continue,
                Err(tokio::sync::broadcast::error::RecvError::Closed) => break,
            }
        }
    });

    let conn_monitor_cleanup = Arc::clone(&state.connection_monitor);
    let conn_id_cleanup = connection_id.clone();
    stream::unfold((rx, state, session_id, heartbeat_handle, conn_monitor_cleanup, conn_id_cleanup), |(mut rx, state, session_id, heartbeat_handle, conn_monitor, conn_id)| async move {
        match rx.recv().await {
            Some(outbound) => {
                let event_id = if outbound.record {
                    outbound
                        .message
                        .session_id()
                        .filter(|sid| *sid == session_id.as_str())
                        .map(|sid| {
                            let seq = state.reconnection_store.record_message(sid, outbound.message.clone());
                            debug!("SSE recorded message: session_id={}, sequence={}", sid, seq);
                            seq
                        })
                        .or(outbound.event_id)
                } else {
                    outbound.event_id
                };

                let payload = serde_json::to_string(&outbound.message).unwrap_or_else(|_| {
                    serde_json::json!({
                        "type": "error",
                        "session_id": session_id,
                        "code": "SERIALIZATION_ERROR",
                        "message": "failed to serialize stream payload"
                    })
                    .to_string()
                });

                let formatted = if let Some(id) = event_id {
                    format!("id: {id}\nevent: {}\ndata: {payload}\n\n", message_event_type(&outbound.message))
                } else {
                    format!("data: {payload}\n\n")
                };

                Some((
                    Ok::<_, Error>(web::Bytes::from(formatted)),
                    (rx, state, session_id, heartbeat_handle, conn_monitor, conn_id),
                ))
            }
            None => {
                heartbeat_handle.abort();
                conn_monitor.unregister_connection(&conn_id, "stream_ended").await;
                None
            }
        }
    })
}

fn message_event_type(message: &StreamMessage) -> &'static str {
    match message {
        StreamMessage::Message { .. } => "message",
        StreamMessage::ToolCall { .. } => "tool_call",
        StreamMessage::ToolResult { .. } => "tool_result",
        StreamMessage::SessionUpdate { .. } => "session_update",
        StreamMessage::Heartbeat { .. } => "heartbeat",
        StreamMessage::Error { .. } => "error",
        StreamMessage::Connected { .. } => "connected",
    }
}

fn event_to_stream_message(event: InternalEvent, session_id: &str) -> Option<StreamMessage> {
    let candidate = StreamMessage::from_internal_event(&event)?;
    match candidate.session_id() {
        Some(source_session) if source_session == session_id => Some(candidate),
        Some(_) => None,
        None => Some(candidate),
    }
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
            return Ok(HttpResponse::InternalServerError().json(StreamMessage::Error {
                session_id: Some(session_id),
                code: "session_load_error".to_string(),
                message: e.to_string(),
            }));
        }
    };

    core_session.add_message(CoreMessage::user(req.message.clone()));

    if let Err(e) = state.storage.save_session(&core_session).await {
        warn!("Failed to save session: {}", e);
    }

    let status = req
        .model
        .clone()
        .unwrap_or_else(|| "message_received".to_string());

    Ok(HttpResponse::Ok().json(StreamMessage::SessionUpdate { session_id, status }))
}

pub fn init(cfg: &mut web::ServiceConfig) {
    cfg.route("", web::get().to(sse_index));
    cfg.route("/{session_id}/message", web::post().to(sse_send_message));
}
