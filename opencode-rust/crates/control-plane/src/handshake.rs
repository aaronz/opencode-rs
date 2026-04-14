use std::time::{Duration, Instant};

const DEFAULT_HANDSHAKE_TIMEOUT: Duration = Duration::from_secs(30);

#[derive(Debug, Clone)]
pub struct AcpHandshakeConfig {
    pub version: String,
    pub client_id: String,
    pub capabilities: Vec<String>,
    pub timeout: Duration,
}

impl AcpHandshakeConfig {
    pub fn new(version: &str, client_id: &str) -> Self {
        Self {
            version: version.to_string(),
            client_id: client_id.to_string(),
            capabilities: Vec::new(),
            timeout: DEFAULT_HANDSHAKE_TIMEOUT,
        }
    }

    pub fn with_capabilities(mut self, capabilities: Vec<String>) -> Self {
        self.capabilities = capabilities;
        self
    }

    pub fn with_timeout(mut self, timeout: Duration) -> Self {
        self.timeout = timeout;
        self
    }
}

#[derive(Debug, Clone, PartialEq, Default)]
pub enum HandshakeState {
    #[default]
    NotStarted,
    Initiated,
    AwaitingResponse,
    ResponseReceived,
    Confirmed,
    Failed(String),
}

pub struct AcpHandshake {
    config: AcpHandshakeConfig,
    state: HandshakeState,
    session_id: Option<String>,
    server_id: Option<String>,
    initiated_at: Option<Instant>,
    completed_at: Option<Instant>,
    error: Option<String>,
}

impl AcpHandshake {
    pub fn new(config: AcpHandshakeConfig) -> Self {
        Self {
            config,
            state: HandshakeState::NotStarted,
            session_id: None,
            server_id: None,
            initiated_at: None,
            completed_at: None,
            error: None,
        }
    }

    pub fn config(&self) -> &AcpHandshakeConfig {
        &self.config
    }

    pub fn state(&self) -> &HandshakeState {
        &self.state
    }

    pub fn session_id(&self) -> Option<&str> {
        self.session_id.as_deref()
    }

    pub fn server_id(&self) -> Option<&str> {
        self.server_id.as_deref()
    }

    pub fn error(&self) -> Option<&str> {
        self.error.as_deref()
    }

    pub fn is_completed(&self) -> bool {
        matches!(
            self.state,
            HandshakeState::Confirmed | HandshakeState::Failed(_)
        )
    }

    pub fn is_successful(&self) -> bool {
        self.state == HandshakeState::Confirmed
    }

    pub fn duration(&self) -> Option<Duration> {
        self.initiated_at.and_then(|started| {
            self.completed_at
                .map(|completed| completed.duration_since(started))
        })
    }

    pub fn initiate(&mut self) -> Result<AcpOutgoingHandshake, String> {
        if self.state != HandshakeState::NotStarted {
            return Err("Handshake already initiated".to_string());
        }

        self.state = HandshakeState::Initiated;
        self.initiated_at = Some(Instant::now());

        Ok(AcpOutgoingHandshake {
            version: self.config.version.clone(),
            client_id: self.config.client_id.clone(),
            capabilities: self.config.capabilities.clone(),
        })
    }

    pub fn process_response(&mut self, response: AcpHandshakeResponse) -> Result<(), String> {
        if self.state != HandshakeState::Initiated {
            return Err("Handshake not initiated".to_string());
        }

        self.state = HandshakeState::AwaitingResponse;
        self.server_id = Some(response.server_id.clone());

        if !response.accepted {
            self.state = HandshakeState::Failed(response.error.clone().unwrap_or_default());
            self.error = response.error;
            self.completed_at = Some(Instant::now());
            return Err(self.error.clone().unwrap_or_default());
        }

        if response.version != self.config.version {
            let err = format!(
                "Version mismatch: expected {}, got {}",
                self.config.version, response.version
            );
            self.state = HandshakeState::Failed(err.clone());
            self.error = Some(err);
            self.completed_at = Some(Instant::now());
            return Err(self.error.clone().unwrap_or_default());
        }

        self.session_id = Some(response.session_id.clone());
        self.state = HandshakeState::ResponseReceived;

        Ok(())
    }

    pub fn confirm(&mut self) -> Result<AcpHandshakeConfirmation, String> {
        if self.state != HandshakeState::ResponseReceived {
            return Err("Cannot confirm: no successful response received".to_string());
        }

        let session_id = self.session_id.clone().ok_or("No session ID")?;

        self.state = HandshakeState::Confirmed;
        self.completed_at = Some(Instant::now());

        Ok(AcpHandshakeConfirmation { session_id })
    }

    pub fn time_since_initiation(&self) -> Option<Duration> {
        self.initiated_at.map(|i| i.elapsed())
    }

    pub fn is_timed_out(&self) -> bool {
        if let Some(initiated) = self.initiated_at {
            initiated.elapsed() > self.config.timeout
        } else {
            false
        }
    }
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct AcpOutgoingHandshake {
    pub version: String,
    pub client_id: String,
    pub capabilities: Vec<String>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct AcpHandshakeResponse {
    pub version: String,
    pub server_id: String,
    pub session_id: String,
    pub accepted: bool,
    pub error: Option<String>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct AcpHandshakeConfirmation {
    pub session_id: String,
}

pub struct AcpHandshakeManager {
    handshakes: Vec<AcpHandshake>,
    max_concurrent: usize,
}

impl AcpHandshakeManager {
    pub fn new() -> Self {
        Self {
            handshakes: Vec::new(),
            max_concurrent: 100,
        }
    }

    pub fn with_max_concurrent(mut self, max: usize) -> Self {
        self.max_concurrent = max;
        self
    }

    pub fn create_handshake(&mut self, config: AcpHandshakeConfig) -> Result<usize, String> {
        if self.handshakes.len() >= self.max_concurrent {
            return Err("Maximum concurrent handshakes reached".to_string());
        }

        let id = self.handshakes.len();
        self.handshakes.push(AcpHandshake::new(config));
        Ok(id)
    }

    pub fn get_handshake(&self, id: usize) -> Option<&AcpHandshake> {
        self.handshakes.get(id)
    }

    pub fn get_handshake_mut(&mut self, id: usize) -> Option<&mut AcpHandshake> {
        self.handshakes.get_mut(id)
    }

    pub fn remove_handshake(&mut self, id: usize) -> Option<AcpHandshake> {
        if id < self.handshakes.len() {
            Some(self.handshakes.remove(id))
        } else {
            None
        }
    }

    pub fn active_handshakes(&self) -> usize {
        self.handshakes.iter().filter(|h| !h.is_completed()).count()
    }

    pub fn cleanup_completed(&mut self) {
        self.handshakes.retain(|h| !h.is_completed());
    }
}

impl Default for AcpHandshakeManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_handshake_config_new() {
        let config = AcpHandshakeConfig::new("1.0", "client-1");

        assert_eq!(config.version, "1.0");
        assert_eq!(config.client_id, "client-1");
        assert!(config.capabilities.is_empty());
        assert_eq!(config.timeout, DEFAULT_HANDSHAKE_TIMEOUT);
    }

    #[test]
    fn test_handshake_config_with_capabilities() {
        let config = AcpHandshakeConfig::new("1.0", "client-1")
            .with_capabilities(vec!["chat".to_string(), "tools".to_string()])
            .with_timeout(Duration::from_secs(60));

        assert_eq!(config.capabilities, vec!["chat", "tools"]);
        assert_eq!(config.timeout, Duration::from_secs(60));
    }

    #[test]
    fn test_handshake_initial_state() {
        let config = AcpHandshakeConfig::new("1.0", "client-1");
        let handshake = AcpHandshake::new(config);

        assert_eq!(handshake.state(), &HandshakeState::NotStarted);
        assert!(handshake.session_id().is_none());
        assert!(handshake.server_id().is_none());
        assert!(!handshake.is_completed());
        assert!(!handshake.is_successful());
    }

    #[test]
    fn test_handshake_initiate() {
        let config =
            AcpHandshakeConfig::new("1.0", "client-1").with_capabilities(vec!["chat".to_string()]);
        let mut handshake = AcpHandshake::new(config);

        let outgoing = handshake.initiate().expect("Should initiate");

        assert_eq!(handshake.state(), &HandshakeState::Initiated);
        assert_eq!(outgoing.version, "1.0");
        assert_eq!(outgoing.client_id, "client-1");
        assert_eq!(outgoing.capabilities, vec!["chat"]);
    }

    #[test]
    fn test_handshake_initiate_twice_fails() {
        let config = AcpHandshakeConfig::new("1.0", "client-1");
        let mut handshake = AcpHandshake::new(config);

        handshake.initiate().expect("Should initiate");
        let result = handshake.initiate();

        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "Handshake already initiated");
    }

    #[test]
    fn test_handshake_process_response_success() {
        let config = AcpHandshakeConfig::new("1.0", "client-1");
        let mut handshake = AcpHandshake::new(config);

        handshake.initiate().expect("Should initiate");

        let response = AcpHandshakeResponse {
            version: "1.0".to_string(),
            server_id: "server-1".to_string(),
            session_id: "session-123".to_string(),
            accepted: true,
            error: None,
        };

        handshake
            .process_response(response)
            .expect("Should process");

        assert_eq!(handshake.state(), &HandshakeState::ResponseReceived);
        assert_eq!(handshake.session_id(), Some("session-123"));
        assert_eq!(handshake.server_id(), Some("server-1"));
    }

    #[test]
    fn test_handshake_process_response_rejected() {
        let config = AcpHandshakeConfig::new("1.0", "client-1");
        let mut handshake = AcpHandshake::new(config);

        handshake.initiate().expect("Should initiate");

        let response = AcpHandshakeResponse {
            version: "1.0".to_string(),
            server_id: "server-1".to_string(),
            session_id: String::new(),
            accepted: false,
            error: Some("Version mismatch".to_string()),
        };

        let result = handshake.process_response(response);

        assert!(result.is_err());
        assert_eq!(
            handshake.state(),
            &HandshakeState::Failed("Version mismatch".to_string())
        );
        assert!(handshake.session_id().is_none());
    }

    #[test]
    fn test_handshake_process_response_version_mismatch() {
        let config = AcpHandshakeConfig::new("1.0", "client-1");
        let mut handshake = AcpHandshake::new(config);

        handshake.initiate().expect("Should initiate");

        let response = AcpHandshakeResponse {
            version: "2.0".to_string(),
            server_id: "server-1".to_string(),
            session_id: "session-123".to_string(),
            accepted: true,
            error: None,
        };

        let result = handshake.process_response(response);

        assert!(result.is_err());
        assert!(handshake.error().is_some());
        assert!(handshake.error().unwrap().contains("Version mismatch"));
    }

    #[test]
    fn test_handshake_confirm_success() {
        let config = AcpHandshakeConfig::new("1.0", "client-1");
        let mut handshake = AcpHandshake::new(config);

        handshake.initiate().expect("Should initiate");

        let response = AcpHandshakeResponse {
            version: "1.0".to_string(),
            server_id: "server-1".to_string(),
            session_id: "session-123".to_string(),
            accepted: true,
            error: None,
        };
        handshake
            .process_response(response)
            .expect("Should process");

        let confirmation = handshake.confirm().expect("Should confirm");

        assert_eq!(handshake.state(), &HandshakeState::Confirmed);
        assert_eq!(confirmation.session_id, "session-123");
        assert!(handshake.is_completed());
        assert!(handshake.is_successful());
    }

    #[test]
    fn test_handshake_confirm_without_response_fails() {
        let config = AcpHandshakeConfig::new("1.0", "client-1");
        let mut handshake = AcpHandshake::new(config);

        handshake.initiate().expect("Should initiate");

        let result = handshake.confirm();

        assert!(result.is_err());
        assert!(result.unwrap_err().contains("no successful response"));
    }

    #[test]
    fn test_handshake_not_initiated_fails() {
        let config = AcpHandshakeConfig::new("1.0", "client-1");
        let mut handshake = AcpHandshake::new(config);

        let response = AcpHandshakeResponse {
            version: "1.0".to_string(),
            server_id: "server-1".to_string(),
            session_id: "session-123".to_string(),
            accepted: true,
            error: None,
        };

        let result = handshake.process_response(response);

        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "Handshake not initiated");
    }

    #[test]
    fn test_handshake_duration() {
        let config = AcpHandshakeConfig::new("1.0", "client-1");
        let mut handshake = AcpHandshake::new(config);

        assert!(handshake.duration().is_none());

        handshake.initiate().expect("Should initiate");

        assert!(handshake.time_since_initiation().is_some());

        let response = AcpHandshakeResponse {
            version: "1.0".to_string(),
            server_id: "server-1".to_string(),
            session_id: "session-123".to_string(),
            accepted: true,
            error: None,
        };
        handshake
            .process_response(response)
            .expect("Should process");
        handshake.confirm().expect("Should confirm");

        assert!(handshake.duration().is_some());
    }

    #[test]
    fn test_handshake_manager_create() {
        let mut manager = AcpHandshakeManager::new();

        let config = AcpHandshakeConfig::new("1.0", "client-1");
        let id = manager.create_handshake(config).expect("Should create");

        assert_eq!(id, 0);
        assert_eq!(manager.active_handshakes(), 1);
    }

    #[test]
    fn test_handshake_manager_max_concurrent() {
        let mut manager = AcpHandshakeManager::new().with_max_concurrent(2);

        let config1 = AcpHandshakeConfig::new("1.0", "client-1");
        let config2 = AcpHandshakeConfig::new("1.0", "client-2");
        let config3 = AcpHandshakeConfig::new("1.0", "client-3");

        manager.create_handshake(config1).expect("Should create");
        manager.create_handshake(config2).expect("Should create");

        let result = manager.create_handshake(config3);
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "Maximum concurrent handshakes reached");
    }

    #[test]
    fn test_handshake_manager_get_remove() {
        let mut manager = AcpHandshakeManager::new();

        let config = AcpHandshakeConfig::new("1.0", "client-1");
        let id = manager.create_handshake(config).expect("Should create");

        let handshake = manager.get_handshake(id).expect("Should get");
        assert_eq!(handshake.config().client_id, "client-1");

        let removed = manager.remove_handshake(id).expect("Should remove");
        assert_eq!(removed.config().client_id, "client-1");

        assert!(manager.get_handshake(id).is_none());
    }

    #[test]
    fn test_handshake_manager_cleanup() {
        let mut manager = AcpHandshakeManager::new();

        let config1 = AcpHandshakeConfig::new("1.0", "client-1");
        let config2 = AcpHandshakeConfig::new("1.0", "client-2");

        let id1 = manager.create_handshake(config1).expect("Should create");
        let _id2 = manager.create_handshake(config2).expect("Should create");

        let mut h1 = manager.remove_handshake(id1).unwrap();
        h1.initiate().expect("Should initiate");
        h1.update_state_for_test(HandshakeState::Confirmed);

        manager.handshakes.push(h1);

        manager.cleanup_completed();

        assert!(manager.active_handshakes() <= 1);
    }

    #[test]
    fn test_handshake_outgoing_serialization() {
        let outgoing = AcpOutgoingHandshake {
            version: "1.0".to_string(),
            client_id: "editor-1".to_string(),
            capabilities: vec!["chat".to_string()],
        };

        let json = serde_json::to_string(&outgoing).expect("Should serialize");
        assert!(json.contains("\"version\":\"1.0\""));
        assert!(json.contains("\"client_id\":\"editor-1\""));
        assert!(json.contains("\"capabilities\""));
    }

    #[test]
    fn test_handshake_response_serialization() {
        let response = AcpHandshakeResponse {
            version: "1.0".to_string(),
            server_id: "server-1".to_string(),
            session_id: "session-123".to_string(),
            accepted: true,
            error: None,
        };

        let json = serde_json::to_string(&response).expect("Should serialize");
        assert!(json.contains("\"version\":\"1.0\""));
        assert!(json.contains("\"accepted\":true"));
    }

    impl AcpHandshake {
        pub fn update_state_for_test(&mut self, state: HandshakeState) {
            self.state = state;
        }
    }

    #[test]
    fn test_acp_handshake_completes_successfully() {
        let config = AcpHandshakeConfig::new("1.0", "test-client")
            .with_capabilities(vec!["chat".to_string(), "tools".to_string()]);
        let mut handshake = AcpHandshake::new(config);

        assert_eq!(handshake.state(), &HandshakeState::NotStarted);

        let outgoing = handshake.initiate().expect("Should initiate");
        assert_eq!(handshake.state(), &HandshakeState::Initiated);
        assert_eq!(outgoing.version, "1.0");
        assert_eq!(outgoing.client_id, "test-client");

        let response = AcpHandshakeResponse {
            version: "1.0".to_string(),
            server_id: "server-1".to_string(),
            session_id: "session-456".to_string(),
            accepted: true,
            error: None,
        };
        handshake
            .process_response(response)
            .expect("Should process response");
        assert_eq!(handshake.state(), &HandshakeState::ResponseReceived);
        assert_eq!(handshake.session_id(), Some("session-456"));

        let confirmation = handshake.confirm().expect("Should confirm");
        assert_eq!(handshake.state(), &HandshakeState::Confirmed);
        assert_eq!(confirmation.session_id, "session-456");
        assert!(handshake.is_completed());
        assert!(handshake.is_successful());
    }

    #[test]
    fn test_acp_handshake_rejected_by_server() {
        let config = AcpHandshakeConfig::new("1.0", "test-client");
        let mut handshake = AcpHandshake::new(config);

        handshake.initiate().expect("Should initiate");

        let response = AcpHandshakeResponse {
            version: "1.0".to_string(),
            server_id: "server-1".to_string(),
            session_id: String::new(),
            accepted: false,
            error: Some("Client not authorized".to_string()),
        };
        let result = handshake.process_response(response);
        assert!(result.is_err());
        assert_eq!(
            handshake.state(),
            &HandshakeState::Failed("Client not authorized".to_string())
        );
        assert!(!handshake.is_successful());
    }
}
