use chrono::Utc;
use clap::{Args, Subcommand};
use std::path::PathBuf;
use std::sync::Arc;

use opencode_acp::AcpClient;
use opencode_core::acp::AcpHandshakeResponse;
use opencode_core::bus::{EventBus, SharedEventBus};
use opencode_core::config::{AcpConfig, AcpSession, Config, ServerConfig};

use crate::cmd::load_config;

#[derive(Args, Debug)]
pub(crate) struct AcpArgs {
    #[command(subcommand)]
    pub action: Option<AcpAction>,

    #[arg(long)]
    pub json: bool,

    #[arg(long)]
    pub server: Option<String>,
}

#[derive(Subcommand, Debug)]
pub(crate) enum AcpAction {
    Start,
    Connect {
        #[arg(long)]
        url: String,
    },
    Handshake {
        #[arg(long)]
        client_id: String,

        #[arg(long, default_value = "1.0")]
        version: String,

        #[arg(long)]
        capabilities: Vec<String>,
    },
    Status,
    Ack {
        #[arg(long)]
        handshake_id: String,

        #[arg(long, num_args = 1..=1)]
        accepted: String,
    },
}

const DEFAULT_SESSION_EXPIRY_SECS: i64 = 3600;

fn get_config_path() -> PathBuf {
    if let Ok(config_dir) = std::env::var("OPENCODE_CONFIG_DIR") {
        if !config_dir.contains("opencode-rs") {
            tracing::warn!(
                "OPENCODE_CONFIG_DIR is set - consider using OPENCODE_RS_CONFIG_DIR instead"
            );
        }
        return PathBuf::from(config_dir).join("config.json");
    }
    if let Ok(env_dir) = std::env::var("OPENCODE_RS_CONFIG_DIR") {
        return PathBuf::from(env_dir).join("config.json");
    }
    if let Some(dirs) = directories::ProjectDirs::from("ai", "opencode", "opencode-rs") {
        return dirs.config_dir().join("config.json");
    }
    PathBuf::from("~/.config/opencode-rs/config.json")
}

fn save_config(config: &Config) -> Result<(), String> {
    let path = get_config_path();
    let content = serde_json::to_string_pretty(config)
        .map_err(|e| format!("Failed to serialize config: {}", e))?;
    std::fs::write(&path, content)
        .map_err(|e| format!("Failed to write config to {}: {}", path.display(), e))?;
    Ok(())
}

fn store_acp_session(
    config: &mut Config,
    server_url: &str,
    client_id: &str,
    version: &str,
    capabilities: &[String],
    response: &AcpHandshakeResponse,
) -> Result<(), String> {
    let session = AcpSession {
        session_id: response.session_id.clone(),
        server_id: response.server_id.clone(),
        server_url: server_url.to_string(),
        client_id: client_id.to_string(),
        version: version.to_string(),
        capabilities: capabilities.to_vec(),
        created_at: Utc::now(),
        expires_at: Some(Utc::now() + chrono::Duration::seconds(DEFAULT_SESSION_EXPIRY_SECS)),
    };

    let server_config = config.server.get_or_insert_with(ServerConfig::default);
    let acp_config = server_config.acp.get_or_insert_with(AcpConfig::default);
    acp_config.session = Some(session);

    save_config(config)
}

fn get_stored_session(config: &Config) -> Option<AcpSession> {
    config.server.as_ref()?.acp.as_ref()?.session.clone()
}

fn clear_stored_session(config: &mut Config) {
    if let Some(server_config) = &mut config.server {
        if let Some(acp_config) = &mut server_config.acp {
            acp_config.session = None;
        }
    }
}

fn is_session_expired(session: &AcpSession) -> bool {
    if let Some(expires_at) = session.expires_at {
        Utc::now() > expires_at
    } else {
        false
    }
}

#[allow(clippy::items_after_test_module)]
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_acp_args_default() {
        let args = AcpArgs {
            action: None,
            json: false,
            server: None,
        };
        assert!(args.action.is_none());
        assert!(!args.json);
        assert!(args.server.is_none());
    }

    #[test]
    fn test_acp_args_with_json() {
        let args = AcpArgs {
            action: None,
            json: true,
            server: None,
        };
        assert!(args.json);
    }

    #[test]
    fn test_acp_args_with_server() {
        let args = AcpArgs {
            action: None,
            json: false,
            server: Some("http://localhost:9000".to_string()),
        };
        assert_eq!(args.server.as_deref(), Some("http://localhost:9000"));
    }

    #[test]
    fn test_acp_action_start() {
        let action = AcpAction::Start;
        match action {
            AcpAction::Start => {}
            _ => panic!("Expected Start"),
        }
    }

    #[test]
    fn test_acp_action_connect() {
        let action = AcpAction::Connect {
            url: "http://example.com".to_string(),
        };
        match action {
            AcpAction::Connect { url } => {
                assert_eq!(url, "http://example.com");
            }
            _ => panic!("Expected Connect"),
        }
    }

    #[test]
    fn test_acp_connect_url_parsing() {
        let action = AcpAction::Connect {
            url: "https://acp.server.example.com:8080/api".to_string(),
        };
        match action {
            AcpAction::Connect { url } => {
                assert!(url.starts_with("https://"));
                assert!(url.contains("acp.server.example.com"));
            }
            _ => panic!("Expected Connect"),
        }
    }

    #[test]
    fn test_acp_action_handshake() {
        let action = AcpAction::Handshake {
            client_id: "client123".to_string(),
            version: "1.0".to_string(),
            capabilities: vec!["chat".to_string(), "tasks".to_string()],
        };
        match action {
            AcpAction::Handshake {
                client_id,
                version,
                capabilities,
            } => {
                assert_eq!(client_id, "client123");
                assert_eq!(version, "1.0");
                assert_eq!(capabilities.len(), 2);
            }
            _ => panic!("Expected Handshake"),
        }
    }

    #[test]
    fn test_acp_action_handshake_default_version() {
        let action = AcpAction::Handshake {
            client_id: "client456".to_string(),
            version: "2.0".to_string(),
            capabilities: vec![],
        };
        match action {
            AcpAction::Handshake { version, .. } => assert_eq!(version, "2.0"),
            _ => panic!("Expected Handshake"),
        }
    }

    #[test]
    fn test_acp_action_status() {
        let action = AcpAction::Status;
        match action {
            AcpAction::Status => {}
            _ => panic!("Expected Status"),
        }
    }

    #[test]
    fn test_acp_action_ack() {
        let action = AcpAction::Ack {
            handshake_id: "handshake123".to_string(),
            accepted: "true".to_string(),
        };
        match action {
            AcpAction::Ack {
                handshake_id,
                accepted,
            } => {
                assert_eq!(handshake_id, "handshake123");
                assert_eq!(accepted, "true");
            }
            _ => panic!("Expected Ack"),
        }
    }

    #[test]
    fn test_acp_action_ack_rejected() {
        let action = AcpAction::Ack {
            handshake_id: "handshake456".to_string(),
            accepted: "false".to_string(),
        };
        match action {
            AcpAction::Ack {
                handshake_id,
                accepted,
            } => {
                assert_eq!(handshake_id, "handshake456");
                assert_eq!(accepted, "false");
            }
            _ => panic!("Expected Ack"),
        }
    }

    #[test]
    fn test_acp_args_with_action() {
        let args = AcpArgs {
            action: Some(AcpAction::Start),
            json: false,
            server: None,
        };
        match args.action {
            Some(AcpAction::Start) => {}
            _ => panic!("Expected Start"),
        }
    }

    #[test]
    fn test_acp_session_is_expired() {
        let session = AcpSession {
            expires_at: Some(Utc::now() - chrono::Duration::hours(1)),
            ..Default::default()
        };
        assert!(session.is_expired());
    }

    #[test]
    fn test_acp_session_not_expired() {
        let session = AcpSession {
            expires_at: Some(Utc::now() + chrono::Duration::hours(1)),
            ..Default::default()
        };
        assert!(!session.is_expired());
    }

    #[test]
    fn test_acp_session_no_expiration() {
        let session = AcpSession::default();
        assert!(!session.is_expired());
    }

    #[test]
    fn test_store_and_retrieve_session() {
        let mut config = Config::default();
        let response = AcpHandshakeResponse {
            version: "1.0".to_string(),
            server_id: "server1".to_string(),
            session_id: "session123".to_string(),
            accepted: true,
            error: None,
        };

        let result = store_acp_session(
            &mut config,
            "http://localhost:8080",
            "client1",
            "1.0",
            &["chat".to_string()],
            &response,
        );
        assert!(result.is_ok());

        let stored = get_stored_session(&config);
        assert!(stored.is_some());
        let session = stored.unwrap();
        assert_eq!(session.session_id, "session123");
        assert_eq!(session.server_id, "server1");
        assert_eq!(session.server_url, "http://localhost:8080");
        assert_eq!(session.client_id, "client1");
    }

    #[test]
    fn test_clear_stored_session() {
        let mut config = Config::default();
        let response = AcpHandshakeResponse {
            version: "1.0".to_string(),
            server_id: "server1".to_string(),
            session_id: "session123".to_string(),
            accepted: true,
            error: None,
        };

        store_acp_session(
            &mut config,
            "http://localhost:8080",
            "client1",
            "1.0",
            &[],
            &response,
        )
        .unwrap();

        clear_stored_session(&mut config);
        let stored = get_stored_session(&config);
        assert!(stored.is_none());
    }

    #[test]
    fn test_session_expiration_check() {
        let expired_session = AcpSession {
            session_id: "expired".to_string(),
            server_id: "server".to_string(),
            server_url: "http://localhost:8080".to_string(),
            client_id: "client".to_string(),
            version: "1.0".to_string(),
            capabilities: vec![],
            created_at: Utc::now() - chrono::Duration::hours(2),
            expires_at: Some(Utc::now() - chrono::Duration::hours(1)),
        };
        assert!(is_session_expired(&expired_session));

        let valid_session = AcpSession {
            session_id: "valid".to_string(),
            server_id: "server".to_string(),
            server_url: "http://localhost:8080".to_string(),
            client_id: "client".to_string(),
            version: "1.0".to_string(),
            capabilities: vec![],
            created_at: Utc::now(),
            expires_at: Some(Utc::now() + chrono::Duration::hours(1)),
        };
        assert!(!is_session_expired(&valid_session));
    }

    #[test]
    fn test_acp_handshake_result_persisted() {
        let mut config = Config::default();
        let response = AcpHandshakeResponse {
            version: "1.0".to_string(),
            server_id: "server-test".to_string(),
            session_id: "session-persist-123".to_string(),
            accepted: true,
            error: None,
        };

        let result = store_acp_session(
            &mut config,
            "http://localhost:9090",
            "client-persist",
            "1.0",
            &["chat".to_string(), "tasks".to_string()],
            &response,
        );
        assert!(result.is_ok(), "store_acp_session should succeed");

        let stored = get_stored_session(&config);
        assert!(stored.is_some(), "Session should be stored");
        let session = stored.unwrap();
        assert_eq!(session.session_id, "session-persist-123");
        assert_eq!(session.server_id, "server-test");
        assert_eq!(session.server_url, "http://localhost:9090");
        assert_eq!(session.client_id, "client-persist");
        assert_eq!(session.capabilities, vec!["chat", "tasks"]);
    }

    #[test]
    fn test_acp_handshake_session_restored() {
        let mut config = Config::default();
        let response = AcpHandshakeResponse {
            version: "1.0".to_string(),
            server_id: "server-restore".to_string(),
            session_id: "session-restore-456".to_string(),
            accepted: true,
            error: None,
        };

        store_acp_session(
            &mut config,
            "http://localhost:7070",
            "client-restore",
            "1.0",
            &["code_review".to_string()],
            &response,
        )
        .expect("First handshake should be stored");

        let restored = get_stored_session(&config);
        assert!(
            restored.is_some(),
            "Should be able to retrieve stored session"
        );
        let session = restored.expect("Session should exist");

        assert_eq!(session.session_id, "session-restore-456");
        assert_eq!(session.server_id, "server-restore");
        assert_eq!(session.server_url, "http://localhost:7070");
        assert_eq!(session.client_id, "client-restore");
        assert!(
            !is_session_expired(&session),
            "Restored session should not be expired"
        );
    }
}

pub fn run(args: AcpArgs) {
    let runtime = tokio::runtime::Runtime::new().expect("Failed to create Tokio runtime");
    runtime.block_on(async {
        if let Err(e) = run_acp(args).await {
            eprintln!("ACP error: {}", e);
            std::process::exit(1);
        }
    });
}

async fn run_acp(args: AcpArgs) -> Result<(), Box<dyn std::error::Error>> {
    let server_url = args
        .server
        .unwrap_or_else(|| "http://127.0.0.1:8080".to_string());

    if args.json {
        let result = match &args.action {
            Some(AcpAction::Start) => serde_json::json!({
                "component": "acp",
                "action": "start",
                "status": "ready",
                "version": "1.0"
            }),
            Some(AcpAction::Connect { url }) => serde_json::json!({
                "component": "acp",
                "action": "connect",
                "url": url,
                "status": "connecting"
            }),
            Some(AcpAction::Handshake {
                client_id,
                version,
                capabilities,
            }) => serde_json::json!({
                "component": "acp",
                "action": "handshake",
                "client_id": client_id,
                "version": version,
                "capabilities": capabilities,
                "status": "handshake_initiated"
            }),
            Some(AcpAction::Status) => {
                let status = get_acp_status(&server_url).await.unwrap_or_else(|_| {
                    serde_json::json!({
                        "status": "unavailable",
                        "version": "unknown",
                        "acp_enabled": false
                    })
                });
                println!(
                    "{}",
                    serde_json::to_string_pretty(&status).expect("failed to serialize JSON output")
                );
                return Ok(());
            }
            Some(AcpAction::Ack { .. }) => {
                return Err(
                    "ACK requires active connection. Use 'opencode acp connect' first.".into(),
                );
            }
            None => serde_json::json!({
                "component": "acp",
                "status": "ready"
            }),
        };
        println!(
            "{}",
            serde_json::to_string_pretty(&result).expect("failed to serialize JSON output")
        );
        return Ok(());
    }

    match &args.action {
        Some(AcpAction::Start) => {
            println!("ACP Protocol Manager");
            println!("  Status: Ready");
            println!("  Version: 1.0");
        }
        Some(AcpAction::Connect { url }) => {
            println!("ACP Protocol Manager");
            println!("  Action: Connect");
            println!("  Target URL: {}", url);
            println!("  Status: Connecting...");

            let http = reqwest::Client::new();
            let bus: SharedEventBus = Arc::new(EventBus::new());
            let client = AcpClient::new(http, uuid::Uuid::new_v4().to_string(), bus);

            match client.connect(url, None).await {
                Ok(()) => {
                    println!("  Status: Connected successfully");
                }
                Err(e) => {
                    println!("  Status: Connection failed - {}", e);
                }
            }
        }
        Some(AcpAction::Handshake {
            client_id,
            version,
            capabilities,
        }) => {
            println!("ACP Protocol Manager - Handshake");
            println!("  Client ID: {}", client_id);
            println!("  Version: {}", version);
            println!("  Capabilities: {:?}", capabilities);
            println!("  Status: Handshake initiated");

            let mut config = load_config();

            if let Some(existing_session) = get_stored_session(&config) {
                if !is_session_expired(&existing_session)
                    && existing_session.server_url == server_url
                    && existing_session.client_id == *client_id
                {
                    println!(
                        "  Status: Recovering existing session {}",
                        existing_session.session_id
                    );
                    let verify_url = format!("{}/api/acp/status", server_url);
                    match reqwest::Client::new().get(&verify_url).send().await {
                        Ok(resp) if resp.status().is_success() => {
                            println!("  Session ID: {} (recovered)", existing_session.session_id);
                            println!("  Status: Session recovered successfully!");
                            return Ok(());
                        }
                        Ok(resp) => {
                            println!(
                                "  Warning: Could not verify session - server returned {}",
                                resp.status()
                            );
                        }
                        Err(e) => {
                            println!("  Warning: Could not verify session - {}", e);
                        }
                    }
                } else if is_session_expired(&existing_session) {
                    println!("  Status: Previous session has expired, creating new session");
                    clear_stored_session(&mut config);
                }
            }

            let handshake_url = format!("{}/api/acp/handshake", server_url);
            let body = serde_json::json!({
                "client_id": client_id,
                "version": version,
                "capabilities": capabilities
            });

            match reqwest::Client::new()
                .post(&handshake_url)
                .json(&body)
                .send()
                .await
            {
                Ok(resp) => {
                    if let Ok(response) = resp.json::<serde_json::Value>().await {
                        println!(
                            "  Response: {}",
                            serde_json::to_string_pretty(&response)
                                .expect("failed to serialize JSON output")
                        );
                        if response["accepted"] == true {
                            println!("  Status: Handshake successful!");
                            println!("  Session ID: {}", response["session_id"]);

                            let handshake_response = AcpHandshakeResponse {
                                version: response["version"].as_str().unwrap_or("1.0").to_string(),
                                server_id: response["server_id"].as_str().unwrap_or("").to_string(),
                                session_id: response["session_id"]
                                    .as_str()
                                    .unwrap_or("")
                                    .to_string(),
                                accepted: response["accepted"].as_bool().unwrap_or(false),
                                error: response["error"].as_str().map(String::from),
                            };

                            if let Err(e) = store_acp_session(
                                &mut config,
                                &server_url,
                                client_id,
                                version,
                                capabilities,
                                &handshake_response,
                            ) {
                                println!("  Warning: Failed to store session: {}", e);
                            } else {
                                println!("  Session stored for future reconnection");
                            }
                        } else {
                            println!(
                                "  Status: Handshake failed - {}",
                                response["error"].as_str().unwrap_or("Unknown error")
                            );
                        }
                    }
                }
                Err(e) => {
                    println!("  Status: Handshake failed - {}", e);
                }
            }
        }
        Some(AcpAction::Status) => {
            println!("ACP Protocol Manager - Status");

            let config = load_config();

            if let Some(session) = get_stored_session(&config) {
                let expired = is_session_expired(&session);
                println!("  Stored Session:");
                println!("    Session ID: {}", session.session_id);
                println!("    Server ID: {}", session.server_id);
                println!("    Server URL: {}", session.server_url);
                println!("    Client ID: {}", session.client_id);
                println!("    Expired: {}", if expired { "Yes" } else { "No" });
                if let Some(expires_at) = session.expires_at {
                    let remaining = expires_at - Utc::now();
                    if remaining.num_seconds() > 0 {
                        println!(
                            "    Time until expiration: {} seconds",
                            remaining.num_seconds()
                        );
                    } else {
                        println!("    Time until expiration: expired");
                    }
                }
            } else {
                println!("  No stored session found");
            }

            match get_acp_status(&server_url).await {
                Ok(status) => {
                    let status_val = status["status"].as_str().unwrap_or("unknown");
                    let version_val = status["version"].as_str().unwrap_or("unknown");
                    let acp_enabled = status["acp_enabled"].as_bool().unwrap_or(false);

                    println!("  Server Status:");
                    println!("    Status: {}", status_val);
                    println!("    Version: {}", version_val);
                    println!("    ACP Enabled: {}", acp_enabled);
                }
                Err(e) => {
                    println!("  Server Status: Unavailable");
                    println!("    Error: {}", e);
                }
            }
        }
        Some(AcpAction::Ack {
            handshake_id,
            accepted,
        }) => {
            println!("ACP Protocol Manager - Ack");
            println!("  Handshake ID: {}", handshake_id);
            println!("  Accepted: {}", accepted);
            println!("  Status: Sending acknowledgement...");

            let accepted_bool = accepted.to_lowercase() == "true" || accepted == "1";
            let http = reqwest::Client::new();
            let bus: SharedEventBus = Arc::new(EventBus::new());
            let client = AcpClient::new(http, uuid::Uuid::new_v4().to_string(), bus);

            match client.connect(&server_url, None).await {
                Ok(()) => match client.ack(handshake_id, accepted_bool).await {
                    Ok(()) => {
                        println!("  Status: Acknowledgement sent successfully");
                    }
                    Err(e) => {
                        println!("  Status: Acknowledgement failed - {}", e);
                    }
                },
                Err(e) => {
                    println!("  Status: Connection failed - {}", e);
                }
            }
        }
        None => {
            println!("ACP Protocol Manager");
            println!("  Status: Ready");
            println!("  Use 'opencode acp start' to start the ACP server");
            println!("  Use 'opencode acp connect --url <url>' to connect to a server");
            println!("  Use 'opencode acp handshake' for handshake operations");
            println!("  Use 'opencode acp status --server <url>' to check ACP status");
        }
    }

    Ok(())
}

async fn get_acp_status(server_url: &str) -> Result<serde_json::Value, Box<dyn std::error::Error>> {
    let url = format!("{}/api/acp/status", server_url);
    let resp = reqwest::Client::new().get(&url).send().await?;
    let status = resp.json::<serde_json::Value>().await?;
    Ok(status)
}
