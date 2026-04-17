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
pub(crate) struct SessionRecord {
    pub id: String,
    pub name: String,
    pub created_at: String,
    pub messages: Vec<SessionMessage>,
    pub redo_history: Vec<SessionMessage>,
}

#[derive(Clone, serde::Serialize, serde::Deserialize)]
pub(crate) struct SessionMessage {
    pub role: String,
    pub content: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_session_record_creation() {
        let record = SessionRecord {
            id: "test-id".to_string(),
            name: "Test Session".to_string(),
            created_at: "2024-01-01T00:00:00Z".to_string(),
            messages: vec![],
            redo_history: vec![],
        };
        assert_eq!(record.id, "test-id");
        assert_eq!(record.name, "Test Session");
        assert!(record.messages.is_empty());
        assert!(record.redo_history.is_empty());
    }

    #[test]
    fn test_session_message_creation() {
        let msg = SessionMessage {
            role: "user".to_string(),
            content: "Hello".to_string(),
        };
        assert_eq!(msg.role, "user");
        assert_eq!(msg.content, "Hello");
    }

    #[test]
    fn test_session_args_default() {
        let args = SessionArgs {
            id: None,
            new: false,
            message: None,
            fork: false,
            share: false,
            json: false,
            action: None,
        };
        assert!(args.id.is_none());
        assert!(!args.new);
        assert!(args.message.is_none());
        assert!(!args.fork);
        assert!(!args.share);
        assert!(!args.json);
        assert!(args.action.is_none());
    }

    #[test]
    fn test_session_args_with_id() {
        let args = SessionArgs {
            id: Some("session-123".to_string()),
            new: false,
            message: None,
            fork: false,
            share: false,
            json: false,
            action: None,
        };
        assert_eq!(args.id.as_deref(), Some("session-123"));
    }

    #[test]
    fn test_session_args_with_new() {
        let args = SessionArgs {
            id: None,
            new: true,
            message: None,
            fork: false,
            share: false,
            json: false,
            action: None,
        };
        assert!(args.new);
    }

    #[test]
    fn test_session_args_with_message() {
        let args = SessionArgs {
            id: Some("session-123".to_string()),
            new: false,
            message: Some("Hello world".to_string()),
            fork: false,
            share: false,
            json: false,
            action: None,
        };
        assert_eq!(args.message.as_deref(), Some("Hello world"));
    }

    #[test]
    fn test_session_args_with_fork() {
        let args = SessionArgs {
            id: Some("session-123".to_string()),
            new: false,
            message: None,
            fork: true,
            share: false,
            json: false,
            action: None,
        };
        assert!(args.fork);
    }

    #[test]
    fn test_session_args_with_share() {
        let args = SessionArgs {
            id: Some("session-123".to_string()),
            new: false,
            message: None,
            fork: false,
            share: true,
            json: false,
            action: None,
        };
        assert!(args.share);
    }

    #[test]
    fn test_session_args_with_json() {
        let args = SessionArgs {
            id: None,
            new: false,
            message: None,
            fork: false,
            share: false,
            json: true,
            action: None,
        };
        assert!(args.json);
    }

    #[test]
    fn test_session_action_create() {
        let action = SessionAction::Create { name: None };
        match action {
            SessionAction::Create { name } => assert!(name.is_none()),
            _ => panic!("Expected Create"),
        }
    }

    #[test]
    fn test_session_action_create_with_name() {
        let action = SessionAction::Create {
            name: Some("My Session".to_string()),
        };
        match action {
            SessionAction::Create { name } => assert_eq!(name.as_deref(), Some("My Session")),
            _ => panic!("Expected Create"),
        }
    }

    #[test]
    fn test_session_action_delete() {
        let action = SessionAction::Delete {
            id: Some("session-123".to_string()),
        };
        match action {
            SessionAction::Delete { id } => assert_eq!(id.as_deref(), Some("session-123")),
            _ => panic!("Expected Delete"),
        }
    }

    #[test]
    fn test_session_action_delete_no_id() {
        let action = SessionAction::Delete { id: None };
        match action {
            SessionAction::Delete { id } => assert!(id.is_none()),
            _ => panic!("Expected Delete"),
        }
    }

    #[test]
    fn test_session_action_show() {
        let action = SessionAction::Show {
            id: Some("session-123".to_string()),
            json: true,
        };
        match action {
            SessionAction::Show { id, json } => {
                assert_eq!(id.as_deref(), Some("session-123"));
                assert!(json);
            }
            _ => panic!("Expected Show"),
        }
    }

    #[test]
    fn test_session_action_show_no_json() {
        let action = SessionAction::Show {
            id: Some("session-123".to_string()),
            json: false,
        };
        match action {
            SessionAction::Show { id, json } => {
                assert_eq!(id.as_deref(), Some("session-123"));
                assert!(!json);
            }
            _ => panic!("Expected Show"),
        }
    }

    #[test]
    fn test_session_action_list() {
        let action = SessionAction::List { json: true };
        match action {
            SessionAction::List { json } => assert!(json),
            _ => panic!("Expected List"),
        }
    }

    #[test]
    fn test_session_action_export() {
        let action = SessionAction::Export;
        match action {
            SessionAction::Export => {}
            _ => panic!("Expected Export"),
        }
    }

    #[test]
    fn test_session_action_message() {
        let action = SessionAction::Message {
            id: Some("session-123".to_string()),
            content: Some("Hello".to_string()),
        };
        match action {
            SessionAction::Message { id, content } => {
                assert_eq!(id.as_deref(), Some("session-123"));
                assert_eq!(content.as_deref(), Some("Hello"));
            }
            _ => panic!("Expected Message"),
        }
    }

    #[test]
    fn test_session_action_undo() {
        let action = SessionAction::Undo {
            id: Some("session-123".to_string()),
            steps: 5,
        };
        match action {
            SessionAction::Undo { id, steps } => {
                assert_eq!(id.as_deref(), Some("session-123"));
                assert_eq!(steps, 5);
            }
            _ => panic!("Expected Undo"),
        }
    }

    #[test]
    fn test_session_action_undo_default_steps() {
        let action = SessionAction::Undo {
            id: Some("session-123".to_string()),
            steps: 1,
        };
        match action {
            SessionAction::Undo { steps, .. } => assert_eq!(steps, 1),
            _ => panic!("Expected Undo"),
        }
    }

    #[test]
    fn test_session_action_redo() {
        let action = SessionAction::Redo {
            id: Some("session-123".to_string()),
            steps: 3,
        };
        match action {
            SessionAction::Redo { id, steps } => {
                assert_eq!(id.as_deref(), Some("session-123"));
                assert_eq!(steps, 3);
            }
            _ => panic!("Expected Redo"),
        }
    }

    #[test]
    fn test_session_action_review() {
        let action = SessionAction::Review {
            file: Some("test.rs".to_string()),
            format: "markdown".to_string(),
        };
        match action {
            SessionAction::Review { file, format } => {
                assert_eq!(file.as_deref(), Some("test.rs"));
                assert_eq!(format, "markdown");
            }
            _ => panic!("Expected Review"),
        }
    }

    #[test]
    fn test_session_action_review_default_format() {
        let action = SessionAction::Review {
            file: Some("test.rs".to_string()),
            format: "text".to_string(),
        };
        match action {
            SessionAction::Review { format, .. } => assert_eq!(format, "text"),
            _ => panic!("Expected Review"),
        }
    }

    #[test]
    fn test_session_action_diff() {
        let action = SessionAction::Diff {
            file: "Cargo.toml".to_string(),
            context: 5,
        };
        match action {
            SessionAction::Diff { file, context } => {
                assert_eq!(file, "Cargo.toml");
                assert_eq!(context, 5);
            }
            _ => panic!("Expected Diff"),
        }
    }

    #[test]
    fn test_session_action_diff_default_context() {
        let action = SessionAction::Diff {
            file: "lib.rs".to_string(),
            context: 3,
        };
        match action {
            SessionAction::Diff { context, .. } => assert_eq!(context, 3),
            _ => panic!("Expected Diff"),
        }
    }

    #[test]
    fn test_session_message_serialization() {
        let msg = SessionMessage {
            role: "assistant".to_string(),
            content: "Hi there!".to_string(),
        };
        let json = serde_json::to_string(&msg).unwrap();
        assert!(json.contains("assistant"));
        assert!(json.contains("Hi there!"));
    }

    #[test]
    fn test_session_record_serialization() {
        let record = SessionRecord {
            id: "123".to_string(),
            name: "My Session".to_string(),
            created_at: "2024-01-01T00:00:00Z".to_string(),
            messages: vec![SessionMessage {
                role: "user".to_string(),
                content: "Test".to_string(),
            }],
            redo_history: vec![],
        };
        let json = serde_json::to_string(&record).unwrap();
        assert!(json.contains("123"));
        assert!(json.contains("My Session"));
        assert!(json.contains("Test"));
    }

    #[test]
    fn test_session_record_with_multiple_messages() {
        let record = SessionRecord {
            id: "456".to_string(),
            name: "Multi Message".to_string(),
            created_at: "2024-01-01T00:00:00Z".to_string(),
            messages: vec![
                SessionMessage {
                    role: "user".to_string(),
                    content: "First".to_string(),
                },
                SessionMessage {
                    role: "assistant".to_string(),
                    content: "Second".to_string(),
                },
                SessionMessage {
                    role: "user".to_string(),
                    content: "Third".to_string(),
                },
            ],
            redo_history: vec![],
        };
        assert_eq!(record.messages.len(), 3);
        assert_eq!(record.messages[0].content, "First");
        assert_eq!(record.messages[1].content, "Second");
        assert_eq!(record.messages[2].content, "Third");
    }

    #[test]
    fn test_append_session_message_returns_false_for_invalid_uuid() {
        let result = append_session_message("invalid-uuid", "test content");
        assert!(!result);
    }
}

pub(crate) fn load_session_records() -> Vec<SessionRecord> {
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

pub(crate) fn save_session_records(_records: &[SessionRecord]) {
    let sharing = get_session_sharing();
    if let Ok(sessions) = sharing.list_sessions() {
        sync_workspace_sessions_from_infos(&sessions);
    }
}

fn get_session_sharing() -> SessionSharing {
    SessionSharing::with_default_path()
}

pub(crate) fn get_session_sharing_for_quick() -> SessionSharing {
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
    *WORKSPACE_SESSIONS.lock().unwrap_or_else(|p| p.into_inner()) = infos.clone();
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
    *WORKSPACE_SESSIONS.lock().unwrap_or_else(|p| p.into_inner()) = workspace_infos.clone();
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

#[expect(
    clippy::expect_used,
    reason = "CLI entry point where failure should panic"
)]
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
            .expect("failed to serialize JSON output")
        );
    } else {
        println!("Session ID: {}", session.id);
        println!("Session created: {}", name);
    }
}

#[expect(
    clippy::expect_used,
    reason = "CLI entry point where failure should panic"
)]
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
                let output = serde_json::to_string_pretty(&session)
                    .expect("failed to serialize JSON output");
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

#[expect(
    clippy::expect_used,
    reason = "CLI entry point where failure should panic"
)]
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
                    .expect("JSON serialization should not fail for simple object")
            );
        }
        Err(e) => {
            eprintln!("Error: Failed to fork session '{}': {}", session_id, e);
            std::process::exit(1);
        }
    }
}

#[expect(
    clippy::expect_used,
    reason = "CLI entry point where failure should panic"
)]
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
                .expect("JSON serialization should not fail for simple object")
            );
        }
        Err(e) => {
            eprintln!("Error: Failed to share session: {}", e);
            std::process::exit(1);
        }
    }
}

#[expect(
    clippy::expect_used,
    reason = "CLI entry point where failure should panic"
)]
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
            let output =
                serde_json::to_string_pretty(&session).expect("failed to serialize JSON output");
            println!("{}", output);
        }
        Err(e) => {
            eprintln!("Error: Session '{}' does not exist: {}", session_id, e);
            std::process::exit(1);
        }
    }
}

#[expect(
    clippy::expect_used,
    reason = "CLI entry point where failure should panic"
)]
fn export_all_sessions() {
    let sharing = get_session_sharing();
    let sessions = sharing.list_sessions().unwrap_or_default();
    let export = serde_json::json!({
        "sessions": sessions,
        "count": sessions.len(),
    });
    let output = serde_json::to_string_pretty(&export).expect("failed to serialize JSON output");
    println!("{}", output);
}

#[expect(
    clippy::expect_used,
    reason = "CLI entry point where failure should panic"
)]
fn list_sessions(json: bool) {
    let sharing = get_session_sharing();
    let sessions = sharing.list_sessions().unwrap_or_default();

    if json {
        let output =
            serde_json::to_string_pretty(&sessions).expect("failed to serialize JSON output");
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
pub(crate) struct SessionArgs {
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
pub(crate) enum SessionAction {
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

pub(crate) fn run(args: SessionArgs) {
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
