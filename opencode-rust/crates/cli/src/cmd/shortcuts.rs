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
