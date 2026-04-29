use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum RuntimeStatus {
    Idle,
    Busy,
    Degraded,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RuntimeResponse {
    pub session_id: Option<String>,
    pub turn_id: Option<String>,
    pub accepted: bool,
    pub message: String,
}
