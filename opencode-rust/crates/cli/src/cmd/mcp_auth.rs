#![allow(clippy::option_as_ref_deref, clippy::if_same_then_else)]

use chrono::Utc;
use clap::{Args, Subcommand};
use opencode_core::config::{Config, McpConfig, McpOAuthUnion};
use opencode_mcp::auth::{McpAuthTokenStore, McpOAuthToken};

#[derive(Args, Debug)]
pub(crate) struct McpAuthArgs {
    #[command(subcommand)]
    pub action: McpAuthAction,
}

#[derive(Subcommand, Debug)]
pub(crate) enum McpAuthAction {
    #[command(about = "List all MCP server tokens")]
    List,

    #[command(about = "Login to an MCP server using OAuth")]
    Login {
        #[arg(help = "Name of the MCP server")]
        server: String,

        #[arg(long, help = "OAuth client ID")]
        client_id: Option<String>,

        #[arg(long, help = "OAuth client secret")]
        client_secret: Option<String>,

        #[arg(long, help = "OAuth scopes (space-separated)")]
        scope: Option<String>,
    },

    #[command(about = "Logout from an MCP server (remove token)")]
    Logout {
        #[arg(help = "Name of the MCP server")]
        server: String,
    },

    #[command(about = "Check authentication status for an MCP server")]
    Status {
        #[arg(help = "Name of the MCP server")]
        server: String,
    },

    #[command(about = "Remove all expired tokens")]
    Cleanup,
}

#[allow(clippy::items_after_test_module)]
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mcp_auth_args_list() {
        let args = McpAuthArgs {
            action: McpAuthAction::List,
        };
        assert!(matches!(args.action, McpAuthAction::List));
    }

    #[test]
    fn test_mcp_auth_action_login_fields() {
        let action = McpAuthAction::Login {
            server: "my-server".to_string(),
            client_id: None,
            client_secret: None,
            scope: None,
        };
        assert!(matches!(action, McpAuthAction::Login { .. }));
    }
}

pub(crate) fn run(args: McpAuthArgs) {
    match args.action {
        McpAuthAction::List => run_list(),
        McpAuthAction::Login {
            server,
            client_id,
            client_secret,
            scope,
        } => run_login(
            &server,
            client_id.as_deref(),
            client_secret.as_deref(),
            scope.as_deref(),
        ),
        McpAuthAction::Logout { server } => run_logout(&server),
        McpAuthAction::Status { server } => run_status(&server),
        McpAuthAction::Cleanup => run_cleanup(),
    }
}

fn get_token_store() -> McpAuthTokenStore {
    McpAuthTokenStore::from_default_location()
}

fn run_list() {
    let mut store = get_token_store();
    if let Err(e) = store.load() {
        eprintln!("Failed to load token store: {}", e);
        std::process::exit(1);
    }

    let tokens = store.list_tokens();
    if tokens.is_empty() {
        println!("No MCP server tokens stored.");
        return;
    }

    println!("Stored MCP server tokens:");
    println!("{:<20} {:<20} {:<15} Scope", "Server", "Type", "Expires");
    println!("{}", "-".repeat(70));

    for token in tokens {
        let token: &McpOAuthToken = token;
        let expiry_status: String = if token.is_expired() {
            "EXPIRED".to_string()
        } else if token.expires_soon(300) {
            let elapsed = Utc::now().signed_duration_since(token.received_at);
            let remaining = (token.expires_in as i64) - elapsed.num_seconds();
            format!("in {}s", remaining)
        } else {
            format!("in {}s", token.expires_in)
        };

        println!(
            "{:<20} {:<20} {:<15} {}",
            token.server_name,
            token.token_type,
            expiry_status,
            token.scope.as_deref().unwrap_or("-")
        );
    }
}

fn run_login(
    server: &str,
    client_id: Option<&str>,
    client_secret: Option<&str>,
    scope: Option<&str>,
) {
    let config = Config::load(&Config::config_path()).unwrap_or_default();

    let mcp_config = config.mcp.as_ref().and_then(|servers| servers.get(server));

    let (oauth_config, server_url) = match mcp_config {
        Some(McpConfig::Remote(remote)) => {
            let oauth = match &remote.oauth {
                Some(McpOAuthUnion::Config(config)) => config.clone(),
                Some(McpOAuthUnion::Disabled(false)) => {
                    eprintln!(
                        "OAuth is disabled for server '{}'. Enable it in config first.",
                        server
                    );
                    std::process::exit(1);
                }
                Some(McpOAuthUnion::Disabled(true)) => {
                    eprintln!(
                        "OAuth is explicitly disabled (true) for server '{}'. Set to a config object to enable.",
                        server
                    );
                    std::process::exit(1);
                }
                None => {
                    eprintln!("No OAuth configuration found for server '{}'. Add oauth config to the server entry.", server);
                    std::process::exit(1);
                }
            };
            (oauth, Some(remote.url.clone()))
        }
        Some(McpConfig::Local(_)) => {
            eprintln!(
                "Server '{}' is a local server. OAuth is only for remote servers.",
                server
            );
            std::process::exit(1);
        }
        Some(McpConfig::Simple { .. }) => {
            eprintln!(
                "Server '{}' has a simple config. OAuth requires a remote server config.",
                server
            );
            std::process::exit(1);
        }
        None => {
            eprintln!("Server '{}' not found in configuration.", server);
            eprintln!("Add the server to your config first with OAuth enabled.");
            std::process::exit(1);
        }
    };

    let client_id = client_id.or(oauth_config.client_id.as_deref());
    let client_secret = client_secret.or(oauth_config.client_secret.as_deref());
    let scope = scope.or(oauth_config.scope.as_deref());

    let (client_id, client_secret) = match (client_id, client_secret) {
        (Some(cid), Some(csec)) => (cid.to_string(), csec.to_string()),
        _ => {
            eprintln!("OAuth client_id and client_secret are required.");
            eprintln!("Provide them via --client-id and --client-secret, or configure them in the server's oauth config.");
            std::process::exit(1);
        }
    };

    println!("Starting OAuth flow for MCP server '{}'...", server);
    println!(
        "Server URL: {}",
        server_url.as_ref().map(|s| s.as_str()).unwrap_or("N/A")
    );

    let mut store = get_token_store();
    if let Err(e) = store.load() {
        eprintln!("Failed to load token store: {}", e);
        std::process::exit(1);
    }

    if store.is_server_oauth_enabled(server) {
        println!("Warning: Token already exists for server '{}'. Use 'mcp auth logout' first to replace.", server);
    }

    let device_code_url = format!(
        "{}/oauth/device/code",
        server_url
            .as_ref()
            .expect("server_url should be available for remote OAuth")
    );
    let token_url = format!(
        "{}/oauth/token",
        server_url
            .as_ref()
            .expect("server_url should be available for remote OAuth")
    );

    let oauth_flow = opencode_auth::oauth::OAuthFlow::new();

    let session =
        match oauth_flow.start_device_code_flow(server, &client_id, &device_code_url, scope) {
            Ok(s) => s,
            Err(e) => {
                eprintln!("Failed to start device code flow: {}", e);
                std::process::exit(1);
            }
        };

    println!();
    println!("Device Authorization Required:");
    println!("1. Visit: {}", session.verification_uri);
    if let Some(ref _complete) = session.verification_uri_complete {
        println!("2. Enter code: {}", session.user_code);
    } else {
        println!("2. Enter user code: {}", session.user_code);
    }
    println!();
    println!("Waiting for authorization...");

    let token = match oauth_flow.poll_device_code_authorization(
        &session,
        &client_id,
        &client_secret,
        &token_url,
        Some(&|_s| {
            print!(".");
            let _ = std::io::Write::flush(&mut std::io::stdout());
        }),
    ) {
        Ok(t) => t,
        Err(e) => {
            eprintln!();
            eprintln!("Authorization failed: {}", e);
            std::process::exit(1);
        }
    };

    let mcp_token = McpOAuthToken {
        server_name: server.to_string(),
        access_token: token.access_token,
        refresh_token: token.refresh_token,
        token_type: token.token_type,
        expires_in: token.expires_in,
        scope: token.scope,
        received_at: token.received_at,
    };

    if let Err(e) = store.store_token(mcp_token) {
        eprintln!("Failed to store token: {}", e);
        std::process::exit(1);
    }

    println!();
    println!("Successfully authenticated with server '{}'.", server);
}

fn run_logout(server: &str) {
    let mut store = get_token_store();
    if let Err(e) = store.load() {
        eprintln!("Failed to load token store: {}", e);
        std::process::exit(1);
    }

    if !store.is_server_oauth_enabled(server) {
        println!("No token found for server '{}'.", server);
        return;
    }

    if let Err(e) = store.remove_token(server) {
        eprintln!("Failed to remove token: {}", e);
        std::process::exit(1);
    }

    println!("Logged out from server '{}'.", server);
}

fn run_status(server: &str) {
    let mut store = get_token_store();
    if let Err(e) = store.load() {
        eprintln!("Failed to load token store: {}", e);
        std::process::exit(1);
    }

    let config = Config::load(&Config::config_path()).unwrap_or_default();
    let mcp_config = config.mcp.as_ref().and_then(|servers| servers.get(server));

    match mcp_config {
        Some(McpConfig::Remote(remote)) => {
            let has_oauth_config = remote
                .oauth
                .as_ref()
                .map(|o| !matches!(o, McpOAuthUnion::Disabled(false)))
                .unwrap_or(false);

            println!("Server: {}", server);
            println!("Type: Remote");
            println!("URL: {}", remote.url);
            println!("OAuth configured: {}", has_oauth_config);
        }
        Some(McpConfig::Local(_)) => {
            println!("Server: {}", server);
            println!("Type: Local");
            println!("OAuth: Not applicable (local servers don't use OAuth)");
        }
        Some(McpConfig::Simple { .. }) => {
            println!("Server: {}", server);
            println!("Type: Simple config");
            println!("OAuth: Not applicable (simple configs don't support OAuth)");
        }
        None => {
            println!("Server '{}' not found in configuration.", server);
        }
    }

    match store.get_token(server) {
        Some(token) => {
            println!();
            println!("Token status:");
            println!("  Type: {}", token.token_type);
            if token.is_expired() {
                println!("  Status: EXPIRED");
            } else if token.expires_soon(300) {
                let elapsed = Utc::now().signed_duration_since(token.received_at);
                let remaining = (token.expires_in as i64) - elapsed.num_seconds();
                println!("  Status: Active (expires in {}s)", remaining);
            } else {
                let elapsed = Utc::now().signed_duration_since(token.received_at);
                let remaining = (token.expires_in as i64) - elapsed.num_seconds();
                println!("  Status: Active (expires in {}s)", remaining);
            }
            println!("  Scope: {}", token.scope.as_deref().unwrap_or("-"));
            println!("  Received at: {}", token.received_at);
        }
        None => {
            println!();
            println!("Token status: Not authenticated");
        }
    }
}

fn run_cleanup() {
    let mut store = get_token_store();
    if let Err(e) = store.load() {
        eprintln!("Failed to load token store: {}", e);
        std::process::exit(1);
    }

    match store.cleanup_expired() {
        Ok(ref expired) => {
            if expired.is_empty() {
                println!("No expired tokens to remove.");
            } else {
                println!("Removed {} expired token(s):", expired.len());
                for name in expired {
                    println!("  - {}", name);
                }
            }
        }
        Err(e) => {
            eprintln!("Failed to cleanup expired tokens: {}", e);
            std::process::exit(1);
        }
    }
}
