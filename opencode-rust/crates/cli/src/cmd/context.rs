use clap::{Args, Subcommand};
use opencode_core::session_sharing::SessionSharing;
use uuid::Uuid;

#[derive(Args, Debug)]
pub(crate) struct ContextArgs {
    #[command(subcommand)]
    pub command: ContextCommand,
}

#[derive(Subcommand, Debug)]
pub(crate) enum ContextCommand {
    /// Inspect the current context bundle
    Inspect {
        /// Session ID to inspect (defaults to current session)
        #[arg(long)]
        session_id: Option<String>,
    },
    /// Explain why specific items are in context
    Explain {
        /// Session ID to explain (defaults to current session)
        #[arg(long)]
        session_id: Option<String>,
    },
    /// Dump context for a specific turn
    Dump {
        /// Turn ID to dump
        #[arg(long)]
        turn_id: String,
        /// Session ID (defaults to current session)
        #[arg(long)]
        session_id: Option<String>,
    },
    /// Explain why a file is or isn't in context
    Why {
        /// File path to check
        #[arg(long)]
        file: String,
        /// Session ID (defaults to current session)
        #[arg(long)]
        session_id: Option<String>,
    },
}

pub fn run_context_command(args: ContextArgs) -> Result<(), String> {
    match args.command {
        ContextCommand::Inspect { session_id } => inspect_context(session_id),
        ContextCommand::Explain { session_id } => explain_context(session_id),
        ContextCommand::Dump {
            turn_id,
            session_id,
        } => dump_context(&turn_id, session_id),
        ContextCommand::Why { file, session_id } => why_file(&file, session_id),
    }
}

fn load_session(session_id: Option<String>) -> Result<Option<opencode_core::Session>, String> {
    let sharing = SessionSharing::with_default_path();
    let session_uuid = match session_id {
        Some(id) => Uuid::parse_str(&id).map_err(|e| format!("Invalid session ID: {}", e))?,
        None => {
            let sessions = sharing.list_sessions().map_err(|e| format!("Failed to list sessions: {}", e))?;
            match sessions.into_iter().next() {
                Some(info) => info.id,
                None => return Ok(None),
            }
        }
    };
    sharing.get_session(&session_uuid)
        .map_err(|e| format!("Failed to load session: {}", e))
        .map(Some)
}

fn inspect_context(session_id: Option<String>) -> Result<(), String> {
    let session = load_session(session_id)?;

    println!("=== Context Inspection ===\n");

    match session {
        Some(session) => {
            println!("Session ID: {}", session.id);
            println!("Created: {}", session.created_at);
            println!("Updated: {}\n", session.updated_at);

            println!("Messages: {} total\n", session.messages.len());
            for (i, msg) in session.messages.iter().enumerate() {
                let preview = if msg.content.len() > 50 {
                    format!("{}...", &msg.content[..50])
                } else {
                    msg.content.clone()
                };
                println!("  [{}] {:?}: {}", i, msg.role, preview);
            }

            println!("\nContext Layers:");
            println!("  L0 (Explicit Input): User-provided input");
            println!("  L1 (Session): Conversation history");
            println!("  L2 (Project): Referenced files, git context");
            println!("  L3 (Structured): Skills, rules, AGENTS.md");
            println!("  L4 (Compressed): Compressed from prior sessions");

            println!("\nToken Budget Allocation:");
            println!("  L0: 15% of main context");
            println!("  L1: 40% of main context");
            println!("  L2: 20% of main context");
            println!("  L3: 15% of main context");
            println!("  L4: 10% of main context");
        }
        None => {
            println!("No session found. Create a session first with 'opencode session new'");
        }
    }

    Ok(())
}

fn explain_context(session_id: Option<String>) -> Result<(), String> {
    let session = load_session(session_id)?;

    println!("=== Context Explanation ===\n");

    println!("How Context Items Are Selected:\n");

    println!("1. LAYER PRIORITY (highest to lowest):");
    println!("   L0 Explicit Input > L1 Session > L2 Project > L3 Structured > L4 Compressed");
    println!("   - Higher layers are preserved when budget is tight\n");

    println!("2. CONTEXT RANKING (used for message trimming):");
    println!("   - Recency (40%): Recent messages are more important");
    println!("   - Relevance (30%): Messages relevant to current task");
    println!("   - Importance (30%): Mathing based on content type\n");

    println!("3. PRESERVE RECENT:");
    println!("   - Last 3 messages are always preserved");
    println!("   - System message is always preserved\n");

    println!("4. FILE SELECTION:");
    println!("   - Explicitly opened files in editor");
    println!("   - Files referenced in conversation");
    println!("   - Git-modified files");
    println!("   - Files matching patterns from AGENTS.md\n");

    if let Some(session) = session {
        println!("Current Session Stats:");
        println!("  Messages: {}", session.messages.len());
        println!("  Turns: {}", session.turns.len());
    }

    Ok(())
}

fn dump_context(turn_id: &str, session_id: Option<String>) -> Result<(), String> {
    let session = load_session(session_id)?;

    println!("=== Context Dump ===\n");
    println!("Turn ID: {}\n", turn_id);

    match session {
        Some(session) => {
            println!("Session: {}", session.id);

            let turn = session.turns.iter().find(|t| t.id.0.to_string() == turn_id);
            match turn {
                Some(turn) => {
                    println!("Turn Status: {:?}", turn.status);
                    println!("Started: {}", turn.started_at);
                    if let Some(completed) = turn.completed_at {
                        println!("Completed: {}", completed);
                    }
                }
                None => {
                    println!("Turn not found. Available turns:");
                    for t in &session.turns {
                        println!("  - {}", t.id.0);
                    }
                }
            }

            println!("\nMessages:");
            for msg in &session.messages {
                println!("  {:?}: {} chars", msg.role, msg.content.len());
            }
        }
        None => {
            println!("No session found.");
        }
    }

    Ok(())
}

fn why_file(file: &str, session_id: Option<String>) -> Result<(), String> {
    let session = load_session(session_id)?;

    println!("=== File Context Analysis ===\n");
    println!("File: {}\n", file);

    println!("Inclusion Signals:");
    println!("  - Explicit mention in user input");
    println!("  - Open/recent file in editor");
    println!("  - Git modified file");
    println!("  - Test failure reference");
    println!("  - Symbol/dependency proximity\n");

    println!("Exclusion Reasons:");
    println!("  - Token budget exceeded");
    println!("  - Low relevance score");
    println!("  - File type excluded by rules");
    println!("  - Not referenced in conversation\n");

    if let Some(session) = session {
        let file_mentioned = session.messages.iter()
            .any(|m| m.content.contains(file));

        if file_mentioned {
            println!("This file IS mentioned in the conversation.");
        } else {
            println!("This file is NOT explicitly mentioned in the conversation.");
            println!("It may be included via:");
            println!("  - Editor open state");
            println!("  - Git context");
            println!("  - Project rules");
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_context_command_variants() {
        use ContextCommand::*;
        let _ = Inspect { session_id: None };
        let _ = Explain { session_id: None };
        let _ = Dump {
            turn_id: "t1".to_string(),
            session_id: None,
        };
        let _ = Why {
            file: "test.rs".to_string(),
            session_id: None,
        };
    }
}
