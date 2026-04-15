use clap::{Args, Subcommand};
use opencode_core::Config;

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

pub fn run(args: QuickArgs) {
    match args.action {
        Some(QuickAction::NewSession { name }) => {
            let sharing = crate::cmd::session::get_session_sharing_for_quick();
            let session = sharing
                .create_session(Some(name.clone()))
                .expect("Failed to create session");
            crate::cmd::session::save_session_records(&[]);
            println!("Quick session created: {} ({})", name, session.id);
            crate::cmd::session::save_session_records(&[]);
            println!("Quick session created: {} ({})", name, session.id);
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
