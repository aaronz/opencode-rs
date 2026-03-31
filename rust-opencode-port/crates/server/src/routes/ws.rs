use actix_web::{web, Error, HttpRequest, HttpResponse};
use actix_ws::Message;
use futures::StreamExt;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::mpsc;
use tracing::{info, warn, debug};
use opencode_core::{Message as CoreMessage, Session};
use crate::ServerState;

#[derive(Debug, Deserialize)]
#[serde(tag = "type")]
pub enum WsClientMessage {
    #[serde(rename = "run")]
    Run {
        session_id: String,
        message: String,
        agent_type: Option<String>,
        model: Option<String>,
    },
    #[serde(rename = "ping")]
    Ping,
    #[serde(rename = "close")]
    Close,
}

#[derive(Debug, Serialize)]
#[serde(tag = "type")]
pub enum WsServerMessage {
    #[serde(rename = "chunk")]
    Chunk { content: String, session_id: String },
    #[serde(rename = "tool_start")]
    ToolStart { tool_name: String, arguments: serde_json::Value },
    #[serde(rename = "tool_end")]
    ToolEnd { tool_name: String, result: String },
    #[serde(rename = "start")]
    Start { session_id: String, model: String },
    #[serde(rename = "end")]
    End { session_id: String, message_count: usize },
    #[serde(rename = "error")]
    Error { message: String, code: Option<String> },
    #[serde(rename = "pong")]
    Pong,
}

const WS_HEARTBEAT_INTERVAL: Duration = Duration::from_secs(30);
const WS_CLIENT_TIMEOUT: Duration = Duration::from_secs(120);

pub async fn ws_index(
    state: web::Data<ServerState>,
    req: HttpRequest,
    stream: web::Payload,
) -> Result<HttpResponse, Error> {
    let (response, mut session, mut msg_stream) = actix_ws::handle(&req, stream)?;
    let state_clone = state.into_inner();

    actix_rt::spawn(async move {
        let mut last_heartbeat = Instant::now();
        let (tx, mut rx) = mpsc::channel::<WsServerMessage>(100);

        let mut session_clone = session.clone();
        let forwarder = actix_rt::spawn(async move {
            while let Some(msg) = rx.recv().await {
                if let Ok(json) = serde_json::to_string(&msg) {
                    if session_clone.text(json).await.is_err() {
                        break;
                    }
                }
            }
        });

        let _ = tx.send(WsServerMessage::Chunk {
            content: "Connected to OpenCode WebSocket".to_string(),
            session_id: String::new(),
        }).await;

        loop {
            if last_heartbeat.elapsed() > WS_CLIENT_TIMEOUT {
                warn!("WebSocket client heartbeat timeout");
                let _ = session.close(None).await;
                break;
            }

            let tick = actix_rt::time::sleep(WS_HEARTBEAT_INTERVAL);
            tokio::pin!(tick);

            tokio::select! {
                _ = &mut tick => {
                    if Instant::now().duration_since(last_heartbeat) > WS_HEARTBEAT_INTERVAL {
                        debug!("Sending WebSocket heartbeat ping");
                        let _ = tx.send(WsServerMessage::Pong).await;
                    }
                }
                Some(Ok(msg)) = msg_stream.next() => {
                    match msg {
                        Message::Ping(bytes) => {
                            if session.pong(&bytes).await.is_err() {
                                break;
                            }
                        }
                        Message::Pong(_) => {
                            last_heartbeat = Instant::now();
                        }
                        Message::Text(text) => {
                            debug!("Received WebSocket message: {}", text);
                            handle_ws_message(&mut session, &text, &state_clone, &tx).await;
                        }
                        Message::Binary(_) => {
                            warn!("Binary messages not supported");
                        }
                        Message::Close(reason) => {
                            info!("WebSocket client closed connection: {:?}", reason);
                            let _ = session.close(reason).await;
                            break;
                        }
                        _ => break,
                    }
                }
                else => break,
            }
        }

        forwarder.abort();
        debug!("WebSocket connection closed");
    });

    Ok(response)
}

async fn handle_ws_message(
    _session: &mut actix_ws::Session,
    text: &str,
    state: &Arc<ServerState>,
    tx: &mpsc::Sender<WsServerMessage>,
) {
    match serde_json::from_str::<WsClientMessage>(text) {
        Ok(WsClientMessage::Ping) => {
            let _ = tx.send(WsServerMessage::Pong).await;
        }
        Ok(WsClientMessage::Close) => {
            let _ = _session.clone().close(None).await;
        }
        Ok(WsClientMessage::Run { session_id, message, agent_type: _, model: _ }) => {
            info!("WebSocket run: session={}", session_id);

            let mut core_session = match state.storage.load_session(&session_id).await {
                Ok(Some(s)) => s,
                Ok(None) => Session::new(),
                Err(e) => {
                    let _ = tx.send(WsServerMessage::Error {
                        message: format!("Failed to load session: {}", e),
                        code: Some("SESSION_LOAD_ERROR".to_string()),
                    }).await;
                    return;
                }
            };

            core_session.add_message(CoreMessage::user(message.clone()));

            if let Err(e) = state.storage.save_session(&core_session).await {
                warn!("Failed to save session: {}", e);
            }

            let _ = tx.send(WsServerMessage::Start {
                session_id: session_id.clone(),
                model: "pending".to_string(),
            }).await;

            let _ = tx.send(WsServerMessage::Chunk {
                content: "Message received. Agent execution not yet integrated.".to_string(),
                session_id: session_id.clone(),
            }).await;

            let _ = tx.send(WsServerMessage::End {
                session_id: session_id.clone(),
                message_count: core_session.messages.len(),
            }).await;
        }
        Err(e) => {
            warn!("Failed to parse WebSocket message: {}", e);
            let _ = tx.send(WsServerMessage::Error {
                message: format!("Invalid message format: {}", e),
                code: Some("PARSE_ERROR".to_string()),
            }).await;
        }
    }
}

pub fn init(cfg: &mut web::ServiceConfig) {
    cfg.route("", web::get().to(ws_index));
}
