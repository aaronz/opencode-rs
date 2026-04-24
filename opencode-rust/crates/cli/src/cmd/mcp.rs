use clap::{Args, Subcommand};
use serde_json::json;

use crate::cmd::mcp_auth::{self, McpAuthArgs};

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
    Install {
        name: String,
        command: String,
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
    fn test_mcp_action_install() {
        let action = McpAction::Install {
            name: "my-mcp-server".to_string(),
            command: "npx".to_string(),
        };
        assert!(matches!(action, McpAction::Install { .. }));
    }
}

pub(crate) fn run(args: McpArgs) {
    if args.json {
        let action_str = match &args.action {
            McpAction::Auth(_) => "auth",
            McpAction::List => "list",
            McpAction::Install { .. } => "install",
            McpAction::Remove { .. } => "remove",
        };
        let result = json!({
            "action": action_str,
            "servers": []
        });
        println!(
            "{}",
            serde_json::to_string_pretty(&result).expect("failed to serialize JSON output")
        );
        return;
    }

    match args.action {
        McpAction::Auth(auth_args) => mcp_auth::run(auth_args),
        _ => println!("MCP action: {:?}", args.action),
    }
}
