use clap::{Args, Subcommand};
use once_cell::sync::Lazy;
use std::path::PathBuf;
use std::sync::Mutex;

pub static WORKSPACE_SESSIONS: Lazy<Mutex<Vec<SessionInfo>>> = Lazy::new(|| Mutex::new(Vec::new()));

#[derive(Clone, serde::Serialize, serde::Deserialize)]
pub struct SessionInfo {
    pub id: String,
    pub name: String,
}

fn workspace_sessions_path() -> PathBuf {
    if let Ok(data_dir) = std::env::var("OPENCODE_DATA_DIR") {
        let path = PathBuf::from(data_dir);
        let _ = std::fs::create_dir_all(&path);
        return path.join("workspace-sessions.json");
    }

    directories::ProjectDirs::from("com", "opencode", "rs")
        .map(|dirs| dirs.data_dir().join("workspace-sessions.json"))
        .unwrap_or_else(|| PathBuf::from("./workspace-sessions.json"))
}

pub fn load_workspace_sessions() -> Vec<SessionInfo> {
    let path = workspace_sessions_path();
    std::fs::read_to_string(path)
        .ok()
        .and_then(|content| serde_json::from_str::<Vec<SessionInfo>>(&content).ok())
        .unwrap_or_default()
}

pub fn save_workspace_sessions(sessions: &[SessionInfo]) {
    let path = workspace_sessions_path();
    if let Some(parent) = path.parent() {
        let _ = std::fs::create_dir_all(parent);
    }
    let serialized =
        serde_json::to_string_pretty(sessions).expect("failed to serialize workspace sessions");
    std::fs::write(&path, serialized).expect("failed to write workspace sessions file");
}

#[derive(Args, Debug)]
pub struct WorkspaceArgs {
    #[command(subcommand)]
    pub action: Option<WorkspaceAction>,
}

#[derive(Subcommand, Debug)]
pub enum WorkspaceAction {
    #[command(about = "List workspace sessions")]
    Sessions {
        #[arg(long)]
        json: bool,
    },

    #[command(about = "Show workspace status")]
    Status {
        #[arg(long)]
        json: bool,
    },

    #[command(about = "Show workspace context")]
    Context {
        #[arg(long)]
        json: bool,
    },
}

pub fn run(args: WorkspaceArgs) {
    match args.action {
        Some(WorkspaceAction::Sessions { json }) => {
            let sessions = load_workspace_sessions();
            if json {
                let output: Vec<_> = sessions
                    .iter()
                    .map(|s| serde_json::json!({"id": s.id, "name": s.name}))
                    .collect();
                println!(
                    "{}",
                    serde_json::to_string(&output).expect("failed to serialize JSON output")
                );
            } else {
                println!("Sessions:");
                for s in sessions.iter() {
                    println!("  {} - {}", s.id, s.name);
                }
            }
        }
        Some(WorkspaceAction::Status { json }) => {
            use crate::cmd::project::load_project_state;
            let state = load_project_state();
            let sessions = load_workspace_sessions();
            if json {
                let status = serde_json::json!({
                    "project": state.current_project.as_deref().unwrap_or("none"),
                    "sessions": sessions.len(),
                    "active": true
                });
                println!(
                    "{}",
                    serde_json::to_string(&status).expect("failed to serialize JSON output")
                );
            } else {
                println!("Workspace Status:");
                println!(
                    "  Project: {}",
                    state.current_project.as_deref().unwrap_or("none")
                );
                println!("  Sessions: {}", sessions.len());
                println!("  Active: true");
            }
        }
        Some(WorkspaceAction::Context { json }) => {
            if json {
                let context = serde_json::json!({
                    "cwd": std::env::current_dir().map(|p| p.to_string_lossy().to_string()).unwrap_or_default(),
                    "files": []
                });
                println!(
                    "{}",
                    serde_json::to_string(&context).expect("failed to serialize JSON output")
                );
            } else {
                println!("Workspace Context:");
                println!(
                    "  CWD: {}",
                    std::env::current_dir()
                        .map(|p| p.to_string_lossy().to_string())
                        .unwrap_or_default()
                );
            }
        }
        None => {
            println!("Usage: opencode-rs workspace <action>");
            println!("Actions: sessions, status, context");
        }
    }
}
