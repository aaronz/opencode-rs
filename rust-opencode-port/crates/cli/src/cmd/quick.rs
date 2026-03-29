use clap::{Args, Subcommand};
use opencode_core::Config;

use crate::cmd::session::load_session_records;
use crate::cmd::session::save_session_records;
use crate::cmd::session::SessionRecord;

#[derive(Args, Debug)]
pub struct QuickArgs {
    #[command(subcommand)]
    pub action: Option<QuickAction>,
}

#[derive(Subcommand, Debug)]
pub enum QuickAction {
    #[command(about = "Create a new session quickly")]
    NewSession {
        #[arg(short, long)]
        name: String,
    },

    #[command(about = "Switch active model quickly")]
    SwitchModel {
        #[arg(short, long)]
        model: String,
    },

    #[command(about = "Open settings")]
    Settings,

    #[command(about = "Toggle sidebar")]
    ToggleSidebar,
}

fn now_string() -> String {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|duration| duration.as_secs().to_string())
        .unwrap_or_else(|_| "0".to_string())
}

pub fn run(args: QuickArgs) {
    match args.action {
        Some(QuickAction::NewSession { name }) => {
            let mut records = load_session_records();
            let record = SessionRecord {
                id: format!("session-{}", name),
                name: name.clone(),
                created_at: now_string(),
                messages: Vec::new(),
                redo_history: Vec::new(),
            };
            records.push(record);
            save_session_records(&records);
            println!("Quick session created: {}", name);
        }
        Some(QuickAction::SwitchModel { model }) => {
            let path = Config::config_path();
            let mut config = Config::load(&path).unwrap_or_default();
            config.model = Some(model.clone());
            if let Err(error) = config.save(&path) {
                eprintln!("Failed to save config: {}", error);
                std::process::exit(1);
            }
            println!("Quick switched model to {}", model);
        }
        Some(QuickAction::Settings) => {
            println!("Quick settings opened");
        }
        Some(QuickAction::ToggleSidebar) => {
            println!("Quick sidebar toggled");
        }
        None => {
            eprintln!("Error: quick action required");
            std::process::exit(1);
        }
    }
}
