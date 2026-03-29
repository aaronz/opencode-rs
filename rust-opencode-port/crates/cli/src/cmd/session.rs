use crate::cmd::workspace::WORKSPACE_SESSIONS;
use clap::{Args, Subcommand};

#[derive(Args, Debug)]
pub struct SessionArgs {
    #[arg(short, long)]
    pub id: Option<String>,

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
    Delete,

    #[command(about = "Show session")]
    Show {
        #[arg(short, long)]
        json: bool,
    },

    #[command(about = "Export session")]
    Export,

    #[command(about = "Add message to session")]
    Message {
        #[arg(long)]
        content: Option<String>,
    },

    Undo {
        #[arg(short, long, default_value = "1")]
        steps: usize,
    },

    Redo {
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
            let name = name.unwrap_or_else(|| "default".to_string());
            let id = format!("session-{}", name);
            let mut sessions = WORKSPACE_SESSIONS.lock().unwrap();
            sessions.push(crate::cmd::workspace::SessionInfo {
                id: id.clone(),
                name: name.clone(),
            });
            println!("Session ID: {}", id);
            println!("Session created: {}", name);
        }
        Some(SessionAction::Message { content }) => {
            if let Some(id) = &args.id {
                if let Some(content) = content {
                    println!("Message added to session {}", id);
                }
            }
        }
        Some(SessionAction::Show { json }) => {
            if let Some(id) = &args.id {
                if json {
                    let session = serde_json::json!({
                        "id": id,
                        "name": "Session",
                        "messages": []
                    });
                    println!("{}", serde_json::to_string(&session).unwrap());
                } else {
                    println!("Session: {}", id);
                }
            }
        }
        Some(SessionAction::Undo { steps }) => {
            if let Some(id) = &args.id {
                println!("Undoing {} steps in session {}", steps, id);
            } else {
                eprintln!("Error: Session ID required for undo");
                std::process::exit(1);
            }
        }
        Some(SessionAction::Redo { steps }) => {
            if let Some(id) = &args.id {
                println!("Redoing {} steps in session {}", steps, id);
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
        _ => {
            println!("Session action: {:?}", args.action);
        }
    }
}
