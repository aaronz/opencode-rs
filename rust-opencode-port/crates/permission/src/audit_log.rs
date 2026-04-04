use chrono::{DateTime, Duration, Utc};
use rusqlite::{params, Connection};
use serde::{Deserialize, Serialize};
use std::path::Path;
use std::sync::{Arc, Mutex};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum AuditDecision {
    Allow,
    Deny,
    Ask,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditEntry {
    pub timestamp: DateTime<Utc>,
    pub tool_name: String,
    pub decision: AuditDecision,
    pub session_id: String,
    pub user_response: Option<String>,
}

#[derive(Clone)]
pub struct AuditLog {
    conn: Arc<Mutex<Connection>>,
}

impl AuditLog {
    pub fn new(path: impl AsRef<Path>) -> Result<Self, rusqlite::Error> {
        let conn = Connection::open(path)?;
        let log = Self {
            conn: Arc::new(Mutex::new(conn)),
        };
        log.init_schema()?;
        let _ = log.cleanup_older_than(30);
        Ok(log)
    }

    fn init_schema(&self) -> Result<(), rusqlite::Error> {
        let conn = self.conn.lock().expect("audit log mutex poisoned");
        conn.execute_batch(
            "
            CREATE TABLE IF NOT EXISTS permission_audit_log (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                timestamp TEXT NOT NULL,
                tool_name TEXT NOT NULL,
                decision TEXT NOT NULL,
                session_id TEXT NOT NULL,
                user_response TEXT
            );

            CREATE INDEX IF NOT EXISTS idx_permission_audit_timestamp
                ON permission_audit_log (timestamp);
            CREATE INDEX IF NOT EXISTS idx_permission_audit_tool_name
                ON permission_audit_log (tool_name);
            ",
        )
    }

    pub fn record_decision(&self, entry: AuditEntry) -> Result<(), rusqlite::Error> {
        let conn = self.conn.lock().expect("audit log mutex poisoned");
        conn.execute(
            "INSERT INTO permission_audit_log (timestamp, tool_name, decision, session_id, user_response)
             VALUES (?1, ?2, ?3, ?4, ?5)",
            params![
                entry.timestamp.to_rfc3339(),
                entry.tool_name,
                decision_to_str(&entry.decision),
                entry.session_id,
                entry.user_response
            ],
        )?;
        Ok(())
    }

    pub fn query_by_time_range(
        &self,
        start: DateTime<Utc>,
        end: DateTime<Utc>,
    ) -> Result<Vec<AuditEntry>, rusqlite::Error> {
        let conn = self.conn.lock().expect("audit log mutex poisoned");
        let mut stmt = conn.prepare(
            "SELECT timestamp, tool_name, decision, session_id, user_response
             FROM permission_audit_log
             WHERE timestamp >= ?1 AND timestamp <= ?2
             ORDER BY timestamp ASC",
        )?;

        let rows = stmt.query_map(params![start.to_rfc3339(), end.to_rfc3339()], map_row)?;
        rows.collect()
    }

    pub fn query_by_tool_name(&self, tool: &str) -> Result<Vec<AuditEntry>, rusqlite::Error> {
        let conn = self.conn.lock().expect("audit log mutex poisoned");
        let mut stmt = conn.prepare(
            "SELECT timestamp, tool_name, decision, session_id, user_response
             FROM permission_audit_log
             WHERE tool_name = ?1
             ORDER BY timestamp DESC",
        )?;

        let rows = stmt.query_map(params![tool], map_row)?;
        rows.collect()
    }

    pub fn get_recent_entries(&self, limit: usize) -> Result<Vec<AuditEntry>, rusqlite::Error> {
        let conn = self.conn.lock().expect("audit log mutex poisoned");
        let mut stmt = conn.prepare(
            "SELECT timestamp, tool_name, decision, session_id, user_response
             FROM permission_audit_log
             ORDER BY timestamp DESC
             LIMIT ?1",
        )?;

        let rows = stmt.query_map(params![limit as i64], map_row)?;
        rows.collect()
    }

    pub fn cleanup_older_than(&self, days: u32) -> Result<usize, rusqlite::Error> {
        let cutoff = Utc::now() - Duration::days(days as i64);
        let conn = self.conn.lock().expect("audit log mutex poisoned");
        let count = conn.execute(
            "DELETE FROM permission_audit_log WHERE timestamp < ?1",
            params![cutoff.to_rfc3339()],
        )?;
        Ok(count)
    }
}

fn decision_to_str(decision: &AuditDecision) -> &'static str {
    match decision {
        AuditDecision::Allow => "allow",
        AuditDecision::Deny => "deny",
        AuditDecision::Ask => "ask",
    }
}

fn str_to_decision(value: &str) -> AuditDecision {
    match value {
        "allow" => AuditDecision::Allow,
        "deny" => AuditDecision::Deny,
        _ => AuditDecision::Ask,
    }
}

fn map_row(row: &rusqlite::Row<'_>) -> Result<AuditEntry, rusqlite::Error> {
    let timestamp_raw: String = row.get(0)?;
    let decision_raw: String = row.get(2)?;
    let timestamp = chrono::DateTime::parse_from_rfc3339(&timestamp_raw)
        .map(|dt| dt.with_timezone(&Utc))
        .unwrap_or_else(|_| Utc::now());

    Ok(AuditEntry {
        timestamp,
        tool_name: row.get(1)?,
        decision: str_to_decision(&decision_raw),
        session_id: row.get(3)?,
        user_response: row.get(4)?,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn records_queries_and_cleans_up() {
        let tmp = tempfile::tempdir().unwrap();
        let log = AuditLog::new(tmp.path().join("audit.db")).unwrap();

        let now = Utc::now();
        log.record_decision(AuditEntry {
            timestamp: now,
            tool_name: "bash".to_string(),
            decision: AuditDecision::Ask,
            session_id: "s1".to_string(),
            user_response: Some("approved".to_string()),
        })
        .unwrap();

        let by_tool = log.query_by_tool_name("bash").unwrap();
        assert_eq!(by_tool.len(), 1);
        assert_eq!(by_tool[0].session_id, "s1");

        let by_time = log
            .query_by_time_range(now - Duration::minutes(1), now + Duration::minutes(1))
            .unwrap();
        assert_eq!(by_time.len(), 1);

        let recent = log.get_recent_entries(10).unwrap();
        assert!(!recent.is_empty());

        let deleted = log.cleanup_older_than(0).unwrap();
        assert!(deleted >= 1);
    }
}
