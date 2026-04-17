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
    InvalidShareMode,
    AccessDenied,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ForkError {
    MessageIndexOutOfBounds { requested: usize, len: usize },
}

impl std::fmt::Display for ShareError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ShareError::SharingDisabled => write!(f, "sharing is disabled for this session"),
            ShareError::InvalidShareMode => write!(f, "invalid share mode for this operation"),
            ShareError::AccessDenied => write!(f, "access denied for this operation"),
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

#[allow(dead_code)]
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
        let messages_md: Vec<String> = self
            .messages
            .iter()
            .map(|msg| {
                let role_label = match msg.role {
                    Role::System => "**System**",
                    Role::User => "**User**",
                    Role::Assistant => "**Assistant**",
                };
                format!(
                    "### {}\n\n{}\n\n",
                    role_label,
                    sanitize_content(&msg.content)
                )
            })
            .collect();
        Ok(format!("# Session {}\n\n{}", self.id, messages_md.join("")))
    }

    pub fn sanitize_for_export(&self) -> Session {
        let mut session = self.clone();
        session.messages = std::mem::take(&mut session.messages)
            .into_iter()
            .map(|mut msg| {
                msg.content = sanitize_content(&msg.content);
                msg
            })
            .collect();
        session
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

    pub(crate) fn latest_summary_metadata(&self) -> Option<SessionSummaryMetadata> {
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

    pub(crate) fn set_state(
        &mut self,
        new_state: SessionState,
    ) -> Result<(), StateTransitionError> {
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
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        std::fs::write(path, json)?;
        Ok(())
    }

    pub fn load(path: &PathBuf) -> Result<Self, crate::OpenCodeError> {
        let content = std::fs::read_to_string(path)?;
        serde_json::from_str(&content).map_err(|e| crate::OpenCodeError::Session(e.to_string()))
    }

    pub(crate) fn sessions_dir() -> PathBuf {
        if let Ok(data_dir) = std::env::var("OPENCODE_DATA_DIR") {
            let path = PathBuf::from(data_dir).join("sessions");
            let _ = std::fs::create_dir_all(&path);
            return path;
        }
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

    pub(crate) fn delete(id: &Uuid) -> Result<(), crate::OpenCodeError> {
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

    pub(crate) fn needs_compaction(&self, max_tokens: usize) -> bool {
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

    pub(crate) fn auto_compact_if_needed(&mut self) -> CompactionResult {
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
    let api_key_patterns = [
        (r"sk-[a-zA-Z0-9]{20,}", "[REDACTED_API_KEY]"),
        (r"ghp_[a-zA-Z0-9]{36}", "[REDACTED_GITHUB_TOKEN]"),
        (r"xoxb-[a-zA-Z0-9-]+", "[REDACTED_SLACK_TOKEN]"),
        (r"gho_[a-zA-Z0-9]{36}", "[REDACTED_GITHUB_OAUTH]"),
        (
            r"eyJ[a-zA-Z0-9_-]+\.eyJ[a-zA-Z0-9_-]+\.[a-zA-Z0-9_-]+",
            "[REDACTED_JWT]",
        ),
    ];

    let result =
        api_key_patterns
            .iter()
            .fold(content.to_string(), |acc, (pattern, replacement)| {
                regex::Regex::new(pattern)
                    .map(|re| re.replace_all(&acc, *replacement).to_string())
                    .unwrap_or(acc)
            });

    let result = result
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
        .join("\n");

    let sql_patterns = [
        r"(?i)\bDROP\s+TABLE\b",
        r"(?i)\bDROP\s+DATABASE\b",
        r"(?i)\bDELETE\s+FROM\b",
        r"(?i)\bINSERT\s+INTO\b",
        r"(?i)\bUNION\s+SELECT\b",
        r"(?i)\bALTER\s+TABLE\b",
        r"(?i)\bTRUNCATE\b",
        r"--+",
        r";\s*$",
        r"'\s*OR\s+'1'\s*=\s*'1",
    ];

    let result = sql_patterns.iter().fold(result, |acc, pattern| {
        regex::Regex::new(pattern)
            .map(|re| re.replace_all(&acc, "[SQL_REDACTED]").to_string())
            .unwrap_or(acc)
    });

    result
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&#x27;")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::checkpoint::CheckpointManager;
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

    // =========================================================================
    // Ownership Tree Tests
    // =========================================================================

    #[test]
    fn test_ownership_tree_fork_creates_child_with_parent_reference() {
        // Test that fork() correctly transfers ownership by establishing parent-child relationship
        let mut parent = Session::new();
        parent.add_message(Message::user("original message"));
        let parent_id = parent.id.to_string();

        let child = parent.fork(Uuid::new_v4());

        // Invariant: child must have parent_session_id pointing to original parent
        assert_eq!(
            child.parent_session_id.as_deref(),
            Some(parent_id.as_str()),
            "Forked session must reference its parent"
        );
        // Invariant: child ID must be different from parent
        assert_ne!(child.id, parent.id, "Forked session must have unique ID");
        // Invariant: messages are transferred but child is independent
        assert_eq!(child.messages.len(), 1);
    }

    #[test]
    fn test_ownership_tree_fork_at_message_preserves_ownership() {
        // Test that fork_at_message() correctly transfers ownership at specific point
        let mut parent = Session::new();
        parent.add_message(Message::user("first"));
        parent.add_message(Message::assistant("second"));
        parent.add_message(Message::user("third"));
        let parent_id = parent.id.to_string();

        let child = parent.fork_at_message(1).unwrap();

        // Invariant: ownership transfer at message index
        assert_eq!(
            child.parent_session_id.as_deref(),
            Some(parent_id.as_str()),
            "Child must reference parent after fork_at_message"
        );
        // Invariant: only messages up to and including index are transferred
        assert_eq!(child.messages.len(), 2, "Child should have messages[0..=1]");
        assert_eq!(child.messages[0].content, "first");
        assert_eq!(child.messages[1].content, "second");
    }

    #[test]
    fn test_ownership_tree_multi_level_fork_lineage_chain() {
        // Test that multi-level forking maintains correct lineage invariant
        let grandparent = Session::new();
        let grandparent_id = grandparent.id.to_string();

        let parent = grandparent.fork(Uuid::new_v4());
        let parent_id = parent.id.to_string();

        let child = parent.fork(Uuid::new_v4());

        assert_eq!(
            child.lineage_path,
            Some(grandparent_id.clone()),
            "Child must track grandparent in lineage_path"
        );
        assert_eq!(
            child.compute_lineage_path(),
            Some(format!("{}/{}", grandparent_id, parent_id)),
            "Lineage path must be grandparent_id/parent_id"
        );
        assert_eq!(
            child.parent_session_id.as_deref(),
            Some(parent_id.as_str()),
            "Parent reference must be immediate parent"
        );
        assert_eq!(
            child.parent_session_id.as_deref(),
            Some(parent_id.as_str()),
            "Parent reference must be immediate parent"
        );
        // Invariant: each session in chain has correct parent
        assert_eq!(
            parent.parent_session_id.as_deref(),
            Some(grandparent_id.as_str()),
            "Parent must reference grandparent"
        );
    }

    #[test]
    fn test_ownership_tree_share_link_generation_manual_mode() {
        // Test that share link generation works correctly in Manual mode
        let mut session = Session::new();
        session.add_message(Message::user("secret content"));

        let link = session.generate_share_link().unwrap();

        // Invariant: share link URL is well-formed
        assert!(
            link.contains("/share/"),
            "Share link must contain /share/ path"
        );
        // Invariant: shared_id is set
        assert!(
            session.shared_id.is_some(),
            "shared_id must be set after generating link"
        );
        // Invariant: share_mode is set to Manual
        assert_eq!(
            session.share_mode,
            Some(ShareMode::Manual),
            "Share mode must be Manual after generating link"
        );
        // Invariant: session reports as shared
        assert!(session.is_shared(), "Session must report as shared");
    }

    #[test]
    fn test_ownership_tree_share_link_generation_auto_mode() {
        // Test that share link generation works correctly in Auto mode
        let mut session = Session::new();
        session.set_share_mode(ShareMode::Auto);

        let link = session.generate_share_link().unwrap();

        // Invariant: share link is generated even in Auto mode
        assert!(link.contains("/share/"));
        assert!(session.shared_id.is_some());
        assert_eq!(session.share_mode, Some(ShareMode::Auto));
        assert!(session.is_shared());
    }

    #[test]
    fn test_ownership_tree_share_link_blocked_when_disabled() {
        // Test that share link generation fails when sharing is disabled
        let mut session = Session::new();
        session.set_share_mode(ShareMode::Disabled);

        let result = session.generate_share_link();

        // Invariant: sharing disabled must prevent link generation
        assert!(
            result.is_err(),
            "Share link generation must fail when disabled"
        );
        assert_eq!(result.unwrap_err(), ShareError::SharingDisabled);
        // Invariant: is_shared must return false
        assert!(
            !session.is_shared(),
            "Session must not be shared when disabled"
        );
    }

    #[test]
    fn test_ownership_tree_share_mode_transitions() {
        let mut session = Session::new();

        assert!(!session.is_shared());
        assert!(session.shared_id.is_none());

        session.set_share_mode(ShareMode::Manual);
        session.generate_share_link().unwrap();
        assert!(session.is_shared());
        assert!(session.shared_id.is_some());

        session.set_share_mode(ShareMode::Disabled);
        assert!(!session.is_shared());
        assert!(
            session.shared_id.is_none(),
            "shared_id must be cleared when disabled"
        );

        session.set_share_mode(ShareMode::Manual);
        assert!(
            session.generate_share_link().is_ok(),
            "Share link generation should work after re-enabling"
        );
        assert!(session.is_shared());
    }

    #[test]
    fn test_ownership_tree_share_expiry_enforces_temporal_bound() {
        // Test that share expiry correctly enforces temporal ownership bound
        let mut session = Session::new();
        session.generate_share_link().unwrap();

        // Invariant: newly shared session is not expired
        assert!(session.is_shared());
        assert!(session.get_share_id().is_some());

        // Set expiry to past
        session.set_share_expiry(Some(Utc::now() - chrono::Duration::minutes(1)));

        // Invariant: expired session is not considered shared
        assert!(!session.is_shared(), "Expired session must not be shared");
        assert!(
            session.get_share_id().is_none(),
            "Expired session must not return share_id"
        );

        // Set expiry to future
        let mut session2 = Session::new();
        session2.generate_share_link().unwrap();
        session2.set_share_expiry(Some(Utc::now() + chrono::Duration::hours(1)));

        // Invariant: future expiry session is still shared
        assert!(session2.is_shared());
        assert!(session2.get_share_id().is_some());
    }

    #[test]
    fn test_ownership_tree_share_id_uniqueness() {
        // Test that each session gets a unique share_id invariant
        let mut session1 = Session::new();
        let mut session2 = Session::new();

        let link1 = session1.generate_share_link().unwrap();
        let link2 = session2.generate_share_link().unwrap();

        let share_id1 = session1.get_share_id();
        let share_id2 = session2.get_share_id();

        // Invariant: each session has a unique share_id
        assert_ne!(
            share_id1, share_id2,
            "Each session must have unique share_id"
        );
        // Invariant: share links are unique
        assert_ne!(link1, link2);
    }

    #[test]
    fn test_ownership_tree_fork_inherits_share_mode_not_shared_id() {
        // Test that fork inherits share mode but generates new share_id
        let mut parent = Session::new();
        parent.set_share_mode(ShareMode::Auto);
        parent.generate_share_link().unwrap();
        let parent_share_id = parent.shared_id.clone();

        let mut child = parent.fork(Uuid::new_v4());

        assert_eq!(
            child.share_mode, parent.share_mode,
            "Child must inherit parent's share_mode"
        );
        assert!(
            child.shared_id.is_none(),
            "Child must NOT inherit parent's shared_id"
        );
        assert!(!child.is_shared());

        child.generate_share_link().unwrap();
        assert_ne!(
            child.shared_id, parent_share_id,
            "Child must generate its own unique share_id"
        );
    }

    #[test]
    fn test_ownership_tree_serialization_preserves_lineage() {
        // Test that serialization preserves ownership lineage invariant
        let grandparent = Session::new();
        let parent = grandparent.fork(Uuid::new_v4());
        let child = parent.fork(Uuid::new_v4());

        let grandparent_id = grandparent.id.to_string();
        let parent_id = parent.id.to_string();
        let child_lineage = child.lineage_path.clone();

        // Serialize and deserialize child
        let tmp = tempfile::TempDir::new().unwrap();
        let path = tmp.path().join("session.json");
        child.save(&path).unwrap();
        let loaded = Session::load(&path).unwrap();

        // Invariant: lineage is preserved through serialization
        assert_eq!(
            loaded.lineage_path, child_lineage,
            "Lineage must survive serialization"
        );
        assert_eq!(
            loaded.compute_lineage_path(),
            Some(format!("{}/{}", grandparent_id, parent_id)),
            "Computed lineage must be preserved"
        );
        // Invariant: parent reference is preserved
        assert_eq!(
            loaded.parent_session_id.as_deref(),
            Some(parent_id.as_str()),
            "Parent reference must be preserved"
        );
    }

    #[test]
    fn test_ownership_tree_serialization_preserves_sharing_state() {
        // Test that serialization preserves sharing ownership invariant
        let mut session = Session::new();
        session.set_share_mode(ShareMode::Manual);
        session.generate_share_link().unwrap();
        let original_share_id = session.shared_id.clone();

        // Serialize and deserialize
        let tmp = tempfile::TempDir::new().unwrap();
        let path = tmp.path().join("session.json");
        session.save(&path).unwrap();
        let loaded = Session::load(&path).unwrap();

        // Invariant: sharing state survives serialization
        assert_eq!(
            loaded.shared_id, original_share_id,
            "shared_id must be preserved"
        );
        assert_eq!(
            loaded.share_mode,
            Some(ShareMode::Manual),
            "share_mode must be preserved"
        );
        assert!(loaded.is_shared(), "Loaded session must still be shared");
        assert_eq!(
            loaded.get_share_id(),
            original_share_id.as_deref(),
            "get_share_id() must return same value"
        );
    }

    #[test]
    fn test_ownership_tree_fork_copies_messages_not_references() {
        // Test that fork creates independent message ownership (copy, not reference)
        let mut parent = Session::new();
        parent.add_message(Message::user("original"));

        let mut child = parent.fork(Uuid::new_v4());

        assert_eq!(child.messages.len(), parent.messages.len());

        child.messages[0] = Message::assistant("modified by child");

        assert_eq!(
            parent.messages[0].content, "original",
            "Parent messages must not be affected by child modifications"
        );
        assert_eq!(
            parent.messages[0].role,
            Role::User,
            "Parent message role must not be affected"
        );
    }

    #[test]
    fn test_ownership_tree_multiple_forks_from_same_parent() {
        let mut parent = Session::new();
        let parent_id = parent.id.to_string();
        parent.add_message(Message::user("parent message"));

        let child1 = parent.fork(Uuid::new_v4());
        let child2 = parent.fork(Uuid::new_v4());

        assert_eq!(
            child1.parent_session_id.as_deref(),
            Some(parent_id.as_str()),
            "First child must reference parent"
        );
        assert_eq!(
            child2.parent_session_id.as_deref(),
            Some(parent_id.as_str()),
            "Second child must reference parent"
        );
        assert_ne!(child1.id, child2.id, "Children must have unique IDs");
        let mut child1_mut = child1;
        child1_mut.messages[0] = Message::assistant("child1 modified");
        assert_eq!(
            parent.messages[0].content, "parent message",
            "Parent must not be affected by child modifications"
        );
    }

    #[test]
    fn test_ownership_tree_lineage_path_format_correctness() {
        // Test that lineage_path maintains correct format invariant
        let session0 = Session::new();
        assert!(
            session0.lineage_path.is_none(),
            "New session must have no lineage_path"
        );

        let session1 = session0.fork(Uuid::new_v4());
        let id0 = session0.id.to_string();
        assert_eq!(
            session1.lineage_path, None,
            "First fork must have lineage_path = None"
        );
        assert_eq!(
            session1.compute_lineage_path(),
            Some(id0.clone()),
            "compute_lineage_path for first fork must return parent ID"
        );

        let session2 = session1.fork(Uuid::new_v4());
        let id1 = session1.id.to_string();
        assert_eq!(
            session2.lineage_path,
            Some(id0.clone()),
            "Second fork lineage_path must be grandparent ID"
        );
        assert_eq!(
            session2.compute_lineage_path(),
            Some(format!("{}/{}", id0, id1)),
            "compute_lineage_path must return full path"
        );

        let session3 = session2.fork(Uuid::new_v4());
        let id2 = session2.id.to_string();
        assert_eq!(
            session3.lineage_path,
            Some(format!("{}/{}", id0, id1)),
            "Third fork lineage_path must be grandparent/parent"
        );
        assert_eq!(
            session3.compute_lineage_path(),
            Some(format!("{}/{}/{}", id0, id1, id2)),
            "compute_lineage_path must return full 3-level path"
        );
    }

    #[test]
    fn test_ownership_tree_share_mode_enum_consistency() {
        // Test that ShareMode enum values are consistent
        let mut session = Session::new();

        // Manual mode
        session.set_share_mode(ShareMode::Manual);
        assert!(session.generate_share_link().is_ok());
        session.set_share_mode(ShareMode::Disabled);

        // Auto mode
        session.set_share_mode(ShareMode::Auto);
        assert!(session.generate_share_link().is_ok());
        session.set_share_mode(ShareMode::Disabled);

        // Disabled mode
        session.set_share_mode(ShareMode::Disabled);
        assert_eq!(
            session.generate_share_link().unwrap_err(),
            ShareError::SharingDisabled
        );
    }

    #[test]
    fn test_ownership_tree_fork_preserves_tool_invocations() {
        // Test that fork correctly handles tool invocation records
        let mut parent = Session::new();
        parent.add_message(Message::user("test"));

        let child = parent.fork(Uuid::new_v4());

        // Invariant: fork transfers tool_invocations (for audit trail)
        assert_eq!(
            child.tool_invocations.len(),
            parent.tool_invocations.len(),
            "Child must have same tool_invocations as parent"
        );
    }

    #[test]
    fn test_ownership_tree_fork_at_message_invalid_index() {
        let mut parent = Session::new();
        parent.add_message(Message::user("only one"));

        let result = parent.fork_at_message(5);

        assert!(result.is_err());
        assert_eq!(
            result.unwrap_err(),
            ForkError::MessageIndexOutOfBounds {
                requested: 5,
                len: 1,
            }
        );
    }

    #[test]
    fn test_ownership_tree_share_link_url_format() {
        // Test that share link URL follows correct format invariant
        let mut session = Session::new();
        session.set_share_mode(ShareMode::Manual);
        let link = session.generate_share_link().unwrap();

        // Invariant: link format is https://opencode-rs.local/share/{share_id}
        assert!(link.starts_with("https://opencode-rs.local/share/"));
        let share_id = link
            .strip_prefix("https://opencode-rs.local/share/")
            .unwrap();
        assert!(!share_id.is_empty(), "Share ID must be non-empty");
        // UUID format check (36 chars with hyphens)
        assert_eq!(share_id.len(), 36, "Share ID must be UUID format");
    }

    #[test]
    fn test_ownership_tree_is_shared_logic() {
        // Test the complete is_shared() invariant logic
        let mut session = Session::new();

        // Case 1: no shared_id -> not shared
        assert!(!session.is_shared());

        // Case 2: shared_id exists but mode is Disabled -> not shared
        session.shared_id = Some("test".to_string());
        session.share_mode = Some(ShareMode::Disabled);
        assert!(!session.is_shared());

        // Case 3: shared_id exists, mode is Manual, no expiry -> shared
        session.share_mode = Some(ShareMode::Manual);
        assert!(session.is_shared());

        // Case 4: shared_id exists, mode is Manual, expired -> not shared
        session.share_expires_at = Some(Utc::now() - chrono::Duration::hours(1));
        assert!(!session.is_shared());

        // Case 5: shared_id exists, mode is Manual, future expiry -> shared
        session.share_expires_at = Some(Utc::now() + chrono::Duration::hours(1));
        assert!(session.is_shared());
    }

    #[test]
    fn test_ownership_tree_session_state_machine_is_independent() {
        // Test that session state is independent from ownership state
        let mut session = Session::new();

        // Initial state
        assert_eq!(session.state, SessionState::Idle);

        // Fork a child - parent state is independent
        let child = session.fork(Uuid::new_v4());
        assert_eq!(session.state, SessionState::Idle);
        assert_eq!(child.state, SessionState::Idle);

        // Change parent state
        session.set_state(SessionState::Thinking).unwrap();
        assert_eq!(session.state, SessionState::Thinking);
        // Child state is independent copy
        assert_eq!(child.state, SessionState::Idle);

        // Sharing doesn't affect state
        session.set_share_mode(ShareMode::Manual);
        session.generate_share_link().unwrap();
        assert_eq!(session.state, SessionState::Thinking);
        assert!(session.is_shared());
    }

    #[test]
    fn test_ownership_tree_export_preserves_ownership_metadata() {
        // Test that export includes ownership-related metadata
        let mut session = Session::new();
        session.add_message(Message::user("test"));
        session.set_share_mode(ShareMode::Manual);
        session.generate_share_link().unwrap();

        let json_export = session.export_json().unwrap();

        // Invariant: exported JSON must contain session ID
        assert!(json_export.contains(&session.id.to_string()));
        // Invariant: exported JSON must contain messages
        assert!(json_export.contains("test"));
    }

    #[test]
    fn test_ownership_tree_fork_history_starts_empty() {
        let mut parent = Session::new();
        parent.add_message(Message::user("parent"));

        let child = parent.fork(Uuid::new_v4());

        assert!(
            child.fork_history.is_empty(),
            "Newly forked session must have empty fork_history"
        );
        assert!(
            parent.fork_history.is_empty(),
            "Parent session must have empty fork_history"
        );
    }

    // =========================================================================
    // Stable ID Semantics Tests (P1-2)
    // =========================================================================

    #[test]
    fn test_stable_id_session_persists_across_save_load() {
        // Test: Session ID must remain stable across save/load operations
        let tmp = tempfile::TempDir::new().unwrap();
        let path = tmp.path().join("session.json");

        let mut session = Session::new();
        let original_id = session.id;
        session.add_message(Message::user("Test message".to_string()));

        session.save(&path).unwrap();
        let loaded = Session::load(&path).unwrap();

        // Invariant: Session ID is stable across serialization
        assert_eq!(
            loaded.id, original_id,
            "Session ID must remain stable after save/load cycle"
        );
    }

    #[test]
    fn test_stable_id_session_persists_across_multiple_save_load_cycles() {
        // Test: Session ID remains stable across multiple save/load cycles
        let tmp = tempfile::TempDir::new().unwrap();
        let path = tmp.path().join("session.json");

        let mut session = Session::new();
        let original_id = session.id;
        session.add_message(Message::user("Initial".to_string()));

        // First save/load cycle
        session.save(&path).unwrap();
        let mut loaded1 = Session::load(&path).unwrap();
        assert_eq!(loaded1.id, original_id);

        // Modify and save again
        loaded1.add_message(Message::assistant("Response".to_string()));
        loaded1.save(&path).unwrap();

        // Second save/load cycle
        let loaded2 = Session::load(&path).unwrap();
        assert_eq!(
            loaded2.id, original_id,
            "Session ID must remain stable across multiple save/load cycles"
        );
    }

    #[test]
    fn test_stable_id_fork_creates_new_id_not_parent_id() {
        // Test: Forked session must have a NEW unique ID, not parent's ID
        let parent = Session::new();
        let parent_id = parent.id;

        let child = parent.fork(Uuid::new_v4());

        // Invariant: Child ID must be different from parent ID
        assert_ne!(
            child.id, parent_id,
            "Forked session must have a new unique ID"
        );
        // Invariant: Child ID must be a valid non-nil UUID
        assert!(!child.id.is_nil(), "Forked session ID must not be nil");
    }

    #[test]
    fn test_stable_id_fork_preserves_parent_id_in_parent_session_id() {
        // Test: Fork correctly establishes parent-child relationship via parent_session_id
        let parent = Session::new();
        let parent_id = parent.id.to_string();

        let child = parent.fork(Uuid::new_v4());

        // Invariant: Child's parent_session_id references the parent
        assert_eq!(
            child.parent_session_id.as_deref(),
            Some(parent_id.as_str()),
            "Child must reference parent ID via parent_session_id"
        );
    }

    #[test]
    fn test_stable_id_unique_across_multiple_sessions() {
        // Test: Each session must have a globally unique ID
        let session1 = Session::new();
        let session2 = Session::new();
        let session3 = Session::new();

        // Invariant: All session IDs must be unique
        assert_ne!(
            session1.id, session2.id,
            "Each session must have a unique ID"
        );
        assert_ne!(
            session2.id, session3.id,
            "Each session must have a unique ID"
        );
        assert_ne!(
            session1.id, session3.id,
            "Each session must have a unique ID"
        );
    }

    #[test]
    fn test_stable_id_lineage_path_format_with_sessions() {
        // Test: Lineage path correctly encodes session ID hierarchy
        let grandparent = Session::new();
        let grandparent_id = grandparent.id.to_string();

        let parent = grandparent.fork(Uuid::new_v4());
        let parent_id = parent.id.to_string();

        let child = parent.fork(Uuid::new_v4());
        let child_lineage = child.compute_lineage_path();

        // Invariant: Lineage path is "grandparent_id/parent_id" format
        assert_eq!(
            child_lineage,
            Some(format!("{}/{}", grandparent_id, parent_id)),
            "Lineage path must be grandparent_id/parent_id"
        );
    }

    #[test]
    fn test_stable_id_lineage_persists_through_save_load() {
        // Test: Lineage information is preserved through serialization
        let grandparent = Session::new();
        let parent = grandparent.fork(Uuid::new_v4());
        let child = parent.fork(Uuid::new_v4());

        let grandparent_id = grandparent.id.to_string();
        let parent_id = parent.id.to_string();

        let tmp = tempfile::TempDir::new().unwrap();
        let path = tmp.path().join("lineage_session.json");
        child.save(&path).unwrap();

        let loaded = Session::load(&path).unwrap();

        // Invariant: Lineage is preserved through save/load
        assert_eq!(
            loaded.lineage_path,
            Some(grandparent_id.clone()),
            "Lineage path must survive serialization"
        );
        assert_eq!(
            loaded.compute_lineage_path(),
            Some(format!("{}/{}", grandparent_id, parent_id)),
            "Computed lineage must be preserved"
        );
    }

    #[test]
    fn test_stable_id_for_fork_at_message() {
        // Test: fork_at_message creates new unique ID
        let mut parent = Session::new();
        parent.add_message(Message::user("first"));
        parent.add_message(Message::assistant("second"));
        let parent_id = parent.id;

        let child = parent.fork_at_message(0).unwrap();

        // Invariant: Child has new unique ID
        assert_ne!(
            child.id, parent_id,
            "fork_at_message must create session with new unique ID"
        );
        assert!(
            !child.id.is_nil(),
            "fork_at_message session ID must not be nil"
        );
    }

    #[test]
    fn test_stable_id_tool_invocation_record_ids() {
        // Test: ToolInvocationRecord entries have unique IDs
        let mut session = Session::new();
        session.add_message(Message::user("test"));

        let record1_id = session.tool_invocations.first().map(|r| r.id);
        let record2_id = session.tool_invocations.last().map(|r| r.id);

        // After adding first message, there may be tool invocations
        // Each should have unique IDs if present
        if let (Some(id1), Some(id2)) = (record1_id, record2_id) {
            assert_ne!(id1, id2, "Tool invocation record IDs must be unique");
        }
    }

    #[test]
    fn test_stable_id_share_id_differs_from_session_id() {
        // Test: Share ID is separate from session ID
        let mut session = Session::new();
        session.generate_share_link().unwrap();

        let session_id = session.id.to_string();
        let share_id = session.get_share_id().unwrap();

        // Invariant: Share ID is not the same as session ID
        assert_ne!(
            share_id, session_id,
            "Share ID must be different from session ID"
        );
    }

    #[test]
    fn test_stable_id_checkpoint_preserves_session_id() {
        // Test: Checkpoint operation preserves the session ID
        let tmp = tempfile::TempDir::new().unwrap();

        let mut session = Session::new();
        session.add_message(Message::user("checkpoint test".to_string()));
        let original_session_id = session.id;

        let manager = CheckpointManager {
            checkpoints_dir: tmp.path().to_path_buf(),
            max_checkpoints: 5,
        };

        let checkpoint = manager.create(&session, "Test checkpoint").unwrap();

        // Invariant: Checkpoint's session_id matches original session's ID
        assert_eq!(
            checkpoint.session_id, original_session_id,
            "Checkpoint must preserve the session ID"
        );

        // Load checkpoint and verify session ID is preserved
        let loaded = manager.load(&session.id, 0).unwrap();
        assert_eq!(
            loaded.id, original_session_id,
            "Loaded session from checkpoint must have same ID"
        );
    }

    #[test]
    fn test_stable_id_multiple_checkpoints_same_session() {
        // Test: Multiple checkpoints all reference the same session ID
        let tmp = tempfile::TempDir::new().unwrap();

        let mut session = Session::new();
        session.add_message(Message::user("message 1".to_string()));
        let original_session_id = session.id;

        let manager = CheckpointManager {
            checkpoints_dir: tmp.path().to_path_buf(),
            max_checkpoints: 10,
        };

        // Create first checkpoint
        manager.create(&session, "Checkpoint 1").unwrap();

        // Add more messages and create second checkpoint
        session.add_message(Message::assistant("message 2".to_string()));
        manager.create(&session, "Checkpoint 2").unwrap();

        // Verify both checkpoints reference the same session
        let checkpoints = manager.list(&session.id).unwrap();
        assert_eq!(checkpoints.len(), 2);

        for cp in checkpoints {
            assert_eq!(
                cp.session_id, original_session_id,
                "All checkpoints must reference the original session ID"
            );
        }
    }

    #[test]
    fn test_stable_id_checkpoint_load_returns_same_session_id() {
        // Test: Loading a checkpoint returns a session with the same ID
        let tmp = tempfile::TempDir::new().unwrap();

        let mut session = Session::new();
        session.add_message(Message::user("Original message".to_string()));
        let original_id = session.id;

        let manager = CheckpointManager {
            checkpoints_dir: tmp.path().to_path_buf(),
            max_checkpoints: 5,
        };

        manager.create(&session, "Test").unwrap();

        // Load checkpoint
        let loaded = manager.load(&session.id, 0).unwrap();

        // Invariant: Loaded session has same ID as original
        assert_eq!(
            loaded.id, original_id,
            "Session loaded from checkpoint must have identical ID"
        );
    }

    #[test]
    fn test_stable_id_forked_session_can_be_independently_checkpointed() {
        // Test: Forked session can be checkpointed independently with its own ID
        let tmp = tempfile::TempDir::new().unwrap();

        let parent = Session::new();
        let child = parent.fork(Uuid::new_v4());

        let parent_manager = CheckpointManager {
            checkpoints_dir: tmp.path().join("parent_checkpoints").to_path_buf(),
            max_checkpoints: 5,
        };

        let child_manager = CheckpointManager {
            checkpoints_dir: tmp.path().join("child_checkpoints").to_path_buf(),
            max_checkpoints: 5,
        };

        let parent_cp = parent_manager.create(&parent, "Parent checkpoint").unwrap();
        let child_cp = child_manager.create(&child, "Child checkpoint").unwrap();

        // Invariant: Parent and child checkpoints have different session IDs
        assert_ne!(
            parent_cp.session_id, child_cp.session_id,
            "Parent and child sessions must have different IDs"
        );

        // Invariant: Each checkpoint references its own session
        assert_eq!(parent_cp.session_id, parent.id);
        assert_eq!(child_cp.session_id, child.id);
    }

    #[test]
    fn test_stable_id_session_id_is_uuid_format() {
        // Test: Session ID follows UUID format
        let session = Session::new();
        let id_str = session.id.to_string();

        // UUID format: 8-4-4-4-12 (36 chars with hyphens)
        assert_eq!(id_str.len(), 36, "UUID must be 36 characters");
        assert!(
            id_str.chars().all(|c| c.is_ascii_hexdigit() || c == '-'),
            "UUID must contain only hex digits and hyphens"
        );
    }

    #[test]
    fn test_stable_id_timestamp_does_not_affect_session_id() {
        // Test: Session ID remains stable regardless of timestamp changes
        let tmp = tempfile::TempDir::new().unwrap();
        let path = tmp.path().join("session.json");

        let mut session = Session::new();
        let original_id = session.id;

        // Add multiple messages (changes updated_at)
        session.add_message(Message::user("msg1".to_string()));
        session.add_message(Message::assistant("msg2".to_string()));
        session.add_message(Message::user("msg3".to_string()));

        // Save/load cycle
        session.save(&path).unwrap();
        let loaded = Session::load(&path).unwrap();

        // Invariant: ID is unchanged despite timestamp updates
        assert_eq!(
            loaded.id, original_id,
            "Session ID must remain stable despite timestamp changes"
        );
        // Verify timestamps did change
        assert!(loaded.updated_at > loaded.created_at);
    }

    // =========================================================================
    // Ownership Tree Acyclicity Tests (P1-4)
    // =========================================================================

    fn build_session_graph(sessions: &[Session]) -> std::collections::HashMap<Uuid, Option<Uuid>> {
        let mut graph: std::collections::HashMap<Uuid, Option<Uuid>> =
            std::collections::HashMap::new();
        for session in sessions {
            graph.insert(
                session.id,
                session
                    .parent_session_id
                    .as_ref()
                    .and_then(|s| s.parse().ok()),
            );
        }
        graph
    }

    fn detect_cycle_in_graph(
        graph: &std::collections::HashMap<Uuid, Option<Uuid>>,
        start_id: Uuid,
    ) -> bool {
        let mut visited = std::collections::HashSet::new();
        let mut rec_stack = std::collections::HashSet::new();

        fn dfs(
            graph: &std::collections::HashMap<Uuid, Option<Uuid>>,
            node_id: Uuid,
            visited: &mut std::collections::HashSet<Uuid>,
            rec_stack: &mut std::collections::HashSet<Uuid>,
        ) -> bool {
            if rec_stack.contains(&node_id) {
                return true;
            }

            if visited.contains(&node_id) {
                return false;
            }

            visited.insert(node_id);
            rec_stack.insert(node_id);

            if let Some(parent_id) = graph.get(&node_id).and_then(|p| *p) {
                if dfs(graph, parent_id, visited, rec_stack) {
                    return true;
                }
            }

            rec_stack.remove(&node_id);
            false
        }

        dfs(graph, start_id, &mut visited, &mut rec_stack)
    }

    #[test]
    fn test_ownership_tree_acyclicity_simple_session() {
        // Test: A single session with no parent has no cycle
        let session = Session::new();
        let sessions = vec![session.clone()];
        let graph = build_session_graph(&sessions);

        let has_cycle = detect_cycle_in_graph(&graph, session.id);
        assert!(!has_cycle, "Single root session cannot have a cycle");
    }

    #[test]
    fn test_ownership_tree_acyclicity_simple_fork_chain() {
        // Test: A simple parent -> child fork chain is acyclic
        let parent = Session::new();
        let child = parent.fork(Uuid::new_v4());

        let sessions = vec![parent.clone(), child.clone()];
        let graph = build_session_graph(&sessions);

        // Check child has no cycle (its parent exists and is not itself)
        let child_has_cycle = detect_cycle_in_graph(&graph, child.id);
        assert!(
            !child_has_cycle,
            "Child session in simple fork chain cannot have cycle"
        );

        // Check parent has no cycle
        let parent_has_cycle = detect_cycle_in_graph(&graph, parent.id);
        assert!(
            !parent_has_cycle,
            "Parent session in simple fork chain cannot have cycle"
        );
    }

    #[test]
    fn test_ownership_tree_acyclicity_multi_level_fork_chain() {
        // Test: A multi-level fork chain (grandparent -> parent -> child) is acyclic
        let grandparent = Session::new();
        let parent = grandparent.fork(Uuid::new_v4());
        let child = parent.fork(Uuid::new_v4());

        let sessions = vec![grandparent.clone(), parent.clone(), child.clone()];
        let graph = build_session_graph(&sessions);

        // Verify no session in the chain has a cycle
        for session in &[&grandparent, &parent, &child] {
            let has_cycle = detect_cycle_in_graph(&graph, session.id);
            assert!(
                !has_cycle,
                "Session in multi-level fork chain cannot have cycle"
            );
        }

        // Verify lineage_path is correct (no cycles in path)
        assert_eq!(
            child.lineage_path,
            Some(grandparent.id.to_string()),
            "Child's lineage_path must reference grandparent"
        );
    }

    #[test]
    fn test_ownership_tree_acyclicity_many_forks() {
        // Test: A chain of many forks remains acyclic
        let mut sessions = vec![Session::new()];

        // Create a chain of 10 forks
        for _ in 0..10 {
            let last = sessions.last().unwrap();
            sessions.push(last.fork(Uuid::new_v4()));
        }

        let graph = build_session_graph(&sessions);

        // Verify all sessions in the chain are acyclic
        for session in &sessions {
            let has_cycle = detect_cycle_in_graph(&graph, session.id);
            assert!(
                !has_cycle,
                "Session in 10-level fork chain cannot have cycle"
            );
        }

        // Verify the final session has correct deep lineage
        let final_session = sessions.last().unwrap();
        let lineage_ids: Vec<Uuid> = final_session
            .lineage_path
            .as_ref()
            .map(|p| p.split('/').filter_map(|s| s.parse().ok()).collect())
            .unwrap_or_default();
        assert_eq!(lineage_ids.len(), 9, "9-level deep lineage expected");
    }

    #[test]
    fn test_ownership_tree_no_self_reference() {
        // Test: A session's parent_session_id cannot point to itself
        let mut session = Session::new();
        session.parent_session_id = Some(session.id.to_string());

        let sessions = vec![session.clone()];
        let graph = build_session_graph(&sessions);

        let has_cycle = detect_cycle_in_graph(&graph, session.id);
        assert!(has_cycle, "Session with self-reference must have cycle");
    }

    #[test]
    fn test_ownership_tree_direct_parent_no_cycle() {
        // Test: A session A pointing to parent B where B is root has no cycle
        let mut session_a = Session::new();
        let session_b = Session::new();

        // A points to B, B is root (no parent)
        session_a.parent_session_id = Some(session_b.id.to_string());

        let sessions = vec![session_a.clone(), session_b.clone()];
        let graph = build_session_graph(&sessions);

        // This should NOT be a cycle because B has no parent to return to
        let a_has_cycle = detect_cycle_in_graph(&graph, session_a.id);
        assert!(
            !a_has_cycle,
            "Session A pointing to root session B must not have cycle"
        );
    }

    #[test]
    fn test_ownership_tree_indirect_cycle_detection() {
        // Test: Detect indirect cycle where A->B->C->A
        let mut session_a = Session::new();
        let mut session_b = Session::new();
        let session_c = Session::new();

        session_a.parent_session_id = Some(session_b.id.to_string());
        session_b.parent_session_id = Some(session_c.id.to_string());
        // session_c.parent_session_id = None (root) - but we need to make it cycle back

        // For cycle: C should point back to A
        // This creates A->B->C->A cycle
        let sessions = vec![session_a.clone(), session_b.clone(), session_c.clone()];
        let mut graph = build_session_graph(&sessions);

        // Create the cycle by making C point back to A
        graph.insert(session_c.id, Some(session_a.id));

        let has_cycle = detect_cycle_in_graph(&graph, session_a.id);
        assert!(has_cycle, "Indirect cycle A->B->C->A must be detected");
    }

    // =========================================================================
    // Fork Acyclicity Tests (P1-4)
    // =========================================================================

    #[test]
    fn test_fork_acyclicity_simple() {
        // Test: Simple fork creates acyclic parent-child relationship
        let parent = Session::new();
        let child = parent.fork(Uuid::new_v4());

        // Child should have parent reference
        assert_eq!(
            child.parent_session_id.as_deref(),
            Some(parent.id.to_string().as_str()),
            "Child must reference parent"
        );

        // Parent should NOT have parent reference
        assert!(
            parent.parent_session_id.is_none(),
            "Parent must not have parent reference"
        );

        // Lineage path computation must not create cycles
        let child_lineage = child.compute_lineage_path();
        assert!(child_lineage.is_some(), "Child must have lineage path");
        assert!(
            !child_lineage.unwrap().contains(&child.id.to_string()),
            "Lineage path must not contain child's own ID"
        );
    }

    #[test]
    fn test_fork_acyclicity_multi_generation() {
        // Test: Multiple generations of forks maintain acyclicity
        let gen0 = Session::new();
        let gen1 = gen0.fork(Uuid::new_v4());
        let gen2 = gen1.fork(Uuid::new_v4());
        let gen3 = gen2.fork(Uuid::new_v4());

        // Verify parent references form a proper chain (no cycles)
        assert_eq!(
            gen3.parent_session_id.as_deref(),
            Some(gen2.id.to_string().as_str()),
            "Gen3 must reference Gen2"
        );
        assert_eq!(
            gen2.parent_session_id.as_deref(),
            Some(gen1.id.to_string().as_str()),
            "Gen2 must reference Gen1"
        );
        assert_eq!(
            gen1.parent_session_id.as_deref(),
            Some(gen0.id.to_string().as_str()),
            "Gen1 must reference Gen0"
        );
        assert!(
            gen0.parent_session_id.is_none(),
            "Gen0 must be root (no parent)"
        );

        // Verify lineage path is correct and acyclic
        let lineage = gen3.lineage_path.as_deref();
        assert!(lineage.is_some(), "Gen3 must have lineage path");
        let lineage_str = lineage.unwrap();
        assert!(
            !lineage_str.contains(&gen3.id.to_string()),
            "Lineage must not contain own ID"
        );
        assert!(
            lineage_str.contains(&gen0.id.to_string()),
            "Lineage must contain root ID"
        );
    }

    #[test]
    fn test_fork_acyclicity_after_serialization() {
        // Test: Fork chain remains acyclic after save/load cycle
        let grandparent = Session::new();
        let parent = grandparent.fork(Uuid::new_v4());
        let child = parent.fork(Uuid::new_v4());

        // Save and reload
        let tmp = tempfile::TempDir::new().unwrap();
        let path = tmp.path().join("session.json");
        child.save(&path).unwrap();
        let loaded = Session::load(&path).unwrap();

        // Verify acyclicity invariants preserved
        assert!(
            loaded.parent_session_id.is_some(),
            "Loaded session must have parent reference"
        );
        assert!(
            loaded.lineage_path.is_some(),
            "Loaded session must have lineage path"
        );
        assert!(
            !loaded
                .lineage_path
                .as_ref()
                .unwrap()
                .contains(&loaded.id.to_string()),
            "Lineage must not contain own ID"
        );

        // Verify lineage chain is still correct
        let grandparent_id = grandparent.id.to_string();
        assert_eq!(
            loaded.lineage_path.as_deref(),
            Some(grandparent_id.as_str()),
            "Loaded lineage must still reference grandparent"
        );
    }

    #[test]
    fn test_fork_acyclicity_lineage_path_integrity() {
        // Test: Lineage path maintains integrity and cannot create cycles
        let session0 = Session::new();
        let session1 = session0.fork(Uuid::new_v4());
        let session2 = session1.fork(Uuid::new_v4());

        // Build expected lineage
        let expected_lineage = format!("{}/{}", session0.id, session1.id);

        assert_eq!(
            session2.compute_lineage_path().as_deref(),
            Some(expected_lineage.as_str()),
            "Computed lineage path must be correct"
        );

        // Verify no ID appears twice in the lineage
        let full_lineage = session2.compute_lineage_path().unwrap();
        let lineage_parts: Vec<&str> = full_lineage.split('/').collect();
        let mut seen = std::collections::HashSet::new();
        for part in &lineage_parts {
            assert!(
                seen.insert(*part),
                "Duplicate ID in lineage path indicates cycle"
            );
        }

        // Verify session's own ID is not in lineage
        assert!(
            !lineage_parts.contains(&session2.id.to_string().as_str()),
            "Session's own ID must not appear in its lineage"
        );
    }

    #[test]
    fn test_fork_acyclicity_multiple_children_from_same_parent() {
        // Test: Multiple forks from same parent all maintain acyclicity
        let parent = Session::new();
        let child1 = parent.fork(Uuid::new_v4());
        let child2 = parent.fork(Uuid::new_v4());
        let child3 = parent.fork(Uuid::new_v4());

        // All children should reference same parent
        let parent_id = parent.id.to_string();
        assert_eq!(
            child1.parent_session_id.as_deref(),
            Some(parent_id.as_str()),
            "Child1 must reference parent"
        );
        assert_eq!(
            child2.parent_session_id.as_deref(),
            Some(parent_id.as_str()),
            "Child2 must reference parent"
        );
        assert_eq!(
            child3.parent_session_id.as_deref(),
            Some(parent_id.as_str()),
            "Child3 must reference parent"
        );

        // None should have cycles
        let sessions = vec![
            parent.clone(),
            child1.clone(),
            child2.clone(),
            child3.clone(),
        ];
        let graph = build_session_graph(&sessions);

        for child in &[&child1, &child2, &child3] {
            let has_cycle = detect_cycle_in_graph(&graph, child.id);
            assert!(!has_cycle, "Child from same parent cannot have cycle");
        }
    }

    #[test]
    fn test_fork_acyclicity_fork_at_message_preserves_chain() {
        // Test: fork_at_message preserves the acyclic parent-child chain
        let mut parent = Session::new();
        parent.add_message(Message::user("msg1"));
        parent.add_message(Message::assistant("msg2"));
        parent.add_message(Message::user("msg3"));

        let child = parent.fork_at_message(1).unwrap();

        // Child must reference parent
        assert_eq!(
            child.parent_session_id.as_deref(),
            Some(parent.id.to_string().as_str()),
            "Child must reference parent"
        );

        // Child's lineage must be correct
        assert_eq!(
            child.lineage_path.as_deref(),
            None,
            "First fork lineage_path is None (computed path is parent ID)"
        );

        // Verify child has correct messages (only up to index 1)
        assert_eq!(child.messages.len(), 2, "Child must have messages[0..=1]");

        // Lineage must not contain own ID
        let child_lineage = child.compute_lineage_path();
        assert!(
            !child_lineage
                .as_ref()
                .unwrap_or(&String::new())
                .contains(&child.id.to_string()),
            "Lineage must not contain own ID"
        );
    }

    #[test]
    fn test_fork_acyclicity_deep_chain_verify_no_back_reference() {
        // Test: Deep fork chain never creates back-references that could cause cycles
        let mut sessions = vec![Session::new()];

        // Create 5-level chain
        for _ in 0..5 {
            let last = sessions.last().unwrap();
            sessions.push(last.fork(Uuid::new_v4()));
        }

        // Verify no session in chain has parent_session_id pointing forward
        for (i, session) in sessions.iter().enumerate() {
            if let Some(parent_id_str) = &session.parent_session_id {
                if let Ok(parent_id) = parent_id_str.parse::<Uuid>() {
                    // Parent must be in sessions before this one
                    let parent_exists_before = sessions[..i].iter().any(|s| s.id == parent_id);
                    assert!(
                        parent_exists_before,
                        "Session at index {} has invalid parent reference (parent not in earlier chain)",
                        i
                    );

                    // Parent ID must not equal this session's ID
                    assert_ne!(session.id, parent_id, "Session cannot be its own parent");
                }
            }
        }
    }

    #[test]
    fn test_fork_acyclicity_compute_lineage_path_never_self_references() {
        // Test: compute_lineage_path() never returns a path containing the session's own ID
        let session = Session::new();

        // New session has no lineage
        let lineage = session.compute_lineage_path();
        assert!(lineage.is_none() || !lineage.as_ref().unwrap().contains(&session.id.to_string()));

        // After fork, verify compute_lineage_path doesn't include own ID
        let child = session.fork(Uuid::new_v4());
        let child_lineage = child.compute_lineage_path();
        if let Some(ref path) = child_lineage {
            assert!(
                !path.contains(&child.id.to_string()),
                "compute_lineage_path must not include own ID"
            );
        }
    }
}
