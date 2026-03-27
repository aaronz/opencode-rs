use clap::{Args, Subcommand};
use serde_json::json;

#[derive(Args, Debug)]
pub struct McpArgs {
    #[arg(long)]
    pub json: bool,

    #[command(subcommand)]
    pub action: McpAction,
}

#[derive(Subcommand, Debug)]
pub enum McpAction {
    List,
    Install { name: String, command: String },
    Remove { name: String },
}

pub fn run(args: McpArgs) {
    if args.json {
        let action_str = match &args.action {
            McpAction::List => "list",
            McpAction::Install { .. } => "install",
            McpAction::Remove { .. } => "remove",
        };
        let result = json!({
            "action": action_str,
            "servers": []
        });
        println!("{}", serde_json::to_string_pretty(&result).unwrap());
        return;
    }

    println!("MCP action: {:?}", args.action);
}
