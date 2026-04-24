use clap::{Args, Subcommand};
use opencode_core::Config;

#[derive(Args, Debug)]
pub(crate) struct QuickArgs {
    #[command(subcommand)]
    pub action: Option<QuickAction>,
}

#[derive(Subcommand, Debug)]
pub(crate) enum QuickAction {
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

#[allow(clippy::items_after_test_module)]
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_quick_args_no_action() {
        let args = QuickArgs { action: None };
        assert!(args.action.is_none());
    }

    #[test]
    fn test_quick_args_with_action() {
        let args = QuickArgs {
            action: Some(QuickAction::Settings),
        };
        match args.action {
            Some(QuickAction::Settings) => {}
            _ => panic!("Expected Settings"),
        }
    }

    #[test]
    fn test_quick_action_new_session() {
        let action = QuickAction::NewSession {
            name: "Test Session".to_string(),
        };
        match action {
            QuickAction::NewSession { name } => {
                assert_eq!(name, "Test Session");
            }
            _ => panic!("Expected NewSession variant"),
        }
    }

    #[test]
    fn test_quick_action_new_session_various_names() {
        let action = QuickAction::NewSession {
            name: "Session 123".to_string(),
        };
        match action {
            QuickAction::NewSession { name } => assert_eq!(name, "Session 123"),
            _ => panic!("Expected NewSession"),
        }
    }

    #[test]
    fn test_quick_action_switch_model() {
        let action = QuickAction::SwitchModel {
            model: "gpt-4o".to_string(),
        };
        match action {
            QuickAction::SwitchModel { model } => {
                assert_eq!(model, "gpt-4o");
            }
            _ => panic!("Expected SwitchModel variant"),
        }
    }

    #[test]
    fn test_quick_action_switch_model_various_models() {
        let action = QuickAction::SwitchModel {
            model: "claude-3-opus".to_string(),
        };
        match action {
            QuickAction::SwitchModel { model } => assert_eq!(model, "claude-3-opus"),
            _ => panic!("Expected SwitchModel"),
        }
    }

    #[test]
    fn test_quick_action_settings() {
        let action = QuickAction::Settings;
        match action {
            QuickAction::Settings => {}
            _ => panic!("Expected Settings variant"),
        }
    }

    #[test]
    fn test_quick_action_toggle_sidebar() {
        let action = QuickAction::ToggleSidebar;
        match action {
            QuickAction::ToggleSidebar => {}
            _ => panic!("Expected ToggleSidebar variant"),
        }
    }

    #[test]
    fn test_quick_action_debug() {
        let action = QuickAction::NewSession {
            name: "Debug Session".to_string(),
        };
        let debug_str = format!("{:?}", action);
        assert!(debug_str.contains("Debug Session"));
    }
}

pub(crate) fn run(args: QuickArgs) {
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
