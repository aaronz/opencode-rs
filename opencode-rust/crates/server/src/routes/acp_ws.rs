use std::collections::HashMap;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};

use actix_web::http::header::{HeaderName, HeaderValue};
use actix_web::{web, Error, HttpRequest, HttpResponse};
use actix_ws::Message;
use futures::StreamExt;
use opencode_control_plane::{AcpAgentEvent, AcpEventType, SharedAcpStream};
use opencode_core::acp::{AcpHandshakeAck, AcpHandshakeRequest, AcpMessage, AcpProtocol};
use serde::{Deserialize, Serialize};
use tokio::sync::mpsc;
use tokio::sync::RwLock;
use tracing::{debug, error, info, warn};

use crate::routes::error::json_error;
use crate::streaming::conn_state::{ConnectionMonitor, ConnectionType};
use crate::streaming::StreamMessage;
use crate::ServerState;

const ACP_WS_HEARTBEAT_INTERVAL: Duration = Duration::from_secs(30);
const ACP_WS_CLIENT_TIMEOUT: Duration = Duration::from_secs(120);

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum AcpWsMessage {
    Handshake {
        version: String,
        client_id: String,
        capabilities: Vec<String>,
    },
    HandshakeAck {
        session_id: String,
        confirmed: bool,
    },
    EditorMessage {
        session_id: String,
        content: String,
    },
    ToolCall {
        session_id: String,
        tool_name: String,
        args: serde_json::Value,
        call_id: String,
    },
    ToolResult {
        session_id: String,
        call_id: String,
        output: String,
        success: bool,
    },
    Status {
        status: String,
    },
    Ping,
    Pong,
    Close,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum AcpWsOutgoing {
    HandshakeResponse {
        version: String,
        server_id: String,
        session_id: String,
        accepted: bool,
        error: Option<String>,
    },
    SessionMessage {
        session_id: String,
        content: String,
        role: String,
    },
    ToolCall {
        session_id: String,
        tool_name: String,
        args: serde_json::Value,
        call_id: String,
    },
    ToolResult {
        session_id: String,
        call_id: String,
        output: String,
        success: bool,
    },
    StatusUpdate {
        session_id: String,
        status: String,
    },
    Heartbeat {
        timestamp: i64,
    },
    Error {
        code: String,
        message: String,
    },
    Connected {
        session_id: Option<String>,
    },
}

#[allow(dead_code)]
impl AcpWsOutgoing {
    fn session_id(&self) -> Option<&str> {
        match self {
            Self::SessionMessage { session_id, .. } => Some(session_id),
            Self::ToolCall { session_id, .. } => Some(session_id),
            Self::ToolResult { session_id, .. } => Some(session_id),
            Self::StatusUpdate { session_id, .. } => Some(session_id),
            Self::Connected { session_id } => session_id.as_deref(),
            _ => None,
        }
    }
}

#[derive(Debug, Clone)]
pub struct AcpClientConnection {
    pub client_id: String,
    pub session_id: String,
    pub connected_at: i64,
    pub last_heartbeat: Option<i64>,
    pub capabilities: Vec<String>,
}

pub type SharedAcpClientRegistry = Arc<RwLock<AcpClientRegistry>>;

pub struct AcpClientRegistry {
    clients: HashMap<String, AcpClientConnection>,
}

#[allow(dead_code)]
impl AcpClientRegistry {
    pub fn new() -> Self {
        Self {
            clients: HashMap::new(),
        }
    }

    pub(crate) fn register(&mut self, connection_id: String, client: AcpClientConnection) {
        self.clients.insert(connection_id, client);
    }

    pub(crate) fn unregister(&mut self, connection_id: &str) -> Option<AcpClientConnection> {
        self.clients.remove(connection_id)
    }

    pub(crate) fn get(&self, connection_id: &str) -> Option<&AcpClientConnection> {
        self.clients.get(connection_id)
    }

    pub(crate) fn get_by_session(&self, session_id: &str) -> Vec<&AcpClientConnection> {
        self.clients
            .values()
            .filter(|c| c.session_id == session_id)
            .collect()
    }

    pub(crate) fn get_by_client(&self, client_id: &str) -> Vec<&AcpClientConnection> {
        self.clients
            .values()
            .filter(|c| c.client_id == client_id)
            .collect()
    }

    pub(crate) fn update_heartbeat(&mut self, connection_id: &str) {
        if let Some(client) = self.clients.get_mut(connection_id) {
            client.last_heartbeat = Some(chrono::Utc::now().timestamp());
        }
    }

    pub(crate) fn active_clients(&self) -> usize {
        self.clients.len()
    }
}

#[allow(dead_code)]
impl Default for AcpClientRegistry {
    fn default() -> Self {
        Self::new()
    }
}

pub async fn acp_ws_index(
    state: web::Data<ServerState>,
    req: HttpRequest,
    stream: web::Payload,
) -> Result<HttpResponse, Error> {
    let query = parse_query(req.query_string());
    let client_id = query
        .get("client_id")
        .cloned()
        .unwrap_or_else(|| "default".to_string());
    let connection_id = format!("acp-ws-{}-{}", client_id, uuid::Uuid::new_v4());

    state
        .connection_monitor
        .register_connection(
            connection_id.clone(),
            ConnectionType::WebSocket,
            client_id.clone(),
        )
        .await;

    let ws_result = actix_ws::handle(&req, stream);

    let (mut response, mut ws_session, mut msg_stream) = match ws_result {
        Ok(result) => result,
        Err(e) => {
            let err_msg = format!("ACP WebSocket handshake failed: {}", e);
            error!(
                "ACP WS handshake error: connection_id={}, client_id={}, error={}",
                connection_id, client_id, e
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
                "acp_websocket_handshake_failed",
                format!("Failed to establish ACP WebSocket connection: {}", e),
            ));
        }
    };

    let state = state.into_inner();
    let conn_monitor = Arc::clone(&state.connection_monitor);
    let acp_stream = state.acp_stream.clone();
    let conn_id = connection_id.clone();
    let client_id_clone = client_id.clone();

    actix_rt::spawn(async move {
        let (tx, mut rx) = mpsc::channel::<AcpWsOutgoing>(128);
        let mut last_heartbeat = Instant::now();
        let conn_id_for_task = conn_id.clone();

        let handshake_completed = Arc::new(AtomicBool::new(false));
        let handshake_completed_for_handler = handshake_completed.clone();
        let mut acp_session_id = String::new();
        let mut client_registry = state.acp_client_registry.write().await;
        client_registry.register(
            conn_id.clone(),
            AcpClientConnection {
                client_id: client_id_clone.clone(),
                session_id: String::new(),
                connected_at: chrono::Utc::now().timestamp(),
                last_heartbeat: Some(chrono::Utc::now().timestamp()),
                capabilities: Vec::new(),
            },
        );
        drop(client_registry);

        let _heartbeat_handle = spawn_acp_heartbeat(tx.clone(), handshake_completed.clone());

        let tx_bus = tx.clone();
        let mut bus_rx = acp_stream.subscribe();
        actix_rt::spawn(async move {
            loop {
                match bus_rx.recv().await {
                    Ok(event) => {
                        if let Some(msg) = acp_event_to_outgoing(event) {
                            if tx_bus.send(msg).await.is_err() {
                                break;
                            }
                        }
                    }
                    Err(tokio::sync::broadcast::error::RecvError::Lagged(_)) => continue,
                    Err(tokio::sync::broadcast::error::RecvError::Closed) => break,
                }
            }
        });

        let _ = tx.send(AcpWsOutgoing::Connected { session_id: None }).await;

        loop {
            if last_heartbeat.elapsed() > ACP_WS_CLIENT_TIMEOUT {
                warn!("ACP WebSocket heartbeat timeout");
                conn_monitor
                    .unregister_connection(&conn_id_for_task, "acp_heartbeat_timeout")
                    .await;
                let _ = ws_session.close(None).await;
                break;
            }

            tokio::select! {
                Some(outgoing) = rx.recv() => {
                    if let Some(session_id) = outgoing.session_id() {
                        if !session_id.is_empty() && session_id != acp_session_id {
                            continue;
                        }
                    }

                    if let Ok(json) = serde_json::to_string(&outgoing) {
                        if ws_session.text(json).await.is_err() {
                            conn_monitor.unregister_connection(&conn_id_for_task, "acp_send_error").await;
                            break;
                        }
                    }
                }
                Some(Ok(msg)) = msg_stream.next() => {
                    match msg {
                        Message::Ping(bytes) => {
                            conn_monitor.heartbeat_success(&conn_id_for_task).await;
                            if ws_session.pong(&bytes).await.is_err() {
                                conn_monitor.unregister_connection(&conn_id_for_task, "acp_pong_error").await;
                                break;
                            }
                            last_heartbeat = Instant::now();
                        }
                        Message::Pong(_) => {
                            conn_monitor.heartbeat_success(&conn_id_for_task).await;
                            last_heartbeat = Instant::now();
                        }
                        Message::Text(text) => {
                            debug!("ACP WS inbound: {}", text);
                            handle_acp_ws_message(
                                &mut ws_session,
                                &text,
                                &state,
                                &tx,
                                &handshake_completed_for_handler,
                                &mut acp_session_id,
                            ).await;
                            conn_monitor.heartbeat_success(&conn_id_for_task).await;
                            last_heartbeat = Instant::now();
                        }
                        Message::Close(reason) => {
                            info!("ACP WS closed: {:?}", reason);
                            conn_monitor.unregister_connection(&conn_id_for_task, "acp_client_close").await;
                            let _ = ws_session.close(reason).await;
                            break;
                        }
                        Message::Binary(_) => {
                            let _ = tx.send(AcpWsOutgoing::Error {
                                code: "unsupported_binary".to_string(),
                                message: "binary ACP WebSocket messages are not supported".to_string(),
                            }).await;
                        }
                        _ => {
                            conn_monitor.unregister_connection(&conn_id_for_task, "acp_unknown_message").await;
                            break;
                        }
                    }
                }
                else => break,
            }
        }

        {
            let mut registry = state.acp_client_registry.write().await;
            registry.unregister(&conn_id);
        }
    });

    Ok(response)
}

async fn handle_acp_ws_message(
    session: &mut actix_ws::Session,
    text: &str,
    state: &Arc<ServerState>,
    tx: &mpsc::Sender<AcpWsOutgoing>,
    handshake_completed: &Arc<AtomicBool>,
    acp_session_id: &mut String,
) {
    match serde_json::from_str::<AcpWsMessage>(text) {
        Ok(AcpWsMessage::Ping) => {
            let _ = tx
                .send(AcpWsOutgoing::Heartbeat {
                    timestamp: chrono::Utc::now().timestamp(),
                })
                .await;
        }
        Ok(AcpWsMessage::Pong) => {}
        Ok(AcpWsMessage::Close) => {
            let _ = session.clone().close(None).await;
        }
        Ok(AcpWsMessage::Handshake {
            version,
            client_id,
            capabilities,
        }) => {
            let protocol = AcpProtocol::new("server", "1.0");
            let request = AcpHandshakeRequest {
                version,
                client_id: client_id.clone(),
                capabilities: capabilities.clone(),
            };
            let response = protocol.process_handshake(request);

            if response.accepted {
                handshake_completed.store(true, Ordering::SeqCst);
                *acp_session_id = response.session_id.clone();

                {
                    let mut registry = state.acp_client_registry.write().await;
                    if let Some(client) = registry.clients.get_mut(&client_id) {
                        client.session_id = response.session_id.clone();
                        client.capabilities = capabilities;
                    }
                }

                let event = AcpAgentEvent::new(
                    "server",
                    AcpEventType::StatusChanged,
                    serde_json::json!({
                        "client_id": client_id,
                        "session_id": response.session_id,
                        "status": "handshake_completed"
                    }),
                );
                state.acp_stream.publish(event);
            }

            let _ = tx
                .send(AcpWsOutgoing::HandshakeResponse {
                    version: response.version,
                    server_id: response.server_id,
                    session_id: response.session_id,
                    accepted: response.accepted,
                    error: response.error,
                })
                .await;
        }
        Ok(AcpWsMessage::HandshakeAck {
            session_id,
            confirmed,
        }) => {
            if !handshake_completed.load(Ordering::SeqCst) {
                let _ = tx
                    .send(AcpWsOutgoing::Error {
                        code: "handshake_not_completed".to_string(),
                        message: "Must complete handshake before sending ack".to_string(),
                    })
                    .await;
                return;
            }

            let protocol = AcpProtocol::new("server", "1.0");
            let ack = AcpHandshakeAck {
                session_id: session_id.clone(),
                confirmed,
            };
            let is_confirmed = protocol.confirm_handshake(ack);

            if is_confirmed {
                let event = AcpAgentEvent::status("server", "session_confirmed");
                state.acp_stream.publish(event);
            }
        }
        Ok(AcpWsMessage::EditorMessage {
            session_id,
            content,
        }) => {
            if !handshake_completed.load(Ordering::SeqCst) || session_id != *acp_session_id {
                let _ = tx
                    .send(AcpWsOutgoing::Error {
                        code: "invalid_session".to_string(),
                        message: "Session not established or mismatch".to_string(),
                    })
                    .await;
                return;
            }

            let event = AcpAgentEvent::new(
                "editor",
                AcpEventType::MessageGenerated,
                serde_json::json!({
                    "session_id": session_id,
                    "content": content
                }),
            );
            state.acp_stream.publish(event);

            let _ = tx
                .send(AcpWsOutgoing::SessionMessage {
                    session_id,
                    content: "Message received by server".to_string(),
                    role: "system".to_string(),
                })
                .await;
        }
        Ok(AcpWsMessage::ToolCall {
            session_id,
            tool_name,
            args,
            call_id,
        }) => {
            if !handshake_completed.load(Ordering::SeqCst) || session_id != *acp_session_id {
                let _ = tx
                    .send(AcpWsOutgoing::Error {
                        code: "invalid_session".to_string(),
                        message: "Session not established or mismatch".to_string(),
                    })
                    .await;
                return;
            }

            let event = AcpAgentEvent::tool_started("editor", &tool_name, &call_id);
            state.acp_stream.publish(event);

            let _ = tx
                .send(AcpWsOutgoing::ToolCall {
                    session_id,
                    tool_name,
                    args,
                    call_id,
                })
                .await;
        }
        Ok(AcpWsMessage::ToolResult {
            session_id,
            call_id,
            output,
            success,
        }) => {
            if !handshake_completed.load(Ordering::SeqCst) || session_id != *acp_session_id {
                return;
            }

            let event = AcpAgentEvent::tool_completed("editor", &call_id, success);
            state.acp_stream.publish(event);

            let _ = tx
                .send(AcpWsOutgoing::ToolResult {
                    session_id,
                    call_id,
                    output,
                    success,
                })
                .await;
        }
        Ok(AcpWsMessage::Status { status }) => {
            if !handshake_completed.load(Ordering::SeqCst) {
                return;
            }

            let event = AcpAgentEvent::status("editor", &status);
            state.acp_stream.publish(event);

            let _ = tx
                .send(AcpWsOutgoing::StatusUpdate {
                    session_id: acp_session_id.clone(),
                    status,
                })
                .await;
        }
        Err(e) => {
            let _ = tx
                .send(AcpWsOutgoing::Error {
                    code: "parse_error".to_string(),
                    message: format!("invalid ACP WebSocket payload: {}", e),
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
            if key.is_empty() {
                return None;
            }
            let value = parts.next().unwrap_or_default();
            Some((key.to_string(), value.to_string()))
        })
        .collect()
}

fn acp_event_to_outgoing(event: AcpAgentEvent) -> Option<AcpWsOutgoing> {
    match event.event_type {
        AcpEventType::StatusChanged => Some(AcpWsOutgoing::StatusUpdate {
            session_id: event.agent_id,
            status: event.payload.get("status")?.as_str()?.to_string(),
        }),
        AcpEventType::ToolCallStarted => Some(AcpWsOutgoing::ToolCall {
            session_id: event.agent_id,
            tool_name: event.payload.get("tool")?.as_str()?.to_string(),
            args: event
                .payload
                .get("args")
                .cloned()
                .unwrap_or(serde_json::Value::Null),
            call_id: event.payload.get("call_id")?.as_str()?.to_string(),
        }),
        AcpEventType::ToolCallCompleted => Some(AcpWsOutgoing::ToolResult {
            session_id: event.agent_id,
            call_id: event.payload.get("call_id")?.as_str()?.to_string(),
            output: String::new(),
            success: event.payload.get("success")?.as_bool()?,
        }),
        AcpEventType::ToolCallFailed => Some(AcpWsOutgoing::ToolResult {
            session_id: event.agent_id,
            call_id: event.payload.get("call_id")?.as_str()?.to_string(),
            output: event.payload.get("error")?.as_str()?.to_string(),
            success: false,
        }),
        AcpEventType::MessageGenerated => Some(AcpWsOutgoing::SessionMessage {
            session_id: event.payload.get("session_id")?.as_str()?.to_string(),
            content: event.payload.get("content")?.as_str()?.to_string(),
            role: "assistant".to_string(),
        }),
        AcpEventType::SessionStarted => Some(AcpWsOutgoing::StatusUpdate {
            session_id: event.agent_id,
            status: "started".to_string(),
        }),
        AcpEventType::SessionEnded => Some(AcpWsOutgoing::StatusUpdate {
            session_id: event.agent_id,
            status: "ended".to_string(),
        }),
        AcpEventType::LogLine => None,
        AcpEventType::Heartbeat => Some(AcpWsOutgoing::Heartbeat {
            timestamp: chrono::Utc::now().timestamp(),
        }),
    }
}

pub fn init(cfg: &mut web::ServiceConfig) {
    cfg.route("", web::get().to(acp_ws_index));
}

fn spawn_acp_heartbeat(
    tx: mpsc::Sender<AcpWsOutgoing>,
    handshake_completed: Arc<AtomicBool>,
) -> tokio::task::JoinHandle<()> {
    tokio::spawn(async move {
        while !handshake_completed.load(Ordering::SeqCst) {
            tokio::time::sleep(Duration::from_millis(100)).await;
        }
        let mut ticker = tokio::time::interval(ACP_WS_HEARTBEAT_INTERVAL);
        ticker.tick().await;
        loop {
            let heartbeat = AcpWsOutgoing::Heartbeat {
                timestamp: chrono::Utc::now().timestamp(),
            };
            if tx.send(heartbeat).await.is_err() {
                break;
            }
            ticker.tick().await;
        }
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_acp_ws_message_handshake_serialization() {
        let msg = AcpWsMessage::Handshake {
            version: "1.0".to_string(),
            client_id: "editor-1".to_string(),
            capabilities: vec!["chat".to_string(), "tools".to_string()],
        };

        let json = serde_json::to_string(&msg).expect("Should serialize");
        assert!(json.contains("\"type\":\"handshake\""));
        assert!(json.contains("\"version\":\"1.0\""));
        assert!(json.contains("\"client_id\":\"editor-1\""));
    }

    #[test]
    fn test_acp_ws_message_editor_message_serialization() {
        let msg = AcpWsMessage::EditorMessage {
            session_id: "session-1".to_string(),
            content: "Hello".to_string(),
        };

        let json = serde_json::to_string(&msg).expect("Should serialize");
        assert!(json.contains("\"type\":\"editor_message\""));
        assert!(json.contains("\"session_id\":\"session-1\""));
        assert!(json.contains("\"content\":\"Hello\""));
    }

    #[test]
    fn test_acp_ws_outgoing_handshake_response_serialization() {
        let msg = AcpWsOutgoing::HandshakeResponse {
            version: "1.0".to_string(),
            server_id: "server-1".to_string(),
            session_id: "session-123".to_string(),
            accepted: true,
            error: None,
        };

        let json = serde_json::to_string(&msg).expect("Should serialize");
        assert!(json.contains("\"type\":\"handshake_response\""));
        assert!(json.contains("\"accepted\":true"));
    }

    #[test]
    fn test_acp_ws_outgoing_session_message_serialization() {
        let msg = AcpWsOutgoing::SessionMessage {
            session_id: "session-1".to_string(),
            content: "Hello from server".to_string(),
            role: "assistant".to_string(),
        };

        let json = serde_json::to_string(&msg).expect("Should serialize");
        assert!(json.contains("\"type\":\"session_message\""));
        assert!(json.contains("\"content\":\"Hello from server\""));
    }

    #[test]
    fn test_acp_client_registry_register_and_get() {
        let mut registry = AcpClientRegistry::new();

        registry.register(
            "conn-1".to_string(),
            AcpClientConnection {
                client_id: "editor-1".to_string(),
                session_id: "session-1".to_string(),
                connected_at: 1234567890,
                last_heartbeat: Some(1234567890),
                capabilities: vec!["chat".to_string()],
            },
        );

        let client = registry.get("conn-1");
        assert!(client.is_some());
        assert_eq!(client.unwrap().client_id, "editor-1");
    }

    #[test]
    fn test_acp_client_registry_unregister() {
        let mut registry = AcpClientRegistry::new();

        registry.register(
            "conn-1".to_string(),
            AcpClientConnection {
                client_id: "editor-1".to_string(),
                session_id: "session-1".to_string(),
                connected_at: 1234567890,
                last_heartbeat: Some(1234567890),
                capabilities: vec![],
            },
        );

        let removed = registry.unregister("conn-1");
        assert!(removed.is_some());
        assert!(registry.get("conn-1").is_none());
    }

    #[test]
    fn test_acp_client_registry_get_by_session() {
        let mut registry = AcpClientRegistry::new();

        registry.register(
            "conn-1".to_string(),
            AcpClientConnection {
                client_id: "editor-1".to_string(),
                session_id: "session-1".to_string(),
                connected_at: 1234567890,
                last_heartbeat: Some(1234567890),
                capabilities: vec![],
            },
        );
        registry.register(
            "conn-2".to_string(),
            AcpClientConnection {
                client_id: "editor-2".to_string(),
                session_id: "session-1".to_string(),
                connected_at: 1234567890,
                last_heartbeat: Some(1234567890),
                capabilities: vec![],
            },
        );

        let clients = registry.get_by_session("session-1");
        assert_eq!(clients.len(), 2);
    }

    #[test]
    fn test_parse_query() {
        let query = "client_id=editor1&session_id=abc123";
        let params = parse_query(query);

        assert_eq!(params.get("client_id"), Some(&"editor1".to_string()));
        assert_eq!(params.get("session_id"), Some(&"abc123".to_string()));
    }

    #[test]
    fn test_parse_query_empty() {
        let query = "";
        let params = parse_query(query);
        assert!(params.is_empty());
    }

    #[test]
    fn test_acp_event_to_outgoing_status_changed() {
        let event = AcpAgentEvent::status("agent-1", "running");
        let outgoing = acp_event_to_outgoing(event);

        assert!(outgoing.is_some());
        match outgoing.unwrap() {
            AcpWsOutgoing::StatusUpdate { session_id, status } => {
                assert_eq!(session_id, "agent-1");
                assert_eq!(status, "running");
            }
            _ => panic!("Expected StatusUpdate"),
        }
    }

    #[test]
    fn test_acp_event_to_outgoing_tool_started() {
        let event = AcpAgentEvent::tool_started("agent-1", "read", "call-1");
        let outgoing = acp_event_to_outgoing(event);

        assert!(outgoing.is_some());
        match outgoing.unwrap() {
            AcpWsOutgoing::ToolCall {
                session_id,
                tool_name,
                call_id,
                ..
            } => {
                assert_eq!(session_id, "agent-1");
                assert_eq!(tool_name, "read");
                assert_eq!(call_id, "call-1");
            }
            _ => panic!("Expected ToolCall"),
        }
    }
}
