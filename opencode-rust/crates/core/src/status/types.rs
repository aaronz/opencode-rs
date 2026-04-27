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