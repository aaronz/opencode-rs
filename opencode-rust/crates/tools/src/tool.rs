use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use opencode_core::OpenCodeError;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolResult {
    pub success: bool,
    pub content: String,
    pub error: Option<String>,
    pub title: Option<String>,
    pub metadata: Option<serde_json::Value>,
}

impl ToolResult {
    pub fn ok(content: impl Into<String>) -> Self {
        Self {
            success: true,
            content: content.into(),
            error: None,
            title: None,
            metadata: None,
        }
    }

    pub fn err(error: impl Into<String>) -> Self {
        Self {
            success: false,
            content: String::new(),
            error: Some(error.into()),
            title: None,
            metadata: None,
        }
    }

    pub fn with_title(mut self, title: impl Into<String>) -> Self {
        self.title = Some(title.into());
        self
    }

    pub fn with_metadata(mut self, metadata: serde_json::Value) -> Self {
        self.metadata = Some(metadata);
        self
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolContext {
    pub session_id: String,
    pub message_id: String,
    pub agent: String,
    pub worktree: Option<String>,
    pub directory: Option<String>,
}

#[async_trait]
pub trait Tool: Send + Sync {
    fn name(&self) -> &str;
    fn description(&self) -> &str;
    fn clone_tool(&self) -> Box<dyn Tool>;
    async fn execute(&self, args: serde_json::Value, ctx: Option<ToolContext>) -> Result<ToolResult, OpenCodeError>;
}
