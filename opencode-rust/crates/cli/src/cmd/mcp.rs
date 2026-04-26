use clap::{Args, Subcommand};
use serde_json::json;
use which::which;

use crate::cmd::mcp_auth::{self, McpAuthArgs};

use opencode_core::config::{Config, McpConfig, McpLocalConfig};

#[derive(Args, Debug)]
pub(crate) struct McpArgs {
    #[arg(long)]
    pub json: bool,

    #[command(subcommand)]
    pub action: McpAction,
}

#[derive(Subcommand, Debug)]
pub(crate) enum McpAction {
    #[command(about = "Manage MCP server authentication")]
    Auth(McpAuthArgs),

    List,
    Add {
        name: String,
        command: String,
        #[arg(short, long, num_args = 0..)]
        args: Option<Vec<String>>,
    },
    Remove {
        name: String,
    },
}

#[allow(clippy::items_after_test_module)]
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mcp_args_default() {
        let args = McpArgs {
            json: false,
            action: McpAction::List,
        };
        assert!(!args.json);
        assert!(matches!(args.action, McpAction::List));
    }

    #[test]
    fn test_mcp_action_add() {
        let action = McpAction::Add {
            name: "my-mcp-server".to_string(),
            command: "npx".to_string(),
            args: Some(vec!["mcp-server".to_string()]),
        };
        assert!(matches!(action, McpAction::Add { .. }));
        if let McpAction::Add {
            name,
            command,
            args,
        } = action
        {
            assert_eq!(name, "my-mcp-server");
            assert_eq!(command, "npx");
            assert_eq!(args, Some(vec!["mcp-server".to_string()]));
        }
    }

    #[test]
    fn test_mcp_action_add_without_args() {
        let action = McpAction::Add {
            name: "simple-server".to_string(),
            command: "echo".to_string(),
            args: None,
        };
        assert!(
            matches!(action, McpAction::Add { name, command, args } if name == "simple-server" && command == "echo" && args.is_none())
        );
    }

    #[test]
    fn test_mcp_action_remove() {
        let action = McpAction::Remove {
            name: "server-to-remove".to_string(),
        };
        assert!(matches!(action, McpAction::Remove { .. }));
    }
}

pub(crate) fn run(args: McpArgs) {
    if args.json {
        let action_str = match &args.action {
            McpAction::Auth(_) => "auth",
            McpAction::List => "list",
            McpAction::Add { .. } => "add",
            McpAction::Remove { .. } => "remove",
        };
        let servers = list_mcp_servers();
        let result = json!({
            "action": action_str,
            "servers": servers
        });
        println!(
            "{}",
            serde_json::to_string_pretty(&result).expect("failed to serialize JSON output")
        );
        return;
    }

    match args.action {
        McpAction::Auth(auth_args) => mcp_auth::run(auth_args),
        McpAction::List => {
            list_mcp_servers();
        }
        McpAction::Add {
            name,
            command,
            args,
        } => {
            if let Err(e) = add_mcp_server(&name, &command, args.as_deref()) {
                eprintln!("Error adding MCP server: {}", e);
                std::process::exit(1);
            }
        }
        McpAction::Remove { name } => {
            if let Err(e) = remove_mcp_server(&name) {
                eprintln!("Error removing MCP server: {}", e);
                std::process::exit(1);
            }
        }
    }
}

fn add_mcp_server(name: &str, command: &str, args: Option<&[String]>) -> Result<(), String> {
    if which(command).is_err() {
        return Err(format!(
            "Command '{}' not found. Please ensure it is installed and available in PATH.",
            command
        ));
    }

    let path = Config::config_path();
    let mut config = Config::load(&path).unwrap_or_default();

    let mcp_config = McpConfig::Local(McpLocalConfig {
        command: {
            let mut cmd_vec = vec![command.to_string()];
            if let Some(args_vec) = args {
                cmd_vec.extend(args_vec.iter().cloned());
            }
            cmd_vec
        },
        enabled: Some(true),
        ..Default::default()
    });

    config.mcp.get_or_insert_with(Default::default);
    if let Some(ref mut mcp_map) = config.mcp {
        mcp_map.insert(name.to_string(), mcp_config);
    }

    config
        .save(&path)
        .map_err(|e| format!("Failed to save config: {}", e))?;

    println!("Added MCP server '{}' with command '{}'", name, command);
    Ok(())
}

fn list_mcp_servers() -> Vec<serde_json::Value> {
    let path = Config::config_path();
    let config = Config::load(&path).unwrap_or_default();

    let mcp_servers = config.mcp.as_ref();
    let servers: Vec<serde_json::Value> = mcp_servers
        .map(|mcp_map| {
            mcp_map
                .iter()
                .map(|(name, cfg)| {
                    let (command, enabled) = match cfg {
                        McpConfig::Local(local) => {
                            (local.command.join(" "), local.enabled.unwrap_or(true))
                        }
                        McpConfig::Remote(remote) => {
                            (remote.url.clone(), remote.enabled.unwrap_or(true))
                        }
                        McpConfig::Simple { enabled } => ("simple".to_string(), *enabled),
                    };
                    serde_json::json!({
                        "name": name,
                        "command": command,
                        "enabled": enabled
                    })
                })
                .collect()
        })
        .unwrap_or_default();

    if servers.is_empty() {
        println!("No MCP servers configured");
    } else {
        println!("MCP Servers:");
        for server in &servers {
            let name = server
                .get("name")
                .and_then(|v| v.as_str())
                .unwrap_or("unknown");
            let command = server.get("command").and_then(|v| v.as_str()).unwrap_or("");
            let enabled = server
                .get("enabled")
                .and_then(|v| v.as_bool())
                .unwrap_or(true);
            let status = if enabled { "enabled" } else { "disabled" };
            println!("  {} - {} ({})", name, command, status);
        }
    }

    servers
}

fn remove_mcp_server(name: &str) -> Result<(), String> {
    let path = Config::config_path();
    let mut config = Config::load(&path).unwrap_or_default();

    let mcp_map = config
        .mcp
        .as_mut()
        .ok_or_else(|| "No MCP servers configured".to_string())?;

    if !mcp_map.contains_key(name) {
        return Err(format!("MCP server '{}' not found", name));
    }

    mcp_map.remove(name);

    config
        .save(&path)
        .map_err(|e| format!("Failed to save config: {}", e))?;

    println!("Removed MCP server '{}'", name);
    Ok(())
}
