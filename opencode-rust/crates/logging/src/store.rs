//! Session log buffer (ring buffer) and persistent log store.

use std::collections::VecDeque;

use chrono::{DateTime, Utc};
use rusqlite::{params, Connection};

use crate::error::LogError;
use crate::event::{
    ErrorContext, LogEvent, LogFields, LogLevel, ReasoningLog, SanitizedValue, ToolConsideration,
    ToolExecutionLog, ToolResult,
};
use crate::query::LogQuery;

pub struct SessionLogBuffer {
    events: VecDeque<LogEvent>,
    capacity: usize,
    next_seq: u64,
}

impl SessionLogBuffer {
    pub fn new(capacity: usize) -> Self {
        Self {
            events: VecDeque::with_capacity(capacity),
            capacity,
            next_seq: 1,
        }
    }

    pub fn push(&mut self, mut event: LogEvent) {
        if event.seq == 0 {
            event.seq = self.next_seq;
            self.next_seq += 1;
        }

        if self.events.len() >= self.capacity {
            self.events.pop_front();
        }
        self.events.push_back(event);
    }

    pub fn get_range(&self, from_seq: u64, to_seq: u64) -> Vec<&LogEvent> {
        self.events
            .iter()
            .filter(|e| e.seq >= from_seq && e.seq <= to_seq)
            .collect()
    }

    pub fn get_by_level(&self, level: LogLevel) -> Vec<&LogEvent> {
        self.events.iter().filter(|e| e.level == level).collect()
    }

    pub fn len(&self) -> usize {
        self.events.len()
    }

    pub fn is_empty(&self) -> bool {
        self.events.is_empty()
    }

    pub fn iter(&self) -> impl Iterator<Item = &LogEvent> {
        self.events.iter()
    }

    pub fn clear(&mut self) {
        self.events.clear();
    }
}

pub struct LogStore {
    conn: Connection,
}

impl LogStore {
    pub fn new(path: &std::path::Path) -> Result<Self, LogError> {
        let conn = Connection::open(path)?;
        conn.execute_batch(
            "CREATE TABLE IF NOT EXISTS logs (
                seq INTEGER PRIMARY KEY,
                timestamp TEXT NOT NULL,
                level TEXT NOT NULL,
                target TEXT NOT NULL,
                message TEXT NOT NULL,
                fields TEXT NOT NULL,
                span_id TEXT,
                parent_seq INTEGER,
                session_id TEXT
            );
            CREATE INDEX IF NOT EXISTS idx_logs_session ON logs(session_id, timestamp);
            CREATE INDEX IF NOT EXISTS idx_logs_level ON logs(level);
            CREATE INDEX IF NOT EXISTS idx_logs_timestamp ON logs(timestamp);
            ",
        )?;

        Ok(Self { conn })
    }

    pub fn append(&self, event: &LogEvent) -> Result<(), LogError> {
        let fields_json = serde_json::to_string(&event.fields)
            .map_err(|e| LogError::Serialization(e.to_string()))?;

        self.conn.execute(
            "INSERT INTO logs (seq, timestamp, level, target, message, fields, span_id, parent_seq, session_id)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
            params![
                event.seq,
                event.timestamp.to_rfc3339(),
                serde_json::to_string(&event.level).unwrap_or_default(),
                event.target,
                event.message,
                fields_json,
                event.span_id,
                event.parent_seq,
                event.fields.session_id,
            ],
        )?;

        Ok(())
    }

    pub fn query(&self, criteria: &LogQuery) -> Result<Vec<LogEvent>, LogError> {
        let mut sql = String::from("SELECT seq, timestamp, level, target, message, fields, span_id, parent_seq, session_id FROM logs WHERE 1=1");
        let mut params_vec: Vec<Box<dyn rusqlite::ToSql>> = Vec::new();

        if let Some(ref session_id) = criteria.session_id {
            sql.push_str(" AND session_id = ?");
            params_vec.push(Box::new(session_id.clone()));
        }

        if let Some(ref level) = criteria.level {
            sql.push_str(" AND level = ?");
            params_vec.push(Box::new(serde_json::to_string(level).unwrap_or_default()));
        }

        if let Some(ref target) = criteria.target {
            sql.push_str(" AND (target = ? OR target LIKE ?)");
            params_vec.push(Box::new(target.clone()));
            params_vec.push(Box::new(target.replace('*', "%")));
        }

        if let Some(ref since) = criteria.since {
            sql.push_str(" AND timestamp >= ?");
            params_vec.push(Box::new(since.to_rfc3339()));
        }

        if let Some(ref until) = criteria.until {
            sql.push_str(" AND timestamp <= ?");
            params_vec.push(Box::new(until.to_rfc3339()));
        }

        sql.push_str(" ORDER BY seq DESC");

        if let Some(limit) = criteria.limit {
            sql.push_str(&format!(" LIMIT {}", limit));
        }

        let params_refs: Vec<&dyn rusqlite::ToSql> =
            params_vec.iter().map(|p| p.as_ref()).collect();

        let mut stmt = self.conn.prepare(&sql)?;
        let rows = stmt.query_map(params_refs.as_slice(), |row| {
            let timestamp_str: String = row.get(1)?;
            let level_str: String = row.get(2)?;
            let fields_str: String = row.get(5)?;
            let session_id_from_db: Option<String> = row.get(8)?;

            let timestamp = DateTime::parse_from_rfc3339(&timestamp_str)
                .map(|dt| dt.with_timezone(&Utc))
                .unwrap_or_else(|_| Utc::now());
            let level: LogLevel = serde_json::from_str(&level_str).unwrap_or(LogLevel::Info);
            let mut fields: LogFields = serde_json::from_str(&fields_str).unwrap_or_default();
            if fields.session_id.is_none() {
                fields.session_id = session_id_from_db;
            }

            Ok(LogEvent {
                seq: row.get(0)?,
                timestamp,
                level,
                target: row.get(3)?,
                message: row.get(4)?,
                fields,
                span_id: row.get(6)?,
                parent_seq: row.get(7)?,
            })
        })?;

        let mut events = Vec::new();
        for event in rows.flatten() {
            events.push(event);
        }

        Ok(events)
    }

    pub fn recent(&self, session_id: &str, limit: usize) -> Result<Vec<LogEvent>, LogError> {
        let query = LogQuery::new()
            .with_session_id(session_id)
            .with_limit(limit);
        self.query(&query)
    }

    pub fn prune(&self, older_than: DateTime<Utc>) -> Result<u64, LogError> {
        let deleted = self.conn.execute(
            "DELETE FROM logs WHERE timestamp < ?",
            params![older_than.to_rfc3339()],
        )?;
        Ok(deleted as u64)
    }
}

pub struct ReasoningLogStore {
    conn: Connection,
}

impl ReasoningLogStore {
    pub fn new(path: &std::path::Path) -> Result<Self, LogError> {
        let conn = Connection::open(path)?;
        conn.execute_batch(
            "CREATE TABLE IF NOT EXISTS reasoning_logs (
                step_id TEXT PRIMARY KEY,
                session_id TEXT NOT NULL,
                timestamp TEXT NOT NULL,
                prompt TEXT NOT NULL,
                response TEXT NOT NULL,
                tools_considered TEXT NOT NULL,
                decision TEXT NOT NULL,
                prompt_tokens INTEGER NOT NULL,
                completion_tokens INTEGER NOT NULL,
                latency_ms INTEGER NOT NULL
            );
            CREATE INDEX IF NOT EXISTS idx_reasoning_session ON reasoning_logs(session_id, timestamp);
            ",
        )?;

        Ok(Self { conn })
    }

    pub fn append(&self, reasoning: &ReasoningLog) -> Result<(), LogError> {
        let tools_json = serde_json::to_string(&reasoning.tools_considered)
            .map_err(|e| LogError::Serialization(e.to_string()))?;

        self.conn.execute(
            "INSERT INTO reasoning_logs (step_id, session_id, timestamp, prompt, response, tools_considered, decision, prompt_tokens, completion_tokens, latency_ms)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)",
            params![
                reasoning.step_id,
                reasoning.session_id,
                reasoning.timestamp.to_rfc3339(),
                reasoning.prompt,
                reasoning.response,
                tools_json,
                reasoning.decision,
                reasoning.prompt_tokens as i64,
                reasoning.completion_tokens as i64,
                reasoning.latency_ms as i64,
            ],
        )?;

        Ok(())
    }

    pub fn query(&self, criteria: &ReasoningLogQuery) -> Result<Vec<ReasoningLog>, LogError> {
        let mut sql = String::from("SELECT step_id, session_id, timestamp, prompt, response, tools_considered, decision, prompt_tokens, completion_tokens, latency_ms FROM reasoning_logs WHERE 1=1");
        let mut params_vec: Vec<Box<dyn rusqlite::ToSql>> = Vec::new();

        if let Some(ref session_id) = criteria.session_id {
            sql.push_str(" AND session_id = ?");
            params_vec.push(Box::new(session_id.clone()));
        }

        if let Some(ref since) = criteria.since {
            sql.push_str(" AND timestamp >= ?");
            params_vec.push(Box::new(since.to_rfc3339()));
        }

        if let Some(ref until) = criteria.until {
            sql.push_str(" AND timestamp <= ?");
            params_vec.push(Box::new(until.to_rfc3339()));
        }

        sql.push_str(" ORDER BY timestamp DESC");

        if let Some(limit) = criteria.limit {
            sql.push_str(&format!(" LIMIT {}", limit));
        }

        let params_refs: Vec<&dyn rusqlite::ToSql> =
            params_vec.iter().map(|p| p.as_ref()).collect();

        let mut stmt = self.conn.prepare(&sql)?;
        let rows = stmt.query_map(params_refs.as_slice(), |row| {
            let timestamp_str: String = row.get(2)?;
            let tools_str: String = row.get(5)?;

            let timestamp = DateTime::parse_from_rfc3339(&timestamp_str)
                .map(|dt| dt.with_timezone(&Utc))
                .unwrap_or_else(|_| Utc::now());
            let tools_considered: Vec<ToolConsideration> =
                serde_json::from_str(&tools_str).unwrap_or_default();

            Ok(ReasoningLog {
                step_id: row.get(0)?,
                session_id: row.get(1)?,
                timestamp,
                prompt: row.get(3)?,
                response: row.get(4)?,
                tools_considered,
                decision: row.get(6)?,
                prompt_tokens: row.get::<_, i64>(7)? as u64,
                completion_tokens: row.get::<_, i64>(8)? as u64,
                latency_ms: row.get::<_, i64>(9)? as u64,
            })
        })?;

        let mut logs = Vec::new();
        for log in rows.flatten() {
            logs.push(log);
        }

        Ok(logs)
    }

    pub fn get(&self, step_id: &str) -> Result<Option<ReasoningLog>, LogError> {
        let mut stmt = self.conn.prepare(
            "SELECT step_id, session_id, timestamp, prompt, response, tools_considered, decision, prompt_tokens, completion_tokens, latency_ms FROM reasoning_logs WHERE step_id = ?",
        )?;

        let mut rows = stmt.query(params![step_id])?;
        if let Some(row) = rows.next()? {
            let timestamp_str: String = row.get(2)?;
            let tools_str: String = row.get(5)?;

            let timestamp = DateTime::parse_from_rfc3339(&timestamp_str)
                .map(|dt| dt.with_timezone(&Utc))
                .unwrap_or_else(|_| Utc::now());
            let tools_considered: Vec<ToolConsideration> =
                serde_json::from_str(&tools_str).unwrap_or_default();

            Ok(Some(ReasoningLog {
                step_id: row.get(0)?,
                session_id: row.get(1)?,
                timestamp,
                prompt: row.get(3)?,
                response: row.get(4)?,
                tools_considered,
                decision: row.get(6)?,
                prompt_tokens: row.get::<_, i64>(7)? as u64,
                completion_tokens: row.get::<_, i64>(8)? as u64,
                latency_ms: row.get::<_, i64>(9)? as u64,
            }))
        } else {
            Ok(None)
        }
    }

    pub fn get_by_session(&self, session_id: &str) -> Result<Vec<ReasoningLog>, LogError> {
        self.query(&ReasoningLogQuery::for_session(session_id))
    }
}

#[derive(Debug, Clone, Default)]
pub struct ReasoningLogQuery {
    pub session_id: Option<String>,
    pub since: Option<DateTime<Utc>>,
    pub until: Option<DateTime<Utc>>,
    pub limit: Option<usize>,
}

impl ReasoningLogQuery {
    pub fn for_session(session_id: impl Into<String>) -> Self {
        Self {
            session_id: Some(session_id.into()),
            ..Default::default()
        }
    }

    pub fn with_since(mut self, since: DateTime<Utc>) -> Self {
        self.since = Some(since);
        self
    }

    pub fn with_until(mut self, until: DateTime<Utc>) -> Self {
        self.until = Some(until);
        self
    }

    pub fn with_limit(mut self, limit: usize) -> Self {
        self.limit = Some(limit);
        self
    }
}

pub struct ToolExecutionLogStore {
    conn: Connection,
}

impl ToolExecutionLogStore {
    pub fn new(path: &std::path::Path) -> Result<Self, LogError> {
        let conn = Connection::open(path)?;
        conn.execute_batch(
            "CREATE TABLE IF NOT EXISTS tool_execution_logs (
                execution_id TEXT PRIMARY KEY,
                session_id TEXT NOT NULL,
                tool_name TEXT NOT NULL,
                timestamp TEXT NOT NULL,
                parameters TEXT NOT NULL,
                result TEXT NOT NULL,
                latency_ms INTEGER NOT NULL,
                error TEXT
            );
            CREATE INDEX IF NOT EXISTS idx_tool_exec_session ON tool_execution_logs(session_id, timestamp);
            ",
        )?;

        Ok(Self { conn })
    }

    pub fn append(&self, log: &ToolExecutionLog) -> Result<(), LogError> {
        let parameters_json = serde_json::to_string(&log.parameters)
            .map_err(|e| LogError::Serialization(e.to_string()))?;
        let result_json = serde_json::to_string(&log.result)
            .map_err(|e| LogError::Serialization(e.to_string()))?;
        let error_json = match &log.error {
            Some(e) => {
                serde_json::to_string(e).map_err(|e| LogError::Serialization(e.to_string()))?
            }
            None => serde_json::Value::Null.to_string(),
        };

        self.conn.execute(
            "INSERT INTO tool_execution_logs (execution_id, session_id, tool_name, timestamp, parameters, result, latency_ms, error)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
            params![
                log.execution_id,
                log.session_id,
                log.tool_name,
                log.timestamp.to_rfc3339(),
                parameters_json,
                result_json,
                log.latency_ms as i64,
                error_json,
            ],
        )?;

        Ok(())
    }

    pub fn query(
        &self,
        criteria: &ToolExecutionLogQuery,
    ) -> Result<Vec<ToolExecutionLog>, LogError> {
        let mut sql = String::from("SELECT execution_id, session_id, tool_name, timestamp, parameters, result, latency_ms, error FROM tool_execution_logs WHERE 1=1");
        let mut params_vec: Vec<Box<dyn rusqlite::ToSql>> = Vec::new();

        if let Some(ref session_id) = criteria.session_id {
            sql.push_str(" AND session_id = ?");
            params_vec.push(Box::new(session_id.clone()));
        }

        if let Some(ref tool_name) = criteria.tool_name {
            sql.push_str(" AND tool_name = ?");
            params_vec.push(Box::new(tool_name.clone()));
        }

        if let Some(ref since) = criteria.since {
            sql.push_str(" AND timestamp >= ?");
            params_vec.push(Box::new(since.to_rfc3339()));
        }

        if let Some(ref until) = criteria.until {
            sql.push_str(" AND timestamp <= ?");
            params_vec.push(Box::new(until.to_rfc3339()));
        }

        sql.push_str(" ORDER BY timestamp DESC");

        if let Some(limit) = criteria.limit {
            sql.push_str(&format!(" LIMIT {}", limit));
        }

        let params_refs: Vec<&dyn rusqlite::ToSql> =
            params_vec.iter().map(|p| p.as_ref()).collect();

        let mut stmt = self.conn.prepare(&sql)?;
        let rows = stmt.query_map(params_refs.as_slice(), |row| {
            let execution_id: String = row.get(0)?;
            let session_id: String = row.get(1)?;
            let tool_name: String = row.get(2)?;
            let timestamp_str: String = row.get(3)?;
            let parameters_str: String = row.get(4)?;
            let result_str: String = row.get(5)?;
            let latency_ms: i64 = row.get(6)?;
            let error_str: Option<String> = row.get(7)?;

            let timestamp = DateTime::parse_from_rfc3339(&timestamp_str)
                .map(|dt| dt.with_timezone(&Utc))
                .unwrap_or_else(|_| Utc::now());
            let parameters: SanitizedValue = serde_json::from_str(&parameters_str)
                .unwrap_or(SanitizedValue::Nested(Default::default()));
            let result: ToolResult = serde_json::from_str(&result_str).unwrap_or(ToolResult {
                success: false,
                message: "Deserialization failed".to_string(),
                output: None,
            });
            let error: Option<ErrorContext> = error_str.and_then(|s| {
                if s == "null" || s.is_empty() {
                    None
                } else {
                    serde_json::from_str(&s).ok()
                }
            });

            Ok(ToolExecutionLog {
                execution_id,
                session_id,
                tool_name,
                timestamp,
                parameters,
                result,
                latency_ms: latency_ms as u64,
                error,
            })
        })?;

        let mut logs = Vec::new();
        for log in rows.flatten() {
            logs.push(log);
        }

        Ok(logs)
    }

    pub fn get(&self, execution_id: &str) -> Result<Option<ToolExecutionLog>, LogError> {
        let mut stmt = self.conn.prepare(
            "SELECT execution_id, session_id, tool_name, timestamp, parameters, result, latency_ms, error FROM tool_execution_logs WHERE execution_id = ?",
        )?;

        let mut rows = stmt.query(params![execution_id])?;
        if let Some(row) = rows.next()? {
            let timestamp_str: String = row.get(3)?;
            let parameters_str: String = row.get(4)?;
            let result_str: String = row.get(5)?;
            let error_str: Option<String> = row.get(7)?;

            let timestamp = DateTime::parse_from_rfc3339(&timestamp_str)
                .map(|dt| dt.with_timezone(&Utc))
                .unwrap_or_else(|_| Utc::now());
            let parameters: SanitizedValue = serde_json::from_str(&parameters_str)
                .unwrap_or(SanitizedValue::Nested(Default::default()));
            let result: ToolResult = serde_json::from_str(&result_str).unwrap_or(ToolResult {
                success: false,
                message: "Deserialization failed".to_string(),
                output: None,
            });
            let error: Option<ErrorContext> = error_str.and_then(|s| {
                if s == "null" || s.is_empty() {
                    None
                } else {
                    serde_json::from_str(&s).ok()
                }
            });

            Ok(Some(ToolExecutionLog {
                execution_id: row.get(0)?,
                session_id: row.get(1)?,
                tool_name: row.get(2)?,
                timestamp,
                parameters,
                result,
                latency_ms: row.get::<_, i64>(6)? as u64,
                error,
            }))
        } else {
            Ok(None)
        }
    }

    pub fn get_by_session(&self, session_id: &str) -> Result<Vec<ToolExecutionLog>, LogError> {
        self.query(&ToolExecutionLogQuery::for_session(session_id))
    }
}

#[derive(Debug, Clone, Default)]
pub struct ToolExecutionLogQuery {
    pub session_id: Option<String>,
    pub tool_name: Option<String>,
    pub since: Option<DateTime<Utc>>,
    pub until: Option<DateTime<Utc>>,
    pub limit: Option<usize>,
}

impl ToolExecutionLogQuery {
    pub fn for_session(session_id: impl Into<String>) -> Self {
        Self {
            session_id: Some(session_id.into()),
            ..Default::default()
        }
    }

    pub fn with_tool_name(mut self, tool_name: impl Into<String>) -> Self {
        self.tool_name = Some(tool_name.into());
        self
    }

    pub fn with_since(mut self, since: DateTime<Utc>) -> Self {
        self.since = Some(since);
        self
    }

    pub fn with_until(mut self, until: DateTime<Utc>) -> Self {
        self.until = Some(until);
        self
    }

    pub fn with_limit(mut self, limit: usize) -> Self {
        self.limit = Some(limit);
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_db() -> (tempfile::TempDir, LogStore) {
        let temp_dir = tempfile::tempdir().unwrap();
        let db_path = temp_dir.path().join("test.db");
        let store = LogStore::new(&db_path).unwrap();
        (temp_dir, store)
    }

    #[test]
    fn test_log_store_new_creates_database_and_schema() {
        let temp_dir = tempfile::tempdir().unwrap();
        let db_path = temp_dir.path().join("test.db");

        let _store = LogStore::new(&db_path).unwrap();

        assert!(db_path.exists());

        let conn = Connection::open(&db_path).unwrap();
        let table_exists: bool = conn
            .query_row(
                "SELECT COUNT(*) > 0 FROM sqlite_master WHERE type='table' AND name='logs'",
                [],
                |row| row.get(0),
            )
            .unwrap();
        assert!(table_exists, "logs table should exist");

        let session_idx_exists: bool = conn
            .query_row(
                "SELECT COUNT(*) > 0 FROM sqlite_master WHERE type='index' AND name LIKE '%session%'",
                [],
                |row| row.get(0),
            )
            .unwrap();
        assert!(session_idx_exists, "session index should exist");

        let level_idx_exists: bool = conn
            .query_row(
                "SELECT COUNT(*) > 0 FROM sqlite_master WHERE type='index' AND name LIKE '%level%'",
                [],
                |row| row.get(0),
            )
            .unwrap();
        assert!(level_idx_exists, "level index should exist");

        let timestamp_idx_exists: bool = conn
            .query_row(
                "SELECT COUNT(*) > 0 FROM sqlite_master WHERE type='index' AND name LIKE '%timestamp%'",
                [],
                |row| row.get(0),
            )
            .unwrap();
        assert!(timestamp_idx_exists, "timestamp index should exist");
    }

    #[test]
    fn test_log_store_new_creates_schema_with_correct_columns() {
        let temp_dir = tempfile::tempdir().unwrap();
        let db_path = temp_dir.path().join("test.db");

        LogStore::new(&db_path).unwrap();

        let conn = Connection::open(&db_path).unwrap();
        let schema_sql: String = conn
            .query_row(
                "SELECT sql FROM sqlite_master WHERE type='table' AND name='logs'",
                [],
                |row| row.get(0),
            )
            .unwrap();

        assert!(
            schema_sql.contains("seq INTEGER PRIMARY KEY"),
            "seq column should be primary key"
        );
        assert!(
            schema_sql.contains("timestamp TEXT NOT NULL"),
            "timestamp column should exist"
        );
        assert!(
            schema_sql.contains("level TEXT NOT NULL"),
            "level column should exist"
        );
        assert!(
            schema_sql.contains("target TEXT NOT NULL"),
            "target column should exist"
        );
        assert!(
            schema_sql.contains("message TEXT NOT NULL"),
            "message column should exist"
        );
        assert!(
            schema_sql.contains("fields TEXT NOT NULL"),
            "fields column should exist"
        );
        assert!(
            schema_sql.contains("span_id TEXT"),
            "span_id column should exist"
        );
        assert!(
            schema_sql.contains("parent_seq INTEGER"),
            "parent_seq column should exist"
        );
    }

    #[test]
    fn test_log_store_append_inserts_log_event_correctly() {
        let (_temp_dir, store) = create_test_db();

        let event = LogEvent::new(1, LogLevel::Info, "test.target", "Test message")
            .with_session_id("sess_123")
            .with_tool_name("test_tool")
            .with_latency_ms(42);

        store.append(&event).unwrap();

        // Query the event back
        let results = store
            .query(&LogQuery::new().with_session_id("sess_123"))
            .unwrap();

        assert_eq!(results.len(), 1);
        let retrieved = &results[0];
        assert_eq!(retrieved.seq, 1);
        assert_eq!(retrieved.level, LogLevel::Info);
        assert_eq!(retrieved.target, "test.target");
        assert_eq!(retrieved.message, "Test message");
        assert_eq!(retrieved.fields.session_id, Some("sess_123".to_string()));
        assert_eq!(retrieved.fields.tool_name, Some("test_tool".to_string()));
        assert_eq!(retrieved.fields.latency_ms, Some(42));
    }

    #[test]
    fn test_log_store_append_with_span_and_parent() {
        let (_temp_dir, store) = create_test_db();

        let event = LogEvent::new(1, LogLevel::Debug, "parent.target", "Parent message")
            .with_span_id("trace_abc:span_123")
            .with_parent_seq(0);

        store.append(&event).unwrap();

        let results = store.query(&LogQuery::new()).unwrap();
        assert_eq!(results.len(), 1);

        let retrieved = &results[0];
        assert_eq!(retrieved.span_id, Some("trace_abc:span_123".to_string()));
        assert_eq!(retrieved.parent_seq, Some(0));
    }

    #[test]
    fn test_log_store_query_returns_filtered_results_by_session() {
        let (_temp_dir, store) = create_test_db();

        // Insert events for different sessions
        store
            .append(&LogEvent::new(1, LogLevel::Info, "test", "msg1").with_session_id("sess_a"))
            .unwrap();
        store
            .append(&LogEvent::new(2, LogLevel::Info, "test", "msg2").with_session_id("sess_b"))
            .unwrap();
        store
            .append(&LogEvent::new(3, LogLevel::Info, "test", "msg3").with_session_id("sess_a"))
            .unwrap();
        store
            .append(&LogEvent::new(4, LogLevel::Info, "test", "msg4").with_session_id("sess_c"))
            .unwrap();

        let results = store
            .query(&LogQuery::new().with_session_id("sess_a"))
            .unwrap();

        assert_eq!(results.len(), 2);
        assert!(results
            .iter()
            .all(|e| e.fields.session_id.as_ref().unwrap() == "sess_a"));
    }

    #[test]
    fn test_log_store_query_returns_filtered_results_by_level() {
        let (_temp_dir, store) = create_test_db();

        store
            .append(&LogEvent::new(1, LogLevel::Info, "test", "info"))
            .unwrap();
        store
            .append(&LogEvent::new(2, LogLevel::Error, "test", "error"))
            .unwrap();
        store
            .append(&LogEvent::new(3, LogLevel::Debug, "test", "debug"))
            .unwrap();
        store
            .append(&LogEvent::new(4, LogLevel::Warn, "test", "warn"))
            .unwrap();

        let results = store
            .query(&LogQuery::new().with_level(LogLevel::Error))
            .unwrap();

        assert_eq!(results.len(), 1);
        assert_eq!(results[0].message, "error");
    }

    #[test]
    fn test_log_store_query_returns_filtered_results_by_target() {
        let (_temp_dir, store) = create_test_db();

        store
            .append(&LogEvent::new(1, LogLevel::Info, "llm.openai", "response"))
            .unwrap();
        store
            .append(&LogEvent::new(
                2,
                LogLevel::Info,
                "llm.anthropic",
                "response",
            ))
            .unwrap();
        store
            .append(&LogEvent::new(3, LogLevel::Info, "tool.read", "read"))
            .unwrap();

        let results = store.query(&LogQuery::new().with_target("llm.*")).unwrap();

        assert_eq!(results.len(), 2);
        assert!(results.iter().all(|e| e.target.starts_with("llm.")));
    }

    #[test]
    fn test_log_store_query_returns_filtered_results_by_time_range() {
        let (_temp_dir, store) = create_test_db();
        let now = chrono::Utc::now();

        store
            .append(&LogEvent::new(1, LogLevel::Info, "test", "old"))
            .unwrap();
        store
            .append(&LogEvent::new(2, LogLevel::Info, "test", "recent"))
            .unwrap();

        let results = store
            .query(
                &LogQuery::new()
                    .with_since(now - chrono::Duration::minutes(1))
                    .with_until(now + chrono::Duration::minutes(1)),
            )
            .unwrap();

        assert_eq!(results.len(), 2);
    }

    #[test]
    fn test_log_store_query_respects_limit() {
        let (_temp_dir, store) = create_test_db();

        for i in 1..=10 {
            store
                .append(&LogEvent::new(
                    i,
                    LogLevel::Info,
                    "test",
                    format!("msg{}", i),
                ))
                .unwrap();
        }

        let results = store.query(&LogQuery::new().with_limit(3)).unwrap();

        assert_eq!(results.len(), 3);
    }

    #[test]
    fn test_log_store_query_with_multiple_filters() {
        let (_temp_dir, store) = create_test_db();

        store
            .append(
                &LogEvent::new(1, LogLevel::Error, "tool.read", "err").with_session_id("sess_x"),
            )
            .unwrap();
        store
            .append(&LogEvent::new(2, LogLevel::Info, "tool.read", "ok").with_session_id("sess_x"))
            .unwrap();
        store
            .append(
                &LogEvent::new(3, LogLevel::Error, "tool.write", "err").with_session_id("sess_y"),
            )
            .unwrap();

        let results = store
            .query(
                &LogQuery::new()
                    .with_session_id("sess_x")
                    .with_level(LogLevel::Error),
            )
            .unwrap();

        assert_eq!(results.len(), 1);
        assert_eq!(results[0].seq, 1);
    }

    #[test]
    fn test_log_store_recent_returns_session_latest_logs() {
        let (_temp_dir, store) = create_test_db();

        // Insert events for different sessions
        for i in 1..=5 {
            store
                .append(
                    &LogEvent::new(i, LogLevel::Info, "test", format!("sess_a_msg{}", i))
                        .with_session_id("sess_a"),
                )
                .unwrap();
        }
        for i in 6..=8 {
            store
                .append(
                    &LogEvent::new(i, LogLevel::Info, "test", format!("sess_b_msg{}", i))
                        .with_session_id("sess_b"),
                )
                .unwrap();
        }

        let results = store.recent("sess_a", 3).unwrap();

        assert_eq!(results.len(), 3);
        // Should be the 3 most recent (highest seq numbers first)
        assert_eq!(results[0].seq, 5);
        assert_eq!(results[1].seq, 4);
        assert_eq!(results[2].seq, 3);
    }

    #[test]
    fn test_log_store_recent_returns_empty_for_nonexistent_session() {
        let (_temp_dir, store) = create_test_db();

        store
            .append(&LogEvent::new(1, LogLevel::Info, "test", "msg").with_session_id("sess_a"))
            .unwrap();

        let results = store.recent("nonexistent", 10).unwrap();

        assert!(results.is_empty());
    }

    #[test]
    fn test_log_store_recent_respects_limit() {
        let (_temp_dir, store) = create_test_db();

        for i in 1..=10 {
            store
                .append(
                    &LogEvent::new(i, LogLevel::Info, "test", format!("msg{}", i))
                        .with_session_id("sess_a"),
                )
                .unwrap();
        }

        let results = store.recent("sess_a", 3).unwrap();

        assert_eq!(results.len(), 3);
    }

    #[test]
    fn test_log_store_prune_removes_old_logs() {
        let temp_dir = tempfile::tempdir().unwrap();
        let db_path = temp_dir.path().join("test.db");
        let store = LogStore::new(&db_path).unwrap();

        store
            .append(&LogEvent::new(1, LogLevel::Info, "test", "old"))
            .unwrap();
        store
            .append(&LogEvent::new(2, LogLevel::Info, "test", "recent"))
            .unwrap();

        let deleted = store
            .prune(chrono::Utc::now() - chrono::Duration::seconds(1))
            .unwrap();
        assert_eq!(deleted, 0);

        let deleted = store
            .prune(chrono::Utc::now() + chrono::Duration::days(1))
            .unwrap();
        assert_eq!(deleted, 2);

        let remaining = store.query(&LogQuery::new()).unwrap();
        assert_eq!(remaining.len(), 0);
    }

    #[test]
    fn test_log_store_prune_returns_correct_count() {
        let (_temp_dir, store) = create_test_db();

        for i in 1..=5 {
            store
                .append(&LogEvent::new(
                    i,
                    LogLevel::Info,
                    "test",
                    format!("msg{}", i),
                ))
                .unwrap();
        }

        let deleted = store
            .prune(chrono::Utc::now() + chrono::Duration::seconds(1))
            .unwrap();
        assert_eq!(deleted, 5);
    }

    #[test]
    fn test_log_store_prune_with_mixed_ages() {
        let temp_dir = tempfile::tempdir().unwrap();
        let db_path = temp_dir.path().join("test.db");
        let store = LogStore::new(&db_path).unwrap();

        store
            .append(&LogEvent::new(1, LogLevel::Info, "test", "keep"))
            .unwrap();
        store
            .append(&LogEvent::new(2, LogLevel::Info, "test", "keep2"))
            .unwrap();

        let deleted = store
            .prune(chrono::Utc::now() - chrono::Duration::seconds(1))
            .unwrap();
        assert_eq!(deleted, 0);

        let remaining = store.query(&LogQuery::new()).unwrap();
        assert_eq!(remaining.len(), 2);
    }

    #[test]
    fn test_log_store_query_empty_when_no_matches() {
        let (_temp_dir, store) = create_test_db();

        store
            .append(&LogEvent::new(1, LogLevel::Info, "test", "msg").with_session_id("sess_a"))
            .unwrap();

        let results = store
            .query(&LogQuery::new().with_session_id("nonexistent"))
            .unwrap();

        assert!(results.is_empty());
    }

    #[test]
    fn test_log_store_query_returns_ordered_by_seq_desc() {
        let (_temp_dir, store) = create_test_db();

        for i in 1..=5 {
            store
                .append(&LogEvent::new(
                    i,
                    LogLevel::Info,
                    "test",
                    format!("msg{}", i),
                ))
                .unwrap();
        }

        let results = store.query(&LogQuery::new()).unwrap();

        assert_eq!(results.len(), 5);
        assert_eq!(results[0].seq, 5);
        assert_eq!(results[4].seq, 1);
    }

    #[test]
    fn test_session_log_buffer_push() {
        let mut buffer = SessionLogBuffer::new(3);

        buffer.push(LogEvent::new(0, LogLevel::Info, "test", "first"));
        buffer.push(LogEvent::new(0, LogLevel::Info, "test", "second"));
        buffer.push(LogEvent::new(0, LogLevel::Info, "test", "third"));

        assert_eq!(buffer.len(), 3);
        assert_eq!(buffer.iter().next().unwrap().message, "first");
    }

    #[test]
    fn test_session_log_buffer_eviction() {
        let mut buffer = SessionLogBuffer::new(2);

        buffer.push(LogEvent::new(0, LogLevel::Info, "test", "first"));
        buffer.push(LogEvent::new(0, LogLevel::Info, "test", "second"));
        buffer.push(LogEvent::new(0, LogLevel::Info, "test", "third"));

        assert_eq!(buffer.len(), 2);
        assert!(buffer.iter().all(|e| e.message != "first"));
    }

    #[test]
    fn test_session_log_buffer_get_range() {
        let mut buffer = SessionLogBuffer::new(10);

        for i in 1..=5 {
            buffer.push(LogEvent::new(
                0,
                LogLevel::Info,
                "test",
                format!("event_{}", i),
            ));
        }

        let range = buffer.get_range(2, 4);
        assert_eq!(range.len(), 3);
    }

    #[test]
    fn test_session_log_buffer_get_by_level() {
        let mut buffer = SessionLogBuffer::new(10);

        buffer.push(LogEvent::new(0, LogLevel::Info, "test", "info1"));
        buffer.push(LogEvent::new(0, LogLevel::Error, "test", "error1"));
        buffer.push(LogEvent::new(0, LogLevel::Info, "test", "info2"));
        buffer.push(LogEvent::new(0, LogLevel::Debug, "test", "debug1"));

        let errors = buffer.get_by_level(LogLevel::Error);
        assert_eq!(errors.len(), 1);
        assert_eq!(errors[0].message, "error1");
    }

    #[test]
    fn test_session_log_buffer_len_and_is_empty() {
        let mut buffer = SessionLogBuffer::new(5);

        assert_eq!(buffer.len(), 0);
        assert!(buffer.is_empty());

        buffer.push(LogEvent::new(0, LogLevel::Info, "test", "first"));
        assert_eq!(buffer.len(), 1);
        assert!(!buffer.is_empty());

        buffer.push(LogEvent::new(0, LogLevel::Info, "test", "second"));
        buffer.push(LogEvent::new(0, LogLevel::Info, "test", "third"));
        assert_eq!(buffer.len(), 3);

        buffer.push(LogEvent::new(0, LogLevel::Info, "test", "fourth"));
        buffer.push(LogEvent::new(0, LogLevel::Info, "test", "fifth"));
        buffer.push(LogEvent::new(0, LogLevel::Info, "test", "sixth"));
        assert_eq!(buffer.len(), 5);
        assert!(!buffer.is_empty());
    }

    #[test]
    fn test_session_log_buffer_wraparound_at_capacity_boundary() {
        let capacity = 3;
        let mut buffer = SessionLogBuffer::new(capacity);

        buffer.push(LogEvent::new(0, LogLevel::Info, "test", "event_1"));
        buffer.push(LogEvent::new(0, LogLevel::Info, "test", "event_2"));
        buffer.push(LogEvent::new(0, LogLevel::Info, "test", "event_3"));

        assert_eq!(buffer.len(), 3);

        buffer.push(LogEvent::new(0, LogLevel::Info, "test", "event_4"));

        assert_eq!(buffer.len(), 3);
        let messages: Vec<_> = buffer.iter().map(|e| e.message.clone()).collect();
        assert!(!messages.contains(&"event_1".to_string()));
        assert!(messages.contains(&"event_2".to_string()));
        assert!(messages.contains(&"event_3".to_string()));
        assert!(messages.contains(&"event_4".to_string()));

        buffer.push(LogEvent::new(0, LogLevel::Info, "test", "event_5"));
        buffer.push(LogEvent::new(0, LogLevel::Info, "test", "event_6"));

        let messages: Vec<_> = buffer.iter().map(|e| e.message.clone()).collect();
        assert!(!messages.contains(&"event_1".to_string()));
        assert!(!messages.contains(&"event_2".to_string()));
        assert!(!messages.contains(&"event_3".to_string()));
        assert!(messages.contains(&"event_4".to_string()));
        assert!(messages.contains(&"event_5".to_string()));
        assert!(messages.contains(&"event_6".to_string()));
    }

    #[test]
    fn test_session_log_buffer_o1_insertion_performance() {
        use std::time::Instant;

        let mut buffer = SessionLogBuffer::new(10000);

        let start = Instant::now();
        for i in 0..10000 {
            buffer.push(LogEvent::new(
                0,
                LogLevel::Info,
                "test",
                format!("event_{}", i),
            ));
        }
        let duration_full = start.elapsed();

        buffer.clear();
        let start = Instant::now();
        for i in 0..10000 {
            buffer.push(LogEvent::new(
                0,
                LogLevel::Info,
                "test",
                format!("event_{}", i),
            ));
        }
        let duration_after_clear = start.elapsed();

        assert!(
            duration_full.as_millis() < 100,
            "Push at capacity took too long: {:?}",
            duration_full
        );
        assert!(
            duration_after_clear.as_millis() < 100,
            "Push after clear took too long: {:?}",
            duration_after_clear
        );
    }

    #[test]
    fn test_session_log_buffer_sequence_number_assignment() {
        let mut buffer = SessionLogBuffer::new(10);

        buffer.push(LogEvent::new(0, LogLevel::Info, "test", "first"));
        buffer.push(LogEvent::new(0, LogLevel::Info, "test", "second"));
        buffer.push(LogEvent::new(0, LogLevel::Info, "test", "third"));

        let seqs: Vec<_> = buffer.iter().map(|e| e.seq).collect();
        assert_eq!(seqs, vec![1, 2, 3]);

        for i in 4..=15 {
            buffer.push(LogEvent::new(
                0,
                LogLevel::Info,
                "test",
                format!("event_{}", i),
            ));
        }

        let seqs: Vec<_> = buffer.iter().map(|e| e.seq).collect();
        assert_eq!(seqs, vec![6, 7, 8, 9, 10, 11, 12, 13, 14, 15]);
    }

    #[test]
    fn test_session_log_buffer_preserves_explicit_seq() {
        let mut buffer = SessionLogBuffer::new(5);

        buffer.push(LogEvent::new(100, LogLevel::Info, "test", "event_100"));
        buffer.push(LogEvent::new(101, LogLevel::Info, "test", "event_101"));

        let seqs: Vec<_> = buffer.iter().map(|e| e.seq).collect();
        assert_eq!(seqs, vec![100, 101]);

        buffer.push(LogEvent::new(0, LogLevel::Info, "test", "auto_seq"));

        let last_seq = buffer.iter().last().unwrap().seq;
        assert_eq!(last_seq, 1);
    }

    #[test]
    fn test_reasoning_log_store_new_creates_table_and_index() {
        let temp_dir = tempfile::tempdir().unwrap();
        let db_path = temp_dir.path().join("reasoning.db");
        let _store = ReasoningLogStore::new(&db_path).unwrap();

        assert!(db_path.exists());

        let conn = Connection::open(&db_path).unwrap();
        let table_exists: bool = conn
            .query_row(
                "SELECT COUNT(*) > 0 FROM sqlite_master WHERE type='table' AND name='reasoning_logs'",
                [],
                |row| row.get(0),
            )
            .unwrap();
        assert!(table_exists, "reasoning_logs table should exist");
    }

    #[test]
    fn test_reasoning_log_store_append_and_get() {
        let temp_dir = tempfile::tempdir().unwrap();
        let db_path = temp_dir.path().join("reasoning.db");
        let store = ReasoningLogStore::new(&db_path).unwrap();

        let reasoning = ReasoningLog {
            step_id: "step_001".to_string(),
            session_id: "sess_test".to_string(),
            timestamp: chrono::Utc::now(),
            prompt: "What should I do?".to_string(),
            response: "You should read the file".to_string(),
            tools_considered: vec![
                ToolConsideration {
                    tool_name: "read".to_string(),
                    reason: "Appropriate for reading files".to_string(),
                    selected: true,
                },
                ToolConsideration {
                    tool_name: "grep".to_string(),
                    reason: "Good for searching".to_string(),
                    selected: false,
                },
            ],
            decision: "Using read tool".to_string(),
            prompt_tokens: 100,
            completion_tokens: 50,
            latency_ms: 150,
        };

        store.append(&reasoning).unwrap();

        let retrieved = store.get("step_001").unwrap().unwrap();
        assert_eq!(retrieved.step_id, "step_001");
        assert_eq!(retrieved.session_id, "sess_test");
        assert_eq!(retrieved.prompt, "What should I do?");
        assert_eq!(retrieved.response, "You should read the file");
        assert_eq!(retrieved.tools_considered.len(), 2);
        assert_eq!(retrieved.tools_considered[0].tool_name, "read");
        assert!(retrieved.tools_considered[0].selected);
        assert_eq!(retrieved.tools_considered[1].tool_name, "grep");
        assert!(!retrieved.tools_considered[1].selected);
        assert_eq!(retrieved.decision, "Using read tool");
        assert_eq!(retrieved.prompt_tokens, 100);
        assert_eq!(retrieved.completion_tokens, 50);
        assert_eq!(retrieved.latency_ms, 150);
    }

    #[test]
    fn test_reasoning_log_store_query_by_session() {
        let temp_dir = tempfile::tempdir().unwrap();
        let db_path = temp_dir.path().join("reasoning.db");
        let store = ReasoningLogStore::new(&db_path).unwrap();

        for i in 1..=3 {
            let reasoning = ReasoningLog {
                step_id: format!("step_a_{}", i),
                session_id: "sess_a".to_string(),
                timestamp: chrono::Utc::now(),
                prompt: format!("Prompt {}", i),
                response: format!("Response {}", i),
                tools_considered: vec![],
                decision: format!("Decision {}", i),
                prompt_tokens: 100 * i,
                completion_tokens: 50 * i,
                latency_ms: 10 * i,
            };
            store.append(&reasoning).unwrap();
        }

        for i in 1..=2 {
            let reasoning = ReasoningLog {
                step_id: format!("step_b_{}", i),
                session_id: "sess_b".to_string(),
                timestamp: chrono::Utc::now(),
                prompt: format!("Prompt B {}", i),
                response: format!("Response B {}", i),
                tools_considered: vec![],
                decision: format!("Decision B {}", i),
                prompt_tokens: 200 * i,
                completion_tokens: 75 * i,
                latency_ms: 20 * i,
            };
            store.append(&reasoning).unwrap();
        }

        let results = store.get_by_session("sess_a").unwrap();
        assert_eq!(results.len(), 3);

        let results_b = store.get_by_session("sess_b").unwrap();
        assert_eq!(results_b.len(), 2);
    }

    #[test]
    fn test_reasoning_log_store_query_with_limit() {
        let temp_dir = tempfile::tempdir().unwrap();
        let db_path = temp_dir.path().join("reasoning.db");
        let store = ReasoningLogStore::new(&db_path).unwrap();

        for i in 1..=5 {
            let reasoning = ReasoningLog {
                step_id: format!("step_limit_{}", i),
                session_id: "sess_limit".to_string(),
                timestamp: chrono::Utc::now(),
                prompt: format!("Prompt {}", i),
                response: format!("Response {}", i),
                tools_considered: vec![],
                decision: format!("Decision {}", i),
                prompt_tokens: 100,
                completion_tokens: 50,
                latency_ms: 100,
            };
            store.append(&reasoning).unwrap();
        }

        let results = store
            .query(&ReasoningLogQuery::for_session("sess_limit").with_limit(3))
            .unwrap();
        assert_eq!(results.len(), 3);
    }

    #[test]
    fn test_reasoning_log_store_get_nonexistent() {
        let temp_dir = tempfile::tempdir().unwrap();
        let db_path = temp_dir.path().join("reasoning.db");
        let store = ReasoningLogStore::new(&db_path).unwrap();

        let result = store.get("nonexistent").unwrap();
        assert!(result.is_none());
    }

    #[test]
    fn test_reasoning_log_store_tools_considered_with_selected_flags() {
        let temp_dir = tempfile::tempdir().unwrap();
        let db_path = temp_dir.path().join("reasoning.db");
        let store = ReasoningLogStore::new(&db_path).unwrap();

        let reasoning = ReasoningLog {
            step_id: "step_tools".to_string(),
            session_id: "sess_tools".to_string(),
            timestamp: chrono::Utc::now(),
            prompt: "Which tool to use?".to_string(),
            response: "Using bash".to_string(),
            tools_considered: vec![
                ToolConsideration {
                    tool_name: "read".to_string(),
                    reason: "Good for reading".to_string(),
                    selected: false,
                },
                ToolConsideration {
                    tool_name: "bash".to_string(),
                    reason: "Can execute commands".to_string(),
                    selected: true,
                },
                ToolConsideration {
                    tool_name: "grep".to_string(),
                    reason: "Good for search".to_string(),
                    selected: false,
                },
            ],
            decision: "Selected bash tool".to_string(),
            prompt_tokens: 500,
            completion_tokens: 100,
            latency_ms: 300,
        };

        store.append(&reasoning).unwrap();

        let retrieved = store.get("step_tools").unwrap().unwrap();
        assert_eq!(retrieved.tools_considered.len(), 3);

        let selected_count = retrieved
            .tools_considered
            .iter()
            .filter(|t| t.selected)
            .count();
        assert_eq!(selected_count, 1);
        assert_eq!(
            retrieved
                .tools_considered
                .iter()
                .find(|t| t.selected)
                .unwrap()
                .tool_name,
            "bash"
        );

        let unselected_count = retrieved
            .tools_considered
            .iter()
            .filter(|t| !t.selected)
            .count();
        assert_eq!(unselected_count, 2);
    }

    #[test]
    fn test_reasoning_log_store_token_counts_and_latency_preserved() {
        let temp_dir = tempfile::tempdir().unwrap();
        let db_path = temp_dir.path().join("reasoning.db");
        let store = ReasoningLogStore::new(&db_path).unwrap();

        let reasoning = ReasoningLog {
            step_id: "step_tokens".to_string(),
            session_id: "sess_tokens".to_string(),
            timestamp: chrono::Utc::now(),
            prompt: "Test prompt".to_string(),
            response: "Test response".to_string(),
            tools_considered: vec![],
            decision: "Decision".to_string(),
            prompt_tokens: 15000,
            completion_tokens: 7500,
            latency_ms: 2500,
        };

        store.append(&reasoning).unwrap();

        let retrieved = store.get("step_tokens").unwrap().unwrap();
        assert_eq!(retrieved.prompt_tokens, 15000);
        assert_eq!(retrieved.completion_tokens, 7500);
        assert_eq!(retrieved.latency_ms, 2500);
    }
}
