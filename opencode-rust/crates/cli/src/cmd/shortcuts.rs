use clap::{Args, Subcommand};
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::{OnceLock, RwLock};

type ShortcutsMap = HashMap<String, String>;
type ShortcutsLock = RwLock<ShortcutsMap>;

static SHORTCUTS_DB: OnceLock<ShortcutsLock> = OnceLock::new();

fn get_shortcuts_db_path() -> PathBuf {
    let project_dirs = directories::ProjectDirs::from("ai", "opencode", "opencode-rs")
        .expect("Failed to determine project directories");
    let data_dir = project_dirs.data_dir();
    std::fs::create_dir_all(data_dir).expect("Failed to create data directory");
    data_dir.join("shortcuts.json")
}

fn get_shortcuts_db() -> &'static ShortcutsLock {
    SHORTCUTS_DB.get_or_init(|| RwLock::new(load_shortcuts_from_file()))
}

fn load_shortcuts_from_file() -> HashMap<String, String> {
    let path = get_shortcuts_db_path();
    if !path.exists() {
        return HashMap::new();
    }
    match std::fs::read_to_string(&path) {
        Ok(content) => serde_json::from_str(&content).unwrap_or_default(),
        Err(_) => HashMap::new(),
    }
}

fn save_shortcuts_to_file(shortcuts: &HashMap<String, String>) -> Result<(), String> {
    let path = get_shortcuts_db_path();
    let content = serde_json::to_string_pretty(shortcuts)
        .map_err(|e| format!("Failed to serialize: {}", e))?;
    std::fs::write(&path, content).map_err(|e| format!("Failed to write file: {}", e))?;
    Ok(())
}

#[derive(Args, Debug)]
pub(crate) struct ShortcutsArgs {
    #[command(subcommand)]
    pub action: Option<ShortcutsAction>,
}

#[derive(Subcommand, Debug)]
pub(crate) enum ShortcutsAction {
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

#[allow(clippy::items_after_test_module)]
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

pub(crate) fn run(args: ShortcutsArgs) {
    match args.action {
        Some(ShortcutsAction::List { json }) => {
            let db = get_shortcuts_db().read().unwrap();
            if json {
                let shortcuts: Vec<_> = db
                    .iter()
                    .map(|(k, v)| serde_json::json!({"command": k, "shortcut": v}))
                    .collect();
                println!(
                    "{}",
                    serde_json::to_string_pretty(&shortcuts)
                        .expect("failed to serialize JSON output")
                );
            } else if db.is_empty() {
                println!("No shortcuts configured");
                println!("Use 'shortcuts set --command <name> --shortcut <key>' to add shortcuts");
            } else {
                println!("Keyboard Shortcuts:");
                for (command, shortcut) in db.iter() {
                    println!("  {} - {}", shortcut, command);
                }
            }
        }
        Some(ShortcutsAction::Set { command, shortcut }) => {
            let mut db = get_shortcuts_db().write().unwrap();
            db.insert(command.clone(), shortcut.clone());
            if let Err(e) = save_shortcuts_to_file(&db) {
                eprintln!("Failed to save shortcut: {}", e);
                std::process::exit(1);
            }
            println!("Set shortcut {} for command {}", shortcut, command);
        }
        Some(ShortcutsAction::Reset { command }) => {
            let mut db = get_shortcuts_db().write().unwrap();
            match command {
                Some(cmd) => {
                    if db.remove(&cmd).is_some() {
                        if let Err(e) = save_shortcuts_to_file(&db) {
                            eprintln!("Failed to save shortcuts: {}", e);
                            std::process::exit(1);
                        }
                        println!("Reset shortcut for command {}", cmd);
                    } else {
                        println!("No shortcut found for command {}", cmd);
                    }
                }
                None => {
                    db.clear();
                    if let Err(e) = save_shortcuts_to_file(&db) {
                        eprintln!("Failed to save shortcuts: {}", e);
                        std::process::exit(1);
                    }
                    println!("Reset all shortcuts");
                }
            }
        }
        Some(ShortcutsAction::Exec { shortcut }) => {
            let db = get_shortcuts_db().read().unwrap();
            if let Some(command) = db.iter().find(|(_, v)| *v == &shortcut) {
                println!("Executing command: {} (shortcut: {})", command.0, shortcut);
            } else {
                eprintln!("Unknown shortcut: {}", shortcut);
                std::process::exit(1);
            }
        }
        None => {
            println!("Usage: opencode-rs shortcuts <action>");
            println!("Actions: list, set, reset, exec");
        }
    }
}
