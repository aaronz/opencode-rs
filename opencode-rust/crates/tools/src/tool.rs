use async_trait::async_trait;
use opencode_core::OpenCodeError;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::path::PathBuf;

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

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ToolContext {
    pub session_id: String,
    pub message_id: String,
    pub agent: String,
    pub worktree: Option<String>,
    pub directory: Option<String>,
    pub permission_scope: Option<opencode_permission::AgentPermissionScope>,
}

impl ToolContext {
    pub fn with_permission_scope(
        mut self,
        scope: opencode_permission::AgentPermissionScope,
    ) -> Self {
        self.permission_scope = Some(scope);
        self
    }
}

#[async_trait]
pub trait Tool: Send + Sync {
    fn name(&self) -> &str;
    fn description(&self) -> &str;
    fn clone_tool(&self) -> Box<dyn Tool>;
    async fn execute(
        &self,
        args: serde_json::Value,
        ctx: Option<ToolContext>,
    ) -> Result<ToolResult, OpenCodeError>;

    fn is_safe(&self) -> bool {
        false
    }

    fn get_dependencies(&self, _args: &serde_json::Value) -> HashSet<PathBuf> {
        HashSet::new()
    }
}
