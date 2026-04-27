//! Core tool types.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolParameter {
    pub name: String,
    pub description: String,
    pub required: bool,
    #[serde(default)]
    pub schema: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ToolDefinition {
    pub name: String,
    pub description: String,
    #[serde(default)]
    pub parameters: Vec<ToolParameter>,
    #[serde(default)]
    pub requires_approval: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolResult {
    pub id: Uuid,
    pub tool_name: String,
    pub success: bool,
    pub result: Option<String>,
    pub error: Option<String>,
    pub started_at: DateTime<Utc>,
    pub completed_at: DateTime<Utc>,
}

impl ToolResult {
    pub fn success(tool_name: String, result: String) -> Self {
        Self {
            id: Uuid::new_v4(),
            tool_name,
            success: true,
            result: Some(result),
            error: None,
            started_at: Utc::now(),
            completed_at: Utc::now(),
        }
    }

    pub fn failure(tool_name: String, error: String) -> Self {
        Self {
            id: Uuid::new_v4(),
            tool_name,
            success: false,
            result: None,
            error: Some(error),
            started_at: Utc::now(),
            completed_at: Utc::now(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolCall {
    pub id: String,
    pub name: String,
    pub arguments: serde_json::Value,
}

pub type ToolExecutor = Arc<dyn Fn(serde_json::Value) -> Result<String, String> + Send + Sync>;
