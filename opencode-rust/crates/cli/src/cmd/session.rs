use crate::cmd::workspace::{save_workspace_sessions, SessionInfo, WORKSPACE_SESSIONS};
use clap::{Args, Subcommand};
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Clone, serde::Serialize, serde::Deserialize)]
pub struct SessionMessage {
    pub role: String,
    pub content: String,
}

#[derive(Clone, serde::Serialize, serde::Deserialize)]
pub struct SessionRecord {
    pub id: String,
    pub name: String,
    pub created_at: String,
    pub messages: Vec<SessionMessage>,
    pub redo_history: Vec<SessionMessage>,
}

fn sessions_path() -> PathBuf {
    if let Ok(data_dir) = std::env::var("OPENCODE_DATA_DIR") {
        let path = PathBuf::from(data_dir);
        let _ = std::fs::create_dir_all(&path);
        return path.join("sessions.json");
    }

    directories::ProjectDirs::from("com", "opencode", "rs")
        .map(|dirs| dirs.data_dir().join("sessions.json"))
        .unwrap_or_else(|| PathBuf::from("./sessions.json"))
}

pub fn load_session_records() -> Vec<SessionRecord> {
    let path = sessions_path();
    std::fs::read_to_string(path)
        .ok()
        .and_then(|content| serde_json::from_str::<Vec<SessionRecord>>(&content).ok())
        .unwrap_or_default()
}

pub fn save_session_records(records: &[SessionRecord]) {
    let path = sessions_path();
    if let Some(parent) = path.parent() {
        let _ = std::fs::create_dir_all(parent);
    }
    let serialized = serde_json::to_string_pretty(records).unwrap();
    std::fs::write(path, serialized).unwrap();
}

fn sync_workspace_sessions(records: &[SessionRecord]) {
    let sessions = records
        .iter()
        .map(|record| SessionInfo {
            id: record.id.clone(),
            name: record.name.clone(),
        })
        .collect::<Vec<_>>();
    *WORKSPACE_SESSIONS.lock().unwrap() = sessions.clone();
    save_workspace_sessions(&sessions);
}

fn now_string() -> String {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_secs().to_string())
        .unwrap_or_else(|_| "0".to_string())
}

fn next_session_id(name: Option<&str>) -> String {
    match name {
        Some(name) => format!("session-{}", name),
        None => format!("session-{}", now_string()),
    }
}

pub fn append_session_message(session_id: &str, content: &str) -> bool {
    let mut records = load_session_records();
    if let Some(record) = records.iter_mut().find(|record| record.id == session_id) {
        record.messages.push(SessionMessage {
            role: "user".to_string(),
            content: content.to_string(),
        });
        record.redo_history.clear();
        save_session_records(&records);
        true
    } else {
        false
    }
}

fn create_session(name: Option<String>, json: bool) {
    let mut records = load_session_records();
    let session_name = name.unwrap_or_else(|| "default".to_string());
    let id = next_session_id(Some(&session_name));
    let record = SessionRecord {
        id: id.clone(),
        name: session_name.clone(),
        created_at: now_string(),
        messages: Vec::new(),
        redo_history: Vec::new(),
    };
    records.push(record.clone());
    save_session_records(&records);
    sync_workspace_sessions(&records);

    if json {
        println!("{}", serde_json::to_string(&record).unwrap());
    } else {
        println!("Session ID: {}", id);
        println!("Session created: {}", session_name);
    }
}

fn show_session(session_id: &str, json: bool) {
    let records = load_session_records();
    if let Some(record) = records.iter().find(|record| record.id == session_id) {
        if json {
            println!("{}", serde_json::to_string(record).unwrap());
        } else {
            println!("Session: {}", record.id);
        }
    } else {
        eprintln!("Error: Session '{}' does not exist", session_id);
        std::process::exit(1);
    }
}

fn delete_session(session_id: &str) {
    let mut records = load_session_records();
    let original_len = records.len();
    records.retain(|record| record.id != session_id);
    if records.len() == original_len {
        eprintln!("Error: Session '{}' does not exist", session_id);
        std::process::exit(1);
    }
    save_session_records(&records);
    sync_workspace_sessions(&records);
    println!("Deleted session: {}", session_id);
}

fn undo_session(session_id: &str, steps: usize) {
    let mut records = load_session_records();
    let record = match records.iter_mut().find(|record| record.id == session_id) {
        Some(record) => record,
        None => {
            eprintln!("Error: Session '{}' does not exist", session_id);
            std::process::exit(1);
        }
    };

    if record.messages.is_empty() {
        eprintln!("Nothing to undo");
        std::process::exit(1);
    }

    let count = steps.min(record.messages.len());
    let mut undone = record.messages.split_off(record.messages.len() - count);
    undone.reverse();
    for message in undone {
        record.redo_history.push(message);
    }
    save_session_records(&records);
    println!("Undid {} step(s)", count);
}

fn redo_session(session_id: &str, steps: usize) {
    let mut records = load_session_records();
    let record = match records.iter_mut().find(|record| record.id == session_id) {
        Some(record) => record,
        None => {
            eprintln!("Error: Session '{}' does not exist", session_id);
            std::process::exit(1);
        }
    };

    if record.redo_history.is_empty() {
        eprintln!("Nothing to redo");
        std::process::exit(1);
    }

    let count = steps.min(record.redo_history.len());
    let start = record.redo_history.len() - count;
    let redo_messages = record.redo_history.split_off(start);
    for message in redo_messages {
        record.messages.push(message);
    }
    save_session_records(&records);
    println!("Redid {} step(s)", count);
}

fn fork_session(session_id: &str) {
    let mut records = load_session_records();
    let record = match records.iter().find(|record| record.id == session_id) {
        Some(record) => record.clone(),
        None => {
            eprintln!("Error: Session '{}' does not exist", session_id);
            std::process::exit(1);
        }
    };
    let new_id = format!("{}-fork-{}", session_id, now_string());
    let forked = SessionRecord {
        id: new_id.clone(),
        name: format!("{}-fork", record.name),
        created_at: now_string(),
        messages: record.messages,
        redo_history: Vec::new(),
    };
    records.push(forked);
    save_session_records(&records);
    sync_workspace_sessions(&records);
    println!(
        "{}",
        serde_json::to_string(&serde_json::json!({ "new_id": new_id })).unwrap()
    );
}

fn share_session(session_id: &str) {
    let records = load_session_records();
    if records.iter().any(|record| record.id == session_id) {
        println!(
            "{}",
            serde_json::to_string(&serde_json::json!({
                "share_url": format!("https://example.com/share/{}", session_id),
            }))
            .unwrap()
        );
    } else {
        eprintln!("Error: Session '{}' does not exist", session_id);
        std::process::exit(1);
    }
}

fn export_session(session_id: &str) {
    let records = load_session_records();
    let session = records.iter().find(|r| r.id == session_id);
    match session {
        Some(s) => {
            let output = serde_json::to_string_pretty(s).unwrap();
            println!("{}", output);
        }
        None => {
            eprintln!("Error: Session '{}' does not exist", session_id);
            std::process::exit(1);
        }
    }
}

fn export_all_sessions() {
    let records = load_session_records();
    let export = serde_json::json!({
        "sessions": records,
        "count": records.len(),
    });
    let output = serde_json::to_string_pretty(&export).unwrap();
    println!("{}", output);
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
            println!("Session action: {:?}", args.action);
        }
    }
}
