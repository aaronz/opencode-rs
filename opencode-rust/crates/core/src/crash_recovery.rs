use crate::{message::Role, Session, ToolInvocationRecord};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::fs;
use std::panic::{set_hook, take_hook, PanicHookInfo};
use std::path::PathBuf;
use std::sync::Mutex;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CrashDump {
    pub version: String,
    pub crashed_at: DateTime<Utc>,
    pub session_id: String,
    pub messages_summary: Vec<MessageSummary>,
    pub tool_invocations_summary: Vec<ToolInvocationSummary>,
    pub state: String,
    pub panic_message: Option<String>,
    pub stack_trace: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MessageSummary {
    pub role: String,
    pub content_preview: String,
    pub timestamp: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolInvocationSummary {
    pub tool_name: String,
    pub started_at: DateTime<Utc>,
    pub completed_at: Option<DateTime<Utc>>,
}

pub struct CrashRecovery {
    dump_dir: PathBuf,
    active_session: Mutex<Option<ActiveSession>>,
}

#[derive(Debug, Clone)]
struct ActiveSession {
    session: Session,
}

impl ActiveSession {
    fn capture_messages_summary(&self, max_messages: usize) -> Vec<MessageSummary> {
        let messages_to_capture = self.session.messages.len().min(max_messages);
        self.session
            .messages
            .iter()
            .rev()
            .take(messages_to_capture)
            .map(|msg| {
                let preview = if msg.content.len() > 200 {
                    format!("{}...", &msg.content[..200])
                } else {
                    msg.content.clone()
                };
                MessageSummary {
                    role: format!("{:?}", msg.role),
                    content_preview: preview,
                    timestamp: msg.timestamp,
                }
            })
            .collect()
    }

    fn capture_tool_invocations_summary(
        &self,
        max_invocations: usize,
    ) -> Vec<ToolInvocationSummary> {
        self.session
            .tool_invocations
            .iter()
            .rev()
            .take(max_invocations)
            .map(|inv| ToolInvocationSummary {
                tool_name: inv.tool_name.clone(),
                started_at: inv.started_at,
                completed_at: inv.completed_at,
            })
            .collect()
    }
}

impl Default for CrashRecovery {
    fn default() -> Self {
        Self::new()
    }
}

impl CrashRecovery {
    pub fn new() -> Self {
        let dump_dir = std::env::var("OPENCODE_CRASH_DUMP_DIR")
            .map(PathBuf::from)
            .unwrap_or_else(|_| {
                dirs::home_dir()
                    .map(|h| h.join(".config/opencode-rs/crashes"))
                    .unwrap_or_else(|| PathBuf::from(".opencode-rs/crashes"))
            });

        Self {
            dump_dir,
            active_session: Mutex::new(None),
        }
    }

    pub fn with_dump_dir(mut self, dump_dir: PathBuf) -> Self {
        self.dump_dir = dump_dir;
        self
    }

    pub fn dump_dir(&self) -> &PathBuf {
        &self.dump_dir
    }

    pub fn set_active_session(&self, session: Session) {
        let mut guard = self
            .active_session
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        *guard = Some(ActiveSession { session });
    }

    pub fn get_active_session(&self) -> Option<Session> {
        let guard = self
            .active_session
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        guard.as_ref().map(|a| a.session.clone())
    }

    pub fn clear_active_session(&self) {
        let mut guard = self
            .active_session
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        *guard = None;
    }

    pub fn save_crash_dump(
        &self,
        panic_message: Option<String>,
        stack_trace: Option<String>,
    ) -> Result<PathBuf, CrashRecoveryError> {
        let guard = self
            .active_session
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        let active = guard.as_ref().ok_or(CrashRecoveryError::NoActiveSession)?;

        let messages_summary = active.capture_messages_summary(20);
        let tool_invocations_summary = active.capture_tool_invocations_summary(50);

        let state = if active.session.state == crate::SessionState::Error {
            "error".to_string()
        } else {
            format!("{:?}", active.session.state)
        };

        let crash_dump = CrashDump {
            version: env!("CARGO_PKG_VERSION").to_string(),
            crashed_at: Utc::now(),
            session_id: active.session.id.to_string(),
            messages_summary,
            tool_invocations_summary,
            state,
            panic_message,
            stack_trace,
        };

        self.write_dump(&crash_dump)
    }

    fn write_dump(&self, dump: &CrashDump) -> Result<PathBuf, CrashRecoveryError> {
        if let Some(parent) = self.dump_dir.parent() {
            fs::create_dir_all(parent)?;
        }
        fs::create_dir_all(&self.dump_dir)?;

        let filename = format!(
            "crash_{}_{}.json",
            dump.session_id,
            dump.crashed_at.timestamp()
        );
        let path = self.dump_dir.join(filename);

        let json = serde_json::to_string_pretty(dump)
            .map_err(|e| CrashRecoveryError::SerializationError(e.to_string()))?;

        fs::write(&path, json)?;
        Ok(path)
    }

    pub fn find_crash_dumps(&self, session_id: &str) -> Vec<PathBuf> {
        let mut dumps = Vec::new();
        if !self.dump_dir.exists() {
            return dumps;
        }

        if let Ok(entries) = fs::read_dir(&self.dump_dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if let Some(filename) = path.file_name().and_then(|n| n.to_str()) {
                    if filename.starts_with(&format!("crash_{}_", session_id))
                        && filename.ends_with(".json")
                    {
                        dumps.push(path);
                    }
                }
            }
        }

        dumps.sort_by_key(|p| {
            p.file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("invalid_filename")
                .to_string()
        });
        dumps.reverse();
        dumps
    }

    pub fn load_crash_dump(&self, path: &PathBuf) -> Result<CrashDump, CrashRecoveryError> {
        let content = fs::read_to_string(path)?;
        serde_json::from_str(&content)
            .map_err(|e| CrashRecoveryError::DeserializationError(e.to_string()))
    }

    pub fn recover_session(&self, path: &PathBuf) -> Result<Session, CrashRecoveryError> {
        let dump = self.load_crash_dump(path)?;

        let messages: Vec<crate::Message> = dump
            .messages_summary
            .iter()
            .map(|summary| {
                let role = match summary.role.as_str() {
                    "User" => Role::User,
                    "Assistant" => Role::Assistant,
                    "System" => Role::System,
                    _ => Role::User,
                };
                crate::Message::new(role, summary.content_preview.clone())
            })
            .collect();

        let tool_invocations: Vec<ToolInvocationRecord> = dump
            .tool_invocations_summary
            .iter()
            .map(|summary| ToolInvocationRecord {
                id: uuid::Uuid::new_v4(),
                tool_name: summary.tool_name.clone(),
                arguments: serde_json::Value::Null,
                args_hash: String::new(),
                result: None,
                started_at: summary.started_at,
                completed_at: summary.completed_at,
                latency_ms: summary
                    .completed_at
                    .map(|end| (end - summary.started_at).num_milliseconds() as u64),
            })
            .collect();

        let session_id = match uuid::Uuid::parse_str(&dump.session_id) {
            Ok(id) => id,
            Err(_) => uuid::Uuid::new_v4(),
        };

        let mut session = Session::new();
        session.id = session_id;
        session.messages = messages;
        session.updated_at = dump.crashed_at;

        for invocation in tool_invocations {
            session.tool_invocations.push(invocation);
        }

        Ok(session)
    }

    pub fn recover_session_latest(
        &self,
        session_id: &str,
    ) -> Result<Option<Session>, CrashRecoveryError> {
        let dumps = self.find_crash_dumps(session_id);
        if let Some(latest) = dumps.first() {
            Ok(Some(self.recover_session(latest)?))
        } else {
            Ok(None)
        }
    }

    pub fn list_recent_crashes(&self, limit: usize) -> Vec<CrashDump> {
        let mut crashes = Vec::new();
        if !self.dump_dir.exists() {
            return crashes;
        }

        if let Ok(entries) = fs::read_dir(&self.dump_dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.extension().and_then(|e| e.to_str()) == Some("json") {
                    if let Ok(dump) = self.load_crash_dump(&path) {
                        crashes.push(dump);
                    }
                }
            }
        }

        crashes.sort_by(|a, b| b.crashed_at.cmp(&a.crashed_at));
        crashes.truncate(limit);
        crashes
    }

    pub fn delete_crash_dump(&self, path: &PathBuf) -> Result<(), CrashRecoveryError> {
        fs::remove_file(path).map_err(|e| CrashRecoveryError::IoError(e.to_string()))
    }

    pub fn cleanup_session_crashes(&self, session_id: &str) -> Result<usize, CrashRecoveryError> {
        let dumps = self.find_crash_dumps(session_id);
        let count = dumps.len();
        for dump in dumps {
            self.delete_crash_dump(&dump)?;
        }
        Ok(count)
    }

    pub fn has_recoverable_crash(&self, session_id: &str) -> bool {
        !self.find_crash_dumps(session_id).is_empty()
    }
}

#[allow(clippy::type_complexity)]
pub struct PanicHandler {
    crash_recovery: CrashRecovery,
    previous_hook: Option<Box<dyn Fn(&PanicHookInfo<'_>) + Send + Sync>>,
}

#[allow(dead_code)]
impl PanicHandler {
    pub fn new() -> Self {
        Self {
            crash_recovery: CrashRecovery::new(),
            previous_hook: None,
        }
    }

    pub fn with_crash_recovery(crash_recovery: CrashRecovery) -> Self {
        Self {
            crash_recovery,
            previous_hook: None,
        }
    }

    pub fn start(&mut self) {
        let crash_recovery = self.crash_recovery.clone();
        let previous = take_hook();
        self.previous_hook = Some(Box::new(previous));
        set_hook(Box::new(move |panic_info: &PanicHookInfo| {
            let panic_message = panic_info
                .payload()
                .downcast_ref::<&str>()
                .map(|s| s.to_string())
                .or_else(|| panic_info.payload().downcast_ref::<String>().cloned());

            let stack_trace = panic_info
                .location()
                .map(|loc| format!("{}:{}:{}", loc.file(), loc.line(), loc.column()));

            let _ = crash_recovery.save_crash_dump(panic_message, stack_trace);
        }));
    }

    pub fn stop(&mut self) {
        if let Some(hook) = self.previous_hook.take() {
            set_hook(hook);
        }
    }
}

impl Default for PanicHandler {
    fn default() -> Self {
        Self::new()
    }
}

impl Clone for CrashRecovery {
    fn clone(&self) -> Self {
        Self {
            dump_dir: self.dump_dir.clone(),
            active_session: Mutex::new(None),
        }
    }
}

#[derive(Debug, thiserror::Error)]
pub enum CrashRecoveryError {
    #[error("no active session to save")]
    NoActiveSession,
    #[error("IO error: {0}")]
    IoError(String),
    #[error("serialization error: {0}")]
    SerializationError(String),
    #[error("deserialization error: {0}")]
    DeserializationError(String),
}

impl From<std::io::Error> for CrashRecoveryError {
    fn from(e: std::io::Error) -> Self {
        CrashRecoveryError::IoError(e.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Message;
    use tempfile::tempdir;

    fn create_test_recovery(tmp: &tempfile::TempDir) -> CrashRecovery {
        CrashRecovery::new().with_dump_dir(tmp.path().join("crashes"))
    }

    fn create_test_session() -> Session {
        let mut session = Session::new();
        session.add_message(Message::user("Hello world"));
        session.add_message(Message::assistant("Hi there!"));
        session.tool_invocations.push(ToolInvocationRecord {
            id: uuid::Uuid::new_v4(),
            tool_name: "read".to_string(),
            arguments: serde_json::json!({"path": "test.txt"}),
            args_hash: "abc123".to_string(),
            result: Some("file content".to_string()),
            started_at: chrono::Utc::now(),
            completed_at: Some(chrono::Utc::now()),
            latency_ms: Some(100),
        });
        session
    }

    #[test]
    fn test_crash_recovery_set_and_clear_session() {
        let tmp = tempdir().unwrap();
        let recovery = create_test_recovery(&tmp);

        assert!(recovery.active_session.lock().unwrap().is_none());

        let session = create_test_session();
        recovery.set_active_session(session);
        assert!(recovery.active_session.lock().unwrap().is_some());

        recovery.clear_active_session();
        assert!(recovery.active_session.lock().unwrap().is_none());
    }

    #[test]
    fn test_save_crash_dump_requires_active_session() {
        let tmp = tempdir().unwrap();
        let recovery = create_test_recovery(&tmp);

        let result = recovery.save_crash_dump(Some("test panic".to_string()), None);
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            CrashRecoveryError::NoActiveSession
        ));
    }

    #[test]
    fn test_save_and_load_crash_dump() {
        let tmp = tempdir().unwrap();
        let recovery = create_test_recovery(&tmp);

        let mut session = create_test_session();
        session.id = uuid::Uuid::parse_str("550e8400-e29b-41d4-a716-446655440000").unwrap();
        recovery.set_active_session(session);

        let path = recovery
            .save_crash_dump(
                Some("test panic".to_string()),
                Some("main.rs:100".to_string()),
            )
            .unwrap();

        assert!(path.exists());

        let loaded = recovery.load_crash_dump(&path).unwrap();
        assert_eq!(loaded.session_id, "550e8400-e29b-41d4-a716-446655440000");
        assert!(loaded.panic_message.is_some());
        assert!(loaded.stack_trace.is_some());
        assert!(!loaded.messages_summary.is_empty());
        assert!(!loaded.tool_invocations_summary.is_empty());
    }

    #[test]
    fn test_save_crash_dump_captures_session_state() {
        let tmp = tempdir().unwrap();
        let recovery = create_test_recovery(&tmp);

        let mut session = create_test_session();
        session.add_message(Message::user("Another message"));
        recovery.set_active_session(session);

        let path = recovery
            .save_crash_dump(Some("panic".to_string()), None)
            .unwrap();

        let loaded = recovery.load_crash_dump(&path).unwrap();
        assert!(!loaded.messages_summary.is_empty());
        assert_eq!(loaded.tool_invocations_summary.len(), 1);
        assert_eq!(loaded.tool_invocations_summary[0].tool_name, "read");
    }

    #[test]
    fn test_find_crash_dumps() {
        let tmp = tempdir().unwrap();
        let recovery = create_test_recovery(&tmp);

        let mut session1 = create_test_session();
        session1.id = uuid::Uuid::parse_str("11111111-1111-1111-1111-111111111111").unwrap();
        recovery.set_active_session(session1);
        recovery
            .save_crash_dump(Some("panic 1".to_string()), None)
            .unwrap();

        let mut session2 = create_test_session();
        session2.id = uuid::Uuid::parse_str("22222222-2222-2222-2222-222222222222").unwrap();
        recovery.set_active_session(session2);
        recovery
            .save_crash_dump(Some("panic 2".to_string()), None)
            .unwrap();

        let dumps1 = recovery.find_crash_dumps("11111111-1111-1111-1111-111111111111");
        assert_eq!(dumps1.len(), 1);

        let dumps2 = recovery.find_crash_dumps("22222222-2222-2222-2222-222222222222");
        assert_eq!(dumps2.len(), 1);
    }

    #[test]
    fn test_list_recent_crashes() {
        let tmp = tempdir().unwrap();
        let recovery = create_test_recovery(&tmp);

        let mut session1 = create_test_session();
        session1.id = uuid::Uuid::parse_str("aaaaaaa1-1111-1111-1111-111111111111").unwrap();
        recovery.set_active_session(session1);
        recovery
            .save_crash_dump(Some("panic A".to_string()), None)
            .unwrap();

        let mut session2 = create_test_session();
        session2.id = uuid::Uuid::parse_str("aaaaaaa2-2222-2222-2222-222222222222").unwrap();
        recovery.set_active_session(session2);
        recovery
            .save_crash_dump(Some("panic B".to_string()), None)
            .unwrap();

        let recent = recovery.list_recent_crashes(10);
        assert_eq!(recent.len(), 2);
    }

    #[test]
    fn test_delete_crash_dump() {
        let tmp = tempdir().unwrap();
        let recovery = create_test_recovery(&tmp);

        let mut session = create_test_session();
        session.id = uuid::Uuid::parse_str("dddddddd-dddd-dddd-dddd-dddddddddd01").unwrap();
        recovery.set_active_session(session);
        let path = recovery
            .save_crash_dump(Some("panic".to_string()), None)
            .unwrap();

        assert!(path.exists());
        recovery.delete_crash_dump(&path).unwrap();
        assert!(!path.exists());
    }

    #[test]
    fn test_cleanup_session_crashes() {
        let tmp = tempdir().unwrap();
        let recovery = create_test_recovery(&tmp);

        let mut session1 = create_test_session();
        session1.id = uuid::Uuid::parse_str("ccccccc1-cccc-cccc-cccc-cccccccccccc").unwrap();
        recovery.set_active_session(session1);
        recovery
            .save_crash_dump(Some("panic 1".to_string()), None)
            .unwrap();

        let mut session2 = create_test_session();
        session2.id = uuid::Uuid::parse_str("ccccccc2-cccc-cccc-cccc-cccccccccccc").unwrap();
        recovery.set_active_session(session2);
        recovery
            .save_crash_dump(Some("panic 2".to_string()), None)
            .unwrap();

        let count = recovery
            .cleanup_session_crashes("ccccccc1-cccc-cccc-cccc-cccccccccccc")
            .unwrap();
        assert_eq!(count, 1);
        assert!(recovery
            .find_crash_dumps("ccccccc1-cccc-cccc-cccc-cccccccccccc")
            .is_empty());
        assert_eq!(
            recovery
                .find_crash_dumps("ccccccc2-cccc-cccc-cccc-cccccccccccc")
                .len(),
            1
        );
    }

    #[test]
    fn test_has_recoverable_crash() {
        let tmp = tempdir().unwrap();
        let recovery = create_test_recovery(&tmp);

        assert!(!recovery.has_recoverable_crash("session-none"));

        let mut session = create_test_session();
        session.id = uuid::Uuid::parse_str("dddddddd-dddd-dddd-dddd-ddddddddaaaa").unwrap();
        recovery.set_active_session(session);
        recovery
            .save_crash_dump(Some("panic".to_string()), None)
            .unwrap();

        assert!(recovery.has_recoverable_crash("dddddddd-dddd-dddd-dddd-ddddddddaaaa"));
    }

    #[test]
    fn test_recover_session() {
        let tmp = tempdir().unwrap();
        let recovery = create_test_recovery(&tmp);

        let mut session = create_test_session();
        session.add_message(Message::user("Recover test"));
        session.id = uuid::Uuid::parse_str("bbbbbbbb-bbbb-bbbb-bbbb-bbbbbbbbbbbb").unwrap();
        recovery.set_active_session(session);

        let path = recovery
            .save_crash_dump(Some("panic".to_string()), None)
            .unwrap();

        let recovered = recovery.recover_session(&path).unwrap();
        assert_eq!(recovered.messages.len(), 3);
        assert!(!recovered.tool_invocations.is_empty());
    }

    #[test]
    fn test_recover_session_latest() {
        let tmp = tempdir().unwrap();
        let recovery = create_test_recovery(&tmp);

        let mut session1 = create_test_session();
        session1.id = uuid::Uuid::parse_str("dddddddd-dddd-dddd-dddd-dddddddd0001").unwrap();
        recovery.set_active_session(session1);
        recovery
            .save_crash_dump(Some("panic 1".to_string()), None)
            .unwrap();

        let mut session2 = create_test_session();
        session2.id = uuid::Uuid::parse_str("dddddddd-dddd-dddd-dddd-dddddddd0002").unwrap();
        recovery.set_active_session(session2);
        std::thread::sleep(std::time::Duration::from_millis(10));
        recovery
            .save_crash_dump(Some("panic 2".to_string()), None)
            .unwrap();

        let recovered = recovery
            .recover_session_latest("dddddddd-dddd-dddd-dddd-dddddddd0002")
            .unwrap();
        assert!(recovered.is_some());
        assert_eq!(recovered.unwrap().messages.len(), 2);
    }

    #[test]
    fn test_get_active_session() {
        let tmp = tempdir().unwrap();
        let recovery = create_test_recovery(&tmp);

        assert!(recovery.get_active_session().is_none());

        let session = create_test_session();
        recovery.set_active_session(session.clone());

        let active = recovery.get_active_session().unwrap();
        assert_eq!(active.messages.len(), 2);
    }

    #[test]
    fn test_panic_handler_start_stop() {
        let tmp = tempdir().unwrap();
        let mut handler = PanicHandler::with_crash_recovery(
            CrashRecovery::new().with_dump_dir(tmp.path().join("crashes")),
        );

        handler.start();
        let _ = take_hook();

        handler.stop();
    }
}
