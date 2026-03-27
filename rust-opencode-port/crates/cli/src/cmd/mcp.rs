use clap::{Args, Subcommand};

#[derive(Args, Debug)]
pub struct McpArgs {
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
    println!("MCP action: {:?}", args.action);
}
