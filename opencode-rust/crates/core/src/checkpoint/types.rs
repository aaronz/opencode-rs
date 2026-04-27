use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Checkpoint {
    pub id: Uuid,
    pub session_id: Uuid,
    pub sequence_number: usize,
    pub created_at: DateTime<Utc>,
    pub description: String,
    pub checkpoint_path: PathBuf,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CheckpointMetadata {
    pub id: Uuid,
    pub session_id: Uuid,
    pub sequence_number: usize,
    pub created_at: DateTime<Utc>,
    pub description: String,
}
