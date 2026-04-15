use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};

use actix_web::http::header::{HeaderName, HeaderValue};
use actix_web::{web, Error, HttpRequest, HttpResponse};
use actix_ws::Message;
use futures::StreamExt;
use opencode_core::bus::InternalEvent;
use opencode_core::{Message as CoreMessage, Session};
use serde::Deserialize;
use tokio::sync::mpsc;
use tracing::{debug, error, info, warn};

use crate::routes::error::json_error;
use crate::streaming::conn_state::ConnectionType;
use crate::streaming::heartbeat::HeartbeatManager;
use crate::streaming::{ReplayEntry, StreamMessage};
use crate::ServerState;

#[derive(Debug, Deserialize)]
pub struct PathParams {
    pub session_id: Option<String>,
}

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
    use super::session_hub::SessionHub;
    use crate::streaming::StreamMessage;

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
}
