use async_trait::async_trait;
use std::process::Command;
use crate::{Tool, ToolResult};
use opencode_core::OpenCodeError;

pub struct GitStatusTool;

#[async_trait]
impl Tool for GitStatusTool {
    fn name(&self) -> &str {
        "git_status"
    }

    fn description(&self) -> &str {
        "Show git repository status"
    }

    fn clone_tool(&self) -> Box<dyn Tool> {
        Box::new(GitStatusTool)
    }

    async fn execute(&self, _args: serde_json::Value, _ctx: Option<crate::ToolContext>) -> Result<ToolResult, OpenCodeError> {
        let output = Command::new("git")
            .args(["status", "--porcelain"])
            .output()
            .map_err(|e| OpenCodeError::Tool(e.to_string()))?;

        let result = String::from_utf8_lossy(&output.stdout).to_string();
        
        if result.is_empty() {
            return Ok(ToolResult::ok("Working tree clean".to_string()));
        }

        Ok(ToolResult::ok(result))
    }
}

pub struct GitDiffTool;

#[async_trait]
impl Tool for GitDiffTool {
    fn name(&self) -> &str {
        "git_diff"
    }

    fn description(&self) -> &str {
        "Show uncommitted changes"
    }

    fn clone_tool(&self) -> Box<dyn Tool> {
        Box::new(GitDiffTool)
    }

    async fn execute(&self, _args: serde_json::Value, _ctx: Option<crate::ToolContext>) -> Result<ToolResult, OpenCodeError> {
        let output = Command::new("git")
            .args(["diff", "--color=never"])
            .output()
            .map_err(|e| OpenCodeError::Tool(e.to_string()))?;

        let result = String::from_utf8_lossy(&output.stdout).to_string();
        
        if result.is_empty() {
            return Ok(ToolResult::ok("No changes".to_string()));
        }

        Ok(ToolResult::ok(result))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_git_status_tool() {
        let tool = GitStatusTool;
        let result = tool.execute(serde_json::json!({}), None).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_git_diff_tool() {
        let tool = GitDiffTool;
        let result = tool.execute(serde_json::json!({}), None).await;
        assert!(result.is_ok());
    }
}
