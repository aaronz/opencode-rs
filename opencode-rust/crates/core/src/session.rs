use crate::compaction::{
    CompactionConfig, CompactionResult, CompactionStatus, CompactionTrigger, Compactor, TokenBudget,
};
use crate::config::{CompactionConfig as RuntimeCompactionConfig, ShareMode};
use crate::context::{Context, ContextBuilder};
use crate::message::{Message, Role};
use crate::session_state::{is_valid_transition, SessionState, StateTransitionError};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use uuid::Uuid;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ShareError {
    SharingDisabled,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ForkError {
    MessageIndexOutOfBounds { requested: usize, len: usize },
}

impl std::fmt::Display for ShareError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ShareError::SharingDisabled => write!(f, "sharing is disabled for this session"),
        }
    }
}

impl std::error::Error for ShareError {}

impl std::fmt::Display for ForkError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ForkError::MessageIndexOutOfBounds { requested, len } => {
                write!(
                    f,
                    "fork message index out of bounds: requested {}, session has {} messages",
                    requested, len
                )
            }
        }
    }
}

impl std::error::Error for ForkError {}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Session {
    pub id: Uuid,
    pub messages: Vec<Message>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub state: SessionState,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub parent_session_id: Option<String>,
    /// Full path of ancestor session IDs for full fork lineage tracking (FR-220, FR-221).
    /// Format: "grandparent_id/parent_id" or empty for root sessions.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub lineage_path: Option<String>,
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub fork_history: Vec<ForkEntry>,
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub tool_invocations: Vec<ToolInvocationRecord>,
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub undo_history: Vec<HistoryEntry>,
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub redo_history: Vec<HistoryEntry>,
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub shared_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub share_mode: Option<ShareMode>,
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub share_expires_at: Option<DateTime<Utc>>,
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
    pub args_hash: String,
    pub result: Option<String>,
    pub started_at: DateTime<Utc>,
    pub completed_at: Option<DateTime<Utc>>,
    pub latency_ms: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SessionSummaryMetadata {
    pub summary: String,
    pub created_at: DateTime<Utc>,
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

impl Default for Session {
    fn default() -> Self {
        Self::new()
    }
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
            lineage_path: None,
            fork_history: Vec::new(),
            tool_invocations: Vec::new(),
            undo_history: Vec::new(),
            redo_history: Vec::new(),
            shared_id: None,
            share_mode: None,
            share_expires_at: None,
        }
    }

    pub fn fork(&self, new_session_id: Uuid) -> Self {
        let now = Utc::now();
        let new_lineage_path = self.compute_lineage_path();

        Self {
            id: new_session_id,
            messages: self.messages.clone(),
            created_at: now,
            updated_at: now,
            state: self.state,
            parent_session_id: Some(self.id.to_string()),
            lineage_path: new_lineage_path,
            fork_history: Vec::new(),
            tool_invocations: Vec::new(),
            undo_history: Vec::new(),
            redo_history: Vec::new(),
            shared_id: None,
            share_mode: self.share_mode.clone(),
            share_expires_at: self.share_expires_at,
        }
    }

    pub fn fork_at_message(&self, message_index: usize) -> Result<Session, ForkError> {
        if message_index >= self.messages.len() {
            return Err(ForkError::MessageIndexOutOfBounds {
                requested: message_index,
                len: self.messages.len(),
            });
        }

        let now = Utc::now();
        let new_lineage_path = self.compute_lineage_path();
        Ok(Session {
            id: Uuid::new_v4(),
            messages: self.messages[..=message_index].to_vec(),
            created_at: now,
            updated_at: now,
            state: self.state,
            parent_session_id: Some(self.id.to_string()),
            lineage_path: new_lineage_path,
            fork_history: Vec::new(),
            tool_invocations: self.tool_invocations.clone(),
            undo_history: Vec::new(),
            redo_history: Vec::new(),
            shared_id: None,
            share_mode: self.share_mode.clone(),
            share_expires_at: self.share_expires_at,
        })
    }

    pub fn compute_lineage_path(&self) -> Option<String> {
        match (&self.lineage_path, &self.parent_session_id) {
            (Some(path), Some(parent_id)) => {
                if path.is_empty() {
                    Some(parent_id.clone())
                } else {
                    Some(format!("{}/{}", path, parent_id))
                }
            }
            (None, Some(parent_id)) => Some(parent_id.clone()),
            _ => None,
        }
    }

    pub fn generate_share_link(&mut self) -> Result<String, ShareError> {
        if matches!(self.share_mode, Some(ShareMode::Disabled)) {
            return Err(ShareError::SharingDisabled);
        }

        let shared_id = self
            .shared_id
            .get_or_insert_with(|| Uuid::new_v4().to_string())
            .clone();

        if self.share_mode.is_none() {
            self.share_mode = Some(ShareMode::Manual);
        }

        self.updated_at = Utc::now();
        Ok(format!("https://opencode-rs.local/share/{shared_id}"))
    }

    pub fn set_share_mode(&mut self, mode: ShareMode) {
        if matches!(mode, ShareMode::Disabled) {
            self.shared_id = None;
            self.share_expires_at = None;
        }
        self.share_mode = Some(mode);
        self.updated_at = Utc::now();
    }

    pub fn is_shared(&self) -> bool {
        if self.shared_id.is_none() {
            return false;
        }
        if matches!(self.share_mode, Some(ShareMode::Disabled)) {
            return false;
        }
        !self.is_share_expired()
    }

    pub fn get_share_id(&self) -> Option<&str> {
        if self.is_shared() {
            self.shared_id.as_deref()
        } else {
            None
        }
    }

    pub fn export_json(&self) -> Result<String, crate::OpenCodeError> {
        #[derive(serde::Serialize)]
        struct ToolInvocationExport {
            tool_name: String,
            args_hash: String,
            result_summary: Option<String>,
            latency_ms: Option<u64>,
        }
        #[derive(serde::Serialize)]
        struct MessageExport {
            role: String,
            content: String,
        }
        #[derive(serde::Serialize)]
        struct SessionExport<'a> {
            version: &'static str,
            session: SessionInfoExport<'a>,
            messages: Vec<MessageExport>,
            tools: Vec<ToolInvocationExport>,
        }
        #[derive(serde::Serialize)]
        struct SessionInfoExport<'a> {
            id: &'a str,
            created_at: &'a str,
            updated_at: &'a str,
        }

        let messages = self
            .messages
            .iter()
            .map(|m| MessageExport {
                role: format!("{:?}", m.role),
                content: sanitize_content(&m.content),
            })
            .collect();

        let tools = self
            .tool_invocations
            .iter()
            .map(|t| ToolInvocationExport {
                tool_name: t.tool_name.clone(),
                args_hash: t.args_hash.clone(),
                result_summary: t.result.clone(),
                latency_ms: t.latency_ms,
            })
            .collect();

        let export = SessionExport {
            version: "1.0",
            session: SessionInfoExport {
                id: &self.id.to_string(),
                created_at: &self.created_at.to_rfc3339(),
                updated_at: &self.updated_at.to_rfc3339(),
            },
            messages,
            tools,
        };

        serde_json::to_string_pretty(&export)
            .map_err(|e| crate::OpenCodeError::Config(e.to_string()))
    }

    pub fn export_markdown(&self) -> Result<String, crate::OpenCodeError> {
        let mut md = format!("# Session {}\n\n", self.id);

        for msg in &self.messages {
            let role_label = match msg.role {
                Role::System => "**System**",
                Role::User => "**User**",
                Role::Assistant => "**Assistant**",
            };
            md.push_str(&format!(
                "### {}\n\n{}\n\n",
                role_label,
                sanitize_content(&msg.content)
            ));
        }

        Ok(md)
    }

    pub fn sanitize_for_export(&self) -> Session {
        let mut sanitized = self.clone();
        for msg in &mut sanitized.messages {
            msg.content = sanitize_content(&msg.content);
        }
        sanitized
    }

    pub fn set_share_expiry(&mut self, expiry: Option<DateTime<Utc>>) {
        self.share_expires_at = expiry;
        self.updated_at = Utc::now();
    }

    pub fn store_summary_metadata(
        &mut self,
        summary: impl Into<String>,
        created_at: DateTime<Utc>,
    ) {
        let summary = summary.into();
        let args = serde_json::json!({
            "kind": "session_summary",
            "created_at": created_at,
        });
        use sha2::{Digest, Sha256};
        let mut hasher = Sha256::new();
        hasher.update(args.to_string().as_bytes());
        let args_hash = format!("{:x}", hasher.finalize());
        let record = ToolInvocationRecord {
            id: Uuid::new_v4(),
            tool_name: "session.summary".to_string(),
            arguments: args,
            args_hash,
            result: Some(summary),
            started_at: created_at,
            completed_at: Some(created_at),
            latency_ms: Some(0),
        };
        self.tool_invocations.push(record);
        self.updated_at = Utc::now();
    }

    pub fn latest_summary_metadata(&self) -> Option<SessionSummaryMetadata> {
        self.tool_invocations
            .iter()
            .rev()
            .find(|inv| inv.tool_name == "session.summary")
            .and_then(|inv| {
                let summary = inv.result.clone()?;
                let created_at = inv
                    .arguments
                    .get("created_at")
                    .and_then(|v| v.as_str())
                    .and_then(|s| s.parse::<DateTime<Utc>>().ok())
                    .unwrap_or(inv.started_at);
                Some(SessionSummaryMetadata {
                    summary,
                    created_at,
                })
            })
    }

    fn is_share_expired(&self) -> bool {
        self.share_expires_at
            .map(|expiry| Utc::now() > expiry)
            .unwrap_or(false)
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
        let _ = Compactor::auto_compact_if_needed(
            self,
            &RuntimeCompactionConfig::default(),
            TokenBudget::default().total,
        );
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
            if path.extension().is_some_and(|ext| ext == "json") {
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
        let mut context = self.build_context();
        if context.budget.max_tokens > max_tokens {
            context.budget.max_tokens = max_tokens;
        }
        self.messages = context.prompt_messages.clone();
        self.messages.clone()
    }

    pub fn build_context(&self) -> Context {
        let model_name = std::env::var("OPENCODE_MODEL").ok();
        let token_budget = model_name
            .as_deref()
            .map(TokenBudget::from_model)
            .unwrap_or_default();
        let registry = crate::tool::build_default_registry();

        ContextBuilder::new(token_budget)
            .with_model_name(model_name.as_deref())
            .collect_file_context(&[], &self.messages)
            .collect_tool_context(&registry)
            .collect_session_context(&self.messages)
            .build()
    }
}

fn sanitize_content(content: &str) -> String {
    let mut result = content.to_string();

    // Strip common API key patterns
    let patterns = [
        (r"sk-[a-zA-Z0-9]{20,}", "[REDACTED_API_KEY]"),
        (r"ghp_[a-zA-Z0-9]{36}", "[REDACTED_GITHUB_TOKEN]"),
        (r"xoxb-[a-zA-Z0-9-]+", "[REDACTED_SLACK_TOKEN]"),
        (r"gho_[a-zA-Z0-9]{36}", "[REDACTED_GITHUB_OAUTH]"),
        (
            r"eyJ[a-zA-Z0-9_-]+\.eyJ[a-zA-Z0-9_-]+\.[a-zA-Z0-9_-]+",
            "[REDACTED_JWT]",
        ),
    ];

    for (pattern, replacement) in &patterns {
        if let Ok(re) = regex::Regex::new(pattern) {
            result = re.replace_all(&result, *replacement).to_string();
        }
    }

    // Strip lines that look like credential assignments
    result
        .lines()
        .map(|line| {
            let lower = line.to_lowercase();
            if (lower.contains("api_key")
                || lower.contains("secret")
                || lower.contains("password")
                || lower.contains("token"))
                && (lower.contains("=") || lower.contains(":"))
                && !lower.contains("http")
            {
                String::from("[REDACTED_CREDENTIAL]")
            } else {
                line.to_string()
            }
        })
        .collect::<Vec<_>>()
        .join("\n")
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
    fn test_session_summary_metadata_roundtrip() {
        let mut session = Session::new();
        let created_at = Utc::now();
        session.store_summary_metadata("Summary text", created_at);

        let metadata = session.latest_summary_metadata().expect("missing metadata");
        assert_eq!(metadata.summary, "Summary text");
        assert_eq!(metadata.created_at.timestamp(), created_at.timestamp());
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

    #[test]
    fn test_generate_share_link_sets_shared_id() {
        let mut session = Session::new();
        let link = session.generate_share_link().unwrap();

        assert!(link.contains("/share/"));
        assert!(session.shared_id.is_some());
        assert!(matches!(session.share_mode, Some(ShareMode::Manual)));
        assert!(session.is_shared());
    }

    #[test]
    fn test_generate_share_link_fails_when_disabled() {
        let mut session = Session::new();
        session.set_share_mode(ShareMode::Disabled);

        let err = session.generate_share_link().unwrap_err();
        assert_eq!(err, ShareError::SharingDisabled);
        assert!(!session.is_shared());
    }

    #[test]
    fn test_share_expiry_hides_share() {
        let mut session = Session::new();
        session.generate_share_link().unwrap();
        session.set_share_expiry(Some(Utc::now() - chrono::Duration::minutes(1)));

        assert!(!session.is_shared());
        assert!(session.get_share_id().is_none());
    }

    #[test]
    fn test_fork_at_message_copies_upto_index() {
        let mut parent = Session::new();
        parent.add_message(Message::user("first"));
        parent.add_message(Message::assistant("second"));
        parent.add_message(Message::user("third"));

        let child = parent.fork_at_message(1).unwrap();
        let parent_id = parent.id.to_string();
        assert_ne!(child.id, parent.id);
        assert_eq!(child.parent_session_id.as_deref(), Some(parent_id.as_str()));
        assert_eq!(child.messages.len(), 2);
        assert_eq!(child.messages[0].content, "first");
        assert_eq!(child.messages[1].content, "second");
    }

    #[test]
    fn test_fork_at_message_out_of_bounds() {
        let mut parent = Session::new();
        parent.add_message(Message::user("first"));

        let err = parent.fork_at_message(5).unwrap_err();
        assert_eq!(
            err,
            ForkError::MessageIndexOutOfBounds {
                requested: 5,
                len: 1,
            }
        );
    }

    #[test]
    fn test_new_session_has_no_lineage() {
        let session = Session::new();
        assert!(session.parent_session_id.is_none());
        assert!(session.lineage_path.is_none());
        assert!(session.compute_lineage_path().is_none());
    }

    #[test]
    fn test_fork_single_level_lineage() {
        let parent = Session::new();
        let parent_id = parent.id.to_string();

        let child = parent.fork(Uuid::new_v4());

        assert_eq!(child.parent_session_id.as_deref(), Some(parent_id.as_str()));
        assert!(child.lineage_path.is_none());
        assert_eq!(child.compute_lineage_path(), Some(parent_id));
    }

    #[test]
    fn test_fork_multi_level_lineage() {
        let grandparent = Session::new();
        let grandparent_id = grandparent.id.to_string();

        let parent = grandparent.fork(Uuid::new_v4());
        let parent_id = parent.id.to_string();

        let child = parent.fork(Uuid::new_v4());
        let child_lineage = child.compute_lineage_path();

        assert_eq!(child.parent_session_id.as_deref(), Some(parent_id.as_str()));
        assert_eq!(child.lineage_path, Some(grandparent_id.clone()));
        assert_eq!(
            child_lineage,
            Some(format!("{}/{}", grandparent_id, parent_id))
        );
    }

    #[test]
    fn test_fork_at_message_lineage() {
        let mut parent = Session::new();
        parent.add_message(Message::user("test"));
        let parent_id = parent.id.to_string();

        let child = parent.fork_at_message(0).unwrap();

        assert_eq!(child.parent_session_id.as_deref(), Some(parent_id.as_str()));
        assert!(child.lineage_path.is_none());
        assert_eq!(child.compute_lineage_path(), Some(parent_id));
    }

    #[test]
    fn test_lineage_persistence_after_save_load() {
        let grandparent = Session::new();
        let parent = grandparent.fork(Uuid::new_v4());
        let child = parent.fork(Uuid::new_v4());

        let tmp = tempfile::TempDir::new().unwrap();
        let path = tmp.path().join("session.json");
        child.save(&path).unwrap();

        let loaded = Session::load(&path).unwrap();
        let grandparent_id = grandparent.id.to_string();
        let parent_id = parent.id.to_string();

        assert_eq!(loaded.lineage_path, Some(grandparent_id.clone()));
        assert_eq!(
            loaded.compute_lineage_path(),
            Some(format!("{}/{}", grandparent_id, parent_id))
        );
    }
}
