//! Session log buffer (ring buffer) and persistent log store.

use std::collections::VecDeque;

use chrono::{DateTime, Utc};
use rusqlite::{params, Connection};

use crate::event::{LogEvent, LogLevel, LogFields};
use crate::query::LogQuery;
use crate::error::LogError;

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
        self.events
            .iter()
            .filter(|e| e.level == level)
            .collect()
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
                parent_seq INTEGER
            );
            CREATE INDEX IF NOT EXISTS idx_logs_session ON logs(session_id);
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
            "INSERT INTO logs (seq, timestamp, level, target, message, fields, span_id, parent_seq)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
            params![
                event.seq,
                event.timestamp.to_rfc3339(),
                serde_json::to_string(&event.level).unwrap_or_default(),
                event.target,
                event.message,
                fields_json,
                event.span_id,
                event.parent_seq,
            ],
        )?;

        Ok(())
    }

    pub fn query(&self, criteria: &LogQuery) -> Result<Vec<LogEvent>, LogError> {
        let mut sql = String::from("SELECT seq, timestamp, level, target, message, fields, span_id, parent_seq FROM logs WHERE 1=1");
        let mut params_vec: Vec<Box<dyn rusqlite::ToSql>> = Vec::new();

        if let Some(ref session_id) = criteria.session_id {
            sql.push_str(" AND json_extract(fields, '$.session_id') = ?");
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

        let params_refs: Vec<&dyn rusqlite::ToSql> = params_vec.iter().map(|p| p.as_ref()).collect();

        let mut stmt = self.conn.prepare(&sql)?;
        let rows = stmt.query_map(params_refs.as_slice(), |row| {
            let timestamp_str: String = row.get(1)?;
            let level_str: String = row.get(2)?;
            let fields_str: String = row.get(5)?;

            let timestamp = DateTime::parse_from_rfc3339(&timestamp_str)
                .map(|dt| dt.with_timezone(&Utc))
                .unwrap_or_else(|_| Utc::now());
            let level: LogLevel = serde_json::from_str(&level_str).unwrap_or(LogLevel::Info);
            let fields: LogFields = serde_json::from_str(&fields_str).unwrap_or_default();

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
        for row in rows {
            if let Ok(event) = row {
                events.push(event);
            }
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_session_log_buffer_push() {
        let mut buffer = SessionLogBuffer::new(3);

        buffer.push(LogEvent::new(0, LogLevel::Info, "test", "first"));
        buffer.push(LogEvent::new(0, LogLevel::Info, "test", "second"));
        buffer.push(LogEvent::new(0, LogLevel::Info, "test", "third"));

        assert_eq!(buffer.len(), 3);
        assert_eq!(buffer.iter().nth(0).unwrap().message, "first");
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
            buffer.push(LogEvent::new(0, LogLevel::Info, "test", format!("event_{}", i)));
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
}