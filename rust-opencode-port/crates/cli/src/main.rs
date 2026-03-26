use clap::{Parser, Subcommand};
use std::path::PathBuf;

#[derive(Parser)]
#[command(name = "opencode-rs")]
#[command(version = "0.1.0")]
#[command(about = "AI coding agent in Rust", long_about = None)]
struct Cli {
    #[arg(short, long, value_name = "PATH")]
    config: Option<PathBuf>,

    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    Session {
        #[arg(short, long)]
        id: Option<String>,

        #[command(subcommand)]
        action: Option<SessionAction>,
    },
    List,
}

#[derive(Subcommand, Debug)]
enum SessionAction {
    Delete,
    Show,
}

fn main() {
    tracing_subscriber::fmt::init();

    let cli = Cli::parse();

    if let Some(config_path) = cli.config {
        println!("Using config: {}", config_path.display());
    }

    match &cli.command {
        Some(Commands::Session { id, action }) => {
            println!("Session: {:?}", id);
            if let Some(act) = action {
                println!("Action: {:?}", act);
            }
        }
        Some(Commands::List) => {
            println!("Listing sessions...");
        }
        None => {
            println!("Starting interactive mode...");
        }
    }
}
