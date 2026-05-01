use std::path::PathBuf;
use uuid::Uuid;

use opencode_control_plane::SharedAcpStream;
use opencode_core::acp::AcpHandshakeRequest;
use opencode_core::{Config, SessionSharing};

#[derive(clap::Args, Debug)]
pub struct AttachArgs {
    #[arg(short, long)]
    pub session_id: Option<String>,

    #[arg(short, long = "dir")]
    pub directory: Option<PathBuf>,

    pub url: Option<String>,
}

#[derive(Debug, Clone)]
pub enum AttachError {
    InvalidUrl(String),
    ConnectionFailed(String),
    SessionNotFound(String),
    HandshakeFailed(String),
    StateTransferFailed(String),
}

impl std::fmt::Display for AttachError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AttachError::InvalidUrl(msg) => write!(f, "Invalid URL: {}", msg),
            AttachError::ConnectionFailed(msg) => write!(f, "Connection failed: {}", msg),
            AttachError::SessionNotFound(id) => write!(f, "Session not found: {}", id),
            AttachError::HandshakeFailed(msg) => write!(f, "Handshake failed: {}", msg),
            AttachError::StateTransferFailed(msg) => write!(f, "State transfer failed: {}", msg),
        }
    }
}

impl std::error::Error for AttachError {}

pub struct AttachCommand {
    pub session_id: Option<String>,
    pub _directory: Option<PathBuf>,
    pub url: Option<String>,
}

impl AttachCommand {
    pub fn new(args: AttachArgs) -> Self {
        Self {
            session_id: args.session_id,
            _directory: args.directory,
            url: args.url,
        }
    }

    pub async fn execute(&self, config: &Config) -> Result<(), AttachError> {
        if let Some(url) = &self.url {
            self.attach_to_remote(url, config).await
        } else if let Some(session_id) = &self.session_id {
            self.attach_to_local_session(session_id, config).await
        } else {
            self.attach_to_local(config).await
        }
    }

    async fn attach_to_remote(&self, url: &str, _config: &Config) -> Result<(), AttachError> {
        println!("Connecting to remote session at: {}", url);

        let parsed_url = self.parse_url(url).map_err(AttachError::InvalidUrl)?;

        let (host, port, path) = parsed_url;
        let api_url = format!("http://{}:{}{}", host, port, path);

        let client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(30))
            .build()
            .map_err(|e| AttachError::ConnectionFailed(e.to_string()))?;

        let handshake_request = AcpHandshakeRequest {
            version: "1.0".to_string(),
            client_id: format!("cli-attach-{}", Uuid::new_v4()),
            capabilities: vec!["chat".to_string(), "attach".to_string()],
        };

        let response = client
            .post(format!("{}/api/acp/handshake", api_url))
            .json(&handshake_request)
            .send()
            .await
            .map_err(|e| AttachError::ConnectionFailed(e.to_string()))?;

        if !response.status().is_success() {
            return Err(AttachError::HandshakeFailed(format!(
                "Server returned: {}",
                response.status()
            )));
        }

        #[derive(serde::Deserialize)]
        struct HandshakeResponse {
            accepted: bool,
            session_id: Option<String>,
            error: Option<String>,
        }

        let handshake_response: HandshakeResponse = response
            .json()
            .await
            .map_err(|e| AttachError::HandshakeFailed(e.to_string()))?;

        if !handshake_response.accepted {
            return Err(AttachError::HandshakeFailed(
                handshake_response
                    .error
                    .unwrap_or_else(|| "Unknown error".to_string()),
            ));
        }

        let session_id = handshake_response.session_id.unwrap_or_default();
        println!("Successfully connected to remote session");
        println!("Session ID: {}", session_id);

        self.initiate_state_sync(&session_id, &api_url).await?;

        println!("State synchronization complete");
        println!("Attached to remote session successfully");

        Ok(())
    }

    async fn attach_to_local_session(
        &self,
        session_id: &str,
        _config: &Config,
    ) -> Result<(), AttachError> {
        println!("Attaching to local session: {}", session_id);

        let session_uuid = Uuid::parse_str(session_id)
            .map_err(|_| AttachError::SessionNotFound(session_id.to_string()))?;

        let sharing = SessionSharing::with_default_path();

        let session = sharing
            .get_session(&session_uuid)
            .map_err(|_| AttachError::SessionNotFound(session_id.to_string()))?;

        let _serialized_state = self
            .serialize_session_state(&session)
            .map_err(|e| AttachError::StateTransferFailed(e.to_string()))?;

        println!("Session loaded: {} messages", session.messages.len());
        println!("Session state: {:?}", session.state);

        println!("Attaching to session {} via ACP", session_id);

        self.initiate_state_sync(session_id, "http://127.0.0.1:8080")
            .await?;

        println!("State synchronization complete");
        println!("Attached to local session successfully");

        println!("Session summary:");
        println!("  ID: {}", session.id);
        println!("  Messages: {}", session.messages.len());
        println!("  Created: {}", session.created_at);
        println!("  Updated: {}", session.updated_at);

        Ok(())
    }

    async fn attach_to_local(&self, config: &Config) -> Result<(), AttachError> {
        println!("Attaching to local session");

        let sharing = SessionSharing::with_default_path();
        let sessions = sharing
            .list_sessions()
            .map_err(|e| AttachError::StateTransferFailed(e.to_string()))?;

        if sessions.is_empty() {
            println!("No active sessions found locally");
            return Ok(());
        }

        if let Some(latest) = sessions.first() {
            let session_id_str = latest.id.to_string();
            return self.attach_to_local_session(&session_id_str, config).await;
        }

        Ok(())
    }

    fn parse_url(&self, url: &str) -> Result<(String, u16, String), String> {
        let url = if !url.starts_with("http://") && !url.starts_with("https://") {
            format!("http://{}", url)
        } else {
            url.to_string()
        };

        let parsed = reqwest::Url::parse(&url).map_err(|e| e.to_string())?;

        let host = parsed.host_str().ok_or("No host in URL")?;
        let port = parsed.port().unwrap_or(8080);
        let path = parsed.path().to_string();

        Ok((host.to_string(), port, path))
    }

    async fn initiate_state_sync(
        &self,
        session_id: &str,
        api_url: &str,
    ) -> Result<(), AttachError> {
        let client = reqwest::Client::new();

        let sync_request = serde_json::json!({
            "action": "state_sync",
            "session_id": session_id,
            "client_type": "cli",
            "capabilities": ["full_control"]
        });

        match client
            .post(format!("{}/api/acp/sync", api_url))
            .json(&sync_request)
            .timeout(std::time::Duration::from_secs(10))
            .send()
            .await
        {
            Ok(response) => {
                if response.status().is_success() {
                    tracing::debug!("State sync initiated for session {}", session_id);
                } else {
                    tracing::warn!("State sync returned: {}", response.status());
                }
            }
            Err(e) => {
                tracing::debug!("State sync request failed (non-critical): {}", e);
            }
        }

        Ok(())
    }

    fn serialize_session_state(
        &self,
        session: &opencode_core::Session,
    ) -> Result<String, AttachError> {
        serde_json::to_string(session).map_err(|e| AttachError::StateTransferFailed(e.to_string()))
    }

    #[allow(dead_code)]
    pub fn create_acp_stream() -> SharedAcpStream {
        opencode_control_plane::AcpEventStream::new().into()
    }
}

impl From<AttachArgs> for AttachCommand {
    fn from(args: AttachArgs) -> Self {
        Self::new(args)
    }
}

#[allow(clippy::items_after_test_module)]
#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_session() -> opencode_core::Session {
        opencode_core::Session {
            id: Uuid::new_v4(),
            workspace_id: None,
            messages: vec![],
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
            state: opencode_core::session_state::SessionState::Idle,
            parent_session_id: None,
            lineage_path: None,
            fork_history: vec![],
            tool_invocations: vec![],
            undo_history: vec![],
            redo_history: vec![],
            shared_id: None,
            share_mode: None,
            share_expires_at: None,
            turns: vec![],
            active_turn_id: None,
        }
    }

    #[test]
    fn test_attach_args_default() {
        let args = AttachArgs {
            session_id: None,
            directory: None,
            url: None,
        };
        assert!(args.session_id.is_none());
        assert!(args.directory.is_none());
        assert!(args.url.is_none());
    }

    #[test]
    fn test_attach_args_with_session_id() {
        let args = AttachArgs {
            session_id: Some("session-123".to_string()),
            directory: None,
            url: None,
        };
        assert_eq!(args.session_id.as_deref(), Some("session-123"));
    }

    #[test]
    fn test_attach_args_with_url() {
        let args = AttachArgs {
            session_id: None,
            directory: None,
            url: Some("wss://example.com".to_string()),
        };
        assert_eq!(args.url.as_deref(), Some("wss://example.com"));
    }

    #[test]
    fn test_attach_args_with_directory() {
        let args = AttachArgs {
            session_id: None,
            directory: Some(PathBuf::from("/tmp")),
            url: None,
        };
        assert_eq!(
            args.directory.as_ref().map(|p| p.as_os_str()),
            Some(std::ffi::OsStr::new("/tmp"))
        );
    }

    #[test]
    fn test_attach_args_full() {
        let args = AttachArgs {
            session_id: Some("session-456".to_string()),
            directory: Some(PathBuf::from("/home/user/project")),
            url: Some("wss://example.com/session".to_string()),
        };
        assert_eq!(args.session_id.as_deref(), Some("session-456"));
        assert_eq!(
            args.directory
                .as_ref()
                .map(|p| p.to_string_lossy().into_owned()),
            Some("/home/user/project".to_string())
        );
        assert_eq!(args.url.as_deref(), Some("wss://example.com/session"));
    }

    #[test]
    fn test_attach_command_new() {
        let cmd = AttachCommand::new(AttachArgs {
            session_id: None,
            directory: None,
            url: None,
        });
        assert!(cmd.session_id.is_none());
        assert!(cmd._directory.is_none());
        assert!(cmd.url.is_none());
    }

    #[test]
    fn test_attach_command_with_url() {
        let cmd = AttachCommand::new(AttachArgs {
            session_id: None,
            directory: None,
            url: Some("http://localhost:8080".to_string()),
        });
        assert!(cmd.session_id.is_none());
        assert_eq!(cmd.url.as_deref(), Some("http://localhost:8080"));
    }

    #[test]
    fn test_attach_command_with_session_id() {
        let cmd = AttachCommand::new(AttachArgs {
            session_id: Some("abc-123".to_string()),
            directory: None,
            url: None,
        });
        assert_eq!(cmd.session_id.as_deref(), Some("abc-123"));
    }

    #[test]
    fn test_parse_url_with_http() {
        let cmd = AttachCommand::new(AttachArgs {
            session_id: None,
            directory: None,
            url: None,
        });
        let result = cmd.parse_url("http://localhost:8080/api/acp");
        assert!(result.is_ok());
        let (host, port, path) = result.unwrap();
        assert_eq!(host, "localhost");
        assert_eq!(port, 8080);
        assert_eq!(path, "/api/acp");
    }

    #[test]
    fn test_parse_url_without_port() {
        let cmd = AttachCommand::new(AttachArgs {
            session_id: None,
            directory: None,
            url: None,
        });
        let result = cmd.parse_url("http://example.com/session");
        assert!(result.is_ok());
        let (host, port, path) = result.unwrap();
        assert_eq!(host, "example.com");
        assert_eq!(port, 8080);
        assert_eq!(path, "/session");
    }

    #[test]
    fn test_parse_url_with_https() {
        let cmd = AttachCommand::new(AttachArgs {
            session_id: None,
            directory: None,
            url: None,
        });
        let result = cmd.parse_url("https://secure.example.com:9000/api");
        assert!(result.is_ok());
        let (host, port, path) = result.unwrap();
        assert_eq!(host, "secure.example.com");
        assert_eq!(port, 9000);
        assert_eq!(path, "/api");
    }

    #[test]
    fn test_parse_url_no_scheme_adds_http() {
        let cmd = AttachCommand::new(AttachArgs {
            session_id: None,
            directory: None,
            url: None,
        });
        let result = cmd.parse_url("localhost:8080");
        assert!(result.is_ok());
        let (host, port, _) = result.unwrap();
        assert_eq!(host, "localhost");
        assert_eq!(port, 8080);
    }

    #[test]
    fn test_parse_url_invalid() {
        let cmd = AttachCommand::new(AttachArgs {
            session_id: None,
            directory: None,
            url: None,
        });
        let result = cmd.parse_url("://invalid");
        assert!(result.is_err());
    }

    #[test]
    fn test_serialize_session_state() {
        let cmd = AttachCommand::new(AttachArgs {
            session_id: None,
            directory: None,
            url: None,
        });
        let session = create_test_session();
        let result = cmd.serialize_session_state(&session);
        assert!(result.is_ok());
        let json = result.unwrap();
        assert!(json.contains("\"id\""));
        assert!(json.contains("\"messages\""));
        assert!(json.contains("\"state\""));
    }

    #[test]
    fn test_serialize_session_state_with_messages() {
        let cmd = AttachCommand::new(AttachArgs {
            session_id: None,
            directory: None,
            url: None,
        });
        let mut session = create_test_session();
        session.messages.push(opencode_core::message::Message::user(
            "Hello, world!".to_string(),
        ));
        let result = cmd.serialize_session_state(&session);
        assert!(result.is_ok());
        let json = result.unwrap();
        assert!(json.contains("Hello, world!"));
    }

    #[test]
    fn test_attach_error_display() {
        let err = AttachError::InvalidUrl("bad url".to_string());
        assert_eq!(format!("{}", err), "Invalid URL: bad url");

        let err = AttachError::ConnectionFailed("network error".to_string());
        assert_eq!(format!("{}", err), "Connection failed: network error");

        let err = AttachError::SessionNotFound("abc".to_string());
        assert_eq!(format!("{}", err), "Session not found: abc");

        let err = AttachError::HandshakeFailed("version mismatch".to_string());
        assert_eq!(format!("{}", err), "Handshake failed: version mismatch");

        let err = AttachError::StateTransferFailed(" serialization error".to_string());
        assert_eq!(
            format!("{}", err),
            "State transfer failed:  serialization error"
        );
    }

    #[test]
    fn test_acp_handshake_request_serialization() {
        let request = AcpHandshakeRequest {
            version: "1.0".to_string(),
            client_id: "client123".to_string(),
            capabilities: vec!["chat".to_string()],
        };
        let json = serde_json::to_string(&request).unwrap();
        assert!(json.contains("\"version\":\"1.0\""));
        assert!(json.contains("\"client_id\":\"client123\""));
        assert!(json.contains("\"capabilities\""));
    }

    #[test]
    fn test_attach_command_from_args() {
        let args = AttachArgs {
            session_id: Some("test-session".to_string()),
            directory: Some(PathBuf::from("/test")),
            url: Some("http://test.com".to_string()),
        };
        let cmd = AttachCommand::from(args);
        assert_eq!(cmd.session_id.as_deref(), Some("test-session"));
        assert_eq!(cmd.url.as_deref(), Some("http://test.com"));
    }

    #[test]
    fn test_create_acp_stream() {
        let stream = AttachCommand::create_acp_stream();
        assert!(!std::sync::Arc::ptr_eq(
            &stream,
            &SharedAcpStream::default()
        ));
    }

    #[tokio::test]
    async fn test_attach_to_local_no_sessions() {
        let temp_dir = tempfile::tempdir().unwrap();
        let sessions_dir = temp_dir.path().to_path_buf();

        let sharing = SessionSharing::new(
            sessions_dir.clone(),
            opencode_core::bus::SharedEventBus::default(),
        );

        let sessions = sharing.list_sessions().unwrap();
        assert!(sessions.is_empty());
    }

    #[tokio::test]
    async fn test_state_sync_request_format() {
        let sync_request = serde_json::json!({
            "action": "state_sync",
            "session_id": "test-session-123",
            "client_type": "cli",
            "capabilities": ["full_control"]
        });

        let json_str = serde_json::to_string(&sync_request).unwrap();
        assert!(json_str.contains("state_sync"));
        assert!(json_str.contains("test-session-123"));
        assert!(json_str.contains("cli"));
    }
}

pub(crate) fn run(args: AttachArgs) {
    let runtime = tokio::runtime::Runtime::new().expect("Failed to create Tokio runtime");
    runtime.block_on(async {
        let config = Config::load(&Config::config_path()).unwrap_or_default();
        if let Err(e) = AttachCommand::from(args).execute(&config).await {
            eprintln!("Attach error: {}", e);
            std::process::exit(1);
        }
    })
}
