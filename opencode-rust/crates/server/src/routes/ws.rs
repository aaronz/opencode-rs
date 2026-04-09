use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};

use actix_web::{Error, HttpRequest, HttpResponse, web};
use actix_web::http::header::{HeaderName, HeaderValue};
use actix_ws::Message;
use futures::StreamExt;
use opencode_core::bus::InternalEvent;
use opencode_core::{Message as CoreMessage, Session};
use serde::Deserialize;
use tokio::sync::mpsc;
use tracing::{debug, info, warn, error};

use crate::ServerState;
use crate::streaming::heartbeat::HeartbeatManager;
use crate::streaming::{ReplayEntry, StreamMessage};
use crate::streaming::conn_state::ConnectionType;

#[derive(Debug, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum WsClientMessage {
    Run {
        session_id: String,
        message: String,
        agent_type: Option<String>,
        model: Option<String>,
    },
    Resume {
        session_id: String,
        token: String,
    },
    Ping,
    Close,
}

const WS_HEARTBEAT_INTERVAL: Duration = Duration::from_secs(30);
const WS_CLIENT_TIMEOUT: Duration = Duration::from_secs(120);

pub async fn ws_index(
    state: web::Data<ServerState>,
    req: HttpRequest,
    stream: web::Payload,
) -> Result<HttpResponse, Error> {
    let query = parse_query(req.query_string());
    let handshake_session_id = query
        .get("session_id")
        .cloned()
        .unwrap_or_else(|| "default".to_string());
    let incoming_token = query.get("token").cloned();
    let connection_id = format!("ws-{}-{}", handshake_session_id, uuid::Uuid::new_v4());

    let resume_from = incoming_token
        .as_ref()
        .and_then(|token| {
            state
                .reconnection_store
                .validate_token(&handshake_session_id, token)
        })
        .unwrap_or(0);
    let reconnect_token = state
        .reconnection_store
        .generate_token(&handshake_session_id, Some(resume_from));
    
    state.connection_monitor.register_connection(
        connection_id.clone(),
        ConnectionType::WebSocket,
        handshake_session_id.clone(),
    ).await;

    let ws_result = actix_ws::handle(&req, stream);
    
    let (mut response, mut session, mut msg_stream) = match ws_result {
        Ok(result) => result,
        Err(e) => {
            let err_msg = format!("WebSocket handshake failed: {}", e);
            error!(
                "WS handshake error: connection_id={}, session_id={}, error={}",
                connection_id, handshake_session_id, e
            );
            state.connection_monitor.connection_failed(&connection_id, &err_msg).await;
            state.connection_monitor.unregister_connection(&connection_id, "handshake_failed").await;
            
            return Ok(HttpResponse::BadRequest()
                .content_type("application/json")
                .json(serde_json::json!({
                    "error": "websocket_handshake_failed",
                    "message": "Failed to establish WebSocket connection",
                    "details": e.to_string()
                })));
        }
    };
    
    if let Ok(header_value) = HeaderValue::from_str(&reconnect_token) {
        response
            .headers_mut()
            .insert(HeaderName::from_static("x-reconnect-token"), header_value);
    }

    let state = state.into_inner();
    let conn_monitor = Arc::clone(&state.connection_monitor);
    let conn_id = connection_id.clone();
    actix_rt::spawn(async move {
        let (tx, mut rx) = mpsc::channel::<StreamMessage>(128);
        let mut last_heartbeat = Instant::now();
        let conn_id_for_task = conn_id.clone();

        let session_replay_id = handshake_session_id.clone();
        let tx_bootstrap = tx.clone();
        let state_bootstrap = Arc::clone(&state);
        actix_rt::spawn(async move {
            let _ = tx_bootstrap
                .send(StreamMessage::Connected {
                    session_id: Some(session_replay_id.clone()),
                })
                .await;

            for ReplayEntry { message, .. } in state_bootstrap
                .reconnection_store
                .replay_from(&session_replay_id, resume_from)
            {
                if tx_bootstrap.send(message).await.is_err() {
                    break;
                }
            }
        });

        let heartbeat = HeartbeatManager::new(WS_HEARTBEAT_INTERVAL);
        let _heartbeat_handle = heartbeat.spawn(tx.clone());

        let tx_bus = tx.clone();
        let session_filter = handshake_session_id.clone();
        let mut bus_rx = state.event_bus.subscribe();
        actix_rt::spawn(async move {
            loop {
                match bus_rx.recv().await {
                    Ok(event) => {
                        if let Some(message) = event_to_stream_message(event, &session_filter) {
                            if tx_bus.send(message).await.is_err() {
                                break;
                            }
                        }
                    }
                    Err(tokio::sync::broadcast::error::RecvError::Lagged(_)) => continue,
                    Err(tokio::sync::broadcast::error::RecvError::Closed) => break,
                }
            }
        });

        loop {
            if last_heartbeat.elapsed() > WS_CLIENT_TIMEOUT {
                warn!("WebSocket heartbeat timeout");
                conn_monitor.unregister_connection(&conn_id_for_task, "heartbeat_timeout").await;
                let _ = session.close(None).await;
                break;
            }

            tokio::select! {
                Some(outgoing) = rx.recv() => {
                    if let Some(session_id) = outgoing.session_id() {
                        state.reconnection_store.record_message(session_id, outgoing.clone());
                    }

                    if let Ok(json) = serde_json::to_string(&outgoing) {
                        if session.text(json).await.is_err() {
                            conn_monitor.unregister_connection(&conn_id_for_task, "send_error").await;
                            break;
                        }
                    } else {
                        let fallback = StreamMessage::Error {
                            session_id: Some(handshake_session_id.clone()),
                            code: "serialization_error".to_string(),
                            message: "failed to serialize websocket stream payload".to_string(),
                        };
                        if let Ok(json) = serde_json::to_string(&fallback) {
                            let _ = session.text(json).await;
                        }
                    }
                }
                Some(Ok(msg)) = msg_stream.next() => {
                    match msg {
                        Message::Ping(bytes) => {
                            conn_monitor.heartbeat_success(&conn_id_for_task).await;
                            if session.pong(&bytes).await.is_err() {
                                conn_monitor.unregister_connection(&conn_id_for_task, "pong_error").await;
                                break;
                            }
                            last_heartbeat = Instant::now();
                        }
                        Message::Pong(_) => {
                            conn_monitor.heartbeat_success(&conn_id_for_task).await;
                            last_heartbeat = Instant::now();
                        }
                        Message::Text(text) => {
                            debug!("WS inbound: {}", text);
                            handle_ws_message(&mut session, &text, &state, &tx).await;
                            conn_monitor.heartbeat_success(&conn_id_for_task).await;
                            last_heartbeat = Instant::now();
                        }
                        Message::Close(reason) => {
                            info!("WS closed: {:?}", reason);
                            conn_monitor.unregister_connection(&conn_id_for_task, "client_close").await;
                            let _ = session.close(reason).await;
                            break;
                        }
                        Message::Binary(_) => {
                            let _ = tx.send(StreamMessage::Error {
                                session_id: Some(handshake_session_id.clone()),
                                code: "unsupported_binary".to_string(),
                                message: "binary websocket messages are not supported".to_string(),
                            }).await;
                        }
                        _ => {
                            conn_monitor.unregister_connection(&conn_id_for_task, "unknown_message").await;
                            break;
                        }
                    }
                }
                else => break,
            }
        }
    });

    Ok(response)
}

async fn handle_ws_message(
    session: &mut actix_ws::Session,
    text: &str,
    state: &Arc<ServerState>,
    tx: &mpsc::Sender<StreamMessage>,
) {
    match serde_json::from_str::<WsClientMessage>(text) {
        Ok(WsClientMessage::Ping) => {
            let _ = tx
                .send(StreamMessage::Heartbeat {
                    timestamp: chrono::Utc::now().timestamp(),
                })
                .await;
        }
        Ok(WsClientMessage::Close) => {
            let _ = session.clone().close(None).await;
        }
        Ok(WsClientMessage::Resume { session_id, token }) => {
            match state.reconnection_store.validate_token(&session_id, &token) {
                Some(sequence) => {
                    for ReplayEntry { message, .. } in
                        state.reconnection_store.replay_from(&session_id, sequence)
                    {
                        let _ = tx.send(message).await;
                    }
                }
                None => {
                    let _ = tx
                        .send(StreamMessage::Error {
                            session_id: Some(session_id),
                            code: "invalid_reconnect_token".to_string(),
                            message: "unable to resume stream for provided token".to_string(),
                        })
                        .await;
                }
            }
        }
        Ok(WsClientMessage::Run {
            session_id,
            message,
            agent_type: _,
            model,
        }) => {
            info!("WebSocket run: session={}", session_id);

            let mut core_session = match state.storage.load_session(&session_id).await {
                Ok(Some(s)) => s,
                Ok(None) => Session::new(),
                Err(e) => {
                    let _ = tx
                        .send(StreamMessage::Error {
                            session_id: Some(session_id),
                            code: "session_load_error".to_string(),
                            message: format!("failed to load session: {e}"),
                        })
                        .await;
                    return;
                }
            };

            core_session.add_message(CoreMessage::user(message.clone()));

            if let Err(e) = state.storage.save_session(&core_session).await {
                warn!("Failed to save session: {}", e);
            }

            let _ = tx
                .send(StreamMessage::SessionUpdate {
                    session_id: session_id.clone(),
                    status: model.unwrap_or_else(|| "pending".to_string()),
                })
                .await;

            let _ = tx
                .send(StreamMessage::Message {
                    session_id: session_id.clone(),
                    content: "Message received. Agent execution not yet integrated.".to_string(),
                    role: "assistant".to_string(),
                })
                .await;

            let _ = tx
                .send(StreamMessage::SessionUpdate {
                    session_id,
                    status: format!("messages:{}", core_session.messages.len()),
                })
                .await;
        }
        Err(e) => {
            let _ = tx
                .send(StreamMessage::Error {
                    session_id: None,
                    code: "parse_error".to_string(),
                    message: format!("invalid websocket payload: {e}"),
                })
                .await;
        }
    }
}

fn parse_query(query: &str) -> HashMap<String, String> {
    query
        .split('&')
        .filter_map(|pair| {
            let mut parts = pair.splitn(2, '=');
            let key = parts.next()?;
            let value = parts.next().unwrap_or_default();
            Some((key.to_string(), value.to_string()))
        })
        .collect()
}

fn event_to_stream_message(event: InternalEvent, session_id: &str) -> Option<StreamMessage> {
    let candidate = StreamMessage::from_internal_event(&event)?;
    match candidate.session_id() {
        Some(source_session) if source_session == session_id => Some(candidate),
        Some(_) => None,
        None => Some(candidate),
    }
}

pub fn init(cfg: &mut web::ServiceConfig) {
    cfg.route("", web::get().to(ws_index));
}
