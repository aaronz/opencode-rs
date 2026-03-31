use actix_web::{web, Error, HttpRequest, HttpResponse};
use actix_ws::Message;
use futures::StreamExt;
use serde::{Deserialize, Serialize};
use std::time::{Duration, Instant};
use tracing::{info, warn, debug};
use crate::ServerState;

/// WebSocket client message types
#[derive(Debug, Deserialize)]
#[serde(tag = "type")]
pub enum WsClientMessage {
    /// Request agent execution with streaming
    #[serde(rename = "run")]
    Run {
        session_id: String,
        message: String,
        agent_type: Option<String>,
    },
    /// Ping to keep connection alive
    #[serde(rename = "ping")]
    Ping,
    /// Close connection
    #[serde(rename = "close")]
    Close,
}

/// WebSocket server message types
#[derive(Debug, Serialize)]
#[serde(tag = "type")]
pub enum WsServerMessage {
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
    /// Pong response
    #[serde(rename = "pong")]
    Pong,
}

/// WebSocket connection configuration
const WS_HEARTBEAT_INTERVAL: Duration = Duration::from_secs(30);
const WS_CLIENT_TIMEOUT: Duration = Duration::from_secs(60);

pub async fn ws_index(
    state: web::Data<ServerState>,
    req: HttpRequest,
    stream: web::Payload,
) -> Result<HttpResponse, Error> {
    let (response, mut session, mut msg_stream) = actix_ws::handle(&req, stream)?;

    actix_rt::spawn(async move {
        let mut last_heartbeat = Instant::now();

        let welcome = WsServerMessage::Chunk {
            content: "Connected to OpenCode WebSocket".to_string(),
        };
        if let Ok(msg) = serde_json::to_string(&welcome) {
            let _ = session.text(msg).await;
        }

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
                        let ping_msg = WsServerMessage::Pong;
                        if let Ok(msg) = serde_json::to_string(&ping_msg) {
                            if session.text(msg).await.is_err() {
                                break;
                            }
                        }
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
                            handle_ws_message(&mut session, &text, &state).await;
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

        debug!("WebSocket connection closed");
    });

    Ok(response)
}

async fn handle_ws_message(
    session: &mut actix_ws::Session,
    text: &str,
    state: &web::Data<ServerState>,
) {
    match serde_json::from_str::<WsClientMessage>(text) {
        Ok(WsClientMessage::Ping) => {
            let response = WsServerMessage::Pong;
            if let Ok(msg) = serde_json::to_string(&response) {
                let _ = session.text(msg).await;
            }
        }
        Ok(WsClientMessage::Close) => {
            let _ = session.clone().close(None).await;
        }
        Ok(WsClientMessage::Run { session_id, message, agent_type }) => {
            info!("Executing agent run: session={}, agent={:?}", session_id, agent_type);

            let start = WsServerMessage::Start {
                session_id: session_id.clone(),
            };
            if let Ok(msg) = serde_json::to_string(&start) {
                let _ = session.text(msg).await;
            }

            // TODO: Integrate actual agent execution with streaming response
            let chunks = vec![
                "Processing request... ",
                "Analyzing code... ",
                "Generating response... ",
                "Complete.",
            ];

            for chunk in chunks {
                let msg = WsServerMessage::Chunk {
                    content: chunk.to_string(),
                };
                if let Ok(json) = serde_json::to_string(&msg) {
                    if session.text(json).await.is_err() {
                        return;
                    }
                }
                tokio::time::sleep(Duration::from_millis(100)).await;
            }

            let end = WsServerMessage::End {
                session_id: session_id.clone(),
            };
            if let Ok(msg) = serde_json::to_string(&end) {
                let _ = session.text(msg).await;
            }
        }
        Err(e) => {
            warn!("Failed to parse WebSocket message: {}", e);
            let error = WsServerMessage::Error {
                message: format!("Invalid message format: {}", e),
            };
            if let Ok(msg) = serde_json::to_string(&error) {
                let _ = session.text(msg).await;
            }
        }
    }
}

pub fn init(cfg: &mut web::ServiceConfig) {
    cfg.route("", web::get().to(ws_index));
}
