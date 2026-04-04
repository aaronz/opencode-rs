use crate::{Message, Role, Session};
use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Debug, Error, Clone, PartialEq, Eq)]
pub enum SummaryError {
    #[error("cannot summarize an empty session")]
    EmptySession,
}

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
    pub fn summarize_session(messages: &[Message]) -> Result<String, SummaryError> {
        if messages.is_empty() {
            return Err(SummaryError::EmptySession);
        }

        let prompt = Self::build_session_summary_prompt(messages);
        let user_count = messages
            .iter()
            .filter(|m| matches!(m.role, Role::User))
            .count();
        let assistant_count = messages
            .iter()
            .filter(|m| matches!(m.role, Role::Assistant))
            .count();

        let latest_user_request = messages
            .iter()
            .rev()
            .find(|m| matches!(m.role, Role::User))
            .map(|m| m.content.trim())
            .unwrap_or("(none)");

        let latest_assistant_response = messages
            .iter()
            .rev()
            .find(|m| matches!(m.role, Role::Assistant))
            .map(|m| m.content.trim())
            .unwrap_or("(none)");

        Ok(format!(
            "Session summary\n- Exchanges: {} messages (user: {}, assistant: {})\n- Latest user request: {}\n- Latest assistant response: {}\n\nPrompt basis:\n{}",
            messages.len(),
            user_count,
            assistant_count,
            latest_user_request,
            latest_assistant_response,
            prompt
        ))
    }

    pub fn build_session_summary_prompt(messages: &[Message]) -> String {
        let transcript = messages
            .iter()
            .map(|m| {
                let role = match m.role {
                    Role::System => "system",
                    Role::User => "user",
                    Role::Assistant => "assistant",
                };
                format!("- {}: {}", role, m.content.trim())
            })
            .collect::<Vec<_>>()
            .join("\n");

        format!(
            "You are an assistant that writes concise technical summaries. Summarize key goals, decisions, and open items from this session transcript:\n{}",
            transcript
        )
    }

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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Message;

    #[test]
    fn summarize_session_rejects_empty_messages() {
        let result = SummaryGenerator::summarize_session(&[]);
        assert!(matches!(result, Err(SummaryError::EmptySession)));
    }

    #[test]
    fn summarize_session_contains_user_and_assistant_content() {
        let messages = vec![
            Message::user("Need a Rust API summary endpoint"),
            Message::assistant("I will add POST /sessions/{id}/summarize"),
        ];
        let summary = SummaryGenerator::summarize_session(&messages).unwrap();
        assert!(summary.contains("Latest user request"));
        assert!(summary.contains("summary endpoint"));
        assert!(summary.contains("POST /sessions/{id}/summarize"));
    }

    #[test]
    fn summary_prompt_includes_roles() {
        let messages = vec![
            Message::system("system setup"),
            Message::user("hello"),
            Message::assistant("hi"),
        ];
        let prompt = SummaryGenerator::build_session_summary_prompt(&messages);
        assert!(prompt.contains("- system: system setup"));
        assert!(prompt.contains("- user: hello"));
        assert!(prompt.contains("- assistant: hi"));
    }

    #[test]
    fn test_summary_generator_empty_session() {
        let session = Session::new();
        let summary = SummaryGenerator::generate(&session);
        assert_eq!(summary.message_count, 0);
        assert_eq!(summary.user_messages, 0);
        assert_eq!(summary.assistant_messages, 0);
    }

    #[test]
    fn test_summary_generator_with_messages() {
        let mut session = Session::new();
        session.add_message(Message::user(
            "Hello, this is a long message about coding in Rust".to_string(),
        ));
        session.add_message(Message::assistant(
            "I can help you with Rust programming".to_string(),
        ));

        let summary = SummaryGenerator::generate(&session);
        assert_eq!(summary.message_count, 2);
        assert_eq!(summary.user_messages, 1);
        assert_eq!(summary.assistant_messages, 1);
    }

    #[test]
    fn test_summarize_text() {
        let mut session = Session::new();
        session.add_message(Message::user("Test message".to_string()));

        let result = SummaryGenerator::summarize_text(&session);
        assert!(result.contains("Session"));
        assert!(result.contains("Messages:"));
    }
}
