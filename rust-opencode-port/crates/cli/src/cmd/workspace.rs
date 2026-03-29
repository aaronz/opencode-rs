use clap::{Args, Subcommand};
use once_cell::sync::Lazy;
use std::sync::Mutex;

pub static WORKSPACE_SESSIONS: Lazy<Mutex<Vec<SessionInfo>>> = Lazy::new(|| Mutex::new(Vec::new()));

#[derive(Clone, serde::Serialize, serde::Deserialize)]
pub struct SessionInfo {
    pub id: String,
    pub name: String,
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
            let sessions = WORKSPACE_SESSIONS.lock().unwrap();
            if json {
                let output: Vec<_> = sessions
                    .iter()
                    .map(|s| serde_json::json!({"id": s.id, "name": s.name}))
                    .collect();
                println!("{}", serde_json::to_string(&output).unwrap());
            } else {
                println!("Sessions:");
                for s in sessions.iter() {
                    println!("  {} - {}", s.id, s.name);
                }
            }
        }
        Some(WorkspaceAction::Status { json }) => {
            use crate::cmd::project::PROJECT_STATE;
            let state = PROJECT_STATE.lock().unwrap();
            let sessions = WORKSPACE_SESSIONS.lock().unwrap();
            if json {
                let status = serde_json::json!({
                    "project": state.current_project.as_deref().unwrap_or("none"),
                    "sessions": sessions.len(),
                    "active": true
                });
                println!("{}", serde_json::to_string(&status).unwrap());
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
                println!("{}", serde_json::to_string(&context).unwrap());
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
