use std::sync::{Arc, Mutex};

pub use crate::protocol::{AcpMessage, AckRequest, AcpStatus, HandshakeRequest, HandshakeResponse};

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub enum AcpConnectionState {
    #[default]
    Disconnected,
    Handshaking,
    Connected,
    Failed(String),
}

impl std::fmt::Display for AcpConnectionState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Disconnected => write!(f, "Disconnected"),
            Self::Handshaking => write!(f, "Handshaking"),
            Self::Connected => write!(f, "Connected"),
            Self::Failed(msg) => write!(f, "Failed({})", msg),
        }
    }
}

pub struct AcpState {
    pub connection_state: AcpConnectionState,
    pub client_id: String,
    pub server_id: Option<String>,
    pub session_token: Option<String>,
    pub capabilities: Vec<String>,
    pub server_url: Option<String>,
}

impl AcpState {
    fn new(client_id: String) -> Self {
        Self {
            connection_state: AcpConnectionState::Disconnected,
            client_id,
            server_id: None,
            session_token: None,
            capabilities: Vec::new(),
            server_url: None,
        }
    }
}

#[derive(Clone)]
pub struct AcpClient {
    http: reqwest::Client,
    state: Arc<Mutex<AcpState>>,
    bus: opencode_core::bus::SharedEventBus,
}

impl AcpClient {
    pub fn new(http: reqwest::Client, client_id: String, bus: opencode_core::bus::SharedEventBus) -> Self {
        Self {
            http,
            state: Arc::new(Mutex::new(AcpState::new(client_id))),
            bus,
        }
    }

    pub fn connection_state(&self) -> AcpConnectionState {
        self.state.lock().unwrap().connection_state.clone()
    }

    pub fn state(&self) -> &Arc<Mutex<AcpState>> {
        &self.state
    }

    pub async fn status(&self) -> Result<AcpStatus, error::AcpError> {
        let state = self.state.lock().unwrap();
        let connected = matches!(state.connection_state, AcpConnectionState::Connected);
        Ok(AcpStatus {
            connected,
            client_id: Some(state.client_id.clone()),
            capabilities: state.capabilities.clone(),
            server_url: state.server_url.clone(),
        })
    }

    pub async fn handshake(
        &self,
        server_url: &str,
        client_id: String,
        capabilities: Vec<String>,
    ) -> Result<HandshakeResponse, error::AcpError> {
        let request = HandshakeRequest {
            client_id,
            capabilities,
            version: "1.0".to_string(),
        };

        let response = self
            .http
            .post(format!("{}/api/acp/handshake", server_url))
            .json(&request)
            .send()
            .await
            .map_err(error::AcpError::Http)?;

        if !response.status().is_success() {
            let error = response
                .text()
                .await
                .unwrap_or_else(|_| "Unknown error".to_string());
            return Err(error::AcpError::ServerError(error));
        }

        response
            .json::<HandshakeResponse>()
            .await
            .map_err(|e| error::AcpError::InvalidResponse(e.to_string()))
    }

    pub async fn connect(
        &self,
        server_url: &str,
        client_id: Option<String>,
    ) -> Result<(), error::AcpError> {
        {
            let mut state = self.state.lock().unwrap();
            state.connection_state = AcpConnectionState::Handshaking;
            state.server_url = Some(server_url.to_string());
        }

        let cid = client_id.unwrap_or_else(|| {
            std::env::var("OPENCODE_CLIENT_ID").unwrap_or_else(|_| uuid::Uuid::new_v4().to_string())
        });

        let response = match self.handshake(server_url, cid, vec!["chat".to_string()]).await {
            Ok(r) => r,
            Err(e) => {
                {
                    let mut state = self.state.lock().unwrap();
                    state.connection_state = AcpConnectionState::Disconnected;
                    state.server_url = None;
                }
                return Err(e);
            }
        };

        {
            let mut state = self.state.lock().unwrap();
            state.connection_state = AcpConnectionState::Connected;
            state.server_id = Some(response.server_id);
            state.session_token = response.session_token;
            state.capabilities = response.accepted_capabilities;
        }

        self.bus.publish(opencode_core::bus::InternalEvent::AcpEventReceived {
            agent_id: "self".to_string(),
            event_type: "acp.connected".to_string(),
        });

        Ok(())
    }

    pub async fn ack(
        &self,
        handshake_id: &str,
        accepted: bool,
    ) -> Result<(), error::AcpError> {
        {
            let state = self.state.lock().unwrap();
            if !matches!(state.connection_state, AcpConnectionState::Connected) {
                return Err(error::AcpError::NotConnected);
            }
        }

        let request = AckRequest {
            handshake_id: handshake_id.to_string(),
            accepted,
        };

        let server_url = {
            self.state
                .lock()
                .unwrap()
                .server_url
                .clone()
                .ok_or(error::AcpError::NotConnected)?
        };

        let response = self
            .http
            .post(format!("{}/api/acp/ack", server_url))
            .json(&request)
            .send()
            .await
            .map_err(error::AcpError::Http)?;

        if !response.status().is_success() {
            let error = response
                .text()
                .await
                .unwrap_or_else(|_| "Unknown error".to_string());
            return Err(error::AcpError::ServerError(error));
        }

        Ok(())
    }

    pub async fn send_message(
        &self,
        to: &str,
        message_type: &str,
        payload: serde_json::Value,
    ) -> Result<(), error::AcpError> {
        let (client_id, server_url) = {
            let state = self.state.lock().unwrap();
            if !matches!(state.connection_state, AcpConnectionState::Connected) {
                return Err(error::AcpError::NotConnected);
            }
            let client_id = state.client_id.clone();
            let server_url = state
                .server_url
                .clone()
                .ok_or(error::AcpError::NotConnected)?;
            (client_id, server_url)
        };

        let message = AcpMessage::new(client_id, to.to_string(), message_type.to_string(), payload);

        let response = self
            .http
            .post(format!("{}/api/acp/message", server_url))
            .json(&message)
            .send()
            .await
            .map_err(error::AcpError::Http)?;

        if !response.status().is_success() {
            let error = response
                .text()
                .await
                .unwrap_or_else(|_| "Unknown error".to_string());
            return Err(error::AcpError::ServerError(error));
        }

        Ok(())
    }

    pub async fn disconnect(&self) -> Result<(), error::AcpError> {
        {
            let mut state = self.state.lock().unwrap();
            state.connection_state = AcpConnectionState::Disconnected;
            state.server_id = None;
            state.session_token = None;
            state.server_url = None;
        }

        self.bus.publish(opencode_core::bus::InternalEvent::AcpEventReceived {
            agent_id: "self".to_string(),
            event_type: "acp.disconnected".to_string(),
        });

        Ok(())
    }
}

pub mod error {
    use thiserror::Error;

    #[derive(Debug, Error)]
    pub enum AcpError {
        #[error("Not connected")]
        NotConnected,

        #[error("Handshake failed: {0}")]
        HandshakeFailed(String),

        #[error("Connection failed: {0}")]
        ConnectionFailed(String),

        #[error("Server returned error: {0}")]
        ServerError(String),

        #[error("Invalid response: {0}")]
        InvalidResponse(String),

        #[error("HTTP error: {0}")]
        Http(#[from] reqwest::Error),

        #[error("State error: {0}")]
        State(String),
    }
}