use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::panic;
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
    session_id: String,
    messages_count: usize,
    tool_invocations_count: usize,
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

    pub fn set_active_session(
        &self,
        session_id: String,
        messages_count: usize,
        tool_invocations_count: usize,
    ) {
        let mut guard = self.active_session.lock().unwrap();
        *guard = Some(ActiveSession {
            session_id,
            messages_count,
            tool_invocations_count,
        });
    }

    pub fn clear_active_session(&self) {
        let mut guard = self.active_session.lock().unwrap();
        *guard = None;
    }

    pub fn save_crash_dump(
        &self,
        panic_message: Option<String>,
        stack_trace: Option<String>,
    ) -> Result<PathBuf, CrashRecoveryError> {
        let guard = self.active_session.lock().unwrap();
        let session = guard.as_ref().ok_or(CrashRecoveryError::NoActiveSession)?;

        let crash_dump = CrashDump {
            version: env!("CARGO_PKG_VERSION").to_string(),
            crashed_at: Utc::now(),
            session_id: session.session_id.clone(),
            messages_summary: Vec::new(),
            tool_invocations_summary: Vec::new(),
            state: "crashed".to_string(),
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

        dumps.sort_by_key(|p| p.file_name().unwrap().to_str().unwrap().to_string());
        dumps.reverse();
        dumps
    }

    pub fn load_crash_dump(&self, path: &PathBuf) -> Result<CrashDump, CrashRecoveryError> {
        let content = fs::read_to_string(path)?;
        serde_json::from_str(&content)
            .map_err(|e| CrashRecoveryError::DeserializationError(e.to_string()))
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

pub struct PanicHandler {
    crash_recovery: CrashRecovery,
    previous_hook: Option<Box<dyn Fn(&panic::PanicInfo<'_>) + Send + Sync>>,
}

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
        let previous = panic::take_hook();
        self.previous_hook = Some(Box::new(previous));
        panic::set_hook(Box::new(move |panic_info| {
            let panic_message = panic_info
                .payload()
                .downcast_ref::<&str>()
                .map(|s| s.to_string())
                .or_else(|| panic_info.payload().downcast_ref::<String>().cloned());

            let stack_trace = panic_info
                .location()
                .map(|loc| format!("{}:{}:{}", loc.file(), loc.line(), loc.column()));

            let _ = crash_recovery.save_crash_dump(panic_message, stack_trace);

            if let Some(ref hook) = self.previous_hook {
                hook(panic_info);
            }
        }));
    }

    pub fn stop(&mut self) {
        if let Some(hook) = self.previous_hook.take() {
            panic::set_hook(hook);
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

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    fn create_test_recovery(tmp: &tempfile::TempDir) -> CrashRecovery {
        CrashRecovery::new().with_dump_dir(tmp.path().join("crashes"))
    }

    #[test]
    fn test_crash_recovery_set_and_clear_session() {
        let tmp = tempdir().unwrap();
        let recovery = create_test_recovery(&tmp);

        assert!(recovery.active_session.lock().unwrap().is_none());

        recovery.set_active_session("session-1".to_string(), 10, 5);
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

        recovery.set_active_session("session-crash-test".to_string(), 15, 8);

        let path = recovery
            .save_crash_dump(
                Some("test panic".to_string()),
                Some("main.rs:100".to_string()),
            )
            .unwrap();

        assert!(path.exists());

        let loaded = recovery.load_crash_dump(&path).unwrap();
        assert_eq!(loaded.session_id, "session-crash-test");
        assert!(loaded.panic_message.is_some());
        assert!(loaded.stack_trace.is_some());
    }

    #[test]
    fn test_find_crash_dumps() {
        let tmp = tempdir().unwrap();
        let recovery = create_test_recovery(&tmp);

        recovery.set_active_session("session-find".to_string(), 5, 2);
        recovery
            .save_crash_dump(Some("panic 1".to_string()), None)
            .unwrap();
        recovery
            .save_crash_dump(Some("panic 2".to_string()), None)
            .unwrap();

        let dumps = recovery.find_crash_dumps("session-find");
        assert_eq!(dumps.len(), 2);
    }

    #[test]
    fn test_list_recent_crashes() {
        let tmp = tempdir().unwrap();
        let recovery = create_test_recovery(&tmp);

        recovery.set_active_session("session-a".to_string(), 1, 0);
        recovery
            .save_crash_dump(Some("panic A".to_string()), None)
            .unwrap();

        recovery.set_active_session("session-b".to_string(), 1, 0);
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

        recovery.set_active_session("session-del".to_string(), 3, 1);
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

        recovery.set_active_session("session-cleanup".to_string(), 5, 2);
        recovery
            .save_crash_dump(Some("panic 1".to_string()), None)
            .unwrap();
        recovery
            .save_crash_dump(Some("panic 2".to_string()), None)
            .unwrap();

        let count = recovery.cleanup_session_crashes("session-cleanup").unwrap();
        assert_eq!(count, 2);
        assert!(recovery.find_crash_dumps("session-cleanup").is_empty());
    }

    #[test]
    fn test_has_recoverable_crash() {
        let tmp = tempdir().unwrap();
        let recovery = create_test_recovery(&tmp);

        assert!(!recovery.has_recoverable_crash("session-none"));

        recovery.set_active_session("session-yes".to_string(), 5, 2);
        recovery
            .save_crash_dump(Some("panic".to_string()), None)
            .unwrap();

        assert!(recovery.has_recoverable_crash("session-yes"));
    }

    #[test]
    fn test_panic_handler_start_stop() {
        let tmp = tempdir().unwrap();
        let mut handler = PanicHandler::with_crash_recovery(
            CrashRecovery::new().with_dump_dir(tmp.path().join("crashes")),
        );

        handler.start();
        assert!(panic::take_hook().is_some());

        handler.stop();
    }
}
