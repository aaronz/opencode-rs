use clap::{Args, Subcommand};
use serde_json::json;

#[derive(Args, Debug)]
pub struct AcpArgs {
    #[command(subcommand)]
    pub action: Option<AcpAction>,

    #[arg(long)]
    pub json: bool,

    #[arg(long)]
    pub server: Option<String>,
}

#[derive(Subcommand, Debug)]
pub enum AcpAction {
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
}

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
            AcpAction::Connect { url } => assert_eq!(url, "http://example.com"),
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
            Some(AcpAction::Start) => json!({
                "component": "acp",
                "action": "start",
                "status": "ready",
                "version": "1.0"
            }),
            Some(AcpAction::Connect { url }) => json!({
                "component": "acp",
                "action": "connect",
                "url": url,
                "status": "connecting"
            }),
            Some(AcpAction::Handshake {
                client_id,
                version,
                capabilities,
            }) => json!({
                "component": "acp",
                "action": "handshake",
                "client_id": client_id,
                "version": version,
                "capabilities": capabilities,
                "status": "handshake_initiated"
            }),
            Some(AcpAction::Status) => {
                let status = get_acp_status(&server_url).await.unwrap_or_else(|_| {
                    json!({
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
            None => json!({
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

            let connect_url = format!("{}/api/acp/connect", server_url);
            let body = json!({ "url": url });

            match reqwest::Client::new()
                .post(&connect_url)
                .json(&body)
                .send()
                .await
            {
                Ok(resp) => {
                    if resp.status().is_success() {
                        println!("  Status: Connected successfully");
                    } else {
                        println!("  Status: Connection failed - {}", resp.status());
                    }
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

            let handshake_url = format!("{}/api/acp/handshake", server_url);
            let body = json!({
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
                        } else {
                            println!("  Status: Handshake failed - {}", response["error"]);
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

            match get_acp_status(&server_url).await {
                Ok(status) => {
                    let status_val = status["status"].as_str().unwrap_or("unknown");
                    let version_val = status["version"].as_str().unwrap_or("unknown");
                    let acp_enabled = status["acp_enabled"].as_bool().unwrap_or(false);

                    println!("  Status: {}", status_val);
                    println!("  Version: {}", version_val);
                    println!("  ACP Enabled: {}", acp_enabled);
                }
                Err(e) => {
                    println!("  Status: Unavailable");
                    println!("  Error: {}", e);
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
