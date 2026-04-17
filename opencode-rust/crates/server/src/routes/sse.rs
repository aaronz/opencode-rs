use std::sync::Arc;

use actix_web::{web, Error, HttpRequest, HttpResponse};
use futures::stream::{self, Stream};

use crate::routes::error::ErrorResponse;
use opencode_core::bus::InternalEvent;
use opencode_core::{Message as CoreMessage, Session};
use serde::Deserialize;
use tokio::sync::mpsc;
use tracing::{debug, info, warn};

use crate::streaming::conn_state::ConnectionType;
use crate::streaming::heartbeat::HeartbeatManager;
use crate::streaming::{ReplayEntry, StreamMessage};
use crate::ServerState;

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

    state
        .connection_monitor
        .register_connection(
            connection_id.clone(),
            ConnectionType::Sse,
            session_id.clone(),
        )
        .await;

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

        connection_monitor
            .heartbeat_success(&conn_id_bootstrap)
            .await;

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
    stream::unfold(
        (
            rx,
            state,
            session_id,
            heartbeat_handle,
            conn_monitor_cleanup,
            conn_id_cleanup,
        ),
        |(mut rx, state, session_id, heartbeat_handle, conn_monitor, conn_id)| async move {
            match rx.recv().await {
                Some(outbound) => {
                    let event_id = if outbound.record {
                        outbound
                            .message
                            .session_id()
                            .filter(|sid| *sid == session_id.as_str())
                            .map(|sid| {
                                let seq = state
                                    .reconnection_store
                                    .record_message(sid, outbound.message.clone());
                                debug!(
                                    "SSE recorded message: session_id={}, sequence={}",
                                    sid, seq
                                );
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
                        format!(
                            "id: {id}\nevent: {}\ndata: {payload}\n\n",
                            message_event_type(&outbound.message)
                        )
                    } else {
                        format!("data: {payload}\n\n")
                    };

                    Some((
                        Ok::<_, Error>(web::Bytes::from(formatted)),
                        (
                            rx,
                            state,
                            session_id,
                            heartbeat_handle,
                            conn_monitor,
                            conn_id,
                        ),
                    ))
                }
                None => {
                    heartbeat_handle.abort();
                    conn_monitor
                        .unregister_connection(&conn_id, "stream_ended")
                        .await;
                    None
                }
            }
        },
    )
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
            return Ok(ErrorResponse::storage_error(format!(
                "Failed to load session {}: {}",
                session_id, e
            ))
            .to_response(actix_web::http::StatusCode::INTERNAL_SERVER_ERROR));
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sse_query_deserialization_default_session() {
        let json = r#"{}"#;
        let query: SseQuery = serde_json::from_str(json).unwrap();
        assert!(query.session_id.is_none());
        assert!(query.reconnect_token.is_none());
    }

    #[test]
    fn test_sse_query_deserialization_with_session_id() {
        let json = r#"{"session_id": "my-session-123"}"#;
        let query: SseQuery = serde_json::from_str(json).unwrap();
        assert_eq!(query.session_id, Some("my-session-123".to_string()));
    }

    #[test]
    fn test_sse_query_deserialization_with_reconnect_token() {
        let json = r#"{"session_id": "sess", "reconnect_token": "token-abc"}"#;
        let query: SseQuery = serde_json::from_str(json).unwrap();
        assert_eq!(query.session_id, Some("sess".to_string()));
        assert_eq!(query.reconnect_token, Some("token-abc".to_string()));
    }

    #[test]
    fn test_sse_message_request_deserialization() {
        let json = r#"{"message": "Hello world", "model": "gpt-4"}"#;
        let req: SseMessageRequest = serde_json::from_str(json).unwrap();
        assert_eq!(req.message, "Hello world");
        assert_eq!(req.model, Some("gpt-4".to_string()));
    }

    #[test]
    fn test_sse_message_request_minimal() {
        let json = r#"{"message": "Just a message"}"#;
        let req: SseMessageRequest = serde_json::from_str(json).unwrap();
        assert_eq!(req.message, "Just a message");
        assert!(req.model.is_none());
    }

    #[test]
    fn test_message_event_type_message() {
        let msg = StreamMessage::Message {
            session_id: "test".to_string(),
            content: "hello".to_string(),
            role: "user".to_string(),
        };
        assert_eq!(message_event_type(&msg), "message");
    }

    #[test]
    fn test_message_event_type_session_update() {
        let msg = StreamMessage::SessionUpdate {
            session_id: "test".to_string(),
            status: "running".to_string(),
        };
        assert_eq!(message_event_type(&msg), "session_update");
    }

    #[test]
    fn test_message_event_type_connected() {
        let msg = StreamMessage::Connected {
            session_id: Some("test".to_string()),
        };
        assert_eq!(message_event_type(&msg), "connected");
    }

    #[test]
    fn test_sse_message_request_unicode() {
        let json = r#"{"message": "Hello 世界 🌍"}"#;
        let req: SseMessageRequest = serde_json::from_str(json).unwrap();
        assert!(req.message.contains("世界"));
    }

    #[test]
    fn test_sse_query_preserves_special_characters() {
        let json = r#"{"session_id": "sess/with/slashes", "reconnect_token": "token+with+plus"}"#;
        let query: SseQuery = serde_json::from_str(json).unwrap();
        assert_eq!(query.session_id.unwrap(), "sess/with/slashes");
        assert_eq!(query.reconnect_token.unwrap(), "token+with+plus");
    }

    #[test]
    fn test_event_to_stream_message_filters_by_session() {
        let event = InternalEvent::SessionStarted("other-session".to_string());
        let result = event_to_stream_message(event, "my-session");
        assert!(result.is_none());
    }

    #[test]
    fn test_event_to_stream_message_passes_through_when_session_matches() {
        let event = InternalEvent::SessionStarted("my-session".to_string());
        let result = event_to_stream_message(event, "my-session");
        assert!(result.is_some());
        match result.unwrap() {
            StreamMessage::SessionUpdate { session_id, status } => {
                assert_eq!(session_id, "my-session");
                assert_eq!(status, "started");
            }
            _ => panic!("Expected SessionUpdate"),
        }
    }

    #[test]
    fn test_event_to_stream_message_handles_message_added() {
        let event = InternalEvent::MessageAdded {
            session_id: "my-session".to_string(),
            message_id: "msg-123".to_string(),
        };
        let result = event_to_stream_message(event, "my-session");
        assert!(result.is_some());
    }

    #[test]
    fn test_event_to_stream_message_with_no_session_id_in_event() {
        let event = InternalEvent::Error {
            source: "test".to_string(),
            message: "error occurred".to_string(),
        };
        let result = event_to_stream_message(event, "any-session");
        assert!(result.is_some());
    }

    #[test]
    fn test_message_event_type_tool_call() {
        let msg = StreamMessage::ToolCall {
            session_id: "test".to_string(),
            tool_name: "read".to_string(),
            args: serde_json::json!({}),
            call_id: "call-1".to_string(),
        };
        assert_eq!(message_event_type(&msg), "tool_call");
    }

    #[test]
    fn test_message_event_type_tool_result() {
        let msg = StreamMessage::ToolResult {
            session_id: "test".to_string(),
            call_id: "call-1".to_string(),
            output: "result".to_string(),
            success: true,
        };
        assert_eq!(message_event_type(&msg), "tool_result");
    }

    #[test]
    fn test_message_event_type_heartbeat() {
        let msg = StreamMessage::Heartbeat { timestamp: 123 };
        assert_eq!(message_event_type(&msg), "heartbeat");
    }

    #[test]
    fn test_message_event_type_error() {
        let msg = StreamMessage::Error {
            session_id: Some("test".to_string()),
            error: "ERR".to_string(),
            code: "ERR".to_string(),
            message: "error".to_string(),
        };
        assert_eq!(message_event_type(&msg), "error");
    }

    #[test]
    fn test_event_to_stream_message_session_ended() {
        let event = InternalEvent::SessionEnded("my-session".to_string());
        let result = event_to_stream_message(event, "my-session");
        assert!(result.is_some());
    }

    #[test]
    fn test_sse_message_request_with_empty_model() {
        let json = r#"{"message": "test", "model": ""}"#;
        let req: SseMessageRequest = serde_json::from_str(json).unwrap();
        assert_eq!(req.model, Some("".to_string()));
    }

    #[test]
    fn test_sse_message_request_long_content() {
        let long_message = "a".repeat(10000);
        let json = format!(r#"{{"message": "{}"}}"#, long_message);
        let req: SseMessageRequest = serde_json::from_str(&json).unwrap();
        assert_eq!(req.message.len(), 10000);
    }

    #[test]
    fn test_sse_query_with_empty_session_id() {
        let json = r#"{"session_id": ""}"#;
        let query: SseQuery = serde_json::from_str(json).unwrap();
        assert_eq!(query.session_id, Some("".to_string()));
    }

    #[test]
    fn test_sse_query_with_numeric_session_id() {
        let json = r#"{"session_id": "12345"}"#;
        let query: SseQuery = serde_json::from_str(json).unwrap();
        assert_eq!(query.session_id, Some("12345".to_string()));
    }

    #[test]
    fn test_sse_message_request_special_characters() {
        let json = r#"{"message": "Hello\nWorld\t!@#$%^&*()"}"#;
        let req: SseMessageRequest = serde_json::from_str(json).unwrap();
        assert!(req.message.contains('\n'));
        assert!(req.message.contains('\t'));
        assert!(req.message.contains("!@#$%^&*()"));
    }

    #[test]
    fn test_message_event_type_all_variants() {
        let test_cases = [
            (
                StreamMessage::Message {
                    session_id: "test".to_string(),
                    content: "hi".to_string(),
                    role: "user".to_string(),
                },
                "message",
            ),
            (
                StreamMessage::ToolCall {
                    session_id: "test".to_string(),
                    tool_name: "read".to_string(),
                    args: serde_json::json!({}),
                    call_id: "c1".to_string(),
                },
                "tool_call",
            ),
            (
                StreamMessage::ToolResult {
                    session_id: "test".to_string(),
                    call_id: "c1".to_string(),
                    output: "ok".to_string(),
                    success: true,
                },
                "tool_result",
            ),
            (
                StreamMessage::SessionUpdate {
                    session_id: "test".to_string(),
                    status: "done".to_string(),
                },
                "session_update",
            ),
            (StreamMessage::Heartbeat { timestamp: 123 }, "heartbeat"),
            (
                StreamMessage::Error {
                    session_id: None,
                    error: "err".to_string(),
                    code: "ERR".to_string(),
                    message: "error".to_string(),
                },
                "error",
            ),
            (
                StreamMessage::Connected {
                    session_id: Some("test".to_string()),
                },
                "connected",
            ),
        ];

        for (msg, expected_type) in test_cases {
            assert_eq!(
                message_event_type(&msg),
                expected_type,
                "type mismatch for {:?}",
                msg
            );
        }
    }

    #[test]
    fn test_event_to_stream_message_all_internal_event_types() {
        use opencode_core::bus::InternalEvent;

        let test_cases = vec![
            (InternalEvent::SessionStarted("sess".to_string()), true),
            (InternalEvent::SessionEnded("sess".to_string()), true),
            (
                InternalEvent::MessageAdded {
                    session_id: "sess".to_string(),
                    message_id: "msg".to_string(),
                },
                true,
            ),
            (
                InternalEvent::MessageUpdated {
                    session_id: "sess".to_string(),
                    message_id: "msg".to_string(),
                },
                true,
            ),
            (
                InternalEvent::ToolCallStarted {
                    session_id: "sess".to_string(),
                    tool_name: "read".to_string(),
                    call_id: "c1".to_string(),
                },
                true,
            ),
            (
                InternalEvent::ToolCallEnded {
                    session_id: "sess".to_string(),
                    call_id: "c1".to_string(),
                    success: true,
                },
                true,
            ),
            (
                InternalEvent::ToolCallOutput {
                    session_id: "sess".to_string(),
                    call_id: "c1".to_string(),
                    output: "out".to_string(),
                },
                true,
            ),
            (
                InternalEvent::AgentStatusChanged {
                    session_id: "sess".to_string(),
                    status: "running".to_string(),
                },
                true,
            ),
            (
                InternalEvent::Error {
                    source: "src".to_string(),
                    message: "msg".to_string(),
                },
                true,
            ),
            (
                InternalEvent::AgentStarted {
                    session_id: "sess".to_string(),
                    agent: "agent".to_string(),
                },
                false,
            ),
        ];

        for (event, should_convert) in test_cases {
            let result = event_to_stream_message(event.clone(), "sess");
            if should_convert {
                assert!(result.is_some(), "Expected Some for {:?}", event);
            }
        }
    }
}
