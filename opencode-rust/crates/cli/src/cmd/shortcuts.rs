use clap::{Args, Subcommand};

#[derive(Args, Debug)]
pub struct ShortcutsArgs {
    #[command(subcommand)]
    pub action: Option<ShortcutsAction>,
}

#[derive(Subcommand, Debug)]
pub enum ShortcutsAction {
    #[command(about = "List all keyboard shortcuts")]
    List {
        #[arg(long)]
        json: bool,
    },

    #[command(about = "Set a keyboard shortcut")]
    Set {
        #[arg(short, long)]
        command: String,

        #[arg(short, long)]
        shortcut: String,
    },

    #[command(about = "Reset a keyboard shortcut")]
    Reset {
        #[arg(short, long)]
        command: Option<String>,
    },

    #[command(about = "Execute a keyboard shortcut")]
    Exec {
        #[arg(short, long)]
        shortcut: String,
    },
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_shortcuts_args_no_action() {
        let args = ShortcutsArgs { action: None };
        assert!(args.action.is_none());
    }

    #[test]
    fn test_shortcuts_args_with_action() {
        let args = ShortcutsArgs {
            action: Some(ShortcutsAction::List { json: false }),
        };
        match args.action {
            Some(ShortcutsAction::List { json }) => assert!(!json),
            _ => panic!("Expected List"),
        }
    }

    #[test]
    fn test_shortcuts_action_list_fields() {
        let action = ShortcutsAction::List { json: true };
        assert!(matches!(action, ShortcutsAction::List { .. }));
    }

    #[test]
    fn test_shortcuts_action_list_no_json() {
        let action = ShortcutsAction::List { json: false };
        match action {
            ShortcutsAction::List { json } => assert!(!json),
            _ => panic!("Expected List"),
        }
    }

    #[test]
    fn test_shortcuts_action_set_fields() {
        let action = ShortcutsAction::Set {
            command: "palette.open".to_string(),
            shortcut: "Ctrl+Shift+P".to_string(),
        };
        assert!(matches!(action, ShortcutsAction::Set { .. }));
    }

    #[test]
    fn test_shortcuts_action_set_various_commands() {
        let action = ShortcutsAction::Set {
            command: "session.new".to_string(),
            shortcut: "Ctrl+N".to_string(),
        };
        match action {
            ShortcutsAction::Set { command, shortcut } => {
                assert_eq!(command, "session.new");
                assert_eq!(shortcut, "Ctrl+N");
            }
            _ => panic!("Expected Set"),
        }
    }

    #[test]
    fn test_shortcuts_action_reset_with_command() {
        let action = ShortcutsAction::Reset {
            command: Some("palette.open".to_string()),
        };
        match action {
            ShortcutsAction::Reset { command } => {
                assert_eq!(command.as_deref(), Some("palette.open"));
            }
            _ => panic!("Expected Reset"),
        }
    }

    #[test]
    fn test_shortcuts_action_reset_without_command() {
        let action = ShortcutsAction::Reset { command: None };
        match action {
            ShortcutsAction::Reset { command } => assert!(command.is_none()),
            _ => panic!("Expected Reset"),
        }
    }

    #[test]
    fn test_shortcuts_action_exec() {
        let action = ShortcutsAction::Exec {
            shortcut: "Ctrl+Shift+P".to_string(),
        };
        match action {
            ShortcutsAction::Exec { shortcut } => {
                assert_eq!(shortcut, "Ctrl+Shift+P");
            }
            _ => panic!("Expected Exec"),
        }
    }
}

pub fn run(args: ShortcutsArgs) {
    match args.action {
        Some(ShortcutsAction::List { json }) => {
            if json {
                let shortcuts = vec![
                    serde_json::json!({"command": "palette.open", "shortcut": "Ctrl+Shift+P"}),
                    serde_json::json!({"command": "session.list", "shortcut": "Ctrl+L"}),
                    serde_json::json!({"command": "models.list", "shortcut": "Ctrl+M"}),
                ];
                println!(
                    "{}",
                    serde_json::to_string(&shortcuts).expect("failed to serialize JSON output")
                );
            } else {
                println!("Keyboard Shortcuts:");
                println!("  Ctrl+Shift+P - Open command palette");
                println!("  Ctrl+L       - List sessions");
                println!("  Ctrl+M       - List models");
            }
        }
        Some(ShortcutsAction::Set { command, shortcut }) => {
            println!("Setting shortcut {} for command {}", shortcut, command);
        }
        Some(ShortcutsAction::Reset { command }) => match command {
            Some(cmd) => {
                println!("Resetting shortcut for command {}", cmd);
            }
            None => {
                println!("Resetting all shortcuts");
            }
        },
        Some(ShortcutsAction::Exec { shortcut }) => {
            println!("Executing shortcut: {}", shortcut);
        }
        None => {
            println!("Usage: opencode-rs shortcuts <action>");
            println!("Actions: list, set, reset, exec");
        }
    }
}
