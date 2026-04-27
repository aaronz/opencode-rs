//! Tool invocation records.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolInvocationRecord {
    pub id: Uuid,
    pub tool_name: String,
    pub arguments: serde_json::Value,
    pub args_hash: String,
    pub result: Option<String>,
    pub started_at: DateTime<Utc>,
    pub completed_at: Option<DateTime<Utc>>,
    pub latency_ms: Option<u64>,
}
