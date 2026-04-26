use clap::Args;
use serde::{Deserialize, Serialize};

use crate::cmd::session::get_session_sharing_for_quick as get_session_sharing;

#[derive(Args, Debug)]
pub(crate) struct StatsArgs {
    #[arg(short, long)]
    pub json: bool,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct CliStats {
    pub total_sessions: usize,
    pub total_messages: usize,
    pub sessions: Vec<SessionStats>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct SessionStats {
    pub id: String,
    pub message_count: usize,
    pub created_at: String,
    pub updated_at: String,
}

#[allow(clippy::items_after_test_module)]
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_stats_args_default() {
        let args = StatsArgs { json: false };
        assert!(!args.json);
    }

    #[test]
    fn test_stats_args_with_json() {
        let args = StatsArgs { json: true };
        assert!(args.json);
    }

    #[test]
    fn test_cli_stats_serialization() {
        let stats = CliStats {
            total_sessions: 2,
            total_messages: 5,
            sessions: vec![
                SessionStats {
                    id: "session-1".to_string(),
                    message_count: 3,
                    created_at: "2024-01-01T00:00:00Z".to_string(),
                    updated_at: "2024-01-01T00:01:00Z".to_string(),
                },
                SessionStats {
                    id: "session-2".to_string(),
                    message_count: 2,
                    created_at: "2024-01-01T00:02:00Z".to_string(),
                    updated_at: "2024-01-01T00:03:00Z".to_string(),
                },
            ],
        };
        let json = serde_json::to_string(&stats).expect("failed to serialize");
        assert!(json.contains("total_sessions"));
        assert!(json.contains("session-1"));
    }
}

pub(crate) fn run(args: StatsArgs) {
    let sharing = get_session_sharing();
    let sessions = sharing.list_sessions().unwrap_or_default();

    let total_messages: usize = sessions.iter().map(|s| s.message_count).sum();

    let stats = CliStats {
        total_sessions: sessions.len(),
        total_messages,
        sessions: sessions
            .into_iter()
            .map(|s| SessionStats {
                id: s.id.to_string(),
                message_count: s.message_count,
                created_at: s.created_at.to_rfc3339(),
                updated_at: s.updated_at.to_rfc3339(),
            })
            .collect(),
    };

    if args.json {
        println!(
            "{}",
            serde_json::to_string_pretty(&stats).expect("failed to serialize JSON output")
        );
    } else {
        println!("Stats:");
        println!("  Total sessions: {}", stats.total_sessions);
        println!("  Total messages: {}", stats.total_messages);
        if !stats.sessions.is_empty() {
            println!("  Sessions:");
            for session in &stats.sessions {
                println!(
                    "    {} - {} messages (updated {})",
                    session.id, session.message_count, session.updated_at
                );
            }
        }
    }
}
