use clap::{Args, Subcommand};

#[derive(Args, Debug)]
pub struct PaletteArgs {
    #[command(subcommand)]
    pub action: Option<PaletteAction>,
}

#[derive(Subcommand, Debug)]
pub enum PaletteAction {
    #[command(about = "Open command palette")]
    Open,

    #[command(about = "Search commands")]
    Search {
        #[arg(short, long)]
        query: Option<String>,

        #[arg(long)]
        json: bool,
    },

    #[command(about = "Execute a command")]
    Execute {
        #[arg(short, long)]
        command: String,
    },

    #[command(about = "Show recent commands")]
    Recent {
        #[arg(long)]
        json: bool,
    },
}

pub fn run(args: PaletteArgs) {
    match args.action {
        Some(PaletteAction::Open) => {
            println!("Command palette opened");
        }
        Some(PaletteAction::Search { query, json }) => {
            if json {
                // Return JSON list of commands
                let commands = vec![
                    serde_json::json!({"name": "session list", "description": "List all sessions"}),
                    serde_json::json!({"name": "models list", "description": "List available models"}),
                    serde_json::json!({"name": "project create", "description": "Create a new project"}),
                ];
                println!("{}", serde_json::to_string(&commands).unwrap());
            } else {
                println!("Searching for: {:?}", query);
                println!("Available commands:");
                println!("  session list - List all sessions");
                println!("  models list - List available models");
                println!("  project create - Create a new project");
            }
        }
        Some(PaletteAction::Execute { command }) => {
            println!("Executing command: {}", command);
        }
        Some(PaletteAction::Recent { json }) => {
            if json {
                let recent = vec![
                    serde_json::json!({"name": "session list", "timestamp": "2024-01-01T00:00:00Z"}),
                    serde_json::json!({"name": "models list", "timestamp": "2024-01-01T00:00:00Z"}),
                ];
                println!("{}", serde_json::to_string(&recent).unwrap());
            } else {
                println!("Recent commands:");
                println!("  session list");
                println!("  models list");
            }
        }
        None => {
            println!("Usage: opencode-rs palette <action>");
            println!("Actions: open, search, execute, recent");
        }
    }
}
