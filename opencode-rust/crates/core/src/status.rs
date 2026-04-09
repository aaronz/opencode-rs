use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionStatus {
    pub session_id: String,
    pub status: Status,
    pub started_at: DateTime<Utc>,
    pub last_activity: DateTime<Utc>,
    pub message_count: usize,
    pub tool_calls: usize,
    pub errors: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Status {
    Idle,
    Processing,
    WaitingForInput,
    Error(String),
    Completed,
}

impl SessionStatus {
    pub fn new(session_id: String) -> Self {
        let now = Utc::now();
        Self {
            session_id,
            status: Status::Idle,
            started_at: now,
            last_activity: now,
            message_count: 0,
            tool_calls: 0,
            errors: 0,
        }
    }

    pub fn update(&mut self, status: Status) {
        self.status = status;
        self.last_activity = Utc::now();
    }

    pub fn increment_messages(&mut self) {
        self.message_count += 1;
        self.last_activity = Utc::now();
    }

    pub fn increment_tool_calls(&mut self) {
        self.tool_calls += 1;
        self.last_activity = Utc::now();
    }

    pub fn increment_errors(&mut self) {
        self.errors += 1;
        self.last_activity = Utc::now();
    }

    pub fn is_processing(&self) -> bool {
        matches!(self.status, Status::Processing)
    }

    pub fn is_idle(&self) -> bool {
        matches!(self.status, Status::Idle)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_session_status_new() {
        let status = SessionStatus::new("test-session".to_string());
        assert_eq!(status.session_id, "test-session");
        assert!(matches!(status.status, Status::Idle));
        assert_eq!(status.message_count, 0);
        assert_eq!(status.tool_calls, 0);
        assert_eq!(status.errors, 0);
    }

    #[test]
    fn test_session_status_update() {
        let mut status = SessionStatus::new("test".to_string());
        status.update(Status::Processing);
        assert!(status.is_processing());
    }

    #[test]
    fn test_session_status_increment_messages() {
        let mut status = SessionStatus::new("test".to_string());
        status.increment_messages();
        assert_eq!(status.message_count, 1);
    }

    #[test]
    fn test_session_status_increment_tool_calls() {
        let mut status = SessionStatus::new("test".to_string());
        status.increment_tool_calls();
        assert_eq!(status.tool_calls, 1);
    }

    #[test]
    fn test_session_status_increment_errors() {
        let mut status = SessionStatus::new("test".to_string());
        status.increment_errors();
        assert_eq!(status.errors, 1);
    }

    #[test]
    fn test_session_status_is_idle() {
        let status = SessionStatus::new("test".to_string());
        assert!(status.is_idle());
    }

    #[test]
    fn test_session_status_is_processing() {
        let mut status = SessionStatus::new("test".to_string());
        status.update(Status::Processing);
        assert!(status.is_processing());
    }
}
