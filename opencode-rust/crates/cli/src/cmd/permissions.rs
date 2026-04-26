use clap::{Args, Subcommand};
use std::collections::HashSet;
use std::path::PathBuf;
use std::sync::OnceLock;

static PERMISSION_DB: OnceLock<std::sync::RwLock<HashSet<String>>> = OnceLock::new();

fn get_permission_db_path() -> PathBuf {
    let project_dirs = directories::ProjectDirs::from("ai", "opencode", "opencode-rs")
        .expect("Failed to determine project directories");
    let data_dir = project_dirs.data_dir();
    std::fs::create_dir_all(data_dir).expect("Failed to create data directory");
    data_dir.join("permissions.json")
}

fn get_permission_db() -> &'static std::sync::RwLock<HashSet<String>> {
    PERMISSION_DB.get_or_init(|| std::sync::RwLock::new(load_permissions_from_file()))
}

fn load_permissions_from_file() -> HashSet<String> {
    let path = get_permission_db_path();
    if !path.exists() {
        return HashSet::new();
    }
    match std::fs::read_to_string(&path) {
        Ok(content) => serde_json::from_str(&content).unwrap_or_default(),
        Err(_) => HashSet::new(),
    }
}

fn save_permissions_to_file(permissions: &HashSet<String>) -> Result<(), String> {
    let path = get_permission_db_path();
    let content = serde_json::to_string_pretty(permissions)
        .map_err(|e| format!("Failed to serialize: {}", e))?;
    std::fs::write(&path, content).map_err(|e| format!("Failed to write file: {}", e))?;
    Ok(())
}

#[derive(Subcommand, Debug)]
pub enum PermissionsAction {
    #[command(about = "Grant permission for a path")]
    Grant {
        #[arg(value_name = "PATH")]
        path: String,
    },

    #[command(about = "Revoke permission for a path")]
    Revoke {
        #[arg(value_name = "PATH")]
        path: String,
    },

    #[command(about = "List granted permissions")]
    List {
        #[arg(short, long)]
        json: bool,
    },
}

#[derive(Args, Debug)]
pub struct PermissionsArgs {
    #[command(subcommand)]
    pub action: Option<PermissionsAction>,
}

pub fn run(args: PermissionsArgs) {
    match args.action {
        Some(PermissionsAction::Grant { path }) => {
            let mut db = get_permission_db().write().unwrap();
            if db.contains(&path) {
                println!("Permission already granted for: {}", path);
            } else {
                db.insert(path.clone());
                if let Err(e) = save_permissions_to_file(&db) {
                    eprintln!("Failed to save permission: {}", e);
                    std::process::exit(1);
                }
                println!("Permission granted for: {}", path);
            }
        }
        Some(PermissionsAction::Revoke { path }) => {
            let mut db = get_permission_db().write().unwrap();
            if !db.contains(&path) {
                println!("Permission not found for: {}", path);
            } else {
                db.remove(&path);
                if let Err(e) = save_permissions_to_file(&db) {
                    eprintln!("Failed to save permission: {}", e);
                    std::process::exit(1);
                }
                println!("Permission revoked for: {}", path);
            }
        }
        Some(PermissionsAction::List { json }) => {
            let db = get_permission_db().read().unwrap();
            let permissions: Vec<&str> = db.iter().map(|s| s.as_str()).collect();
            if json {
                println!(
                    "{}",
                    serde_json::to_string_pretty(&permissions)
                        .expect("failed to serialize JSON output")
                );
            } else if permissions.is_empty() {
                println!("No permissions granted");
            } else {
                println!("Granted permissions:");
                for perm in &permissions {
                    println!("  {}", perm);
                }
            }
        }
        None => {
            eprintln!("Error: action required. Use 'grant', 'revoke', or 'list'");
            std::process::exit(1);
        }
    }
}
