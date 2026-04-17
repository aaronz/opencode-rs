//! WebSocket streaming module for real-time bidirectional communication.
//!
//! This module provides full bidirectional WebSocket streaming capabilities for the OpenCode server.
//! Unlike SSE (Server-Sent Events) which is unidirectional, WebSocket connections allow both the
//! client and server to send messages at any time.
//!
//! ## Features
//!
//! - **Bidirectional Streaming**: Full duplex communication where both client and server can send messages
//! - **Session Management**: Multiple concurrent WebSocket clients per session with broadcast support
//! - **Heartbeat/Keepalive**: Automatic connection health monitoring with 30-second intervals
//! - **Reconnection Support**: Token-based reconnection with message replay for dropped connections
//! - **Multi-Session Broadcasting**: Broadcast messages to all clients in a session
//!
//! ## Endpoint
//!
//! ```text
//! GET /ws[/{session_id}]
//! GET /ws?session_id={session_id}&token={reconnect_token}
//! ```
//!
//! ## Connection Flow
//!
//! 1. Client initiates WebSocket handshake to `/ws` or `/ws/{session_id}`
//! 2. Server responds with `x-reconnect-token` header for reconnection support
//! 3. Server sends `Connected` message with session_id
//! 4. Server replays last 100 messages if reconnecting with valid token
//! 5. Bidirectional message exchange begins
//!
//! ## Server-to-Client Messages (StreamMessage)
//!
//! The server sends JSON messages with a `type` field:
//!
//! ```json
//! {"type": "connected", "session_id": "abc123"}
//! {"type": "message", "session_id": "abc123", "content": "...", "role": "assistant"}
//! {"type": "tool_call", "session_id": "abc123", "tool_name": "...", "call_id": "..."}
//! {"type": "tool_result", "session_id": "abc123", "call_id": "...", "output": "...", "success": true}
//! {"type": "session_update", "session_id": "abc123", "status": "running"}
//! {"type": "heartbeat", "timestamp": 1234567890}
//! {"type": "error", "error": "...", "code": "...", "message": "..."}
//! ```
//!
//! ## Client-to-Server Messages (WsClientMessage)
//!
//! ```json
//! {"type": "run", "session_id": "abc123", "message": "Hello", "agent_type": "build", "model": "gpt-4"}
//! {"type": "resume", "session_id": "abc123", "token": "reconnect-token"}
//! {"type": "ping"}
//! {"type": "close"}
//! ```
//!
//! ## Example Client Usage (JavaScript)
//!
//! ```javascript
//! // Connect to WebSocket
//! const ws = new WebSocket('ws://localhost:8080/ws/test-session');
//!
//! ws.onopen = () => {
//!   console.log('Connected');
//!   // Send a message to run
//!   ws.send(JSON.stringify({
//!     type: 'run',
//!     session_id: 'test-session',
//! message: 'Hello, world!'
//!   }));
//! };
//!
//! ws.onmessage = (event) => {
//!   const msg = JSON.parse(event.data);
//!   switch (msg.type) {
//!     case 'connected':
//!       console.log('Session:', msg.session_id);
//!       break;
//!     case 'message':
//!       console.log(`${msg.role}: ${msg.content}`);
//!       break;
//!     case 'tool_call':
//!       console.log('Tool:', msg.tool_name);
//!       break;
//!   }
//! };
//! ```
//!
//! ## Comparison with SSE
//!
//! | Feature | WebSocket | SSE |
//! |---------|-----------|-----|
//! | Direction | Full duplex (bidirectional) | Server-to-client only |
//! | Client-to-server messages | Native | Requires separate HTTP request |
//! | Connection overhead | Single TCP connection | Single HTTP connection |
//! | Browser support | Universal | Modern browsers |
//! | Automatic reconnection | Manual implementation | Built-in |
//! | Binary data | Supported | Text only |
//!
//! ## Heartbeat Mechanism
//!
//! - Server sends ping frames every 30 seconds
//! - Client must respond with pong
//! - If no activity for 120 seconds, connection is terminated
//! - Connection monitor tracks all WebSocket connections

use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};

use actix_web::http::header::{HeaderName, HeaderValue};
use actix_web::{web, Error, HttpRequest, HttpResponse};
use actix_ws::Message;
use futures::StreamExt;
use opencode_core::bus::InternalEvent;
use opencode_core::{Message as CoreMessage, Session};
use serde::{Deserialize, Serialize};
use tokio::sync::mpsc;
use tracing::{debug, error, info, warn};

use crate::routes::error::json_error;
use crate::streaming::conn_state::ConnectionType;
use crate::streaming::heartbeat::HeartbeatManager;
use crate::streaming::{ReplayEntry, StreamMessage};
use crate::ServerState;

/// Path parameters for WebSocket endpoint.
#[derive(Debug, Deserialize)]
pub struct PathParams {
    /// Optional session ID passed in URL path: /ws/{session_id}
    pub session_id: Option<String>,
}

/// Messages sent from WebSocket client to server.
///
/// Clients send these JSON messages over the WebSocket connection to:
/// - Start a new agent run
/// - Resume a session with a reconnection token
/// - Keep the connection alive (ping)
/// - Gracefully close the connection
#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum WsClientMessage {
    /// Start a new agent run in the specified session.
    Run {
        /// The session ID to run in (or create new if not exists)
        session_id: String,
        /// The user message to process
        message: String,
        /// Optional agent type (e.g., "build", "plan", "general")
        agent_type: Option<String>,
        /// Optional model name (e.g., "gpt-4", "claude-3-opus")
        model: Option<String>,
    },
    /// Resume a session after reconnection using a token from previous connection.
    Resume {
        /// The session ID to resume
        session_id: String,
        /// The reconnection token received from x-reconnect-token header
        token: String,
    },
    /// Keepalive ping - server will respond with heartbeat.
    Ping,
    /// Gracefully close the WebSocket connection.
    Close,
}

const WS_HEARTBEAT_INTERVAL: Duration = Duration::from_secs(30);
const WS_CLIENT_TIMEOUT: Duration = Duration::from_secs(120);

pub async fn ws_index(
    state: web::Data<ServerState>,
    req: HttpRequest,
    stream: web::Payload,
    path_params: web::Query<PathParams>,
) -> Result<HttpResponse, Error> {
    let query = parse_query(req.query_string());
    let handshake_session_id = path_params
        .session_id
        .clone()
        .or_else(|| query.get("session_id").cloned())
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

    state
        .connection_monitor
        .register_connection(
            connection_id.clone(),
            ConnectionType::WebSocket,
            handshake_session_id.clone(),
        )
        .await;

    let ws_result = actix_ws::handle(&req, stream);

    let (mut response, mut session, mut msg_stream) = match ws_result {
        Ok(result) => result,
        Err(e) => {
            let err_msg = format!("WebSocket handshake failed: {}", e);
            error!(
                "WS handshake error: connection_id={}, session_id={}, error={}",
                connection_id, handshake_session_id, e
            );
            state
                .connection_monitor
                .connection_failed(&connection_id, &err_msg)
                .await;
            state
                .connection_monitor
                .unregister_connection(&connection_id, "handshake_failed")
                .await;

            return Ok(json_error(
                actix_web::http::StatusCode::BAD_REQUEST,
                "websocket_handshake_failed",
                format!("Failed to establish WebSocket connection: {}", e),
            ));
        }
    };

    if let Ok(header_value) = HeaderValue::from_str(&reconnect_token) {
        response
            .headers_mut()
            .insert(HeaderName::from_static("x-reconnect-token"), header_value);
    }

    let state = state.into_inner();
    let conn_monitor = Arc::clone(&state.connection_monitor);
    let session_hub = Arc::clone(&state.session_hub);
    let conn_id = connection_id.clone();
    let hub_session_id = handshake_session_id.clone();
    let hub_conn_id = connection_id.clone();

    let mut hub_receiver = session_hub
        .register_client(&handshake_session_id, &connection_id)
        .await;

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
                tokio::select! {
                    event = bus_rx.recv() => {
                        match event {
                            Ok(evt) => {
                                if let Some(message) = event_to_stream_message(evt, &session_filter) {
                                    if tx_bus.send(message).await.is_err() {
                                        break;
                                    }
                                }
                            }
                            Err(tokio::sync::broadcast::error::RecvError::Lagged(_)) => continue,
                            Err(tokio::sync::broadcast::error::RecvError::Closed) => break,
                        }
                    }
                    hub_msg = hub_receiver.recv() => {
                        match hub_msg {
                            Ok(msg) => {
                                if tx_bus.send(msg).await.is_err() {
                                    break;
                                }
                            }
                            Err(tokio::sync::broadcast::error::RecvError::Lagged(_)) => continue,
                            Err(tokio::sync::broadcast::error::RecvError::Closed) => break,
                        }
                    }
                }
            }
        });

        loop {
            if last_heartbeat.elapsed() > WS_CLIENT_TIMEOUT {
                warn!("WebSocket heartbeat timeout");
                conn_monitor
                    .unregister_connection(&conn_id_for_task, "heartbeat_timeout")
                    .await;
                session_hub
                    .unregister_client(&hub_session_id, &hub_conn_id)
                    .await;
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
                            session_hub.unregister_client(&hub_session_id, &hub_conn_id).await;
                            break;
                        }
                    } else {
                        let fallback = StreamMessage::Error {
                            session_id: Some(handshake_session_id.clone()),
                            error: "serialization_error".to_string(),
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
                                session_hub.unregister_client(&hub_session_id, &hub_conn_id).await;
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
                            session_hub.unregister_client(&hub_session_id, &hub_conn_id).await;
                            let _ = session.close(reason).await;
                            break;
                        }
                        Message::Binary(_) => {
                            let _ = tx.send(StreamMessage::Error {
                                session_id: Some(handshake_session_id.clone()),
                                error: "unsupported_binary".to_string(),
                                code: "unsupported_binary".to_string(),
                                message: "binary websocket messages are not supported".to_string(),
                            }).await;
                        }
                        _ => {
                            conn_monitor.unregister_connection(&conn_id_for_task, "unknown_message").await;
                            session_hub.unregister_client(&hub_session_id, &hub_conn_id).await;
                            break;
                        }
                    }
                }
                else => {
                    conn_monitor.unregister_connection(&conn_id_for_task, "channel_closed").await;
                    session_hub.unregister_client(&hub_session_id, &hub_conn_id).await;
                    break;
                },
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
                            error: "invalid_reconnect_token".to_string(),
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
                            error: "session_load_error".to_string(),
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
                    error: "parse_error".to_string(),
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
    cfg.route("/{session_id}", web::get().to(ws_index));
    cfg.route("", web::get().to(ws_index));
}

pub mod session_hub;

pub use session_hub::{ClientInfo, SessionClients, SessionHub};

#[cfg(test)]
mod ws_lifecycle_tests {
    use super::parse_query;
    use super::session_hub::SessionHub;
    use super::WsClientMessage;
    use crate::streaming::StreamMessage;

    #[test]
    fn test_ws_client_message_run_deserialization() {
        let json = r#"{"type": "run", "session_id": "sess-123", "message": "Hello", "agent_type": "build", "model": "gpt-4"}"#;
        let msg: WsClientMessage = serde_json::from_str(json).unwrap();
        match msg {
            WsClientMessage::Run {
                session_id,
                message,
                agent_type,
                model,
            } => {
                assert_eq!(session_id, "sess-123");
                assert_eq!(message, "Hello");
                assert_eq!(agent_type, Some("build".to_string()));
                assert_eq!(model, Some("gpt-4".to_string()));
            }
            _ => panic!("expected Run variant"),
        }
    }

    #[test]
    fn test_ws_client_message_run_minimal() {
        let json = r#"{"type": "run", "session_id": "sess-123", "message": "Hello"}"#;
        let msg: WsClientMessage = serde_json::from_str(json).unwrap();
        match msg {
            WsClientMessage::Run {
                session_id,
                message,
                agent_type,
                model,
            } => {
                assert_eq!(session_id, "sess-123");
                assert_eq!(message, "Hello");
                assert!(agent_type.is_none());
                assert!(model.is_none());
            }
            _ => panic!("expected Run variant"),
        }
    }

    #[test]
    fn test_ws_client_message_resume_deserialization() {
        let json = r#"{"type": "resume", "session_id": "sess-123", "token": "abc-token"}"#;
        let msg: WsClientMessage = serde_json::from_str(json).unwrap();
        match msg {
            WsClientMessage::Resume { session_id, token } => {
                assert_eq!(session_id, "sess-123");
                assert_eq!(token, "abc-token");
            }
            _ => panic!("expected Resume variant"),
        }
    }

    #[test]
    fn test_ws_client_message_ping_deserialization() {
        let json = r#"{"type": "ping"}"#;
        let msg: WsClientMessage = serde_json::from_str(json).unwrap();
        match msg {
            WsClientMessage::Ping => {}
            _ => panic!("expected Ping variant"),
        }
    }

    #[test]
    fn test_ws_client_message_close_deserialization() {
        let json = r#"{"type": "close"}"#;
        let msg: WsClientMessage = serde_json::from_str(json).unwrap();
        match msg {
            WsClientMessage::Close => {}
            _ => panic!("expected Close variant"),
        }
    }

    #[test]
    fn test_ws_client_message_serialization_roundtrip() {
        let msg = WsClientMessage::Run {
            session_id: "test-session".to_string(),
            message: "test message".to_string(),
            agent_type: Some("general".to_string()),
            model: Some("claude-3".to_string()),
        };
        let json = serde_json::to_string(&msg).unwrap();
        let parsed: WsClientMessage = serde_json::from_str(&json).unwrap();
        match parsed {
            WsClientMessage::Run {
                session_id,
                message,
                agent_type,
                model,
            } => {
                assert_eq!(session_id, "test-session");
                assert_eq!(message, "test message");
                assert_eq!(agent_type, Some("general".to_string()));
                assert_eq!(model, Some("claude-3".to_string()));
            }
            _ => panic!("expected Run variant"),
        }
    }

    #[tokio::test]
    async fn test_ws_lifecycle_connection_setup_and_teardown() {
        let hub = SessionHub::new(256);

        let session_id = "test-session-lifecycle";
        let client_id = "test-client-lifecycle";

        assert_eq!(hub.get_session_client_count(session_id).await, 0);
        assert_eq!(hub.session_count().await, 0);
        assert_eq!(hub.total_client_count().await, 0);

        let _receiver = hub.register_client(session_id, client_id).await;

        assert_eq!(hub.get_session_client_count(session_id).await, 1);
        assert_eq!(hub.session_count().await, 1);
        assert_eq!(hub.total_client_count().await, 1);

        hub.unregister_client(session_id, client_id).await;

        assert_eq!(hub.get_session_client_count(session_id).await, 0);
        assert_eq!(hub.session_count().await, 0);
        assert_eq!(hub.total_client_count().await, 0);
    }

    #[tokio::test]
    async fn test_ws_lifecycle_multiple_clients_setup_teardown() {
        let hub = SessionHub::new(256);

        let session_id = "test-session-multi";

        let _receiver1 = hub.register_client(session_id, "client-1").await;
        let _receiver2 = hub.register_client(session_id, "client-2").await;
        let _receiver3 = hub.register_client(session_id, "client-3").await;

        assert_eq!(hub.get_session_client_count(session_id).await, 3);
        assert_eq!(hub.total_client_count().await, 3);

        hub.unregister_client(session_id, "client-1").await;
        assert_eq!(hub.get_session_client_count(session_id).await, 2);
        assert_eq!(hub.total_client_count().await, 2);

        hub.unregister_client(session_id, "client-2").await;
        assert_eq!(hub.get_session_client_count(session_id).await, 1);
        assert_eq!(hub.total_client_count().await, 1);

        hub.unregister_client(session_id, "client-3").await;
        assert_eq!(hub.get_session_client_count(session_id).await, 0);
        assert_eq!(hub.session_count().await, 0);
        assert_eq!(hub.total_client_count().await, 0);
    }

    #[tokio::test]
    async fn test_ws_lifecycle_broadcast_after_cleanup() {
        let hub = SessionHub::new(256);

        let session_id = "broadcast-cleanup-test";

        let mut receiver1 = hub.register_client(session_id, "client-1").await;
        let mut receiver2 = hub.register_client(session_id, "client-2").await;

        hub.unregister_client(session_id, "client-1").await;

        let message = StreamMessage::Message {
            session_id: session_id.to_string(),
            content: "Hello remaining client!".to_string(),
            role: "assistant".to_string(),
        };

        hub.broadcast(session_id, message).await;

        let msg2 = receiver2
            .recv()
            .await
            .expect("client-2 should receive message");
        match msg2 {
            StreamMessage::Message { content, .. } => {
                assert_eq!(content, "Hello remaining client!");
            }
            _ => panic!("expected Message variant"),
        }

        let err = receiver1.try_recv();
        assert!(err.is_err(), "client-1 should not receive after unregister");
    }

    #[tokio::test]
    async fn test_ws_lifecycle_error_graceful_handling() {
        let hub = SessionHub::new(256);

        let session_id = "error-handling-session";
        let client_id = "error-client";

        let _receiver = hub.register_client(session_id, client_id).await;

        assert_eq!(hub.get_session_client_count(session_id).await, 1);

        hub.unregister_client(session_id, client_id).await;

        assert_eq!(hub.get_session_client_count(session_id).await, 0);
        assert_eq!(hub.session_count().await, 0);

        hub.unregister_client(session_id, client_id).await;

        assert_eq!(hub.get_session_client_count(session_id).await, 0);
        assert_eq!(hub.session_count().await, 0);
    }

    #[tokio::test]
    async fn test_ws_lifecycle_multiple_sessions_independent() {
        let hub = SessionHub::new(256);

        let _r1 = hub.register_client("session-a", "client-1").await;
        let _r2 = hub.register_client("session-b", "client-2").await;
        let _r3 = hub.register_client("session-a", "client-3").await;

        assert_eq!(hub.session_count().await, 2);
        assert_eq!(hub.get_session_client_count("session-a").await, 2);
        assert_eq!(hub.get_session_client_count("session-b").await, 1);
        assert_eq!(hub.total_client_count().await, 3);

        hub.unregister_client("session-a", "client-1").await;
        assert_eq!(hub.session_count().await, 2);
        assert_eq!(hub.get_session_client_count("session-a").await, 1);
        assert_eq!(hub.get_session_client_count("session-b").await, 1);

        hub.unregister_client("session-b", "client-2").await;
        assert_eq!(hub.session_count().await, 1);
        assert_eq!(hub.get_session_client_count("session-b").await, 0);
        assert_eq!(hub.get_session_client_count("session-a").await, 1);
    }

    #[tokio::test]
    async fn test_ws_lifecycle_broadcast_all() {
        let hub = SessionHub::new(256);

        let mut r1 = hub.register_client("session-1", "client-1").await;
        let mut r2 = hub.register_client("session-2", "client-2").await;
        let mut r3 = hub.register_client("session-3", "client-3").await;

        let message = StreamMessage::SessionUpdate {
            session_id: "all".to_string(),
            status: "broadcast_all".to_string(),
        };

        hub.broadcast_all(message).await;

        let msg1 = r1.recv().await.expect("client-1 should receive broadcast");
        let msg2 = r2.recv().await.expect("client-2 should receive broadcast");
        let msg3 = r3.recv().await.expect("client-3 should receive broadcast");

        match (&msg1, &msg2, &msg3) {
            (
                StreamMessage::SessionUpdate { status: s1, .. },
                StreamMessage::SessionUpdate { status: s2, .. },
                StreamMessage::SessionUpdate { status: s3, .. },
            ) => {
                assert_eq!(s1, "broadcast_all");
                assert_eq!(s2, "broadcast_all");
                assert_eq!(s3, "broadcast_all");
            }
            _ => panic!("expected SessionUpdate variant"),
        }
    }

    #[tokio::test]
    async fn test_disconnect_removes_client_from_hub() {
        let hub = SessionHub::new(256);

        let session_id = "disconnect-test-session";
        let client_id = "disconnect-test-client";

        let _receiver = hub.register_client(session_id, client_id).await;

        assert_eq!(hub.get_session_client_count(session_id).await, 1);
        assert_eq!(hub.total_client_count().await, 1);
        assert_eq!(hub.session_count().await, 1);

        hub.unregister_client(session_id, client_id).await;

        assert_eq!(hub.get_session_client_count(session_id).await, 0);
        assert_eq!(hub.total_client_count().await, 0);
        assert_eq!(hub.session_count().await, 0);
    }

    #[tokio::test]
    async fn test_disconnect_remaining_clients_still_receive_events() {
        let hub = SessionHub::new(256);

        let session_id = "disconnect-regression-session";

        let mut receiver1 = hub.register_client(session_id, "client-1").await;
        let mut receiver2 = hub.register_client(session_id, "client-2").await;
        let mut receiver3 = hub.register_client(session_id, "client-3").await;

        assert_eq!(hub.get_session_client_count(session_id).await, 3);

        hub.unregister_client(session_id, "client-1").await;
        assert_eq!(hub.get_session_client_count(session_id).await, 2);

        let broadcast_msg = StreamMessage::Message {
            session_id: session_id.to_string(),
            content: "Event after disconnect".to_string(),
            role: "assistant".to_string(),
        };
        hub.broadcast(session_id, broadcast_msg).await;

        let msg2 = receiver2
            .recv()
            .await
            .expect("client-2 should still receive events after client-1 disconnect");
        let msg3 = receiver3
            .recv()
            .await
            .expect("client-3 should still receive events after client-1 disconnect");

        match (&msg2, &msg3) {
            (
                StreamMessage::Message { content: c2, .. },
                StreamMessage::Message { content: c3, .. },
            ) => {
                assert_eq!(c2, "Event after disconnect");
                assert_eq!(c3, "Event after disconnect");
            }
            _ => panic!("expected Message variant"),
        }

        let err1 = receiver1.try_recv();
        assert!(
            err1.is_err(),
            "disconnected client-1 should not receive events"
        );
    }

    #[tokio::test]
    async fn test_disconnect_last_client_session_removed() {
        let hub = SessionHub::new(256);

        let session_id = "last-client-disconnect";

        let receiver = hub.register_client(session_id, "only-client").await;
        assert_eq!(hub.session_count().await, 1);
        assert_eq!(hub.get_session_client_count(session_id).await, 1);

        drop(receiver);
        hub.unregister_client(session_id, "only-client").await;

        assert_eq!(hub.session_count().await, 0);
        assert_eq!(hub.get_session_client_count(session_id).await, 0);
    }

    #[tokio::test]
    async fn test_disconnect_multiple_sessions_independent() {
        let hub = SessionHub::new(256);

        let _r1 = hub.register_client("sess-A", "client-A1").await;
        let mut r2 = hub.register_client("sess-A", "client-A2").await;
        let mut r3 = hub.register_client("sess-B", "client-B1").await;

        hub.unregister_client("sess-A", "client-A1").await;

        let msg = StreamMessage::SessionUpdate {
            session_id: "sess-A".to_string(),
            status: "after_disconnect".to_string(),
        };
        hub.broadcast("sess-A", msg.clone()).await;
        hub.broadcast("sess-B", msg.clone()).await;

        let msg_a2 = r2.recv().await.expect("sess-A client should receive");
        match msg_a2 {
            StreamMessage::SessionUpdate { status, .. } => {
                assert_eq!(status, "after_disconnect");
            }
            _ => panic!("expected SessionUpdate"),
        }

        let msg_b1 = r3.recv().await.expect("sess-B client should receive");
        match msg_b1 {
            StreamMessage::SessionUpdate { status, .. } => {
                assert_eq!(status, "after_disconnect");
            }
            _ => panic!("expected SessionUpdate"),
        }

        let err = r2.try_recv();
        assert!(err.is_err(), "disconnected client-A1 should not receive");
    }

    #[test]
    fn test_ws_parse_query_empty() {
        let query = "";
        let params = parse_query(query);
        assert_eq!(params.len(), 1);
        assert!(params.contains_key(""));
    }

    #[test]
    fn test_ws_parse_query_single_param() {
        let query = "client_id=editor1";
        let params = parse_query(query);
        assert_eq!(params.get("client_id"), Some(&"editor1".to_string()));
    }

    #[test]
    fn test_ws_parse_query_multiple_params() {
        let query = "client_id=editor1&session_id=abc123";
        let params = parse_query(query);
        assert_eq!(params.get("client_id"), Some(&"editor1".to_string()));
        assert_eq!(params.get("session_id"), Some(&"abc123".to_string()));
    }

    #[test]
    fn test_ws_parse_query_with_empty_value() {
        let query = "key1=value1&key2=";
        let params = parse_query(query);
        assert_eq!(params.get("key1"), Some(&"value1".to_string()));
        assert_eq!(params.get("key2"), Some(&"".to_string()));
    }

    #[test]
    fn test_ws_parse_query_no_value() {
        let query = "keyonly";
        let params = parse_query(query);
        assert_eq!(params.get("keyonly"), Some(&"".to_string()));
    }

    #[test]
    fn test_ws_parse_query_with_multiple_equals() {
        let query = "key=value=with=equals";
        let params = parse_query(query);
        assert_eq!(params.get("key"), Some(&"value=with=equals".to_string()));
    }

    #[test]
    fn test_ws_parse_query_url_encoded() {
        let query = "session_id=abc%40123&token=xyz%26abc";
        let params = parse_query(query);
        assert_eq!(params.get("session_id"), Some(&"abc%40123".to_string()));
        assert_eq!(params.get("token"), Some(&"xyz%26abc".to_string()));
    }

    #[test]
    fn test_ws_parse_query_special_chars() {
        let query = "key1=hello&key2=world-test_pets.123";
        let params = parse_query(query);
        assert_eq!(params.get("key1"), Some(&"hello".to_string()));
        assert_eq!(params.get("key2"), Some(&"world-test_pets.123".to_string()));
    }

    fn event_to_stream_message(
        event: opencode_core::bus::InternalEvent,
        session_id: &str,
    ) -> Option<crate::streaming::StreamMessage> {
        let candidate = crate::streaming::StreamMessage::from_internal_event(&event)?;
        match candidate.session_id() {
            Some(source_session) if source_session == session_id => Some(candidate),
            Some(_) => None,
            None => Some(candidate),
        }
    }

    #[test]
    fn test_ws_event_to_stream_message_filters_by_session() {
        let event = opencode_core::bus::InternalEvent::SessionStarted("other-session".to_string());
        let result = event_to_stream_message(event, "my-session");
        assert!(result.is_none());
    }

    #[test]
    fn test_ws_event_to_stream_message_passes_when_session_matches() {
        let event = opencode_core::bus::InternalEvent::SessionStarted("my-session".to_string());
        let result = event_to_stream_message(event, "my-session");
        assert!(result.is_some());
    }

    #[test]
    fn test_ws_event_to_stream_message_handles_error_without_session() {
        let event = opencode_core::bus::InternalEvent::Error {
            source: "test".to_string(),
            message: "error".to_string(),
        };
        let result = event_to_stream_message(event, "any-session");
        assert!(result.is_some());
    }

    #[test]
    fn test_ws_event_to_stream_message_message_added() {
        let event = opencode_core::bus::InternalEvent::MessageAdded {
            session_id: "my-session".to_string(),
            message_id: "msg-123".to_string(),
        };
        let result = event_to_stream_message(event, "my-session");
        assert!(result.is_some());
    }

    #[test]
    fn test_ws_event_to_stream_message_session_ended() {
        let event = opencode_core::bus::InternalEvent::SessionEnded("my-session".to_string());
        let result = event_to_stream_message(event, "my-session");
        assert!(result.is_some());
    }

    #[test]
    fn test_ws_client_message_all_variants_serializable() {
        let variants = [
            WsClientMessage::Run {
                session_id: "test-session".to_string(),
                message: "test message".to_string(),
                agent_type: Some("build".to_string()),
                model: Some("gpt-4".to_string()),
            },
            WsClientMessage::Resume {
                session_id: "test-session".to_string(),
                token: "abc123".to_string(),
            },
            WsClientMessage::Ping,
            WsClientMessage::Close,
        ];

        for variant in variants {
            let json = serde_json::to_string(&variant).expect("should serialize");
            let parsed: WsClientMessage = serde_json::from_str(&json).expect("should deserialize");
            match (&variant, &parsed) {
                (
                    WsClientMessage::Run { session_id: s1, .. },
                    WsClientMessage::Run { session_id: s2, .. },
                ) => {
                    assert_eq!(s1, s2);
                }
                (
                    WsClientMessage::Resume {
                        session_id: s1,
                        token: t1,
                        ..
                    },
                    WsClientMessage::Resume {
                        session_id: s2,
                        token: t2,
                        ..
                    },
                ) => {
                    assert_eq!(s1, s2);
                    assert_eq!(t1, t2);
                }
                (WsClientMessage::Ping, WsClientMessage::Ping) => {}
                (WsClientMessage::Close, WsClientMessage::Close) => {}
                _ => panic!("variant mismatch"),
            }
        }
    }

    #[tokio::test]
    async fn test_ws_session_hub_register_and_broadcast() {
        use crate::streaming::StreamMessage;

        let hub = SessionHub::new(256);
        let session_id = "broadcast-test-ws";
        let client1 = "client-ws-1";
        let client2 = "client-ws-2";

        let mut receiver1 = hub.register_client(session_id, client1).await;
        let mut receiver2 = hub.register_client(session_id, client2).await;

        let msg = StreamMessage::Message {
            session_id: session_id.to_string(),
            content: "Hello from WS!".to_string(),
            role: "assistant".to_string(),
        };

        hub.broadcast(session_id, msg).await;

        let received1 = receiver1.recv().await.expect("client1 should receive");
        let received2 = receiver2.recv().await.expect("client2 should receive");

        match (&received1, &received2) {
            (
                StreamMessage::Message { content: c1, .. },
                StreamMessage::Message { content: c2, .. },
            ) => {
                assert_eq!(c1, "Hello from WS!");
                assert_eq!(c2, "Hello from WS!");
            }
            _ => panic!("Expected Message variant"),
        }
    }

    #[tokio::test]
    async fn test_ws_session_hub_broadcast_all() {
        use crate::streaming::StreamMessage;

        let hub = SessionHub::new(256);

        let mut r1 = hub.register_client("session-1", "client-1").await;
        let mut r2 = hub.register_client("session-2", "client-2").await;
        let mut r3 = hub.register_client("session-3", "client-3").await;

        let msg = StreamMessage::SessionUpdate {
            session_id: "all".to_string(),
            status: "test_broadcast_all".to_string(),
        };

        hub.broadcast_all(msg).await;

        let msg1 = r1.recv().await.expect("client-1 should receive broadcast");
        let msg2 = r2.recv().await.expect("client-2 should receive broadcast");
        let msg3 = r3.recv().await.expect("client-3 should receive broadcast");

        match (&msg1, &msg2, &msg3) {
            (
                StreamMessage::SessionUpdate { status: s1, .. },
                StreamMessage::SessionUpdate { status: s2, .. },
                StreamMessage::SessionUpdate { status: s3, .. },
            ) => {
                assert_eq!(s1, "test_broadcast_all");
                assert_eq!(s2, "test_broadcast_all");
                assert_eq!(s3, "test_broadcast_all");
            }
            _ => panic!("expected SessionUpdate variant"),
        }
    }
}
