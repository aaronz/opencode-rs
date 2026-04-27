//! Undo/redo history management.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use crate::message::Message;

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
