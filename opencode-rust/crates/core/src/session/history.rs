//! Undo/redo history management.

use crate::message::Message;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

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
