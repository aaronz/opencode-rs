use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};

use tokio::sync::{broadcast, RwLock};

use crate::acp_stream::AcpAgentEvent;

#[allow(dead_code)]
const DEFAULT_HEARTBEAT_INTERVAL: Duration = Duration::from_secs(30);
#[allow(dead_code)]
const DEFAULT_CONNECTION_TIMEOUT: Duration = Duration::from_secs(120);

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum AcpOutgoingMessage {
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

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum AcpIncomingMessage {
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

#[derive(Debug, Clone)]
pub struct AcpClientInfo {
    pub client_id: String,
    pub session_id: Option<String>,
    pub connected_at: i64,
    pub last_heartbeat: Option<i64>,
    pub capabilities: Vec<String>,
}

impl AcpClientInfo {
    pub fn new(client_id: String) -> Self {
        Self {
            client_id,
            session_id: None,
            connected_at: chrono::Utc::now().timestamp(),
            last_heartbeat: Some(chrono::Utc::now().timestamp()),
            capabilities: Vec::new(),
        }
    }

    pub fn with_capabilities(mut self, capabilities: Vec<String>) -> Self {
        self.capabilities = capabilities;
        self
    }

    pub fn set_session(&mut self, session_id: String) {
        self.session_id = Some(session_id);
    }

    pub fn update_heartbeat(&mut self) {
        self.last_heartbeat = Some(chrono::Utc::now().timestamp());
    }
}

pub struct AcpConnectionManager {
    connections: HashMap<String, AcpTransportClient>,
}

impl AcpConnectionManager {
    pub fn new() -> Self {
        Self {
            connections: HashMap::new(),
        }
    }

    pub fn register(&mut self, connection_id: String, client: AcpTransportClient) {
        self.connections.insert(connection_id, client);
    }

    pub fn unregister(&mut self, connection_id: &str) -> Option<AcpTransportClient> {
        self.connections.remove(connection_id)
    }

    pub fn get(&self, connection_id: &str) -> Option<&AcpTransportClient> {
        self.connections.get(connection_id)
    }

    pub fn get_mut(&mut self, connection_id: &str) -> Option<&mut AcpTransportClient> {
        self.connections.get_mut(connection_id)
    }

    pub fn active_connections(&self) -> usize {
        self.connections.len()
    }

    pub fn connection_ids(&self) -> Vec<String> {
        self.connections.keys().cloned().collect()
    }
}

impl Default for AcpConnectionManager {
    fn default() -> Self {
        Self::new()
    }
}

pub struct AcpTransportClient {
    connection_id: String,
    server_url: String,
    client_info: AcpClientInfo,
    state: AcpConnectionState,
    event_tx: broadcast::Sender<AcpAgentEvent>,
    last_message: Option<Instant>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum AcpConnectionState {
    Disconnected,
    Connecting,
    HandshakeSent,
    HandshakeConfirmed,
    Connected,
    Error(String),
}

impl AcpTransportClient {
    pub fn new(server_url: String, client_id: String) -> Self {
        let (event_tx, _) = broadcast::channel(512);
        Self {
            connection_id: format!("acp-client-{}", uuid::Uuid::new_v4()),
            server_url,
            client_info: AcpClientInfo::new(client_id),
            state: AcpConnectionState::Disconnected,
            event_tx,
            last_message: None,
        }
    }

    pub fn with_capabilities(mut self, capabilities: Vec<String>) -> Self {
        self.client_info = self.client_info.with_capabilities(capabilities);
        self
    }

    pub fn connection_id(&self) -> &str {
        &self.connection_id
    }

    pub fn client_id(&self) -> &str {
        &self.client_info.client_id
    }

    pub fn session_id(&self) -> Option<&str> {
        self.client_info.session_id.as_deref()
    }

    pub fn state(&self) -> &AcpConnectionState {
        &self.state
    }

    pub fn is_connected(&self) -> bool {
        self.state == AcpConnectionState::Connected
    }

    pub fn server_url(&self) -> &str {
        &self.server_url
    }

    pub fn client_info(&self) -> &AcpClientInfo {
        &self.client_info
    }

    pub fn subscribe(&self) -> broadcast::Receiver<AcpAgentEvent> {
        self.event_tx.subscribe()
    }

    pub fn update_state(&mut self, new_state: AcpConnectionState) {
        self.state = new_state;
        self.last_message = Some(Instant::now());
    }

    pub fn set_session(&mut self, session_id: String) {
        self.client_info.set_session(session_id);
    }

    pub fn update_heartbeat(&mut self) {
        self.client_info.update_heartbeat();
        self.last_message = Some(Instant::now());
    }

    pub fn time_since_last_message(&self) -> Option<Duration> {
        self.last_message.map(|i| i.elapsed())
    }

    pub fn create_handshake_message(&self, version: &str) -> AcpOutgoingMessage {
        AcpOutgoingMessage::Handshake {
            version: version.to_string(),
            client_id: self.client_info.client_id.clone(),
            capabilities: self.client_info.capabilities.clone(),
        }
    }

    pub fn create_handshake_ack(&self, session_id: &str) -> AcpOutgoingMessage {
        AcpOutgoingMessage::HandshakeAck {
            session_id: session_id.to_string(),
            confirmed: true,
        }
    }

    pub fn create_editor_message(&self, session_id: &str, content: &str) -> AcpOutgoingMessage {
        AcpOutgoingMessage::EditorMessage {
            session_id: session_id.to_string(),
            content: content.to_string(),
        }
    }

    pub fn create_tool_call(
        &self,
        session_id: &str,
        tool_name: &str,
        args: serde_json::Value,
        call_id: &str,
    ) -> AcpOutgoingMessage {
        AcpOutgoingMessage::ToolCall {
            session_id: session_id.to_string(),
            tool_name: tool_name.to_string(),
            args,
            call_id: call_id.to_string(),
        }
    }

    pub fn create_tool_result(
        &self,
        session_id: &str,
        call_id: &str,
        output: &str,
        success: bool,
    ) -> AcpOutgoingMessage {
        AcpOutgoingMessage::ToolResult {
            session_id: session_id.to_string(),
            call_id: call_id.to_string(),
            output: output.to_string(),
            success,
        }
    }

    pub fn create_status_update(&self, _session_id: &str, status: &str) -> AcpOutgoingMessage {
        AcpOutgoingMessage::Status {
            status: status.to_string(),
        }
    }

    pub fn create_ping(&self) -> AcpOutgoingMessage {
        AcpOutgoingMessage::Ping
    }

    pub fn create_close(&self) -> AcpOutgoingMessage {
        AcpOutgoingMessage::Close
    }

    pub fn handle_incoming_message(&mut self, msg: AcpIncomingMessage) -> Option<AcpAgentEvent> {
        self.last_message = Some(Instant::now());

        match msg {
            AcpIncomingMessage::HandshakeResponse {
                version: _,
                server_id,
                session_id,
                accepted,
                error,
            } => {
                self.update_state(AcpConnectionState::HandshakeSent);
                if accepted {
                    self.set_session(session_id.clone());
                    self.update_state(AcpConnectionState::HandshakeConfirmed);
                    Some(AcpAgentEvent::new(
                        &self.client_info.client_id,
                        crate::acp_stream::AcpEventType::StatusChanged,
                        serde_json::json!({
                            "server_id": server_id,
                            "session_id": session_id,
                            "status": "handshake_accepted"
                        }),
                    ))
                } else {
                    self.update_state(AcpConnectionState::Error(error.clone().unwrap_or_default()));
                    Some(AcpAgentEvent::new(
                        &self.client_info.client_id,
                        crate::acp_stream::AcpEventType::StatusChanged,
                        serde_json::json!({
                            "status": "handshake_rejected",
                            "error": error
                        }),
                    ))
                }
            }
            AcpIncomingMessage::Connected { session_id } => {
                self.update_state(AcpConnectionState::Connected);
                Some(AcpAgentEvent::new(
                    &self.client_info.client_id,
                    crate::acp_stream::AcpEventType::StatusChanged,
                    serde_json::json!({
                        "status": "connected",
                        "session_id": session_id
                    }),
                ))
            }
            AcpIncomingMessage::SessionMessage {
                session_id,
                content,
                role,
            } => Some(AcpAgentEvent::new(
                &session_id,
                crate::acp_stream::AcpEventType::MessageGenerated,
                serde_json::json!({
                    "session_id": session_id,
                    "content": content,
                    "role": role
                }),
            )),
            AcpIncomingMessage::ToolCall {
                session_id,
                tool_name,
                call_id,
                ..
            } => Some(AcpAgentEvent::tool_started(
                &session_id,
                &tool_name,
                &call_id,
            )),
            AcpIncomingMessage::ToolResult {
                session_id,
                call_id,
                success,
                ..
            } => Some(AcpAgentEvent::tool_completed(
                &session_id,
                &call_id,
                success,
            )),
            AcpIncomingMessage::StatusUpdate { session_id, status } => {
                Some(AcpAgentEvent::status(&session_id, &status))
            }
            AcpIncomingMessage::Heartbeat { timestamp: _ } => {
                self.update_heartbeat();
                None
            }
            AcpIncomingMessage::Error { code, message } => {
                self.update_state(AcpConnectionState::Error(message.clone()));
                Some(AcpAgentEvent::new(
                    &self.client_info.client_id,
                    crate::acp_stream::AcpEventType::StatusChanged,
                    serde_json::json!({
                        "status": "error",
                        "code": code,
                        "message": message
                    }),
                ))
            }
        }
    }

    pub fn publish_event(&self, event: AcpAgentEvent) {
        let _ = self.event_tx.send(event);
    }
}

pub type SharedConnectionManager = Arc<RwLock<AcpConnectionManager>>;

pub fn create_connection_manager() -> SharedConnectionManager {
    Arc::new(RwLock::new(AcpConnectionManager::new()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_acp_client_new() {
        let client = AcpTransportClient::new(
            "ws://localhost:8080/acp".to_string(),
            "editor-1".to_string(),
        );

        assert_eq!(client.client_id(), "editor-1");
        assert_eq!(client.server_url(), "ws://localhost:8080/acp");
        assert_eq!(client.state(), &AcpConnectionState::Disconnected);
        assert!(client.session_id().is_none());
        assert!(!client.is_connected());
    }

    #[test]
    fn test_acp_client_with_capabilities() {
        let client = AcpTransportClient::new(
            "ws://localhost:8080/acp".to_string(),
            "editor-1".to_string(),
        )
        .with_capabilities(vec!["chat".to_string(), "tools".to_string()]);

        assert_eq!(client.client_info.capabilities, vec!["chat", "tools"]);
    }

    #[test]
    fn test_acp_client_create_handshake_message() {
        let client = AcpTransportClient::new(
            "ws://localhost:8080/acp".to_string(),
            "editor-1".to_string(),
        )
        .with_capabilities(vec!["chat".to_string()]);

        let msg = client.create_handshake_message("1.0");

        match msg {
            AcpOutgoingMessage::Handshake {
                version,
                client_id,
                capabilities,
            } => {
                assert_eq!(version, "1.0");
                assert_eq!(client_id, "editor-1");
                assert_eq!(capabilities, vec!["chat"]);
            }
            _ => panic!("Expected Handshake message"),
        }
    }

    #[test]
    fn test_acp_client_create_handshake_ack() {
        let client = AcpTransportClient::new(
            "ws://localhost:8080/acp".to_string(),
            "editor-1".to_string(),
        );

        let msg = client.create_handshake_ack("session-123");

        match msg {
            AcpOutgoingMessage::HandshakeAck {
                session_id,
                confirmed,
            } => {
                assert_eq!(session_id, "session-123");
                assert!(confirmed);
            }
            _ => panic!("Expected HandshakeAck message"),
        }
    }

    #[test]
    fn test_acp_client_create_editor_message() {
        let client = AcpTransportClient::new(
            "ws://localhost:8080/acp".to_string(),
            "editor-1".to_string(),
        );

        let msg = client.create_editor_message("session-123", "Hello world");

        match msg {
            AcpOutgoingMessage::EditorMessage {
                session_id,
                content,
            } => {
                assert_eq!(session_id, "session-123");
                assert_eq!(content, "Hello world");
            }
            _ => panic!("Expected EditorMessage"),
        }
    }

    #[test]
    fn test_acp_client_create_tool_call() {
        let client = AcpTransportClient::new(
            "ws://localhost:8080/acp".to_string(),
            "editor-1".to_string(),
        );

        let args = serde_json::json!({"path": "/tmp/test.txt"});
        let msg = client.create_tool_call("session-123", "read", args, "call-1");

        match msg {
            AcpOutgoingMessage::ToolCall {
                session_id,
                tool_name,
                args,
                call_id,
            } => {
                assert_eq!(session_id, "session-123");
                assert_eq!(tool_name, "read");
                assert_eq!(args["path"], "/tmp/test.txt");
                assert_eq!(call_id, "call-1");
            }
            _ => panic!("Expected ToolCall"),
        }
    }

    #[test]
    fn test_acp_client_handle_handshake_response_accepted() {
        let mut client = AcpTransportClient::new(
            "ws://localhost:8080/acp".to_string(),
            "editor-1".to_string(),
        );

        let msg = AcpIncomingMessage::HandshakeResponse {
            version: "1.0".to_string(),
            server_id: "server-1".to_string(),
            session_id: "session-123".to_string(),
            accepted: true,
            error: None,
        };

        let event = client.handle_incoming_message(msg.clone());

        assert!(event.is_some());
        assert_eq!(client.session_id(), Some("session-123"));
        assert_eq!(client.state(), &AcpConnectionState::HandshakeConfirmed);
    }

    #[test]
    fn test_acp_client_handle_handshake_response_rejected() {
        let mut client = AcpTransportClient::new(
            "ws://localhost:8080/acp".to_string(),
            "editor-1".to_string(),
        );

        let msg = AcpIncomingMessage::HandshakeResponse {
            version: "1.0".to_string(),
            server_id: "server-1".to_string(),
            session_id: String::new(),
            accepted: false,
            error: Some("Version mismatch".to_string()),
        };

        let event = client.handle_incoming_message(msg);

        assert!(event.is_some());
        assert!(client.session_id().is_none());
        assert!(matches!(client.state(), AcpConnectionState::Error(_)));
    }

    #[test]
    fn test_acp_client_handle_session_message() {
        let mut client = AcpTransportClient::new(
            "ws://localhost:8080/acp".to_string(),
            "editor-1".to_string(),
        );

        client.set_session("session-123".to_string());
        client.update_state(AcpConnectionState::Connected);

        let msg = AcpIncomingMessage::SessionMessage {
            session_id: "session-123".to_string(),
            content: "Hello from server".to_string(),
            role: "assistant".to_string(),
        };

        let event = client.handle_incoming_message(msg);

        assert!(event.is_some());
        let event = event.unwrap();
        assert_eq!(event.agent_id, "session-123");
    }

    #[test]
    fn test_acp_client_handle_tool_result() {
        let mut client = AcpTransportClient::new(
            "ws://localhost:8080/acp".to_string(),
            "editor-1".to_string(),
        );

        let msg = AcpIncomingMessage::ToolResult {
            session_id: "session-123".to_string(),
            call_id: "call-1".to_string(),
            output: "File content".to_string(),
            success: true,
        };

        let event = client.handle_incoming_message(msg);

        assert!(event.is_some());
        let event = event.unwrap();
        assert_eq!(
            event.event_type,
            crate::acp_stream::AcpEventType::ToolCallCompleted
        );
    }

    #[test]
    fn test_acp_client_connection_manager() {
        let mut manager = AcpConnectionManager::new();

        let client = AcpTransportClient::new(
            "ws://localhost:8080/acp".to_string(),
            "editor-1".to_string(),
        );
        let conn_id = client.connection_id().to_string();

        manager.register(conn_id.clone(), client);

        assert_eq!(manager.active_connections(), 1);
        assert!(manager.get(&conn_id).is_some());

        let removed = manager.unregister(&conn_id);
        assert!(removed.is_some());
        assert!(manager.get(&conn_id).is_none());
        assert_eq!(manager.active_connections(), 0);
    }

    #[test]
    fn test_acp_client_info() {
        let info = AcpClientInfo::new("client-1".to_string())
            .with_capabilities(vec!["chat".to_string(), "tools".to_string()]);

        assert_eq!(info.client_id, "client-1");
        assert!(info.session_id.is_none());
        assert_eq!(info.capabilities, vec!["chat", "tools"]);

        let mut info = info;
        info.set_session("session-123".to_string());
        assert_eq!(info.session_id, Some("session-123".to_string()));
    }

    #[test]
    fn test_acp_outgoing_message_serialization() {
        let msg = AcpOutgoingMessage::Handshake {
            version: "1.0".to_string(),
            client_id: "editor-1".to_string(),
            capabilities: vec!["chat".to_string()],
        };

        let json = serde_json::to_string(&msg).expect("Should serialize");
        assert!(json.contains("\"type\":\"handshake\""));
        assert!(json.contains("\"version\":\"1.0\""));
        assert!(json.contains("\"client_id\":\"editor-1\""));
    }

    #[test]
    fn test_acp_incoming_message_serialization() {
        let msg = AcpIncomingMessage::HandshakeResponse {
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
    fn test_connection_state_transitions() {
        let mut client = AcpTransportClient::new(
            "ws://localhost:8080/acp".to_string(),
            "editor-1".to_string(),
        );

        assert_eq!(client.state(), &AcpConnectionState::Disconnected);

        client.update_state(AcpConnectionState::Connecting);
        assert_eq!(client.state(), &AcpConnectionState::Connecting);

        client.update_state(AcpConnectionState::HandshakeSent);
        assert_eq!(client.state(), &AcpConnectionState::HandshakeSent);

        client.update_state(AcpConnectionState::HandshakeConfirmed);
        assert_eq!(client.state(), &AcpConnectionState::HandshakeConfirmed);

        client.update_state(AcpConnectionState::Connected);
        assert!(client.is_connected());

        client.update_state(AcpConnectionState::Error("test error".to_string()));
        assert!(!client.is_connected());
        assert!(matches!(client.state(), AcpConnectionState::Error(_)));
    }

    #[test]
    fn test_acp_transport_sends_and_receives_messages() {
        let mut client = AcpTransportClient::new(
            "ws://localhost:8080/acp".to_string(),
            "editor-1".to_string(),
        )
        .with_capabilities(vec!["chat".to_string(), "tools".to_string()]);

        let handshake_msg = client.create_handshake_message("1.0");
        match handshake_msg {
            AcpOutgoingMessage::Handshake {
                version,
                client_id,
                capabilities,
            } => {
                assert_eq!(version, "1.0");
                assert_eq!(client_id, "editor-1");
                assert_eq!(capabilities, vec!["chat", "tools"]);
            }
            _ => panic!("Expected Handshake message"),
        }

        client.set_session("session-123".to_string());
        client.update_state(AcpConnectionState::Connected);

        let ack_msg = client.create_handshake_ack("session-123");
        match ack_msg {
            AcpOutgoingMessage::HandshakeAck {
                session_id,
                confirmed,
            } => {
                assert_eq!(session_id, "session-123");
                assert!(confirmed);
            }
            _ => panic!("Expected HandshakeAck message"),
        }

        let tool_call_msg = client.create_tool_call(
            "session-123",
            "read",
            serde_json::json!({"path": "/tmp/test.txt"}),
            "call-1",
        );
        match tool_call_msg {
            AcpOutgoingMessage::ToolCall {
                session_id,
                tool_name,
                args,
                call_id,
            } => {
                assert_eq!(session_id, "session-123");
                assert_eq!(tool_name, "read");
                assert_eq!(args["path"], "/tmp/test.txt");
                assert_eq!(call_id, "call-1");
            }
            _ => panic!("Expected ToolCall message"),
        }

        let result_msg = client.create_tool_result("session-123", "call-1", "file content", true);
        match result_msg {
            AcpOutgoingMessage::ToolResult {
                session_id,
                call_id,
                output,
                success,
            } => {
                assert_eq!(session_id, "session-123");
                assert_eq!(call_id, "call-1");
                assert_eq!(output, "file content");
                assert!(success);
            }
            _ => panic!("Expected ToolResult message"),
        }
    }

    #[test]
    fn test_acp_transport_client_connection_lifecycle() {
        let server_url = "ws://localhost:8080/acp".to_string();
        let client_id = "editor-1".to_string();

        let client = AcpTransportClient::new(server_url.clone(), client_id.clone());

        assert_eq!(client.connection_id(), client.connection_id());
        assert_eq!(client.server_url(), &server_url);
        assert_eq!(client.client_id(), &client_id);
        assert!(client.session_id().is_none());
        assert!(!client.is_connected());
        assert_eq!(client.state(), &AcpConnectionState::Disconnected);

        let mut client = client;
        client.update_state(AcpConnectionState::Connecting);
        assert_eq!(client.state(), &AcpConnectionState::Connecting);

        client.update_state(AcpConnectionState::HandshakeSent);

        let response = AcpIncomingMessage::HandshakeResponse {
            version: "1.0".to_string(),
            server_id: "server-1".to_string(),
            session_id: "session-123".to_string(),
            accepted: true,
            error: None,
        };
        let event = client.handle_incoming_message(response);
        assert!(event.is_some());
        assert_eq!(client.session_id(), Some("session-123"));
        assert_eq!(client.state(), &AcpConnectionState::HandshakeConfirmed);

        client.update_state(AcpConnectionState::Connected);
        assert!(client.is_connected());
    }
}
