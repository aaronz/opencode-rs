use crate::Session;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionSummary {
    pub session_id: String,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub message_count: usize,
    pub user_messages: usize,
    pub assistant_messages: usize,
    pub tools_used: Vec<String>,
    pub topics: Vec<String>,
    pub key_decisions: Vec<String>,
}

pub struct SummaryGenerator;

impl SummaryGenerator {
    pub fn generate(session: &Session) -> SessionSummary {
        let mut user_messages = 0;
        let mut assistant_messages = 0;
        let mut topics = Vec::new();

        for msg in &session.messages {
            match msg.role {
                crate::message::Role::User => {
                    user_messages += 1;
                    if msg.content.len() > 50 {
                        topics.push(format!("{}...", &msg.content[..50]));
                    }
                }
                crate::message::Role::Assistant => {
                    assistant_messages += 1;
                }
                _ => {}
            }
        }

        SessionSummary {
            session_id: session.id.to_string(),
            created_at: session.created_at,
            message_count: session.messages.len(),
            user_messages,
            assistant_messages,
            tools_used: Vec::new(),
            topics: topics[..topics.len().min(5)].to_vec(),
            key_decisions: Vec::new(),
        }
    }

    pub fn summarize_text(session: &Session) -> String {
        let summary = Self::generate(session);
        format!(
            "Session {}\n\
            Created: {}\n\
            Messages: {} (User: {}, Assistant: {})\n\
            Topics: {}",
            summary.session_id,
            summary.created_at.format("%Y-%m-%d %H:%M:%S"),
            summary.message_count,
            summary.user_messages,
            summary.assistant_messages,
            summary.topics.join(", ")
        )
    }
}
