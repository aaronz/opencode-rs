//! Fork-related types and errors.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ForkError {
    MessageIndexOutOfBounds { requested: usize, len: usize },
}

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
pub struct ForkEntry {
    pub forked_at: DateTime<Utc>,
    pub child_session_id: Uuid,
}
