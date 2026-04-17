use clap::{Args, Subcommand};
use once_cell::sync::Lazy;
use std::path::PathBuf;
use std::sync::Mutex;

pub(crate) static WORKSPACE_SESSIONS: Lazy<Mutex<Vec<SessionInfo>>> =
    Lazy::new(|| Mutex::new(Vec::new()));

#[derive(Clone, serde::Serialize, serde::Deserialize)]
pub(crate) struct SessionInfo {
    pub id: String,
    pub name: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_session_info_creation() {
        let info = SessionInfo {
            id: "session-1".to_string(),
            name: "Test Session".to_string(),
        };
        assert_eq!(info.id, "session-1");
        assert_eq!(info.name, "Test Session");
    }

    #[test]
    fn test_session_info_serialization() {
        let info = SessionInfo {
            id: "sess-123".to_string(),
            name: "My Session".to_string(),
        };
        let json = serde_json::to_string(&info).unwrap();
        assert!(json.contains("sess-123"));
        assert!(json.contains("My Session"));
    }

    #[test]
    fn test_session_info_deserialization() {
        let json = r#"{"id":"id-456","name":"Deserialized"}"#;
        let info: SessionInfo = serde_json::from_str(json).unwrap();
        assert_eq!(info.id, "id-456");
        assert_eq!(info.name, "Deserialized");
    }
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

pub(crate) fn load_workspace_sessions() -> Vec<SessionInfo> {
    let path = workspace_sessions_path();
    std::fs::read_to_string(path)
        .ok()
        .and_then(|content| serde_json::from_str::<Vec<SessionInfo>>(&content).ok())
        .unwrap_or_default()
}

pub(crate) fn save_workspace_sessions(sessions: &[SessionInfo]) {
    let path = workspace_sessions_path();
    if let Some(parent) = path.parent() {
        let _ = std::fs::create_dir_all(parent);
    }
    let serialized =
        serde_json::to_string_pretty(sessions).expect("failed to serialize workspace sessions");
    std::fs::write(&path, serialized).expect("failed to write workspace sessions file");
}

#[derive(Args, Debug)]
pub(crate) struct WorkspaceArgs {
    #[command(subcommand)]
    pub action: Option<WorkspaceAction>,
}

#[derive(Subcommand, Debug)]
pub(crate) enum WorkspaceAction {
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

#[cfg(test)]
mod workspace_tests {
    use super::*;

    #[test]
    fn test_workspace_args_no_action() {
        let args = WorkspaceArgs { action: None };
        assert!(args.action.is_none());
    }

    #[test]
    fn test_workspace_args_with_action() {
        let args = WorkspaceArgs {
            action: Some(WorkspaceAction::Sessions { json: false }),
        };
        match args.action {
            Some(WorkspaceAction::Sessions { json }) => assert!(!json),
            _ => panic!("Expected Sessions"),
        }
    }

    #[test]
    fn test_workspace_action_sessions_fields() {
        let action = WorkspaceAction::Sessions { json: true };
        assert!(matches!(action, WorkspaceAction::Sessions { .. }));
    }

    #[test]
    fn test_workspace_action_sessions_no_json() {
        let action = WorkspaceAction::Sessions { json: false };
        match action {
            WorkspaceAction::Sessions { json } => assert!(!json),
            _ => panic!("Expected Sessions"),
        }
    }

    #[test]
    fn test_workspace_action_status_fields() {
        let action = WorkspaceAction::Status { json: false };
        assert!(matches!(action, WorkspaceAction::Status { .. }));
    }

    #[test]
    fn test_workspace_action_status_with_json() {
        let action = WorkspaceAction::Status { json: true };
        match action {
            WorkspaceAction::Status { json } => assert!(json),
            _ => panic!("Expected Status"),
        }
    }

    #[test]
    fn test_workspace_action_context() {
        let action = WorkspaceAction::Context { json: true };
        match action {
            WorkspaceAction::Context { json } => assert!(json),
            _ => panic!("Expected Context"),
        }
    }

    #[test]
    fn test_workspace_action_context_no_json() {
        let action = WorkspaceAction::Context { json: false };
        match action {
            WorkspaceAction::Context { json } => assert!(!json),
            _ => panic!("Expected Context"),
        }
    }

    #[test]
    fn test_workspace_sessions_path_default() {
        std::env::remove_var("OPENCODE_DATA_DIR");
        let path = workspace_sessions_path();
        assert!(path.to_string_lossy().contains("workspace-sessions.json"));
    }

    #[test]
    fn test_workspace_sessions_path_with_data_dir() {
        let temp_dir = std::env::temp_dir();
        std::env::set_var("OPENCODE_DATA_DIR", temp_dir.to_string_lossy().as_ref());
        let path = workspace_sessions_path();
        assert!(path.to_string_lossy().contains("workspace-sessions.json"));
        std::env::remove_var("OPENCODE_DATA_DIR");
    }

    #[test]
    fn test_load_workspace_sessions_empty() {
        std::env::remove_var("OPENCODE_DATA_DIR");
        let sessions = load_workspace_sessions();
        assert!(sessions.is_empty());
    }
}

pub(crate) fn run(args: WorkspaceArgs) {
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
