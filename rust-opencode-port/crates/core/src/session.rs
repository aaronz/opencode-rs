use crate::compaction::{
    CompactionConfig, CompactionResult, CompactionStatus, CompactionTrigger, Compactor, TokenBudget,
};
use crate::message::Message;
use crate::session_state::{is_valid_transition, SessionState, StateTransitionError};
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
    pub state: SessionState,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub parent_session_id: Option<Uuid>,
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub fork_history: Vec<ForkEntry>,
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub tool_invocations: Vec<ToolInvocationRecord>,
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub undo_history: Vec<HistoryEntry>,
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub redo_history: Vec<HistoryEntry>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ForkEntry {
    pub forked_at: DateTime<Utc>,
    pub child_session_id: Uuid,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolInvocationRecord {
    pub id: Uuid,
    pub tool_name: String,
    pub arguments: serde_json::Value,
    pub result: Option<String>,
    pub started_at: DateTime<Utc>,
    pub completed_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HistoryEntry {
    pub action: Action,
    pub messages: Vec<Message>,
    pub timestamp: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Action {
    AddMessage,
    RemoveMessage,
    ClearSession,
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
            state: SessionState::Idle,
            parent_session_id: None,
            fork_history: Vec::new(),
            tool_invocations: Vec::new(),
            undo_history: Vec::new(),
            redo_history: Vec::new(),
        }
    }

    pub fn fork(&self, new_session_id: Uuid) -> Self {
        let now = Utc::now();
        let forked = Self {
            id: new_session_id,
            messages: self.messages.clone(),
            created_at: now,
            updated_at: now,
            state: self.state,
            parent_session_id: Some(self.id),
            fork_history: Vec::new(),
            tool_invocations: Vec::new(),
            undo_history: Vec::new(),
            redo_history: Vec::new(),
        };
        forked
    }

    pub fn set_state(&mut self, new_state: SessionState) -> Result<(), StateTransitionError> {
        if !is_valid_transition(self.state, new_state) {
            return Err(StateTransitionError {
                from: self.state,
                to: new_state,
            });
        }
        self.state = new_state;
        self.updated_at = Utc::now();
        Ok(())
    }

    pub fn add_message(&mut self, message: Message) {
        self.undo_history.push(HistoryEntry {
            action: Action::AddMessage,
            messages: self.messages.clone(),
            timestamp: Utc::now(),
        });
        self.redo_history.clear();
        self.messages.push(message);
        self.updated_at = Utc::now();
    }

    pub fn undo(&mut self, steps: usize) -> Result<usize, String> {
        let mut undone = 0;
        for _ in 0..steps {
            if let Some(entry) = self.undo_history.pop() {
                self.redo_history.push(HistoryEntry {
                    action: entry.action.clone(),
                    messages: self.messages.clone(),
                    timestamp: Utc::now(),
                });
                self.messages = entry.messages;
                self.updated_at = Utc::now();
                undone += 1;
            } else {
                break;
            }
        }
        if undone == 0 {
            return Err("Nothing to undo".to_string());
        }
        Ok(undone)
    }

    pub fn redo(&mut self, steps: usize) -> Result<usize, String> {
        let mut redone = 0;
        for _ in 0..steps {
            if let Some(entry) = self.redo_history.pop() {
                self.undo_history.push(HistoryEntry {
                    action: entry.action.clone(),
                    messages: self.messages.clone(),
                    timestamp: Utc::now(),
                });
                self.messages = entry.messages;
                self.updated_at = Utc::now();
                redone += 1;
            } else {
                break;
            }
        }
        if redone == 0 {
            return Err("Nothing to redo".to_string());
        }
        Ok(redone)
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

    pub fn compact_messages(&mut self, max_tokens: usize) -> CompactionResult {
        let config = CompactionConfig {
            max_tokens,
            preserve_system_messages: true,
            preserve_recent_messages: 10,
            ..Default::default()
        };
        let compactor = Compactor::new(config);
        let messages = std::mem::take(&mut self.messages);
        let result = compactor.compact_to_fit(messages);
        self.messages = result.messages.clone();
        if result.was_compacted {
            self.updated_at = Utc::now();
        }
        result
    }

    pub fn needs_compaction(&self, max_tokens: usize) -> bool {
        let config = CompactionConfig {
            max_tokens,
            ..Default::default()
        };
        let compactor = Compactor::new(config);
        compactor.needs_compaction(&self.messages)
    }

    pub fn estimate_token_count(&self) -> usize {
        let config = CompactionConfig::default();
        let compactor = Compactor::new(config);
        self.messages
            .iter()
            .map(|m| compactor.estimate_tokens(&m.content))
            .sum()
    }

    pub fn get_compaction_status(&self) -> CompactionStatus {
        let budget = TokenBudget::default();
        let used = self.estimate_token_count();
        CompactionStatus::check(&budget, used)
    }

    pub fn auto_compact_if_needed(&mut self) -> CompactionResult {
        let status = self.get_compaction_status();
        match status.trigger {
            CompactionTrigger::AutoCompact | CompactionTrigger::ForceContinuation => {
                let budget = TokenBudget::default();
                self.compact_messages(budget.main_context_tokens())
            }
            _ => CompactionResult {
                messages: self.messages.clone(),
                was_compacted: false,
                pruned_count: 0,
                summary_inserted: false,
            },
        }
    }

    pub fn prepare_messages_for_prompt(&mut self, max_tokens: usize) -> Vec<Message> {
        if self.needs_compaction(max_tokens) {
            self.compact_messages(max_tokens);
        }
        self.messages.clone()
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

    #[test]
    fn test_session_undo_single() {
        let mut session = Session::new();
        session.add_message(Message::user("First".to_string()));
        session.add_message(Message::assistant("Second".to_string()));

        assert_eq!(session.messages.len(), 2);

        let result = session.undo(1);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 1);
        assert_eq!(session.messages.len(), 1);
        assert_eq!(session.messages[0].content, "First");
    }

    #[test]
    fn test_session_undo_multiple() {
        let mut session = Session::new();
        session.add_message(Message::user("First".to_string()));
        session.add_message(Message::assistant("Second".to_string()));
        session.add_message(Message::user("Third".to_string()));

        let result = session.undo(2);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 2);
        assert_eq!(session.messages.len(), 1);
    }

    #[test]
    fn test_session_undo_nothing() {
        let mut session = Session::new();
        let result = session.undo(1);
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "Nothing to undo");
    }

    #[test]
    fn test_session_redo() {
        let mut session = Session::new();
        session.add_message(Message::user("First".to_string()));
        session.add_message(Message::assistant("Second".to_string()));

        session.undo(1).unwrap();
        assert_eq!(session.messages.len(), 1);

        let result = session.redo(1);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 1);
        assert_eq!(session.messages.len(), 2);
        assert_eq!(session.messages[1].content, "Second");
    }

    #[test]
    fn test_session_redo_nothing() {
        let mut session = Session::new();
        let result = session.redo(1);
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "Nothing to redo");
    }

    #[test]
    fn test_session_undo_redo_clears_redo() {
        let mut session = Session::new();
        session.add_message(Message::user("First".to_string()));
        session.add_message(Message::assistant("Second".to_string()));

        session.undo(1).unwrap();
        assert_eq!(session.redo_history.len(), 1);

        session.add_message(Message::user("New".to_string()));
        assert!(session.redo_history.is_empty());
    }

    #[test]
    fn test_session_undo_persistence() {
        let tmp = TempDir::new().unwrap();
        let path = tmp.path().join("session.json");

        let mut session = Session::new();
        session.add_message(Message::user("Test".to_string()));

        assert_eq!(session.undo_history.len(), 1);

        session.undo(1).unwrap();

        session.save(&path).unwrap();
        let loaded = Session::load(&path).unwrap();

        assert!(!loaded.redo_history.is_empty());
        assert!(loaded.messages.is_empty());
    }
}
