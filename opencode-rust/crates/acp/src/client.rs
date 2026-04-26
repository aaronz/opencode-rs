use std::sync::{Arc, Mutex};
use std::time::Duration;

pub use crate::protocol::{AckRequest, AcpMessage, AcpStatus, HandshakeRequest, HandshakeResponse};

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
    pub base_url: Option<String>,
    pub connection_timeout: Option<Duration>,
    pub retry_config: Option<opencode_util::retry::RetryConfig>,
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
            base_url: None,
            connection_timeout: None,
            retry_config: None,
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
    pub fn new(
        http: reqwest::Client,
        client_id: String,
        bus: opencode_core::bus::SharedEventBus,
    ) -> Self {
        Self {
            http,
            state: Arc::new(Mutex::new(AcpState::new(client_id))),
            bus,
        }
    }

    pub fn with_base_url(
        http: reqwest::Client,
        client_id: String,
        bus: opencode_core::bus::SharedEventBus,
        base_url: String,
    ) -> Self {
        let mut state = AcpState::new(client_id);
        state.base_url = Some(base_url);
        Self {
            http,
            state: Arc::new(Mutex::new(state)),
            bus,
        }
    }

    pub fn set_base_url(&self, base_url: String) {
        let mut state = self.state.lock().unwrap();
        state.base_url = Some(base_url);
    }

    pub fn get_base_url(&self) -> Option<String> {
        self.state.lock().unwrap().base_url.clone()
    }

    pub fn set_connection_timeout(&self, timeout: Duration) {
        let mut state = self.state.lock().unwrap();
        state.connection_timeout = Some(timeout);
    }

    pub fn get_connection_timeout(&self) -> Option<Duration> {
        self.state.lock().unwrap().connection_timeout
    }

    pub fn set_retry_config(&self, config: opencode_util::retry::RetryConfig) {
        let mut state = self.state.lock().unwrap();
        state.retry_config = Some(config);
    }

    pub fn get_retry_config(&self) -> Option<opencode_util::retry::RetryConfig> {
        self.state.lock().unwrap().retry_config.clone()
    }

    pub fn clear_retry_config(&self) {
        let mut state = self.state.lock().unwrap();
        state.retry_config = None;
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
            version: Some("1.0".to_string()),
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

        let base = self.state.lock().unwrap().base_url.clone();
        let url_base = base.as_deref().unwrap_or(server_url);
        let url = format!("{}/api/acp/handshake", url_base);

        let mut req_builder = self.http.post(&url).json(&request);

        if let Some(timeout) = self.state.lock().unwrap().connection_timeout {
            req_builder = req_builder.timeout(timeout);
        }

        let response = req_builder.send().await.map_err(|e| {
            if e.is_timeout() {
                let timeout_secs = self
                    .state
                    .lock()
                    .unwrap()
                    .connection_timeout
                    .map(|d| d.as_secs())
                    .unwrap_or(0);
                error::AcpError::ConnectionTimeout {
                    timeout: timeout_secs,
                }
            } else {
                error::AcpError::Http(e)
            }
        })?;

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
        let retry_config = self.state.lock().unwrap().retry_config.clone();

        let connect_attempt = |attempt: u32| {
            let server_url = server_url.to_string();
            let client_id = client_id.clone();
            let state = self.state.clone();
            let this = self.clone();

            async move {
                {
                    let mut state_guard = state.lock().unwrap();
                    state_guard.connection_state = AcpConnectionState::Handshaking;
                    state_guard.server_url = Some(server_url.clone());
                    if state_guard.base_url.is_none() {
                        state_guard.base_url = Some(server_url.clone());
                    }
                }

                let cid = client_id.unwrap_or_else(|| {
                    std::env::var("OPENCODE_CLIENT_ID")
                        .unwrap_or_else(|_| uuid::Uuid::new_v4().to_string())
                });

                tracing::debug!("Connect attempt {} to {}", attempt, server_url);

                let result = this
                    .handshake(&server_url, cid, vec!["chat".to_string()])
                    .await;

                match result {
                    Ok(response) => {
                        {
                            let mut state_guard = state.lock().unwrap();
                            state_guard.connection_state = AcpConnectionState::Connected;
                            state_guard.server_id = Some(response.server_id.clone());
                            state_guard.session_token = response.session_token;
                            state_guard.capabilities = response.accepted_capabilities.clone();
                        }

                        let server_id = state.lock().unwrap().server_id.clone().unwrap();
                        let capabilities = state.lock().unwrap().capabilities.clone();
                        this.bus
                            .publish(opencode_core::bus::InternalEvent::AcpConnected {
                                server_id,
                                capabilities,
                            });

                        Ok(())
                    }
                    Err(e) => {
                        {
                            let mut state_guard = state.lock().unwrap();
                            state_guard.connection_state = AcpConnectionState::Disconnected;
                            state_guard.server_url = None;
                        }
                        tracing::debug!("Connect attempt {} failed: {}", attempt, e);
                        Err(e)
                    }
                }
            }
        };

        if let Some(config) = retry_config {
            let result = opencode_util::retry::retry(config, connect_attempt).await;

            match result {
                Ok(()) => Ok(()),
                Err((e, attempts)) => {
                    tracing::debug!("All connect attempts exhausted after {} tries", attempts);
                    Err(e)
                }
            }
        } else {
            connect_attempt(0).await
        }
    }

    pub async fn ack(&self, handshake_id: &str, accepted: bool) -> Result<(), error::AcpError> {
        let base_url;
        {
            let state = self.state.lock().unwrap();
            if !matches!(state.connection_state, AcpConnectionState::Connected) {
                return Err(error::AcpError::NotConnected);
            }
            base_url = state.base_url.clone().or(state.server_url.clone());
        }

        let request = AckRequest {
            handshake_id: handshake_id.to_string(),
            accepted,
        };

        let url_base = base_url.ok_or(error::AcpError::NotConnected)?;

        let mut req_builder = self
            .http
            .post(format!("{}/api/acp/ack", url_base))
            .json(&request);

        if let Some(timeout) = self.state.lock().unwrap().connection_timeout {
            req_builder = req_builder.timeout(timeout);
        }

        let response = req_builder.send().await.map_err(|e| {
            if e.is_timeout() {
                let timeout_secs = self
                    .state
                    .lock()
                    .unwrap()
                    .connection_timeout
                    .map(|d| d.as_secs())
                    .unwrap_or(0);
                error::AcpError::ConnectionTimeout {
                    timeout: timeout_secs,
                }
            } else {
                error::AcpError::Http(e)
            }
        })?;

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
        let (client_id, base_url) = {
            let state = self.state.lock().unwrap();
            if !matches!(state.connection_state, AcpConnectionState::Connected) {
                return Err(error::AcpError::NotConnected);
            }
            let client_id = state.client_id.clone();
            let base_url = state.base_url.clone().or(state.server_url.clone());
            (client_id, base_url)
        };

        let url_base = base_url.ok_or(error::AcpError::NotConnected)?;
        let message = AcpMessage::new(client_id, to.to_string(), message_type.to_string(), payload);

        let mut req_builder = self
            .http
            .post(format!("{}/api/acp/message", url_base))
            .json(&message);

        if let Some(timeout) = self.state.lock().unwrap().connection_timeout {
            req_builder = req_builder.timeout(timeout);
        }

        let response = req_builder.send().await.map_err(|e| {
            if e.is_timeout() {
                let timeout_secs = self
                    .state
                    .lock()
                    .unwrap()
                    .connection_timeout
                    .map(|d| d.as_secs())
                    .unwrap_or(0);
                error::AcpError::ConnectionTimeout {
                    timeout: timeout_secs,
                }
            } else {
                error::AcpError::Http(e)
            }
        })?;

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
            state.base_url = None;
        }

        self.bus
            .publish(opencode_core::bus::InternalEvent::AcpDisconnected);

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

        #[error("Connection timeout after {timeout}s")]
        ConnectionTimeout { timeout: u64 },

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
