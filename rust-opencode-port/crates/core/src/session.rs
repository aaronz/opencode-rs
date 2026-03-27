use crate::message::Message;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Session {
    pub id: Uuid,
    pub messages: Vec<Message>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionInfo {
    pub id: Uuid,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub message_count: usize,
    pub preview: String,
}

impl Session {
    pub fn new() -> Self {
        let now = Utc::now();
        Self {
            id: Uuid::new_v4(),
            messages: Vec::new(),
            created_at: now,
            updated_at: now,
        }
    }

    pub fn add_message(&mut self, message: Message) {
        self.messages.push(message);
        self.updated_at = Utc::now();
    }

    pub fn save(&self, path: &PathBuf) -> Result<(), crate::OpenCodeError> {
        let json = serde_json::to_string_pretty(self)?;
        std::fs::create_dir_all(path.parent().unwrap())?;
        std::fs::write(path, json)?;
        Ok(())
    }

    pub fn load(path: &PathBuf) -> Result<Self, crate::OpenCodeError> {
        let content = std::fs::read_to_string(path)?;
        serde_json::from_str(&content).map_err(|e| crate::OpenCodeError::Session(e.to_string()))
    }

    pub fn sessions_dir() -> PathBuf {
        directories::ProjectDirs::from("com", "opencode", "rs")
            .map(|dirs| dirs.data_dir().join("sessions"))
            .unwrap_or_else(|| PathBuf::from("~/.local/share/opencode-rs/sessions"))
    }

    pub fn session_path(id: &Uuid) -> PathBuf {
        Self::sessions_dir().join(format!("{}.json", id))
    }

    pub fn load_by_id(id: &Uuid) -> Result<Self, crate::OpenCodeError> {
        let path = Self::session_path(id);
        Self::load(&path)
    }

    pub fn delete(id: &Uuid) -> Result<(), crate::OpenCodeError> {
        let path = Self::session_path(id);
        if path.exists() {
            std::fs::remove_file(path)?;
        }
        Ok(())
    }

    pub fn list() -> Result<Vec<SessionInfo>, crate::OpenCodeError> {
        let dir = Self::sessions_dir();
        if !dir.exists() {
            return Ok(Vec::new());
        }

        let mut sessions = Vec::new();
        for entry in std::fs::read_dir(dir)? {
            let entry = entry?;
            let path = entry.path();
            if path.extension().map_or(false, |ext| ext == "json") {
                if let Ok(session) = Self::load(&path) {
                    let preview = session
                        .messages
                        .last()
                        .map(|m| m.content.chars().take(50).collect())
                        .unwrap_or_default();

                    sessions.push(SessionInfo {
                        id: session.id,
                        created_at: session.created_at,
                        updated_at: session.updated_at,
                        message_count: session.messages.len(),
                        preview,
                    });
                }
            }
        }

        sessions.sort_by(|a, b| b.updated_at.cmp(&a.updated_at));
        Ok(sessions)
    }

    pub fn truncate_for_context(&mut self, max_tokens: usize) {
        let estimated_chars_per_token = 4;
        let max_chars = max_tokens * estimated_chars_per_token;

        let total_chars: usize = self.messages.iter().map(|m| m.content.len()).sum();

        if total_chars <= max_chars {
            return;
        }

        while self.messages.iter().map(|m| m.content.len()).sum::<usize>() > max_chars
            && self.messages.len() > 1
        {
            if self.messages[0].role == crate::message::Role::System {
                break;
            }
            self.messages.remove(0);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_session_new() {
        let session = Session::new();
        assert!(!session.id.is_nil());
        assert!(session.messages.is_empty());
        assert_eq!(session.created_at, session.updated_at);
    }

    #[test]
    fn test_session_add_message() {
        let mut session = Session::new();
        let msg = Message::user("Hello".to_string());
        session.add_message(msg);

        assert_eq!(session.messages.len(), 1);
        assert!(session.updated_at >= session.created_at);
    }

    #[test]
    fn test_session_save_load() {
        let tmp = TempDir::new().unwrap();
        let path = tmp.path().join("session.json");

        let mut session = Session::new();
        session.add_message(Message::user("Test message".to_string()));

        session.save(&path).unwrap();
        let loaded = Session::load(&path).unwrap();

        assert_eq!(loaded.id, session.id);
        assert_eq!(loaded.messages.len(), 1);
        assert_eq!(loaded.messages[0].content, "Test message");
    }

    #[test]
    fn test_session_truncate() {
        let mut session = Session::new();
        session.add_message(Message::user("A".repeat(100)));
        session.add_message(Message::assistant("B".repeat(100)));

        session.truncate_for_context(10);

        assert!(session.messages.len() < 2);
    }
}
