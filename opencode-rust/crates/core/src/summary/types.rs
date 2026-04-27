use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Debug, Error, Clone, PartialEq, Eq)]
pub enum SummaryError {
    #[error("cannot summarize an empty session")]
    EmptySession,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionSummary {
    pub session_id: String,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub message_count: usize,
    pub user_messages: usize,
    pub assistant_messages: usize,
    pub tools_used: Vec<String>,
    pub topics: Vec<String>,
    pub key_decisions: Vec<String>,
}
