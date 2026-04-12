use crate::cmd::workspace::{
    save_workspace_sessions, SessionInfo as WorkspaceSessionInfo, WORKSPACE_SESSIONS,
};
use clap::{Args, Subcommand};
use opencode_core::message::Message;
use opencode_core::session::SessionInfo;
use opencode_core::session_sharing::SessionSharing;
use opencode_core::Session;
use uuid::Uuid;

#[derive(Clone, serde::Serialize, serde::Deserialize)]
pub struct SessionRecord {
    pub id: String,
    pub name: String,
    pub created_at: String,
    pub messages: Vec<SessionMessage>,
    pub redo_history: Vec<SessionMessage>,
}

#[derive(Clone, serde::Serialize, serde::Deserialize)]
pub struct SessionMessage {
    pub role: String,
    pub content: String,
}

pub fn load_session_records() -> Vec<SessionRecord> {
    let sharing = get_session_sharing();
    let infos = sharing.list_sessions().unwrap_or_default();

    infos
        .into_iter()
        .filter_map(|info| {
            sharing
                .get_session(&info.id)
                .ok()
                .map(|session| SessionRecord {
                    id: info.id.to_string(),
                    name: info.preview.chars().take(30).collect(),
                    created_at: info.created_at.to_rfc3339(),
                    messages: session
                        .messages
                        .iter()
                        .map(|m| SessionMessage {
                            role: format!("{:?}", m.role).to_lowercase(),
                            content: m.content.clone(),
                        })
                        .collect(),
                    redo_history: Vec::new(),
                })
        })
        .collect()
}

pub fn save_session_records(_records: &[SessionRecord]) {
    let sharing = get_session_sharing();
    if let Ok(sessions) = sharing.list_sessions() {
        sync_workspace_sessions_from_infos(&sessions);
    }
}

fn get_session_sharing() -> SessionSharing {
    SessionSharing::with_default_path()
}

pub fn get_session_sharing_for_quick() -> SessionSharing {
    SessionSharing::with_default_path()
}

fn sync_workspace_sessions_from_sharing(sessions: &[Session]) {
    let infos: Vec<WorkspaceSessionInfo> = sessions
        .iter()
        .map(|session| WorkspaceSessionInfo {
            id: session.id.to_string(),
            name: session
                .messages
                .first()
                .map(|m| m.content.chars().take(30).collect())
                .unwrap_or_else(|| "Untitled".to_string()),
        })
        .collect();
    *WORKSPACE_SESSIONS.lock().unwrap() = infos.clone();
    save_workspace_sessions(&infos);
}

fn sync_workspace_sessions_from_infos(infos: &[SessionInfo]) {
    let workspace_infos: Vec<WorkspaceSessionInfo> = infos
        .iter()
        .map(|info| WorkspaceSessionInfo {
            id: info.id.to_string(),
            name: info.preview.chars().take(30).collect(),
        })
        .collect();
    *WORKSPACE_SESSIONS.lock().unwrap() = workspace_infos.clone();
    save_workspace_sessions(&workspace_infos);
}

pub fn append_session_message(session_id: &str, content: &str) -> bool {
    let sharing = get_session_sharing();
    let id = match Uuid::parse_str(session_id) {
        Ok(id) => id,
        Err(_) => return false,
    };

    sharing
        .add_message(&id, Message::user(content.to_string()))
        .is_ok()
}

fn create_session(name: Option<String>, json: bool) {
    let sharing = get_session_sharing();
    let session = sharing
        .create_session(name)
        .expect("Failed to create session");

    sync_workspace_sessions_from_sharing(std::slice::from_ref(&session));

    let name = session
        .messages
        .first()
        .map(|m| m.content.chars().take(30).collect())
        .unwrap_or_else(|| "Untitled".to_string());

    if json {
        println!(
            "{}",
            serde_json::to_string(&serde_json::json!({
                "id": session.id.to_string(),
                "name": name,
                "created_at": session.created_at.to_rfc3339(),
            }))
            .unwrap()
        );
    } else {
        println!("Session ID: {}", session.id);
        println!("Session created: {}", name);
    }
}

fn show_session(session_id: &str, json: bool) {
    let sharing = get_session_sharing();
    let id = match Uuid::parse_str(session_id) {
        Ok(id) => id,
        Err(_) => {
            eprintln!("Error: Invalid session ID format '{}'", session_id);
            std::process::exit(1);
        }
    };

    match sharing.get_session(&id) {
        Ok(session) => {
            if json {
                let output = serde_json::to_string_pretty(&session).unwrap();
                println!("{}", output);
            } else {
                println!("Session: {}", session.id);
                println!("Messages: {}", session.messages.len());
                println!(
                    "Created: {}",
                    session.created_at.format("%Y-%m-%d %H:%M:%S")
                );
                println!(
                    "Updated: {}",
                    session.updated_at.format("%Y-%m-%d %H:%M:%S")
                );
            }
        }
        Err(e) => {
            eprintln!("Error: Session '{}' does not exist: {}", session_id, e);
            std::process::exit(1);
        }
    }
}

fn delete_session(session_id: &str) {
    let sharing = get_session_sharing();
    let id = match Uuid::parse_str(session_id) {
        Ok(id) => id,
        Err(_) => {
            eprintln!("Error: Invalid session ID format '{}'", session_id);
            std::process::exit(1);
        }
    };

    match sharing.delete_session(&id) {
        Ok(_) => {
            let sessions = sharing.list_sessions().unwrap_or_default();
            sync_workspace_sessions_from_infos(&sessions);
            println!("Deleted session: {}", session_id);
        }
        Err(e) => {
            eprintln!("Error: Failed to delete session '{}': {}", session_id, e);
            std::process::exit(1);
        }
    }
}

fn undo_session(session_id: &str, steps: usize) {
    let sharing = get_session_sharing();
    let id = match Uuid::parse_str(session_id) {
        Ok(id) => id,
        Err(_) => {
            eprintln!("Error: Invalid session ID format '{}'", session_id);
            std::process::exit(1);
        }
    };

    let mut session = match sharing.get_session(&id) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("Error: Session '{}' does not exist: {}", session_id, e);
            std::process::exit(1);
        }
    };

    match session.undo(steps) {
        Ok(count) => {
            if let Err(e) = sharing.save_session(&session) {
                eprintln!("Error: Failed to save session: {}", e);
                std::process::exit(1);
            }
            println!("Undid {} step(s)", count);
        }
        Err(e) => {
            eprintln!("Error: {}", e);
            std::process::exit(1);
        }
    }
}

fn redo_session(session_id: &str, steps: usize) {
    let sharing = get_session_sharing();
    let id = match Uuid::parse_str(session_id) {
        Ok(id) => id,
        Err(_) => {
            eprintln!("Error: Invalid session ID format '{}'", session_id);
            std::process::exit(1);
        }
    };

    let mut session = match sharing.get_session(&id) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("Error: Session '{}' does not exist: {}", session_id, e);
            std::process::exit(1);
        }
    };

    match session.redo(steps) {
        Ok(count) => {
            if let Err(e) = sharing.save_session(&session) {
                eprintln!("Error: Failed to save session: {}", e);
                std::process::exit(1);
            }
            println!("Redid {} step(s)", count);
        }
        Err(e) => {
            eprintln!("Error: {}", e);
            std::process::exit(1);
        }
    }
}

fn fork_session(session_id: &str) {
    let sharing = get_session_sharing();
    let id = match Uuid::parse_str(session_id) {
        Ok(id) => id,
        Err(_) => {
            eprintln!("Error: Invalid session ID format '{}'", session_id);
            std::process::exit(1);
        }
    };

    match sharing.fork_session(&id, Uuid::new_v4()) {
        Ok(child) => {
            let sessions = sharing.list_sessions().unwrap_or_default();
            sync_workspace_sessions_from_infos(&sessions);
            println!(
                "{}",
                serde_json::to_string(&serde_json::json!({ "new_id": child.id.to_string() }))
                    .unwrap()
            );
        }
        Err(e) => {
            eprintln!("Error: Failed to fork session '{}': {}", session_id, e);
            std::process::exit(1);
        }
    }
}

fn share_session(session_id: &str) {
    let sharing = get_session_sharing();
    let id = match Uuid::parse_str(session_id) {
        Ok(id) => id,
        Err(_) => {
            eprintln!("Error: Invalid session ID format '{}'", session_id);
            std::process::exit(1);
        }
    };

    let mut session = match sharing.get_session(&id) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("Error: Session '{}' does not exist: {}", session_id, e);
            std::process::exit(1);
        }
    };

    match session.generate_share_link() {
        Ok(url) => {
            println!(
                "{}",
                serde_json::to_string(&serde_json::json!({
                    "share_url": url,
                }))
                .unwrap()
            );
        }
        Err(e) => {
            eprintln!("Error: Failed to share session: {}", e);
            std::process::exit(1);
        }
    }
}

fn export_session(session_id: &str) {
    let sharing = get_session_sharing();
    let id = match Uuid::parse_str(session_id) {
        Ok(id) => id,
        Err(_) => {
            eprintln!("Error: Invalid session ID format '{}'", session_id);
            std::process::exit(1);
        }
    };

    match sharing.get_session(&id) {
        Ok(session) => {
            let output = serde_json::to_string_pretty(&session).unwrap();
            println!("{}", output);
        }
        Err(e) => {
            eprintln!("Error: Session '{}' does not exist: {}", session_id, e);
            std::process::exit(1);
        }
    }
}

fn export_all_sessions() {
    let sharing = get_session_sharing();
    let sessions = sharing.list_sessions().unwrap_or_default();
    let export = serde_json::json!({
        "sessions": sessions,
        "count": sessions.len(),
    });
    let output = serde_json::to_string_pretty(&export).unwrap();
    println!("{}", output);
}

fn list_sessions(json: bool) {
    let sharing = get_session_sharing();
    let sessions = sharing.list_sessions().unwrap_or_default();

    if json {
        let output = serde_json::to_string_pretty(&sessions).unwrap();
        println!("{}", output);
    } else {
        if sessions.is_empty() {
            println!("No sessions found");
            return;
        }
        for session_info in &sessions {
            println!(
                "{} - {} ({} messages, updated {})",
                session_info.id,
                session_info.preview,
                session_info.message_count,
                session_info.updated_at.format("%Y-%m-%d %H:%M")
            );
        }
    }
}

#[derive(Args, Debug)]
pub struct SessionArgs {
    #[arg(short, long)]
    pub id: Option<String>,

    #[arg(long)]
    pub new: bool,

    #[arg(long)]
    pub message: Option<String>,

    #[arg(long)]
    pub fork: bool,

    #[arg(long)]
    pub share: bool,

    #[arg(long)]
    pub json: bool,

    #[command(subcommand)]
    pub action: Option<SessionAction>,
}

#[derive(Subcommand, Debug)]
pub enum SessionAction {
    #[command(about = "Create a new session")]
    Create {
        #[arg(short, long)]
        name: Option<String>,
    },

    #[command(about = "Delete a session")]
    Delete {
        #[arg(short, long)]
        id: Option<String>,
    },

    #[command(about = "Show session")]
    Show {
        #[arg(short, long)]
        id: Option<String>,

        #[arg(short, long)]
        json: bool,
    },

    #[command(about = "List all sessions")]
    List {
        #[arg(short, long)]
        json: bool,
    },

    #[command(about = "Export session")]
    Export,

    #[command(about = "Add message to session")]
    Message {
        #[arg(short, long)]
        id: Option<String>,

        #[arg(long)]
        content: Option<String>,
    },

    Undo {
        #[arg(short, long)]
        id: Option<String>,

        #[arg(short, long, default_value = "1")]
        steps: usize,
    },

    Redo {
        #[arg(short, long)]
        id: Option<String>,

        #[arg(short, long, default_value = "1")]
        steps: usize,
    },

    Review {
        #[arg(short, long)]
        file: Option<String>,
        #[arg(short, long, default_value = "text")]
        format: String,
    },

    Diff {
        #[arg(short, long)]
        file: String,
        #[arg(short, long, default_value = "3")]
        context: usize,
    },
}

pub fn run(args: SessionArgs) {
    match args.action {
        Some(SessionAction::Create { name }) => {
            create_session(name, false);
        }
        Some(SessionAction::Delete { id }) => {
            let id = id.or(args.id);
            if let Some(id) = id.as_deref() {
                delete_session(id);
            } else {
                eprintln!("Error: Session ID required for delete");
                std::process::exit(1);
            }
        }
        Some(SessionAction::Show { id, json }) => {
            let id = id.or(args.id);
            if let Some(id) = id.as_deref() {
                show_session(id, json);
            } else {
                eprintln!("Error: Session ID required for show");
                std::process::exit(1);
            }
        }
        Some(SessionAction::List { json }) => {
            list_sessions(json);
        }
        Some(SessionAction::Message { id, content }) => {
            let id = id.or(args.id);
            if let Some(id) = id.as_deref() {
                if let Some(content) = content {
                    if !append_session_message(id, &content) {
                        eprintln!("Error: Session '{}' does not exist", id);
                        std::process::exit(1);
                    }
                    println!("Message added to session {}", id);
                }
            }
        }
        Some(SessionAction::Undo { id, steps }) => {
            let id = id.or(args.id);
            if let Some(id) = id.as_deref() {
                undo_session(id, steps);
            } else {
                eprintln!("Error: Session ID required for undo");
                std::process::exit(1);
            }
        }
        Some(SessionAction::Redo { id, steps }) => {
            let id = id.or(args.id);
            if let Some(id) = id.as_deref() {
                redo_session(id, steps);
            } else {
                eprintln!("Error: Session ID required for redo");
                std::process::exit(1);
            }
        }
        Some(SessionAction::Review { file, format }) => {
            if let Some(id) = &args.id {
                println!(
                    "Reviewing session {} (file: {:?}, format: {})",
                    id, file, format
                );
            } else {
                eprintln!("Error: Session ID required for review");
                std::process::exit(1);
            }
        }
        Some(SessionAction::Diff { file, context }) => {
            if let Some(id) = &args.id {
                println!(
                    "Showing diff for {} in session {} (context: {})",
                    file, id, context
                );
            } else {
                eprintln!("Error: Session ID required for diff");
                std::process::exit(1);
            }
        }
        Some(SessionAction::Export) => {
            if let Some(id) = &args.id {
                export_session(id);
            } else {
                export_all_sessions();
            }
        }
        None => {
            if args.new {
                create_session(None, args.json);
                return;
            }
            if let Some(id) = &args.id {
                if let Some(message) = args.message {
                    if !append_session_message(id, &message) {
                        eprintln!("Error: Session '{}' does not exist", id);
                        std::process::exit(1);
                    }
                    if args.json {
                        show_session(id, true);
                    } else {
                        println!("Message added to session {}", id);
                    }
                    return;
                }
                if args.fork {
                    fork_session(id);
                    return;
                }
                if args.share {
                    share_session(id);
                    return;
                }
                show_session(id, args.json);
                return;
            }
            list_sessions(args.json);
        }
    }
}
